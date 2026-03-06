mod ignore;
mod media;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub use ignore::{IgnoreEngine, IgnorePatternError};
pub use media::{MediaKind, classify_media_kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRoot {
    pub source_root_id: i64,
    pub normalized_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexCandidate {
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub media_kind: MediaKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexingSummary {
    pub scanned_roots: usize,
    pub skipped_roots: usize,
    pub missing_roots: usize,
    pub candidate_files: usize,
    pub ignored_entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexingResult {
    pub summary: IndexingSummary,
    pub candidates: Vec<IndexCandidate>,
}

pub fn scan_roots(roots: &[ScanRoot], ignore_engine: &IgnoreEngine) -> IndexingResult {
    let mut result = IndexingResult::default();

    for root in roots {
        if !root.normalized_path.is_dir() {
            result.summary.missing_roots += 1;
            result.summary.skipped_roots += 1;
            continue;
        }

        result.summary.scanned_roots += 1;

        for entry in WalkDir::new(&root.normalized_path)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path == root.normalized_path {
                continue;
            }

            if ignore_engine.is_ignored(path) {
                result.summary.ignored_entries += 1;
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let Some(kind) = classify_media_kind(path) else {
                continue;
            };

            result.candidates.push(IndexCandidate {
                source_root_id: root.source_root_id,
                absolute_path: path.to_path_buf(),
                media_kind: kind,
            });
            result.summary.candidate_files += 1;
        }
    }

    result
}

pub fn candidates_for_storage(result: &IndexingResult) -> Vec<(i64, &Path, &'static str)> {
    result
        .candidates
        .iter()
        .map(|candidate| {
            (
                candidate.source_root_id,
                candidate.absolute_path.as_path(),
                candidate.media_kind.as_str(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "librapix-indexer-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    #[test]
    fn scan_applies_ignore_and_detects_media() {
        let root = temp_dir("scan");
        fs::create_dir_all(root.join("cache")).expect("cache dir should be created");
        fs::write(root.join("shot.png"), []).expect("image file should be created");
        fs::write(root.join("clip.mp4"), []).expect("video file should be created");
        fs::write(root.join("cache").join("ignored.png"), [])
            .expect("ignored file should be created");
        fs::write(root.join("note.txt"), []).expect("text file should be created");

        let ignore = IgnoreEngine::new(&["**/cache/**".to_owned()]).expect("ignore should build");
        let roots = vec![ScanRoot {
            source_root_id: 1,
            normalized_path: root.clone(),
        }];

        let result = scan_roots(&roots, &ignore);
        assert_eq!(result.summary.scanned_roots, 1);
        assert_eq!(result.summary.candidate_files, 2);
        assert_eq!(result.summary.ignored_entries, 1);
        assert!(
            result
                .candidates
                .iter()
                .any(|candidate| candidate.absolute_path.ends_with("shot.png"))
        );
        assert!(
            result
                .candidates
                .iter()
                .any(|candidate| candidate.absolute_path.ends_with("clip.mp4"))
        );

        let _ = fs::remove_dir_all(root);
    }
}
