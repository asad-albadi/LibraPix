mod migration;

use rusqlite::{Connection, OptionalExtension, params};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedMediaRecord {
    pub id: i64,
    pub source_root_id: i64,
    pub absolute_path: PathBuf,
    pub media_kind: String,
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
            "SELECT id, normalized_path, lifecycle_state
             FROM source_roots
             ORDER BY id ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let path_str: String = row.get(1)?;
            let lifecycle_str: String = row.get(2)?;

            let lifecycle = SourceRootLifecycle::from_str(&lifecycle_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(lifecycle_str.clone()))?;

            Ok::<SourceRootRecord, rusqlite::Error>(SourceRootRecord {
                id: row.get(0)?,
                normalized_path: PathBuf::from(path_str),
                lifecycle,
            })
        })?;

        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        collected.map_err(StorageError::Sql)
    }

    pub fn list_eligible_source_roots(&self) -> Result<Vec<SourceRootRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, normalized_path, lifecycle_state
             FROM source_roots
             WHERE lifecycle_state = 'active'
             ORDER BY id ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let lifecycle_str: String = row.get(2)?;
            let lifecycle = SourceRootLifecycle::from_str(&lifecycle_str)
                .ok_or_else(|| rusqlite::Error::InvalidParameterName(lifecycle_str.clone()))?;

            Ok::<SourceRootRecord, rusqlite::Error>(SourceRootRecord {
                id: row.get(0)?,
                normalized_path: PathBuf::from(row.get::<usize, String>(1)?),
                lifecycle,
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

    pub fn replace_indexed_media(
        &mut self,
        entries: &[(i64, &Path, &str)],
    ) -> Result<(), StorageError> {
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM indexed_media", [])?;
        {
            let mut insert = transaction.prepare(
                "INSERT INTO indexed_media (source_root_id, absolute_path, media_kind)
                 VALUES (?1, ?2, ?3)",
            )?;
            for (root_id, absolute_path, media_kind) in entries {
                insert.execute(params![
                    root_id,
                    absolute_path.to_string_lossy().to_string(),
                    media_kind
                ])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn list_indexed_media(&self) -> Result<Vec<IndexedMediaRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, source_root_id, absolute_path, media_kind
             FROM indexed_media
             ORDER BY id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(IndexedMediaRecord {
                id: row.get(0)?,
                source_root_id: row.get(1)?,
                absolute_path: PathBuf::from(row.get::<usize, String>(2)?),
                media_kind: row.get(3)?,
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
        assert_eq!(version, 3);

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
}
