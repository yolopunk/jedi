#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Jedi 工具箱 —— Windows NSIS 安装器品牌位图生成器。

依据 docs/installer-ux-redesign-2026-11-04.md 第 3 节（视觉规格 / 位图资源）：

  src-tauri/windows/nsis/sidebar.bmp   164 x 314  24-bit，无 alpha
      · 深色渐变 #16161C -> #0A0A0E，顶部居中 #539BF5 径向光晕（<=16% 不透明度）
      · 白色 Koala 标志（public/branding/jedi-mark-white.png）宽约 64px，居中，距顶 ~42px
      · 字标「Jedi 工具箱」品牌展示字体（public/fonts/CangErJinKai05-W04.ttf）约 20px，白色
      · 副标题「开发者实用工具集」11px #A1A1AA
      · 40 x 2 px 品牌蓝 #3574F0 分隔线（80% 不透明度）
      · 底部弱文字 9px #6C707E（行距 14）：本地优先 · 无需账号 / Hosts · 壁纸 · 播客 · AI
      · 不出现版本号（位图是静态资源，版本由模板文案注入）

  src-tauri/windows/nsis/header.bmp    150 x 57   24-bit，无 alpha
      · 深色品牌铭牌（产品追加决定，2026-11-04 已确认，覆盖 docs 3.2 原本的「纯白底」方案）：
        底色 #111113，左侧 3px 品牌蓝 #3574F0 竖条
      · 白色 Koala 标志（约 30px 高，public/branding/jedi-mark-white.png）
      · 「Jedi 工具箱」13px 粗体 #F4F4F5，「安装程序」9px #A1A1AA
      · 安装界面统一用白色 logo，不再使用白底黑标版本

  src-tauri/windows/nsis/logo.bmp    192 x 192   24-bit，无 alpha
      · 单窗口（无向导）深色安装器界面的主 logo（产品决定，2026-11-04）
      · 底色严格 #0E0E12（与安装器窗口底色完全一致，像素级相同才能无缝贴合）
      · 白色 Koala 标志居中，墨迹约占画布宽 78%
      · 不画渐变 / 阴影 / 描边 —— 位图贴在纯色底上，任何渐变或描边都会露出方框
      · 标志边缘的半透明抗锯齿像素直接与 #0E0E12 合成（无 alpha，也不抖动：
        抖动会让底色不再是精确的 (14,14,18)）
      · 192px 是为了 100%~200% DPI 下（150% 时显示约 135~145px）都有足够采样

> 说明：sidebar.bmp / header.bmp 属于「MUI 向导式界面」方案，安装器改成单窗口深色
> 界面后已不再使用，但生成逻辑保留（产品可能回退），仍然参与一条命令重建。

设计要点（为什么这样画）：
  · NSIS 的位图是纯位图，没有 alpha 通道。所有半透明元素（光晕、分隔线、
    标志的透明底）都在脚本里预先合成到不透明底色上，最终只写出 24-bit BMP。
  · 安装器在 100%~200% DPI 下都会被系统拉伸（150% 时 164x314 -> 246x471）。
    因此：不画 1px 细线，最小字号 9px，大色块 + 大字号优先。
  · 所有内容图层按 SUPERSAMPLE 倍（默认 3x）渲染再 LANCZOS 降采样到目标尺寸。
    原因：9px 中文直接让 FreeType 在最终尺寸上栅格化，笔画会被压成 1px 硬边、
    小字号密集字形糊成一团；3x 渲染后降采样得到的是「按面积平均」的灰阶覆盖，
    笔画比例不变（不会被描边糊死），但边缘平滑、抗拉伸能力更好。
  · 深色渐变用 8x8 Bayer 抖动（±1 色阶）打散色带：314 行铺 12 级灰阶，
    不抖动在 150% 拉伸后会出现 ~26px 高的条带。
  · Pillow 不能解析 SVG（本机无 cairosvg / resvg / inkscape / sharp），所以 Koala
    标志使用仓库内同源的 1024px 官方 PNG 光栅图（jedi-mark-white.png /
    jedi-mark-black.png，内容框 816x657），LANCZOS 缩放到目标尺寸后按 alpha 合成。

用法（一条命令重建三张 BMP；幂等、无网络、除 Pillow 外无第三方依赖）：

    python scripts/gen-installer-art.py
    python scripts/gen-installer-art.py --out src-tauri/windows/nsis
    python scripts/gen-installer-art.py --preview-dir .tmp/art      # 附带预览 PNG

脚本只写 --out 下的 logo.bmp / sidebar.bmp / header.bmp；只有显式给 --preview-dir 时
才额外写预览 PNG（logo-on-bg.png 用来人工确认「贴在 #0E0E12 上无可见接缝」）。
"""

from __future__ import annotations

import argparse
import math
import os
import struct
import sys
from pathlib import Path

try:
    from PIL import Image, ImageChops, ImageDraw, ImageFont
except ImportError:  # pragma: no cover
    sys.exit("需要 Pillow：请用带 Pillow 的 Python 运行本脚本（本仓库使用 DSH runtime python）。")

REPO_ROOT = Path(__file__).resolve().parents[1]

# ---------------------------------------------------------------- 资源路径
MARK_WHITE_PNG = REPO_ROOT / "public" / "branding" / "jedi-mark-white.png"
MARK_BLACK_PNG = REPO_ROOT / "public" / "branding" / "jedi-mark-black.png"
BRAND_FONT = REPO_ROOT / "public" / "fonts" / "CangErJinKai05-W04.ttf"
# 中文 UI 字体：微软雅黑（Windows 自带）。缺失就直接报错，别悄悄退回 Pillow 位图字体。
UI_FONT_REGULAR = Path(os.environ.get("JEDI_UI_FONT", r"C:\Windows\Fonts\msyh.ttc"))
UI_FONT_BOLD = Path(os.environ.get("JEDI_UI_FONT_BOLD", r"C:\Windows\Fonts\msyhbd.ttc"))

DEFAULT_OUT_DIR = "src-tauri/windows/nsis"
SUPERSAMPLE = 3

# ---------------------------------------------------------------- 设计令牌（docs 3.1）
C_BG_TOP = "#16161C"
C_BG_BOTTOM = "#0A0A0E"
C_BRAND_BLUE = "#3574F0"
C_GLOW = "#539BF5"
C_TEXT_DARK_MAIN = "#FFFFFF"     # 3.2：字标「白色」（3.1 的 #F4F4F5 是同一档主文字色）
C_TEXT_DARK_SUB = "#A1A1AA"      # 3.1：深色次文字
C_TEXT_DARK_WEAK = "#6C707E"     # 3.1：深色弱文字
C_PAGE_MAIN = "#27282E"          # 3.1：页面主文字
C_PAGE_SECONDARY = "#6C707E"     # 3.1：页面次文字
C_PAGE_BG = "#FFFFFF"            # 3.1：页面底

GLOW_MAX_ALPHA = 0.16            # 3.1：光晕 <= 16% 不透明度
DIVIDER_ALPHA = 0.80             # 3.2：分隔线 80% 不透明度

SIDEBAR_SIZE = (164, 314)
HEADER_SIZE = (150, 57)
LOGO_SIZE = (192, 192)

# 单窗口安装器主 logo：纯色底 + 单一白色图形，无渐变/阴影/描边
LOGO = {
    "bg": "#0E0E12",        # 必须与安装器窗口底色像素级一致
    "ink_ratio": 0.78,      # 墨迹宽 = 画布宽 * 0.78 ≈ 150px
    "supersample": 4,       # 4x 超采样后按面积平均降采样（BOX，无振铃 → 无光晕环）
}

# 侧栏版式（单位：最终 px；依据 3.2「距顶 ~42px」「距顶 ~132px」）
SIDEBAR = {
    "logo_width": 64,
    "logo_top": 42,
    "wordmark_size": 20,
    "wordmark_top": 130,        # 字标墨迹顶端
    "wordmark_stroke": 1,       # 超采样单位；1/3px 实心描边，给品牌展示字体补一点笔画厚度
    "subtitle_size": 11,
    "subtitle_gap": 11,         # 字标墨迹底 -> 副标题墨迹顶
    "divider_w": 40,
    "divider_h": 2,
    "divider_gap": 15,          # 副标题墨迹底 -> 分隔线顶
    "bottom_size": 9,
    "bottom_line_height": 14,
    "bottom_bottom_margin": 20,  # 最后一行墨迹底 -> 画布底
    "bottom_stroke": 1,         # 超采样单位；同上，9px 弱文字补 1/3px 避免拉伸后消失
    "bottom_lines": ("本地优先 · 无需账号", "Hosts · 壁纸 · 播客 · AI"),
    # 光晕：椭圆径向衰减，中心略高于画布顶端，让顶部整条呈蓝色洗染
    "glow_center": (82.0, -8.0),
    "glow_radius": (100.0, 140.0),
    "glow_falloff": 1.6,
}

# 页头版式：深色品牌铭牌
HEADER = {
    "bg": "#111113",
    "bar_w": 3,                 # 左侧品牌蓝竖条
    "bar_gap": 10,              # 竖条右 -> 标志左
    "logo_height": 30,
    "text_gap": 7,              # 标志右 -> 文字左
    "title_size": 13,
    "title_color": "#F4F4F5",
    "sub_size": 9,
    "sub_color": "#A1A1AA",
    "sub_tracking": 1.5,        # 9px 中文补字距，拉伸后字形不会互相粘连
    "line_gap": 6,              # 标题墨迹底 -> 副标题墨迹顶
    "title_stroke": 1,
    "sub_stroke": 1,
}

# 确定性 8x8 Bayer 抖动矩阵（0..63）
_BAYER8 = (
    (0, 32, 8, 40, 2, 34, 10, 42),
    (48, 16, 56, 24, 50, 18, 58, 26),
    (12, 44, 4, 36, 14, 46, 6, 38),
    (60, 28, 52, 20, 62, 30, 54, 22),
    (3, 35, 11, 43, 1, 33, 9, 41),
    (51, 19, 59, 27, 49, 17, 57, 25),
    (15, 47, 7, 39, 13, 45, 5, 37),
    (63, 31, 55, 23, 61, 29, 53, 21),
)


# ------------------------------------------------------------------ 小工具
def hex_rgb(value: str) -> tuple[int, int, int]:
    v = value.lstrip("#")
    return (int(v[0:2], 16), int(v[2:4], 16), int(v[4:6], 16))


def require(path: Path, what: str) -> Path:
    if not path.is_file():
        sys.exit(f"缺少{what}：{path}")
    return path


def load_font(path: Path, size: int) -> ImageFont.FreeTypeFont:
    require(path, "字体文件")
    return ImageFont.truetype(str(path), max(1, int(round(size))))


def ink_box(
    draw: ImageDraw.ImageDraw, text: str, font: ImageFont.FreeTypeFont, stroke_width: int = 0
) -> tuple[int, int, int, int]:
    """文字墨迹框（相对于 (0,0) 且 anchor='la' 的偏移量）。"""
    return draw.textbbox((0, 0), text, font=font, stroke_width=stroke_width)


def draw_text_ink(
    draw: ImageDraw.ImageDraw,
    text: str,
    font: ImageFont.FreeTypeFont,
    fill: tuple[int, int, int, int],
    *,
    ink_top: int | None = None,
    left: float | None = None,
    center_x: float | None = None,
    stroke_width: int = 0,
    tracking: float = 0.0,
) -> tuple[int, int, int, int]:
    """按「墨迹框」定位绘制文字，返回实际墨迹框 (l, t, r, b)。

    用墨迹框而不是行框定位，才能在 ascent/descent 差异极大的字体（仓耳今楷
    vs 微软雅黑）之间让视觉间距可控。

    tracking > 0 时逐字绘制并追加字距：对 9px 这类极小字号的中文，
    字距比字号更能提升辨识度（字腔不会被相邻笔画粘住）。
    """
    l, t, r, b = ink_box(draw, text, font, stroke_width)
    y = 0 if ink_top is None else int(round(ink_top)) - t

    if tracking == 0.0:
        x = round(center_x - (l + r) / 2) if center_x is not None else round(left or 0)
        draw.text((x, y), text, font=font, fill=fill, stroke_width=stroke_width, stroke_fill=fill)
        return (l + x, t + y, r + x, b + y)

    advances = [draw.textlength(ch, font=font) for ch in text]
    total = sum(advances) + tracking * max(0, len(text) - 1)
    origin = (center_x - total / 2) if center_x is not None else (left or 0)
    x = origin
    for ch, adv in zip(text, advances):
        draw.text(
            (round(x), y), ch, font=font, fill=fill, stroke_width=stroke_width, stroke_fill=fill
        )
        x += adv + tracking
    return (round(origin), t + y, round(origin + total), b + y)


def mark_rgba(png_path: Path, *, width: int | None = None, height: int | None = None) -> Image.Image:
    """读取透明底标志 PNG，裁到墨迹框并按比例缩放到目标宽或高。"""
    require(png_path, "标志位图")
    mark = Image.open(png_path).convert("RGBA")
    bbox = mark.getchannel("A").getbbox()
    if bbox is None:
        sys.exit(f"标志位图为空：{png_path}")
    mark = mark.crop(bbox)
    if width is not None:
        out_w = max(1, int(round(width)))
        out_h = max(1, round(mark.height * out_w / mark.width))
    elif height is not None:
        out_h = max(1, int(round(height)))
        out_w = max(1, round(mark.width * out_h / mark.height))
    else:
        return mark
    return mark.resize((out_w, out_h), Image.LANCZOS)


def gradient_float(
    w: int, h: int, top_hex: str, bottom_hex: str, glow: dict | None = None
) -> list[tuple[float, float, float]]:
    """竖直渐变（+ 可选椭圆径向光晕），返回浮点 RGB 缓冲（统一在最后抖动量化）。"""
    top = hex_rgb(top_hex)
    bottom = hex_rgb(bottom_hex)
    glow_rgb = hex_rgb(C_GLOW)
    buf: list[tuple[float, float, float]] = [top] * (w * h)
    cx, cy = glow["glow_center"] if glow else (0.0, 0.0)
    rx, ry = glow["glow_radius"] if glow else (1.0, 1.0)
    falloff = glow["glow_falloff"] if glow else 1.0
    denom = max(1, h - 1)

    for y in range(h):
        t = y / denom
        base = (
            top[0] + (bottom[0] - top[0]) * t,
            top[1] + (bottom[1] - top[1]) * t,
            top[2] + (bottom[2] - top[2]) * t,
        )
        row = y * w
        for x in range(w):
            px = base
            if glow is not None:
                d = math.hypot((x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry)
                if d < 1.0:
                    a = GLOW_MAX_ALPHA * (1.0 - d) ** falloff
                    px = (
                        base[0] + (glow_rgb[0] - base[0]) * a,
                        base[1] + (glow_rgb[1] - base[1]) * a,
                        base[2] + (glow_rgb[2] - base[2]) * a,
                    )
            buf[row + x] = px
    return buf


def premultiply(layer: Image.Image) -> Image.Image:
    """非预乘 RGBA -> 预乘 RGBA。

    Pillow 的 resize 对非预乘 RGBA 是分别插值各通道的，透明区域里 RGB=0 的黑像素
    会渗进文字/标志的边缘，产生黑边。先预乘再 LANCZOS 降采样才不会渗色。
    """
    r, g, b, a = layer.split()
    return Image.merge(
        "RGBA",
        (ImageChops.multiply(r, a), ImageChops.multiply(g, a), ImageChops.multiply(b, a), a),
    )


def flatten_and_dither(
    bg: list[tuple[float, float, float]], layer: Image.Image, *, dither: bool = True
) -> Image.Image:
    """浮点背景 + 预乘 RGBA 内容层 -> 8-bit RGB。

    dither=True 时叠 8x8 Bayer 抖动打散色带（渐变侧栏需要）；
    dither=False 时用四舍五入 —— logo.bmp 要求底色严格等于 #0E0E12，
    任何 ±1 的抖动都会让底色不再是精确值。
    """
    w, h = layer.size
    layer_px = layer.load()
    buf = bytearray(w * h * 3)
    i = 0
    for y in range(h):
        bayer_row = _BAYER8[y & 7]
        for x in range(w):
            pr, pg, pb, a = layer_px[x, y]
            br, bg_, bb = bg[y * w + x]
            if a:
                f = (255 - a) / 255.0
                br = pr + br * f
                bg_ = pg + bg_ * f
                bb = pb + bb * f
            if dither:
                thr = (bayer_row[x & 7] + 0.5) / 64.0
                values = (br + thr, bg_ + thr, bb + thr)
                ints = [int(math.floor(v)) for v in values]
            else:
                ints = [int(round(br)), int(round(bg_)), int(round(bb))]
            for v in ints:
                buf[i] = 0 if v < 0 else (255 if v > 255 else v)
                i += 1
    return Image.frombytes("RGB", (w, h), bytes(buf))


def downsample(layer: Image.Image, size: tuple[int, int]) -> Image.Image:
    """超采样层降采样。

    用 BOX（区域内平均）而不是 LANCZOS：整数倍下采样时 BOX 就是精确的像素面积
    覆盖度平均，不会出现 LANCZOS 的振铃/过冲——在深底 + 白色文字这种高反差组合
    上，振铃会渲染成文字边缘的灰/蓝色光晕。
    """
    return layer.resize(size, Image.BOX)


def save_bmp(image: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(path, format="BMP")


def save_preview(image: Image.Image, path: Path, scale: float = 1.5) -> None:
    """模拟 Windows 在 150% DPI 下对位图的拉伸，生成人工自查用的预览 PNG。"""
    w = max(1, round(image.width * scale))
    h = max(1, round(image.height * scale))
    path.parent.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").resize((w, h), Image.LANCZOS).save(path, format="PNG")


# MUI 默认对话框尺寸（96 dpi 下的经典 503x360 欢迎页），用来做「真实上下文」预览
MUI_DIALOG_SIZE = (503, 360)
MUI_FACE = "#F0F0F0"


def save_context_preview(bitmap: Image.Image, path: Path, scale: float = 1.5) -> None:
    """把位图放进 MUI 对话框上下文再放大，检查边缘/对比度是否成立。

    MUI 的 header 图与 welcome 侧栏图都贴在对话框 (0,0)，其余区域是对话框面色，
    所以能直接看出深色铭牌/侧栏与浅色对话框之间的硬边是否可接受。
    """
    dialog = Image.new("RGB", MUI_DIALOG_SIZE, hex_rgb(MUI_FACE))
    dialog.paste(bitmap.convert("RGB"), (0, 0))
    save_preview(dialog, path, scale)


# ------------------------------------------------------------------ 侧栏位图
def render_sidebar(ss: int = SUPERSAMPLE) -> Image.Image:
    w, h = SIDEBAR_SIZE
    cfg = SIDEBAR
    full = (w * ss, h * ss)

    # 内容图层：RGBA 画布，超采样绘制后统一降采样（不透明底色在外层合成）
    layer = Image.new("RGBA", full, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)

    def px(v: float) -> int:
        return int(round(v * ss))

    # 1) 白色 Koala：宽 64，水平居中，距顶 42
    logo = mark_rgba(MARK_WHITE_PNG, width=px(cfg["logo_width"]))
    logo_left = round(full[0] / 2 - logo.width / 2)
    logo_top = px(cfg["logo_top"])
    layer.alpha_composite(logo, (logo_left, logo_top))
    logo_bottom = cfg["logo_top"] + logo.height / ss

    # 2) 字标「Jedi 工具箱」：品牌展示字体 20px 白色
    wordmark_font = load_font(BRAND_FONT, cfg["wordmark_size"] * ss)
    wm_box = draw_text_ink(
        draw,
        "Jedi 工具箱",
        wordmark_font,
        hex_rgb(C_TEXT_DARK_MAIN) + (255,),
        ink_top=px(max(cfg["wordmark_top"], logo_bottom + 24)),
        center_x=full[0] / 2,
        stroke_width=cfg["wordmark_stroke"],
    )

    # 3) 副标题「开发者实用工具集」11px #A1A1AA
    subtitle_font = load_font(UI_FONT_REGULAR, cfg["subtitle_size"] * ss)
    sub_box = draw_text_ink(
        draw,
        "开发者实用工具集",
        subtitle_font,
        hex_rgb(C_TEXT_DARK_SUB) + (255,),
        ink_top=wm_box[3] + px(cfg["subtitle_gap"]),
        center_x=full[0] / 2,
        stroke_width=1,
    )

    # 4) 40 x 2 品牌蓝分隔线（80% 不透明度；直接写 alpha，合成时按覆盖度落地）
    div_w, div_h = px(cfg["divider_w"]), max(1, px(cfg["divider_h"]))
    div_left = round((full[0] - div_w) / 2)
    div_top = sub_box[3] + px(cfg["divider_gap"])
    draw.rectangle(
        (div_left, div_top, div_left + div_w - 1, div_top + div_h - 1),
        fill=hex_rgb(C_BRAND_BLUE) + (round(255 * DIVIDER_ALPHA),),
    )

    # 5) 底部弱文字（9px，行距 14，水平居中，整块贴底）
    weak_font = load_font(UI_FONT_REGULAR, cfg["bottom_size"] * ss)
    lines = cfg["bottom_lines"]
    line_h = px(cfg["bottom_line_height"])
    ink_h = ink_box(draw, lines[-1], weak_font, cfg["bottom_stroke"])[3]
    block_bottom = full[1] - px(cfg["bottom_bottom_margin"])
    block_top = block_bottom - ink_h - line_h * (len(lines) - 1)
    for i, line in enumerate(lines):
        draw_text_ink(
            draw,
            line,
            weak_font,
            hex_rgb(C_TEXT_DARK_WEAK) + (255,),
            ink_top=block_top + i * line_h,
            center_x=full[0] / 2,
            stroke_width=cfg["bottom_stroke"],
        )

    bg = gradient_float(w, h, C_BG_TOP, C_BG_BOTTOM, cfg)
    return flatten_and_dither(bg, downsample(premultiply(layer), (w, h)))


# ------------------------------------------------------------------ 页头位图（深色品牌铭牌）
def render_header(ss: int = SUPERSAMPLE) -> Image.Image:
    w, h = HEADER_SIZE
    cfg = HEADER
    full = (w * ss, h * ss)

    layer = Image.new("RGBA", full, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)

    def px(v: float) -> int:
        return int(round(v * ss))

    # 1) 左侧 3px 品牌蓝竖条（贯穿全高；3px 在 150% 下约 4.5px，不会被拉伸成细线）
    bar_w = max(1, px(cfg["bar_w"]))
    draw.rectangle((0, 0, bar_w - 1, full[1] - 1), fill=hex_rgb(C_BRAND_BLUE) + (255,))

    # 2) 白色 Koala：高 30，垂直居中
    logo = mark_rgba(MARK_WHITE_PNG, height=px(cfg["logo_height"]))
    logo_left = bar_w + px(cfg["bar_gap"])
    logo_top = round(full[1] / 2 - logo.height / 2)
    layer.alpha_composite(logo, (logo_left, logo_top))
    text_left = logo_left + logo.width + px(cfg["text_gap"])

    title_font = load_font(UI_FONT_BOLD, cfg["title_size"] * ss)
    sub_font = load_font(UI_FONT_REGULAR, cfg["sub_size"] * ss)

    # 3) 两行文字作为一个整体在 57px 高里垂直居中
    title_h = ink_box(draw, "Jedi 工具箱", title_font, cfg["title_stroke"])[3]
    sub_h = ink_box(draw, "安装程序", sub_font, cfg["sub_stroke"])[3]
    block_h = title_h + px(cfg["line_gap"]) + sub_h
    title_top = round((full[1] - block_h) / 2)

    title_box = draw_text_ink(
        draw,
        "Jedi 工具箱",
        title_font,
        hex_rgb(cfg["title_color"]) + (255,),
        ink_top=title_top,
        left=text_left,
        stroke_width=cfg["title_stroke"],
    )
    sub_box = draw_text_ink(
        draw,
        "安装程序",
        sub_font,
        hex_rgb(cfg["sub_color"]) + (255,),
        ink_top=title_box[3] + px(cfg["line_gap"]),
        left=text_left,
        stroke_width=cfg["sub_stroke"],
        tracking=cfg["sub_tracking"] * ss,
    )

    right_edge = max(title_box[2], sub_box[2]) / ss
    if right_edge > w - 3:
        sys.exit(f"header.bmp 文字超出画布：右边界 {right_edge:.1f} > {w - 3}，请调小字号或间距")

    bg = gradient_float(w, h, cfg["bg"], cfg["bg"])
    return flatten_and_dither(bg, downsample(premultiply(layer), (w, h)))


# ------------------------------------------------------------------ 单窗口 logo 位图
def render_logo(ss: int | None = None) -> Image.Image:
    """192x192 单窗口安装器 logo：严格纯色 #0E0E12 底 + 居中白色 Koala。

    做法：内容层在 ss 倍画布上只画白色 alpha（1x 画布直接画会产生 1px 硬边），
    然后用 BOX（面积平均）降采样 ——
      · BOX 没有 LANCZOS 的振铃，标志外圈不会出现比底色更亮的「光晕环」；
      · 半透明抗锯齿像素在这里就等价于「白色 × 覆盖度」，再与 #0E0E12 精确合成；
      · 整张图不做抖动，保证每个背景像素都严格等于 (14,14,18)。
    """
    w, h = LOGO_SIZE
    cfg = LOGO
    ss = cfg["supersample"] if ss is None else ss
    full = (w * ss, h * ss)

    ink_w = int(round(full[0] * cfg["ink_ratio"]))
    mark = mark_rgba(MARK_WHITE_PNG, width=ink_w)
    ink_h = mark.height
    ink_left = round((full[0] - mark.width) / 2)
    ink_top = round((full[1] - ink_h) / 2)

    layer = Image.new("RGBA", full, (0, 0, 0, 0))
    layer.alpha_composite(mark, (ink_left, ink_top))
    layer = downsample_plain(premultiply(layer), (w, h))

    bg_rgb = hex_rgb(cfg["bg"])
    bg = [(float(bg_rgb[0]), float(bg_rgb[1]), float(bg_rgb[2]))] * (w * h)
    return flatten_and_dither(bg, layer, dither=False)


def downsample_plain(layer: Image.Image, size: tuple[int, int]) -> Image.Image:
    """整数倍面积平均降采样（不走 LANCZOS：logo 贴在纯色底上不能有振铃光晕）。"""
    return layer.resize(size, Image.BOX)


# ------------------------------------------------------------------ 校验
def verify_bmp(path: Path, expect_size: tuple[int, int]) -> None:
    """独立校验：Pillow 读回尺寸/mode，并直接解析 BMP 头确认 24-bit 无调色板/alpha。"""
    with Image.open(path) as im:
        assert im.size == expect_size, f"{path.name} 尺寸 {im.size} != {expect_size}"
        assert im.mode == "RGB", f"{path.name} mode {im.mode} != RGB"
        im.load()

    raw = path.read_bytes()
    assert raw[0:2] == b"BM", f"{path.name} 不是 BMP（magic={raw[0:2]!r}）"
    # BITMAPFILEHEADER: bfType(2) bfSize(4) bfReserved1(2) bfReserved2(2) bfOffBits(4)
    # BITMAPINFOHEADER: biSize(4) biWidth(4) biHeight(4) biPlanes(2) biBitCount(2) ...
    offset, dib_size, width, height, planes, bit_count = struct.unpack_from("<IIiiHH", raw, 10)
    assert dib_size >= 40, f"{path.name} 用了非 BITMAPINFOHEADER 头（biSize={dib_size}）"
    assert (width, abs(height)) == expect_size, f"{path.name} BMP 头尺寸 {(width, height)} 不符"
    assert planes == 1, f"{path.name} biPlanes {planes} != 1"
    assert bit_count == 24, f"{path.name} 位深 {bit_count} != 24"
    assert offset == 14 + dib_size, (
        f"{path.name} bfOffBits={offset} != 14+{dib_size}，像素数据前还有调色板"
    )
    pixel_bytes = (width * 3 + 3) // 4 * 4 * abs(height)
    assert len(raw) == offset + pixel_bytes, (
        f"{path.name} 文件大小不符（疑似带 alpha）：offset={offset} "
        f"pixel={pixel_bytes} file={len(raw)}"
    )
    print(
        f"  ok  {path.as_posix():52s} {width}x{abs(height)}  24-bit  {len(raw):>7d} bytes"
    )


def verify_logo_bmp(path: Path) -> None:
    """logo.bmp 专项校验：底色必须严格精确、无描边/阴影、墨迹比例与居中合格。"""
    verify_bmp(path, LOGO_SIZE)  # 尺寸 / 24-bit / 无调色板 / 文件大小精确
    bg = hex_rgb(LOGO["bg"])
    w, h = LOGO_SIZE
    border = 6

    with Image.open(path) as im:
        px = im.convert("RGB").load()

    # 1) 四角严格等于底色
    for corner in ((0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)):
        assert px[corner] == bg, f"{path.name} 角落 {corner} = {px[corner]} != {bg}"

    # 2) 外圈 border 像素必须全是底色（贴在同色窗口上无缝，不能有任何方框/渐变边）
    for y in range(h):
        for x in range(w):
            if x < border or y < border or x >= w - border or y >= h - border:
                assert px[x, y] == bg, f"{path.name} 外圈 ({x},{y}) = {px[x, y]} != {bg}"

    # 3) 统计非底色像素：求墨迹包围盒，检查比例；同时确认没有「比底色更暗」的像素
    xs: list[int] = []
    ys: list[int] = []
    aa_pixels = 0
    white_pixels = 0
    for y in range(h):
        for x in range(w):
            r, g, b = px[x, y]
            assert r >= bg[0] and g >= bg[1] and b >= bg[2], (
                f"{path.name} ({x},{y}) = {(r, g, b)} 比底色暗：出现描边/阴影"
            )
            if (r, g, b) != bg:
                xs.append(x)
                ys.append(y)
                if (r, g, b) == (255, 255, 255):
                    white_pixels += 1
                else:
                    aa_pixels += 1

    assert xs, f"{path.name} 找不到白色标志"
    left, right, top, bottom = min(xs), max(xs), min(ys), max(ys)
    ink_w = right - left + 1
    ink_h = bottom - top + 1
    ratio = ink_w / w
    assert 0.75 <= ratio <= 0.81, f"{path.name} 墨迹占宽 {ratio:.1%} 不在 75%~81%（期望 ~78%）"
    center_dx = abs((left + right) / 2 - (w - 1) / 2)
    center_dy = abs((top + bottom) / 2 - (h - 1) / 2)
    assert center_dx <= 1.0 and center_dy <= 1.0, (
        f"{path.name} 标志没有居中：偏移 ({center_dx:.1f}, {center_dy:.1f})"
    )
    assert white_pixels > 0.15 * ink_w * ink_h, (
        f"{path.name} 纯白像素过少（{white_pixels}），标志可能被压灰"
    )
    bg_pixels = w * h - len(xs)
    print(
        f"  ok  {path.as_posix():52s} 192x192  24-bit  底色严格 #{bg[0]:02X}{bg[1]:02X}{bg[2]:02X}"
        f"  墨迹 {ink_w}x{ink_h} ({ratio:.1%} 宽)  抗锯齿像素 {aa_pixels}  纯底色像素 {bg_pixels}"
    )


# ------------------------------------------------------------------ 预览
def save_logo_on_bg(logo: Image.Image, path: Path, canvas: tuple[int, int] = (560, 480)) -> None:
    """把 logo.bmp 居中贴到更大的 #0E0E12 画布上：接缝若可见就是失败。"""
    bg = hex_rgb(LOGO["bg"])
    sheet = Image.new("RGB", canvas, bg)
    sheet.paste(logo, ((canvas[0] - logo.width) // 2, (canvas[1] - logo.height) // 2))
    path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(path, format="PNG")
    print(f"  ok  {path.as_posix():52s} {canvas[0]}x{canvas[1]} bg=#{bg[0]:02X}{bg[1]:02X}{bg[2]:02X}")


# ------------------------------------------------------------------ main
def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="生成 Jedi 工具箱 NSIS 安装器品牌位图（sidebar.bmp / header.bmp）",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--out",
        default=DEFAULT_OUT_DIR,
        help=f"BMP 输出目录（相对仓库根，默认 {DEFAULT_OUT_DIR}）",
    )
    parser.add_argument(
        "--preview-dir",
        default=None,
        help="额外输出 150%% DPI 拉伸预览 PNG 的目录（相对仓库根，例如 .tmp/art）；默认不输出",
    )
    parser.add_argument("--preview-scale", type=float, default=1.5, help="预览放大倍数，默认 1.5")
    parser.add_argument(
        "--preview-context",
        action="store_true",
        help="额外输出贴进 MUI 对话框（浅色面色）的上下文预览，用来检查深色位图的边缘/对比度",
    )
    parser.add_argument(
        "--supersample",
        type=int,
        default=SUPERSAMPLE,
        help=f"内容层超采样倍率，默认 {SUPERSAMPLE}",
    )
    args = parser.parse_args(argv)
    if args.supersample < 1:
        sys.exit("--supersample 必须 >= 1")

    out_dir = Path(args.out)
    if not out_dir.is_absolute():
        out_dir = REPO_ROOT / out_dir

    print("Jedi 工具箱 · NSIS 安装器品牌位图")
    print(f"  repo root   : {REPO_ROOT}")
    print(f"  out dir     : {out_dir}")
    print(f"  supersample : {args.supersample}x")

    # 单窗口安装器主 logo（当前主力资源）
    logo = render_logo()
    # MUI 向导式界面用的两张位图（已不再被模板引用，逻辑保留以便回退）
    sidebar = render_sidebar(args.supersample)
    header = render_header(args.supersample)

    logo_path = out_dir / "logo.bmp"
    sidebar_path = out_dir / "sidebar.bmp"
    header_path = out_dir / "header.bmp"
    save_bmp(logo, logo_path)
    save_bmp(sidebar, sidebar_path)
    save_bmp(header, header_path)

    print("\n校验：")
    verify_logo_bmp(logo_path)
    verify_bmp(sidebar_path, SIDEBAR_SIZE)
    verify_bmp(header_path, HEADER_SIZE)

    if args.preview_dir:
        pdir = Path(args.preview_dir)
        if not pdir.is_absolute():
            pdir = REPO_ROOT / pdir
        save_logo_on_bg(logo, pdir / "logo-on-bg.png")
        save_preview(logo, pdir / "logo-150pct.png", args.preview_scale)
        save_preview(sidebar, pdir / "sidebar-150pct.png", args.preview_scale)
        save_preview(header, pdir / "header-150pct.png", args.preview_scale)
        print(f"\n预览（logo 接缝用整块同色画布验证；其余为 {int(round(args.preview_scale * 100))}% 拉伸）：")
        print(f"  ok  {(pdir / 'logo-on-bg.png').as_posix()}")
        print(f"  ok  {(pdir / 'logo-150pct.png').as_posix()}")
        print(f"  ok  {(pdir / 'sidebar-150pct.png').as_posix()}")
        print(f"  ok  {(pdir / 'header-150pct.png').as_posix()}")
        if args.preview_context:
            save_context_preview(sidebar, pdir / "sidebar-context-150pct.png", args.preview_scale)
            save_context_preview(header, pdir / "header-context-150pct.png", args.preview_scale)
            print(f"  ok  {(pdir / 'sidebar-context-150pct.png').as_posix()}")
            print(f"  ok  {(pdir / 'header-context-150pct.png').as_posix()}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
