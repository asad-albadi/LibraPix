# Layers

## Layer responsibilities

- Bootstrap (`librapix-app`)
  - Process startup
  - Iced application wiring
  - Theme/window/runtime integration
- Presentation (`librapix-app`)
  - View composition
  - Message emission from user actions
  - No direct storage/indexing logic
- Application orchestration (`librapix-core::app`)
  - Explicit app state and state transitions
  - Route-level coordination
  - Library root and media selection orchestration state
- Domain (`librapix-core::domain`)
  - Product invariants and non-destructive rules
  - Media/tag concepts (expanding in next phases)
- i18n (`librapix-i18n`)
  - Text keys, locale catalogs, fallback behavior
- Config (`librapix-config`)
  - Typed config model
  - Config file persistence and validation
  - Platform path resolution for config/data/cache defaults
- Indexing (`librapix-indexer`)
  - Scan orchestration against eligible source roots
  - Centralized ignore-rule matching
  - Candidate production for storage persistence
- Search (`librapix-search`)
  - Replaceable search strategy contracts
  - Baseline fuzzy-capable ranking over read-model documents
- Projections (`librapix-projections`)
  - Timeline grouping projections (day/month/year)
  - Gallery filtering/sorting projections
- Thumbnails (`librapix-thumbnails`)
  - Deterministic cache-key/path generation
  - Read-only image thumbnail rendering pipeline
- Infrastructure (`librapix-storage` + future crates/modules)
  - SQLite persistence and migrations
  - Source root and ignore-rule persistence baseline
  - Search-facing read-model queries over indexed media and tags
  - Richer search ranking/fuzzy behavior (future phases)

## Dependency direction

- `librapix-app` -> `librapix-core`, `librapix-i18n`, `librapix-config`, `librapix-storage`, `librapix-indexer` (orchestration-only usage)
- `librapix-core` -> no UI/framework dependencies
- `librapix-i18n` -> no UI/framework dependencies
- `librapix-config` -> no UI/framework dependencies
- `librapix-storage` -> no UI/framework dependencies
- `librapix-indexer` -> no UI/framework dependencies

UI and domain must not depend on persistence implementation details.
