# Catalog-First Architecture

## Status

- Document status: implemented for planning on `feat/catalog-first-architecture`
- Branch baseline: `master` at commit `a1215cd`
- This document is the source of truth for this branch's architecture direction

## Problem Statement

LibraPix already has dedicated crates for storage, indexing, projections, search, and thumbnails, but the runtime shape still concentrates too much durable coordination in `crates/librapix-app/src/main.rs`.

Current pressure points:

- `indexed_media` is both source-facts storage and the effective browse/search read model.
- timeline grouping is computed from raw timestamps during runtime refresh instead of from an explicit persisted timeline-oriented catalog surface.
- thumbnail files exist on disk, but there is no canonical artifact catalog describing which tiers are ready, failed, or stale.
- one background worker performs scan, incremental writes, auto-tag application, thumbnail work, browse projection shaping, search hydration, and cache hydration in one app-owned flow.
- `librapix-app` still owns too much data shaping that should live in catalog/storage-oriented seams.

This shape works for MVP, but it is not a strong long-lived foundation for very large libraries or future derived work.

## Architectural Goal

Move LibraPix toward a catalog-first architecture that keeps source facts, normalized browse/search facts, and derived artifacts separate while preserving the non-destructive desktop-first model.

## Hard Constraints

- Implemented: source media remains read-only from the application's perspective.
- Implemented: all app-managed state remains in Librapix-owned storage and cache directories.
- Implemented: UI state/message/update/view separation remains explicit.
- Implemented: heavy work must stay off the UI thread.
- Planned: runtime orchestration must move out of the app crate's large monolithic flow in phases.
- Deferred: no destructive migration or one-shot rewrite of the MVP shell.

## Target Model

### 1. Source facts

Implemented baseline:

- `indexed_media` remains the durable source-facts table for filesystem-observed media.
- It owns source-root linkage, file identity, incremental metadata, and missing-file state.

Planned evolution:

- richer extracted source facts can extend `indexed_media` or adjacent source-fact tables without turning it back into the browse/search query surface.

### 2. Normalized catalog facts

Implemented in this branch:

- introduce a materialized catalog layer for normalized browse/search/timeline facts.
- the catalog stores file-name/extension splits, normalized search text, tag payload, and persisted timeline keys.

Planned evolution:

- query-specific read APIs should increasingly read from the catalog layer instead of rebuilding browse/search state from ad hoc joins.
- catalog maintenance should move from full refresh compatibility bridges toward more targeted incremental updates.

### 3. Derived artifacts

Implemented in this branch:

- introduce a derived-artifact catalog for thumbnail tiers.
- artifact records describe the kind, variant/tier, stored path, and readiness status of app-owned derivatives.

Planned evolution:

- extend the same artifact model to future derived work such as richer thumbnail pyramids, analysis jobs, and memory/resurfacing assets.

## Subsystem Responsibilities

### `librapix-storage`

Implemented:

- persists source facts, normalized catalog facts, and derived-artifact records.
- owns migrations and materialization APIs for catalog refresh.

Planned:

- own more durable query surfaces so `librapix-app` stops shaping catalog rows manually.

### `librapix-indexer`

Implemented:

- remains responsible for filesystem traversal, ignore handling, media classification, and incremental source-fact candidates.

Planned:

- stay source-facts focused and avoid absorbing projection or thumbnail policy.

### `librapix-projections`

Implemented in this branch:

- projections can consume persisted timeline keys instead of recomputing all grouping semantics from raw timestamps when those keys are available.

Planned:

- add projection/query contracts that can take more explicit catalog-oriented inputs.

### `librapix-search`

Implemented:

- remains the replaceable ranking subsystem.

Planned:

- operate over catalog materializations and, later, index-backed search layers without changing the UI contract.

### `librapix-thumbnails`

Implemented:

- remains the file-generation subsystem for thumbnails.

Implemented in this branch:

- generated thumbnails are now expected to be recorded as derived artifacts with explicit variant names.

Planned:

- evolve toward a canonical thumbnail pyramid strategy with explicit eager/on-demand policy per tier.

### `librapix-app`

Implemented:

- remains the Iced shell, message loop, and product-facing interaction layer.

Planned:

- shed catalog shaping and background coordination responsibilities in phases.
- move toward smaller app-facing orchestration seams instead of a giant runtime worker inside `main.rs`.

## Storage and Catalog Direction

Implemented in this branch:

- keep `indexed_media` as the source-facts baseline.
- add `media_catalog` as the normalized browse/search/timeline surface.
- add `derived_artifacts` for app-owned thumbnail metadata.

Planned:

- move browse/search hydration to catalog-first queries.
- retain migration-based evolution only; no reset-based schema changes.

Intentionally deferred:

- replacing `indexed_media` outright.
- introducing a separate FTS or alternate search engine in this slice.
- adding large numbers of narrowly-scoped projection tables before the normalized catalog shape is proven.

## Timeline Indexing Direction

Implemented in this branch:

- persist local-day, local-month, and local-year keys as catalog fields.
- allow projections to consume those keys as explicit timeline indexing data.

Partially implemented:

- runtime still assembles timeline buckets in memory from catalog rows.

Deferred:

- persisted aggregate timeline bucket tables.
- per-filter or per-root precomputed timeline rollups.

Rationale:

- persisted per-media timeline keys establish the durable indexing seam first without committing to an aggregate-table design that may conflict with later filter combinatorics.

## Thumbnail Architecture Direction

Implemented in this branch:

- define canonical named variants for thumbnails as derived artifacts.
- record readiness/failure for generated thumbnail tiers in storage.

Partially implemented:

- the branch still uses two concrete tiers:
  - eager browse tier
  - on-demand detail tier

Deferred:

- deeper thumbnail pyramids beyond the current named tiers.
- background recovery/backfill policies for all possible artifact gaps.

## Search and Query Direction

Implemented in this branch:

- search documents can be built from normalized catalog rows instead of raw source-fact joins.

Planned:

- migrate more query flows toward storage/catalog surfaces with explicit contracts.
- evaluate index-backed search only after catalog semantics are stable.

Deferred:

- FTS adoption.
- memories/resurfacing query materialization.

## Job and Runtime Orchestration Direction

Implemented baseline:

- heavy work continues to run off-thread via `Task::perform`.

Planned:

- split the current monolithic worker into clearer job families after the catalog seam is in place.
- move durable pipeline responsibilities out of `librapix-app/src/main.rs`.

Implemented as runtime reconciliation on this branch:

- startup/runtime activity is now staged again instead of being cleared by a single monolithic background result.
- snapshot hydrate, reconcile, projection, and thumbnail batches now surface explicit runtime stages to the shell.
- startup/runtime policy now distinguishes startup-critical work from deferred catch-up:
  - startup projection prioritizes the visible route
  - startup cache warm-up is bounded
  - full browse-tier thumbnail backlog no longer blocks ready-enough state
  - deferred thumbnail catch-up continues after the shell becomes usable again

Deferred:

- full staged coordinator replacement in this branch slice.
- streaming or cancellation-heavy runtime redesign before the catalog/data model is stabilized on `master` baseline behavior.

## Migration Phases

### Phase 0: baseline and branch

- Implemented: create `feat/catalog-first-architecture` from the repository default branch.
- Implemented: document current constraints and pressure points.

### Phase 1: architecture documentation

- Implemented: this plan document.
- Implemented: workstream checklist.
- Implemented: ADR for catalog-first storage direction.

### Phase 2: catalog/data-model foundation

- Implemented in this branch slice:
  - normalized catalog table
  - derived-artifact table
  - migration-backed materialization APIs
  - projection/search pipeline consumption of catalog rows

### Phase 3: runtime/orchestration restructuring

- Partially implemented in this branch slice:
  - app runtime uses stronger storage seams and explicit staged startup/runtime jobs.
  - activity/ready-state transitions are reconnected to those staged jobs.
  - top-level orchestration still remains in `crates/librapix-app/src/main.rs`.

### Phase 4: timeline indexing foundation

- Partially implemented in this branch slice:
  - explicit persisted timeline keys
  - projection support for those keys

### Phase 5: thumbnail architecture evolution

- Partially implemented in this branch slice:
  - named artifact variants and persistence

### Phase 6: search/projection evolution

- Partially implemented in this branch slice:
  - catalog-backed query preparation

### Phase 7: UI/runtime adaptation

- Planned:
  - only adapt the shell when required by the new data/runtime seams.

## Risks

- catalog refresh can become too expensive if it remains a broad compatibility rebuild for too long.
- artifact records can drift from disk contents if failure and cleanup policies are not explicit.
- persisted local-time timeline keys depend on honest local-time semantics and need tests around boundary cases.
- runtime/product regressions can still occur if future catalog refactors bypass the staged activity/state machine while changing data-preparation seams.
- the branch can still accumulate too much app-level orchestration if later phases stop at "better storage" and never complete the runtime split.

## Rollback and Fallback

- all schema changes are additive and migration-based.
- existing `indexed_media` source facts remain intact, so browse/search can fall back to the pre-catalog read path if a later milestone must be reverted.
- timeline persisted keys are a compatibility addition, not a destructive replacement of raw timestamps.
- derived artifacts remain rebuildable cache metadata; removing their catalog layer would not endanger user media.

## Intentionally Deferred in This Branch

- search-engine replacement or FTS adoption
- memories/resurfacing subsystem
- persisted aggregate timeline rollups
- full staged runtime coordinator redesign
- aggressive UI shell churn
- any behavior that weakens the non-destructive guarantee

## Validation Standard

Every milestone in this branch must keep:

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- app smoke-run when runtime behavior is affected
