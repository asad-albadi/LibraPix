# Catalog-First Architecture Checklist

- Branch: `feat/catalog-first-architecture`
- Source baseline: repository default branch `master` (the repo does not currently have a `main` branch)

## Milestones

- [x] Architecture-doc milestone
- [x] Storage/catalog milestone
- [ ] Orchestration milestone
- [x] Timeline milestone
- [x] Thumbnail milestone
- [x] Search/projection milestone
- [ ] UI adaptation milestone
- [x] Validation milestone
- [x] Documentation completion milestone

## Current Slice

- Implemented: branch creation from the reconciled default branch baseline.
- Implemented: architecture plan written before significant refactor work.
- Implemented: additive migration `0009_catalog_first_foundation.sql` introducing `media_catalog` and `derived_artifacts`.
- Implemented: storage catalog materialization and derived-artifact query APIs.
- Implemented: background browse/search/timeline preparation now consumes catalog rows.
- Implemented: persisted timeline keys and named thumbnail variants (`gallery-400`, `detail-800`).
- Implemented: validation loop (`fmt`, `check`, `clippy`, `test`) passed and the app was smoke-run to launch.
- Partially implemented: orchestration still enters through `crates/librapix-app/src/main.rs`, though it now leans on stronger storage seams.
- Planned next: extract background/runtime coordination further out of the app crate and define the next catalog-driven job boundaries.
