# UI/UX Redesign Checklist

This checklist tracks mandatory visual/product-quality redesign work for Librapix.

## 1. App shell redesign

- [x] Implement desktop app shell (header + sidebar + main content + details pane)
- [x] Move root/indexing controls into structured management area
- [x] Keep media-first composition in main content area
- [x] Verification loop + smoke run
- [ ] Commit app shell milestone

## 2. Design system / styling centralization

- [x] Add centralized spacing/typography/layout tokens
- [x] Add reusable panel/card helpers
- [x] Ensure consistent button hierarchy and status presentation
- [x] Verification loop + smoke run
- [ ] Commit design system milestone

## 3. Gallery redesign

- [x] Render gallery as thumbnail-first card/grid experience
- [x] Keep clear selection states and card actions
- [x] Keep metadata concise and readable
- [x] Verification loop + smoke run
- [ ] Commit gallery redesign milestone

## 4. Timeline redesign

- [x] Render grouped timeline sections visually
- [x] Show media items visually in timeline mode
- [x] Keep timeline selection integrated with details pane
- [x] Verification loop + smoke run
- [ ] Commit timeline redesign milestone

## 5. Details/actions/tags redesign

- [ ] Redesign details pane into clear metadata/tag/action sections
- [ ] Keep open/copy/tag actions consistent and discoverable
- [ ] Integrate game tags cleanly in details flow
- [ ] Verification loop + smoke run
- [ ] Commit details/actions redesign milestone

## 6. Search/root/indexing UX redesign

- [ ] Integrate search into top bar and browsing flow
- [ ] Improve root lifecycle visibility in management panel
- [ ] Improve indexing status readability
- [ ] Keep ignore-rule management integrated and uncluttered
- [ ] Verification loop + smoke run
- [ ] Commit root/search/indexing redesign milestone

## 7. Empty/loading/error visual states

- [ ] Add intentional visual empty states
- [ ] Add intentional loading/progress feedback surfaces
- [ ] Add intentional action/indexing error surfaces
- [ ] Verification loop + smoke run
- [ ] Commit state-hardening redesign milestone

## 8. Final visual MVP reconciliation

- [ ] Add/update `docs/architecture/ui.md`
- [ ] Add ADR for UI shell/design-system decisions
- [ ] Reconcile architecture docs and README with redesign
- [ ] Final verification loop + smoke run
- [ ] Final redesign commit
- [ ] Clean working tree
