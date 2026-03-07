# About, Scrollbars, and Library Management Milestone Checklist

## Scope

Product refinements: panel scrollbar gutters, About flow, timeline chip cleanup, and a unified add/edit library management dialog.

## Checklist

- [x] **Settings/details scrollbar overlap fix**: reserve scrollbar gutter so controls are not obscured
- [x] **About button**: add header About action next to Settings
- [x] **About dialog**: polished in-app modal with project/creator notes
- [x] **Timeline chip refinement**: remove Total from timeline headers, keep image/video chips
- [x] **Unified add/edit library dialog**: one organized flow for add + modify
- [x] **Library edit action**: explicit Edit control per library entry
- [x] **Library edit support**: display name + tag add/remove + path re-browse/update
- [x] **Add-more flow**: add additional libraries from same dialog flow
- [x] **Docs updates**: README/CHANGELOG/architecture/troubleshooting/checklist docs
- [x] **Verification loop**: fmt/check/clippy/test after meaningful steps
- [x] **Smoke runs**: `cargo run -p librapix-app` after milestones
- [ ] **Commit checkpoints**: meaningful commits and clean working tree

## Commit Plan

1. `fix: prevent settings and details scrollbars from overlapping content`
2. `feat: add about dialog and header action`
3. `feat: add unified library add/edit dialog with tag management`
4. `feat: refine timeline chips and library edit flow`
5. `docs: record about and library management UX updates`
