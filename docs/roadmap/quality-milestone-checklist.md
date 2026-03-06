# Quality Milestone Checklist

## Thumbnail sharpness
- [x] Investigate thumbnail rendering path
- [x] Switch from nearest-neighbor to Lanczos3 resampling
- [x] Include max_edge in thumbnail cache key
- [x] Increase gallery thumbnail size to 400px
- [x] Increase detail preview size to 800px
- [x] Add test for different max_edge producing different cache paths

## Media pane layout
- [x] Identify scrollbar overlapping toolbar controls
- [x] Move toolbar outside scrollable region
- [x] Verify refresh button and item count are always visible

## Type filter
- [x] Add filter_media_kind state
- [x] Add SetFilterMediaKind message
- [x] Add filter chip UI (All / Images / Videos)
- [x] Apply to gallery projection
- [x] Apply to timeline projection
- [x] Apply to search results

## Extension filter
- [x] Add extension field to GalleryQuery
- [x] Add filter_extension state
- [x] Add SetFilterExtension message
- [x] Add extension chip UI (context-sensitive to type)
- [x] Apply to gallery projection
- [x] Apply to timeline projection
- [x] Apply to search results
- [x] Add extension filter test in projections

## Size-based exclusion
- [x] Add ScanOptions struct to indexer
- [x] Add min_file_size_bytes parameter
- [x] Skip files below threshold during scan
- [x] Add min_file_size_bytes to Librapix state
- [x] Add sidebar UI (input + KB label + Apply button)
- [x] Pass options to scan_roots in run_indexing
- [x] Update indexer tests

## i18n
- [x] Add FilterAllLabel, FilterImagesLabel, FilterVideosLabel
- [x] Add MinFileSizeLabel, MinFileSizeKbSuffix, ApplyLabel
- [x] Add English translations

## UI styling
- [x] Add filter_chip_style to ui.rs

## Verification
- [x] cargo fmt --all
- [x] cargo check --workspace
- [x] cargo clippy --workspace --all-targets -- -D warnings
- [x] cargo test --workspace (33 tests passing)
- [x] Smoke run passes

## Documentation
- [x] Update CHANGELOG.md
- [x] Update docs/architecture/ui.md
- [x] Update docs/architecture/thumbnails.md
- [x] Update docs/architecture/indexing.md
- [x] Update docs/architecture/search.md
- [x] Add ADR 0014
- [x] Add quality milestone checklist
- [x] Update docs/README.md
