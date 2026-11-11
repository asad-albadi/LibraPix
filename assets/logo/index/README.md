# INDEX Brand Assets

Approved lowercase INDEX direction with blue pixel accents. These replace the
legacy black/blue/white logo directories, not the general UI icon set.

## Masters And Integration Files

- `librapix-icon-dark-source.png`: unchanged approved master, 1254 x 1254.
- `librapix-wordmark-dark-source.png`: unchanged approved master, 2172 x 724.
- `librapix-icon-dark.png` / `librapix-icon-light.png`: trimmed, 554 x 833.
- `librapix-wordmark-dark.png` / `librapix-wordmark-light.png`: trimmed, 1893 x 535.

`dark` means black artwork for light surfaces; `light` means white artwork for
dark surfaces. Both retain the blue accents. Always preserve aspect ratio.
The tightly cropped symbol is intentionally not square; use `icon-N.png` for
square window/desktop icons, not the header image.

The approved masters were originally generated as faithful standalone extractions
from the approved blue INDEX board. They are raster masters, not vector artwork.
No generative tool is used in this derivation, and no replacement SVG is invented.

Master SHA-256 checksums:

```text
2dc1489002980db73a53447b80b74fd26175e8e9de52b5922765c8ef3517e401  librapix-icon-dark-source.png
6642359327df278b3713c5a9fb6bd4b093da9cc5c4e63eefc3726faca6cfa511  librapix-wordmark-dark-source.png
```

## Reproduction

Using Python 3.11 and Pillow 10.4.0 (local authoring tools, not app dependencies),
run from the repository root:

```powershell
python -m pip install Pillow==10.4.0
python assets/logo/index/derive.py
```

The script resolves paths relative to itself and never writes the masters.
With the same Pillow version the derived files are reproducible byte-for-byte.

1. Clear only pixels with alpha <= 3/255 to remove nearly invisible stray noise.
2. Crop to remaining alpha bounds and add two transparent pixels on every side.
   Original crop bounds are (347, 223, 897, 1052) for the symbol and
   (146, 140, 2035, 671) for the wordmark (right/bottom exclusive).
3. Derive white artwork by setting non-blue visible RGB to (255, 255, 255).
   Blue is defined by B > R + 16 and B > G + 16 and is left byte-for-byte intact.
   The normalized dark/light pair has exactly identical alpha. This is a direct
   recolor, not a separately generated or reconstructed shape.
4. Center the dark symbol on a square with 5% nominal vertical clearance per side,
   preserving geometry. Resize in RGBA with Pillow's alpha-aware Lanczos filter.
5. Write `icon-16.png`, `icon-24.png`, `icon-32.png`, `icon-48.png`, `icon-64.png`,
   `icon-128.png`, `icon-256.png`, `icon-512.png`, and `icon-1024.png`.
6. Write `icon.ico` with 16/24/32/48/64/128/256 px 32-bit BMP frames and alpha/masks;
   write `icon.icns` with PNG-backed 32/64/128/256/512/1024 px representations,
   including Pillow's standard Retina entries.

## Quality And Limitations

The normalization removes the source's distant low-alpha speckles without tracing,
reshaping, sharpening, flattening, or making opaque backplates. Retained edge alpha
and blue RGB are unchanged by recoloring. Native source alpha is mostly 253/254,
not 255; that slight translucency is deliberately preserved. Source blue texture
and raster edge irregularities remain; white lettering is uniform RGB, not noisy.
The 1024 px package icon is upsampled from an approximately 829 px tall silhouette;
this does not create additional detail. Tiny icons are resampled, not hand-hinted.
The black package artwork can have lower contrast on dark OS surfaces; the light
integration PNGs are available to theme-aware UI consumers.

Validation covers alpha/crop bounds, exact recolor invariants, master checksums,
all PNG sizes, ICO frames, ICNS entries, and deterministic regeneration. Composite
previews were inspected on light and dark backgrounds. Native Explorer/Finder and
AppImage launch appearance still require platform smoke testing; macOS packaging
remains disabled in CI as before.

Official references consulted:

- [Pillow image formats (ICO and ICNS)](https://pillow.readthedocs.io/en/stable/handbook/image-file-formats.html)
- [Pillow image operations and resizing](https://pillow.readthedocs.io/en/stable/reference/Image.html)
- [winresource WindowsResource](https://docs.rs/winresource/latest/winresource/struct.WindowsResource.html)
