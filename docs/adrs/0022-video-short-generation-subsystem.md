# ADR 0022: Video Short Generation Subsystem (`librapix-video-tools`)

## Status
Accepted

## Context

Issue `#18` adds a first-version `Make Short` feature for video items.
The feature must preserve strict script parity with an existing PowerShell implementation while keeping UI responsive and preserving architecture boundaries.

The previous codebase had no dedicated subsystem for ffprobe/ffmpeg command construction and video-short validation/probing flow.
Placing this logic directly in `librapix-app` would mix process orchestration with presentation and weaken maintainability.

## Decision

Create a new workspace crate: `crates/librapix-video-tools`.

Responsibilities:

- request/options models and enums
- script-parity validation rules
- ffprobe duration probing
- crop/effect/speed/fade filter construction
- audio atempo chain construction
- ffmpeg argument construction
- ffmpeg process execution wrapper
- typed error surface for app orchestration

`librapix-app` remains responsible for:

- showing `Make Short` only for videos
- dialog state + hover help + warning rendering
- dispatching background tasks with `Task::perform`
- mapping background result states to dialog UI

## Alternatives considered

1. Keep all logic inside `librapix-app/src/main.rs`
- Rejected: violates separation of concerns and would make script-parity logic harder to test.

2. Reuse `librapix-thumbnails`
- Rejected: thumbnail extraction and short generation are different responsibilities and lifecycle concerns.

## Consequences

Positive:
- process-heavy logic is isolated, testable, and reusable.
- UI stays orchestration-focused.
- script parity can be validated with targeted crate tests.

Tradeoffs:
- one additional crate to maintain.
- app integration requires explicit request/result mapping and dialog-state coordination.
