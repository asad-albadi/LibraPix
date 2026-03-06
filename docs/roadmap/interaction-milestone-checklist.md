# Interaction Milestone Checklist

This checklist tracks the product-quality interaction milestone for Librapix.

## 1. Human-readable metadata formatting

- [x] Add centralized `format.rs` with `format_file_size` and `format_timestamp`
- [x] Add `format_dimensions` helper
- [x] Update `load_media_details` to use formatted values
- [x] Update gallery card subtitles with formatted file size
- [x] Verification loop

## 2. Startup state restore + auto-indexing

- [x] Change app init to return a startup Task via `Task::done`
- [x] On startup, auto-run indexing and gallery/timeline projection
- [x] Surface startup activity status in header
- [x] Verification loop

## 3. Directory picker integration

- [x] Add `rfd` dependency (verified from official docs, v0.15)
- [x] Add Browse button to sidebar library section (primary action)
- [x] Wire folder picker to root input path
- [x] Keep manual path input as secondary flow
- [x] Document dependency decision in DEPENDENCIES.md
- [x] Verification loop

## 4. Double-click to open media

- [x] Track last-click state (media id + timestamp) in app state
- [x] On double-click same media within 400ms threshold, open in external app
- [x] Keep single-click as selection behavior
- [x] Verification loop

## 5. Background activity indicators

- [x] Track activity state as string in app model
- [x] Show subtle accent-colored status caption in header
- [x] Update status during indexing and restore flows
- [x] Clear status when operation completes
- [x] Verification loop

## 6. Auto-refresh after operations

- [x] Auto-refresh gallery/timeline after indexing completes (startup flow)
- [x] Auto-index and refresh gallery after adding a root
- [x] Refresh gallery after removing a root
- [x] Verification loop

## 7. i18n updates

- [x] Add `BrowseFolderButton` key
- [x] Add `StatusRestoringLabel` key
- [x] Add metadata label keys (DetailsKindLabel, DetailsSizeLabel, etc.)

## 8. Final reconciliation

- [x] Update CHANGELOG.md
- [x] Update docs/DEPENDENCIES.md
- [x] Update docs/architecture/ui.md
- [x] Update docs/architecture/message-flow.md
- [x] Update docs/architecture/actions.md
- [x] Add ADR 0013 for interaction milestone decisions
- [x] Final verification loop + smoke run
- [x] Commit
