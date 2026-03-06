# Config and Storage Foundation Checklist

This checklist tracks the configuration and persistence foundation phase.

- [x] Verify official docs for config/storage dependencies (Serde, TOML, directories, rusqlite)
- [x] Add workspace crates `librapix-config` and `librapix-storage`
- [x] Implement typed config model and TOML persistence baseline
- [x] Define config defaults, validation, and path normalization behavior
- [x] Run full verification loop for config milestone
- [x] Run app smoke test for config milestone
- [x] Commit config subsystem checkpoint
- [x] Implement SQLite storage subsystem with migration runner
- [x] Define and add baseline SQLite schema migration(s)
- [x] Document required policy decisions (paths, missing files, cache/thumbnails ownership, source root ownership)
- [x] Wire source registration bootstrap path to persisted config/storage if cleanly fitting
- [x] Run full verification loop for storage milestone
- [x] Run app smoke test for storage milestone
- [x] Commit storage/schema milestone
