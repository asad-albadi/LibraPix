# LibraPix

![LibraPix logo](assets/logo/blue/icon-128.png)

LibraPix is a cross-platform desktop application for browsing and managing local screenshots and recordings without modifying the original files.

## Why LibraPix exists

Many screenshot and clip workflows are folder-based and grow over time across multiple directories. LibraPix provides a single local view over those folders, with indexing, search, timeline browsing, tags, and thumbnail caching, while keeping source media read-only.

## Project status

LibraPix is currently in an MVP-complete baseline (`0.4.0`) focused on a usable, non-destructive desktop workflow. The codebase is actively evolving, but the main end-to-end flow is implemented.

## Current feature set

Implemented in the current codebase:

- Multiple library roots with add/edit/remove/deactivate/reactivate flows
- Optional display names per library
- Root-level app/game tags that are automatically applied during indexing
- Read-only media scanning with centralized ignore rules
- Gallery and timeline browsing surfaces
- Search over indexed media using fuzzy matching
- Metadata/details panel (type, size, modified date, dimensions, path)
- App/game tag attach/detach/edit flows in the UI
- Filtering by media type, extension, tag, and library
- Minimum file size filter for indexing
- SQLite-backed catalog and projection snapshots
- App-owned thumbnail cache for images and videos
- Filesystem watching and new-media announcement UI
- OS actions: open file, open containing folder, copy file, copy path
- Background GitHub release check (startup + periodic)

## Screenshot

![LibraPix application screenshot](assets/screenshots/Screenshot%202026-03-11%20190524.png)

## Technology stack

| Area | Technologies in use |
| --- | --- |
| Language/runtime | Rust (Edition 2024, MSRV 1.85) |
| Desktop UI | `iced` |
| Configuration | `serde`, `toml`, `directories` |
| Storage | SQLite via `rusqlite` (`bundled`), SQL migrations |
| Indexing & filesystem | `walkdir`, `globset`, `notify`, `imagesize` |
| Search | `strsim` (normalized Levenshtein strategy) |
| Imaging/thumbnails | `image`, `sha2`, system `ffmpeg` for video thumbnails |
| Platform integrations | `rfd` (native folder picker), `opener` |
| Networking | `ureq` + `serde_json` (release update check) |
| CI/release | GitHub Actions workflow for Linux AppImage and Windows `.exe` release artifacts |

## Architecture overview

LibraPix is a Rust workspace with focused crates and explicit boundaries:

- `librapix-app`: Iced application shell, state/update/view wiring, runtime orchestration
- `librapix-core`: shared app/domain state and message types
- `librapix-config`: config schema, validation, and platform path resolution
- `librapix-storage`: SQLite persistence and migrations
- `librapix-indexer`: media scanning and ignore-rule evaluation
- `librapix-search`: search interfaces and fuzzy strategy
- `librapix-projections`: gallery/timeline read projections
- `librapix-thumbnails`: deterministic thumbnail generation/cache paths
- `librapix-i18n`: keyed UI text and locale handling

## System design principles in the current implementation

- **Non-destructive behavior**: source media is scanned/read, not modified.
- **Local-first operation**: config, database, tags, ignore rules, and thumbnails are stored locally.
- **Explicit state transitions**: message-driven update flow (`Message` + update logic) in the desktop app.
- **Separation of concerns**: storage, indexing, projections, search, config, and UI are split into dedicated crates.
- **Deterministic caching**: thumbnail output paths are hash-derived from source fingerprint fields.
- **Background work for responsiveness**: indexing, projection refreshes, thumbnail work, and update checks are scheduled as background tasks.

## Project structure

```text
.
├── crates/
│   ├── librapix-app/
│   ├── librapix-config/
│   ├── librapix-core/
│   ├── librapix-i18n/
│   ├── librapix-indexer/
│   ├── librapix-projections/
│   ├── librapix-search/
│   ├── librapix-storage/
│   └── librapix-thumbnails/
├── docs/
├── assets/
└── .github/workflows/
```

## Data flow and state management

High-level runtime flow:

1. Load/create config and resolve app-owned paths.
2. Open SQLite storage and apply migrations.
3. Reconcile configured library roots and lifecycle states.
4. Scan roots through the indexer with ignore rules and optional size filtering.
5. Persist indexed rows and maintain catalog/read models.
6. Build gallery/timeline projections for the active surface.
7. Resolve/reuse/generate thumbnails from app-owned cache.
8. Render UI from current app state; user actions emit messages that trigger explicit update steps.

## Storage and persistence

LibraPix persists application state in app-owned storage only:

- `config.toml` for preferences and path overrides
- SQLite database for roots, indexed media, tags, ignore rules, statistics, catalog data, and snapshots
- Thumbnail/cache files under app cache directories
- Startup/runtime logs under app state/log directories

No app metadata is written into user media files.

## Performance-oriented choices already present

- Incremental indexing (`new` / `changed` / `unchanged` detection)
- Snapshot-assisted startup path for faster initial surface restore
- Viewport/windowed rendering strategy for large gallery/timeline surfaces
- Thumbnail reuse before generation, with background catch-up
- Filesystem watch integration to avoid full manual rescans for new media

## Platform support

- **Windows**: actively packaged in CI (`.exe` release artifact)
- **Linux**: actively packaged in CI (AppImage release artifact)
- **macOS**: codebase is cross-platform, but CI DMG packaging is currently disabled

## Installation

### Prerequisites

- Rust `1.85` or newer
- `ffmpeg` available on `PATH` (required for video thumbnail extraction)

### Clone and build

```bash
git clone <repository-url>
cd LibraPix
cargo build -p librapix-app
```

## Running the app

```bash
cargo run -p librapix-app
```

## Development checks

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Release notes

- Release artifacts are built by `.github/workflows/release.yml`.
- Current CI packaging targets: Linux AppImage and Windows executable.
- macOS DMG packaging is present in workflow logic but disabled until signing/notarization is available in CI.

## Known limitations and current scope boundaries

- Current localization is effectively `en-US` only in the shipped locale set.
- Video thumbnail generation depends on external `ffmpeg` availability.
- Update checking uses GitHub Releases latest endpoint and therefore requires network access.
- Memories-style resurfacing is listed in roadmap docs but not implemented in the current app baseline.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture overview](docs/architecture/overview.md)
- [Repository rules](docs/REPOSITORY_RULES.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

## Contributing

Contributions are welcome. Before opening a PR:

1. Read `AGENTS.md` and `docs/REPOSITORY_RULES.md`.
2. Keep changes scoped and maintain crate boundaries.
3. Update relevant docs and `CHANGELOG.md` for meaningful changes.
4. Run the development checks listed above.

## License

LibraPix is licensed under the MIT License. See [LICENSE](LICENSE).
