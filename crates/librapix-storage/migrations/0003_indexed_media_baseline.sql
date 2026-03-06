CREATE TABLE IF NOT EXISTS indexed_media (
    id INTEGER PRIMARY KEY,
    source_root_id INTEGER NOT NULL,
    absolute_path TEXT NOT NULL UNIQUE,
    media_kind TEXT NOT NULL CHECK (media_kind IN ('image', 'video')),
    indexed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (source_root_id) REFERENCES source_roots(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_indexed_media_source_root_id
ON indexed_media (source_root_id);
