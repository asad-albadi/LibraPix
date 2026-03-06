# ADR 0007: Replaceable fuzzy search baseline

## Status

Accepted

## Context

Librapix needs a real search subsystem boundary that can evolve independently from UI and storage internals.

## Decision

- Introduce `librapix-search` with explicit contracts:
  - `SearchDocument`
  - `SearchQuery`
  - `SearchHit`
  - `SearchStrategy` trait
- Provide baseline `FuzzySearchStrategy` implementation:
  - term-based matching over path/filename/media kind/tags
  - exact, partial, and fuzzy (`normalized_levenshtein`) scoring
  - all query terms must match for inclusion
- Keep orchestration in app layer and avoid embedding search ranking logic in widgets.

## Alternatives considered

- Keep search in storage SQL only: less replaceable ranking evolution.
- UI-owned scoring: violates layering and testability goals.

## Consequences

- Search behavior is replaceable and testable by contract.
- Baseline fuzzy ranking is simple and explainable.
- Larger-scale search can later replace strategy internals without changing UI contracts.
