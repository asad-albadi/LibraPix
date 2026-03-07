CREATE TABLE IF NOT EXISTS source_root_statistics (
    source_root_id INTEGER PRIMARY KEY,
    total_size_bytes INTEGER NOT NULL DEFAULT 0,
    total_media_count INTEGER NOT NULL DEFAULT 0,
    total_images_count INTEGER NOT NULL DEFAULT 0,
    total_videos_count INTEGER NOT NULL DEFAULT 0,
    total_image_size_bytes INTEGER NOT NULL DEFAULT 0,
    total_video_size_bytes INTEGER NOT NULL DEFAULT 0,
    missing_count INTEGER NOT NULL DEFAULT 0,
    oldest_modified_unix_seconds INTEGER,
    newest_modified_unix_seconds INTEGER,
    last_indexed_unix_seconds INTEGER,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (source_root_id) REFERENCES source_roots(id) ON DELETE CASCADE
);
