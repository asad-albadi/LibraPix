# Troubleshooting

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
