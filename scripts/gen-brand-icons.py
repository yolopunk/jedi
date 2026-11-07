#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Jedi 工具箱 —— 品牌图标生成器（白底 App 图标瓦片版）。

产品定稿（2026-11-05）：应用图标、任务栏/窗口图标、托盘图标统一使用
「白底圆角方块 + 黑色 Koala」的 App 图标（public/branding/jedi-app-icon.png），
也就是安装页面品牌区的那款 logo。

产出（全部相对仓库根）：
  src-tauri/icons/tray-icon.png     64x64   托盘
  src-tauri/icons/32x32.png         32x32   }
  src-tauri/icons/128x128.png       128x128 } 应用/窗口图标
  src-tauri/icons/128x128@2x.png    256x256 }
  src-tauri/icons/icon.png          512x512 }
  src-tauri/icons/icon.ico          多尺寸 16/24/32/48/64/128/256     窗口/任务栏/安装器
  src-tauri/icons/16x16.png 24x24.png 48x48.png 64x64.png                常规尺寸（保持同源）
  src-tauri/icons/StoreLogo.png  Square{44,71,89,107,142,150,284,310}x*Logo.png   Windows 打包图标
  src-tauri/icons/icon.icns         macOS 应用图标（同款瓦片）
  .tmp/icons/preview-dark.png / preview-light.png           深底/浅底可读性自查

不动：src-tauri/icons/tray-template.png（macOS 菜单栏模板图标，需黑色单色）。

用法（幂等、无网络、除 Pillow 外无第三方依赖）：

    python scripts/gen-brand-icons.py
    python scripts/gen-brand-icons.py --out src-tauri/icons --preview-dir .tmp/icons
    python scripts/gen-brand-icons.py --no-preview
"""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    sys.exit("需要 Pillow：请用带 Pillow 的 Python 运行本脚本（本仓库使用 DSH runtime python）。")

REPO_ROOT = Path(__file__).resolve().parents[1]
SOURCE_APP_ICON = REPO_ROOT / "public" / "branding" / "jedi-app-icon.png"
DEFAULT_OUT_DIR = "src-tauri/icons"
DEFAULT_PREVIEW_DIR = ".tmp/icons"

PNG_TARGETS = {
    # 托盘 / 应用 / 窗口
    "tray-icon.png": 64,
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    # Tauri 图标集常规尺寸（bundle.icon 未引用，保持同源避免误用旧图）
    "16x16.png": 16,
    "24x24.png": 24,
    "48x48.png": 48,
    "64x64.png": 64,
    # Windows 打包图标
    "StoreLogo.png": 50,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
}

ICO_NAME = "icon.ico"
ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)


def require(path: Path, what: str) -> Path:
    if not path.is_file():
        sys.exit(f"缺少{what}：{path}")
    return path


def load_app_icon() -> Image.Image:
    """读取白底 App 图标（1024×1024 RGBA，圆角方块 + 黑色 Koala，四角透明）。"""
    require(SOURCE_APP_ICON, "白底 App 图标 PNG")
    icon = Image.open(SOURCE_APP_ICON).convert("RGBA")
    bbox = icon.getchannel("A").getbbox()
    if bbox is None:
        sys.exit(f"App 图标位图为空：{SOURCE_APP_ICON}")
    return icon.crop(bbox)


def render_icon(tile: Image.Image, size: int) -> Image.Image:
    """把白底瓦片缩放到目标边长（≤48px 用 4x 超采样保住 Koala 轮廓）。"""
    if size <= 48:
        ss = tile.resize((size * 4, size * 4), Image.LANCZOS)
        return ss.resize((size, size), Image.LANCZOS)
    return tile.resize((size, size), Image.LANCZOS)


def save_png_icons(tile: Image.Image, out_dir: Path) -> dict[str, Image.Image]:
    out_dir.mkdir(parents=True, exist_ok=True)
    made: dict[str, Image.Image] = {}
    for name, size in PNG_TARGETS.items():
        icon = render_icon(tile, size)
        icon.save(out_dir / name, format="PNG", optimize=True)
        made[name] = icon
    return made


def save_ico(tile: Image.Image, path: Path) -> dict[int, Image.Image]:
    """写多尺寸 ICO（帧用 32-bit BMP 编码，Shell/NSIS/System.Drawing 兼容性最好）。"""
    frames = [render_icon(tile, size) for size in ICO_SIZES]
    by_size = {f.size[0]: f for f in frames}
    base = by_size[256]
    path.parent.mkdir(parents=True, exist_ok=True)
    base.save(
        path,
        format="ICO",
        sizes=[(s, s) for s in ICO_SIZES],
        append_images=[by_size[s] for s in ICO_SIZES if s != 256],
        bitmap_format="bmp",
    )
    return by_size


def save_icns(tile: Image.Image, path: Path) -> bool:
    """macOS 应用图标（Pillow 支持 ICNS 写出；失败则跳过并提示）。"""
    try:
        sizes = [(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512)]
        frames = [render_icon(tile, s) for s in sizes]
        base = frames[-1]
        path.parent.mkdir(parents=True, exist_ok=True)
        base.save(path, format="ICNS", append_images=frames[:-1])
        return True
    except Exception as e:  # noqa: BLE001
        print(f"  !! icns 生成失败（保持原文件不动）：{e}")
        return False


# ------------------------------------------------------------------ 校验
def verify_png(path: Path, size: int) -> None:
    with Image.open(path) as im:
        assert im.size == (size, size), f"{path.name} 尺寸 {im.size} != {(size, size)}"
        assert im.mode == "RGBA", f"{path.name} mode {im.mode} != RGBA"
        alpha = im.getchannel("A")
        lo, hi = alpha.getextrema()
        # 圆角缩放后角落允许有极淡残留（≤8），主体透明度不变
        assert lo <= 8, f"{path.name} alpha 最小值 {lo} > 8（疑似底板）"
        assert hi == 255, f"{path.name} alpha 最大值 {hi} != 255"
        px = alpha.load()
        for corner in ((0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)):
            # 圆角缩放后角落允许有极淡的抗锯齿残留（≤32），但不得是不透明底板
            assert px[corner] <= 32, f"{path.name} 角落 {corner} 不透明（疑似底板）"
        rgb = im.convert("RGB").load()
        ink = 0
        white = 0
        for y in range(0, size, max(1, size // 16)):
            for x in range(0, size, max(1, size // 16)):
                if px[x, y] == 255:
                    r, g, b = rgb[x, y]
                    if max(r, g, b) <= 60:
                        ink += 1          # 黑色 Koala 墨迹
                    elif min(r, g, b) >= 240:
                        white += 1        # 白色方块底
        assert ink > 0, f"{path.name} 没有黑色 Koala 墨迹"
        assert white > 0, f"{path.name} 没有白色方块底"
        ink_ratio = _ink_ratio(alpha)
        print(
            f"  ok  {path.as_posix():40s} {size}x{size} RGBA  白底瓦片+黑标  "
            f"墨迹占宽 {ink_ratio:.0%}"
        )


def _ink_ratio(alpha: Image.Image) -> float:
    bbox = alpha.getbbox()
    if not bbox:
        return 0.0
    return (bbox[2] - bbox[0]) / alpha.width


def verify_ico(path: Path) -> list[tuple[int, int]]:
    """解析 ICO 目录（不依赖 Pillow 的读取路径），返回 (w, h) 列表。"""
    raw = path.read_bytes()
    reserved, ico_type, count = struct.unpack_from("<HHH", raw, 0)
    assert reserved == 0, f"{path.name} reserved={reserved} != 0"
    assert ico_type == 1, f"{path.name} type={ico_type} != 1（不是 icon）"
    entries = []
    for i in range(count):
        w, h, colors, res, planes, bits, nbytes, offset = struct.unpack_from(
            "<BBBBHHII", raw, 6 + i * 16
        )
        entries.append(((w or 256), (h or 256), bits, nbytes, offset))
    sizes = [(w, h) for w, h, _, _, _ in entries]
    assert len(sizes) == len(set(sizes)), f"{path.name} 有重复尺寸帧：{sizes}"
    for w, h, bits, nbytes, offset in entries:
        assert bits in (32, 24, 8, 4, 1), f"{path.name} {w}x{h} 位深异常：{bits}"
        assert offset + nbytes <= len(raw), f"{path.name} {w}x{h} 帧越界"
    missing = [s for s in ICO_SIZES if (s, s) not in sizes]
    assert not missing, f"{path.name} 缺少尺寸：{missing}"
    print(
        f"  ok  {path.as_posix():40s} {len(sizes)} 帧 {sorted(sizes)}  {len(raw):>7d} bytes"
    )
    return sizes


# ------------------------------------------------------------------ 预览
PREVIEW_DARK = (0x20, 0x20, 0x20)
PREVIEW_LIGHT = (0xF2, 0xF2, 0xF2)


def build_preview(icons: dict[int, Image.Image], bg: tuple[int, int, int], path: Path) -> None:
    """把 16/24/32/36/64 图标并排铺在给定底色上：上行原始尺寸、下行 4x 放大自查。"""
    sizes = [16, 24, 32, 36, 64]
    gap = 12
    pad = 12
    row1_h = max(sizes)
    row2_h = max(sizes) * 4
    width = pad * 2 + sum(max(s, s * 4) for s in sizes) + gap * (len(sizes) - 1)
    height = pad * 2 + row1_h + gap + row2_h

    sheet = Image.new("RGB", (width, height), bg)
    x = pad
    for s in sizes:
        icon = icons[s]
        sheet.paste(icon, (x, pad + (row1_h - s) // 2), icon)
        zoom = icon.resize((s * 4, s * 4), Image.NEAREST)
        sheet.paste(zoom, (x, pad + row1_h + gap + (row2_h - s * 4) // 2), zoom)
        x += max(s, s * 4) + gap

    path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(path, format="PNG")
    print(f"  ok  {path.as_posix():44s} {width}x{height} bg=#{bg[0]:02X}{bg[1]:02X}{bg[2]:02X}")


def build_preview_all(png_images: dict[str, Image.Image], bg: tuple[int, int, int], path: Path) -> None:
    """整套 PNG 图标并排：≤64px 按原始尺寸一行，>64px 统一缩到 128 一行。"""
    cap = 128
    gap, pad, row_gap = 14, 14, 16
    small = sorted(
        ((name, im) for name, im in png_images.items() if im.width <= 64),
        key=lambda kv: (kv[1].width, kv[0]),
    )
    large = sorted(
        ((name, im) for name, im in png_images.items() if im.width > 64),
        key=lambda kv: (kv[1].width, kv[0]),
    )

    def displayed(items):
        return [(name, im if im.width <= cap else im.resize((cap, cap), Image.LANCZOS)) for name, im in items]

    rows = [displayed(small), displayed(large)]
    row_heights = [max((im.height for _, im in row), default=0) for row in rows]
    row_widths = [
        sum(im.width for _, im in row) + gap * max(0, len(row) - 1) for row in rows
    ]
    width = pad * 2 + max(row_widths)
    height = pad * 2 + sum(row_heights) + row_gap * (len(rows) - 1)

    sheet = Image.new("RGB", (width, height), bg)
    y = pad
    for row, row_h in zip(rows, row_heights):
        x = pad
        for _, im in row:
            sheet.paste(im, (x, y + (row_h - im.height) // 2), im)
            x += im.width + gap
        y += row_h + row_gap

    path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(path, format="PNG")
    print(f"  ok  {path.as_posix():44s} {width}x{height} bg=#{bg[0]:02X}{bg[1]:02X}{bg[2]:02X}")


# ------------------------------------------------------------------ main
def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="生成 Jedi 白底 App 图标瓦片（应用 / 任务栏 / 托盘 / 安装器）",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--out",
        default=DEFAULT_OUT_DIR,
        help=f"图标输出目录（相对仓库根，默认 {DEFAULT_OUT_DIR}）",
    )
    parser.add_argument(
        "--preview-dir",
        default=DEFAULT_PREVIEW_DIR,
        help=f"预览 PNG 目录（相对仓库根，默认 {DEFAULT_PREVIEW_DIR}）",
    )
    parser.add_argument("--no-preview", action="store_true", help="只写图标，不写自查预览")
    args = parser.parse_args(argv)

    out_dir = Path(args.out)
    if not out_dir.is_absolute():
        out_dir = REPO_ROOT / out_dir
    pdir = Path(args.preview_dir)
    if not pdir.is_absolute():
        pdir = REPO_ROOT / pdir

    print("Jedi 工具箱 · 白底 App 图标瓦片")
    print(f"  repo root : {REPO_ROOT}")
    print(f"  source    : {SOURCE_APP_ICON.relative_to(REPO_ROOT) if SOURCE_APP_ICON.is_relative_to(REPO_ROOT) else SOURCE_APP_ICON}")
    print(f"  out dir   : {out_dir}")

    tile = load_app_icon()
    print(f"  app icon  : {tile.width}x{tile.height}")

    save_png_icons(tile, out_dir)
    save_ico(tile, out_dir / ICO_NAME)

    # macOS 应用图标（同款瓦片；失败保持原文件）
    if save_icns(tile, out_dir / "icon.icns"):
        print(f"  ok  {(out_dir / 'icon.icns').as_posix()}  ICNS（同款瓦片）")

    print("\n校验 PNG：")
    for name, size in PNG_TARGETS.items():
        verify_png(out_dir / name, size)

    print("\n校验 ICO：")
    verify_ico(out_dir / ICO_NAME)

    if not args.no_preview:
        print("\n预览：")
        zoom_icons = {s: render_icon(tile, s) for s in (16, 24, 32, 36, 64)}
        build_preview(zoom_icons, PREVIEW_DARK, pdir / "preview-dark.png")
        build_preview(zoom_icons, PREVIEW_LIGHT, pdir / "preview-light.png")
        all_icons = {name: render_icon(tile, size) for name, size in PNG_TARGETS.items()}
        build_preview_all(all_icons, PREVIEW_DARK, pdir / "preview-all-dark.png")
        build_preview_all(all_icons, PREVIEW_LIGHT, pdir / "preview-all-light.png")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
