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
- Domain (`librapix-core::domain`)
  - Product invariants and non-destructive rules
  - Media/tag concepts (expanding in next phases)
- i18n (`librapix-i18n`)
  - Text keys, locale catalogs, fallback behavior
- Infrastructure (planned dedicated crates/modules)
  - Storage, indexing/scanning, search, config

## Dependency direction

- `librapix-app` -> `librapix-core`, `librapix-i18n`
- `librapix-core` -> no UI/framework dependencies
- `librapix-i18n` -> no UI/framework dependencies

UI and domain must not depend on persistence implementation details.
