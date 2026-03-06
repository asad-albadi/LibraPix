# Startup / Aggregation / Ingestion Milestone Checklist

## Startup Responsiveness
- [x] Identify blocking calls in StartupRestore handler
- [x] Create BackgroundWorkResult struct for async results
- [x] Implement do_background_work standalone function
- [x] Use Task::perform to move startup work off UI thread
- [x] Show activity status during background work
- [x] Handle BackgroundWorkComplete message to apply results
- [x] Make FilesystemChanged handler async
- [x] Make RunIndexing handler async

## Multi-Library Aggregation
- [x] Remove 500-row limit on timeline projection query
- [x] Remove 500-row limit on gallery projection query
- [x] Remove 120-item limit on gallery display
- [x] Verify gallery aggregates all active roots

## Ingestion Completeness
- [x] Remove 200-row limit on thumbnail generation read models
- [x] Remove 200-row limit on search query read models
- [x] Verify all valid media across registered roots are indexed

## Documentation
- [x] Update CHANGELOG.md
- [x] Update docs/TROUBLESHOOTING.md
- [x] Update docs/architecture/message-flow.md
- [x] Update docs/architecture/indexing.md
- [x] Create ADR 0017 for async startup architecture

## Verification
- [x] cargo fmt / check / clippy / test pass
- [x] Smoke run passes
- [x] Commits created
- [x] Working tree clean
