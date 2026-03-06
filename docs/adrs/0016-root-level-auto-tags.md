# ADR 0016: Root-Level Automatic Tag Assignment

## Context

Users often know what content a library root contains (e.g. "this folder is all World of Warcraft screenshots"). Tagging every file individually is impractical. A root-level auto-tag system allows users to declare tags once per root and have them propagate to all media under that root.

## Decision

Introduce a `source_root_tags` table that stores per-root default tags. During indexing, after media upsert and kind-tag attachment, the system ensures all referenced tags exist in the `tags` table and then bulk-attaches them to media under the corresponding root via `INSERT OR IGNORE`.

### Schema

```sql
CREATE TABLE source_root_tags (
    id INTEGER PRIMARY KEY,
    source_root_id INTEGER NOT NULL,
    tag_name TEXT NOT NULL,
    tag_kind TEXT NOT NULL CHECK (tag_kind IN ('app', 'game')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (source_root_id) REFERENCES source_roots(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX idx_source_root_tags_unique
ON source_root_tags (source_root_id, tag_name);
```

### Behavior

- Auto-tags are applied as `INSERT OR IGNORE` into `media_tags`, so they coexist with manual tags.
- Removing a root tag removes the rule but does not strip existing tags from media (non-destructive).
- Root tags are visible in the sidebar when a root is selected.
- Auto-tagged media shows these tags in the details panel alongside manually-added tags.

## Alternatives Considered

1. **Config-based rules**: Store tag rules in TOML config. Rejected because storage already owns root records and tag relationships; splitting ownership adds complexity.
2. **Pattern-based conditional tagging**: Allow glob patterns to conditionally apply tags. Deferred as overengineering for the current use case; the root-level rule is sufficient.
3. **Propagation on tag removal**: Automatically strip tags from media when a root tag is removed. Rejected as destructive and inconsistent with the non-destructive guarantee.

## Consequences

- Users can efficiently tag entire libraries with a few clicks.
- The tag system becomes richer without adding complexity to the per-media tag workflow.
- Future conditional/pattern-based tagging can build on this table structure.
