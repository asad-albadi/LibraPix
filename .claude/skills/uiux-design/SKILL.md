---
name: uiux-design
description: >-
  Apply UI/UX design fundamentals — usability, visual hierarchy, layout,
  spacing, color, typography, accessibility, and look-and-feel — when building
  or reviewing the desktop GUI. Use when adding/changing screens, widgets,
  styling, themes, or flows, or when asked to make something cleaner, clearer,
  or more pleasant to use.
---

# UI/UX Design Fundamentals

LibraPix is an Iced 0.14 desktop app. Pair this with [[rust-iced]] for the
implementation API. This skill is about *what good looks like* and *why*.

## Decision order (apply top-down)

1. **Usability first.** Can the user do the task without thinking? Clarity beats
   cleverness. If a choice is between pretty and obvious, pick obvious.
2. **Hierarchy second.** The eye should land on the primary action first.
3. **Polish last.** Spacing, color, motion — only after flow and hierarchy work.

## Usability — ease of use

- **One primary action per screen.** Make it the most prominent button; demote
  the rest to secondary/text styles. Never two equally-loud buttons.
- **Recognition over recall.** Show options (lists, icons + labels) instead of
  making users remember commands. Label icon-only buttons with tooltips.
- **Forgiveness.** Destructive actions (delete, overwrite) need confirmation or
  undo. Prefer undo over a modal nag.
- **Feedback within 100ms.** Every click acknowledges instantly — hover state,
  spinner, disabled-while-loading. Long work (indexing, thumbnails) shows
  progress, never a frozen window. Offload via `Task::perform`, never block
  `update`/`view`.
- **Empty / loading / error states are real states.** Design all three, not just
  the happy "has data" view. Empty state = a hint of what to do next.
- **Sensible defaults.** Pre-fill, remember last choice, minimize required input.

## Visual hierarchy

- Size, weight, and color create rank. Use **2–3 text sizes** max (e.g. title /
  body / caption) and **2 weights** (regular / semibold).
- Group related controls; separate unrelated ones with whitespace, not lines.
- Align everything to a grid. Ragged left edges read as broken.
- Primary action bottom-right (or bottom-center) of a form; cancel to its left.

## Spacing & layout

- **8px spacing scale**: 4, 8, 12, 16, 24, 32. Pick from the scale, don't
  freehand pixel values. In Iced: `.spacing(8)`, `.padding(16)`.
- **Generous padding > cramped density.** Whitespace is not wasted space; it's
  how the eye parses groups.
- Cap content/line width — text past ~70 chars per line is hard to scan.
- Consistent margins around the window edge (e.g. 16–24px).

## Color

- Define a small palette: 1 brand/accent, neutrals (bg / surface / border /
  text), and semantic (success/green, warning/amber, error/red). Don't invent
  colors per-widget — centralize in the theme/`palette`.
- Accent color = reserved for the primary action and active/selected state. If
  everything is accent-colored, nothing stands out.
- Contrast: body text vs background must meet **WCAG AA (4.5:1)**; large text
  3:1. Light gray text on white fails — don't.
- Support both light and dark themes; never hardcode `Color` in widgets — pull
  from `Theme`/palette so both themes work.

## Typography

- One UI font family. Use size + weight for hierarchy, not multiple families.
- Numbers in tables/counts: prefer tabular/monospaced alignment so columns line
  up.
- Don't center long text; left-align for readability (RTL-aware — this app has
  Arabic via `librapix-i18n`, so respect text direction).

## Accessibility & inclusivity

- Hit targets ≥ 32×32px (ideally 40+) — don't ship 16px clickable icons.
- Don't rely on color alone to convey meaning (add icon/label/text).
- Keyboard: every action reachable without a mouse; visible focus states; common
  shortcuts (Esc closes, Enter confirms).
- All user-facing strings go through `librapix-i18n` — no hardcoded text, and
  design layouts that tolerate longer translated strings (German/Arabic expand).

## Look & feel / polish

- **Consistency** is the cheapest path to "feels professional": same button
  shapes, same corner radius, same spacing rhythm everywhere.
- Subtle is better than flashy — gentle hover/press states, ~150ms transitions,
  soft shadows for elevation. Avoid bouncy animation in a productivity tool.
- Rounded corners (4–8px) read friendlier than sharp; pick one radius and reuse.
- Icons: one set, one stroke weight, consistent size.

## Review checklist

When reviewing a screen, ask:
- [ ] What's the one thing the user is here to do? Is it the most prominent?
- [ ] Empty, loading, and error states handled?
- [ ] Spacing from the 8px scale; aligned to a grid?
- [ ] Contrast passes AA in both light and dark?
- [ ] Works keyboard-only; hit targets big enough?
- [ ] Strings localized; layout survives longer text and RTL?
- [ ] No blocking work on the UI thread?
