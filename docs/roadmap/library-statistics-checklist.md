# Library Statistics Dialog Milestone Checklist

## Scope

Add a separate Library Statistics dialog backed by maintained persisted statistics per library/root.

## Checklist

- [x] **Library statistics data model**: persisted per-root statistics schema/model
- [x] **Maintained-on-indexing update path**: stats refreshed during indexing/re-index
- [x] **Storage/read-model access**: clean API to read per-root stats without recomputation
- [x] **Separate Library Statistics dialog**: dedicated modal, not part of Edit Library
- [x] **Library stats action entry point**: clear, discoverable Stats action in library UI
- [x] **Docs updates**: README/changelog/architecture/troubleshooting/checklist updates
- [x] **Verification loop**: fmt/check/clippy/test after meaningful steps
- [x] **Smoke runs**: app launch and quick stats-dialog open validation
- [x] **Commit checkpoints**: meaningful commits and clean working tree

## Commit Plan

1. `feat: add maintained library statistics model`
2. `feat: add library statistics dialog`
3. `docs: record library statistics maintenance and dialog flow`
