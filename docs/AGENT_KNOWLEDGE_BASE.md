<!-- markdownlint-disable MD032 MD012 -->

# LibraPix Agent Knowledge Base

## Ingestion Metadata

- Last reviewed against repo state: 2026-03-09
- Intended audience: future AI engineering agents working on LibraPix
- Canonical source priority for conflict resolution:
  1. code
  2. ADRs
  3. architecture docs
  4. roadmap docs
  5. README/changelog

## 1) Project Identity

- Canonical product name: `LibraPix` (PascalCase, no space).
- Product type: cross-platform desktop app (Rust + Iced), not a web app.
- License: MIT (`LICENSE`).
- Core contract: non-destructive local media manager for screenshots and recordings; source files are read-only from app behavior.
- Target users: users managing large local screenshot/video libraries (gaming-heavy use case, but not gaming-only).
- Current direction: documentation-driven, explicit architecture boundaries, maintainable multi-crate workspace.
- Current release baseline in workspace metadata: `0.3.1`, MSRV `1.85`.

## 2) Product Scope and User-Visible Behavior (Current)

Implemented now:
- Multi-library management with add/edit/remove/deactivate/reactivate.
- Unified Add/Edit Library dialog:
  - browse-first folder picker (`rfd`)
  - optional manual path field toggle
  - display-name editing
  - root-level auto-tag chips (app/game) with add/edit/remove.
- Dedicated Library Statistics dialog (persisted maintained values per root).
- Gallery and Timeline browse routes with shared card rendering model.
- Timeline scrubber with anchor-based fast navigation and year markers.
- Header search (enter-to-run), fuzzy ranking, with filter integration.
- Filter dialog with four axes: library/type/extension/tag.
- Details pane with preview, metadata, tags, open/copy actions.
- Tag management in details (app/game tags, chip edit/remove).
- Ignore-rule management in settings (chip add/edit/remove + enable/disable).
- Settings dialog for operational controls (indexing settings, ignore rules, diagnostics).
- About dialog and GitHub header action.
- Update checker in header chip:
  - startup check
  - periodic 24h check
  - manual click check with 5-minute cooldown
  - click opens latest release URL when update is available.
- Live filesystem watching + background refresh + new-file modal announcement.
- Copy/Open actions:
  - open file
  - open containing folder
  - copy file object
  - copy path text.
- Keyboard shortcuts:
  - `Cmd/Ctrl+C`: copy selected file
  - `Cmd/Ctrl+Shift+C`: copy selected path.

## 3) Current UX Shell Layout

Shell (single-window Iced app):
- Header: branding (`Libra` + `Pix`), search input, update chip, activity text, Settings, About, GitHub.
- Sidebar: browse nav + library list + library row actions (`Edit`, `Stats`) + add-library action.
- Main pane: gallery/timeline/search content and top toolbar stats + Filters button.
- Details pane: selected media preview, metadata, tag chips, action buttons.

Dialogs/overlays:
- Filter dialog
- Settings dialog
- About dialog
- Add/Edit Library dialog
- Library Statistics dialog
- New-file announcement modal.

Modal behavior:
- backdrop click closes dialogs (`CloseAllDialogs`)
- dialog content consumes clicks (`ModalContentClicked`) to avoid click-through.

## 4) Core App Flow (State/Message/Tasks)

Architecture style:
- Explicit message loop (`Message` enum in `librapix-app/src/main.rs`).
- `Librapix` holds app runtime/UI state.
- `AppState` from `librapix-core` tracks route/root/media/query summaries.
- Heavy work uses `Task::perform` with explicit staged jobs (`snapshot hydrate`, `scan`, `projection`, `thumbnail batches`).

Startup:
- `init()` returns `Task::done(Message::StartupRestore)`.
- Startup restore:
  - hydrate persisted projection snapshot first
  - if roots exist: queue delayed startup reconcile
  - trigger startup update-check task.

Background work:
- `do_snapshot_hydrate` loads roots/ignore rules and optional persisted projection snapshot.
- `do_scan_job` opens storage in worker context, reconciles roots, scans/indexes, applies auto-tags/statistics, and returns indexing summary data.
- `do_projection_job` refreshes `media_catalog`, queries normalized catalog rows, shapes gallery/timeline/search results, and schedules missing thumbnail work.
- `do_thumbnail_batch` generates a bounded batch of thumbnails and records artifact readiness in storage.
- Results are applied through staged messages, not a single monolithic completion handler.

Non-blocking policy:
- Indexing, projection refresh, search refresh, filter changes, filesystem-triggered updates use background tasks.

## 5) Crate Responsibilities

- `librapix-app`: executable, Iced UI, message handling, orchestration, update checker, platform actions, dialogs, subscriptions.
- `librapix-core`: app/domain primitives (`AppState`, route/root concepts, non-destructive rule enum).
- `librapix-config`: typed TOML config, path defaults/normalization, load/save/validation.
- `librapix-storage`: SQLite access, migrations, root/ignore/tag/indexed-media/statistics read-write APIs.
- `librapix-indexer`: recursive scanning, ignore matching, extension-based media classification, image dimension extraction, incremental change classification.
- `librapix-search`: replaceable search contracts + default fuzzy strategy.
- `librapix-projections`: gallery/timeline read projections and timeline anchor generation.
- `librapix-thumbnails`: deterministic thumbnail cache paths; image and video thumbnail generation.
- `librapix-i18n`: keyed text catalog, locale handling (`en-US` currently).

## 6) Persistence, Data Model, and Ownership

Storage engine:
- SQLite via `rusqlite` (`bundled`), migrations tracked in `schema_migrations`.
- Current migration version: `8`.

Tables (current):
- `source_roots`: path, lifecycle, availability timestamp, optional display name.
- `app_settings`: key/value.
- `ignore_rules`: scope/pattern/enabled.
- `indexed_media`: media rows (path/kind/size/mtime/dimensions/metadata status/missing markers).
- `tags`: app/game tags.
- `media_tags`: many-to-many media-tag links.
- `source_root_tags`: per-root auto-tag rules.
- `source_root_statistics`: maintained per-root summary/index stats.

Important persistence behaviors:
- Missing files are marked (`metadata_status = 'missing'`), not physically touched.
- Root lifecycle is explicit (`active`, `unavailable`, `deactivated`).
- Config is synchronized from storage root records after root mutations.
- Startup imports config roots only when storage has no roots.

Config model (`config.toml`):
- schema version, locale, theme, library roots, optional path overrides.
- lexical path normalization; duplicate roots rejected.
- default dirs resolved via `directories::ProjectDirs`.

## 7) Indexing and Refresh Model

Indexer input:
- active roots from storage
- enabled global ignore patterns
- min-size threshold (`ScanOptions.min_file_size_bytes`).

Scan behavior:
- recursive traversal with `walkdir` (no depth cap)
- ignore matching centralized in `IgnoreEngine` (`globset`)
- extension-based media kind detection:
  - images: `png jpg jpeg gif bmp webp tif tiff`
  - videos: `mp4 mov mkv webm avi`.

Incremental behavior:
- compares existing rows by path + file size + modified time
- marks `new`, `changed`, `unchanged`
- re-extracts dimensions for unchanged images if dims missing
- unreadable files become metadata `Unreadable`
- non-seen files in scanned roots become `missing`.

Post-scan pipeline:
- upsert indexed media
- attach `kind:image` / `kind:video`
- ensure/apply root auto-tags
- refresh per-root maintained stats
- run thumbnail generation for all current rows.

Live refresh:
- filesystem watcher subscription (`notify`) over active roots, recursive.
- create/modify/remove events trigger debounced `FilesystemChanged`.
- background index+project runs; new media IDs can trigger announcement modal.

## 8) Search and Filtering

Search strategy:
- `librapix-search::FuzzySearchStrategy` with all-terms-must-match behavior.
- scores exact > partial > fuzzy (`normalized_levenshtein`, threshold 0.70).
- query limit is explicit and derived (`all_rows.len()`), no fixed 20 cap.

Filter axes in app state:
- `filter_source_root_id`
- `filter_media_kind`
- `filter_extension`
- `filter_tag`.

Filter behavior:
- extension options adapt by kind filter.
- tag filter options derived from read models; internal `kind:*` tags excluded.
- if selected tag disappears, active filter tag is cleared.
- filters apply to gallery, timeline, and search paths.

## 9) Projection and Media Presentation Model

Unified browse item model (`BrowseItem`) powers gallery/timeline/search cards.

Gallery:
- projection sort: modified desc, then path asc.
- adaptive justified rows (target height 200, clamped 100..350).
- no hidden UI cap.

Timeline:
- grouped by local-time day (`TimelineGranularity::Day` in app usage).
- includes group headers with image/video counts.
- unknown timestamp bucket supported.
- anchors from projection model (`TimelineAnchor`) with structure-weighted normalized positions.

Scrubber:
- active in Timeline mode.
- continuous scrub value + nearest-anchor selection.
- year markers derived from anchor positions.
- programmatic scroll via `operation::scroll_to` with fallback.
- viewport sync updates scrub state.

Selection/details:
- selection is explicit (`selected_media_id`).
- details load from in-memory cache first; storage fallback.
- detail thumbnail path pre-resolved in cache to avoid click-path I/O.

## 10) Thumbnail Pipeline

Ownership:
- thumbnails are app-managed cache artifacts, never written into user folders.

Keying:
- SHA-256 over source path + file size + modified time + requested size.

Generation:
- image: `image` crate, Lanczos3, PNG output.
- video: ffmpeg subprocess at ~1s, scaled to requested max edge.
- default sizes in app:
  - gallery thumb: 400
  - detail thumb: 800.

Failure policy:
- per-item failure counted; pipeline continues.
- UI gracefully uses placeholder when unavailable.

Windows-specific:
- uses `ffmpeg.exe` explicitly
- normalizes paths to forward slashes for ffmpeg compatibility
- subprocess uses `CREATE_NO_WINDOW` to avoid console flicker.

## 11) Platform-Specific Behavior

Open actions:
- macOS: `open`
- Windows: `opener::open(...)`
- Linux/Unix: `xdg-open`.

Copy path text:
- macOS: `pbcopy`
- Windows: `clip`
- Linux/Unix: `xclip -selection clipboard`.

Copy file object:
- macOS: `osascript` file clipboard flow.
- Windows: native `CF_HDROP` payload via Win32 clipboard API (`SetClipboardData`), with retry/open/ownership handling.
- Linux/Unix: `xclip` with `x-special/gnome-copied-files` payload.

Window/process behavior:
- Windows binary uses `windows_subsystem = "windows"` to avoid console window on app launch.

Packaging/release baseline:
- Current CI release workflow builds:
  - Windows `.exe`
  - Linux `.AppImage`
- macOS packaging steps are present but currently disabled in workflow comments pending codesigning/notarization in CI.
- `packaging/windows/` scripts were removed from repo (recorded in changelog).

## 12) Branding and Asset Integration

Canonical branding:
- Blue variant is default/canonical (`assets/logo/blue/...`).

Alt variants:
- white and black logo sets exist for surface contrast variants.

In-app asset integration:
- logos/icons are embedded via `include_bytes!` in `librapix-app/src/assets.rs`.
- lazy static handles avoid per-render handle rebuild.

README/repo branding state:
- README shows logo/screenshot sections and branded feature list.
- MIT license explicitly present.

Notable asset naming caveat:
- icon file path uses `gallary.png` in repository/assets code (spelling preserved in path; treat as canonical filename unless renamed everywhere).

## 13) Update Checking Model

State machine:
- `Unknown`
- `Checking`
- `UpToDate`
- `UpdateAvailable { version, url }`
- `Failed`.

Source:
- GitHub latest release API: `GET /repos/asad-albadi/LibraPix/releases/latest`
- draft/prerelease excluded.
- semver comparison after trimming optional `v`/`V` prefix.

Scheduling:
- startup check after first render.
- periodic check every 24h while app is open.
- manual chip click check with 5-minute cooldown.
- no overlapping checks while in `Checking`.

UI behavior:
- chip text/style varies by state.
- if update available, chip click opens release URL.
- otherwise click triggers manual re-check (if cooldown permits).

## 14) Implemented vs Partial vs Deferred

Implemented (high confidence from code + docs):
- end-to-end desktop browse/index/search/details/actions/tag flows
- root lifecycle + statistics + root auto-tags
- unified library dialog and settings/about/filter dialogs
- update check state machine + scheduling + chip behavior
- timeline scrubber anchor navigation
- live watcher-driven refresh and announcement modal
- platform-specific copy/open actions including Windows CF_HDROP.

Partially implemented:
- i18n architecture exists but only `en-US` catalog is implemented.
- theme preference model supports `System/Dark/Light`; current product styling is dark-first.
- diagnostics/event-log exists mainly as internal observability.

Deferred / future:
- memories-style resurfacing workflow
- advanced search/faceting/ranking and indexed-search scaling
- richer video metadata extraction (duration/codecs/etc.)
- optional plugin points (only if proven necessary)
- further large-library performance profiling/optimization
- broader locale packs/tooling
- macOS CI packaging/sign/notarization completion in release workflow.

## 15) Known Caveats and Risks

- External tool dependencies:
  - ffmpeg required for video thumbnails
  - xclip required for Linux clipboard flows.
- `list_all_media_read_models_*` is intentionally unbounded; good for correctness, may need explicit pagination/virtualization strategy for very large datasets.
- Some architecture docs can lag implementation details in spots (for example older action-path wording); treat code + ADRs as source of truth.
- Background task results are applied atomically; concurrent task completions use last-completed result semantics.

## 16) Safe Extension Guidance for Future Agents

Do:
- preserve non-destructive guarantee in every new feature.
- keep UI presentation logic in `librapix-app`; keep persistence/indexing/search in their crates.
- keep side effects message-driven and explicit; schedule heavy work in `Task::perform`.
- update docs + changelog in same workstream for structural changes.
- add migration + storage API + architecture docs together for persistence changes.
- verify cross-platform behavior when touching copy/open/thumbnail subprocess paths.

Do not:
- write metadata/tags into source files.
- add hidden caps in aggregate browse/search flows without explicit UX.
- move storage/indexing logic into widgets or view composition code.
- bypass centralized ignore engine with ad hoc filtering.

## 17) Quick Orientation Map for New Agents

Start reading order:
1. `AGENTS.md`
2. this file (`docs/AGENT_KNOWLEDGE_BASE.md`)
3. `docs/architecture/*` and relevant ADRs
4. `crates/librapix-app/src/main.rs`
5. crate-level files for subsystem being changed.

Hotspots by concern:
- Message/state/view orchestration: `crates/librapix-app/src/main.rs`
- Styles/tokens: `crates/librapix-app/src/ui.rs`
- Assets/branding embedding: `crates/librapix-app/src/assets.rs`
- Storage schema and APIs: `crates/librapix-storage/src/lib.rs` + `migrations/*.sql`
- Scan/index behavior: `crates/librapix-indexer/src/lib.rs`
- Search ranking: `crates/librapix-search/src/lib.rs`
- Timeline/gallery projections: `crates/librapix-projections/src/*`
- Thumbnail generation: `crates/librapix-thumbnails/src/lib.rs`

<!-- markdownlint-enable MD032 MD012 -->
