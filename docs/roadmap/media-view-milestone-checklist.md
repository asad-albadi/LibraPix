# Media-View Architecture Milestone Checklist

## Centralized media-view architecture
- [x] Add `aspect_ratio` to `BrowseItem`
- [x] Create `render_media_card()` shared card primitive
- [x] Create `resolve_thumbnail()` unified image+video thumbnail resolver
- [x] Create `aspect_ratio_from()` helper
- [x] Unify gallery, timeline, and search to use shared primitives
- [x] Timeline renders date-grouped justified mini-grids using same card

## Adaptive justified gallery layout
- [x] Implement greedy row-building algorithm from aspect ratios
- [x] Use Iced `responsive` widget for width-aware layout
- [x] `FillPortion` proportional to aspect ratio for width distribution
- [x] Clamp row heights between 100px and 350px
- [x] Target row height of 200px
- [x] Gallery gap reduced to 4px

## Selection lag reduction
- [x] Add `CachedDetails` struct with full read-model data
- [x] Add `media_cache` HashMap to app state
- [x] Populate cache during gallery/timeline projection builds
- [x] `load_media_details_cached()` checks cache before storage query
- [x] Fallback to `load_media_details()` on cache miss

## Dimensions display fix
- [x] Storage SQL fix: `COALESCE(excluded.width_px, indexed_media.width_px)`
- [x] Unchanged files no longer lose stored dimensions on re-index

## Move min-size exclusion to Ignores
- [x] Removed min-size input from Indexing sidebar section
- [x] Added min-size input to Exclusions/Ignores sidebar section
- [x] Indexing pipeline still consumes `ScanOptions.min_file_size_bytes`

## Header branding
- [x] Split "Libra" + "Pix" with accent color on "Pix"
- [x] Added "· Media Library" subtitle in tertiary text
- [x] Maintained Fluent-inspired dark theme aesthetic

## Video thumbnails
- [x] Added `ensure_video_thumbnail()` to `librapix-thumbnails`
- [x] Uses `ffmpeg` command-line extraction (frame at 1 second)
- [x] Same cache key mechanism as image thumbnails
- [x] Integrated into indexing pipeline (generates both image and video thumbnails)
- [x] `resolve_thumbnail()` routes to image or video based on `media_kind`
- [x] Graceful failure when `ffmpeg` is unavailable

## Documentation
- [x] Updated CHANGELOG.md
- [x] Updated docs/architecture/ui.md
- [x] Updated docs/architecture/thumbnails.md
- [x] Updated docs/README.md
- [x] Created ADR 0015
- [x] Created this checklist

## Verification
- [x] cargo fmt --all
- [x] cargo check --workspace
- [x] cargo clippy --workspace --all-targets -- -D warnings
- [x] cargo test --workspace (34 tests pass)
- [x] Smoke run: app launches cleanly
