CREATE TABLE IF NOT EXISTS source_root_tags (
    id INTEGER PRIMARY KEY,
    source_root_id INTEGER NOT NULL,
    tag_name TEXT NOT NULL,
    tag_kind TEXT NOT NULL CHECK (tag_kind IN ('app', 'game')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (source_root_id) REFERENCES source_roots(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_source_root_tags_unique
ON source_root_tags (source_root_id, tag_name);
