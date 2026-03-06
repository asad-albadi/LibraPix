# Library and Indexing Foundation Checklist

This checklist tracks the library-management and indexing foundation phase.

- [x] Verify official docs for new indexing dependencies (`globset`, `walkdir`)
- [x] Add source-root lifecycle schema migration and storage APIs
- [x] Implement missing-root reconciliation policy in storage layer
- [x] Run verification loop for lifecycle/schema milestone
- [x] Run app smoke test for lifecycle/schema milestone
- [x] Commit schema/migration milestone
- [x] Implement explicit library root orchestration flows (add, edit, deactivate, remove, list)
- [x] Keep UI thin while wiring end-to-end root management behavior
- [x] Run verification loop for root orchestration milestone
- [x] Run app smoke test for root orchestration milestone
- [ ] Commit library root orchestration milestone
- [ ] Create `librapix-indexer` crate with clear indexing boundaries
- [ ] Implement centralized ignore-rule engine with tests
- [ ] Implement minimal indexing pipeline against persisted source roots
- [ ] Add minimal indexed-media persistence baseline and migration
- [ ] Wire app orchestration to trigger and display indexing baseline results
- [ ] Run verification loop for ignore/indexer milestone
- [ ] Run app smoke test for ignore/indexer milestone
- [ ] Commit ignore/indexer foundation milestone
- [ ] Update architecture docs, dependency records, changelog, and ADRs
- [ ] Run final verification loop
- [ ] Run final app smoke test
- [ ] Commit final documentation/ADR checkpoint (if needed)
