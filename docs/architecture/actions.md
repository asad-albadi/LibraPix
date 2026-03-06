# Media Actions Architecture

Librapix exposes simple file-oriented actions without mutating source media.

## Baseline actions

- Open selected media with system default app.
- Open selected media containing folder.
- Copy selected media path to clipboard via platform command.

## Orchestration boundary

- Action commands are triggered from app orchestration handlers.
- Storage provides selected media lookup by media id.
- Media id comes from explicit selection state populated by search/gallery/timeline route panels.
- UI remains a thin input/button surface.

## Platform behavior

- macOS:
  - open: `open`
  - copy path: `pbcopy`
- Windows:
  - open: `cmd /C start`
  - copy path: `clip`
- Linux/other Unix:
  - open: `xdg-open`
  - copy path: `xclip -selection clipboard`

## Non-destructive guarantee

- Actions only read source file paths and invoke OS handlers.
- No source-file writes, renames, moves, or metadata mutation.
