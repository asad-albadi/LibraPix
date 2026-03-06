# Bootstrap Checklist

This checklist tracks the initial architecture baseline phase.

- [x] Inspect repository and current docs
- [x] Verify official docs for Iced, Cargo workspace, and Rust edition/MSRV baseline
- [x] Establish required documentation files and architecture baseline
- [x] Define and document MSRV policy (`1.85`)
- [x] Restructure project into workspace crates with clear dependency direction
- [x] Build i18n-ready Iced app shell with explicit state/message/update/view flow
- [x] Run full verification loop (`fmt`, `check`, `clippy`, `test`)
- [x] Run app startup smoke test and confirm clean initialization
- [x] Commit milestone checkpoint
