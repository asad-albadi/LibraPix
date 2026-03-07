# ADR 0020: Chip-based tag and ignore-rule interactions

## Status

Accepted

## Context

Tag and ignore-rule interactions across Library Edit, Details, and Settings had become list/text driven and operational rather than product-like. The UX needed a consistent, compact, and visual interaction model without pushing business logic into widgets.

## Decision

- Introduce a reusable managed-chip presentation model in `librapix-app` with centralized styles and deterministic color mapping.
- Use deterministic string hashing against a predefined dark-theme palette so chip colors are stable across sessions.
- Apply the same chip interaction surface to:
  - Library dialog root tags
  - Details media tags
  - Settings ignore rules
- Keep app-side orchestration explicit:
  - widgets emit messages
  - update handlers call storage APIs
  - views remain presentation focused
- Keep inherited vs manual tags understandable in Details by rendering inherited tags as a distinct chip group.
- Preserve ignore-rule enable/disable semantics while adding explicit chip-level edit/remove flows.

## Alternatives considered

- Keep existing text-list forms and only style them: rejected because interaction quality still feels raw and inconsistent.
- Introduce a new cross-crate tag/rule subsystem: rejected as unnecessary scope for this milestone.
- Random chip colors: rejected because colors would shift per render/session and hurt recognition.

## Consequences

- Tag/rule interactions are consistent and visually organized across the app.
- Chip styling and color logic are centralized, reducing duplication and drift.
- Storage required small query/mutation extensions (`list_media_tags`, `delete_ignore_rule_by_id`) to support clean chip edit/remove flows.
- Manual visual QA remains important for spacing/hover polish because these flows are primarily UX-facing.
