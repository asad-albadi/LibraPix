# ADR 0021: Staged Runtime Coordination and Snapshot Hydration

## Status

Accepted

## Context

The previous background runtime shape caused several product issues:

- startup frequently rebuilt too much work before browse became usable
- watcher/add-library/manual triggers could overlap and produce race-prone completion ordering
- stale background completions could overwrite fresher projection state
- thumbnail retries were coarse and could leave placeholders until a later full cycle
- add-library behavior could leave the new root hidden behind active filters

LibraPix needs a responsive, non-blocking runtime that preserves multi-root correctness and non-destructive behavior.

## Decision

- Split startup into two phases:
  - fast snapshot hydrate (`projection_snapshots`) for immediate browse-ready render
  - background reconcile scan/projection/thumbnail pipeline
- Replace monolithic background completion with staged typed jobs:
  - `HydrateSnapshotComplete`
  - `ScanJobComplete`
  - `ProjectionJobComplete`
  - `ThumbnailBatchComplete`
- Introduce generation-guarded apply paths per stage family and ignore stale completions.
- Add background coordinator state:
  - per-family in-flight flags
  - pending reconcile/projection requests with reason merge
  - watcher path aggregation
  - bounded thumbnail queue + retry state
- Persist default projection snapshots after successful projection refresh for next startup hydrate.
- Keep "All libraries" semantics explicit when add-library would otherwise remain hidden by an active root filter.

## Alternatives considered

1. Keep single large background result and rely on "last write wins":
   rejected because stale writes remained race-prone and hard to reason about.
2. Force full rebuild at startup to guarantee consistency:
   rejected because startup responsiveness degraded significantly on large libraries.
3. Introduce cancellation-only without generation guards:
   rejected because cancellation alone does not guarantee stale apply safety.

## Consequences

- Startup becomes usable earlier through persisted projection hydration.
- Reconcile/projection/thumbnail overlap is coordinated instead of implicitly racing.
- Watcher bursts and rapid UI refresh triggers coalesce predictably.
- Thumbnail reliability improves through explicit per-item states and bounded retries.
- Documentation must describe staged runtime behavior as implemented, not aspirational.
