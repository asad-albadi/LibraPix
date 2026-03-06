# Timeline and Gallery Projections

Timeline and gallery views are read projections built from indexed media read models.

## Baseline projection source

- Source rows come from `librapix-storage` read-model queries (`indexed_media` + `tags` joins).
- Projection builders live in `librapix-projections`.
- UI consumes projection outputs for gallery/timeline card rendering and selection.
- App orchestration derives `available_filter_tags` from read-model rows (excluding internal `kind:*` tags) for the tag filter axis.

## Timeline baseline

- Driving date field: `modified_unix_seconds`.
- Day/month/year grouping uses local timezone conversion (user-facing local day semantics), not raw UTC calendar buckets.
- Grouping supports:
  - day
  - month
  - year
- Missing/invalid timestamps are grouped in an `unknown` bucket.
- Buckets are ordered newest-first.
- Timeline projection now exposes two read models:
  - `TimelineBucket`: grouped media rows for timeline rendering.
  - `TimelineAnchor`: lightweight navigation anchors for fast date scrub.

### Timeline anchor model

- `group_index`: stable index of the corresponding timeline bucket.
- `label`: date label used by timeline headers (`YYYY-MM-DD`, `YYYY-MM`, `YYYY`, or `unknown`).
- `year` / `month` / `day`: parsed date parts when available, `None` for unknown groups.
- `item_count`: number of media items in the group.
- `normalized_position`: stable index-based `0.0..=1.0` anchor position used for scrub mapping and programmatic scrolling.

Anchor construction is projection-driven (`build_timeline_anchors`) and does not inspect rendered widgets.

## Gallery baseline

- Projection supports filtering by:
  - media kind
  - extension
  - tag
- Sorting supports:
  - modified timestamp descending
  - path ascending
- Pagination supports `offset` and `limit`.

## Scope boundary

- Projections are read-only and do not mutate source media.
- This layer is intentionally UI-agnostic and replaceable.
- Route panels consume projection outputs as selectable media cards wired through explicit app selection state.
- Projection inputs must not be silently pre-truncated by hard-coded UI caps; gallery/timeline views consume full projected item sets.
- Fast timeline navigation is projection-backed: scrubber interactions map to `TimelineAnchor` entries, not ad-hoc view geometry probes.
