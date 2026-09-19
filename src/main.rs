use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::dtos::{ErrProp, ProjectConfig, ProjectErrors, Severity, Template};

mod dtos;
mod stuff;

use include_dir::{Dir, include_dir};

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

fn builtin(name: &str) -> Option<&'static str> {
    TEMPLATES.get_file(format!("{name}.toml"))?.contents_utf8()
}

#[derive(Parser)]
#[command(
    name = "newerr",
    version,
    about = "cli tool for software error management and automation."
)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Init the project config folder
    Init,
    /// Generates the error file for targeted frontend
    /// - markdown: Documentation
    /// - odin: Language implementation
    Gen {
        #[arg(short = 'F', long, default_value = "odin")]
        frontend: String,
        #[arg(short, long)]
        package: Option<String>,
    },

    /// Create a new error
    Err {
        name: String,
        #[arg(short, long)]
        message: Option<String>,
        #[arg(short = 'H', long)]
        hint: Option<String>,
        #[arg(short, long)]
        category: String,
        #[arg(short, long, value_enum)]
        severity: Severity,
    },
    /// Create a new category
    Cat { name: String },
    /// Modify an error property
    Modify {
        id: String,
        property: String,
        value: String,
    },
    /// Generates the autocompletion script
    Completions { shell: Shell },
}

fn errors_path() -> anyhow::Result<std::path::PathBuf> {
    let mut path = env::current_dir()?;
    path.push(ProjectConfig::FOLDER_NAME);
    path.push(ProjectConfig::ERRORS_FILE_NAME);
    Ok(path)
}

fn config_path() -> anyhow::Result<std::path::PathBuf> {
    let mut path = env::current_dir()?;
    path.push(ProjectConfig::FOLDER_NAME);
    path.push(ProjectConfig::CONFIG_FILE_NAME);
    Ok(path)
}

fn load_errors() -> anyhow::Result<ProjectErrors> {
    let content = fs::read_to_string(errors_path()?)?;
    Ok(toml::from_str(&content)?)
}

fn load_config() -> anyhow::Result<ProjectConfig> {
    let content = fs::read_to_string(config_path()?)?;
    Ok(toml::from_str(&content)?)
}

fn save_errors(errors: &ProjectErrors) -> anyhow::Result<()> {
    let content = toml::to_string_pretty(errors)?;
    fs::write(errors_path()?, content)?;
    Ok(())
}

fn parse_err_prop(s: &str) -> anyhow::Result<ErrProp> {
    match s {
        "name" => Ok(ErrProp::Name),
        "message" => Ok(ErrProp::Message),
        "hint" => Ok(ErrProp::Hint),
        "category" | "cathegory" => Ok(ErrProp::Category),
        "severity" => Ok(ErrProp::Severity),
        _ => anyhow::bail!("invalid property: {s}"),
    }
}

fn load_template(project: &Path, name: &str) -> Result<Template, Box<dyn std::error::Error>> {
    let custom = project
        .join(".newerr/templates")
        .join(format!("{name}.toml"));

    let text = if custom.exists() {
        std::fs::read_to_string(&custom)?
    } else {
        builtin(name)
            .ok_or_else(|| format!("unknown frontend: {name}"))?
            .to_string()
    };

    Ok(toml::from_str(&text)?)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            ProjectConfig::init()?;
            Ok(())
        }

        Commands::Err {
            name,
            message,
            hint,
            category: cathegory,
            severity,
        } => {
            let config = load_config()?;
            let mut errors = load_errors()?;
            errors.new_error(
                &config,
                name,
                cathegory,
                severity,
                message.unwrap_or("".to_string()),
                hint.unwrap_or("".to_string()),
            )?;
            save_errors(&errors)
        }

        Commands::Gen { frontend, package } => {
            let config = load_config()?;
            let errors = load_errors()?;

            let package = package
                .as_deref()
                .or(config.package.as_deref())
                .unwrap_or("errors");

            let t = load_template(&env::current_dir()?, &frontend)
                .map_err(|e| anyhow::anyhow!("failed to load template '{frontend}': {e}"))?;

            let output = Path::new(&config.generated_errors_file_path);
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = File::create(output)?;
            let content = t.write_down(package, &errors);
            file.write_all(content.as_bytes())?;

            if cli.verbose {
                println!(
                    "Generated {} error(s) into '{}'",
                    errors.errors.len(),
                    output.display()
                );
            }

            Ok(())
        }

        Commands::Cat { name } => {
            let mut errors = load_errors()?;
            errors.new_cathegory(name, String::new());
            save_errors(&errors)
        }

        Commands::Modify {
            id,
            property,
            value,
        } => {
            let mut errors = load_errors()?;
            let prop = parse_err_prop(&property)?;
            errors.modify_error(&id, prop, value);
            save_errors(&errors)
        }

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
    }
}
