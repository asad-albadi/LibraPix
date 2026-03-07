-- Add optional display_name to source_roots for user-defined library labels.
ALTER TABLE source_roots ADD COLUMN display_name TEXT;
