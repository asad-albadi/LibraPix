use directories::ProjectDirs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub config_file: PathBuf,
    pub database_file: PathBuf,
}

pub fn default_paths() -> Result<ConfigPaths, &'static str> {
    let project_dirs =
        ProjectDirs::from("org", "Librapix", "Librapix").ok_or("project dirs unavailable")?;

    let config_dir = project_dirs.config_dir().to_path_buf();
    let data_dir = project_dirs.data_dir().to_path_buf();
    let cache_dir = project_dirs.cache_dir().to_path_buf();
    let thumbnails_dir = cache_dir.join("thumbnails");
    let config_file = config_dir.join("config.toml");
    let database_file = data_dir.join("librapix.db");

    Ok(ConfigPaths {
        config_dir,
        data_dir,
        cache_dir,
        thumbnails_dir,
        config_file,
        database_file,
    })
}

/// Lexically normalize a path without requiring filesystem existence.
pub fn lexical_normalize_path(path: &Path, cwd: &Path) -> PathBuf {
    let base = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    let mut components: Vec<Component<'_>> = Vec::new();
    for component in base.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let can_pop = components
                    .last()
                    .is_some_and(|last| !matches!(last, Component::RootDir | Component::Prefix(_)));
                if can_pop {
                    let _ = components.pop();
                }
            }
            _ => components.push(component),
        }
    }

    components
        .iter()
        .fold(PathBuf::new(), |mut acc, component| {
            acc.push(component.as_os_str());
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_relative_path() {
        let normalized = lexical_normalize_path(Path::new("a/./b/../c"), Path::new("/tmp"));
        assert_eq!(normalized, PathBuf::from("/tmp/a/c"));
    }
}
