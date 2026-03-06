# UI/UX Redesign Checklist

This checklist tracks mandatory visual/product-quality redesign work for Librapix.

## 1. App shell redesign

- [x] Implement desktop app shell (header + sidebar + main content + details pane)
- [x] Move root/indexing controls into structured management area
- [x] Keep media-first composition in main content area
- [x] Verification loop + smoke run
- [x] Commit app shell milestone

## 2. Design system / styling centralization

- [x] Add centralized spacing/typography/layout tokens
- [x] Add reusable panel/card helpers
- [x] Ensure consistent button hierarchy and status presentation
- [x] Verification loop + smoke run
- [x] Commit design system milestone

## 3. Gallery redesign

- [x] Render gallery as thumbnail-first card/grid experience
- [x] Keep clear selection states and card actions
- [x] Keep metadata concise and readable
- [x] Verification loop + smoke run
- [x] Commit gallery redesign milestone

## 4. Timeline redesign

- [x] Render grouped timeline sections visually
- [x] Show media items visually in timeline mode
- [x] Keep timeline selection integrated with details pane
- [x] Verification loop + smoke run
- [x] Commit timeline redesign milestone

## 5. Details/actions/tags redesign

- [x] Redesign details pane into clear metadata/tag/action sections
- [x] Keep open/copy/tag actions consistent and discoverable
- [x] Integrate game tags cleanly in details flow
- [x] Verification loop + smoke run
- [x] Commit details/actions redesign milestone

## 6. Search/root/indexing UX redesign

- [x] Integrate search into top bar and browsing flow
- [x] Improve root lifecycle visibility in management panel
- [x] Improve indexing status readability
- [x] Keep ignore-rule management integrated and uncluttered
- [x] Verification loop + smoke run
- [x] Commit root/search/indexing redesign milestone

## 7. Empty/loading/error visual states

- [x] Add intentional visual empty states
- [x] Add intentional loading/progress feedback surfaces
- [x] Add intentional action/indexing error surfaces
- [x] Verification loop + smoke run
- [x] Commit state-hardening redesign milestone

## 8. Final visual MVP reconciliation

- [x] Add/update `docs/architecture/ui.md`
- [x] Add ADR for UI shell/design-system decisions
- [x] Reconcile architecture docs and README with redesign
- [x] Final verification loop + smoke run
- [x] Final redesign commit
- [x] Clean working tree

## 9. Fluent-inspired visual redesign (Phase 2)

- [x] Expand design tokens into comprehensive Fluent-inspired system (colors, spacing, typography, radii)
- [x] Add custom button styles (primary, subtle, action, nav, card)
- [x] Add custom text input styles (search pill, field)
- [x] Add custom container styles (header, sidebar, details, cards, empty states)
- [x] Redesign header with centered search bar and Fluent-style spacing
- [x] Redesign sidebar with sectioned nav, library roots with status indicators, structured management
- [x] Redesign gallery as thumbnail-first grid with card selection states
- [x] Redesign timeline with styled group headers and card rows
- [x] Redesign details panel with clear sections, dividers, and action layout
- [x] Update i18n to product-oriented language
- [x] Increase gallery limit for richer browsing
- [x] Clean up BrowseItem subtitles to be user-facing
- [x] Remove dead code (lifecycle_text, unused ui helpers)
- [x] Full verification loop (fmt, check, clippy, test) passes clean
- [x] Smoke run passes
- [x] Documentation reconciled
