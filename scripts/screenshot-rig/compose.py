# -*- coding: utf-8 -*-
"""Compose raw rig screenshots into README-ready PNGs.

Rounded window + hairline border + soft shadow on a dark brand backdrop
(Jedi dark theme: bg #0d0d11, accent #539bf5 blue, accent-2 #a371f7 purple).

Run: python scripts/screenshot-rig/compose.py
"""
from __future__ import annotations

import os

from PIL import Image, ImageDraw, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "output")
DEST = os.path.join(HERE, "..", "..", "public", "images", "screenshots")
os.makedirs(DEST, exist_ok=True)

MAP = {
    "ai-chat.png": "ai-chat.png",
    "skills.png": "skills.png",
    "hosts.png": "hosts-manager.png",
    "wallpapers.png": "knowledge-wallpapers.png",
    "podcast.png": "podcast.png",
    "screenshot-tool.png": "screenshot-studio.png",
}

MARGIN = 110          # backdrop padding around the window (2x scale)
RADIUS = 26           # window corner radius
BORDER = 2
BORDER_ALPHA = 34
SHADOW_BLUR = 64
SHADOW_ALPHA = 150
SHADOW_OFFSET_Y = 28
FINAL_WIDTH = 2560    # downscale target (crisp at GitHub display sizes)


def backdrop(w, h):
    """Dark gradient + two soft brand-color glows."""
    img = Image.new("RGB", (w, h))
    top, bottom = (10, 12, 20), (15, 18, 30)
    px = img.load()
    for y in range(h):
        t = y / max(1, h - 1)
        row = tuple(int(top[i] + (bottom[i] - top[i]) * t) for i in range(3))
        for x in range(w):
            px[x, y] = row
    for center, radius, color, strength in (
        ((int(w * 0.16), int(h * 0.18)), int(h * 0.75), (83, 155, 245), 34),
        ((int(w * 0.86), int(h * 0.85)), int(h * 0.8), (163, 113, 247), 30),
    ):
        glow = Image.new("L", (w, h), 0)
        gd = ImageDraw.Draw(glow)
        steps = 24
        for i in range(steps, 0, -1):
            r = radius * i / steps
            a = int(strength * (1 - i / steps) ** 2)
            gd.ellipse([center[0] - r, center[1] - r, center[0] + r, center[1] + r], fill=a)
        overlay = Image.new("RGB", (w, h), color)
        img.paste(overlay, (0, 0), glow)
    return img


def compose(src_path, dest_path):
    shot = Image.open(src_path).convert("RGBA")
    sw, sh = shot.size
    cw, ch = sw + MARGIN * 2, sh + MARGIN * 2 + SHADOW_OFFSET_Y

    canvas = backdrop(cw, ch).convert("RGBA")

    # window shadow
    shadow = Image.new("RGBA", (sw + SHADOW_BLUR * 2, sh + SHADOW_BLUR * 2), (0, 0, 0, 0))
    smask = Image.new("L", (sw, sh), 0)
    ImageDraw.Draw(smask).rounded_rectangle([0, 0, sw - 1, sh - 1], radius=RADIUS, fill=SHADOW_ALPHA)
    black = Image.new("RGBA", (sw, sh), (0, 0, 0, 255))
    shadow.paste(black, (SHADOW_BLUR, SHADOW_BLUR), smask)
    shadow = shadow.filter(ImageFilter.GaussianBlur(SHADOW_BLUR // 2))
    canvas.alpha_composite(shadow, (MARGIN - SHADOW_BLUR, MARGIN - SHADOW_BLUR + SHADOW_OFFSET_Y))

    # rounded window + hairline border
    win = Image.new("RGBA", (sw, sh), (0, 0, 0, 0))
    mask = Image.new("L", (sw, sh), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, sw - 1, sh - 1], radius=RADIUS, fill=255)
    win.paste(shot, (0, 0), mask)
    ImageDraw.Draw(win).rounded_rectangle(
        [BORDER // 2, BORDER // 2, sw - 1 - BORDER // 2, sh - 1 - BORDER // 2],
        radius=RADIUS - BORDER // 2,
        outline=(255, 255, 255, BORDER_ALPHA),
        width=BORDER,
    )
    canvas.alpha_composite(win, (MARGIN, MARGIN))

    # downscale for repo size, keep retina-crisp at GitHub widths
    out_w = FINAL_WIDTH
    out_h = int(cw and ch * out_w / cw)
    canvas = canvas.convert("RGB").resize((out_w, out_h), Image.LANCZOS)
    canvas.save(dest_path, "PNG", optimize=True)
    print("composed", os.path.basename(dest_path), f"{out_w}x{out_h}")


def main():
    for src, dest in MAP.items():
        compose(os.path.join(SRC, src), os.path.join(DEST, dest))


if __name__ == "__main__":
    main()
