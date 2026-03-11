CREATE TABLE IF NOT EXISTS projection_snapshots (
    snapshot_key TEXT PRIMARY KEY,
    payload_json TEXT NOT NULL,
    updated_unix_seconds INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS timeline_navigation_snapshots (
    snapshot_key TEXT PRIMARY KEY,
    payload_json TEXT NOT NULL,
    updated_unix_seconds INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
