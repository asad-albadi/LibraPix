# Repository-Level Engineering Rules

This document defines repository-wide implementation constraints for Librapix.

## 1. MSRV Policy

- Current MSRV: `1.85`.
- MSRV must be documented in:
  - `README.md`
  - workspace/root `Cargo.toml`
  - relevant contributor or CI docs when introduced
- Any MSRV increase must include:
  - justification
  - impact notes
  - `CHANGELOG.md` update
  - ADR when architecturally significant

## 2. Dependency Documentation Policy

- Major dependencies must be documented in `docs/DEPENDENCIES.md`.
- Every entry must include:
  - purpose
  - reason for selection
  - alternatives considered (when relevant)
  - official documentation consulted
  - usage notes and risks/tradeoffs
- New major dependencies must not be added without consulting official docs first.

## 3. Workspace and Boundary Policy

- Librapix uses a Rust workspace.
- Crate responsibilities and dependency direction must stay explicit.
- UI/framework concerns must not leak into domain/storage/indexing logic.

## 4. Verification Policy

Meaningful changes must run the baseline verification loop:

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- app startup smoke test for milestone-level changes

## 5. Documentation and Changelog Policy

- Meaningful structural changes require same-stream docs updates.
- `CHANGELOG.md` must be updated for non-trivial changes.
- New recurring issues must be recorded in `docs/TROUBLESHOOTING.md`.