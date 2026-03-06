# Windows Publisher, Signing, and Distribution

This directory defines the baseline Windows packaging/signing workflow for Librapix.

## Why "Unknown publisher" happens

Windows shows `Unknown publisher` when a binary/package is unsigned or signed with an untrusted certificate.

For packaged apps (MSIX/AppX), the certificate subject must match the manifest publisher identity.

- Manifest identity: `Publisher="CN=Asad"` (see `msix/AppxManifest.xml`)
- Signing certificate subject: `CN=Asad`

## Baseline publisher identity

Librapix Windows publisher identity is currently:

- `CN=Asad`

If this changes, update both:

1. Certificate subject used by signing scripts.
2. Manifest `Identity Publisher` value.

## Prerequisites

- Windows SDK tools available in PATH:
  - `signtool.exe`
  - `makeappx.exe` (MSIX packaging flow)
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

## 2) Sign unpackaged EXE build

Build and sign the desktop executable:

```powershell
cargo build --release -p librapix-app
pwsh -File packaging/windows/scripts/Sign-LibrapixBinary.ps1 `
  -BinaryPath "target/release/librapix-app.exe" `
  -PfxPath "packaging/windows/certs/librapix-dev.pfx" `
  -Password "change-me"
```

## 3) Build + sign MSIX package

Create MSIX from a staging directory and sign it:

```powershell
pwsh -File packaging/windows/scripts/New-LibrapixMsix.ps1 `
  -StageDir "packaging/windows/stage" `
  -OutputMsix "packaging/windows/dist/Librapix.msix" `
  -PfxPath "packaging/windows/certs/librapix-dev.pfx" `
  -Password "change-me"
```

Expected staging contents include:

- `librapix-app.exe`
- assets referenced by `msix/AppxManifest.xml` (icons, etc.)
- `AppxManifest.xml` copied from `packaging/windows/msix/AppxManifest.xml`

## 4) Verify signature

```powershell
signtool verify /pa /v "target/release/librapix-app.exe"
signtool verify /pa /v "packaging/windows/dist/Librapix.msix"
```

## Release signing guidance

For production distribution:

- Use a trusted OV/EV code-signing certificate from a public CA.
- Keep subject aligned with manifest publisher (`CN=Asad`, or update both together).
- Timestamp signatures (`/tr` and `/td sha256`) so signatures remain valid after cert expiration.

## Official documentation consulted

- Microsoft SignTool reference:
  - <https://learn.microsoft.com/windows/win32/seccrypto/signtool>
- MSIX package signing guidance:
  - <https://learn.microsoft.com/windows/msix/package/signing-package-overview>
- Create certificate for package signing:
  - <https://learn.microsoft.com/windows/msix/package/create-certificate-package-signing>
