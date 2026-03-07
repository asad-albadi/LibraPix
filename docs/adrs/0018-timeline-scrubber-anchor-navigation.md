# ADR 0018: Projection-Driven Timeline Scrubber Navigation

## Status

Accepted

## Context

Librapix timeline browsing works for moderate libraries, but large datasets (thousands to tens of thousands of items) make manual scrolling inefficient.

We need a fast date navigator that:
- jumps quickly across timeline groups
- remains stable as data grows
- avoids coupling navigation logic to rendered widget geometry
- stays aligned with Iced's explicit state/message/update/view model

The existing timeline projection already groups by date buckets. That grouping is the correct source of truth for scrub navigation.

Follow-up validation uncovered two correctness issues:
- detached/evenly-stacked year labels did not reflect real anchor positions in the scrubber track
- index-only snap mapping made scrub movement feel sticky/non-continuous during drag + viewport sync
- UTC date bucketing caused local-midnight files to appear under the previous day

## Decision

### Anchor model in projections

Add a projection-level timeline anchor model (`TimelineAnchor`) derived from timeline buckets:
- stable `group_index`
- `label`
- date parts (`year`, `month`, `day`)
- `item_count`
- structure-weighted `normalized_position` in `0.0..=1.0`, derived from ordered bucket sizes

Anchors are produced by `build_timeline_anchors` in `librapix-projections`, not by inspecting rendered widgets.

Timeline date buckets are derived from `modified_unix_seconds` using local timezone conversion before day/month/year extraction.

### Timeline-mode scrubber in app layer

Add a right-side scrubber control in Timeline mode only:
- drag/click updates scrub value
- scrub value remains continuous (`0.0..=1.0`) while nearest anchor is derived from normalized positions
- a floating date chip displays the active anchor label while dragging
- year markers are positioned on the scrub track from anchor normalized positions and provide quick jumps

### Programmatic scrolling with Iced operations

Use Iced widget operations for jumps:
- primary: `iced::widget::operation::scroll_to` (absolute viewport offset from normalized scrub value)
- fallback: `iced::widget::operation::snap_to` when viewport max-offset is not yet known

The media-pane scrollable uses a stable `Id` (`media-pane-scrollable`), and viewport events keep scrub state synchronized with manual scrolling.

## Alternatives Considered

1. Derive scrub targets from widget positions after render.
   - Rejected: brittle, tightly couples navigation logic to layout internals.
2. Replace timeline with a virtualized custom list before adding scrubber.
   - Rejected for this milestone: larger scope and unnecessary to deliver fast navigation now.
3. Generic scrollbar styling only.
   - Rejected: does not provide date-aware jumps or anchor semantics.

## Consequences

- Timeline navigation is now projection-driven and scalable for large libraries.
- Scrub interactions avoid projection rebuilds, reuse precomputed anchors, and keep marker/scroll mapping sourced from the same anchor model.
- Architecture remains extensible: gallery can later reuse anchor-based navigation patterns without changing projection boundaries.
- UI shell structure remains unchanged (header/sidebar/media/details).
