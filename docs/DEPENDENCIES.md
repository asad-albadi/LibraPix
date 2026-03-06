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
