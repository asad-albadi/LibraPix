use crate::StorageError;
use rusqlite::{Connection, params};

struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: [Migration; 6] = [
    Migration {
        version: 1,
        name: "baseline_foundation",
        sql: include_str!("../migrations/0001_baseline.sql"),
    },
    Migration {
        version: 2,
        name: "source_root_lifecycle",
        sql: include_str!("../migrations/0002_source_root_lifecycle.sql"),
    },
    Migration {
        version: 3,
        name: "indexed_media_baseline",
        sql: include_str!("../migrations/0003_indexed_media_baseline.sql"),
    },
    Migration {
        version: 4,
        name: "indexed_media_metadata_incremental",
        sql: include_str!("../migrations/0004_indexed_media_metadata_incremental.sql"),
    },
    Migration {
        version: 5,
        name: "tags_baseline",
        sql: include_str!("../migrations/0005_tags_baseline.sql"),
    },
    Migration {
        version: 6,
        name: "source_root_tags",
        sql: include_str!("../migrations/0006_source_root_tags.sql"),
    },
];

pub fn apply_migrations(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    let current_version: u32 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    for migration in MIGRATIONS
        .iter()
        .filter(|migration| migration.version > current_version)
    {
        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            params![migration.version, migration.name],
        )?;
        transaction.commit()?;
    }

    Ok(())
}
