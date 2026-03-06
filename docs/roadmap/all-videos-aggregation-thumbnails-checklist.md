# All / Videos / Aggregation / Thumbnails Milestone Checklist

## "All" Filter
- [x] Investigate filter path (gallery, timeline, search)
- [x] Verify media_kind filter semantics when None
- [x] Add mkv, webm to extension chips for "All" type

## Multi-Library Aggregation
- [x] Investigate query ORDER BY (path ASC favored few roots)
- [x] Change to modified DESC for interleaved unified view
- [x] Verify all active roots included in queries
- [x] Persist roots added via UI to config file

## Video Thumbnails
- [x] Add Windows ffmpeg.exe explicit invocation
- [x] Normalize paths to forward slashes for ffmpeg on Windows
- [x] Document Windows-specific behavior

## Documentation
- [x] Update CHANGELOG.md
- [x] Update docs/TROUBLESHOOTING.md
- [x] Update docs/architecture/thumbnails.md
- [x] Update docs/architecture/storage.md

## Verification
- [x] cargo fmt / check / clippy / test pass
- [x] Smoke run passes
- [x] Commits created
- [x] Working tree clean
