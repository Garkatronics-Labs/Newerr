use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ErrorId = String;
pub type CategoryId = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorEntry {
    pub id: ErrorId,
    pub name: String,
    pub category_id: CategoryId,
    pub severity: Severity,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub hint: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// .newerr/config.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub generated_errors_file_path: String,
    /// Ruta donde `newerr doc` escribe la documentación markdown por defecto.
    #[serde(default)]
    pub generated_docs_file_path: Option<String>,
    pub id_generator: IdGenerator,
    #[serde(default)]
    pub package: Option<String>,
}

/// .newerr/errors.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectErrors {
    #[serde(default)]
    pub categories: HashMap<CategoryId, Category>,
    #[serde(default)]
    pub banned_ids: Vec<ErrorId>,
    #[serde(default)]
    pub errors: Vec<ErrorEntry>,
    pub last_id: Option<ErrorId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IdGenerator {
    /// 0001, 0002, 0003... contador decimal incremental
    Sequential { digits: u8 },

    /// 0x1a, 0x1b... contador hex incremental
    SequentialHex { digits: u8 },

    /// string hex aleatorio, longitud fija
    RandomHex { length: u8 },

    /// hash aleatorio (no incremental por naturaleza)
    RandomHash { length: u8 },

    /// uuid — longitud y alfabeto ya definidos por spec
    Uuid { version: UuidVersion },

    /// usuario define regex — longitud implícita en el patrón
    Custom { regex: String },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UuidVersion {
    V4,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrProp {
    Name,
    Message,
    Hint,
    Category,
    Severity,
}

impl ErrProp {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Message => "message",
            Self::Hint => "hint",
            Self::Category => "category",
            Self::Severity => "severity",
        }
    }
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    #[serde(default)]
    pub header: String,
    #[serde(default)]
    pub footer: String,
    #[serde(default, rename = "section")]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
    /// Se añade tras cada pieza emitida.
    #[serde(default = "default_end")]
    pub end: String,
    /// Se añade tras las piezas de categoría (antes de sus errores).
    #[serde(default)]
    pub category_end: String,
    #[serde(default)]
    pub category: Vec<Piece>,
    #[serde(default)]
    pub error: Vec<Piece>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub text: String,
    #[serde(default)]
    pub skip_if_empty: Option<Placeholder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Placeholder {
    Id,
    Name,
    Message,
    Hint,
    Severity,
    Category,
    CategoryDescription,
    /// First error id of the current category (empty if it has no errors).
    CategoryIdStart,
    /// Last error id of the current category.
    CategoryIdEnd,
    Package,
}

fn default_end() -> String {
    "\n".to_string()
}

impl Template {
    pub fn write_down(&self, package: &str, errors: &ProjectErrors) -> String {
        let mut r = String::new();

        r.push_str(&Self::replace_in_text(
            &self.header,
            package,
            None,
            &[],
            None,
        ));

        for sec in &self.sections {
            r.push_str(&Self::replace_in_text(
                &sec.prefix,
                package,
                None,
                &[],
                None,
            ));

            let grouped = Self::group_by_category(errors);

            for (cat_id, errs) in grouped {
                let cat = errors.categories.get(&cat_id);

                for piece in &sec.category {
                    if Self::should_skip(piece, cat, &errs, None) {
                        continue;
                    }
                    r.push_str(&Self::replace_in_text(
                        &piece.text,
                        package,
                        cat,
                        &errs,
                        None,
                    ));
                }
                r.push_str(&Self::replace_in_text(
                    &sec.category_end,
                    package,
                    cat,
                    &errs,
                    None,
                ));

                for (i, err) in errs.iter().enumerate() {
                    for piece in &sec.error {
                        if Self::should_skip(piece, cat, &errs, Some(err)) {
                            continue;
                        }
                        r.push_str(&Self::replace_in_text(
                            &piece.text,
                            package,
                            cat,
                            &errs,
                            Some(err),
                        ));
                    }
                    if i + 1 < errs.len() {
                        r.push_str(&Self::replace_in_text(
                            &sec.end,
                            package,
                            cat,
                            &errs,
                            None,
                        ));
                    }
                }
            }

            r.push_str(&Self::replace_in_text(
                &sec.suffix,
                package,
                None,
                &[],
                None,
            ));
        }

        r.push_str(&Self::replace_in_text(
            &self.footer,
            package,
            None,
            &[],
            None,
        ));

        r
    }

    fn group_by_category(errors: &ProjectErrors) -> Vec<(CategoryId, Vec<&ErrorEntry>)> {
        let mut result: Vec<(CategoryId, Vec<&ErrorEntry>)> = Vec::new();
        let mut index_by_id: HashMap<CategoryId, usize> = HashMap::new();

        for err in &errors.errors {
            if let Some(&idx) = index_by_id.get(&err.category_id) {
                result[idx].1.push(err);
            } else {
                index_by_id.insert(err.category_id.clone(), result.len());
                result.push((err.category_id.clone(), vec![err]));
            }
        }

        result
    }

    fn should_skip(
        piece: &Piece,
        cat: Option<&Category>,
        errs: &[&ErrorEntry],
        err: Option<&ErrorEntry>,
    ) -> bool {
        let Some(placeholder) = piece.skip_if_empty else {
            return false;
        };
        if placeholder == Placeholder::Package {
            return false;
        }
        Self::placeholder_value(placeholder, cat, errs, err)
            .trim()
            .is_empty()
    }

    fn placeholder_value(
        ph: Placeholder,
        cat: Option<&Category>,
        errs: &[&ErrorEntry],
        err: Option<&ErrorEntry>,
    ) -> String {
        match ph {
            Placeholder::Id => err.map(|e| e.id.clone()).unwrap_or_default(),
            Placeholder::Name => err.map(|e| e.name.clone()).unwrap_or_default(),
            Placeholder::Message => err.map(|e| e.message.clone()).unwrap_or_default(),
            Placeholder::Hint => err.map(|e| e.hint.clone()).unwrap_or_default(),
            Placeholder::Severity => err
                .map(|e| e.severity.as_str().to_string())
                .unwrap_or_default(),
            Placeholder::Category => cat.map(|c| c.name.clone()).unwrap_or_default(),
            Placeholder::CategoryDescription => {
                cat.map(|c| c.description.clone()).unwrap_or_default()
            }
            Placeholder::CategoryIdStart => {
                errs.first().map(|e| e.id.clone()).unwrap_or_default()
            }
            Placeholder::CategoryIdEnd => {
                errs.last().map(|e| e.id.clone()).unwrap_or_default()
            }
            Placeholder::Package => unreachable!(),
        }
    }

    fn replace_in_text(
        text: &str,
        package: &str,
        cat: Option<&Category>,
        errs: &[&ErrorEntry],
        err: Option<&ErrorEntry>,
    ) -> String {
        let mut s = text.to_string();
        s = s.replace("{{package}}", package);
        s = s.replace("{{category}}", cat.map(|c| c.name.as_str()).unwrap_or(""));
        s = s.replace(
            "{{category_description}}",
            cat.map(|c| c.description.as_str()).unwrap_or(""),
        );
        s = s.replace(
            "{{category_id_start}}",
            errs.first().map(|e| e.id.as_str()).unwrap_or(""),
        );
        s = s.replace(
            "{{category_id_end}}",
            errs.last().map(|e| e.id.as_str()).unwrap_or(""),
        );
        if let Some(e) = err {
            s = s.replace("{{id}}", &e.id);
            s = s.replace("{{name}}", &e.name);
            s = s.replace("{{message}}", &e.message);
            s = s.replace("{{hint}}", &e.hint);
            s = s.replace("{{severity}}", e.severity.as_str());
        } else {
            s = s.replace("{{id}}", "");
            s = s.replace("{{name}}", "");
            s = s.replace("{{message}}", "");
            s = s.replace("{{hint}}", "");
            s = s.replace("{{severity}}", "");
        }
        s
    }
}
