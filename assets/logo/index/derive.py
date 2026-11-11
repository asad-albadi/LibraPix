"""Rebuild INDEX integration assets from the immutable approved raster masters."""

from pathlib import Path

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parent
SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)


def normalized(source):
    image = Image.open(source).convert("RGBA")
    # Remove only near-invisible source speckles; retain all other edge coverage.
    image.putdata([(r, g, b, a) if a > 3 else (0, 0, 0, 0)
                   for r, g, b, a in image.getdata()])
    return ImageOps.expand(image.crop(image.getchannel("A").getbbox()), border=2)


def light_variant(dark):
    light = dark.copy()
    # Preserve blue RGB and every alpha value, including antialiased coverage.
    light.putdata([(r, g, b, a) if not a or (b > r + 16 and b > g + 16)
                   else (255, 255, 255, a) for r, g, b, a in dark.getdata()])
    return light


def main():
    for kind in ("icon", "wordmark"):
        dark = normalized(ROOT / f"librapix-{kind}-dark-source.png")
        dark.save(ROOT / f"librapix-{kind}-dark.png")
        light_variant(dark).save(ROOT / f"librapix-{kind}-light.png")

    icon = Image.open(ROOT / "librapix-icon-dark.png").convert("RGBA")
    # Center the unscaled crop on a square with 5% clear space on each side.
    side = round(max(icon.size) / 0.9)
    square = Image.new("RGBA", (side, side))
    square.paste(icon, ((side - icon.width) // 2, (side - icon.height) // 2))
    images = {size: square.resize((size, size), Image.Resampling.LANCZOS)
              for size in SIZES}
    for size, image in images.items():
        image.save(ROOT / f"icon-{size}.png")
    images[256].save(ROOT / "icon.ico", sizes=[(s, s) for s in SIZES if s <= 256],
                     append_images=[images[s] for s in SIZES if s < 256],
                     bitmap_format="bmp")
    images[1024].save(ROOT / "icon.icns",
                      append_images=[images[s] for s in (32, 64, 128, 256, 512, 1024)])


if __name__ == "__main__":
    main()
