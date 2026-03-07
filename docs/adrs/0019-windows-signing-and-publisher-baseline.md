# ADR 0019: Windows EXE Signing Baseline

## Status

Accepted

## Context

Windows users reported Librapix appearing as `Unknown publisher` during launch/install flows.

Changing UI labels or app title does not affect Windows publisher trust. Publisher identity comes from signing certificates and package identity metadata.

## Decision

- Establish a baseline Windows signing/distribution workflow under `packaging/windows/`.
- Set current publisher identity baseline to `CN=Asad`.
- Add scripts for operational signing workflow:
  - `New-LibrapixDevCertificate.ps1` (dev certificate generation/export)
  - `Sign-LibrapixBinary.ps1` (EXE signing + verification)
- Document local-dev certificate behavior and production signing expectations in `packaging/windows/README.md`.

## Alternatives considered

1. Keep unsigned binaries for development and release.
   - Rejected: does not solve publisher trust and keeps SmartScreen/unknown-publisher warnings.
2. Change visible publisher/app name only.
   - Rejected: cosmetic only; does not change cryptographic publisher identity.
3. Defer Windows signing setup to a later phase.
   - Rejected: user-visible trust warnings are a product distribution blocker.

## Consequences

- Windows builds now have a documented, repeatable signing baseline.
- Dev self-signed certs are sufficient for local testing but not public trust.
- Public releases still require trusted OV/EV code-signing certificates and timestamped signatures.
- Any future publisher identity change must be reflected in signing documentation and certificates.
