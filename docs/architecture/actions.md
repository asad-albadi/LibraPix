# Media Actions Architecture

Librapix exposes simple file-oriented actions without mutating source media.

## Baseline actions

- Open selected media with system default app.
- Open selected media containing folder.
- Copy selected media file to clipboard (file object copy, not just path text).
- Copy selected media path to clipboard via platform command.
- Keyboard shortcuts:
  - `Cmd/Ctrl+C`: copy selected media file
  - `Cmd/Ctrl+Shift+C`: copy selected media path

## Double-click open

- Double-clicking a media item in gallery or timeline opens it in the OS default external app.
- Single click selects the item and loads its details.
- Double-click is detected at app level by tracking last-click media id and timestamp.
- Threshold: 400ms between clicks on the same media item.

## Orchestration boundary

- Action commands are triggered from app orchestration handlers.
- Storage provides selected media lookup by media id.
- Media id comes from explicit selection state populated by search/gallery/timeline route panels.
- UI remains a thin input/button surface.
- Actions are presented in a dedicated details-pane section rather than mixed with browsing content.
- Shortcut events are consumed through Iced `keyboard::listen` (ignored events only), so focused text inputs keep normal copy behavior.

## Platform behavior

- macOS:
  - open: `open`
  - copy file: `osascript` (`set the clipboard to POSIX file ...`)
  - copy path: `pbcopy`
- Windows:
  - open: `cmd /C start`
  - copy file: `powershell Set-Clipboard -LiteralPath`
  - copy path: `clip`
- Linux/other Unix:
  - open: `xdg-open`
  - copy file: `xclip` with `x-special/gnome-copied-files` payload
  - copy path: `xclip -selection clipboard`

## Non-destructive guarantee

- Actions only read source file paths and invoke OS handlers.
- No source-file writes, renames, moves, or metadata mutation.
