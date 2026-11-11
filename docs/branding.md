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
