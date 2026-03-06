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
  - [https://github.com/iced-rs/iced/releases](https://github.com/iced-rs/iced/releases)
- Notes:
  - Latest stable verified at baseline: `0.14.0`.
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
- Why chosen: Stable date/time primitives with straightforward UTC timestamp conversion.
- Alternatives considered:
  - manual timestamp math: less readable and easier to get wrong for calendar grouping.
- Official docs consulted:
  - [https://docs.rs/chrono/latest/chrono/](https://docs.rs/chrono/latest/chrono/)
- Notes:
  - Baseline uses UTC timestamp conversion only; timezone-specific timeline behavior is deferred.
- Risks/tradeoffs:
  - Local timezone-aware grouping is intentionally deferred and may change bucket semantics in future phases.

## `image` (0.25.9)

- Purpose: Decode source images and render thumbnail cache outputs.
- Why chosen: Mature Rust-native decoding/encoding stack with straightforward thumbnail operations.
- Alternatives considered:
  - custom decoder stack: unnecessary complexity and lower maintainability.
- Official docs consulted:
  - [https://docs.rs/image/latest/image/](https://docs.rs/image/latest/image/)
- Notes:
  - Baseline uses image thumbnails only; video thumbnails remain deferred.
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
