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
- Implemented: additive compatibility migrations `0010_projection_snapshots.sql` and `0011_catalog_history_reconciliation.sql` so existing real databases from the older runtime line receive both projection-snapshot support and catalog/artifact tables.
- Implemented: storage catalog materialization and derived-artifact query APIs.
- Implemented: background browse/search/timeline preparation now consumes catalog rows.
- Implemented: persisted timeline keys and named thumbnail variants (`gallery-400`, `detail-800`).
- Implemented: startup/runtime regression fix for the catalog-first branch:
  - restored staged startup activity transitions (`snapshot hydrate -> reconcile -> projection -> thumbnail batches`)
  - removed the silent monolithic `BackgroundWorkComplete` flow
  - kept ready-state transitions honest until all staged work is idle
- Implemented: targeted regression tests for startup activity state, ready-state finalization, and thumbnail-stage handoff.
- Implemented: validation loop (`fmt`, `check`, `clippy`, `test`) passed after the runtime reconciliation work.
- Partially implemented: orchestration still enters through `crates/librapix-app/src/main.rs`, but it now uses explicit staged jobs instead of one silent background result apply path.
- Partially implemented: UI adaptation is restored for activity/status visibility, but full interactive product validation on a real populated library still needs manual confirmation outside this terminal environment.
- Planned next: extract the staged coordinator further out of the app crate and formalize the next catalog-driven job boundaries without regressing runtime visibility again.
