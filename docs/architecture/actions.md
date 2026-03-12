# Media Actions Architecture

Librapix exposes simple file-oriented actions without mutating source media.

## Baseline actions

- Open selected media with system default app.
- Open selected media containing folder.
- Copy selected media file to clipboard (file object copy, not just path text).
- Copy selected media path to clipboard via platform command.
- Generate a video short (`Make Short`) from the selected video through ffprobe/ffmpeg background tasks.
- Header update-status chip action:
  - click opens latest release page when a newer release is available
  - otherwise click triggers a manual release re-check (5-minute cooldown)
- Keyboard shortcuts:
  - `Cmd/Ctrl+C`: copy selected media file
  - `Cmd/Ctrl+Shift+C`: copy selected media path

## Library actions

- Edit library opens the unified add/edit library dialog (path/display-name/root-level tag chips).
- Stats opens a separate Library Statistics dialog for the selected library/root.
- Library statistics are read from maintained persisted storage values; opening the dialog is read-only and does not trigger indexing work.

## Tag and rule interactions

- Library and Details tag management use chip-based add/edit/remove interactions.
- Settings ignore-rule management uses chip-based add/edit/remove plus explicit enable/disable toggle.
- Chip actions emit explicit app messages; storage side effects stay in orchestration handlers.

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
- Video command building and process invocation are delegated to `librapix-video-tools`; UI only owns dialog state and message dispatch.
- Actions are presented in a dedicated details-pane section rather than mixed with browsing content.
- Shortcut events are consumed through Iced `keyboard::listen` (ignored events only), so focused text inputs keep normal copy behavior.

## Platform behavior

- macOS:
  - open: `open`
  - copy file: `osascript` (`set the clipboard to POSIX file ...`)
  - copy path: `pbcopy`
- Windows:
  - open: `cmd /C start`
  - copy file: native Win32 clipboard write (`CF_HDROP` via `SetClipboardData`) with explicit failure handling
  - copy path: `clip`
- Linux/other Unix:
  - open: `xdg-open`
  - copy file: `xclip` with `x-special/gnome-copied-files` payload
  - copy path: `xclip -selection clipboard`

## Non-destructive guarantee

- Actions only read source file paths and invoke OS handlers.
- No source-file writes, renames, moves, or metadata mutation.
