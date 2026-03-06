# Troubleshooting

## Auto refresh does not react to file changes

- Symptoms
  - Adding/modifying media files in active roots does not update gallery/timeline automatically.
  - Manual Index + Refresh still works.
- Affected area
  - Filesystem watch subscription and runtime message delivery.
- Likely cause
  - Filesystem events are detected, but the app does not receive the refresh message.
- Confirmed cause
  - The watcher worker used a blocking `std::sync::mpsc::recv()` inside an async Iced subscription stream.
  - This blocked runtime delivery of `Message::FilesystemChanged` even though events were detected.
- Resolution
  - Switched watcher event transport to async `iced::futures::channel::mpsc::unbounded`.
  - Replaced blocking `recv()` with `next().await`.
  - On `FilesystemChanged`, app now runs incremental indexing and refreshes gallery/timeline (and active search results).
- Prevention guidance
  - Avoid blocking std channels inside async subscription workers.
  - Use async stream/channel primitives for all Iced subscription event pipelines.

## Clipboard action fails on Linux

- Symptoms
  - Copy-path action reports failure while the app is otherwise healthy.
- Affected area
  - Media actions (clipboard integration).
- Likely cause
  - `xclip` command not installed on host OS.
- Confirmed cause
  - Baseline Linux clipboard flow invokes `xclip -selection clipboard`.
- Resolution
  - Install `xclip` package and retry copy action.
- Prevention guidance
  - Keep platform action dependencies documented and validate them in release notes.
