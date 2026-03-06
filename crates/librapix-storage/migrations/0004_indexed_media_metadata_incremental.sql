ALTER TABLE indexed_media
ADD COLUMN file_size_bytes INTEGER NOT NULL DEFAULT 0;

ALTER TABLE indexed_media
ADD COLUMN modified_unix_seconds INTEGER;

ALTER TABLE indexed_media
ADD COLUMN width_px INTEGER;

ALTER TABLE indexed_media
ADD COLUMN height_px INTEGER;

ALTER TABLE indexed_media
ADD COLUMN metadata_status TEXT NOT NULL DEFAULT 'ok'
CHECK (metadata_status IN ('ok', 'partial', 'unreadable', 'missing'));

ALTER TABLE indexed_media
ADD COLUMN last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE indexed_media
ADD COLUMN missing_since TEXT;

UPDATE indexed_media
SET last_seen_at = indexed_at;
