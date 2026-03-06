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
  - Library root selection and orchestration state
- Domain (`librapix-core::domain`)
  - Product invariants and non-destructive rules
  - Media/tag concepts (expanding in next phases)
- i18n (`librapix-i18n`)
  - Text keys, locale catalogs, fallback behavior
- Config (`librapix-config`)
  - Typed config model
  - Config file persistence and validation
  - Platform path resolution for config/data/cache defaults
- Infrastructure (`librapix-storage` + future crates/modules)
  - SQLite persistence and migrations
  - Source root and ignore-rule persistence baseline
  - Indexing/scanning and search (future phases)

## Dependency direction

- `librapix-app` -> `librapix-core`, `librapix-i18n`, `librapix-config`, `librapix-storage` (orchestration-only usage)
- `librapix-core` -> no UI/framework dependencies
- `librapix-i18n` -> no UI/framework dependencies
- `librapix-config` -> no UI/framework dependencies
- `librapix-storage` -> no UI/framework dependencies

UI and domain must not depend on persistence implementation details.
