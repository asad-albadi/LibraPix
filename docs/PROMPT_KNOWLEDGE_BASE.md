# LibraPix Prompt Knowledge Base

## Purpose

This document explains how prompts for LibraPix should be written for AI coding agents.

It is intended to help future agents generate high-quality, safe, architecture-aligned prompts for work on LibraPix without losing system context or violating project rules.

This file should be read together with:

1. `AGENTS.md`
2. `docs/AGENT_KNOWLEDGE_BASE.md`
3. relevant files under `docs/architecture/`
4. relevant ADRs
5. the current codebase

---

## 1. What LibraPix is

LibraPix is a cross-platform desktop media manager built with Rust + Iced.

It is:

- local-first
- non-destructive
- multi-library
- media-focused
- desktop-native
- architecture-driven
- documentation-driven

Prompts must always respect that LibraPix is **not** a web app and must not be treated like one.

Do not generate prompts that:
- assume backend/frontend deployment separation
- suggest writing into user media files
- suggest quick hacks that bypass architecture
- collapse clean subsystem boundaries

---

## 2. Core rules every prompt must preserve

Every prompt for LibraPix must preserve these project truths:

- source media files are read-only from the app’s perspective
- all tags, metadata state, ignore rules, stats, thumbnails, and app state are app-managed
- explicit state/message/update/view separation must remain intact
- heavy work must not block the UI
- docs must be updated with meaningful structural changes
- UX must remain media-first and product-like
- no hidden caps that break all-media aggregation or search completeness
- cross-platform behavior matters
- Windows, macOS, and Linux may need different implementation paths

If a prompt risks violating any of those, it is a bad prompt.

---

## 3. Prompt generation philosophy

A good LibraPix prompt should be:

- specific
- architecture-aware
- scoped
- non-destructive
- verification-oriented
- documentation-aware
- explicit about stop conditions
- explicit about what must not break

A bad LibraPix prompt is:

- vague
- “just make it work”
- silent about docs
- silent about checks/tests
- silent about architecture boundaries
- UI-only with no state/update/storage awareness
- technically correct but product-ignorant
- product-desired but architecture-breaking

---

## 4. Default structure every serious LibraPix prompt should follow

A strong LibraPix prompt should usually contain these sections:

### A. Objective
State exactly what feature/fix/change is required.

### B. Product behavior
Describe how the feature should behave from the user’s perspective.

### C. Architecture constraints
State what must not break:
- shell layout
- state/message/update/view separation
- storage/indexing/query boundaries
- non-destructive guarantee

### D. Required reading first
Tell the agent exactly which docs/code to read before changing anything.

### E. Implementation requirements
Describe what the system should do technically.

### F. Checklist requirements
Require the agent to create/update a milestone checklist.

### G. Verification loop
Require:
- `cargo fmt --all`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

### H. Smoke-run rule
Require app launch verification after meaningful milestones.

### I. Documentation requirements
Specify which docs must be updated.

### J. Commit discipline
Require sensible commits with clear messages.

### K. Reporting style
Require milestone reports including:
- what changed
- docs changed
- checks run
- errors fixed
- smoke-run result
- commit created
- remaining gaps

### L. Stop condition
Define what “done” means.

---

## 5. Prompt categories

Most LibraPix prompts fall into one of these categories.

### 5.1 Bug-fix prompt
Use when something is wrong now.

Must include:
- exact observed failure
- expected behavior
- investigation requirement
- explicit instruction to find the root cause, not just patch the symptom

Example framing:
- “Diagnose exactly why X fails on Windows while it works on macOS”
- “Find why videos show in Videos but not in All”
- “Investigate whether hidden caps, filter logic, or projection queries are causing this”

### 5.2 Feature prompt
Use when adding a new product capability.

Must include:
- product intent
- user behavior
- architecture boundaries
- persistence/model implications
- documentation updates
- future-safe but not overengineered guidance

### 5.3 UX/refinement prompt
Use when behavior works but UX is bad.

Must include:
- what feels wrong
- what the target experience should feel like
- visual/system constraints
- component/layout expectations
- no-hack instruction

### 5.4 Documentation prompt
Use when building/reconciling docs.

Must include:
- exact file(s) to create/update
- required sections
- requirement to verify against real code
- clear distinction between implemented / deferred / partial

---

## 6. What a LibraPix prompt must always mention when relevant

Depending on the task, prompts should explicitly mention these system concerns.

### For indexing or browse changes
Mention:
- multi-root aggregation
- no hidden caps
- recursive traversal
- ignore rules
- min-size exclusion
- incremental indexing
- background/non-blocking behavior

### For media presentation changes
Mention:
- gallery
- timeline
- details pane
- shared media-view architecture
- thumbnail-first rendering
- no original-file loading in browse grids
- smooth interaction
- stable identity
- projection-driven behavior

### For tag changes
Mention:
- direct/manual tags
- library/root-level auto-tags
- inherited tag behavior if relevant
- storage model implications
- filter/search integration

### For Windows/macOS/Linux behavior
Mention:
- platform-specific differences
- expected native behavior
- real runtime correctness, not fake success
- packaging/signing if relevant

### For update-checker behavior
Mention:
- startup check
- periodic re-check
- manual re-check cooldown
- latest GitHub release endpoint
- clear state model

---

## 7. Prompt language rules

When generating prompts for LibraPix:

### Do say:
- “diagnose the actual root cause”
- “do not guess”
- “do not stop after planning”
- “implement all requested items below”
- “keep the working tree clean”
- “do not break the shell layout”
- “keep business logic out of widgets”
- “update all relevant docs”
- “fix all errors before moving on”

### Do not say:
- “just fix it quickly”
- “do whatever is easiest”
- “skip docs”
- “ignore tests”
- “no need to verify”
- “rewrite everything”
- “hardcode it for now”

---

## 8. Prompt stop-condition rules

Every substantial LibraPix prompt should define a stop condition.

A good stop condition says:
- exactly what must be implemented
- checks must pass
- smoke run must pass
- docs must be reconciled
- commits must be made
- working tree must be clean

This prevents partial, misleading completion.

---

## 9. Required reporting format for prompts

Prompts should require milestone reports with this shape:

- milestone name
- root cause(s) found, if a bug-fix task
- what changed
- what docs changed
- what checks were run
- what errors were found and fixed
- whether smoke-run passed
- what commit was created
- updated checklist status
- next milestone being started

Also include:
- Visual MVP status: yes/no
- Technical MVP status: yes/no
- Remaining UX gaps

This reporting format works well for LibraPix and should remain the default.

---

## 10. How to write prompts for this specific system

When writing a LibraPix prompt, first answer these questions:

1. Is this a bug, feature, UX refinement, or docs task?
2. Which subsystem(s) does it touch?
   - app/UI
   - core state
   - storage
   - indexer
   - projections
   - search
   - thumbnails
   - i18n
   - packaging/platform
3. Does it affect:
   - non-destructive guarantee?
   - multi-root behavior?
   - background task behavior?
   - platform-specific behavior?
   - persisted data?
   - docs/ADRs?
4. What is the product-visible expected behavior?
5. What exact files/docs should be read first?
6. What is the clear stop condition?

If a prompt cannot answer those, it is probably not good enough yet.

---

## 11. Prompt template for LibraPix

Use this as the default prompt template:

### Objective
Explain exactly what is required.

### Product behavior
Describe the expected user-facing behavior.

### Architecture constraints
State what must not break.

### Required reading first
List docs and code paths to inspect before coding.

### Implementation requirements
Describe what must be changed technically.

### Checklist requirements
Require a checklist to be created/updated.

### Verification loop
Require fmt/check/clippy/tests.

### Smoke-run rule
Require app startup verification after milestones.

### Documentation requirements
List the docs that must be updated.

### Commit discipline
Require sensible commits.

### Reporting style
Require milestone reports.

### Stop condition
Define done precisely.

---

## 12. Example micro-template

```text
Continue autonomously from the current repository state.

Objective:
[describe exact fix/feature]

Product behavior:
[describe user-facing expected behavior]

Architecture constraints:
- do not break shell layout
- keep business logic out of widgets
- preserve explicit state/messages/update/view
- preserve non-destructive behavior

Required reading first:
[list exact docs/code]

Implementation requirements:
[list exact work]

Checklist requirements:
[require checklist]

Verification loop:
- cargo fmt --all
- cargo check --workspace
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

Smoke-run rule:
- cargo run -p librapix-app
- verify relevant behavior
- stop cleanly

Documentation requirements:
[list docs]

Commit discipline:
[list expected commit style]

Reporting style:
[list expected report shape]

Stop condition:
[list exact done criteria]
````

---

## 13. Special LibraPix warning areas for future prompts

Future prompts must be especially careful around:

* browse aggregation correctness
* filter semantics (`All` must really mean all)
* timeline grouping and scrubber anchors
* background task/non-blocking behavior
* thumbnail quality and cache correctness
* Windows clipboard/file operations
* packaging/signing behavior on Windows
* root-level tags vs media-level tags
* persistent statistics maintained during indexing, not recomputed in UI
* UI polish that should not break current architecture

---

## 14. Final rule

A good LibraPix prompt is one that lets an agent make progress **without losing product intent, architectural integrity, or documentation discipline**.

If a prompt is not specific enough to preserve those things, rewrite it before using it.

