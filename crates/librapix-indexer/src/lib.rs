mod ignore;
mod media;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

pub use ignore::{IgnoreEngine, IgnorePatternError};
pub use media::{MediaKind, classify_media_kind, extract_image_dimensions};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingIndexedEntry {
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    New,
    Changed,
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataStatus {
    Ok,
    Partial,
    Unreadable,
}

impl MetadataStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MetadataStatus::Ok => "ok",
            MetadataStatus::Partial => "partial",
            MetadataStatus::Unreadable => "unreadable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedMediaUpsert {
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub media_kind: MediaKind,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub metadata_status: MetadataStatus,
    pub change_kind: ChangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexingSummary {
    pub scanned_roots: usize,
    pub skipped_roots: usize,
    pub missing_roots: usize,
    pub candidate_files: usize,
    pub ignored_entries: usize,
    pub unreadable_entries: usize,
    pub new_files: usize,
    pub changed_files: usize,
    pub unchanged_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexingResult {
    pub summary: IndexingSummary,
    pub candidates: Vec<IndexedMediaUpsert>,
    pub scanned_root_ids: Vec<i64>,
}

pub fn scan_roots(
    roots: &[ScanRoot],
    ignore_engine: &IgnoreEngine,
    existing_entries: &[ExistingIndexedEntry],
) -> IndexingResult {
    let mut result = IndexingResult::default();
    let existing_by_path: HashMap<PathBuf, &ExistingIndexedEntry> = existing_entries
        .iter()
        .map(|entry| (entry.absolute_path.clone(), entry))
        .collect();

    for root in roots {
        if !root.normalized_path.is_dir() {
            result.summary.missing_roots += 1;
            result.summary.skipped_roots += 1;
            continue;
        }

        result.summary.scanned_roots += 1;
        result.scanned_root_ids.push(root.source_root_id);

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
            let path_buf = path.to_path_buf();

            let fs_meta = fs::metadata(path);
            let (file_size_bytes, modified_unix_seconds) = match fs_meta {
                Ok(meta) => {
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                        .map(|duration| duration.as_secs() as i64);
                    (meta.len(), modified)
                }
                Err(_) => {
                    result.summary.unreadable_entries += 1;
                    result.candidates.push(IndexedMediaUpsert {
                        source_root_id: root.source_root_id,
                        absolute_path: path_buf,
                        media_kind: kind,
                        file_size_bytes: 0,
                        modified_unix_seconds: None,
                        width_px: None,
                        height_px: None,
                        metadata_status: MetadataStatus::Unreadable,
                        change_kind: ChangeKind::Changed,
                    });
                    result.summary.candidate_files += 1;
                    result.summary.changed_files += 1;
                    continue;
                }
            };

            let existing = existing_by_path.get(&path_buf).copied();
            let change_kind = match existing {
                None => ChangeKind::New,
                Some(previous)
                    if previous.file_size_bytes == file_size_bytes
                        && previous.modified_unix_seconds == modified_unix_seconds =>
                {
                    ChangeKind::Unchanged
                }
                Some(_) => ChangeKind::Changed,
            };

            let (width_px, height_px, metadata_status) =
                if kind == MediaKind::Image && change_kind != ChangeKind::Unchanged {
                    match extract_image_dimensions(path) {
                        Some((width, height)) => (Some(width), Some(height), MetadataStatus::Ok),
                        None => (None, None, MetadataStatus::Partial),
                    }
                } else {
                    (None, None, MetadataStatus::Ok)
                };

            match change_kind {
                ChangeKind::New => result.summary.new_files += 1,
                ChangeKind::Changed => result.summary.changed_files += 1,
                ChangeKind::Unchanged => result.summary.unchanged_files += 1,
            }

            result.candidates.push(IndexedMediaUpsert {
                source_root_id: root.source_root_id,
                absolute_path: path_buf,
                media_kind: kind,
                file_size_bytes,
                modified_unix_seconds,
                width_px,
                height_px,
                metadata_status,
                change_kind,
            });
            result.summary.candidate_files += 1;
        }
    }

    result
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

        let result = scan_roots(&roots, &ignore, &[]);
        assert_eq!(result.summary.scanned_roots, 1);
        assert_eq!(result.summary.candidate_files, 2);
        assert_eq!(result.summary.ignored_entries, 1);
        assert_eq!(result.summary.new_files, 2);
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

    #[test]
    fn detects_unchanged_entries() {
        let root = temp_dir("unchanged");
        fs::create_dir_all(&root).expect("root should be created");
        let file = root.join("same.png");
        fs::write(&file, []).expect("file should be created");
        let metadata = fs::metadata(&file).expect("metadata should load");
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs() as i64);

        let existing = vec![ExistingIndexedEntry {
            source_root_id: 1,
            absolute_path: file.clone(),
            file_size_bytes: metadata.len(),
            modified_unix_seconds: modified,
        }];
        let roots = vec![ScanRoot {
            source_root_id: 1,
            normalized_path: root.clone(),
        }];
        let ignore = IgnoreEngine::new(&[]).expect("ignore should compile");

        let result = scan_roots(&roots, &ignore, &existing);
        assert_eq!(result.summary.unchanged_files, 1);
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].change_kind, ChangeKind::Unchanged);

        let _ = fs::remove_dir_all(root);
    }
}
