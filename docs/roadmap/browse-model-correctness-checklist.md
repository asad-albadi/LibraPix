# Browse Model Correctness Milestone Checklist

## "All" Filter
- [x] Investigate filter path (gallery, timeline, search)
- [x] Add per-media-kind cap (5k images, 5k videos per root)
- [x] Verify "All" includes both when both exist

## Multi-Root Aggregation
- [x] Add per-root cap (10k per root)
- [x] Use ROW_NUMBER() PARTITION BY source_root_id
- [x] Verify all active roots represented

## Documentation
- [x] Update CHANGELOG.md
- [x] Update docs/TROUBLESHOOTING.md
- [x] Update docs/architecture/storage.md

## Verification
- [x] cargo fmt / check / clippy / test pass
- [x] Smoke run passes
- [x] Commits created
- [x] Working tree clean
