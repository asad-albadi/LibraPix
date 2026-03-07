# Windows Signing and Distribution

This directory defines the Windows signing workflow for Librapix EXE artifacts.

## Why "Unknown publisher" happens

Windows shows `Unknown publisher` when an executable is unsigned or signed with an untrusted certificate.

## Baseline signing identity

Current local baseline subject:

- `CN=Asad`

If this changes, update the certificate subject used by the signing scripts.

## Prerequisites

- Windows SDK tools available in PATH:
  - `signtool.exe`
- PowerShell 5+ (or PowerShell 7+)
- Rust target/toolchain for Windows builds

## 1) Dev certificate (local/testing)

Generate and optionally trust a self-signed code-signing cert:

```powershell
pwsh -File packaging/windows/scripts/New-LibrapixDevCertificate.ps1 `
  -Subject "CN=Asad" `
  -CertOutputPath "packaging/windows/certs/librapix-dev.pfx" `
  -Password "change-me"
```

Notes:

- A self-signed cert is valid for local testing only.
- On other machines, it must be imported into Trusted Root/Trusted People or SmartScreen will still warn.

## 2) Build + sign EXE

Build and sign the desktop executable:

```powershell
cargo build --release -p librapix-app
pwsh -File packaging/windows/scripts/Sign-LibrapixBinary.ps1 `
  -BinaryPath "target/release/librapix-app.exe" `
  -PfxPath "packaging/windows/certs/librapix-dev.pfx" `
  -Password "change-me"
```

## 3) Verify signature

```powershell
signtool verify /pa /v "target/release/librapix-app.exe"
```

## Icon metadata in EXE

`librapix-app` embeds Windows icon and version metadata at build time via `build.rs`:

- Icon: `assets/logo/blue/icon.ico`
- Product name: `LibraPix`

## Release signing guidance

For production distribution:

- Use a trusted OV/EV code-signing certificate from a public CA.
- Timestamp signatures (`/tr` and `/td sha256`) so signatures remain valid after cert expiration.

## Official documentation consulted

- Microsoft SignTool reference:
  - <https://learn.microsoft.com/windows/win32/seccrypto/signtool>
