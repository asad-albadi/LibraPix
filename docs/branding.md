# Branding

LibraPix branding assets and usage guidelines.

## Canonical app name

**LibraPix** (PascalCase, no space).

## INDEX identity

The approved INDEX identity uses a custom lowercase wordmark, a standalone
geometric mark, and blue pixel accents. Product copy remains **LibraPix**.

All brand assets live under `assets/logo/index/`:

- `librapix-icon-dark.png` and `librapix-wordmark-dark.png`: black lettering for light surfaces.
- `librapix-icon-light.png` and `librapix-wordmark-light.png`: white lettering for dark surfaces.
- `icon.ico`, `icon.icns`, and sized PNGs: desktop packaging derivatives.

The app embeds theme-specific PNG handles centrally in `assets.rs`. The header
uses the custom wordmark; About uses the mark and wordmark. The README uses a
`picture` element to select the appropriate foreground for GitHub's theme.
See the asset directory README for production details. No legacy logo set remains.

## Placement Policy And References

`ui/components/brand.rs` owns shared Header, AboutWordmark, and AboutMark
placements. It composes native Iced widgets without app state, persistence,
new dependencies, or a custom renderer.

- Retain the native 52px header. Wordmark artwork is 36px high and approximately
  127.38px wide (36 * 1893 / 535), centered with at least 8px above and below.
  Retain the shell's 24px left inset and at least 8px to the next element.
- About uses a 32px-high wordmark (approximately 113.23px wide) beside a
  44px-high mark (approximately 29.26px wide, using 554 / 833), with an 8px gap.
  The mark centers against the wordmark/version group, not an arbitrary square.
- Width derives from the asset ratio and target height; native Shrink height
  and Contain fitting reduce artwork proportionally if width is limited.
  Never stretch, crop, redraw, or separately constrain unrelated dimensions.
- Preserve dark-on-light / white-on-dark variants and blue accents. System
  appearance selects these same variants through the existing theme state.

These are **LibraPix-specific design decisions**, not universal logo standards.
[IBM's 8-bar guidance](https://www.ibm.com/design/language/ibm-logos/8-bar/)
informs unchanged art, isolation/clearspace, alignment, and positive/reverse usage;
its measurements and geometry apply to IBM, not LibraPix.
[Carbon header specifications](https://www.carbondesignsystem.com/building-blocks/core/components/ui-shell-header/specifications)
provide a precedent for deliberate spacing (48px header, 16px left and 32px right
product padding), not a web framework to transplant into this desktop app.
We retain our header, insets, and native composition instead.
[Iced 0.14 Image](https://docs.rs/iced/0.14.0/iced/widget/image/struct.Image.html)
documents aspect-preserving image layout. Latest stable Iced was verified as
0.14.0 for this correction; no dependency change is required.

### Asset Fidelity

Compared the integration wordmark against the approved INDEX board
`exec-2449221a-21b8-48eb-aef5-b642e5095563.png`. It retains the lowercase geometric
direction and two blue accents, but the standalone master was generated from
the board, not mechanically extracted vector art. Glyph contours/spacing may
vary, notably the terminal x and letter joins; exact board fidelity is not proven.
This correction changes layout only, not asset bytes. Do not stretch or redesign
it to compensate, or regenerate it without owner approval.

## Icon assets

UI icons live under `assets/icons/` with color variants:

- **White** (`assets/icons/white/*`): Default for dark UI surfaces (header, sidebar, cards, details).
- **Black** (`assets/icons/black/*`): Use on light backgrounds when needed.

### Icon set

| Icon        | Path                         | Usage                          |
|-------------|------------------------------|--------------------------------|
| Gallery     | `icons/white/gallary.png`     | Browse gallery tab             |
| Timeline    | `icons/white/timeline.png`    | Browse timeline tab            |
| Search      | `icons/white/search.png`      | Search                         |
| Refresh     | `icons/white/refresh.png`     | Refresh actions                |
| GitHub      | `icons/white/github.png`     | Header GitHub link             |
| Open        | `icons/white/open.png`       | Open file action               |
| Show in folder | `icons/white/show-in-folder.png` | Show in folder action       |
| Copy file   | `icons/white/copy-file.png`   | Copy file action               |
| Copy path   | `icons/white/copy-path.png`   | Copy path action               |
| Filter      | `icons/white/filter.png`     | Filter controls                |
| Filter remove | `icons/white/filter-remove.png` | Clear filter                 |
| Index       | `icons/white/index.png`       | Index Library action           |
| Type image  | `icons/white/type-image.png`  | Media kind badge (image)       |
| Type video  | `icons/white/type-video.png`  | Media kind badge (video)       |

## App integration

- **Header**: Theme-aware INDEX wordmark and GitHub repository action.
- **Sidebar**: White icons for Gallery, Timeline, Index Library.
- **Media pane**: White refresh icon, Filters button (opens filter dialog with type/extension/tag controls).
- **Details panel**: White icons for Open, Show in Folder, Copy File, Copy Path.
- **Media cards**: Type-image/type-video icons in card corner (no container/backdrop).

## Screenshots

Screenshots for README and marketing live in `assets/screenshots/`.
