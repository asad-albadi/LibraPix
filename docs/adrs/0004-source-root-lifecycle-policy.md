# ADR 0004: Source root lifecycle policy

## Status

Accepted

## Context

Librapix needs deterministic behavior for library roots that become unavailable (offline/removable drives) without destructive side effects.

## Decision

- Introduce source-root lifecycle states in storage:
  - `active`
  - `unavailable`
  - `deactivated`
- Reconciliation checks root path existence and transitions:
  - existing path -> `active`
  - missing path -> `unavailable`
- User deactivation sets `deactivated`.
- Explicit remove deletes Librapix-owned records only.

## Alternatives considered

- Boolean active/inactive only: too weak for missing-drive semantics.
- Automatic deletion of missing roots: unsafe and destructive to user intent.

## Consequences

- Startup and indexing can reason about root availability consistently.
- Offline roots are preserved without destructive changes.
- UI and orchestration can expose explicit lifecycle controls.
