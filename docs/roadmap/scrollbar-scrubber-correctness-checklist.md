# Scrollbar + Scrubber Correctness Checklist

Milestone date: 2026-03-07

## 1. Scrollbar layout investigation

- [x] Audit media-pane scrollable/container composition in `librapix-app`
- [x] Confirm why scrollbar overlays media cards instead of reserving layout space
- [x] Verify affected browsing surfaces (gallery + timeline)

## 2. Scrollbar/content separation fix

- [x] Implement structural media-pane scrollbar separation (no padding hack)
- [x] Ensure cards/content are never obscured by the scrollbar
- [x] Preserve shell layout and existing spacing quality

## 3. Timeline scrubber first-click investigation

- [x] Audit scrubber pointer-down and drag initialization path
- [x] Confirm why first click causes visible snap/jump behavior
- [x] Validate whether visual scrubber layout shifts on initial interaction

## 4. Timeline scrubber interaction fix

- [x] Remove first-click snap/jump behavior
- [x] Keep first click stable from current position and continue smooth drag behavior
- [x] Preserve existing anchor-based timeline mapping behavior

## 5. Documentation updates

- [x] Update `CHANGELOG.md`
- [x] Update `docs/TROUBLESHOOTING.md`
- [x] Update `docs/architecture/ui.md`
- [x] Update `docs/architecture/media-ui.md`
- [x] Update `docs/architecture/message-flow.md`
- [x] Update relevant roadmap checklist references

## 6. Verification loop checkpoints

- [x] Run `cargo fmt --all` after each meaningful implementation step
- [x] Run `cargo check --workspace` after each meaningful implementation step
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings` after each meaningful implementation step
- [x] Run `cargo test --workspace` after each meaningful implementation step
- [x] Verification checkpoint A completed after scrollbar layout fix
- [x] Verification checkpoint B completed after scrubber interaction fix
- [x] Final integrated verification loop completed after docs reconciliation

## 7. Smoke runs

- [x] Milestone A smoke run: verify media scrollbar sits beside content
- [x] Milestone B smoke run: verify scrubber first-click behavior is stable
- [x] Final smoke run before completion

## 8. Commit checkpoints

- [x] Commit `fix: move media scrollbar outside card content area`
- [x] Commit `fix: remove initial snap from timeline scrubber interaction`
- [ ] Commit `docs: record scrollbar and scrubber interaction fixes`
- [ ] Final working tree clean check
