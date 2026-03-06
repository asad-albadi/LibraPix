# Indexing Architecture

Indexing is a dedicated subsystem (`librapix-indexer`) isolated from UI rendering.

## Baseline decisions

- Indexing reads source media metadata in read-only mode.
- Ignore rules are applied before metadata extraction.
- Index data is stored in Librapix-managed storage only.
- Indexing events are consumed by search and presentation layers through explicit application flow.
- Missing source files are expected operationally and must be handled as state transitions, not destructive actions.

## Baseline components

- Source root selection from storage (`active` lifecycle roots only)
- Ignore matcher via centralized `IgnoreEngine` and glob rules
- Filesystem traversal with recursive walk
- Media-kind detection by supported extension set
- Candidate writer to app-managed `indexed_media` table
- Missing-root reconciliation delegated to storage lifecycle updates

## Baseline pipeline

1. Reconcile source-root availability.
2. Load eligible roots.
3. Load enabled ignore rules.
4. Scan filesystem and filter ignored entries.
5. Emit image/video candidates.
6. Persist candidates in Librapix storage.

No indexing logic should be embedded inside view widgets.
