# ADR 0010: Media details, tags, and actions baseline

## Status

Accepted

## Context

MVP needs actionable media workflows beyond indexing/search: inspect metadata, manage tags, and launch practical file actions.

## Decision

- Extend storage read-model API with media-by-id lookup.
- Add storage tag operations:
  - attach tag by name
  - detach tag by name
  - list tags
- Keep tag kinds explicit (`app`, `game`) with tag rows in app-managed storage.
- Implement app-orchestrated actions:
  - open selected file
  - open containing folder
  - copy selected file as a file-object clipboard payload
  - copy selected path via platform commands

## Alternatives considered

- UI-owned DB queries and direct file operations: rejected due to layering.
- Writing tags into source files: rejected by non-destructive guarantee.
- Deferring all actions to future phase: rejected due to MVP usability requirements.

## Consequences

- MVP gains usable details/tags/action workflows without violating source-file safety.
- Windows file-object copy uses native CF_HDROP clipboard payload semantics; other platforms use host-native command integrations.
- Future UX can replace selected-media-id input flow with richer selection state while keeping storage/action boundaries.
