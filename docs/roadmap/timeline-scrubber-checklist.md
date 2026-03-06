# Timeline Scrubber Milestone Checklist

This checklist tracks the fast date-navigation scrubber milestone for Timeline mode.

## 1. Timeline anchor model

- [x] Add projection-level timeline anchor model (year/month/day label + group index + media count + normalized position)
- [x] Build anchors from timeline projection output (not from rendered widgets)
- [x] Add tests for anchor ordering and unknown bucket handling

## 2. Scrubber interaction design

- [x] Add Timeline-only right-side scrubber control
- [x] Support click and drag scrub interactions
- [x] Show floating date chip while scrubbing
- [x] Keep visuals subtle and aligned with current Fluent-inspired styling

## 3. Programmatic scroll integration

- [x] Add scrollable `Id` for media-pane content
- [x] Map scrub value to nearest timeline anchor
- [x] Scroll timeline programmatically using Iced scrollable operations

## 4. Timeline integration

- [x] Keep grouped timeline browsing behavior intact
- [x] Preserve selection/details interactions
- [x] Preserve existing shell layout (header + sidebar + media pane + details pane)

## 5. Performance and stability

- [x] Reuse precomputed timeline anchors across scrub events
- [x] Avoid full projection rebuild on scrub interaction
- [x] Keep model future-ready for gallery reuse without overengineering

## 6. Documentation updates

- [x] Update `README.md`
- [x] Update `CHANGELOG.md`
- [x] Update `docs/README.md`
- [x] Update `docs/TROUBLESHOOTING.md`
- [x] Update `docs/architecture/ui.md`
- [x] Update `docs/architecture/media-ui.md`
- [x] Update `docs/architecture/projections.md`
- [x] Update `docs/architecture/message-flow.md`
- [x] Update relevant roadmap/checklist references
- [x] Add ADR if architectural decision scope requires one

## 7. Verification loop (run after each meaningful implementation step)

- [x] `cargo fmt --all`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## 8. Smoke runs (milestone checkpoints)

- [x] `cargo run -p librapix-app`
- [x] Verify timeline still renders
- [x] Verify scrubber appears in Timeline mode
- [x] Verify scrub drag/click jumps through date groups
- [x] Stop app cleanly

## 9. Commit checkpoints

- [x] `feat: add timeline anchor model for fast date navigation`
- [x] `feat: add timeline scrubber with date chip and scroll integration`
- [x] `docs: record fast timeline navigation architecture`
- [x] Final clean working tree check
