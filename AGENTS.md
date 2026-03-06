# AGENTS.md

# Librapix Agent Instructions

This document defines how AI agents must operate in the Librapix repository.

Librapix is a cross-platform FOSS desktop application built in Rust with Iced. It is a non-destructive local media gallery/manager for screenshots and recordings.

Agents working in this repository must follow these instructions strictly.

---

## 1. Project Identity

Librapix is:

- a desktop application
- cross-platform
- open source
- non-destructive by design
- documentation-driven
- structured for long-term maintainability
- built with Rust and Iced
- designed for screenshots, recordings, and local media libraries
- not limited to gaming, but gaming is a major use case

Librapix is **not** a web app, and must not be treated like one.

Do not impose unnecessary backend/frontend patterns as if this were a distributed application.

However, Librapix **must** still maintain strong internal architectural boundaries.

---

## 2. Core Product Rules

Librapix must support:

- multiple local library directories
- images and videos
- gallery view
- timeline view
- metadata inspection
- app-side tags
- game tags
- fuzzy search
- memories-style resurfacing
- ignore rules similar to gitignore/dockerignore
- copy/share/open-containing-folder workflows

Librapix must **never**:

- move user media files
- rename user media files
- rewrite metadata into user media files
- alter source folder structures
- depend on hacks or workarounds to make the architecture function

---

## 3. Non-Destructive Guarantee

This is a hard rule.

Agents must preserve the non-destructive nature of Librapix at all times.

### Required behavior
- Original media files are read-only from the application's perspective.
- Tags, albums, memories, ignore rules, and app state must be stored in Librapix-managed storage only.
- Thumbnail generation, cache data, indexing metadata, and search data must be stored only in app-owned storage.
- Metadata extraction from source media must be read-only.

### Forbidden behavior
- Writing tags into source files
- Renaming source files
- Reorganizing user directories
- Embedding app-specific metadata into user files unless explicitly approved in future architecture docs
- Any implementation shortcut that violates file safety

If a proposed change risks violating this rule, stop and document the concern before proceeding.

---

## 4. Engineering Principles

All work in this repository must follow these principles:

- simplicity first
- correctness first
- clear separation of concerns
- strong documentation
- maintainability over cleverness
- explicit behavior over hidden magic
- no overengineering
- no workaround-driven development
- no guessing
- no undocumented architecture

If the code becomes hard to explain, simplify it.

If a design requires a workaround, reassess the design.

If a dependency or API is unclear, verify it from official documentation before using it.

---

## 5. Official Documentation First

Before using any library, framework, crate, or external tool:

- read the official documentation
- verify current behavior from authoritative sources
- do not rely on memory alone
- do not rely on old tutorials when official docs exist
- do not assume examples from older versions still apply

This is especially important for:
- Iced
- SQLite-related crates
- search crates
- i18n/localization crates
- metadata extraction crates
- image/video crates
- async/task/runtime-related crates
- filesystem watching/scanning crates

When a major dependency is adopted, updated, or replaced:
- document it in `docs/DEPENDENCIES.md`
- record why it was chosen
- record which official documentation was consulted

---

## 6. Dependency Freshness Policy

Use the latest stable, appropriate versions of major dependencies unless there is a documented reason not to.

For Iced specifically:
- always verify the latest stable release before implementing or refactoring Iced-specific code
- use current official patterns
- do not introduce outdated APIs into the codebase

If an older version is intentionally chosen:
- document the reason
- document the constraint
- document the upgrade path if known

---

## 7. Required Documentation Discipline

Librapix is documentation-driven.

Agents must treat documentation as part of the product, not an afterthought.

### Mandatory files
The repository must maintain and update:

- `README.md`
- `CHANGELOG.md`
- `docs/README.md`
- `docs/TROUBLESHOOTING.md`
- `docs/DEPENDENCIES.md`
- `docs/REPOSITORY_RULES.md`
- `docs/architecture/overview.md`
- `docs/architecture/layers.md`
- `docs/architecture/message-flow.md`
- `docs/architecture/indexing.md`
- `docs/architecture/storage.md`
- `docs/architecture/i18n.md`
- `docs/architecture/ignore-rules.md`
- `docs/architecture/non-destructive-guarantee.md`
- `docs/roadmap/mvp.md`
- `docs/roadmap/future.md`
- `docs/adrs/`

### Structural rule
Every meaningful structural decision must be documented.

That includes:
- new crates
- new modules
- new architectural boundaries
- new storage models
- new indexing behavior
- new cache behavior
- new ignore-rule semantics
- new i18n mechanisms
- new message flow patterns
- new search behavior
- changes to dependency direction

If it changes how the system is built or understood, document it.

---

## 8. CHANGELOG Rules

`CHANGELOG.md` must be maintained continuously.

Meaningful changes must be recorded under structured headings such as:

- Added
- Changed
- Fixed
- Removed
- Docs

Do not leave changelog updates for later.

If an agent makes a meaningful repository change, it must update the changelog in the same workstream.

---

## 9. Troubleshooting Rules

`docs/TROUBLESHOOTING.md` is mandatory and must be updated whenever a real issue is discovered.

For each issue, document:

- symptoms
- affected area
- likely cause
- confirmed cause, if known
- resolution
- prevention guidance, if applicable

This file is intended to help future maintainers and future AI agents avoid repeating the same mistakes.

Do not treat troubleshooting notes as disposable.

---

## 10. Repository-Wide Rules

Agents must read and follow:

- `docs/REPOSITORY_RULES.md`

This includes:
- MSRV policy
- dependency documentation policy
- repository-wide engineering expectations

If repository rules and implementation convenience conflict, repository rules win.

---

## 11. Architecture Expectations

Librapix must have strong internal architectural boundaries.

The system should remain clearly separated between concerns such as:

- app/bootstrap
- UI/presentation
- application orchestration
- domain/business logic
- storage/infrastructure
- indexing/scanning
- search
- i18n
- config

### Hard rules
- UI must not own persistence logic
- widgets must not perform storage access directly
- domain logic must not depend on Iced-specific presentation types
- storage code must not leak into view code
- indexing logic must remain isolated from rendering
- message/update flow must remain explicit and understandable

---

## 12. Iced-Specific Guidance

Iced must be used according to current official architecture guidance.

Agents must structure Iced code around:
- state
- messages
- update logic
- view logic

### Rules
- do not create giant monolithic application files
- do not place all state and all messages into a single unmaintainable blob
- do not mix view composition with persistence logic
- keep widgets presentation-focused
- keep update logic explicit
- keep screen state understandable
- prefer simple, idiomatic current Iced patterns over clever abstractions

If the current Iced version changes the recommended approach, verify from official docs and update the architecture accordingly.

---

## 13. i18n Rules

Internationalization must be supported from the beginning.

### Required behavior
- all user-facing text must be keyed
- no scattered hardcoded UI strings
- locale switching must be architecturally supported
- fallback locale behavior must be designed clearly
- adding a new language later must not require major refactoring

### Documentation requirement
Any i18n mechanism must be documented in:
- `docs/architecture/i18n.md`

---

## 14. Search Rules

Search must be designed as a subsystem, not as a one-off screen feature.

It should support:
- filenames
- tags
- game names
- collections/albums
- indexed metadata where appropriate

Search behavior must remain replaceable and not tightly coupled to the UI.

Document search architecture and tradeoffs clearly.

---

## 15. Ignore Rule System

Ignore rules are a first-class capability.

They must be:
- centralized
- documented
- testable
- consistently applied by indexing/scanning

Examples include:
- `**/thumbnails/**`
- `**/cache/**`
- `**/*.tmp`

Do not spread ignore behavior across unrelated modules.

Document syntax, precedence, and evaluation behavior in:
- `docs/architecture/ignore-rules.md`

---

## 16. Storage Rules

Use SQLite as the primary application database unless a documented decision changes this.

Persistence code must:
- be isolated
- use documented migrations
- avoid leaking persistence concerns into UI code
- keep domain models and persistence models distinct where beneficial

Configuration should be kept in a structured config system such as TOML where appropriate.

Document all storage decisions.

---

## 17. MSRV Rules

Librapix must declare and document its minimum supported Rust version.

Agents must not casually introduce code that raises the MSRV.

If an MSRV increase becomes necessary:
- document why
- document impact
- update relevant files
- update `CHANGELOG.md`
- document the decision in architecture/repository docs as needed

---

## 18. Simplicity Rule

Simplicity is a hard rule.

Agents must prefer:
- fewer moving parts
- clearer names
- smaller modules
- straightforward data flow
- understandable ownership boundaries
- obvious behavior

Agents must avoid:
- speculative abstractions
- premature plugin systems
- unnecessary indirection
- "future-proofing" that adds real present-day complexity without clear benefit

Design for extension, but do not overbuild.

---

## 19. No Workarounds Rule

No workaround-based architecture is allowed.

If a library appears difficult to use:
- read the official docs again
- simplify the design
- verify the intended usage pattern
- document the limitation if needed

Do not patch over misunderstanding with brittle code.

If a workaround seems unavoidable, stop and document:
- the exact problem
- why it exists
- what was verified
- what alternatives were considered
- why the workaround is being proposed

Then wait for a deliberate decision.

---

## 20. How Agents Must Work

Before implementing meaningful changes, agents must:

1. inspect the current repository state
2. read relevant docs in `/docs`
3. read `AGENTS.md`
4. read `docs/REPOSITORY_RULES.md`
5. verify current official docs for major libraries being touched
6. identify affected architecture areas
7. update docs first when structural changes are involved
8. implement in small, reviewable steps
9. update changelog and troubleshooting docs where relevant

Agents must not jump directly into code without understanding the current architecture.

---

## 21. Required Reading Order

When starting work, agents should read in this order:

1. `AGENTS.md`
2. `README.md`
3. `docs/README.md`
4. `docs/REPOSITORY_RULES.md`
5. relevant files under `docs/architecture/`
6. relevant files under `docs/roadmap/`
7. `docs/TROUBLESHOOTING.md`
8. `docs/DEPENDENCIES.md`
9. then inspect code related to the task

If a document does not exist yet, create it when that area becomes active and note it in the changelog/docs updates.

---

## 22. Coding Workflow Expectations

When coding, agents must:

- keep changes scoped
- keep names clear
- preserve layering
- avoid hidden coupling
- add or update tests where reasonable
- update documentation as part of the same change
- keep the repository teachable to future maintainers

Agents must not:
- dump logic into `main.rs`
- create vague `utils` dumping grounds
- add dependencies casually
- ignore warnings without explanation
- leave architectural changes undocumented

---

## 23. Documentation Workflow Expectations

When making a structural change, agents must update the relevant documents in the same work.

Typical examples:

### New crate or subsystem
Update:
- `docs/architecture/overview.md`
- `docs/architecture/layers.md`
- relevant ADR
- `CHANGELOG.md`

### New dependency
Update:
- `docs/DEPENDENCIES.md`
- `CHANGELOG.md`

### New known issue
Update:
- `docs/TROUBLESHOOTING.md`

### New feature affecting MVP scope
Update:
- `docs/roadmap/mvp.md`
- `CHANGELOG.md`

---

## 24. ADR Rules

Use `docs/adrs/` for meaningful architecture decisions.

Create an ADR when:
- choosing a major dependency
- changing architectural boundaries
- changing storage strategy
- changing indexing strategy
- changing ignore-rule semantics
- changing i18n approach
- changing search architecture
- changing MSRV significantly

ADRs should explain:
- context
- decision
- alternatives considered
- consequences

---

## 25. Preferred Deliverable Style

When reporting work, agents should be explicit and structured.

Good outputs include:
- what changed
- why it changed
- what docs were updated
- what tradeoffs were made
- any open risks or next steps

Avoid vague summaries.

---

## 26. If Something Is Unclear

If requirements, architecture, or library behavior are unclear:

- do not invent details
- do not assume behavior
- verify from code and docs
- document uncertainty clearly
- prefer a smaller correct step over a broad uncertain one

---

## 27. Initial Priorities for Librapix

Unless explicitly instructed otherwise, early work should prioritize:

1. documentation foundation
2. architecture definition
3. workspace and layering decisions
4. non-destructive guarantees
5. storage and config design
6. indexing design
7. i18n readiness
8. simple, correct UI structure
9. MVP implementation planning
10. then feature implementation

---

## 28. Final Rule

Build Librapix like a serious long-term product.

Do not treat it like a throwaway prototype.
Do not choose convenience over structure.
Do not choose cleverness over clarity.
Do not choose speed over maintainability without documenting the tradeoff.