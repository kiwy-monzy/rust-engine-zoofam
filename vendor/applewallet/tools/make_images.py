#!/usr/bin/env python3
"""
Generates the per-sample Wallet images into assets/samples/<id>/.

The crate ships only icon/logo, which every pass needs. What makes the five
Wallet *styles* look like themselves is the style-specific artwork - strip,
thumbnail, background, footer - and Apple is strict about which of those each
style may carry. This writes the right set, at @1x/@2x/@3x, tinted to match the
colour each sample already declares in samples.rs.

Pure stdlib: PNGs are encoded by hand (zlib + struct) so this needs no Pillow.

    python crates/applewallet/tools/make_images.py
"""

from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
ASSETS = HERE.parent / "assets" / "samples"

# ---------------------------------------------------------------- encoding ---


def encode_png(width: int, height: int, pixel) -> bytes:
    """Encodes an RGB PNG. `pixel(x, y, w, h)` returns an (r, g, b) triple."""
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter type 0 (None) for every scanline
        for x in range(width):
            r, g, b = pixel(x, y, width, height)
            raw += bytes((r & 255, g & 255, b & 255))

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


# ------------------------------------------------------------------ colour ---


def shade(rgb, factor: float):
    """Lightens (factor > 1) or darkens (factor < 1) a colour."""
    return tuple(max(0, min(255, int(c * factor))) for c in rgb)


def mix(a, b, t: float):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def artwork(base):
    """A vertical gradient with a soft diagonal sheen.

    The sheen keeps large flat areas (the strip is 375x123) from looking dead
    without introducing anything that competes with the pass text on top.
    """
    top = shade(base, 1.30)
    bottom = shade(base, 0.62)

    def pixel(x, y, w, h):
        c = mix(top, bottom, y / max(1, h - 1))
        # Diagonal band, brightest around the 40% line across the image.
        d = abs((x / w) * 0.75 + (y / h) * 0.25 - 0.40)
        sheen = max(0.0, 1.0 - d * 3.2) ** 2
        return mix(c, shade(c, 1.55), sheen * 0.5)

    return pixel


def tile(base):
    """Denser treatment for small square art (thumbnail/icon)."""
    top = shade(base, 1.22)
    bottom = shade(base, 0.58)

    def pixel(x, y, w, h):
        cx, cy = (x - w / 2) / w, (y - h / 2) / h
        c = mix(top, bottom, y / max(1, h - 1))
        # Radial vignette so the tile reads as a rounded object.
        r = min(1.0, math.hypot(cx, cy) * 1.9)
        return mix(c, shade(c, 0.72), r * r * 0.55)

    return pixel


def bar(base):
    """Footer strip: a thin horizontal rule, brightest in the middle."""
    def pixel(x, y, w, h):
        t = 1.0 - abs(x / w - 0.5) * 2.0
        return mix(shade(base, 0.55), shade(base, 1.35), t)

    return pixel


# ------------------------------------------------------------------- specs ---

# Point sizes; @2x and @3x are multiplied from these.
SIZES = {
    "strip": (375, 123),
    "thumbnail": (90, 90),
    "background": (180, 220),
    "footer": (286, 15),
    "logo": (160, 50),
    "icon": (29, 29),
}

RENDERERS = {
    "strip": artwork,
    "background": artwork,
    "thumbnail": tile,
    "icon": tile,
    "logo": artwork,
    "footer": bar,
}

# Which artwork each style is allowed to carry. Apple rejects, or silently
# ignores, images that do not belong to the style - a boarding pass has no
# strip, and an event ticket may use a strip *or* background+thumbnail, but
# not both.
STYLE_IMAGES = {
    "boardingPass": ["icon", "logo", "footer"],
    "coupon": ["icon", "logo", "strip"],
    "storeCard": ["icon", "logo", "strip"],
    "eventTicket": ["icon", "logo", "background", "thumbnail"],
    "generic": ["icon", "logo", "thumbnail"],
}

# id -> (style, base colour). Colours mirror sample_catalog() in samples.rs.
SAMPLES = {
    "boarding-train":   ("boardingPass", (0, 71, 142)),
    "boarding-air":     ("boardingPass", (20, 40, 100)),
    "boarding-bus":     ("boardingPass", (0, 120, 60)),
    "boarding-boat":    ("boardingPass", (0, 90, 130)),
    "boarding-generic": ("boardingPass", (80, 80, 80)),
    "event":            ("eventTicket",  (90, 20, 120)),
    "coupon":           ("coupon",       (180, 40, 40)),
    "store-card":       ("storeCard",    (30, 100, 50)),
    "generic":          ("generic",      (50, 50, 60)),
}


def main() -> None:
    ASSETS.mkdir(parents=True, exist_ok=True)
    total = 0

    for sample_id, (style, base) in SAMPLES.items():
        out = ASSETS / sample_id
        out.mkdir(parents=True, exist_ok=True)

        for name in STYLE_IMAGES[style]:
            w, h = SIZES[name]
            render = RENDERERS[name](base)
            for scale, suffix in ((1, ""), (2, "@2x"), (3, "@3x")):
                data = encode_png(w * scale, h * scale, render)
                (out / f"{name}{suffix}.png").write_bytes(data)
                total += 1

        print(f"  {sample_id:<18} {style:<13} {', '.join(STYLE_IMAGES[style])}")

    print(f"\nwrote {total} images under {ASSETS}")


if __name__ == "__main__":
    main()
