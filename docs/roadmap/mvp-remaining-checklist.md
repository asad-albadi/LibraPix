# MVP Remaining Checklist

This checklist tracks the remaining work to reach a usable Librapix MVP.

## A. Thumbnail subsystem

- [x] Define thumbnail architecture boundary and cache policy
- [x] Implement thumbnail generation for images (read-only source access)
- [x] Implement deterministic thumbnail cache key/path strategy
- [x] Integrate thumbnail generation into indexing workflow
- [x] Expose thumbnail lookup for gallery/timeline rendering
- [x] Document deferred video-thumbnails policy
- [x] Verification loop + smoke run
- [ ] Commit thumbnail subsystem milestone

## B. Gallery UX completion

- [x] Implement real gallery media list/grid rendering
- [x] Use gallery projections with paging/sort baseline
- [x] Show thumbnail where available
- [ ] Add media selection behavior
- [ ] Add empty-state and loading/status hints
- [x] Verification loop + smoke run
- [ ] Commit gallery UX milestone

## C. Timeline UX completion

- [x] Implement timeline grouped rendering from projections
- [ ] Add group navigation and media access controls
- [x] Align timeline route behavior with app state
- [ ] Add empty-state and loading/status hints
- [x] Verification loop + smoke run
- [ ] Commit timeline UX milestone

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
- [ ] Add explicit reindex flow messaging/status
- [x] Ensure ignore rules are visible in user-facing status
- [ ] Add actionable recoverable error surfaces
- [x] Verification loop + smoke run
- [ ] Commit root/indexing UX milestone

## G. MVP finalization

- [ ] Reconcile docs with implemented behavior
- [ ] Update architecture docs for thumbnails/details/actions/tags UX
- [ ] Update README MVP status and usage flow
- [ ] Update TROUBLESHOOTING with discovered issues/resolutions
- [ ] Run final full verification loop
- [ ] Run final app smoke test
- [ ] Commit final MVP hardening/docs milestone
- [ ] Confirm clean working tree
