use globset::{Glob, GlobSet, GlobSetBuilder};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Debug)]
pub struct IgnorePatternError(String);

impl Display for IgnorePatternError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid ignore pattern: {}", self.0)
    }
}

impl Error for IgnorePatternError {}

#[derive(Debug, Clone)]
pub struct IgnoreEngine {
    set: GlobSet,
}

impl IgnoreEngine {
    pub fn new(patterns: &[String]) -> Result<Self, IgnorePatternError> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            let glob = Glob::new(pattern).map_err(|_| IgnorePatternError(pattern.clone()))?;
            builder.add(glob);
        }
        let set = builder
            .build()
            .map_err(|error| IgnorePatternError(error.to_string()))?;
        Ok(Self { set })
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        self.set.is_match(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_glob_patterns() {
        let engine = IgnoreEngine::new(&["**/*.tmp".to_owned(), "**/cache/**".to_owned()])
            .expect("engine should compile");
        assert!(engine.is_ignored(Path::new("/tmp/x.tmp")));
        assert!(engine.is_ignored(Path::new("/tmp/cache/a.png")));
        assert!(!engine.is_ignored(Path::new("/tmp/media/a.png")));
    }
}
