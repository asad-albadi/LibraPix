# ADR 0021: Catalog-First Architecture Foundation

## Status

Accepted

## Context

LibraPix MVP established `indexed_media` as both the durable source-facts table and the effective browse/search read model. That was acceptable for the initial product baseline, but it leaves several architectural gaps:

- browse/search/timeline query data is not clearly separated from source facts
- thumbnail cache files exist without a durable artifact catalog
- timeline grouping semantics are recomputed from raw timestamps during runtime refresh
- `librapix-app` still owns too much data shaping and background coordination

The next long-lived architecture branch needs a safer transition toward a catalog-first design without replacing the entire app at once.

## Decision

Adopt a three-layer data model:

1. `indexed_media` remains the durable source-facts table.
2. `media_catalog` becomes the normalized browse/search/timeline facts table.
3. `derived_artifacts` records app-owned generated outputs such as thumbnail tiers.

For timeline readiness:

- persist local day/month/year keys in `media_catalog`
- allow projections to consume persisted keys when available

For thumbnail readiness:

- record thumbnail variants as named derived artifacts instead of relying on disk presence alone

For migration safety:

- keep all changes additive and migration-based
- retain fallback paths through existing source-fact data while the catalog layer matures

## Alternatives Considered

### 1. Keep using `indexed_media` as the only read model

Rejected because it keeps source facts, normalized query facts, and derivative readiness collapsed together.

### 2. Introduce many specialized projection tables immediately

Rejected because the query semantics are not stable enough yet, and it would create schema sprawl before the normalized catalog shape is proven.

### 3. Rewrite the runtime and storage model in one pass

Rejected because it would risk product regressions and make rollback difficult.

## Consequences

- LibraPix gets a durable transition path toward catalog-first query behavior.
- Source facts remain intact and understandable during the migration.
- Thumbnail policy becomes more explicit and extensible.
- Timeline grouping gains an explicit persisted seam without forcing an early aggregate-table design.
- Some temporary compatibility bridges remain acceptable while the app moves off direct source-fact shaping.
