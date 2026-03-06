# Storage Architecture

SQLite is the planned primary persistent store for Librapix-owned metadata.

## Scope of app-managed storage

- Library registrations
- App-side tags and game tags
- Search indexes
- Memories/resurfacing data
- Ignore rules
- UI/app preferences

## Hard constraints

- Never write app metadata into source media files.
- Never modify user media file names or locations.
- Keep persistence concerns isolated from view code.

## Planned implementation notes

- Use migrations from day one once persistence is introduced.
- Keep domain models separated from storage models where useful.
