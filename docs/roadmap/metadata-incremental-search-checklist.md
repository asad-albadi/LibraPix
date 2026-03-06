# Metadata, Incremental Indexing, and Read Model Checklist

This checklist tracks the metadata extraction and incremental indexing phase.

- [x] Verify official docs for metadata extraction dependency (`imagesize`)
- [x] Define metadata extraction baseline (scope now vs deferred)
- [x] Add schema updates for extracted metadata and incremental reconciliation
- [x] Implement metadata extraction stage in `librapix-indexer`
- [x] Implement incremental change detection (`new` / `unchanged` / `changed`)
- [x] Implement missing-file reconciliation for indexed media records
- [x] Run verification loop for metadata extraction milestone
- [x] Run app smoke test for metadata extraction milestone
- [ ] Commit metadata extraction baseline
- [ ] Add minimal tag-readiness schema and storage/query surface
- [ ] Add search-facing read models over indexed media + tags
- [ ] Wire minimal app validation flow for read models/search
- [ ] Run verification loop for schema/read-model milestone
- [ ] Run app smoke test for schema/read-model milestone
- [ ] Commit schema/read-model milestone
- [ ] Update incremental indexing policy docs and architecture decisions
- [ ] Run verification loop for incremental indexing milestone
- [ ] Run app smoke test for incremental indexing milestone
- [ ] Commit incremental indexing milestone
- [ ] Update README/CHANGELOG/dependencies/architecture docs/ADRs
- [ ] Run final verification loop
- [ ] Run final app smoke test
- [ ] Commit final docs/ADR checkpoint (if needed)
