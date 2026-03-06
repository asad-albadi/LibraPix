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

- [ ] Implement real gallery media list/grid rendering
- [ ] Use gallery projections with paging/sort baseline
- [ ] Show thumbnail where available
- [ ] Add media selection behavior
- [ ] Add empty-state and loading/status hints
- [ ] Verification loop + smoke run
- [ ] Commit gallery UX milestone

## C. Timeline UX completion

- [ ] Implement timeline grouped rendering from projections
- [ ] Add group navigation and media access controls
- [ ] Align timeline route behavior with app state
- [ ] Add empty-state and loading/status hints
- [ ] Verification loop + smoke run
- [ ] Commit timeline UX milestone

## D. Media details + actions

- [ ] Add details panel for selected media metadata
- [ ] Add open file action
- [ ] Add open containing folder action
- [ ] Add copy-path action baseline
- [ ] Document platform behavior and constraints
- [ ] Verification loop + smoke run
- [ ] Commit details/actions milestone

## E. Tags and game tags baseline

- [ ] Add storage API for listing tags and media-tag links
- [ ] Add create tag / attach tag / detach tag flows
- [ ] Add game-tag creation/attachment baseline (`TagKind::Game`)
- [ ] Integrate tag filtering in search/gallery flow
- [ ] Add tests for tag workflows
- [ ] Verification loop + smoke run
- [ ] Commit tags/game-tags milestone

## F. Library + indexing UX hardening

- [ ] Improve root lifecycle visibility and indexing controls
- [ ] Add explicit reindex flow messaging/status
- [ ] Ensure ignore rules are visible in user-facing status
- [ ] Add actionable recoverable error surfaces
- [ ] Verification loop + smoke run
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
