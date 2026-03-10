# Timeline and Gallery Projections

Timeline and gallery views are read projections built from normalized catalog rows.

## Baseline projection source

- Source rows now come from `librapix-storage` catalog queries (`media_catalog`).
- Projection builders live in `librapix-projections`.
- UI consumes projection outputs for gallery/timeline card rendering and selection.
- App orchestration derives `available_filter_tags` from catalog rows (excluding internal `kind:*` tags) for the tag filter axis.

## Timeline baseline

- Driving date field: persisted timeline keys when available, otherwise `modified_unix_seconds`.
- `media_catalog` stores local `timeline_day_key`, `timeline_month_key`, and `timeline_year_key`.
- Projection falls back to local-time conversion from raw timestamps only when persisted keys are unavailable.
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
- `normalized_position`: structure-weighted `0.0..=1.0` anchor position derived from ordered bucket sizes, used as the shared source for scrub mapping, marker placement, and scroll targeting.

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
- Aggregate persisted timeline rollups are deferred; this branch establishes persisted per-media timeline keys first.
