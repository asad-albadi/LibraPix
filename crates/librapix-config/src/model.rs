use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LocalePreference {
    EnUs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreference {
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibrarySourceRoot {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PathOverrides {
    pub data_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub thumbnails_dir: Option<PathBuf>,
    pub database_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub schema_version: u32,
    pub locale: LocalePreference,
    pub theme: ThemePreference,
    pub library_source_roots: Vec<LibrarySourceRoot>,
    pub path_overrides: PathOverrides,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            locale: LocalePreference::EnUs,
            theme: ThemePreference::System,
            library_source_roots: Vec::new(),
            path_overrides: PathOverrides::default(),
        }
    }
}
