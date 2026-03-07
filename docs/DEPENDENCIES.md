# Dependencies

This file tracks major direct dependencies that shape architecture and maintenance.

## `iced` (0.14.0)

- Purpose: Cross-platform desktop UI framework for Librapix.
- Why chosen: Native Rust GUI, explicit state/message/update/view model, suitable for long-term desktop architecture boundaries.
- Alternatives considered:
  - `egui`: very productive immediate-mode UI, but less aligned with the explicit flow we want for strict architectural separation.
  - `slint`: strong UI tooling, not selected for the current baseline.
- Official docs consulted:
  - [https://docs.rs/crate/iced/latest](https://docs.rs/crate/iced/latest)
  - [https://docs.iced.rs/iced/](https://docs.iced.rs/iced/)
  - [https://docs.rs/iced/latest/iced/widget/operation/scrollable/fn.scroll_to.html](https://docs.rs/iced/latest/iced/widget/operation/scrollable/fn.scroll_to.html)
  - [https://docs.rs/iced/latest/iced/widget/scrollable/fn.scroll_to.html](https://docs.rs/iced/latest/iced/widget/scrollable/fn.scroll_to.html)
  - [https://github.com/iced-rs/iced/releases](https://github.com/iced-rs/iced/releases)
- Notes:
  - Latest stable verified at baseline: `0.14.0`.
  - Timeline scrubber uses `operation::scroll_to` for absolute offset targeting with `operation::snap_to` fallback during early viewport initialization.
  - Keep presentation logic in `librapix-app` and prevent leakage into domain/storage.
- Risks/tradeoffs:
  - API evolution can require incremental refactors.
  - Advanced Rust knowledge is required for smooth development.

## Rust workspace tooling (Cargo)

- Purpose: Multi-crate repository structure with shared lockfile, shared target dir, and explicit dependency direction.
- Why chosen: Keeps crate boundaries clear and testable as the product grows.
- Official docs consulted:
  - [https://doc.rust-lang.org/cargo/reference/workspaces.html](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- Notes:
  - Workspace uses resolver `3`.
  - Shared package metadata includes MSRV and edition.

## Rust toolchain baseline

- Purpose: Define language/runtime baseline and edition guarantees.
- Why chosen: Rust 2024 edition requires an explicit compatible minimum compiler.
- Official docs consulted:
  - [https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- Notes:
  - MSRV set to `1.85` for this baseline.

## `serde` (1.0.228)

- Purpose: Typed serialization/deserialization for config models.
- Why chosen: Stable ecosystem standard with derive support and strong compatibility across Rust formats.
- Alternatives considered:
  - Hand-rolled parsing: unnecessary complexity and weaker safety.
- Official docs consulted:
  - [https://serde.rs/](https://serde.rs/)
  - [https://docs.rs/crate/serde/latest](https://docs.rs/crate/serde/latest)
- Notes:
  - Used in `librapix-config` for config schema modeling.
- Risks/tradeoffs:
  - Schema changes must be versioned and documented to avoid deserialization breakage.

## `toml` (1.0.4)

- Purpose: Parse and serialize `config.toml`.
- Why chosen: TOML is human-readable and already familiar in Rust ecosystems.
- Alternatives considered:
  - JSON/YAML: workable, but TOML better matches repo ergonomics and expected manual editing style.
- Official docs consulted:
  - [https://docs.rs/toml/latest/toml/](https://docs.rs/toml/latest/toml/)
- Notes:
  - `to_string_pretty` is used for predictable formatting.
- Risks/tradeoffs:
  - Manual edits can still produce invalid files; validation and clear errors are required.

## `directories` (6.0.0)

- Purpose: Resolve platform-specific config/data/cache directories.
- Why chosen: Minimal cross-platform API with explicit project directory helpers.
- Alternatives considered:
  - Hardcoded platform paths: fragile and not maintainable.
- Official docs consulted:
  - [https://docs.rs/directories/latest/directories/](https://docs.rs/directories/latest/directories/)
- Notes:
  - `ProjectDirs` is used to compute config/data/cache defaults.
- Risks/tradeoffs:
  - Directory conventions differ by platform; docs must define behavior clearly.

## `rusqlite` (0.38.0, `bundled` feature)

- Purpose: SQLite access layer for Librapix-managed persistence.
- Why chosen: Direct SQLite wrapper, small dependency footprint, good fit for desktop local state.
- Alternatives considered:
  - `sqlx`: richer abstraction, but unnecessary complexity for current local embedded scope.
  - `diesel`: strong ORM/migrations, heavier model and boilerplate than needed now.
- Official docs consulted:
  - [https://docs.rs/rusqlite/latest/rusqlite/](https://docs.rs/rusqlite/latest/rusqlite/)
- Notes:
  - `bundled` feature avoids system SQLite dependency variance across platforms.
  - Used in `librapix-storage` with SQL migrations and `schema_migrations` tracking.
- Risks/tradeoffs:
  - Bundled SQLite increases compile time.
  - Raw SQL requires disciplined migration/version management.

## `globset` (0.4.18)

- Purpose: Centralized glob-based ignore-rule matching for indexing.
- Why chosen: Fast compiled matching for multiple glob rules over many file paths.
- Alternatives considered:
  - `glob`: simpler one-pattern matching, weaker fit for grouped ignore-rule evaluation.
- Official docs consulted:
  - [https://docs.rs/globset/latest/globset/](https://docs.rs/globset/latest/globset/)
- Notes:
  - Used in `librapix-indexer::IgnoreEngine`.
- Risks/tradeoffs:
  - Invalid glob patterns must be surfaced clearly to avoid silent misconfiguration.

## `walkdir` (2.5.0)

- Purpose: Recursive filesystem traversal for indexing scans.
- Why chosen: Efficient cross-platform directory walking with robust iterator controls.
- Alternatives considered:
  - manual `std::fs` recursion: more error-prone and repetitive.
- Official docs consulted:
  - [https://docs.rs/walkdir/latest/walkdir/](https://docs.rs/walkdir/latest/walkdir/)
- Notes:
  - Scans run with symlink following disabled by default.
- Risks/tradeoffs:
  - Deep directory traversal can be expensive; future tuning may be required for very large libraries.

## `imagesize` (0.14.0)

- Purpose: Read image width/height quickly for indexing metadata baseline.
- Why chosen: Header-based dimension probing without full image decoding.
- Alternatives considered:
  - `image`: powerful decoder stack, heavier than needed for baseline dimensions-only extraction.
- Official docs consulted:
  - [https://docs.rs/imagesize/latest/imagesize/](https://docs.rs/imagesize/latest/imagesize/)
- Notes:
  - Used for image-only dimensions; video metadata remains deferred in this phase.
- Risks/tradeoffs:
  - Not a full metadata parser; richer extraction requires future subsystem expansion.

## `strsim` (0.11.1)

- Purpose: Baseline fuzzy similarity scoring for replaceable search strategy.
- Why chosen: Small focused crate with normalized Levenshtein/Jaro metrics suitable for simple, explainable ranking.
- Alternatives considered:
  - hand-rolled fuzzy scoring: unnecessary complexity and higher bug risk.
- Official docs consulted:
  - [https://docs.rs/strsim/latest/strsim/](https://docs.rs/strsim/latest/strsim/)
- Notes:
  - Current baseline uses `normalized_levenshtein`.
- Risks/tradeoffs:
  - In-memory fuzzy scoring can become expensive at larger scales and may require indexed search later.

## `chrono` (0.4.44)

- Purpose: Convert indexed timestamps into timeline projection buckets.
- Why chosen: Stable date/time primitives with straightforward local/UTC conversion for user-facing day grouping.
- Alternatives considered:
  - manual timestamp math: less readable and easier to get wrong for calendar grouping.
- Official docs consulted:
  - [https://docs.rs/chrono/latest/chrono/](https://docs.rs/chrono/latest/chrono/)
- Notes:
  - Timeline grouping uses local timezone day boundaries derived from indexed Unix timestamps.
- Risks/tradeoffs:
  - Local timezone behavior can vary around DST/offset transitions; projection tests cover boundary scenarios.

## `image` (0.25.9)

- Purpose: Decode source images and render thumbnail cache outputs.
- Why chosen: Mature Rust-native decoding/encoding stack with straightforward thumbnail operations.
- Alternatives considered:
  - custom decoder stack: unnecessary complexity and lower maintainability.
- Official docs consulted:
  - [https://docs.rs/image/latest/image/](https://docs.rs/image/latest/image/)
- Notes:
  - Baseline uses image thumbnails via Lanczos3 resampling; video thumbnails use system `ffmpeg`.
- Risks/tradeoffs:
  - Decoder support breadth can increase compile times and binary size.

## `sha2` (0.10.9)

- Purpose: Deterministic thumbnail cache-key hashing.
- Why chosen: Widely used hashing crate with stable one-shot and incremental APIs.
- Alternatives considered:
  - ad-hoc hashing: weaker portability and higher collision risk.
- Official docs consulted:
  - [https://docs.rs/sha2/latest/sha2/](https://docs.rs/sha2/latest/sha2/)
- Notes:
  - Baseline uses SHA-256 digest of source path and fingerprint fields.
- Risks/tradeoffs:
  - Cryptographic hashing is slightly heavier than non-crypto hashes, but acceptable for baseline cache keying.

## `rfd` (0.15)

- Purpose: Cross-platform native file/folder dialog for library root management.
- Why chosen: Mature Rust-native crate providing system-native open/save/folder dialogs on macOS, Windows, and Linux.
- Alternatives considered:
  - `native-dialog`: similar purpose, less actively maintained and lower download count.
  - `tinyfiledialogs`: C library wrapper, weaker Rust integration and maintainability.
  - manual path typing only: poor UX for a desktop media application.
- Official docs consulted:
  - [https://docs.rs/rfd/latest/rfd/](https://docs.rs/rfd/latest/rfd/)
  - [https://github.com/PolyMeilex/rfd](https://github.com/PolyMeilex/rfd)
- Notes:
  - Synchronous `FileDialog::pick_folder()` is used since the OS dialog is naturally modal.
  - On macOS uses Cocoa dialogs, on Windows uses COM, on Linux uses GTK3 or xdg-desktop-portal.
- Risks/tradeoffs:
  - On Linux, requires either GTK3 or xdg-desktop-portal runtime support.
  - Synchronous API blocks the main thread during dialog interaction, which is acceptable for modal folder selection.

## `notify` (8.2.0)

- Purpose: Cross-platform filesystem change watching for live indexing refresh.
- Why chosen: Mature Rust crate with native backends and explicit recursive directory watch support.
- Alternatives considered:
  - polling-only refresh loops: less responsive and less efficient.
  - custom OS-specific watchers: unnecessary complexity and maintenance cost.
- Official docs consulted:
  - [https://docs.rs/notify/latest/notify/](https://docs.rs/notify/latest/notify/)
  - [https://docs.rs/notify/latest/notify/fn.recommended_watcher.html](https://docs.rs/notify/latest/notify/fn.recommended_watcher.html)
- Notes:
  - `recommended_watcher` is used with `RecursiveMode::Recursive` for active source roots.
  - Events are forwarded through async `iced::futures::channel::mpsc` for integration with Iced subscriptions.
- Risks/tradeoffs:
  - Some network or pseudo filesystems may not emit reliable native events.
  - Event behavior can vary by editor/save strategy and platform backend.

## `opener` (0.7)

- Purpose: Open URLs (e.g. GitHub repository) in the system default browser from the app header.
- Why chosen: Lightweight, cross-platform, respects `$BROWSER` on Unix.
- Alternatives considered:
  - `webbrowser`: similar purpose, slightly different API.
  - `open`: alternative crate with comparable functionality.
- Official docs consulted:
  - [https://docs.rs/opener/latest/opener/](https://docs.rs/opener/latest/opener/)
- Notes:
  - Used for the GitHub link button in the app header.
- Risks/tradeoffs:
  - Requires a configured default browser on the system.

## `ffmpeg` (system dependency, optional)

- Purpose: Extract representative video frames for thumbnail generation.
- Why chosen: Universal video processing tool available on all desktop platforms; avoids heavy Rust video decoding dependencies.
- Alternatives considered:
  - `ffmpeg-next` Rust crate: full FFmpeg bindings, but heavy C dependency and complex build requirements.
  - `gstreamer` Rust crate: powerful but heavy and platform-variable.
  - No video thumbnails: poor UX for a media manager.
- Official docs consulted:
  - [https://ffmpeg.org/ffmpeg.html](https://ffmpeg.org/ffmpeg.html)
- Notes:
  - Invoked via `std::process::Command`; no Rust crate dependency.
  - Extracts a single frame at 1 second with `scale` filter for size control.
  - Failure is graceful: videos without thumbnails show a placeholder.
- Risks/tradeoffs:
  - Requires `ffmpeg` installed on the user's system and accessible in PATH.
  - Process invocation has startup overhead per video, acceptable for batch indexing.

## `windows-sys` (0.61.2, Windows target only)

- Purpose: Access native Win32 clipboard APIs for reliable Explorer-compatible file copy (`CF_HDROP`).
- Why chosen: Direct Win32 API access removes PowerShell host/runtime variability and enables explicit ownership/error handling around clipboard and global memory operations.
- Alternatives considered:
  - `powershell -STA` + .NET clipboard API: rejected for runtime indirection and weaker guarantee around actual clipboard payload success.
  - external clipboard crates: deferred to keep dependency surface minimal and behavior explicit.
- Official docs consulted:
  - [https://learn.microsoft.com/en-us/windows/win32/shell/clipboard](https://learn.microsoft.com/en-us/windows/win32/shell/clipboard)
  - [https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats)
  - [https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata)
  - [https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_powershell_exe](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_powershell_exe)
  - [https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/set-clipboard](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/set-clipboard)
  - [https://docs.rs/windows-sys/latest/windows_sys/Win32/System/DataExchange/fn.SetClipboardData.html](https://docs.rs/windows-sys/latest/windows_sys/Win32/System/DataExchange/fn.SetClipboardData.html)
- Notes:
  - `Copy File` on Windows now writes a real `CF_HDROP` payload with Win32 `SetClipboardData`.
  - Payload is built as a UTF-16 multi-string file list with `DROPFILES` header.
  - `Copy Path` remains text clipboard via `clip`.
- Risks/tradeoffs:
  - Includes `unsafe` Win32 interop in a narrow, platform-gated path that must remain well-tested and documented.
