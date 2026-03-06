# Non-Destructive Guarantee

This is a hard architectural contract.

## Guaranteed behavior

- Source media files are treated as read-only.
- Librapix organizational metadata is stored only in app-managed storage.
- Metadata extraction is read-only.
- Thumbnail/cache/index/search artifacts remain in app-owned storage.

## Forbidden behavior

- Writing tags or app metadata into source media files.
- Renaming or moving source media files.
- Rewriting user folder structures.

## Enforcement

- Domain-level invariants must reflect this guarantee.
- Storage/indexing code must be reviewed against this document.
- Any proposed exception requires explicit architecture approval and ADR documentation.
