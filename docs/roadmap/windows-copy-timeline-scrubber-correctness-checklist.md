# Windows Copy + Timeline Scrubber Correctness Checklist

Milestone date: 2026-03-07

## 1. Windows copy-file diagnosis

- [x] Audit current Windows clipboard command path and runtime assumptions
- [x] Verify payload expectation against Windows shell clipboard/file-drop behavior (CF_HDROP)
- [x] Verify STA/threading expectations and whether current flow satisfies them
- [x] Identify concrete root cause for Windows failure while macOS path succeeds

## 2. Windows clipboard/file-drop fix

- [x] Implement reliable Windows file-copy clipboard write with real file-drop payload
- [x] Ensure failure is surfaced when clipboard payload is not actually set
- [x] Preserve `Copy Path` behavior on Windows
- [x] Add/update tests for Windows clipboard command/payload handling where practical

## 3. Platform behavior comparison

- [x] Document why macOS copy-file path works and why previous Windows path failed
- [x] Record Windows-specific constraints and guarantees in architecture/troubleshooting docs

## 4. Timeline scrubber diagnosis

- [x] Audit anchor generation, marker generation, and scrub mapping model
- [x] Confirm root cause of incorrect year marker placement
- [x] Confirm root cause of scrubber jumpiness/sticky behavior
- [x] Verify marker and scroll mappings are derived from same source-of-truth model

## 5. Marker placement fix

- [x] Correct marker positions to align with timeline anchor positions
- [x] Ensure year labels do not appear in invalid/unexpected locations
- [x] Add/update tests for marker generation/placement logic

## 6. Smooth scrub behavior fix

- [x] Correct mapping from scrubber Y-position to scroll target
- [x] Correct mapping from viewport scroll position back to scrubber state
- [x] Ensure scrub interaction is smooth/predictable (no sticky anchor lockups)
- [x] Add/update tests for scrub mapping and anchor-selection behavior

## 7. Documentation updates

- [x] Update `README.md`
- [x] Update `CHANGELOG.md`
- [x] Update `docs/README.md`
- [x] Update `docs/TROUBLESHOOTING.md`
- [x] Update `docs/DEPENDENCIES.md`
- [x] Update `docs/architecture/ui.md`
- [x] Update `docs/architecture/media-ui.md`
- [x] Update `docs/architecture/actions.md`
- [x] Update `docs/architecture/message-flow.md`
- [x] Update `docs/architecture/projections.md`
- [x] Update relevant roadmap/checklist docs
- [x] Update relevant ADR(s)

## 8. Verification loop

- [x] Run `cargo fmt --all` after each meaningful implementation step
- [x] Run `cargo check --workspace` after each meaningful implementation step
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings` after each meaningful implementation step
- [x] Run `cargo test --workspace` after each meaningful implementation step

## 9. Smoke runs

- [x] Run `cargo run -p librapix-app` after each meaningful milestone step
- [ ] Verify timeline scrubber marker placement and drag behavior as far as environment allows
- [ ] Verify Windows copy logic path as far as environment allows

## 10. Commit checkpoints

- [ ] Commit: `fix: restore Windows file copy with verified file-drop clipboard payload`
- [ ] Commit: `fix: align timeline scrubber markers and smooth scroll mapping`
- [ ] Commit: `docs: record Windows clipboard and timeline scrubber root-cause fixes`

## 11. Final reconciliation

- [ ] Run final full verification loop
- [ ] Confirm smoke run status
- [ ] Confirm all checklist items complete
- [ ] Confirm working tree clean
