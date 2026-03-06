# Media Navigation Product Updates Checklist

This checklist tracks the required product changes after the timeline scrubber milestone.

## 1. Tag filter axis

- [x] Add explicit `filter_tag` state and message flow
- [x] Populate available filter tags from indexed read models
- [x] Add product-style tag filter UI integrated with existing chips
- [x] Apply tag filter to gallery, timeline, and search projections

## 2. Media card metadata row polish

- [x] Fix metadata padding/alignment to avoid clipping near card radius
- [x] Include file size in card metadata line
- [x] Include dimensions in card metadata line when available
- [x] Keep metadata subtle and compact

## 3. Corner media-type icons

- [x] Add image/video corner icon badge on media cards
- [x] Keep icon placement and spacing visually consistent

## 4. Copy actual media file

- [x] Add details action for copying the selected file object (not only path text)
- [x] Keep existing copy-path action
- [x] Implement practical cross-platform baseline and document behavior

## 5. New-file in-app announcement

- [x] Add announcement state model for newly detected files
- [x] Detect newly indexed files during live filesystem refresh
- [x] Show dismissible in-app notification with metadata
- [x] Add quick actions (open file, copy file, select/view)

## 6. Search cap removal

- [x] Remove hidden search result cap of 20
- [x] Keep result completeness explicit in app search path

## 7. Search box radius consistency

- [x] Align search input radius with design system component language

## 8. Windows publisher/signing setup

- [x] Add baseline Windows packaging/signing docs and scripts
- [x] Configure publisher identity for Windows manifest/signing flow (`CN=Asad`)
- [x] Document dev cert generation/import for local testing
- [x] Document release signing flow (SignTool + trusted certificate)

## 9. Docs updates

- [x] Update `README.md`
- [x] Update `CHANGELOG.md`
- [x] Update `docs/README.md`
- [x] Update `docs/TROUBLESHOOTING.md`
- [x] Update `docs/DEPENDENCIES.md` (not required; no new dependency introduced)
- [x] Update architecture docs (`ui.md`, `media-ui.md`, `projections.md`, `search.md`, `actions.md`, `message-flow.md`, and other affected docs)
- [x] Add/update ADR for meaningful new architectural decision(s)

## 10. Verification loop

- [x] `cargo fmt --all`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## 11. Smoke runs

- [x] Run `cargo run -p librapix-app` for milestone checkpoints
- [x] Verify UI/interaction behavior for each implemented milestone
- [x] Stop app cleanly

## 12. Commit checkpoints

- [x] Commit filter + media card improvements
- [x] Commit copy-file + announcement flow
- [x] Commit search/radius fixes and Windows signing setup docs/scripts
- [ ] Final clean working tree confirmation
