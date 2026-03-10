mod catalog;
mod migration;

use rusqlite::{Connection, OptionalExtension, params, params_from_iter};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use catalog::{
    CatalogMediaRecord, CatalogRefreshSummary, DerivedArtifactKind, DerivedArtifactRecord,
    DerivedArtifactStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRootLifecycle {
    Active,
    Unavailable,
    Deactivated,
}

impl SourceRootLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            SourceRootLifecycle::Active => "active",
            SourceRootLifecycle::Unavailable => "unavailable",
            SourceRootLifecycle::Deactivated => "deactivated",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "unavailable" => Some(Self::Unavailable),
            "deactivated" => Some(Self::Deactivated),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRootRecord {
    pub id: i64,
    pub normalized_path: PathBuf,
    pub lifecycle: SourceRootLifecycle,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedMediaRecord {
    pub id: i64,
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub media_kind: String,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub metadata_status: IndexedMetadataStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexedMetadataStatus {
    Ok,
    Partial,
    Unreadable,
    Missing,
}

impl IndexedMetadataStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            IndexedMetadataStatus::Ok => "ok",
            IndexedMetadataStatus::Partial => "partial",
            IndexedMetadataStatus::Unreadable => "unreadable",
            IndexedMetadataStatus::Missing => "missing",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "partial" => Some(Self::Partial),
            "unreadable" => Some(Self::Unreadable),
            "missing" => Some(Self::Missing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedMediaSnapshot {
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedMediaWrite {
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub media_kind: String,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub metadata_status: IndexedMetadataStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IncrementalApplySummary {
    pub upserted_count: usize,
    pub missing_marked_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    App,
    Game,
}

impl TagKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TagKind::App => "app",
            TagKind::Game => "game",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "app" => Some(Self::App),
            "game" => Some(Self::Game),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRecord {
    pub id: i64,
    pub name: String,
    pub kind: TagKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaReadModel {
    pub media_id: i64,
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub media_kind: String,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub metadata_status: IndexedMetadataStatus,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRootTagRecord {
    pub id: i64,
    pub source_root_id: i64,
    pub tag_name: String,
    pub tag_kind: TagKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRootStatisticsRecord {
    pub source_root_id: i64,
    pub total_size_bytes: u64,
    pub total_media_count: u64,
    pub total_images_count: u64,
    pub total_videos_count: u64,
    pub total_image_size_bytes: u64,
    pub total_video_size_bytes: u64,
    pub missing_count: u64,
    pub oldest_modified_unix_seconds: Option<i64>,
    pub newest_modified_unix_seconds: Option<i64>,
    pub last_indexed_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoreRuleRecord {
    pub id: i64,
    pub scope: String,
    pub pattern: String,
    pub is_enabled: bool,
}

#[derive(Debug)]
pub enum StorageError {
    InvalidSourcePath(PathBuf),
    Io(std::io::Error),
    Sql(rusqlite::Error),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::InvalidSourcePath(path) => {
                write!(
                    f,
                    "source path must be absolute and non-empty: {}",
                    path.display()
                )
            }
            StorageError::Io(error) => write!(f, "{error}"),
            StorageError::Sql(error) => write!(f, "{error}"),
        }
    }
}

impl Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn open(database_file: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = database_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(database_file)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        migration::apply_migrations(&connection)?;

        Ok(Self { connection })
    }

    pub fn migration_version(&self) -> Result<u32, StorageError> {
        let version = self
            .connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get::<usize, u32>(0),
            )
            .optional()?
            .unwrap_or(0);

        Ok(version)
    }

    pub fn upsert_source_root(&self, normalized_path: &Path) -> Result<(), StorageError> {
        if normalized_path.as_os_str().is_empty() || !normalized_path.is_absolute() {
            return Err(StorageError::InvalidSourcePath(
                normalized_path.to_path_buf(),
            ));
        }

        self.connection.execute(
            "INSERT INTO source_roots (normalized_path, is_active, lifecycle_state, last_availability_check_at)
             VALUES (?1, 1, 'active', CURRENT_TIMESTAMP)
             ON CONFLICT(normalized_path)
             DO UPDATE SET
                is_active = excluded.is_active,
                lifecycle_state = 'active',
                last_availability_check_at = CURRENT_TIMESTAMP",
            params![normalized_path.to_string_lossy().to_string()],
        )?;
        Ok(())
    }

    pub fn update_source_root_display_name(
        &self,
        root_id: i64,
        display_name: &str,
    ) -> Result<(), StorageError> {
        let value = if display_name.trim().is_empty() {
            None::<&str>
        } else {
            Some(display_name.trim())
        };
        self.connection.execute(
            "UPDATE source_roots SET display_name = ?1 WHERE id = ?2",
            params![value, root_id],
        )?;
        Ok(())
    }

    pub fn update_source_root_path(
        &self,
        root_id: i64,
        normalized_path: &Path,
    ) -> Result<(), StorageError> {
        if normalized_path.as_os_str().is_empty() || !normalized_path.is_absolute() {
            return Err(StorageError::InvalidSourcePath(
                normalized_path.to_path_buf(),
            ));
        }

        self.connection.execute(
            "UPDATE source_roots
             SET normalized_path = ?1,
                 lifecycle_state = 'active',
                 is_active = 1,
                 last_availability_check_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
            params![normalized_path.to_string_lossy().to_string(), root_id],
        )?;
        Ok(())
    }

    pub fn set_source_root_lifecycle(
        &self,
        root_id: i64,
        lifecycle: SourceRootLifecycle,
    ) -> Result<(), StorageError> {
        let is_active = lifecycle != SourceRootLifecycle::Deactivated;
        self.connection.execute(
            "UPDATE source_roots
             SET lifecycle_state = ?1,
                 is_active = ?2,
                 last_availability_check_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![lifecycle.as_str(), if is_active { 1 } else { 0 }, root_id],
        )?;
        Ok(())
    }

    pub fn remove_source_root(&self, root_id: i64) -> Result<(), StorageError> {
        self.connection
            .execute("DELETE FROM source_roots WHERE id = ?1", params![root_id])?;
        Ok(())
    }

    pub fn list_source_roots(&self) -> Result<Vec<SourceRootRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, normalized_path, lifecycle_state, display_name
             FROM source_roots
             ORDER BY id ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let path_str: String = row.get(1)?;
            let lifecycle_str: String = row.get(2)?;
            let display_name: Option<String> = row.get(3)?;

            let lifecycle = SourceRootLifecycle::from_str(&lifecycle_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(lifecycle_str.clone()))?;

            Ok::<SourceRootRecord, rusqlite::Error>(SourceRootRecord {
                id: row.get(0)?,
                normalized_path: PathBuf::from(path_str),
                lifecycle,
                display_name,
            })
        })?;

        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        collected.map_err(StorageError::Sql)
    }

    pub fn list_eligible_source_roots(&self) -> Result<Vec<SourceRootRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, normalized_path, lifecycle_state, display_name
             FROM source_roots
             WHERE lifecycle_state = 'active'
             ORDER BY id ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let lifecycle_str: String = row.get(2)?;
            let display_name: Option<String> = row.get(3)?;
            let lifecycle = SourceRootLifecycle::from_str(&lifecycle_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(lifecycle_str.clone()))?;

            Ok::<SourceRootRecord, rusqlite::Error>(SourceRootRecord {
                id: row.get(0)?,
                normalized_path: PathBuf::from(row.get::<usize, String>(1)?),
                lifecycle,
                display_name,
            })
        })?;

        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        collected.map_err(StorageError::Sql)
    }

    pub fn reconcile_source_root_availability(&self) -> Result<(), StorageError> {
        let roots = self.list_source_roots()?;
        for root in roots
            .iter()
            .filter(|root| root.lifecycle != SourceRootLifecycle::Deactivated)
        {
            let next = if root.normalized_path.is_dir() {
                SourceRootLifecycle::Active
            } else {
                SourceRootLifecycle::Unavailable
            };
            self.set_source_root_lifecycle(root.id, next)?;
        }
        Ok(())
    }

    pub fn upsert_ignore_rule(
        &self,
        scope: &str,
        pattern: &str,
        is_enabled: bool,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO ignore_rules (scope, pattern, is_enabled)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(scope, pattern)
             DO UPDATE SET is_enabled = excluded.is_enabled",
            params![scope, pattern, if is_enabled { 1 } else { 0 }],
        )?;
        Ok(())
    }

    pub fn list_enabled_ignore_patterns(&self, scope: &str) -> Result<Vec<String>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT pattern
             FROM ignore_rules
             WHERE scope = ?1 AND is_enabled = 1
             ORDER BY id ASC",
        )?;
        let rows = statement.query_map(params![scope], |row| row.get::<usize, String>(0))?;
        let patterns: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(patterns?)
    }

    pub fn list_ignore_rules(&self, scope: &str) -> Result<Vec<IgnoreRuleRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, scope, pattern, is_enabled
             FROM ignore_rules
             WHERE scope = ?1
             ORDER BY id ASC",
        )?;
        let rows = statement.query_map(params![scope], |row| {
            Ok(IgnoreRuleRecord {
                id: row.get(0)?,
                scope: row.get(1)?,
                pattern: row.get(2)?,
                is_enabled: row.get::<usize, i64>(3)? == 1,
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn delete_ignore_rule_by_id(&self, id: i64) -> Result<(), StorageError> {
        self.connection
            .execute("DELETE FROM ignore_rules WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn replace_indexed_media(
        &mut self,
        entries: &[(i64, &Path, &str)],
    ) -> Result<(), StorageError> {
        let writes = entries
            .iter()
            .map(|(root_id, absolute_path, media_kind)| IndexedMediaWrite {
                source_root_id: *root_id,
                absolute_path: (*absolute_path).to_path_buf(),
                media_kind: (*media_kind).to_owned(),
                file_size_bytes: 0,
                modified_unix_seconds: None,
                width_px: None,
                height_px: None,
                metadata_status: IndexedMetadataStatus::Ok,
            })
            .collect::<Vec<_>>();
        let root_ids = writes
            .iter()
            .map(|entry| entry.source_root_id)
            .collect::<Vec<_>>();
        self.apply_incremental_index(&writes, &root_ids).map(|_| ())
    }

    pub fn apply_incremental_index(
        &mut self,
        entries: &[IndexedMediaWrite],
        scanned_root_ids: &[i64],
    ) -> Result<IncrementalApplySummary, StorageError> {
        let transaction = self.connection.transaction()?;
        transaction.execute_batch(
            "CREATE TEMP TABLE IF NOT EXISTS temp_seen_paths (
                absolute_path TEXT PRIMARY KEY
            );
            DELETE FROM temp_seen_paths;",
        )?;

        {
            let mut mark_seen = transaction.prepare(
                "INSERT OR IGNORE INTO temp_seen_paths (absolute_path)
                 VALUES (?1)",
            )?;
            let mut upsert = transaction.prepare(
                "INSERT INTO indexed_media (
                    source_root_id,
                    absolute_path,
                    media_kind,
                    file_size_bytes,
                    modified_unix_seconds,
                    width_px,
                    height_px,
                    metadata_status,
                    last_seen_at,
                    missing_since
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP, NULL)
                 ON CONFLICT(absolute_path) DO UPDATE SET
                    source_root_id = excluded.source_root_id,
                    media_kind = excluded.media_kind,
                    file_size_bytes = excluded.file_size_bytes,
                    modified_unix_seconds = excluded.modified_unix_seconds,
                    width_px = COALESCE(excluded.width_px, indexed_media.width_px),
                    height_px = COALESCE(excluded.height_px, indexed_media.height_px),
                    metadata_status = excluded.metadata_status,
                    last_seen_at = CURRENT_TIMESTAMP,
                    missing_since = NULL",
            )?;

            for entry in entries {
                let path_string = entry.absolute_path.to_string_lossy().to_string();
                mark_seen.execute(params![path_string])?;
                upsert.execute(params![
                    entry.source_root_id,
                    entry.absolute_path.to_string_lossy().to_string(),
                    entry.media_kind,
                    i64::try_from(entry.file_size_bytes).unwrap_or(i64::MAX),
                    entry.modified_unix_seconds,
                    entry.width_px.map(i64::from),
                    entry.height_px.map(i64::from),
                    entry.metadata_status.as_str(),
                ])?;
            }
        }

        let missing_marked_count = if scanned_root_ids.is_empty() {
            0
        } else {
            let placeholders = vec!["?"; scanned_root_ids.len()].join(", ");
            let query = format!(
                "UPDATE indexed_media
                 SET metadata_status = 'missing',
                     missing_since = COALESCE(missing_since, CURRENT_TIMESTAMP)
                 WHERE source_root_id IN ({placeholders})
                   AND absolute_path NOT IN (SELECT absolute_path FROM temp_seen_paths)
                   AND metadata_status != 'missing'"
            );
            transaction.execute(&query, params_from_iter(scanned_root_ids.iter()))?
        };

        transaction.commit()?;
        Ok(IncrementalApplySummary {
            upserted_count: entries.len(),
            missing_marked_count,
        })
    }

    pub fn list_existing_indexed_media_snapshots(
        &self,
        root_ids: &[i64],
    ) -> Result<Vec<IndexedMediaSnapshot>, StorageError> {
        if root_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = vec!["?"; root_ids.len()].join(", ");
        let query = format!(
            "SELECT source_root_id, absolute_path, file_size_bytes, modified_unix_seconds,
                    width_px, height_px
             FROM indexed_media
             WHERE source_root_id IN ({placeholders})"
        );
        let mut statement = self.connection.prepare(&query)?;
        let rows = statement.query_map(params_from_iter(root_ids.iter()), |row| {
            Ok(IndexedMediaSnapshot {
                source_root_id: row.get(0)?,
                absolute_path: PathBuf::from(row.get::<usize, String>(1)?),
                file_size_bytes: row.get::<usize, i64>(2).map_or(0, |v| v.max(0) as u64),
                modified_unix_seconds: row.get(3)?,
                width_px: row
                    .get::<usize, Option<i64>>(4)?
                    .and_then(|v| u32::try_from(v).ok()),
                height_px: row
                    .get::<usize, Option<i64>>(5)?
                    .and_then(|v| u32::try_from(v).ok()),
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn list_indexed_media(&self) -> Result<Vec<IndexedMediaRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, source_root_id, absolute_path, media_kind,
                    file_size_bytes, modified_unix_seconds, width_px, height_px, metadata_status
             FROM indexed_media
             ORDER BY id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            let status_string: String = row.get(8)?;
            let metadata_status = IndexedMetadataStatus::from_str(&status_string)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(status_string.clone()))?;
            Ok(IndexedMediaRecord {
                id: row.get(0)?,
                source_root_id: row.get(1)?,
                absolute_path: PathBuf::from(row.get::<usize, String>(2)?),
                media_kind: row.get(3)?,
                file_size_bytes: row.get::<usize, i64>(4).map_or(0, |v| v.max(0) as u64),
                modified_unix_seconds: row.get(5)?,
                width_px: row
                    .get::<usize, Option<i64>>(6)?
                    .and_then(|v| u32::try_from(v).ok()),
                height_px: row
                    .get::<usize, Option<i64>>(7)?
                    .and_then(|v| u32::try_from(v).ok()),
                metadata_status,
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn ensure_default_ignore_rules(&self) -> Result<(), StorageError> {
        self.upsert_ignore_rule("global", "**/thumbnails/**", true)?;
        self.upsert_ignore_rule("global", "**/cache/**", true)?;
        self.upsert_ignore_rule("global", "**/*.tmp", true)?;
        Ok(())
    }

    pub fn upsert_tag(&self, name: &str, kind: TagKind) -> Result<i64, StorageError> {
        self.connection.execute(
            "INSERT INTO tags (name, kind)
             VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET kind = excluded.kind",
            params![name, kind.as_str()],
        )?;
        let id = self.connection.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn attach_tag_to_media(&self, media_id: i64, tag_id: i64) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT OR IGNORE INTO media_tags (media_id, tag_id) VALUES (?1, ?2)",
            params![media_id, tag_id],
        )?;
        Ok(())
    }

    pub fn attach_tag_name_to_media(
        &self,
        media_id: i64,
        tag_name: &str,
        kind: TagKind,
    ) -> Result<(), StorageError> {
        let tag_id = self.upsert_tag(tag_name, kind)?;
        self.attach_tag_to_media(media_id, tag_id)
    }

    pub fn detach_tag_name_from_media(
        &self,
        media_id: i64,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "DELETE FROM media_tags
             WHERE media_id = ?1 AND tag_id IN (
                SELECT id FROM tags WHERE name = ?2
             )",
            params![media_id, tag_name],
        )?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<TagRecord>, StorageError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, kind FROM tags ORDER BY name ASC")?;
        let rows = statement.query_map([], |row| {
            let kind_str: String = row.get(2)?;
            let kind = TagKind::from_str(&kind_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(kind_str.clone()))?;
            Ok::<TagRecord, rusqlite::Error>(TagRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                kind,
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn list_media_tags(&self, media_id: i64) -> Result<Vec<TagRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT t.id, t.name, t.kind
             FROM tags t
             JOIN media_tags mt ON mt.tag_id = t.id
             WHERE mt.media_id = ?1
             ORDER BY t.name ASC",
        )?;
        let rows = statement.query_map(params![media_id], |row| {
            let kind_str: String = row.get(2)?;
            let kind = TagKind::from_str(&kind_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(kind_str.clone()))?;
            Ok::<TagRecord, rusqlite::Error>(TagRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                kind,
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn ensure_media_kind_tags_attached(&self) -> Result<(), StorageError> {
        let image_tag = self.upsert_tag("kind:image", TagKind::App)?;
        let video_tag = self.upsert_tag("kind:video", TagKind::App)?;

        self.connection.execute(
            "INSERT OR IGNORE INTO media_tags (media_id, tag_id)
             SELECT id, ?1
             FROM indexed_media
             WHERE media_kind = 'image' AND metadata_status != 'missing'",
            params![image_tag],
        )?;
        self.connection.execute(
            "INSERT OR IGNORE INTO media_tags (media_id, tag_id)
             SELECT id, ?1
             FROM indexed_media
             WHERE media_kind = 'video' AND metadata_status != 'missing'",
            params![video_tag],
        )?;

        Ok(())
    }

    pub fn count_indexed_media(&self) -> Result<i64, StorageError> {
        self.connection
            .query_row(
                "SELECT COUNT(*) FROM indexed_media WHERE metadata_status != 'missing'",
                [],
                |row| row.get(0),
            )
            .map_err(StorageError::Sql)
    }

    pub fn upsert_projection_snapshot(
        &self,
        snapshot_key: &str,
        payload_json: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO projection_snapshots (
                snapshot_key,
                payload_json,
                updated_unix_seconds,
                updated_at
             ) VALUES (
                ?1,
                ?2,
                CAST(strftime('%s', 'now') AS INTEGER),
                CURRENT_TIMESTAMP
             )
             ON CONFLICT(snapshot_key) DO UPDATE SET
                payload_json = excluded.payload_json,
                updated_unix_seconds = CAST(strftime('%s', 'now') AS INTEGER),
                updated_at = CURRENT_TIMESTAMP",
            params![snapshot_key, payload_json],
        )?;
        Ok(())
    }

    pub fn load_projection_snapshot(
        &self,
        snapshot_key: &str,
    ) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row(
                "SELECT payload_json
                 FROM projection_snapshots
                 WHERE snapshot_key = ?1",
                params![snapshot_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(StorageError::Sql)
    }

    pub fn refresh_source_root_statistics(&self, root_ids: &[i64]) -> Result<(), StorageError> {
        if root_ids.is_empty() {
            return Ok(());
        }

        let now_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        for root_id in root_ids {
            let (
                total_size_bytes,
                total_media_count,
                total_images_count,
                total_videos_count,
                total_image_size_bytes,
                total_video_size_bytes,
                missing_count,
                oldest_modified_unix_seconds,
                newest_modified_unix_seconds,
            ) = self.connection.query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN metadata_status != 'missing' THEN file_size_bytes ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN metadata_status != 'missing' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN metadata_status != 'missing' AND media_kind = 'image' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN metadata_status != 'missing' AND media_kind = 'video' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN metadata_status != 'missing' AND media_kind = 'image' THEN file_size_bytes ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN metadata_status != 'missing' AND media_kind = 'video' THEN file_size_bytes ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN metadata_status = 'missing' THEN 1 ELSE 0 END), 0),
                    MIN(CASE WHEN metadata_status != 'missing' THEN modified_unix_seconds END),
                    MAX(CASE WHEN metadata_status != 'missing' THEN modified_unix_seconds END)
                 FROM indexed_media
                 WHERE source_root_id = ?1",
                params![root_id],
                |row| {
                    Ok((
                        row.get::<usize, i64>(0)?,
                        row.get::<usize, i64>(1)?,
                        row.get::<usize, i64>(2)?,
                        row.get::<usize, i64>(3)?,
                        row.get::<usize, i64>(4)?,
                        row.get::<usize, i64>(5)?,
                        row.get::<usize, i64>(6)?,
                        row.get::<usize, Option<i64>>(7)?,
                        row.get::<usize, Option<i64>>(8)?,
                    ))
                },
            )?;

            self.connection.execute(
                "INSERT INTO source_root_statistics (
                    source_root_id,
                    total_size_bytes,
                    total_media_count,
                    total_images_count,
                    total_videos_count,
                    total_image_size_bytes,
                    total_video_size_bytes,
                    missing_count,
                    oldest_modified_unix_seconds,
                    newest_modified_unix_seconds,
                    last_indexed_unix_seconds,
                    updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)
                 ON CONFLICT(source_root_id) DO UPDATE SET
                    total_size_bytes = excluded.total_size_bytes,
                    total_media_count = excluded.total_media_count,
                    total_images_count = excluded.total_images_count,
                    total_videos_count = excluded.total_videos_count,
                    total_image_size_bytes = excluded.total_image_size_bytes,
                    total_video_size_bytes = excluded.total_video_size_bytes,
                    missing_count = excluded.missing_count,
                    oldest_modified_unix_seconds = excluded.oldest_modified_unix_seconds,
                    newest_modified_unix_seconds = excluded.newest_modified_unix_seconds,
                    last_indexed_unix_seconds = excluded.last_indexed_unix_seconds,
                    updated_at = CURRENT_TIMESTAMP",
                params![
                    root_id,
                    total_size_bytes.max(0),
                    total_media_count.max(0),
                    total_images_count.max(0),
                    total_videos_count.max(0),
                    total_image_size_bytes.max(0),
                    total_video_size_bytes.max(0),
                    missing_count.max(0),
                    oldest_modified_unix_seconds,
                    newest_modified_unix_seconds,
                    now_unix_seconds,
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_source_root_statistics(
        &self,
        root_id: i64,
    ) -> Result<Option<SourceRootStatisticsRecord>, StorageError> {
        self.connection
            .query_row(
                "SELECT
                    source_root_id,
                    total_size_bytes,
                    total_media_count,
                    total_images_count,
                    total_videos_count,
                    total_image_size_bytes,
                    total_video_size_bytes,
                    missing_count,
                    oldest_modified_unix_seconds,
                    newest_modified_unix_seconds,
                    last_indexed_unix_seconds
                 FROM source_root_statistics
                 WHERE source_root_id = ?1",
                params![root_id],
                |row| {
                    Ok(SourceRootStatisticsRecord {
                        source_root_id: row.get(0)?,
                        total_size_bytes: row.get::<usize, i64>(1).map_or(0, |v| v.max(0) as u64),
                        total_media_count: row.get::<usize, i64>(2).map_or(0, |v| v.max(0) as u64),
                        total_images_count: row.get::<usize, i64>(3).map_or(0, |v| v.max(0) as u64),
                        total_videos_count: row.get::<usize, i64>(4).map_or(0, |v| v.max(0) as u64),
                        total_image_size_bytes: row
                            .get::<usize, i64>(5)
                            .map_or(0, |v| v.max(0) as u64),
                        total_video_size_bytes: row
                            .get::<usize, i64>(6)
                            .map_or(0, |v| v.max(0) as u64),
                        missing_count: row.get::<usize, i64>(7).map_or(0, |v| v.max(0) as u64),
                        oldest_modified_unix_seconds: row.get(8)?,
                        newest_modified_unix_seconds: row.get(9)?,
                        last_indexed_unix_seconds: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(StorageError::Sql)
    }

    pub fn list_media_read_models(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MediaReadModel>, StorageError> {
        self.query_media_read_models(None, limit, offset)
    }

    pub fn list_all_media_read_models(&self) -> Result<Vec<MediaReadModel>, StorageError> {
        self.query_media_read_models_unbounded(None, None)
    }

    pub fn list_all_media_read_models_filtered(
        &self,
        source_root_id: Option<i64>,
    ) -> Result<Vec<MediaReadModel>, StorageError> {
        self.query_media_read_models_unbounded(None, source_root_id)
    }

    pub fn search_media_read_models(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MediaReadModel>, StorageError> {
        self.query_media_read_models(Some(query), limit, 0)
    }

    pub fn get_media_read_model_by_id(
        &self,
        media_id: i64,
    ) -> Result<Option<MediaReadModel>, StorageError> {
        let sql = "
            SELECT m.id, m.source_root_id, m.absolute_path, m.media_kind,
                   m.file_size_bytes, m.modified_unix_seconds, m.width_px, m.height_px,
                   m.metadata_status, COALESCE(GROUP_CONCAT(t.name, ','), '')
            FROM indexed_media m
            LEFT JOIN media_tags mt ON mt.media_id = m.id
            LEFT JOIN tags t ON t.id = mt.tag_id
            WHERE m.metadata_status != 'missing' AND m.id = ?1
            GROUP BY m.id";

        let mut statement = self.connection.prepare(sql)?;
        let row = statement
            .query_row(params![media_id], map_media_read_model_row)
            .optional()?;
        Ok(row)
    }

    fn query_media_read_models_unbounded(
        &self,
        query: Option<&str>,
        source_root_id: Option<i64>,
    ) -> Result<Vec<MediaReadModel>, StorageError> {
        let base_where = "m.metadata_status != 'missing'";
        let path_filter = query
            .map(|_| " AND (m.absolute_path LIKE '%' || ?1 || '%' OR t.name LIKE '%' || ?1 || '%')")
            .unwrap_or("");
        let root_filter = source_root_id
            .map(|_| {
                if query.is_some() {
                    " AND m.source_root_id = ?2"
                } else {
                    " AND m.source_root_id = ?1"
                }
            })
            .unwrap_or("");

        let sql = format!(
            "SELECT m.id, m.source_root_id, m.absolute_path, m.media_kind,
                    m.file_size_bytes, m.modified_unix_seconds, m.width_px, m.height_px,
                    m.metadata_status, COALESCE(GROUP_CONCAT(t.name, ','), '')
             FROM indexed_media m
             LEFT JOIN media_tags mt ON mt.media_id = m.id
             LEFT JOIN tags t ON t.id = mt.tag_id
             WHERE {base_where}{path_filter}{root_filter}
             GROUP BY m.id
             ORDER BY m.modified_unix_seconds DESC, m.absolute_path ASC"
        );

        let mut statement = self.connection.prepare(&sql)?;
        let mapper = |row: &rusqlite::Row<'_>| -> Result<MediaReadModel, rusqlite::Error> {
            map_media_read_model_row(row)
        };

        let rows = match (query, source_root_id) {
            (Some(text), Some(root_id)) => statement.query_map(params![text, root_id], mapper)?,
            (Some(text), None) => statement.query_map(params![text], mapper)?,
            (None, Some(root_id)) => statement.query_map(params![root_id], mapper)?,
            (None, None) => statement.query_map([], mapper)?,
        };
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    fn query_media_read_models(
        &self,
        query: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MediaReadModel>, StorageError> {
        let sql_with_filter = "
            SELECT m.id, m.source_root_id, m.absolute_path, m.media_kind,
                   m.file_size_bytes, m.modified_unix_seconds, m.width_px, m.height_px,
                   m.metadata_status, COALESCE(GROUP_CONCAT(t.name, ','), '')
            FROM indexed_media m
            LEFT JOIN media_tags mt ON mt.media_id = m.id
            LEFT JOIN tags t ON t.id = mt.tag_id
            WHERE m.metadata_status != 'missing'
              AND (m.absolute_path LIKE '%' || ?1 || '%' OR t.name LIKE '%' || ?1 || '%')
            GROUP BY m.id
            ORDER BY m.modified_unix_seconds DESC, m.absolute_path ASC
            LIMIT ?2 OFFSET ?3";

        let sql_without_filter = "
            SELECT m.id, m.source_root_id, m.absolute_path, m.media_kind,
                   m.file_size_bytes, m.modified_unix_seconds, m.width_px, m.height_px,
                   m.metadata_status, COALESCE(GROUP_CONCAT(t.name, ','), '')
            FROM indexed_media m
            LEFT JOIN media_tags mt ON mt.media_id = m.id
            LEFT JOIN tags t ON t.id = mt.tag_id
            WHERE m.metadata_status != 'missing'
            GROUP BY m.id
            ORDER BY m.modified_unix_seconds DESC, m.absolute_path ASC
            LIMIT ?1 OFFSET ?2";

        let mut statement = self.connection.prepare(if query.is_some() {
            sql_with_filter
        } else {
            sql_without_filter
        })?;

        let mapper = |row: &rusqlite::Row<'_>| -> Result<MediaReadModel, rusqlite::Error> {
            map_media_read_model_row(row)
        };

        let rows = if let Some(text) = query {
            statement.query_map(params![text, limit as i64, offset as i64], mapper)?
        } else {
            statement.query_map(params![limit as i64, offset as i64], mapper)?
        };
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn upsert_source_root_tag(
        &self,
        source_root_id: i64,
        tag_name: &str,
        tag_kind: TagKind,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO source_root_tags (source_root_id, tag_name, tag_kind)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(source_root_id, tag_name) DO UPDATE SET tag_kind = excluded.tag_kind",
            params![source_root_id, tag_name, tag_kind.as_str()],
        )?;
        Ok(())
    }

    pub fn remove_source_root_tag(
        &self,
        source_root_id: i64,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "DELETE FROM source_root_tags WHERE source_root_id = ?1 AND tag_name = ?2",
            params![source_root_id, tag_name],
        )?;
        Ok(())
    }

    pub fn list_source_root_tags(
        &self,
        source_root_id: i64,
    ) -> Result<Vec<SourceRootTagRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, source_root_id, tag_name, tag_kind
             FROM source_root_tags
             WHERE source_root_id = ?1
             ORDER BY tag_name ASC",
        )?;
        let rows = statement.query_map(params![source_root_id], |row| {
            let kind_str: String = row.get(3)?;
            let kind = TagKind::from_str(&kind_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(kind_str))?;
            Ok(SourceRootTagRecord {
                id: row.get(0)?,
                source_root_id: row.get(1)?,
                tag_name: row.get(2)?,
                tag_kind: kind,
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn apply_root_auto_tags(&self) -> Result<usize, StorageError> {
        let count = self.connection.execute(
            "INSERT OR IGNORE INTO media_tags (media_id, tag_id)
             SELECT m.id, t.id
             FROM indexed_media m
             JOIN source_root_tags srt ON srt.source_root_id = m.source_root_id
             JOIN tags t ON t.name = srt.tag_name
             WHERE m.metadata_status != 'missing'",
            [],
        )?;
        Ok(count)
    }

    pub fn ensure_root_tags_exist(&self) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT OR IGNORE INTO tags (name, kind)
             SELECT DISTINCT srt.tag_name, srt.tag_kind
             FROM source_root_tags srt",
            [],
        )?;
        Ok(())
    }
}

fn map_media_read_model_row(row: &rusqlite::Row<'_>) -> Result<MediaReadModel, rusqlite::Error> {
    let status_string: String = row.get(8)?;
    let metadata_status = IndexedMetadataStatus::from_str(&status_string)
        .ok_or_else(|| rusqlite::Error::InvalidParameterName(status_string.clone()))?;
    let tags_csv: String = row.get(9)?;
    Ok(MediaReadModel {
        media_id: row.get(0)?,
        source_root_id: row.get(1)?,
        absolute_path: PathBuf::from(row.get::<usize, String>(2)?),
        media_kind: row.get(3)?,
        file_size_bytes: row.get::<usize, i64>(4).map_or(0, |v| v.max(0) as u64),
        modified_unix_seconds: row.get(5)?,
        width_px: row
            .get::<usize, Option<i64>>(6)?
            .and_then(|v| u32::try_from(v).ok()),
        height_px: row
            .get::<usize, Option<i64>>(7)?
            .and_then(|v| u32::try_from(v).ok()),
        metadata_status,
        tags: if tags_csv.is_empty() {
            Vec::new()
        } else {
            tags_csv.split(',').map(ToOwned::to_owned).collect()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_file(name: &str) -> PathBuf {
        let unique = format!(
            "librapix-storage-{name}-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    #[test]
    fn opens_and_applies_baseline_migration() {
        let db = temp_db_file("migration");
        let storage = Storage::open(&db).expect("database should open");
        let version = storage
            .migration_version()
            .expect("migration version should be queryable");
        assert_eq!(version, 11);

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn projection_snapshot_round_trip() {
        let db = temp_db_file("projection-snapshot");
        let storage = Storage::open(&db).expect("database should open");

        storage
            .upsert_projection_snapshot("default", "{\"gallery\":[]}")
            .expect("snapshot upsert should succeed");
        let snapshot = storage
            .load_projection_snapshot("default")
            .expect("snapshot load should succeed");
        assert_eq!(snapshot.as_deref(), Some("{\"gallery\":[]}"));

        storage
            .upsert_projection_snapshot("default", "{\"gallery\":[1]}")
            .expect("snapshot update should succeed");
        let updated = storage
            .load_projection_snapshot("default")
            .expect("snapshot load should succeed");
        assert_eq!(updated.as_deref(), Some("{\"gallery\":[1]}"));

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn upsert_source_root_is_idempotent() {
        let db = temp_db_file("sources");
        let storage = Storage::open(&db).expect("database should open");
        let path = Path::new("/tmp/librapix-library");

        storage
            .upsert_source_root(path)
            .expect("first insert should succeed");
        storage
            .upsert_source_root(path)
            .expect("second insert should succeed");

        let roots = storage.list_source_roots().expect("roots should be listed");
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].normalized_path, path);
        assert_eq!(roots[0].lifecycle, SourceRootLifecycle::Active);

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn lifecycle_changes_and_reconciliation_work() {
        let db = temp_db_file("lifecycle");
        let storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/path/that/should/not/exist"))
            .expect("insert should work");

        storage
            .reconcile_source_root_availability()
            .expect("reconciliation should work");

        let roots = storage.list_source_roots().expect("roots should list");
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].lifecycle, SourceRootLifecycle::Unavailable);

        storage
            .set_source_root_lifecycle(roots[0].id, SourceRootLifecycle::Deactivated)
            .expect("deactivate should work");
        let roots_after = storage.list_source_roots().expect("roots should list");
        assert_eq!(roots_after[0].lifecycle, SourceRootLifecycle::Deactivated);

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn incremental_apply_marks_missing_entries() {
        let db = temp_db_file("incremental");
        let mut storage = Storage::open(&db).expect("database should open");
        let root = Path::new("/tmp/librapix-incremental-root");
        storage
            .upsert_source_root(root)
            .expect("root insert should work");
        let root_id = storage
            .list_source_roots()
            .expect("roots should list")
            .first()
            .expect("one root expected")
            .id;

        let first = vec![IndexedMediaWrite {
            source_root_id: root_id,
            absolute_path: PathBuf::from("/tmp/librapix-incremental-root/a.png"),
            media_kind: "image".to_owned(),
            file_size_bytes: 10,
            modified_unix_seconds: Some(100),
            width_px: Some(10),
            height_px: Some(10),
            metadata_status: IndexedMetadataStatus::Ok,
        }];
        storage
            .apply_incremental_index(&first, &[root_id])
            .expect("first apply should work");

        let summary = storage
            .apply_incremental_index(&[], &[root_id])
            .expect("second apply should work");
        assert_eq!(summary.missing_marked_count, 1);

        let media = storage.list_indexed_media().expect("media should list");
        assert_eq!(media.len(), 1);
        assert_eq!(media[0].metadata_status, IndexedMetadataStatus::Missing);

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn refresh_source_root_statistics_persists_expected_totals() {
        let db = temp_db_file("root-stats");
        let mut storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/tmp/librapix-root-stats"))
            .expect("root insert should work");
        let root_id = storage
            .list_source_roots()
            .expect("roots should list")
            .first()
            .expect("one root expected")
            .id;

        storage
            .apply_incremental_index(
                &[
                    IndexedMediaWrite {
                        source_root_id: root_id,
                        absolute_path: PathBuf::from("/tmp/librapix-root-stats/a.png"),
                        media_kind: "image".to_owned(),
                        file_size_bytes: 100,
                        modified_unix_seconds: Some(1000),
                        width_px: Some(10),
                        height_px: Some(10),
                        metadata_status: IndexedMetadataStatus::Ok,
                    },
                    IndexedMediaWrite {
                        source_root_id: root_id,
                        absolute_path: PathBuf::from("/tmp/librapix-root-stats/b.mp4"),
                        media_kind: "video".to_owned(),
                        file_size_bytes: 300,
                        modified_unix_seconds: Some(2000),
                        width_px: None,
                        height_px: None,
                        metadata_status: IndexedMetadataStatus::Ok,
                    },
                ],
                &[root_id],
            )
            .expect("apply should work");
        storage
            .apply_incremental_index(
                &[IndexedMediaWrite {
                    source_root_id: root_id,
                    absolute_path: PathBuf::from("/tmp/librapix-root-stats/a.png"),
                    media_kind: "image".to_owned(),
                    file_size_bytes: 100,
                    modified_unix_seconds: Some(1000),
                    width_px: Some(10),
                    height_px: Some(10),
                    metadata_status: IndexedMetadataStatus::Ok,
                }],
                &[root_id],
            )
            .expect("incremental refresh should work");

        storage
            .refresh_source_root_statistics(&[root_id])
            .expect("stats refresh should work");
        let stats = storage
            .get_source_root_statistics(root_id)
            .expect("stats query should work")
            .expect("stats should exist");

        assert_eq!(stats.total_size_bytes, 100);
        assert_eq!(stats.total_media_count, 1);
        assert_eq!(stats.total_images_count, 1);
        assert_eq!(stats.total_videos_count, 0);
        assert_eq!(stats.total_image_size_bytes, 100);
        assert_eq!(stats.total_video_size_bytes, 0);
        assert_eq!(stats.missing_count, 1);
        assert!(stats.last_indexed_unix_seconds.is_some());

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn read_model_search_matches_tag() {
        let db = temp_db_file("read-model-tags");
        let mut storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/tmp/librapix-read-model"))
            .expect("root insert should work");
        let root_id = storage
            .list_source_roots()
            .expect("roots should list")
            .first()
            .expect("one root expected")
            .id;

        let writes = vec![IndexedMediaWrite {
            source_root_id: root_id,
            absolute_path: PathBuf::from("/tmp/librapix-read-model/a.png"),
            media_kind: "image".to_owned(),
            file_size_bytes: 12,
            modified_unix_seconds: Some(200),
            width_px: Some(20),
            height_px: Some(20),
            metadata_status: IndexedMetadataStatus::Ok,
        }];
        storage
            .apply_incremental_index(&writes, &[root_id])
            .expect("apply should work");
        storage
            .ensure_media_kind_tags_attached()
            .expect("kind tags should attach");

        let rows = storage
            .search_media_read_models("kind:image", 10)
            .expect("search should work");
        assert_eq!(rows.len(), 1);

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn attach_and_detach_tag_by_name() {
        let db = temp_db_file("tag-attach-detach");
        let mut storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/tmp/librapix-tag-root"))
            .expect("root insert should work");
        let root_id = storage
            .list_source_roots()
            .expect("roots should list")
            .first()
            .expect("one root expected")
            .id;
        storage
            .apply_incremental_index(
                &[IndexedMediaWrite {
                    source_root_id: root_id,
                    absolute_path: PathBuf::from("/tmp/librapix-tag-root/a.png"),
                    media_kind: "image".to_owned(),
                    file_size_bytes: 1,
                    modified_unix_seconds: Some(1),
                    width_px: Some(1),
                    height_px: Some(1),
                    metadata_status: IndexedMetadataStatus::Ok,
                }],
                &[root_id],
            )
            .expect("apply should work");
        let media_id = storage
            .list_media_read_models(10, 0)
            .expect("rows should list")
            .first()
            .expect("row should exist")
            .media_id;

        storage
            .attach_tag_name_to_media(media_id, "boss-fight", TagKind::Game)
            .expect("attach should work");
        let attached = storage
            .get_media_read_model_by_id(media_id)
            .expect("row lookup should work")
            .expect("row should exist");
        assert!(attached.tags.iter().any(|tag| tag == "boss-fight"));

        storage
            .detach_tag_name_from_media(media_id, "boss-fight")
            .expect("detach should work");
        let detached = storage
            .get_media_read_model_by_id(media_id)
            .expect("row lookup should work")
            .expect("row should exist");
        assert!(!detached.tags.iter().any(|tag| tag == "boss-fight"));

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn list_all_media_read_models_does_not_truncate() {
        let db = temp_db_file("read-model-all");
        let mut storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/tmp/librapix-read-model-all"))
            .expect("root insert should work");
        let root_id = storage
            .list_source_roots()
            .expect("roots should list")
            .first()
            .expect("one root expected")
            .id;

        let writes = vec![
            IndexedMediaWrite {
                source_root_id: root_id,
                absolute_path: PathBuf::from("/tmp/librapix-read-model-all/new-image.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 12,
                modified_unix_seconds: Some(300),
                width_px: Some(20),
                height_px: Some(20),
                metadata_status: IndexedMetadataStatus::Ok,
            },
            IndexedMediaWrite {
                source_root_id: root_id,
                absolute_path: PathBuf::from("/tmp/librapix-read-model-all/old-video.mp4"),
                media_kind: "video".to_owned(),
                file_size_bytes: 14,
                modified_unix_seconds: Some(100),
                width_px: None,
                height_px: None,
                metadata_status: IndexedMetadataStatus::Ok,
            },
            IndexedMediaWrite {
                source_root_id: root_id,
                absolute_path: PathBuf::from("/tmp/librapix-read-model-all/mid-image.jpg"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(200),
                width_px: Some(10),
                height_px: Some(10),
                metadata_status: IndexedMetadataStatus::Ok,
            },
        ];
        storage
            .apply_incremental_index(&writes, &[root_id])
            .expect("apply should work");

        let capped = storage
            .list_media_read_models(2, 0)
            .expect("capped query should work");
        assert_eq!(capped.len(), 2);
        assert!(!capped.iter().any(|row| row.media_kind == "video"));

        let all_rows = storage
            .list_all_media_read_models()
            .expect("all query should work");
        assert_eq!(all_rows.len(), 3);
        assert!(all_rows.iter().any(|row| row.media_kind == "video"));

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn refresh_catalog_materializes_normalized_fields() {
        let db = temp_db_file("catalog-refresh");
        let mut storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/tmp/librapix-catalog-root"))
            .expect("root insert should work");
        let root = storage
            .list_source_roots()
            .expect("roots should list")
            .into_iter()
            .next()
            .expect("root should exist");
        storage
            .update_source_root_display_name(root.id, "Screenshots")
            .expect("display name should update");

        storage
            .apply_incremental_index(
                &[IndexedMediaWrite {
                    source_root_id: root.id,
                    absolute_path: PathBuf::from("/tmp/librapix-catalog-root/Boss Fight.PNG"),
                    media_kind: "image".to_owned(),
                    file_size_bytes: 42,
                    modified_unix_seconds: Some(1_710_000_000),
                    width_px: Some(1920),
                    height_px: Some(1080),
                    metadata_status: IndexedMetadataStatus::Ok,
                }],
                &[root.id],
            )
            .expect("index apply should work");
        storage
            .ensure_media_kind_tags_attached()
            .expect("kind tags should attach");
        let media_id = storage
            .list_all_media_read_models()
            .expect("read models should list")
            .first()
            .expect("media row should exist")
            .media_id;
        storage
            .attach_tag_name_to_media(media_id, "Boss", TagKind::Game)
            .expect("tag attach should work");

        let summary = storage
            .refresh_catalog()
            .expect("catalog refresh should work");
        assert_eq!(summary.upserted_count, 1);

        let catalog = storage
            .list_catalog_media_filtered(None)
            .expect("catalog rows should list");
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].file_name, "Boss Fight.PNG");
        assert_eq!(catalog[0].file_extension.as_deref(), Some("PNG"));
        assert_eq!(
            catalog[0].source_root_display_name.as_deref(),
            Some("Screenshots")
        );
        assert!(catalog[0].timeline_day_key.is_some());
        assert!(catalog[0].tags.iter().any(|tag| tag == "Boss"));
        assert!(catalog[0].search_text.contains("screenshots"));

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn derived_artifact_round_trip_filters_by_variant_and_media() {
        let db = temp_db_file("derived-artifacts");
        let mut storage = Storage::open(&db).expect("database should open");
        storage
            .upsert_source_root(Path::new("/tmp/librapix-derived-root"))
            .expect("root insert should work");
        let root_id = storage
            .list_source_roots()
            .expect("roots should list")
            .first()
            .expect("one root expected")
            .id;
        storage
            .apply_incremental_index(
                &[IndexedMediaWrite {
                    source_root_id: root_id,
                    absolute_path: PathBuf::from("/tmp/librapix-derived-root/a.png"),
                    media_kind: "image".to_owned(),
                    file_size_bytes: 10,
                    modified_unix_seconds: Some(10),
                    width_px: Some(10),
                    height_px: Some(10),
                    metadata_status: IndexedMetadataStatus::Ok,
                }],
                &[root_id],
            )
            .expect("index apply should work");
        let media_id = storage
            .list_all_media_read_models()
            .expect("read models should list")
            .first()
            .expect("media row should exist")
            .media_id;

        storage
            .upsert_derived_artifact(
                media_id,
                DerivedArtifactKind::Thumbnail,
                "gallery-400",
                Some(Path::new("abc123.png")),
                DerivedArtifactStatus::Ready,
            )
            .expect("artifact upsert should work");
        storage
            .upsert_derived_artifact(
                media_id,
                DerivedArtifactKind::Thumbnail,
                "detail-800",
                None,
                DerivedArtifactStatus::Failed,
            )
            .expect("failed artifact upsert should work");

        let gallery_artifacts = storage
            .list_ready_derived_artifacts_for_media_ids(
                &[media_id],
                DerivedArtifactKind::Thumbnail,
                "gallery-400",
            )
            .expect("gallery artifacts should list");
        assert_eq!(gallery_artifacts.len(), 1);
        assert_eq!(
            gallery_artifacts[0].relative_path,
            Some(PathBuf::from("abc123.png"))
        );

        let detail_artifacts = storage
            .list_ready_derived_artifacts_for_media_ids(
                &[media_id],
                DerivedArtifactKind::Thumbnail,
                "detail-800",
            )
            .expect("detail artifacts should list");
        assert!(detail_artifacts.is_empty());

        let _ = std::fs::remove_file(db);
    }
}
