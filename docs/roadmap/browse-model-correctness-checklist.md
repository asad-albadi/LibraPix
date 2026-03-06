# Browse Model Correctness Milestone Checklist

## "All" Filter
- [x] Investigate filter path (gallery, timeline, search)
- [x] Verify `All` maps to "no media-kind predicate" in projection queries
- [x] Verify "All" includes both when both exist

## Multi-Root Aggregation
- [x] Remove hidden UI caps that truncate browse sets (`gallery.take(120)`, `timeline.min(200)`)
- [x] Remove aggregate read-model truncation from browse/index/search hydration paths
- [x] Verify all active roots represented

## Recursive Ingestion
- [x] Verify recursive traversal has no depth cap
- [x] Verify deep nested files across multiple roots are indexed

## Documentation
- [x] Update CHANGELOG.md
- [x] Update docs/TROUBLESHOOTING.md
- [x] Update docs/architecture/storage.md

## Verification
- [x] cargo fmt / check / clippy / test pass
- [x] Smoke run passes
- [x] Commits created
- [x] Working tree clean
