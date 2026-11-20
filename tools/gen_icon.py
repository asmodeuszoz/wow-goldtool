#!/usr/bin/env python3
"""Reproduce icons/icon.png.

The icon is generated, not downloaded: every pixel comes from this file, so the
repo carries no unexplained binary blob and the same script re-generates the
committed PNG byte-for-byte (fixed seed, no randomness). Tauri's codegen reads
build.bundle.icon at COMPILE time (ADR 0001), which is why the file must exist
before the first GUI build - `cargo build` fails without it, and this script is
the fix.

Design: rounded navy plate, gold coin with a ring, and a rising price line
inside the coin - the two things the app is about (gold, prices).
"""

import os
from PIL import Image, ImageDraw

SIZE = 1024          # supersample, then downscale to 256 for smooth edges
TARGET = 256
NAVY = (15, 20, 28)
PANEL = (22, 30, 41)
GOLD = (240, 180, 39)
GOLD_DEEP = (196, 142, 26)
EDGE = (42, 57, 80)


def plate(size: int) -> Image.Image:
    """Rounded background plate with a hairline border."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    radius = size // 6
    draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=NAVY)
    draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, outline=EDGE, width=max(2, size // 128))
    return img


def coin(img: Image.Image) -> None:
    """The gold coin: two rings plus a notch, so it reads as a coin, not a dot."""
    draw = ImageDraw.Draw(img)
    size = img.width
    cx = cy = size // 2
    outer = int(size * 0.36)
    inner = int(size * 0.29)
    draw.ellipse([cx - outer, cy - outer, cx + outer, cy + outer], fill=GOLD)
    draw.ellipse([cx - inner, cy - inner, cx + inner, cy + inner], fill=GOLD_DEEP)
    draw.ellipse([cx - inner, cy - inner, cx + inner, cy + inner], outline=GOLD, width=max(2, size // 96))


def price_line(img: Image.Image) -> None:
    """A rising, jagged price line carved out of the coin in the plate's navy."""
    draw = ImageDraw.Draw(img)
    size = img.width
    cx = cy = size // 2
    span = int(size * 0.22)
    points = [
        (cx - span, cy + int(span * 0.75)),
        (cx - int(span * 0.45), cy + int(span * 0.2)),
        (cx, cy + int(span * 0.45)),
        (cx + int(span * 0.5), cy - int(span * 0.5)),
        (cx + span, cy - int(span * 0.85)),
    ]
    width = max(3, size // 42)
    draw.line(points, fill=NAVY, width=width, joint="curve")
    for (x, y) in points:
        r = width
        draw.ellipse([x - r, y - r, x + r, y + r], fill=NAVY)


def main() -> None:
    os.makedirs("icons", exist_ok=True)
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    plate_layer = plate(SIZE)
    img.alpha_composite(plate_layer)
    coin(img)
    price_line(img)
    img = img.resize((TARGET, TARGET), Image.LANCZOS)
    img.save("icons/icon.png")
    print("icons/icon.png", img.size, "LANCZOS from", SIZE)


if __name__ == "__main__":
    main()
