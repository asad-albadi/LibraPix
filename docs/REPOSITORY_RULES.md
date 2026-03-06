# Repository-Level Engineering Rules

This document defines additional repository-wide engineering rules for Librapix.

These rules apply across the entire codebase and are separate from feature-specific implementation guidance.

## 1. Minimum Supported Rust Version (MSRV)

Librapix must define and document a **Minimum Supported Rust Version (MSRV)**.

### Rule
- The project must explicitly declare its MSRV.
- The MSRV must be recorded in:
  - `README.md`
  - `Cargo.toml` where appropriate
  - relevant CI or contributor documentation
- The MSRV must not change casually or implicitly.
- Any MSRV update must be **intentional, reviewed, and documented**.

### Why this exists
Pinning an MSRV provides:
- predictable build expectations for contributors
- reproducible development environments
- clearer compatibility guarantees
- less accidental drift caused by adopting newer language or library features without discussion

### Requirements
- Choose an MSRV deliberately.
- Document the reason for the chosen MSRV.
- Keep it aligned with the needs of the codebase and selected dependencies.
- If a dependency requires a higher Rust version, document:
  - which dependency caused the change
  - why the upgrade is necessary
  - which version changed
  - what contributors need to do

### MSRV update policy
When updating the MSRV:
1. verify the need for the update
2. confirm the impact on contributors and CI
3. update all relevant documentation
4. record the change in `CHANGELOG.md`
5. explain the reason in an ADR or equivalent architectural note if the change is significant

### Implementation guidance
At minimum, the repository should:
- define the MSRV in a visible, canonical location
- test against it where practical
- avoid using newer Rust features unless the MSRV policy allows them

---

## 2. Dependency Documentation

Librapix must maintain a dedicated dependency decision record at:

`docs/DEPENDENCIES.md`

This file is mandatory.

### Rule
Every major crate or external dependency used by the project must be documented in `docs/DEPENDENCIES.md`.

The purpose is not to list every tiny transitive crate, but to document the important direct dependencies that shape architecture, behavior, maintenance, or user-facing functionality.

### Why this exists
This ensures:
- dependency choices are intentional
- maintainers understand why a crate is present
- future contributors and AI agents do not replace or duplicate dependencies blindly
- official documentation is treated as the source of truth
- architectural drift is reduced

### Each documented dependency entry must include
For every major dependency, record:

- **crate name**
- **purpose in Librapix**
- **why it was chosen**
- **what alternatives were considered** if relevant
- **official documentation consulted**
- **important usage notes or constraints**
- **known risks / tradeoffs**
- **version expectations or compatibility notes** if relevant

### Examples of dependencies that should be documented
This likely includes crates such as:
- `iced`
- SQLite-related crates
- configuration parsing crates
- localization/i18n crates
- filesystem scanning crates
- image/video metadata crates
- search-related crates
- logging/tracing crates
- error handling crates
- async/runtime crates, if used

### Documentation expectations
The dependency file must be updated whenever:
- a major dependency is added
- a major dependency is removed
- a major dependency is replaced
- the reason for using a dependency changes significantly

### Official documentation rule
Before adopting a new major dependency:
- read the official documentation
- verify that the crate is suitable for the intended use
- avoid relying on unofficial assumptions when official docs exist
- record the official documentation source in `docs/DEPENDENCIES.md`

### Suggested structure for `docs/DEPENDENCIES.md`
A recommended format is:

```md
# Dependencies

## iced
- Purpose:
- Why chosen:
- Alternatives considered:
- Official docs consulted:
- Notes:
- Risks / tradeoffs:

## sqlx
- Purpose:
- Why chosen:
- Alternatives considered:
- Official docs consulted:
- Notes:
- Risks / tradeoffs: