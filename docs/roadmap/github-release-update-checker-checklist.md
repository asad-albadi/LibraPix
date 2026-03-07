# GitHub Release Update Checker Checklist

This checklist tracks the update-indicator milestone for startup checks, periodic re-checks, and header chip interactions.

## 1. Update state model

- [x] Add explicit update-check state model (`Unknown`, `Checking`, `UpToDate`, `UpdateAvailable`, `Failed`)
- [x] Track last successful check timestamp
- [x] Track last manual check timestamp
- [x] Keep update-check state and transitions explicit in app orchestration

## 2. Startup update check

- [x] Trigger update check on startup after first render
- [x] Keep startup update check non-blocking
- [x] Keep startup check independent from indexing/project background work

## 3. 24-hour periodic re-check

- [x] Add periodic tick/subscription for long-running sessions
- [x] Enforce 24-hour auto re-check policy from last successful check
- [x] Ensure re-check does not run while a check is already in progress

## 4. 5-minute manual cooldown

- [x] Add manual click-driven re-check path
- [x] Enforce 5-minute manual cooldown cleanly
- [x] Handle cooldown-blocked clicks quietly (no noisy error surface)

## 5. Header chip UI

- [x] Add header update chip aligned with current design system
- [x] Show `Checking…` while update check is in flight
- [x] Show `Up to date` when current
- [x] Show `New release` when a newer version exists
- [x] Keep failed-state presentation subtle and non-noisy

## 6. Release page open behavior

- [x] Open latest release page when update is available and chip is clicked
- [x] Otherwise chip click triggers manual re-check (if cooldown allows)
- [x] Use repository releases page fallback when appropriate

## 7. Verification loop

- [x] Run `cargo fmt --all`
- [x] Run `cargo check --workspace`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo test --workspace`

## 8. Smoke runs

- [x] Launch `cargo run -p librapix-app`
- [x] Verify header chip appears
- [x] Verify startup update check does not block UI
- [x] Verify manual click behavior and cooldown handling
- [x] Stop app cleanly

## 9. Documentation reconciliation

- [x] Update `README.md`
- [x] Update `CHANGELOG.md`
- [x] Update `docs/README.md`
- [x] Update `docs/architecture/ui.md`
- [x] Update `docs/architecture/message-flow.md`
- [x] Update `docs/architecture/actions.md`
- [x] Update `docs/TROUBLESHOOTING.md` if needed
- [x] Update roadmap/checklist docs
- [x] Add/update ADR if architecture decisions changed materially

## 10. Commit checkpoints

- [x] Commit feature implementation milestone
- [x] Commit docs reconciliation milestone
- [x] Confirm working tree clean
