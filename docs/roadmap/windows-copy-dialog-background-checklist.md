# Windows Copy, Dialog, and Background Responsiveness Checklist

Milestone date: 2026-03-07

- [x] Investigate current Windows copy-file clipboard path and confirm root cause
- [x] Implement Windows file-copy clipboard fix (actual file object copy)
- [x] Verify copy-path behavior remains intact
- [x] Redesign new-file dialog layout as centered/constrained modal
- [x] Investigate remaining blocking operations on UI thread
- [x] Move large operations off blocking path via background tasks where needed
- [x] Add/adjust explicit loading or activity indicators for background operations
- [x] Update tests for new behavior (clipboard command selection, background task routing, modal layout helpers where testable)
- [x] Run verification loop (`fmt`, `check`, `clippy -D warnings`, `test`) after each meaningful step
- [x] Run app smoke checks after each milestone step (`cargo run -p librapix-app`)
- [x] Update docs (`README.md`, `CHANGELOG.md`, docs index/architecture/troubleshooting/dependencies)
- [ ] Commit meaningful checkpoints
- [ ] Final verification loop and clean working tree
