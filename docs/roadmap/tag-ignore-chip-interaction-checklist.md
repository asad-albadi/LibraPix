# Tag + Ignore Chip Interaction Checklist

This checklist tracks the chip-based redesign milestone for library tags, details tags, and settings ignore rules.

## 1. Reusable chip component/system

- [x] Add centralized chip visual primitives/styles in `crates/librapix-app/src/ui.rs`
- [x] Add reusable chip rendering helpers in `crates/librapix-app/src/main.rs`
- [x] Keep widget logic presentation-only and message-driven

## 2. Deterministic chip coloring

- [x] Add deterministic string-to-palette color mapping for tags/rules
- [x] Ensure color mapping is stable across sessions (non-random)
- [x] Ensure chip text contrast is readable on dark theme

## 3. Edit Library tag redesign

- [x] Replace raw list rows with organized chip-based tags
- [x] Add chip remove action (`x`)
- [x] Add chip edit action with clean inline flow
- [x] Keep explicit add flow for app/game tags

## 4. Details tag redesign

- [x] Render media tags as colored chips
- [x] Distinguish inherited/library tags from manual tags
- [x] Add chip remove action (`x`)
- [x] Add chip edit flow where appropriate
- [x] Keep explicit add flow for app/game tags

## 5. Settings ignore-rule redesign

- [x] Replace raw preview list with chip-like rule items
- [x] Add chip remove action (`x`)
- [x] Add chip edit flow
- [x] Keep add flow explicit
- [x] Preserve enable/disable semantics
- [x] Preserve min-size exclusion flow

## 6. Verification loop

- [x] Run `cargo fmt --all`
- [x] Run `cargo check --workspace`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo test --workspace`

## 7. Smoke runs

- [x] Launch `cargo run -p librapix-app`
- [x] Verify chip rendering and interactions in Edit Library
- [x] Verify chip rendering and interactions in Details
- [x] Verify chip rendering and interactions in Settings ignore rules
- [x] Stop app cleanly

## 8. Documentation reconciliation

- [x] Update `README.md`
- [x] Update `CHANGELOG.md`
- [x] Update `docs/README.md`
- [x] Update architecture docs (`ui`, `message-flow`, `actions`, `storage`)
- [x] Update roadmap/checklist docs
- [x] Add/update ADR if architecture decisions changed materially

## 9. Commit checkpoints

- [ ] Commit reusable chip + deterministic color base
- [ ] Commit library/details tag redesign
- [ ] Commit settings ignore-rule redesign
- [ ] Commit docs reconciliation
- [ ] Confirm working tree clean
