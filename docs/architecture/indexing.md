# Indexing Architecture

Indexing is planned as a dedicated subsystem, isolated from UI rendering.

## Baseline decisions

- Indexing reads source media metadata in read-only mode.
- Ignore rules are applied before metadata extraction.
- Index data is stored in Librapix-managed storage only.
- Indexing events are consumed by search and presentation layers through explicit application flow.
- Missing source files are expected operationally and must be handled as state transitions, not destructive actions.

## Planned components

- Source discovery (library roots + watcher hooks)
- Ignore matcher (centralized)
- Metadata extractor (read-only)
- Index writer (app-managed store)
- Missing-file reconciler that updates index state without touching source media

No indexing logic should be embedded inside view widgets.
