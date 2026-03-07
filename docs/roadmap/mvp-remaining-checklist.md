# MVP Remaining Checklist

This checklist tracks the remaining work to reach a usable Librapix MVP.

## Current closure pass

- [x] Selection UX milestone plan updated
- [x] Empty/loading/error hardening completion
- [x] Root/indexing UX hardening completion
- [x] Final MVP reconciliation completion
- [x] GitHub release update checker + header status chip completion
- [x] Full final verification + smoke run + clean-tree confirmation

## A. Thumbnail subsystem

- [x] Define thumbnail architecture boundary and cache policy
- [x] Implement thumbnail generation for images (read-only source access)
- [x] Implement deterministic thumbnail cache key/path strategy
- [x] Integrate thumbnail generation into indexing workflow
- [x] Expose thumbnail lookup for gallery/timeline rendering
- [x] Document deferred video-thumbnails policy
- [x] Verification loop + smoke run
- [x] Commit thumbnail subsystem milestone

## B. Gallery UX completion

- [x] Implement real gallery media list/grid rendering
- [x] Use gallery projections with paging/sort baseline
- [x] Show thumbnail where available
- [x] Add media selection behavior
- [x] Add empty-state and loading/status hints
- [x] Verification loop + smoke run
- [x] Commit gallery UX milestone

## C. Timeline UX completion

- [x] Implement timeline grouped rendering from projections
- [x] Add group navigation and media access controls
- [x] Align timeline route behavior with app state
- [x] Add empty-state and loading/status hints
- [x] Verification loop + smoke run
- [x] Commit timeline UX milestone

## D. Media details + actions

- [x] Add details panel for selected media metadata
- [x] Add open file action
- [x] Add open containing folder action
- [x] Add copy-path action baseline
- [x] Document platform behavior and constraints
- [x] Verification loop + smoke run
- [x] Commit details/actions milestone

## E. Tags and game tags baseline

- [x] Add storage API for listing tags and media-tag links
- [x] Add create tag / attach tag / detach tag flows
- [x] Add game-tag creation/attachment baseline (`TagKind::Game`)
- [x] Integrate tag filtering in search/gallery flow
- [x] Add tests for tag workflows
- [x] Verification loop + smoke run
- [x] Commit tags/game-tags milestone

## F. Library + indexing UX hardening

- [ ] Improve root lifecycle visibility and indexing controls
- [x] Add explicit reindex flow messaging/status
- [x] Ensure ignore rules are visible in user-facing status
- [x] Add actionable recoverable error surfaces
- [x] Verification loop + smoke run
- [x] Commit root/indexing UX milestone

## G. MVP finalization

- [x] Reconcile docs with implemented behavior
- [x] Update architecture docs for thumbnails/details/actions/tags UX
- [x] Update README MVP status and usage flow
- [x] Update TROUBLESHOOTING with discovered issues/resolutions
- [x] Run final full verification loop
- [x] Run final app smoke test
- [x] Commit final MVP hardening/docs milestone
- [x] Confirm clean working tree
