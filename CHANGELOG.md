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
- Added live filesystem watch subscription over active roots using `notify`.
- Auto-refresh now reacts to create/modify/remove file events without manual index/refresh.
- Replaced blocking watcher channel receive with async channel delivery in Iced subscription worker.
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
- Added media type filter (All / Images / Videos) as filter chips in the media pane toolbar.
- Added file extension filter (PNG, JPG, GIF, WEBP, MP4, MOV, etc.) as filter chips below the type filter.
- Filters apply to gallery, timeline, and search results simultaneously.
- Added size-based file exclusion during indexing via `ScanOptions.min_file_size_bytes`.
- Min file size is configurable from the sidebar with a KB input and Apply button.
- Gallery and detail preview thumbnail sizes increased to 400px and 800px for HiDPI clarity.
- MVP roadmap and README now reflect a complete usable baseline.
- Remaining MVP checklist now tracks completion of selection UX, state hardening, and final reconciliation.
- Added UI redesign checklist and app-shell/design-token architecture baseline.
- Redesigned app into header/sidebar/main/details shell with thumbnail-first gallery and visual timeline entries.
- Refined details pane into explicit preview, metadata, tags, and actions sections with consistent interaction layout.
- Updated architecture/roadmap docs to include UI shell, visual browsing, and redesign completion tracking.

### Added
- Root-level automatic tag assignment: tags can be assigned to a library root and automatically applied to all media under that root during indexing.
- Migration `0006_source_root_tags.sql` with `source_root_tags` table.
- Storage functions: `upsert_source_root_tag`, `remove_source_root_tag`, `list_source_root_tags`, `ensure_root_tags_exist`, `apply_root_auto_tags`.
- Auto-tag UI section in sidebar when a root is selected.
- i18n keys for auto-tag UI labels.

### Fixed
- Read-model query ordering changed from `ORDER BY absolute_path ASC` to `ORDER BY modified_unix_seconds DESC, absolute_path ASC` so gallery/timeline show most-recent-first across all roots instead of filling the limit with alphabetically-first paths (which favored one or two libraries).
- Video thumbnails on Windows: use `ffmpeg.exe` explicitly and normalize paths to forward slashes for ffmpeg subprocess compatibility.
- Extension filter chips for "All" type now include mkv and webm; video type includes avi.
- Roots added via UI are now persisted to config file so they survive restarts and bootstrap correctly.
- Startup no longer blocks the UI ("Not Responding" on Windows) while indexing/thumbnails run; all heavy work now executes via `Task::perform` on a background thread, keeping the UI interactive immediately.
- Gallery and timeline now aggregate all active libraries; removed hard query limits (200/500/120) that truncated multi-library results to a fraction of actual media.
- All valid media across registered roots are now indexed and displayed; thumbnail generation query limit raised from 200 to 50,000, gallery display limit from 120 to 50,000, timeline and search limits similarly increased.
- `FilesystemChanged`, `RunIndexing`, `ApplyMinFileSize`, `AddRoot`, and auto-tag operations now run indexing/projections asynchronously instead of blocking the UI thread.
- Dimensions now show correctly for files that were initially indexed before dimension extraction was implemented; indexer re-extracts dimensions for unchanged images missing width/height in the database.
- Media selection lag on first click eliminated by pre-caching detail-size thumbnail paths during projection builds instead of resolving them synchronously on click.
- Thumbnails now use Lanczos3 resampling instead of nearest-neighbor for substantially sharper results.
- Thumbnail cache key now includes the requested size, preventing gallery and detail views from sharing a single low-resolution cached file.
- Refresh button and item count in the media pane are no longer hidden behind the scrollbar; the toolbar is now outside the scrollable region.
- Dimensions now display correctly in the details panel; storage no longer overwrites existing dimensions with NULL for unchanged files.
- Media selection lag reduced by caching read-model data alongside browse items, avoiding per-click storage roundtrips.

### Changed (Media-View Architecture Milestone)
- Gallery and timeline now share a centralized media-view architecture with unified card rendering, thumbnail resolution, and justified row layout.
- Gallery rendering replaced with a Google-Photos-style adaptive justified layout using Iced `responsive` widget; images maintain aspect ratios, rows adapt to available width.
- Timeline rendering redesigned to show date-grouped justified mini-grids using the same card and layout primitives as gallery.
- Search results rendering unified to use the same justified layout as gallery.
- Header branding improved: "Libra" + "Pix" split with accent color on "Pix" and subtle "Media Library" subtitle.
- Minimum file size exclusion moved from Indexing sidebar section to Exclusions/Ignores section where it conceptually belongs.
- Video thumbnails are now generated via `ffmpeg` during indexing and displayed in gallery, timeline, search, and details views.
- Gallery projection limit increased from 60 to 120 items.
- Gallery gap reduced from 6px to 4px for tighter justified grid.
- Browse items now carry `aspect_ratio` for justified row computation.

### Docs
- Added ADR `0014` for filtering, exclusion, and thumbnail quality decisions.
- Added ADR `0015` for media-view architecture, justified layout, and video thumbnail decisions.
- Added quality milestone checklist.
- Updated architecture docs for thumbnails, UI, indexing, and search.
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
