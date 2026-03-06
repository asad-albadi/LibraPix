# Timeline and Gallery Projections

Timeline and gallery views are read projections built from indexed media read models.

## Baseline projection source

- Source rows come from `librapix-storage` read-model queries (`indexed_media` + `tags` joins).
- Projection builders live in `librapix-projections`.
- UI consumes projection outputs only for validation previews.

## Timeline baseline

- Driving date field: `modified_unix_seconds`.
- Grouping supports:
  - day
  - month
  - year
- Missing/invalid timestamps are grouped in an `unknown` bucket.
- Buckets are ordered newest-first.

## Gallery baseline

- Projection supports filtering by:
  - media kind
  - tag
- Sorting supports:
  - modified timestamp descending
  - path ascending
- Pagination supports `offset` and `limit`.

## Scope boundary

- Projections are read-only and do not mutate source media.
- This layer is intentionally UI-agnostic and replaceable.
- Route panels consume projection outputs as selectable media entries wired through explicit app selection state.
