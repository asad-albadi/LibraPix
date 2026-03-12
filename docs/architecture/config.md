# Config Architecture

`librapix-config` owns typed application configuration and config-file persistence.

## Format and location strategy

- Format: TOML (`config.toml`), serialized/deserialized with Serde + `toml`.
- Location: platform-specific project config directory from `directories::ProjectDirs`.
- Path defaults are resolved by `librapix-config`:
  - config file: `<config_dir>/config.toml`
  - data dir: `<data_dir>`
  - cache dir: `<cache_dir>`
  - thumbnails dir: `<cache_dir>/thumbnails`
  - database file: `<data_dir>/librapix.db`

## Typed model baseline

- `schema_version`
- `locale`
- `theme`
- `library_source_roots`
- `path_overrides` for future path relocation support
- `video_tools.default_shorts_output_dir` (optional default output base directory for Make Short)

## Validation behavior

- Relative source paths are converted to absolute paths using current working directory.
- Paths are lexically normalized (`.` and `..` handling) without requiring file existence.
- Duplicate normalized source roots are rejected.
- Unknown schema versions are rejected.
- Optional path overrides are normalized the same way.

## Path and existence policy

- Config validation does not require source roots to exist at save/load time.
- Canonicalization through filesystem resolution is deferred to indexing-time checks.
- This preserves support for removable/offline drives while keeping deterministic normalized path storage.

## Evolution strategy

- Schema version starts at `1`.
- Config schema changes must be backward migration-aware and documented.
- Runtime should load existing configs conservatively and persist only validated values.
