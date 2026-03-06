# Selection / Dimensions / Auto-Tag Milestone Checklist

## Selection Lag
- [x] Investigate current selection path (SelectMedia -> load_media_details_cached)
- [x] Remove synchronous thumbnail generation from click path
- [x] Cache detail-size thumbnails during projection builds
- [x] Verify first-click feels instant

## Dimensions Display
- [x] Investigate why dimensions show as "—" for existing media
- [x] Add width_px/height_px to IndexedMediaSnapshot and ExistingIndexedEntry
- [x] Fix indexer to re-extract dimensions for unchanged images with missing width/height
- [x] Verify COALESCE upsert preserves dimensions correctly
- [x] Verify details panel shows dimensions after fix

## Root-Level Auto-Tags
- [x] Design schema: source_root_tags table
- [x] Add migration 0006
- [x] Add storage functions: upsert/list/remove root tags
- [x] Add auto-tag application during indexing (ensure_root_tags_exist + apply_root_auto_tags)
- [x] Ensure root tags appear in read models and details
- [x] Add UI for managing root-level tags (sidebar section when root selected)

## Documentation
- [x] Update CHANGELOG.md
- [x] Update docs/architecture/storage.md (root auto-tags)
- [x] Update docs/architecture/indexing.md (dimension backfill, auto-tag pipeline)
- [x] Update docs/architecture/media-ui.md (selection optimization)
- [x] Update docs/TROUBLESHOOTING.md (dimensions backfill, selection lag)
- [x] Create ADR 0016 for auto-tag design

## Verification
- [x] cargo fmt / check / clippy / test pass
- [x] Smoke run passes
- [x] Commits created
- [x] Working tree clean
