# Centralized LibraPix UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace LibraPix's monolithic style/view composition with a centralized, app-local, media-first shadcn-inspired Iced component system and independently persisted hybrid sidebars.

**Architecture:** Keep all Iced-specific UI code inside `librapix-app`, split into `ui/{tokens,styles,components,patterns}` and product composition under `views/`. Preserve existing messages and background behavior wherever possible; only pane toggles, persisted pane preferences, and a bounded shell notice require new presentation state.

**Tech Stack:** Rust 2024, Iced 0.14, serde/TOML configuration, existing LibraPix SVG assets and i18n system.

**Spec:** `docs/superpowers/specs/2026-11-11-centralized-ui-design.md`

## Global Constraints

- Preserve the non-destructive guarantee: never modify, rename, move, or annotate source media files.
- Keep Rust MSRV at `1.85` and do not add dependencies unless the approved design is revised first.
- Use Iced 0.14 built-in widgets and closure-based styling; do not enable `advanced`, create a custom renderer, or add web UI technology.
- Keep the design system app-local; do not create a `librapix-ui` crate.
- Keep all user-facing strings in `librapix-i18n`.
- Preserve light, dark, and system themes and every existing product/background-work behavior.
- Keep the media grid, timeline virtualization, width-freeze, layout caches, selection semantics, and Make Short execution behavior unchanged.
- Implement only variants used by real LibraPix call sites.
- Run formatting, check, clippy, and tests serially before completion.

## Review Focus

- Legacy config without pane fields must load with both panes expanded; Task 2 pins this with a parse test.
- A preference-save failure must leave the visible pane toggled while exposing a dismissible error notice; Task 2 pins this with state-transition tests.
- Rapid independent left/right toggles must not affect route, selection, or trigger projection/indexing work; Task 4 pins this with transition tests.
- Collapsed controls must remain discoverable and keyboard-accessible through labels/tooltips; Task 4 pins the required rail/edge action inventory.
- Migrating media cards and shell composition must not defeat viewport-bounded rendering or layout caches; Task 7 adds render-helper boundary tests and Task 9 performs diagnostic/manual comparison.

---

## File Structure

### Create

- `crates/librapix-app/src/ui/mod.rs` - public design-system facade.
- `crates/librapix-app/src/ui/tokens/{mod,color,elevation,motion,radius,sizing,spacing,typography}.rs` - semantic constants and palette selection.
- `crates/librapix-app/src/ui/styles/{mod,button,checkbox,container,input,progress,scrollable,select,slider,text}.rs` - Iced status-to-style mapping.
- `crates/librapix-app/src/ui/components/{mod,badge,button,card,checkbox,empty_state,icon_button,input,loading,notice,select,separator,tabs,tooltip}.rs` - reusable controls.
- `crates/librapix-app/src/ui/patterns/{mod,details_pane,dialog,field,sidebar,status}.rs` - recurring LibraPix compositions.
- `crates/librapix-app/src/views/mod.rs` - product-view facade.
- `crates/librapix-app/src/views/{shell,library,details}.rs` - shell and primary product surfaces.
- `crates/librapix-app/src/views/dialogs/{mod,about,filters,library,library_statistics,make_short,new_media,settings}.rs` - dialog compositions.
- `docs/adrs/0025-centralized-app-ui-system.md` - selected boundary and rejected alternatives.

### Modify

- `crates/librapix-app/src/main.rs` - state/update orchestration, module declarations, and progressive removal of render helpers.
- `crates/librapix-config/src/model.rs` - persisted `UiConfig` and app config field.
- `crates/librapix-config/src/lib.rs` - export and tests for backward-compatible UI preferences.
- `crates/librapix-i18n/src/lib.rs` - collapse, expand, details-edge, and preference-save-error keys.
- `CHANGELOG.md` - centralized UI system and collapsible panes.
- `docs/architecture/layers.md` - design-system and views boundaries.
- `docs/architecture/media-ui.md` - hybrid pane behavior and preserved responsive layout.
- `docs/architecture/message-flow.md` - pane toggle and preference-save flow.
- `.gitignore` - ignore `.superpowers/` visual brainstorming artifacts.

### Remove

- `crates/librapix-app/src/ui.rs` after all exports have moved and all call sites compile.

## Task 0: Prepare Isolated Execution and Record the Approved Design

**Files:**
- Add: `docs/superpowers/specs/2026-11-11-centralized-ui-design.md`
- Add: `docs/superpowers/plans/2026-11-11-centralized-ui.md`

**Interfaces:**
- Produces the reviewed specification and executable plan used by every later task.

- [ ] **Step 1: Create an isolated worktree and feature branch**

Use `superpowers:using-git-worktrees` at execution time. Start from current `master` and use branch `codex/centralized-ui-design-system` unless the maintainer supplies another branch name.

- [ ] **Step 2: Verify the worktree baseline**

Run: `git status --short --branch && cargo check --workspace`

Expected: clean feature worktree and successful workspace check.

- [ ] **Step 3: Commit the approved design and plan**

```bash
git add docs/superpowers/specs/2026-11-11-centralized-ui-design.md docs/superpowers/plans/2026-11-11-centralized-ui.md
git commit -m "docs: plan centralized LibraPix UI system"
```

## Task 1: Establish the Module Boundary and Preserve Existing Styles

**Files:**
- Create: all `ui/tokens/*.rs`, `ui/styles/*.rs`, and their `mod.rs` files listed above
- Create: `crates/librapix-app/src/ui/mod.rs`
- Modify: `crates/librapix-app/src/main.rs`
- Remove: `crates/librapix-app/src/ui.rs`
- Modify: `.gitignore`

**Interfaces:**
- Produces: `ui::tokens::{palette, Palette, SPACE_*, FONT_*, ICON_*, RADIUS_*, SIDEBAR_WIDTH, DETAILS_WIDTH, HEADER_HEIGHT, GALLERY_GAP}`.
- Produces: existing style function names re-exported from `ui::styles` so current `main.rs` call sites remain source-compatible during extraction.
- Produces: `ui::tokens::color::chip_tone_for_key(key: &str, dark: bool) -> ChipTone` with unchanged deterministic output.

- [ ] **Step 1: Add token regression tests before moving code**

Add `#[cfg(test)]` tests under `ui/tokens/color.rs` asserting that repeated tag keys map to equal tones, distinct representative keys map deterministically, and `palette(&Theme::Dark)`/`palette(&Theme::Light)` select the corresponding semantic palette.

- [ ] **Step 2: Run the focused tests and verify the new module is absent**

Run: `cargo test -p librapix-app ui::tokens -- --nocapture`

Expected: FAIL because `ui/tokens` does not exist yet.

- [ ] **Step 3: Move the existing palette, elevations, constants, text styles, widget styles, and chip tones into focused files**

Keep all concrete values unchanged in this task. `ui/mod.rs` must re-export the old names used by `main.rs`; do not apply the new visual direction yet.

- [ ] **Step 4: Switch `main.rs` from `mod ui;` backed by `ui.rs` to the directory module and remove `ui.rs`**

No render function or application state moves in this task.

- [ ] **Step 5: Ignore visual-companion output**

Add `.superpowers/` to `.gitignore`; do not remove the current local visual references.

- [ ] **Step 6: Verify extraction**

Run: `cargo fmt --all && cargo check -p librapix-app && cargo test -p librapix-app ui::tokens`

Expected: all commands PASS with no visual or behavioral changes.

- [ ] **Step 7: Commit**

```bash
git add .gitignore crates/librapix-app/src/main.rs crates/librapix-app/src/ui
git rm crates/librapix-app/src/ui.rs
git commit -m "refactor(ui): split design tokens and widget styles"
```

## Task 2: Add Persisted Pane State and Shell Notice State

**Files:**
- Modify: `crates/librapix-config/src/model.rs`
- Modify: `crates/librapix-config/src/lib.rs`
- Modify: `crates/librapix-i18n/src/lib.rs`
- Create: `crates/librapix-app/src/ui/components/notice.rs`
- Create: `crates/librapix-app/src/ui/components/mod.rs`
- Modify: `crates/librapix-app/src/main.rs`

**Interfaces:**
- Produces: `UiConfig { left_sidebar_collapsed: bool, details_sidebar_collapsed: bool }`, deriving `Default`, `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, and `Eq`.
- Produces: `AppConfig::ui: UiConfig` with `#[serde(default)]`.
- Produces messages: `ToggleLeftSidebar`, `ToggleDetailsSidebar`, `DismissShellNotice`.
- Produces state: `left_sidebar_collapsed: bool`, `details_sidebar_collapsed: bool`, `shell_notice: Option<ShellNotice>`.
- Produces: `ShellNotice { tone: NoticeTone, summary: String }` and `ui::components::notice::NoticeTone::{Info, Success, Warning, Error}`. This task defines the type; Task 3 adds its reusable renderer.
- Consumes: existing `load_from_path` and `save_to_path` configuration flow.

- [ ] **Step 1: Write failing config compatibility tests**

In `librapix-config/src/lib.rs`, add:

```rust
#[test]
fn ui_preferences_default_to_expanded_when_absent() { /* parse legacy TOML; assert both false */ }

#[test]
fn ui_preferences_round_trip() { /* serialize and parse true/false values; assert equality */ }
```

- [ ] **Step 2: Run config tests and verify failure**

Run: `cargo test -p librapix-config ui_preferences`

Expected: FAIL because `UiConfig` and `AppConfig::ui` do not exist.

- [ ] **Step 3: Implement `UiConfig` and backward-compatible `AppConfig::ui`**

Do not increment `schema_version`; the serde default makes this additive and backward compatible.

- [ ] **Step 4: Run config tests**

Run: `cargo test -p librapix-config`

Expected: PASS.

- [ ] **Step 5: Write failing application transition tests**

Add unit tests in `main.rs` around a pure helper:

```rust
fn apply_pane_preference(
    app: &mut Librapix,
    pane: Pane,
    collapsed: bool,
    save: impl FnOnce(&AppConfig) -> Result<(), String>,
);
```

Tests assert successful independent state/config updates and failed-save behavior: visible state remains changed, config in memory reflects it, and `shell_notice` becomes `NoticeTone::Error`.

- [ ] **Step 6: Run focused application tests and verify failure**

Run: `cargo test -p librapix-app pane_preference`

Expected: FAIL because the helper and state do not exist.

- [ ] **Step 7: Implement pane messages, state hydration, persistence helper, and notice dismissal**

Initialize pane state from `config.ui`. Use existing config file resolution and save functions. Log the full save error; show only localized concise text in the notice.

- [ ] **Step 8: Add i18n keys**

Add keys for Collapse Libraries, Expand Libraries, Collapse Details, Expand Details, Details, Add Library, Refresh, Dismiss, and `Could not save layout preference.` Reuse existing keys where exact wording already exists.

- [ ] **Step 9: Verify and commit**

Run: `cargo fmt --all && cargo test -p librapix-config && cargo test -p librapix-app pane_preference`

```bash
git add crates/librapix-config crates/librapix-i18n/src/lib.rs crates/librapix-app/src/main.rs crates/librapix-app/src/ui/components
git commit -m "feat(ui): persist collapsible pane preferences"
```

## Task 3: Build the Central Component Primitives

**Files:**
- Create: all remaining `ui/components/*.rs` files listed in File Structure
- Modify: `crates/librapix-app/src/ui/components/notice.rs`
- Modify: `crates/librapix-app/src/ui/mod.rs`
- Modify: `crates/librapix-app/src/ui/styles/*.rs`

**Interfaces:**
- Produces: `ButtonVariant::{Primary, Secondary, Outline, Ghost, Destructive, Link}`.
- Produces: `ButtonSize::{Small, Default, Large, Icon}`.
- Produces: `BadgeVariant::{Default, Secondary, Outline, Success, Warning, Destructive}`.
- Produces: `CardVariant::{Default, Interactive, Selected}`.
- Produces: `pub fn app_button<'a, Message: Clone + 'a>(content: impl Into<Element<'a, Message>>, on_press: Option<Message>, variant: ButtonVariant, size: ButtonSize) -> iced::widget::Button<'a, Message>`.
- Produces: `pub fn icon_button<'a, Message: Clone + 'a>(icon: Element<'a, Message>, label: &'a str, on_press: Option<Message>, variant: ButtonVariant) -> Element<'a, Message>`; the returned element always includes a tooltip.
- Produces centralized constructors for badge, card, input, select, checkbox, tabs, separators, tooltip, loading, empty state, and notice using only variants required by current call sites.

- [ ] **Step 1: Write failing variant/style tests**

Test each button and badge variant against light/dark themes and status transitions. Assert disabled button construction occurs when `on_press` is `None`, destructive uses the destructive semantic token, and selected cards use the focus/selection token.

- [ ] **Step 2: Run component tests and verify failure**

Run: `cargo test -p librapix-app ui::components`

Expected: FAIL because component modules and variants do not exist.

- [ ] **Step 3: Implement button, icon button, badge, card, input, select, checkbox, tabs, separator, and tooltip components**

Use built-in Iced widgets. Keep constructors concise; do not reproduce every Iced builder method.

- [ ] **Step 4: Implement loading, empty-state, and notice components**

The notice is an overlay-friendly card with tone, summary, and optional dismiss message. Loading supports compact indicator, indeterminate working row, and skeleton block without claiming numeric progress.

- [ ] **Step 5: Apply the approved media-first semantic token values**

Retain LibraPix blue as primary/focus/selection, use quieter neutral surfaces and one-pixel borders, and keep gallery backgrounds darker than raised controls. Verify WCAG-conscious text contrast manually; do not add a color library.

- [ ] **Step 6: Verify and commit**

Run: `cargo fmt --all && cargo check -p librapix-app && cargo test -p librapix-app ui::components`

```bash
git add crates/librapix-app/src/ui
git commit -m "feat(ui): add centralized component primitives"
```

## Task 4: Implement the Hybrid Shell and Collapsible Panes

**Files:**
- Create: `crates/librapix-app/src/ui/patterns/sidebar.rs`
- Create: `crates/librapix-app/src/ui/patterns/details_pane.rs`
- Create: `crates/librapix-app/src/views/shell.rs`
- Create: `crates/librapix-app/src/views/mod.rs`
- Modify: `crates/librapix-app/src/ui/patterns/mod.rs`
- Modify: `crates/librapix-app/src/main.rs`

**Interfaces:**
- Consumes Task 2 pane state/messages and Task 3 components.
- Produces: `SidebarMode::{Expanded, Rail}` and `DetailsPaneMode::{Expanded, Edge}`.
- Produces: `GlobalRailAction::{Library, AddLibrary, Refresh, Expand}`; no dynamic library-root action is valid in rail mode.
- Produces: `views::shell::view(app: &Librapix) -> Element<'_, Message>`.
- Produces presentation helpers that accept already-built content/messages rather than storage or domain services.

- [ ] **Step 1: Write failing shell-state tests**

Add tests with names prefixed `pane_` asserting:

- `SidebarMode::Rail` maps exactly to Library, Add Library, Refresh, Expand.
- `DetailsPaneMode::Edge` exposes Details and Expand Details.
- toggling either pane preserves route and selected media ID.
- toggling does not enqueue indexing or projection work.

- [ ] **Step 2: Run focused tests and verify failure**

Run: `cargo test -p librapix-app pane_`

Expected: FAIL because modes and shell view do not exist.

- [ ] **Step 3: Implement sidebar and details-pane patterns**

Expanded mode preserves all current content. Rail mode contains global actions only with localized tooltips. Details edge uses vertical `Details` labeling if Iced text rotation is practical; otherwise use a compact details icon plus tooltip and horizontal label without a custom renderer.

- [ ] **Step 4: Move top-level shell composition from `main.rs::view` to `views/shell.rs`**

Keep update logic and product state in `main.rs`. Place the shell notice as a bottom-right overlay that does not resize media content.

- [ ] **Step 5: Verify resize behavior manually**

Run: `cargo run -p librapix-app`

Expected: either pane toggles independently; gallery/timeline naturally reflow; selection and route remain stable; restarting restores both states.

- [ ] **Step 6: Verify and commit**

Run: `cargo fmt --all && cargo check --workspace && cargo test -p librapix-app`

```bash
git add crates/librapix-app/src/main.rs crates/librapix-app/src/views crates/librapix-app/src/ui/patterns
git commit -m "feat(ui): add hybrid collapsible application panes"
```

## Task 5: Build Shared Field, Dialog, and Status Patterns

**Files:**
- Create: `crates/librapix-app/src/ui/patterns/dialog.rs`
- Create: `crates/librapix-app/src/ui/patterns/field.rs`
- Create: `crates/librapix-app/src/ui/patterns/status.rs`
- Modify: `crates/librapix-app/src/ui/patterns/mod.rs`
- Modify: `crates/librapix-app/src/main.rs`

**Interfaces:**
- Produces: `DialogSpec<'a, Message> { title, close_message, body, footer, max_width, max_height }` and `dialog(spec) -> Element<'a, Message>`.
- Produces: `FieldSpec<'a, Message> { label, control, help, error }` and `field(spec) -> Element<'a, Message>`.
- Produces: `StatusTone::{Neutral, Working, Success, Warning, Error}` and reusable status row.
- Replaces: `render_dialog_frame`, `render_modal_overlay`, `render_help_badge`, and repeated label/help/error rows after call-site migration.

- [ ] **Step 1: Write failing pattern contract tests**

Test pure `DialogParts`/`FieldParts` decision helpers: footer optionality, error taking precedence over help text, close action presence, and Working status never exposing a determinate percentage.

- [ ] **Step 2: Run tests and verify failure**

Run: `cargo test -p librapix-app ui::patterns`

Expected: FAIL because shared patterns do not exist.

- [ ] **Step 3: Implement dialog, field, and status patterns**

Keep dialog layout stable: header, bounded scrollable body, separator, optional footer. All text inputs retain existing IDs and message wiring.

- [ ] **Step 4: Migrate one representative dialog and compare behavior**

Migrate About first because it has no form validation or background state. Remove only superseded local composition.

- [ ] **Step 5: Verify and commit**

Run: `cargo fmt --all && cargo check -p librapix-app && cargo test -p librapix-app ui::patterns`

```bash
git add crates/librapix-app/src/ui/patterns crates/librapix-app/src/main.rs
git commit -m "feat(ui): centralize dialog field and status patterns"
```

## Task 6: Migrate Every Dialog to Shared Components

**Files:**
- Create: all files under `crates/librapix-app/src/views/dialogs/`
- Modify: `crates/librapix-app/src/views/mod.rs`
- Modify: `crates/librapix-app/src/main.rs`

**Interfaces:**
- Consumes Task 3 components and Task 5 patterns.
- Produces one `view(app: &Librapix) -> Element<'_, Message>` function per dialog module.
- Preserves existing dialog state, messages, validation, tooltips, loading, cancellation, success, failure, and close behavior.

- [ ] **Step 1: Add/retain presentation-state tests before moving dialogs**

Add tests with names prefixed `dialog_` that pin the following behaviors: filter chips reflect active filters; library validation remains visible; smooth warning is shown only when Smooth is selected; Make Short generating state contains one primary status line, one indeterminate indicator, and Cancel; canceled is distinct from failed; new-media loading text remains meaningful.

- [ ] **Step 2: Run focused tests as baseline**

Run: `cargo test -p librapix-app dialog_`

Expected: existing relevant tests PASS; newly added composition decision tests FAIL until modules exist.

- [ ] **Step 3: Migrate settings, filters, library, and statistics dialogs**

Use shared fields, buttons, tabs/chips, separators, and dialog frames. Do not change save/update logic.

- [ ] **Step 4: Migrate new-media and Make Short dialogs**

Preserve background execution and cancellation exactly. Use the centralized loading component and truthful stage text; do not introduce numeric progress.

- [ ] **Step 5: Remove migrated dialog render helpers from `main.rs`**

Leave all update handlers and request-building functions in orchestration code.

- [ ] **Step 6: Verify visually in light and dark themes**

Run: `cargo run -p librapix-app`

Expected: every dialog opens, validates, scrolls, loads, cancels, succeeds/fails, and closes as before with consistent framing.

- [ ] **Step 7: Verify and commit**

Run: `cargo fmt --all && cargo check --workspace && cargo test --workspace`

```bash
git add crates/librapix-app/src/views/dialogs crates/librapix-app/src/views/mod.rs crates/librapix-app/src/main.rs
git commit -m "refactor(ui): migrate dialogs to shared components"
```

## Task 7: Migrate Media, Details, Navigation, and Runtime Status UI

**Files:**
- Create: `crates/librapix-app/src/views/library.rs`
- Create: `crates/librapix-app/src/views/details.rs`
- Modify: `crates/librapix-app/src/views/shell.rs`
- Modify: `crates/librapix-app/src/main.rs`
- Modify: relevant `ui/components/*.rs` only for variants proven necessary by these call sites

**Interfaces:**
- Consumes centralized cards, badges, tabs, chips, buttons, inputs, loading, empty states, notices, sidebar/details patterns.
- Produces: `views::library::view(app: &Librapix) -> Element<'_, Message>` and `views::details::view(app: &Librapix) -> Element<'_, Message>`.
- Keeps justified layout, viewport calculations, timeline scrubber, and cache ownership in existing functions/modules unless moving them is mechanical and test-covered.

- [ ] **Step 1: Add regression tests around presentation decisions**

Add tests with names prefixed `presentation_` for video-only Make Short visibility, selected-card variant mapping, empty-state selection, grid/timeline active-tab mapping, and render-window helper output for representative viewport bounds before moving code.

- [ ] **Step 2: Run focused tests as baseline**

Run: `cargo test -p librapix-app presentation_`

Expected: baseline tests PASS; new centralized mapping tests FAIL until migration.

- [ ] **Step 3: Migrate header, navigation, search, filters, tabs, badges, and runtime activity**

Remove direct recurring low-level styles from call sites. Keep route and search messages unchanged.

- [ ] **Step 4: Migrate details panel and actions**

Preserve tags, metadata, OS actions, keyboard hints, video-only Make Short action, and action status.

- [ ] **Step 5: Migrate media cards, loading blocks, and empty states**

Do not introduce additional per-card dynamic allocations or wrappers beyond equivalent current composition. Preserve selection border/check behavior and double-click action.

- [ ] **Step 6: Compare runtime diagnostics**

Run representative large Library Grid and Timeline sessions and compare `interaction.surface_render.window` / timeline window diagnostics before and after. Expected: bounded visible/overscan behavior remains unchanged.

- [ ] **Step 7: Verify and commit**

Run: `cargo fmt --all && cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/librapix-app/src/views crates/librapix-app/src/main.rs crates/librapix-app/src/ui
git commit -m "refactor(ui): migrate library and details surfaces"
```

## Task 8: Finish View Extraction and Remove Legacy UI Paths

**Files:**
- Modify: `crates/librapix-app/src/main.rs`
- Modify: all `crates/librapix-app/src/views/*.rs`
- Modify: all `crates/librapix-app/src/ui/**/*.rs`

**Interfaces:**
- Produces a `main.rs` focused on state, messages, update orchestration, subscriptions, and background tasks.
- Ensures reusable visual composition is under `ui/`; product composition is under `views/`.
- Removes obsolete compatibility re-exports and superseded style/render helpers.

- [ ] **Step 1: Inventory remaining direct recurring widget styles and render helpers**

Run: `rg -n "\.style\(|fn render_|fn view" crates/librapix-app/src/main.rs`

Expected: only intentionally product-specific or one-off composition remains; document each retained direct style in the commit body if any.

- [ ] **Step 2: Move remaining product-only render functions into the appropriate view module**

Do not move update, persistence, indexing, projection, thumbnail, or ffmpeg logic.

- [ ] **Step 3: Remove compatibility exports and dead code**

Run: `cargo clippy -p librapix-app --all-targets -- -D warnings` after each removal batch.

- [ ] **Step 4: Verify no duplicate visual vocabulary remains**

Run: `rg -n "primary_button_style|subtle_button_style|action_button_style|render_dialog_frame|render_modal_overlay|render_help_badge" crates/librapix-app/src`

Expected: no obsolete symbols remain.

- [ ] **Step 5: Verify and commit**

Run: `cargo fmt --all && cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/librapix-app/src
git commit -m "refactor(ui): complete centralized view composition"
```

## Task 9: Document, Validate, and Polish the Delivered System

**Files:**
- Create: `docs/adrs/0025-centralized-app-ui-system.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/architecture/layers.md`
- Modify: `docs/architecture/media-ui.md`
- Modify: `docs/architecture/message-flow.md`
- Modify: `docs/TROUBLESHOOTING.md` only for confirmed migration issues

**Interfaces:**
- Documents final boundaries and behavior; no new runtime interface.

- [ ] **Step 1: Write ADR 0025**

Record context, selected app-local design system, media-first shadcn inspiration, rejected separate crate/styles-only alternatives, and consequences for future UI work.

- [ ] **Step 2: Update architecture and changelog documentation**

Document tokens/styles/components/patterns/views boundaries, pane toggle persistence flow, hybrid collapsed states, and unchanged non-destructive/background-work behavior.

- [ ] **Step 3: Run the full serial validation loop**

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands PASS.

- [ ] **Step 4: Run the desktop visual matrix**

Manually verify on Windows and, where available, Linux:

- Light, Dark, and System themes
- common and narrow window widths
- both panes expanded, each collapsed independently, and both collapsed
- persistence after restart
- all dialogs and validation states
- loading, warning, error, success, canceled, disabled, and empty states
- Grid and Timeline reflow and scrollbar behavior
- keyboard shortcuts, focus visibility, icon tooltips, and double-click behavior
- Make Short generation/cancellation remains responsive and console-free on Windows

- [ ] **Step 5: Inspect scope and cleanliness**

Run: `git diff --stat master...HEAD && git status --short`

Expected: only UI/config/i18n/docs changes described by this plan; no untracked files except intentionally ignored `.superpowers/`; working tree clean after commit.

- [ ] **Step 6: Commit**

```bash
git add CHANGELOG.md docs/adrs/0025-centralized-app-ui-system.md docs/architecture/layers.md docs/architecture/media-ui.md docs/architecture/message-flow.md docs/TROUBLESHOOTING.md
git commit -m "docs: document centralized LibraPix UI system"
```

## Final Review Gate

- [ ] Confirm every completion criterion in the design spec maps to passing code/tests or a recorded manual check.
- [ ] Request a whole-branch review focused on behavior regressions, Iced lifetime complexity, pane persistence, accessibility, and render performance.
- [ ] Resolve review findings with focused commits.
- [ ] Re-run the full serial validation loop.
- [ ] Confirm the working tree is clean.

