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
- [x] Commit metadata extraction baseline
- [x] Add minimal tag-readiness schema and storage/query surface
- [x] Add search-facing read models over indexed media + tags
- [x] Wire minimal app validation flow for read models/search
- [x] Run verification loop for schema/read-model milestone
- [x] Run app smoke test for schema/read-model milestone
- [x] Commit schema/read-model milestone
- [x] Update incremental indexing policy docs and architecture decisions
- [x] Run verification loop for incremental indexing milestone
- [x] Run app smoke test for incremental indexing milestone
- [x] Commit incremental indexing milestone
- [x] Update README/CHANGELOG/dependencies/architecture docs/ADRs
- [x] Run final verification loop
- [x] Run final app smoke test
- [x] Commit final docs/ADR checkpoint (if needed)
