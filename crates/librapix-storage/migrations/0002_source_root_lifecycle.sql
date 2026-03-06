ALTER TABLE source_roots
ADD COLUMN lifecycle_state TEXT NOT NULL DEFAULT 'active'
CHECK (lifecycle_state IN ('active', 'unavailable', 'deactivated'));

ALTER TABLE source_roots
ADD COLUMN last_availability_check_at TEXT;

UPDATE source_roots
SET lifecycle_state = CASE
    WHEN is_active = 1 THEN 'active'
    ELSE 'deactivated'
END;
