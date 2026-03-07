# Settings, Library UX, and Branding Milestone Checklist

## Scope

Product refinements: Settings dialog, hidden path field, library display names, library filter, timeline count chips, stat label, header branding.

## Checklist

- [x] **Settings dialog relocation**: Move indexing, ignore rules, diagnostics into Settings dialog
- [x] **Hidden path field**: Folder path text field hidden by default, toggleable
- [x] **Library display names**: User-defined display name per library/root
- [x] **Library filter**: Filter media by library/root
- [x] **Timeline count chips**: Total, image, video counts beside date headers
- [x] **Stat label**: Change "Shown" to "Total"
- [x] **Header branding**: Remove "- Media Library", tighten Librapix presentation
- [ ] **Docs updates**: ui.md, media-ui.md, storage.md, message-flow.md, etc.
- [ ] **Verification**: cargo fmt, check, clippy, test
- [ ] **Smoke run**: cargo run -p librapix-app
- [ ] **Commits**: Sensible checkpoints, clean working tree

## Commit Plan

1. `feat: move operational controls into settings dialog`
2. `feat: hide path field by default, add toggle`
3. `feat: add library display names and library filter`
4. `feat: add timeline count chips and refine header branding`
5. `docs: record settings and library metadata UX changes`
