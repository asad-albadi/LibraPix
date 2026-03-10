use crate::StorageError;
use rusqlite::{Connection, params};
use std::time::{Duration, Instant};

struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: [Migration; 11] = [
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
    Migration {
        version: 7,
        name: "source_root_display_name",
        sql: include_str!("../migrations/0007_source_root_display_name.sql"),
    },
    Migration {
        version: 8,
        name: "source_root_statistics",
        sql: include_str!("../migrations/0008_source_root_statistics.sql"),
    },
    Migration {
        version: 9,
        name: "catalog_first_foundation",
        sql: include_str!("../migrations/0009_catalog_first_foundation.sql"),
    },
    Migration {
        version: 10,
        name: "projection_snapshots",
        sql: include_str!("../migrations/0010_projection_snapshots.sql"),
    },
    Migration {
        version: 11,
        name: "catalog_history_reconciliation",
        sql: include_str!("../migrations/0011_catalog_history_reconciliation.sql"),
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigrationMetric {
    pub version: u32,
    pub name: &'static str,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationMetrics {
    pub previous_version: u32,
    pub final_version: u32,
    pub schema_setup_duration: Duration,
    pub version_lookup_duration: Duration,
    pub total_duration: Duration,
    pub applied: Vec<AppliedMigrationMetric>,
}

pub fn apply_migrations(connection: &Connection) -> Result<MigrationMetrics, StorageError> {
    let total_started_at = Instant::now();

    let schema_setup_started_at = Instant::now();
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;
    let schema_setup_duration = schema_setup_started_at.elapsed();

    let version_lookup_started_at = Instant::now();
    let current_version: u32 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    let version_lookup_duration = version_lookup_started_at.elapsed();

    let mut applied = Vec::new();
    let mut final_version = current_version;

    for migration in MIGRATIONS
        .iter()
        .filter(|migration| migration.version > current_version)
    {
        let migration_started_at = Instant::now();
        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            params![migration.version, migration.name],
        )?;
        transaction.commit()?;
        applied.push(AppliedMigrationMetric {
            version: migration.version,
            name: migration.name,
            duration: migration_started_at.elapsed(),
        });
        final_version = migration.version;
    }

    Ok(MigrationMetrics {
        previous_version: current_version,
        final_version,
        schema_setup_duration,
        version_lookup_duration,
        total_duration: total_started_at.elapsed(),
        applied,
    })
}
