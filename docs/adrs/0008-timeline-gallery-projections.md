# ADR 0008: Timeline and gallery projection baseline

## Status

Accepted

## Context

Librapix requires read projections for browsing experiences (timeline/gallery) without coupling projection logic to UI widgets.

## Decision

- Introduce `librapix-projections` crate for read-only projection logic.
- Timeline projection:
  - uses `modified_unix_seconds` as baseline date driver
  - supports day/month/year grouping
  - groups missing timestamps into `unknown`
- Gallery projection:
  - supports media-kind and tag filtering
  - supports modified-desc and path-asc sorting
  - supports offset/limit pagination

## Alternatives considered

- Build projections directly in UI update/view logic: rejected due to layering concerns.
- Persist precomputed projection tables now: deferred to avoid premature complexity.

## Consequences

- Projection behavior is explicit, testable, and UI-agnostic.
- Timeline/gallery behavior can evolve independently from storage schema.
- Future memories and richer gallery policies can build on clear projection boundaries.
