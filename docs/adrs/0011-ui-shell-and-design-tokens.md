# ADR 0011: App shell and design-token baseline

## Status

Accepted

## Context

Librapix had feature-complete behavior but UI presentation resembled an internal debug surface instead of a media-first desktop product.

## Decision

- Adopt an app-shell layout:
  - header
  - sidebar
  - main media pane
  - details pane
- Centralize baseline visual tokens/helpers in `librapix-app/src/ui.rs`.
- Keep selection explicit in app state and unify details/actions around selected media.
- Keep management controls in sidebar so browsing remains content-first.

## Alternatives considered

- Keep stacked single-column controls and add minor polish: rejected as insufficient product UX.
- Introduce a heavy custom widget framework: rejected as unnecessary complexity for MVP.

## Consequences

- UI becomes structurally coherent and media-oriented.
- Styling choices remain consistent and maintainable.
- Further UI iterations can build on a stable shell/token foundation.
