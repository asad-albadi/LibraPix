mod model;
mod pathing;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

pub use model::{AppConfig, LibrarySourceRoot, LocalePreference, ThemePreference};
pub use pathing::{ConfigPaths, default_paths, lexical_normalize_path};

const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub enum ConfigError {
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    DuplicateLibraryPath(PathBuf),
    InvalidLibraryPath(PathBuf),
    MissingProjectDirs,
    Io(std::io::Error),
    ParseToml(toml::de::Error),
    SerializeToml(toml::ser::Error),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::UnsupportedSchemaVersion { found, supported } => {
                write!(
                    f,
                    "unsupported config schema version {found}; supported version is {supported}"
                )
            }
            ConfigError::DuplicateLibraryPath(path) => {
                write!(f, "duplicate library source path: {}", path.display())
            }
            ConfigError::InvalidLibraryPath(path) => {
                write!(f, "invalid library source path: {}", path.display())
            }
            ConfigError::MissingProjectDirs => write!(f, "unable to resolve project directories"),
            ConfigError::Io(error) => write!(f, "{error}"),
            ConfigError::ParseToml(error) => write!(f, "{error}"),
            ConfigError::SerializeToml(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::ParseToml(value)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(value: toml::ser::Error) -> Self {
        Self::SerializeToml(value)
    }
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub paths: ConfigPaths,
    pub config: AppConfig,
}

pub fn load_or_create() -> Result<LoadedConfig, ConfigError> {
    let paths = default_paths().map_err(|_| ConfigError::MissingProjectDirs)?;

    fs::create_dir_all(&paths.config_dir)?;
    fs::create_dir_all(&paths.data_dir)?;
    fs::create_dir_all(&paths.cache_dir)?;
    fs::create_dir_all(&paths.thumbnails_dir)?;

    let config = if paths.config_file.exists() {
        load_from_path(&paths.config_file)?
    } else {
        let default = AppConfig::default();
        save_to_path(&paths.config_file, &default)?;
        default
    };

    Ok(LoadedConfig { paths, config })
}

pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    let contents = fs::read_to_string(path)?;
    let mut parsed: AppConfig = toml::from_str(&contents)?;
    parsed.normalize_and_validate()?;
    Ok(parsed)
}

pub fn save_to_path(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    let mut to_save = config.clone();
    to_save.normalize_and_validate()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let serialized = toml::to_string_pretty(&to_save)?;
    fs::write(path, serialized.as_bytes())?;
    Ok(())
}

impl AppConfig {
    pub fn normalize_and_validate(&mut self) -> Result<(), ConfigError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }

        let cwd = std::env::current_dir()?;
        let mut normalized: Vec<PathBuf> = Vec::new();

        for source in &mut self.library_source_roots {
            let path = lexical_normalize_path(&source.path, &cwd);
            if path.as_os_str().is_empty() {
                return Err(ConfigError::InvalidLibraryPath(source.path.clone()));
            }

            source.path = path.clone();

            if normalized.contains(&path) {
                return Err(ConfigError::DuplicateLibraryPath(path));
            }
            normalized.push(path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_config_validates() {
        let mut config = AppConfig::default();
        config
            .normalize_and_validate()
            .expect("default config should validate");
    }

    #[test]
    fn duplicate_paths_are_rejected() {
        let mut config = AppConfig {
            library_source_roots: vec![
                LibrarySourceRoot {
                    path: PathBuf::from("/tmp/a"),
                },
                LibrarySourceRoot {
                    path: PathBuf::from("/tmp/./a"),
                },
            ],
            ..AppConfig::default()
        };

        let err = config
            .normalize_and_validate()
            .expect_err("duplicate roots must fail validation");
        assert!(matches!(err, ConfigError::DuplicateLibraryPath(_)));
    }
}
