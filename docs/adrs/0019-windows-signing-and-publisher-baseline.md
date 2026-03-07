# ADR 0019: Windows EXE Signing Baseline

## Status

Accepted

## Context

Windows users reported Librapix appearing as `Unknown publisher` during launch/install flows.

Changing UI labels or app title does not affect Windows publisher trust. Publisher identity comes from signing certificates and package identity metadata.

## Decision

- Set current publisher identity baseline to `CN=Asad`.
- Document the Windows signing/distribution expectations in repository docs and release notes.

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
