"""Draws the Bahia icon: a bay at midday on a Windows 11 style tile.

A rounded square in a turquoise-to-deep-blue gradient, a warm sun, two white
waves and a strip of sand along the bottom. Drawn at 4x and downsampled, so
edges are smooth and the area outside the tile is fully transparent.

Writes assets/brand/bahia-1024.png and assets/brand/bahia.ico, copies the .ico
to the Settings app, and writes the small title-bar icon Settings shows.

Usage: python scripts/make_icon.py
"""

import math
import shutil
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
BRAND = ROOT / "assets" / "brand"
SETTINGS_ICON = ROOT / "apps" / "settings" / "src-tauri" / "icons" / "icon.ico"
TITLE_ICON = ROOT / "apps" / "settings" / "src" / "assets" / "bahia-64.png"

SS = 4
SIZE = 1024
N = SIZE * SS
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)

# The tile: Windows 11 icons sit on softer, less rounded squares than macOS.
MARGIN = 80
RADIUS = 190

SKY_TOP = (64, 214, 214)  # turquoise
SEA_DEEP = (16, 84, 190)  # deep blue
SUN = (255, 214, 102)
SAND = (247, 214, 156)
SAND_SHADE = (232, 184, 112)
FOAM = (255, 255, 255)


def s(value: float) -> int:
    """Design units (0..1024) to supersampled pixels."""
    return round(value * SS)


def gradient() -> Image.Image:
    """Diagonal gradient from the top-left to the bottom-right."""
    small = Image.new("RGB", (256, 256))
    pixels = small.load()
    for y in range(256):
        for x in range(256):
            t = min(1.0, max(0.0, (x * 0.35 + y * 0.65) / 255))
            t = t * t * (3 - 2 * t)
            pixels[x, y] = tuple(round(a + (b - a) * t) for a, b in zip(SKY_TOP, SEA_DEEP))
    return small.resize((N, N), Image.BICUBIC)


def wave_points(y: float, amplitude: float, left: float, right: float, phase: float) -> list[tuple[int, int]]:
    points = []
    steps = 120
    for i in range(steps + 1):
        x = left + (right - left) * i / steps
        angle = phase + (x - left) / (right - left) * 2.2 * math.pi
        points.append((s(x), s(y + amplitude * math.sin(angle))))
    return points


def draw() -> Image.Image:
    tile = Image.new("L", (N, N), 0)
    ImageDraw.Draw(tile).rounded_rectangle(
        (s(MARGIN), s(MARGIN), s(SIZE - MARGIN), s(SIZE - MARGIN)), radius=s(RADIUS), fill=255
    )

    art = gradient().convert("RGBA")
    layer = Image.new("RGBA", (N, N), (0, 0, 0, 0))
    paint = ImageDraw.Draw(layer)

    # Sun, with a soft halo.
    paint.ellipse((s(555), s(195), s(805), s(445)), fill=(255, 244, 200, 70))
    paint.ellipse((s(595), s(235), s(765), s(405)), fill=SUN + (255,))

    # Waves, with round ends: the far one thinner and fainter.
    for y, amplitude, left, right, phase, width, alpha in (
        (560, 24, 230, 794, 0.4, 34, 150),
        (665, 32, 190, 834, 1.3, 52, 255),
    ):
        points = wave_points(y, amplitude, left, right, phase)
        paint.line(points, fill=FOAM + (alpha,), width=s(width), joint="curve")
        radius = s(width) // 2
        for x, y_end in (points[0], points[-1]):
            paint.ellipse((x - radius, y_end - radius, x + radius, y_end + radius), fill=FOAM + (alpha,))

    # Sand along the bottom: a wide arc, shaded at its edge.
    paint.ellipse((s(40), s(752), s(984), s(1240)), fill=SAND_SHADE + (255,))
    paint.ellipse((s(40), s(778), s(984), s(1266)), fill=SAND + (255,))

    art = Image.alpha_composite(art, layer)

    # A faint highlight along the top edge, as Fluent tiles have.
    shine = Image.new("L", (N, N), 0)
    ImageDraw.Draw(shine).rounded_rectangle(
        (s(MARGIN), s(MARGIN), s(SIZE - MARGIN), s(SIZE - MARGIN)), radius=s(RADIUS), outline=60, width=s(6)
    )
    white = Image.new("RGBA", (N, N), (255, 255, 255, 0))
    white.putalpha(ImageChops.multiply(shine, Image.linear_gradient("L").rotate(180).resize((N, N))))
    art = Image.alpha_composite(art, white)

    art.putalpha(ImageChops.multiply(art.getchannel("A"), tile))
    return art.resize((SIZE, SIZE), Image.LANCZOS)


def main() -> None:
    icon = draw()
    BRAND.mkdir(parents=True, exist_ok=True)
    icon.save(BRAND / "bahia-1024.png")
    icon.save(BRAND / "bahia.ico", sizes=[(size, size) for size in ICO_SIZES])
    shutil.copyfile(BRAND / "bahia.ico", SETTINGS_ICON)
    TITLE_ICON.parent.mkdir(parents=True, exist_ok=True)
    icon.resize((64, 64), Image.LANCZOS).save(TITLE_ICON)
    print("wrote", BRAND / "bahia-1024.png", BRAND / "bahia.ico", SETTINGS_ICON, TITLE_ICON)


if __name__ == "__main__":
    main()
