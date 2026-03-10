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
- [ ] Validation milestone
- [x] Documentation completion milestone

## Current Slice

- Implemented: branch creation from the reconciled default branch baseline.
- Implemented: architecture plan written before significant refactor work.
- Implemented: dedicated startup completion audit documenting the current runtime critical path, remaining eager work, and the corrected startup policy for finishing this branch.
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
- Implemented: startup runtime completion pass now distinguishes startup-critical work from deferred catch-up.
- Implemented: startup projection refresh prioritizes the current surface and bounds startup cache warm-up to a visible slice.
- Implemented: startup browse-tier thumbnail work is now split into startup-priority items plus delayed background catch-up.
- Implemented: ready state no longer waits for the full browse-tier thumbnail backlog.
- Implemented: route switches can trigger deferred surface refresh when startup intentionally leaves a non-visible surface for later.
- Implemented: targeted regression coverage now covers deferred thumbnail catch-up readiness, startup route deferral, and startup thumbnail prioritization.
- Implemented: final verification loop (`fmt`, `check`, `clippy`, `test`) passed after the startup completion changes.
- Partially implemented: orchestration still enters through `crates/librapix-app/src/main.rs`, even though startup/catch-up boundaries are now materially cleaner.
- Partially implemented: UI adaptation is restored for activity/status visibility and duplicate header status was removed, but full interactive product validation on a real populated library still needs manual confirmation outside this terminal environment.
- Partially implemented: smoke validation from this environment can confirm build/run invocation only; populated-library GUI behavior still requires manual confirmation.
- Planned next: manual populated-library validation before any merge to the default branch.
