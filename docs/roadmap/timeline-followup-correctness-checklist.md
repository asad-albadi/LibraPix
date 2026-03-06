# Timeline & Interaction Follow-Up Checklist

This checklist tracks the correctness + UX fixes requested after the media-navigation milestone.

## 1. Timeline scrubber smoothness/reliability

- [x] Audit scrubber anchor generation and anchor-to-scroll mapping
- [x] Remove date/year "stuck" behavior under drag and click
- [x] Ensure stable mapping when timeline data changes at runtime
- [x] Improve scrubber visual polish and interaction clarity

## 2. New-file dialog redesign

- [x] Replace top-of-grid announcement card with modal dialog overlay
- [x] Show media preview in dialog
- [x] Show useful metadata in dialog
- [x] Provide quick actions (open, copy file, view/select, dismiss)

## 3. Keyboard shortcuts for copy actions

- [x] Add keyboard shortcut for copy selected file
- [x] Add keyboard shortcut for copy selected path
- [x] Ensure shortcuts do not fire while text inputs handle key events
- [ ] Document shortcut behavior in architecture/UI docs

## 4. Details action layout responsiveness

- [x] Fix details action row clipping/cutoff on narrow widths
- [x] Use responsive action layout without spacing hacks
- [x] Keep action styling consistent with current design system

## 5. Top browse statistics correctness

- [x] Replace inconsistent top count with derived current-result stats
- [x] Show total shown count
- [x] Show image count
- [x] Show video count
- [x] Ensure stats use active view/filter/search result source consistently

## 6. Timeline regrouping for new files

- [x] Confirm root cause for wrong day bucket assignment
- [x] Fix grouping date conversion to chosen user-facing timeline day semantics
- [x] Verify filesystem-triggered runtime refresh places new files in correct day bucket
- [x] Add regression tests for grouping edge case

## 7. Documentation updates

- [ ] Update `README.md`
- [ ] Update `CHANGELOG.md`
- [ ] Update `docs/README.md`
- [ ] Update `docs/TROUBLESHOOTING.md`
- [ ] Update affected architecture docs (`ui.md`, `media-ui.md`, `projections.md`, `message-flow.md`, `search.md`, `actions.md`, `indexing.md`, `storage.md`)
- [ ] Add/update ADR if architectural decision changes

## 8. Verification loop per milestone

- [x] `cargo fmt --all`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## 9. Smoke runs per milestone

- [x] `cargo run -p librapix-app` after meaningful milestones
- [x] Verify scrubber behavior, dialog behavior, shortcuts, stats, and timeline grouping
- [x] Stop cleanly

## 10. Commit checkpoints

 - [x] Commit timeline scrubber + regrouping fixes
- [ ] Commit dialog + shortcut + details/stats fixes
- [ ] Commit docs reconciliation
- [ ] Final clean working tree check
