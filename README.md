# LibraPix

![LibraPix logo](assets/logo/blue/icon-128.png)

**LibraPix** is a cross-platform, desktop-first, non-destructive local media gallery and manager for screenshots and recordings.

## Screenshot

![LibraPix screenshot](assets/screenshots/Screenshot%202026-03-07%20at%202.55.48%E2%80%AFPM.png)

## Status

Project phase: **MVP complete** (technical + visual shell baseline).

## Core Principles

- **Non-destructive by design**: source media is treated as read-only.
- **Documentation-driven**: architecture and repository rules are first-class deliverables.
- **Clear boundaries**: UI, application flow, domain logic, storage/indexing/search, i18n, and config remain isolated.
- **Simplicity first**: small modules, explicit state transitions, and maintainable code.

## Features

- Multiple local library directories
- Unified Add/Edit Library dialog (browse-first), including display name and chip-based root-level tag management
- Separate Library Statistics dialog with maintained per-library totals (size, media/image/video counts, size split, indexed/missing/date stats)
- Gallery and timeline views with justified layout
- Fuzzy search over filenames, tags, and metadata
- App-side and game tags
- Chip-based tag management in Library and Details surfaces (add/edit/remove with deterministic colors)
- Chip-based ignore-rule management in Settings (add/edit/remove/enable-disable with deterministic colors)
- Media type filters (images/videos) and extension chips
- Library filter chips (All libraries or selected library)
- Open file, show in folder, copy file, copy path actions
- Keyboard shortcuts: `Cmd/Ctrl+C` (copy file), `Cmd/Ctrl+Shift+C` (copy path)
- Live filesystem watching with new-file announcement dialog
- Deterministic thumbnail cache (images and videos)
- Header About dialog with product and creator information
- Header update-status chip backed by GitHub Releases latest checks
  - checks once after startup render
  - re-checks every 24 hours while app remains open
  - click-to-recheck with 5-minute cooldown
  - click opens latest release page when a newer version is available

## Build & Run

### Prerequisites

- Rust 1.85 or later (MSRV)
- FFmpeg (for video thumbnails)

### Commands

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p librapix-app
```

### Platform Notes

- **Windows**: Copy File uses native `CF_HDROP` clipboard (Explorer-paste compatible). UI icons/logo are embedded in the executable binary for release builds.
- **macOS**: Apple Silicon DMG for releases.
- **Linux**: AppImage for releases with embedded UI icons/logo.

## Workspace Layout

| Crate | Purpose |
|-------|---------|
| `librapix-app` | Iced desktop executable (presentation + app bootstrap) |
| `librapix-config` | Typed config, path strategy, TOML load/save |
| `librapix-core` | Domain and application orchestration |
| `librapix-indexer` | Indexing pipeline, centralized ignore matching |
| `librapix-i18n` | Key-based localization with locale fallback |
| `librapix-projections` | Timeline and gallery read projections |
| `librapix-search` | Replaceable search contracts, fuzzy strategy |
| `librapix-storage` | SQLite storage and migrations |
| `librapix-thumbnails` | App-owned thumbnail cache |

## MVP Usage Flow

1. Add one or more libraries from the Add/Edit Library dialog.
2. Configure ignore rules as needed.
3. Run indexing.
4. Browse gallery or timeline and select media.
5. Inspect details, attach tags, run search, use open/copy actions.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture overview](docs/architecture/overview.md)
- [Branding guidelines](docs/branding.md)

## License

MIT License. See [LICENSE](LICENSE) for details.
