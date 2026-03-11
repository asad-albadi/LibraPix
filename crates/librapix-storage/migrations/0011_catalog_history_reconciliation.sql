CREATE TABLE IF NOT EXISTS media_catalog (
    media_id INTEGER PRIMARY KEY,
    source_root_id INTEGER NOT NULL,
    source_root_display_name TEXT,
    absolute_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_extension TEXT,
    media_kind TEXT NOT NULL CHECK (media_kind IN ('image', 'video')),
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    modified_unix_seconds INTEGER,
    width_px INTEGER,
    height_px INTEGER,
    metadata_status TEXT NOT NULL DEFAULT 'ok'
        CHECK (metadata_status IN ('ok', 'partial', 'unreadable', 'missing')),
    tags_csv TEXT NOT NULL DEFAULT '',
    search_text TEXT NOT NULL DEFAULT '',
    timeline_day_key TEXT,
    timeline_month_key TEXT,
    timeline_year_key TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (media_id) REFERENCES indexed_media(id) ON DELETE CASCADE,
    FOREIGN KEY (source_root_id) REFERENCES source_roots(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_media_catalog_source_root_modified
ON media_catalog (source_root_id, modified_unix_seconds DESC, absolute_path ASC);

CREATE INDEX IF NOT EXISTS idx_media_catalog_day_key
ON media_catalog (timeline_day_key, modified_unix_seconds DESC, absolute_path ASC);

CREATE INDEX IF NOT EXISTS idx_media_catalog_kind_extension
ON media_catalog (media_kind, file_extension);

CREATE TABLE IF NOT EXISTS derived_artifacts (
    id INTEGER PRIMARY KEY,
    media_id INTEGER NOT NULL,
    artifact_kind TEXT NOT NULL CHECK (artifact_kind IN ('thumbnail')),
    artifact_variant TEXT NOT NULL,
    relative_path TEXT,
    status TEXT NOT NULL CHECK (status IN ('ready', 'failed')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (media_id) REFERENCES indexed_media(id) ON DELETE CASCADE,
    UNIQUE (media_id, artifact_kind, artifact_variant)
);

CREATE INDEX IF NOT EXISTS idx_derived_artifacts_lookup
ON derived_artifacts (artifact_kind, artifact_variant, status, media_id);
