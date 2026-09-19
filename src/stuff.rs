use anyhow::{Result, bail};
#[allow(unused, dead_code)]
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::dtos::{
    Category, CategoryId, ErrProp, ErrorEntry, ErrorId, IdGenerator, ProjectConfig, ProjectErrors,
    Severity, UuidVersion,
};

use rand::Rng;
use rand::{
    RngExt,
    distr::{Alphanumeric, SampleString},
};

impl IdGenerator {
    pub fn generate_id(&self, last: &str) -> Result<String> {
        match self {
            IdGenerator::Sequential { digits } => Self::sequential(last, *digits),
            IdGenerator::SequentialHex { digits } => Self::sequential_hex(last, *digits),
            IdGenerator::RandomHex { length } => Self::random_hex(*length),
            IdGenerator::RandomHash { length } => Self::random_hash(*length),
            IdGenerator::Custom { regex } => Self::custom(regex),
            IdGenerator::Uuid { version } => Self::uuid(version),
        }
    }

    fn uuid(version: &UuidVersion) -> Result<String> {
        match version {
            UuidVersion::V4 => Ok(Uuid::new_v4().to_string()),
        }
    }

    fn sequential(last: &str, digits: u8) -> Result<String> {
        let v: u32 = last.parse()?;
        Ok(format!("{:0width$}", v + 1, width = digits as usize))
    }

    fn sequential_hex(last: &str, digits: u8) -> Result<String> {
        let v = u32::from_str_radix(last, 16)?;
        Ok(format!("{:0width$x}", v + 1, width = digits as usize))
    }

    fn random_hex(length: u8) -> Result<String> {
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..length.div_ceil(2)).map(|_| rng.random()).collect();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        Ok(hex[..length as usize].to_string())
    }

    fn random_hash(length: u8) -> Result<String> {
        let mut rng = rand::rng();
        Ok(Alphanumeric.sample_string(&mut rng, length as usize))
    }

    fn custom(_regex: &str) -> Result<String> {
        anyhow::bail!("custom id generation not implemented yet")
    }
}

impl ProjectConfig {
    pub const FOLDER_NAME: &str = ".newerr";
    pub const TEMPLATE_FOLDER_NAME: &str = "templates";
    pub const CONFIG_FILE_NAME: &str = "config.toml";
    pub const ERRORS_FILE_NAME: &str = "errors.toml";

    pub fn init() -> Result<()> {
        let mut working_folder = env::current_dir()?;
        working_folder.push(Self::FOLDER_NAME);

        if working_folder.join(Self::CONFIG_FILE_NAME).exists()
            || working_folder.join(Self::ERRORS_FILE_NAME).exists()
        {
            bail!(
                "{} already initialized in this directory",
                Self::FOLDER_NAME
            );
        }

        let template_folder = working_folder.join(Self::TEMPLATE_FOLDER_NAME);
        if !template_folder.exists() {
            Self::create_folder(&template_folder);
        }

        Self::create_folder(&working_folder)?;
        let mut config = Self::create_toml(&working_folder, Self::CONFIG_FILE_NAME)?;
        let mut errors = Self::create_toml(&working_folder, Self::ERRORS_FILE_NAME)?;

        let defconfig = ProjectConfig {
            generated_errors_file_path: "unknown".to_string(),
            generated_docs_file_path: Some("ERRORS.md".to_string()),
            id_generator: IdGenerator::SequentialHex { digits: 4 },
            package: None,
        };

        let mut categories = HashMap::new();
        categories.insert(
            "0".to_string(),
            Category {
                name: "general".to_string(),
                description: "Default category.".to_string(),
            },
        );

        let deferrors = ProjectErrors {
            banned_ids: vec![],
            categories,
            errors: vec![],
            last_id: None,
        };

        config.write_all(toml::to_string_pretty(&defconfig)?.as_bytes())?;
        errors.write_all(toml::to_string_pretty(&deferrors)?.as_bytes())?;

        Ok(())
    }

    pub fn create_folder(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    pub fn create_toml(path: &Path, name: &str) -> Result<File> {
        if !path.exists() {
            bail!("folder {:?} does not exist", path);
        }
        let p = path.join(name);
        Ok(File::create(p)?)
    }
}

//

impl ProjectErrors {
    pub fn new_error(
        &mut self,
        config: &ProjectConfig,
        name: String,
        category_id: CategoryId,
        severity: Severity,
        message: String,
        hint: String,
    ) -> Result<()> {
        let next_id = config
            .id_generator
            .generate_id(self.last_id.as_deref().unwrap_or("0"))?;

        let resolved = if self.categories.contains_key(&category_id) {
            category_id
        } else {
            self.categories
                .iter()
                .find(|(_, cat)| cat.name == category_id)
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| "0".to_string())
        };

        self.errors.push(ErrorEntry {
            id: next_id.clone(),
            name,
            category_id: resolved,
            severity,
            message,
            hint,
        });
        self.last_id = Some(next_id);
        Ok(())
    }

    pub fn new_cathegory(&mut self, name: String, description: String) -> CategoryId {
        let next_id = self
            .categories
            .keys()
            .filter_map(|k| k.parse::<u32>().ok())
            .max()
            .map(|n| (n + 1).to_string())
            .unwrap_or_else(|| "0".to_string());

        self.categories
            .insert(next_id.clone(), Category { name, description });
        next_id
    }

    pub fn modify_error(&mut self, id: &ErrorId, prop: ErrProp, value: String) {
        if let Some(entry) = self.errors.iter_mut().find(|e| e.id == *id) {
            match prop {
                ErrProp::Name => entry.name = value,
                ErrProp::Message => entry.message = value,
                ErrProp::Hint => entry.hint = value,
                ErrProp::Category => entry.category_id = value,
                ErrProp::Severity => {
                    entry.severity = match value.as_str() {
                        "info" => Severity::Info,
                        "warning" => Severity::Warning,
                        "error" => Severity::Error,
                        "critical" => Severity::Critical,
                        _ => return,
                    }
                }
            }
        }
    }

    pub fn is_banned(&self, id: &ErrorId) -> bool {
        self.banned_ids.contains(id)
    }

    pub fn get_last(&self) -> Option<ErrorId> {
        self.last_id.clone()
    }
}

//
