# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added
- Introduced a workspace layout with `librapix-app`, `librapix-core`, and `librapix-i18n`.
- Added initial architecture documentation set under `docs/architecture/`.
- Added roadmap documentation for MVP and future phases.
- Added ADR `0001` covering workspace boundaries and dependency direction.
- Added an i18n-ready app shell where user-facing text is key-based.
- Added `librapix-config` crate with typed config schema, TOML persistence, validation, and path normalization.
- Added `librapix-storage` crate with SQLite connection handling, migration tracking, and source-root persistence API.
- Added baseline SQLite migration `0001_baseline.sql` with `source_roots`, `app_settings`, and `ignore_rules`.
- Added `docs/architecture/config.md` and a phase checklist for config/storage foundation.
- Added ADR `0002` (config/path policy) and ADR `0003` (SQLite migration baseline).
- Added ADR `0004` defining source-root lifecycle reconciliation policy.
- Added storage migrations:
  - `0002_source_root_lifecycle.sql`
  - `0003_indexed_media_baseline.sql`
- Added `librapix-indexer` crate with scan roots pipeline, media candidate generation, and ignore engine tests.
- Added ADR `0005` for indexing + ignore-rule baseline.
- Added metadata extraction baseline for indexed media (size, modified time, image dimensions where supported).
- Added `docs/architecture/metadata.md`.
- Added ADR `0006` for metadata extraction, incremental indexing policy, and read-model baseline.
- Added tag-readiness baseline tables (`tags`, `media_tags`) and read-model queries.
- Added `librapix-search` crate with replaceable search contracts and baseline fuzzy ranking strategy.
- Added `librapix-projections` crate for timeline and gallery read projections.
- Added `librapix-thumbnails` crate for deterministic image thumbnail cache generation.
- Added media-details action baseline (load metadata, open file/folder, copy path).

### Changed
- App now auto-indexes and loads gallery/timeline on startup from persisted state.
- App now auto-indexes and refreshes gallery after adding a library root.
- Gallery now refreshes after removing a library root.
- Library root management now supports a native folder picker dialog (Browse button).
- Manual path input remains as a secondary/advanced flow below the Browse button.
- Double-clicking a media item in gallery or timeline opens it in the OS default external app.
- Single-click remains selection-focused; double-click is open-focused.
- File sizes in details panel now display as human-readable values (KB, MB, GB).
- Dates in details panel now display as human-readable local timestamps.
- Dimensions in details panel now display as formatted width × height.
- Details metadata now uses labeled key-value pairs (Type, Size, Modified, Dimensions, Path).
- Gallery card subtitles now show media kind and formatted file size.
- Gallery projection row lookup consolidated to single pass per item.
- Activity status indicator shows in the header during indexing and restore operations.
- Added centralized formatting module with format_file_size, format_timestamp, and format_dimensions helpers.
- Added new i18n keys for folder picker, activity status, and metadata labels.
- Redesigned UI with Fluent-inspired design system: comprehensive color palette, spacing scale, typography hierarchy, and component styles.
- Gallery now renders as a thumbnail-first card grid with selection states instead of a vertical list.
- Timeline renders with styled group headers and selectable media rows.
- Header now features a centered pill-shaped search bar with Fluent-style spacing.
- Sidebar now uses sectioned navigation with root status dot indicators.
- Details panel now uses clear sections separated by dividers with structured action layout.
- i18n strings updated to product-oriented language throughout.
- Gallery projection limit increased from 20 to 60 for richer browsing.
- BrowseItem subtitles simplified to user-facing format.
- Migrated from a single-crate starter to a multi-crate workspace.
- Declared MSRV `1.85` in workspace metadata and repository docs.
- App bootstrap now loads persisted config, opens storage, and syncs configured source roots.
- Storage now tracks root lifecycle states (`active`, `unavailable`, `deactivated`) and supports availability reconciliation.
- App orchestration now supports explicit root management flows (add, update, deactivate/reactivate, remove, refresh, select).
- App now runs an indexing baseline flow against persisted roots and stores candidates in `indexed_media`.
- Ignore rules are now centrally evaluated through the indexer engine.
- Indexing now applies incremental change detection (`new` / `changed` / `unchanged`) and marks missing files without destructive deletion.
- App can run storage read-model queries over indexed media and tags for verification-focused UI output.
- Search queries now run through the dedicated search subsystem instead of UI-owned matching logic.
- Timeline projections now group by indexed modified timestamp using day/month/year buckets.
- Gallery projections now support baseline filtering/sorting over read-model rows.
- Indexing now generates/reuses image thumbnails in app-owned cache with per-run status reporting.
- Storage read-model APIs now include media-by-id lookup and tag attach/detach by tag name.
- App orchestration now supports app-tag/game-tag attachment and media action workflows by selected media id.
- Gallery and timeline routes now render projection-specific panels instead of mixed debug output.
- App now supports enabling/disabling global ignore rules from UI and shows current rule status.
- App browsing now supports direct media selection from search/gallery/timeline route panels.
- Added explicit empty/loading/status messaging for roots, indexing, search, gallery, and timeline flows.
- MVP roadmap and README now reflect a complete usable baseline.
- Remaining MVP checklist now tracks completion of selection UX, state hardening, and final reconciliation.
- Added UI redesign checklist and app-shell/design-token architecture baseline.
- Redesigned app into header/sidebar/main/details shell with thumbnail-first gallery and visual timeline entries.
- Refined details pane into explicit preview, metadata, tags, and actions sections with consistent interaction layout.
- Updated architecture/roadmap docs to include UI shell, visual browsing, and redesign completion tracking.

### Docs
- Established baseline documentation for dependencies, troubleshooting, architecture, and repository map.
- Recorded dependency decisions for `serde`, `toml`, `directories`, and `rusqlite`.
- Recorded dependency decisions for `globset` and `walkdir`.
- Recorded dependency decisions for `imagesize`.
- Recorded dependency decisions for `strsim`.
- Recorded dependency decisions for `chrono`.
- Recorded dependency decisions for `image` and `sha2`.
- Recorded dependency decision for `rfd` (native file dialog).
- Added ADR `0013` for interaction milestone architecture decisions.
- Added interaction milestone checklist.
