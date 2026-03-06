mod migration;

use rusqlite::{Connection, OptionalExtension, params};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRootRecord {
    pub id: i64,
    pub normalized_path: PathBuf,
    pub is_active: bool,
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
            "INSERT INTO source_roots (normalized_path, is_active)
             VALUES (?1, 1)
             ON CONFLICT(normalized_path)
             DO UPDATE SET is_active = excluded.is_active",
            params![normalized_path.to_string_lossy().to_string()],
        )?;
        Ok(())
    }

    pub fn list_source_roots(&self) -> Result<Vec<SourceRootRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, normalized_path, is_active
             FROM source_roots
             ORDER BY id ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let path_str: String = row.get(1)?;
            let is_active_int: i64 = row.get(2)?;

            Ok(SourceRootRecord {
                id: row.get(0)?,
                normalized_path: PathBuf::from(path_str),
                is_active: is_active_int == 1,
            })
        })?;

        let collected: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(collected?)
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
        assert_eq!(version, 1);

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

        let _ = std::fs::remove_file(db);
    }
}
