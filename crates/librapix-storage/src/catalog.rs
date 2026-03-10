use crate::{IndexedMetadataStatus, Storage, StorageError};
use chrono::{Datelike, Local, TimeZone};
use rusqlite::{params, params_from_iter};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogMediaRecord {
    pub media_id: i64,
    pub source_root_id: i64,
    pub source_root_display_name: Option<String>,
    pub absolute_path: PathBuf,
    pub file_name: String,
    pub file_extension: Option<String>,
    pub media_kind: String,
    pub file_size_bytes: u64,
    pub modified_unix_seconds: Option<i64>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub metadata_status: IndexedMetadataStatus,
    pub tags: Vec<String>,
    pub search_text: String,
    pub timeline_day_key: Option<String>,
    pub timeline_month_key: Option<String>,
    pub timeline_year_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CatalogRefreshSummary {
    pub upserted_count: usize,
    pub removed_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedArtifactKind {
    Thumbnail,
}

impl DerivedArtifactKind {
    fn as_str(self) -> &'static str {
        match self {
            DerivedArtifactKind::Thumbnail => "thumbnail",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "thumbnail" => Some(Self::Thumbnail),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedArtifactStatus {
    Ready,
    Failed,
}

impl DerivedArtifactStatus {
    fn as_str(self) -> &'static str {
        match self {
            DerivedArtifactStatus::Ready => "ready",
            DerivedArtifactStatus::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedArtifactRecord {
    pub media_id: i64,
    pub artifact_kind: DerivedArtifactKind,
    pub artifact_variant: String,
    pub relative_path: Option<PathBuf>,
    pub status: DerivedArtifactStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CatalogSourceRow {
    media_id: i64,
    source_root_id: i64,
    source_root_display_name: Option<String>,
    absolute_path: PathBuf,
    media_kind: String,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    width_px: Option<u32>,
    height_px: Option<u32>,
    metadata_status: IndexedMetadataStatus,
    tags_csv: String,
}

impl Storage {
    pub fn refresh_catalog(&mut self) -> Result<CatalogRefreshSummary, StorageError> {
        let source_rows = self.load_catalog_source_rows()?;
        let transaction = self.connection.transaction()?;
        transaction.execute_batch(
            "CREATE TEMP TABLE IF NOT EXISTS temp_seen_catalog_media (
                media_id INTEGER PRIMARY KEY
            );
            DELETE FROM temp_seen_catalog_media;",
        )?;

        {
            let mut mark_seen = transaction.prepare(
                "INSERT OR IGNORE INTO temp_seen_catalog_media (media_id)
                 VALUES (?1)",
            )?;
            let mut upsert = transaction.prepare(
                "INSERT INTO media_catalog (
                    media_id,
                    source_root_id,
                    source_root_display_name,
                    absolute_path,
                    file_name,
                    file_extension,
                    media_kind,
                    file_size_bytes,
                    modified_unix_seconds,
                    width_px,
                    height_px,
                    metadata_status,
                    tags_csv,
                    search_text,
                    timeline_day_key,
                    timeline_month_key,
                    timeline_year_key,
                    updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, CURRENT_TIMESTAMP)
                 ON CONFLICT(media_id) DO UPDATE SET
                    source_root_id = excluded.source_root_id,
                    source_root_display_name = excluded.source_root_display_name,
                    absolute_path = excluded.absolute_path,
                    file_name = excluded.file_name,
                    file_extension = excluded.file_extension,
                    media_kind = excluded.media_kind,
                    file_size_bytes = excluded.file_size_bytes,
                    modified_unix_seconds = excluded.modified_unix_seconds,
                    width_px = excluded.width_px,
                    height_px = excluded.height_px,
                    metadata_status = excluded.metadata_status,
                    tags_csv = excluded.tags_csv,
                    search_text = excluded.search_text,
                    timeline_day_key = excluded.timeline_day_key,
                    timeline_month_key = excluded.timeline_month_key,
                    timeline_year_key = excluded.timeline_year_key,
                    updated_at = CURRENT_TIMESTAMP",
            )?;

            for row in &source_rows {
                let record = materialize_catalog_record(row);
                mark_seen.execute(params![record.media_id])?;
                upsert.execute(params![
                    record.media_id,
                    record.source_root_id,
                    record.source_root_display_name,
                    record.absolute_path.to_string_lossy().to_string(),
                    record.file_name,
                    record.file_extension,
                    record.media_kind,
                    i64::try_from(record.file_size_bytes).unwrap_or(i64::MAX),
                    record.modified_unix_seconds,
                    record.width_px.map(i64::from),
                    record.height_px.map(i64::from),
                    record.metadata_status.as_str(),
                    record.tags.join(","),
                    record.search_text,
                    record.timeline_day_key,
                    record.timeline_month_key,
                    record.timeline_year_key,
                ])?;
            }
        }

        let removed_count = transaction.execute(
            "DELETE FROM media_catalog
             WHERE media_id NOT IN (SELECT media_id FROM temp_seen_catalog_media)",
            [],
        )?;
        transaction.commit()?;

        Ok(CatalogRefreshSummary {
            upserted_count: source_rows.len(),
            removed_count,
        })
    }

    pub fn list_catalog_media_filtered(
        &self,
        source_root_id: Option<i64>,
    ) -> Result<Vec<CatalogMediaRecord>, StorageError> {
        let (sql, params) = if source_root_id.is_some() {
            (
                "SELECT media_id, source_root_id, source_root_display_name, absolute_path,
                        file_name, file_extension, media_kind, file_size_bytes,
                        modified_unix_seconds, width_px, height_px, metadata_status,
                        tags_csv, search_text, timeline_day_key, timeline_month_key, timeline_year_key
                 FROM media_catalog
                 WHERE source_root_id = ?1
                 ORDER BY modified_unix_seconds DESC, absolute_path ASC",
                Some(source_root_id.unwrap_or_default()),
            )
        } else {
            (
                "SELECT media_id, source_root_id, source_root_display_name, absolute_path,
                        file_name, file_extension, media_kind, file_size_bytes,
                        modified_unix_seconds, width_px, height_px, metadata_status,
                        tags_csv, search_text, timeline_day_key, timeline_month_key, timeline_year_key
                 FROM media_catalog
                 ORDER BY modified_unix_seconds DESC, absolute_path ASC",
                None,
            )
        };

        let mut statement = self.connection.prepare(sql)?;
        let mapper = |row: &rusqlite::Row<'_>| map_catalog_media_row(row);
        let rows = if let Some(root_id) = params {
            statement.query_map(params![root_id], mapper)?
        } else {
            statement.query_map([], mapper)?
        };
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    pub fn upsert_derived_artifact(
        &self,
        media_id: i64,
        artifact_kind: DerivedArtifactKind,
        artifact_variant: &str,
        relative_path: Option<&Path>,
        status: DerivedArtifactStatus,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO derived_artifacts (
                media_id,
                artifact_kind,
                artifact_variant,
                relative_path,
                status,
                updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
             ON CONFLICT(media_id, artifact_kind, artifact_variant) DO UPDATE SET
                relative_path = excluded.relative_path,
                status = excluded.status,
                updated_at = CURRENT_TIMESTAMP",
            params![
                media_id,
                artifact_kind.as_str(),
                artifact_variant,
                relative_path.map(|path| path.to_string_lossy().to_string()),
                status.as_str(),
            ],
        )?;
        Ok(())
    }

    pub fn list_ready_derived_artifacts_for_media_ids(
        &self,
        media_ids: &[i64],
        artifact_kind: DerivedArtifactKind,
        artifact_variant: &str,
    ) -> Result<Vec<DerivedArtifactRecord>, StorageError> {
        if media_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = vec!["?"; media_ids.len()].join(", ");
        let sql = format!(
            "SELECT media_id, artifact_kind, artifact_variant, relative_path, status
             FROM derived_artifacts
             WHERE artifact_kind = ?1
               AND artifact_variant = ?2
               AND status = 'ready'
               AND media_id IN ({placeholders})
             ORDER BY media_id ASC"
        );
        let mut parameters = Vec::with_capacity(media_ids.len() + 2);
        parameters.push(rusqlite::types::Value::from(
            artifact_kind.as_str().to_owned(),
        ));
        parameters.push(rusqlite::types::Value::from(artifact_variant.to_owned()));
        parameters.extend(media_ids.iter().copied().map(rusqlite::types::Value::from));

        let mut statement = self.connection.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(parameters.iter()), |row| {
            map_derived_artifact_row(row)
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }

    fn load_catalog_source_rows(&self) -> Result<Vec<CatalogSourceRow>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT m.id, m.source_root_id, sr.display_name, m.absolute_path, m.media_kind,
                    m.file_size_bytes, m.modified_unix_seconds, m.width_px, m.height_px,
                    m.metadata_status, COALESCE(GROUP_CONCAT(t.name, ','), '')
             FROM indexed_media m
             JOIN source_roots sr ON sr.id = m.source_root_id
             LEFT JOIN media_tags mt ON mt.media_id = m.id
             LEFT JOIN tags t ON t.id = mt.tag_id
             WHERE m.metadata_status != 'missing'
             GROUP BY m.id
             ORDER BY m.id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            let status_string: String = row.get(9)?;
            let metadata_status = IndexedMetadataStatus::from_str(&status_string)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(status_string.clone()))?;
            Ok(CatalogSourceRow {
                media_id: row.get(0)?,
                source_root_id: row.get(1)?,
                source_root_display_name: row.get(2)?,
                absolute_path: PathBuf::from(row.get::<usize, String>(3)?),
                media_kind: row.get(4)?,
                file_size_bytes: row
                    .get::<usize, i64>(5)
                    .map_or(0, |value| value.max(0) as u64),
                modified_unix_seconds: row.get(6)?,
                width_px: row
                    .get::<usize, Option<i64>>(7)?
                    .and_then(|value| u32::try_from(value).ok()),
                height_px: row
                    .get::<usize, Option<i64>>(8)?
                    .and_then(|value| u32::try_from(value).ok()),
                metadata_status,
                tags_csv: row.get(10)?,
            })
        })?;
        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
    }
}

fn materialize_catalog_record(row: &CatalogSourceRow) -> CatalogMediaRecord {
    let tags = split_tags_csv(&row.tags_csv);
    let file_name = row
        .absolute_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| row.absolute_path.display().to_string());
    let file_extension = row
        .absolute_path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string());
    let (timeline_day_key, timeline_month_key, timeline_year_key) =
        build_timeline_keys(row.modified_unix_seconds);

    CatalogMediaRecord {
        media_id: row.media_id,
        source_root_id: row.source_root_id,
        source_root_display_name: row.source_root_display_name.clone(),
        absolute_path: row.absolute_path.clone(),
        file_name: file_name.clone(),
        file_extension: file_extension.clone(),
        media_kind: row.media_kind.clone(),
        file_size_bytes: row.file_size_bytes,
        modified_unix_seconds: row.modified_unix_seconds,
        width_px: row.width_px,
        height_px: row.height_px,
        metadata_status: row.metadata_status,
        tags: tags.clone(),
        search_text: build_search_text(
            row.source_root_display_name.as_deref(),
            &row.absolute_path,
            &file_name,
            file_extension.as_deref(),
            &row.media_kind,
            &tags,
        ),
        timeline_day_key,
        timeline_month_key,
        timeline_year_key,
    }
}

fn build_search_text(
    source_root_display_name: Option<&str>,
    absolute_path: &Path,
    file_name: &str,
    file_extension: Option<&str>,
    media_kind: &str,
    tags: &[String],
) -> String {
    let mut parts = Vec::with_capacity(tags.len() + 5);
    parts.push(absolute_path.display().to_string());
    parts.push(file_name.to_owned());
    parts.push(media_kind.to_owned());
    if let Some(extension) = file_extension {
        parts.push(extension.to_owned());
    }
    if let Some(display_name) = source_root_display_name.filter(|value| !value.trim().is_empty()) {
        parts.push(display_name.to_owned());
    }
    parts.extend(tags.iter().cloned());
    parts.join(" ").to_lowercase()
}

fn build_timeline_keys(
    modified_unix_seconds: Option<i64>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(timestamp) = modified_unix_seconds else {
        return (None, None, None);
    };
    let Some(local_dt) = Local.timestamp_opt(timestamp, 0).single() else {
        return (None, None, None);
    };
    (
        Some(format!(
            "{:04}-{:02}-{:02}",
            local_dt.year(),
            local_dt.month(),
            local_dt.day()
        )),
        Some(format!("{:04}-{:02}", local_dt.year(), local_dt.month())),
        Some(format!("{:04}", local_dt.year())),
    )
}

fn split_tags_csv(tags_csv: &str) -> Vec<String> {
    if tags_csv.is_empty() {
        Vec::new()
    } else {
        tags_csv.split(',').map(ToOwned::to_owned).collect()
    }
}

fn map_catalog_media_row(row: &rusqlite::Row<'_>) -> Result<CatalogMediaRecord, rusqlite::Error> {
    let status_string: String = row.get(11)?;
    let metadata_status = IndexedMetadataStatus::from_str(&status_string)
        .ok_or_else(|| rusqlite::Error::InvalidParameterName(status_string.clone()))?;
    let tags_csv: String = row.get(12)?;

    Ok(CatalogMediaRecord {
        media_id: row.get(0)?,
        source_root_id: row.get(1)?,
        source_root_display_name: row.get(2)?,
        absolute_path: PathBuf::from(row.get::<usize, String>(3)?),
        file_name: row.get(4)?,
        file_extension: row.get(5)?,
        media_kind: row.get(6)?,
        file_size_bytes: row
            .get::<usize, i64>(7)
            .map_or(0, |value| value.max(0) as u64),
        modified_unix_seconds: row.get(8)?,
        width_px: row
            .get::<usize, Option<i64>>(9)?
            .and_then(|value| u32::try_from(value).ok()),
        height_px: row
            .get::<usize, Option<i64>>(10)?
            .and_then(|value| u32::try_from(value).ok()),
        metadata_status,
        tags: split_tags_csv(&tags_csv),
        search_text: row.get(13)?,
        timeline_day_key: row.get(14)?,
        timeline_month_key: row.get(15)?,
        timeline_year_key: row.get(16)?,
    })
}

fn map_derived_artifact_row(
    row: &rusqlite::Row<'_>,
) -> Result<DerivedArtifactRecord, rusqlite::Error> {
    let artifact_kind = row
        .get::<usize, String>(1)
        .ok()
        .and_then(|value| DerivedArtifactKind::from_str(&value))
        .ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                1,
                "artifact_kind".to_owned(),
                rusqlite::types::Type::Text,
            )
        })?;
    let status = row
        .get::<usize, String>(4)
        .ok()
        .and_then(|value| DerivedArtifactStatus::from_str(&value))
        .ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(4, "status".to_owned(), rusqlite::types::Type::Text)
        })?;

    Ok(DerivedArtifactRecord {
        media_id: row.get(0)?,
        artifact_kind,
        artifact_variant: row.get(2)?,
        relative_path: row.get::<usize, Option<String>>(3)?.map(PathBuf::from),
        status,
    })
}
