# -*- coding: utf-8 -*-
"""Generate demo images for the Jedi screenshot rig (wallpapers, covers,
styled screenshot gallery items). Output goes to ./assets next to this file.

Run: python scripts/screenshot-rig/gen_assets.py
"""
from __future__ import annotations

import math
import os
import sys

from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "assets")
os.makedirs(OUT, exist_ok=True)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))

FONT_DIRS = [
    r"C:\Windows\Fonts",
    os.path.join(REPO, "public", "fonts"),
]

KAI_CANDIDATES = ["CangErJinKai05-W04.ttf", "STKAITI.TTF", "simkai.ttf"]
MSYH_CANDIDATES = ["msyh.ttc", "msyhbd.ttc", "simhei.ttf"]
MSYH_BOLD_CANDIDATES = ["msyhbd.ttc", "msyh.ttc", "simhei.ttf"]
MONO_CANDIDATES = ["consolab.ttf", "consola.ttf", "CascadiaCode.ttf", "sarasa-mono-sc-regular.ttf"]


def find_font(candidates):
    for name in candidates:
        for d in FONT_DIRS:
            p = os.path.join(d, name)
            if os.path.exists(p):
                return p
    return None


def font(candidates, size):
    path = find_font(candidates)
    if not path:
        raise SystemExit("no CJK font found; candidates: %s" % candidates)
    return ImageFont.truetype(path, size)


KAI = lambda s: font(KAI_CANDIDATES, s)
SANS = lambda s: font(MSYH_CANDIDATES, s)
SANS_B = lambda s: font(MSYH_BOLD_CANDIDATES, s)
MONO = lambda s: font(MONO_CANDIDATES, s)

ACCENT_BLUE = (83, 155, 245)
ACCENT_PURPLE = (163, 113, 247)
ACCENT_GREEN = (63, 185, 80)
ACCENT_AMBER = (210, 153, 34)
ACCENT_PINK = (247, 120, 186)
ACCENT_CYAN = (88, 213, 217)


# ---------------------------------------------------------------- helpers
def vgrad(size, top, bottom):
    """Vertical gradient image."""
    w, h = size
    img = Image.new("RGB", (1, h))
    for y in range(h):
        t = y / max(1, h - 1)
        img.putpixel((0, y), tuple(int(top[i] + (bottom[i] - top[i]) * t) for i in range(3)))
    return img.resize((w, h))


def radial_glow(img, center, radius, color, strength=110):
    """Additive soft radial glow."""
    w, h = img.size
    glow = Image.new("L", (w, h), 0)
    gd = ImageDraw.Draw(glow)
    steps = 28
    for i in range(steps, 0, -1):
        r = radius * i / steps
        a = int(strength * (1 - i / steps) ** 2)
        gd.ellipse([center[0] - r, center[1] - r, center[0] + r, center[1] + r], fill=a)
    overlay = Image.new("RGB", (w, h), color)
    img.paste(overlay, (0, 0), glow)
    return img


def dot_grid(img, step=56, alpha=10, origin=(0, 0)):
    overlay = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(overlay)
    for x in range(origin[0], img.size[0], step):
        for y in range(origin[1], img.size[1], step):
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(255, 255, 255, alpha))
    img = img.convert("RGBA")
    img.alpha_composite(overlay)
    return img


def rrect(d, box, radius, fill=None, outline=None, width=2):
    d.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def ctext(d, pos, text, fnt, fill, anchor="la"):
    d.text(pos, text, font=fnt, fill=fill, anchor=anchor)


def card_header(img, d, title, subtitle, accent, tag="JEDI KNOWLEDGE · 桌面知识卡"):
    w, h = img.size
    pad = 72
    ctext(d, (pad, 58), tag, SANS(22), (255, 255, 255, 118))
    ctext(d, (pad, 96), title, KAI(84), (245, 246, 248, 255))
    ctext(d, (pad + 4, 208), subtitle, SANS(26), (255, 255, 255, 150))
    # accent underline
    d.rounded_rectangle([pad, 186, pad + 132, 192], radius=3, fill=accent + (230,))


def arrow(d, start, end, color, width=3, head=9):
    d.line([start, end], fill=color, width=width)
    ang = math.atan2(end[1] - start[1], end[0] - start[0])
    for s in (math.radians(152), math.radians(-152)):
        p = (end[0] + head * math.cos(ang + s), end[1] + head * math.sin(ang + s))
        d.line([end, p], fill=color, width=width)


W = 1280
H = 800


def new_card(top, bottom, accent):
    img = vgrad((W, H), top, bottom).convert("RGBA")
    img = dot_grid(img, step=60, alpha=9)
    d = ImageDraw.Draw(img, "RGBA")
    img = radial_glow(img, (W - 160, 140), 330, accent, 60)
    d = ImageDraw.Draw(img, "RGBA")
    return img, d


def save(img, name):
    img.convert("RGB").save(os.path.join(OUT, name), "PNG", optimize=True)
    print("wrote", name)


# ---------------------------------------------------------------- wall-1 系统设计
def wall1():
    img, d = new_card((9, 14, 34), (16, 26, 48), ACCENT_BLUE)
    card_header(img, d, "System Design 入门图谱", "负载均衡 · 缓存 · 消息队列 · 存储分层", ACCENT_BLUE)
    y0 = 300
    # client row
    rrect(d, (120, y0, 400, y0 + 60), 12, fill=(255, 255, 255, 18), outline=(255, 255, 255, 70), width=2)
    ctext(d, (260, y0 + 30), "Client 客户端", SANS(22), (240, 242, 246, 255), anchor="mm")
    # LB
    rrect(d, (500, y0, 780, y0 + 60), 12, fill=ACCENT_BLUE + (56,), outline=ACCENT_BLUE + (200,), width=2)
    ctext(d, (640, y0 + 30), "Load Balancer", SANS_B(22), (226, 238, 255, 255), anchor="mm")
    arrow(d, (400, y0 + 30), (496, y0 + 30), (255, 255, 255, 120))
    # services
    sy = y0 + 120
    names = ["Web 服务", "API 服务", "Job 服务"]
    for i, n in enumerate(names):
        x = 500 + i * 170
        rrect(d, (x, sy, x + 150, sy + 56), 10, fill=(255, 255, 255, 14), outline=(255, 255, 255, 60), width=2)
        ctext(d, (x + 75, sy + 28), n, SANS(20), (235, 238, 244, 255), anchor="mm")
        arrow(d, (640, y0 + 62), (x + 75, sy - 2), (255, 255, 255, 80), width=2)
    # storage layer
    ty = sy + 130
    boxes = [("Cache 缓存", ACCENT_PURPLE), ("Queue 队列", ACCENT_AMBER), ("Database", ACCENT_GREEN)]
    for i, (n, c) in enumerate(boxes):
        x = 330 + i * 220
        rrect(d, (x, ty, x + 190, ty + 64), 12, fill=c + (46,), outline=c + (190,), width=2)
        ctext(d, (x + 95, ty + 32), n, SANS_B(20), (245, 246, 250, 255), anchor="mm")
    for i in range(3):
        x = 500 + i * 170
        arrow(d, (x + 75, sy + 58), (330 + i * 220 + 95, ty - 2), (255, 255, 255, 70), width=2)
    save(img, "wall-1.png")


# ---------------------------------------------------------------- wall-2 SOLID
def wall2():
    img, d = new_card((13, 10, 30), (22, 16, 44), ACCENT_PURPLE)
    card_header(img, d, "SOLID 原则速记", "面向对象设计的五块基石", ACCENT_PURPLE)
    letters = [
        ("S", "单一职责", "一个类只做一件事", ACCENT_BLUE),
        ("O", "开闭原则", "扩展开放，修改关闭", ACCENT_PURPLE),
        ("L", "里氏替换", "子类能替换父类", ACCENT_GREEN),
        ("I", "接口隔离", "接口小而专", ACCENT_AMBER),
        ("D", "依赖倒置", "依赖抽象而非实现", ACCENT_PINK),
    ]
    x = 120
    for letter, name, desc, c in letters:
        rrect(d, (x, 300, x + 200, 620), 16, fill=c + (40,), outline=c + (170,), width=2)
        ctext(d, (x + 100, 380), letter, KAI(120), c + (255,), anchor="mm")
        ctext(d, (x + 100, 500), name, SANS_B(26), (242, 244, 248, 255), anchor="mm")
        ctext(d, (x + 100, 548), desc, SANS(18), (255, 255, 255, 140), anchor="mm")
        x += 218
    save(img, "wall-2.png")


# ---------------------------------------------------------------- wall-3 TCP
def wall3():
    img, d = new_card((7, 20, 26), (12, 34, 40), ACCENT_CYAN)
    card_header(img, d, "TCP 握手与挥手", "三次握手 · 四次挥手 · 状态时序", ACCENT_CYAN)

    def seq(x0, title, steps, color):
        lx, rx = x0 + 70, x0 + 430
        y = 330
        rrect(d, (x0 + 20, 280, x0 + 480, 316), 8, fill=(255, 255, 255, 14))
        ctext(d, (x0 + 250, 298), title, SANS_B(20), (235, 240, 245, 255), anchor="mm")
        for label, data in ((None, None), (None, None)):
            pass
        for host in ("Client", "Server"):
            ctext(d, (lx if host == "Client" else rx, y), host, SANS_B(18), (220, 228, 238, 255), anchor="mm")
            d.line([(lx if host == "Client" else rx, y + 14), (lx if host == "Client" else rx, y + 300)], fill=(255, 255, 255, 60), width=2)
        yy = y + 46
        for label, direction in steps:
            if direction == ">":
                arrow(d, (lx + 6, yy), (rx - 6, yy), color, width=3)
                ctext(d, ((lx + rx) // 2, yy - 14), label, MONO(18), (226, 240, 248, 255), anchor="mm")
            else:
                arrow(d, (rx - 6, yy), (lx + 6, yy), ACCENT_AMBER, width=3)
                ctext(d, ((lx + rx) // 2, yy - 14), label, MONO(18), (250, 236, 200, 255), anchor="mm")
            yy += 62
        return yy

    seq(80, "三次握手 · Connection Established", [("SYN seq=x", ">"), ("SYN+ACK seq=y ack=x+1", "<"), ("ACK ack=y+1", ">")], ACCENT_CYAN)
    seq(720, "四次挥手 · Connection Teardown", [("FIN seq=u", ">"), ("ACK ack=u+1", "<"), ("FIN seq=w", "<"), ("ACK ack=w+1", ">")], ACCENT_CYAN)
    save(img, "wall-3.png")


# ---------------------------------------------------------------- wall-4 数据结构
def wall4():
    img, d = new_card((10, 12, 20), (18, 22, 36), ACCENT_GREEN)
    card_header(img, d, "常用数据结构全景", "数组 · 链表 · 树 · 堆 · 图", ACCENT_GREEN)
    cx, cy = 640, 470
    # array
    ax = 110
    for i in range(6):
        rrect(d, (ax + i * 56, 330, ax + i * 56 + 50, 386), 6, fill=(255, 255, 255, 14), outline=(255, 255, 255, 70), width=2)
        ctext(d, (ax + i * 56 + 25, 358), str(i + 1), MONO(20), (230, 238, 240, 255), anchor="mm")
    ctext(d, (ax + 165, 414), "Array  O(1) 随机访问", SANS(20), (255, 255, 255, 150), anchor="mm")
    # linked list
    lx = 130
    for i in range(5):
        rrect(d, (lx, 500, lx + 70, 552), 8, fill=ACCENT_GREEN + (36,), outline=ACCENT_GREEN + (170,), width=2)
        if i < 4:
            arrow(d, (lx + 72, 526), (lx + 108, 526), (255, 255, 255, 110), width=2, head=7)
        lx += 112
    ctext(d, (390, 590), "LinkedList  O(1) 插入删除", SANS(20), (255, 255, 255, 150), anchor="mm")
    # tree
    def node(p, off, depth=0):
        x, y = p
        ctext(d, (x, y), "●", MONO(22), ACCENT_BLUE + (230,), anchor="mm")
        return (x, y)
    t = (820, 330)
    for dx, dy in ((-60, 60), (60, 60)):
        d.line([t, (t[0] + dx, t[1] + dy)], fill=(255, 255, 255, 70), width=2)
        for dx2, dy2 in ((-30, 52), (30, 52)):
            d.line([(t[0] + dx, t[1] + dy), (t[0] + dx + dx2, t[1] + dy + dy2)], fill=(255, 255, 255, 50), width=2)
            ctext(d, (t[0] + dx + dx2, t[1] + dy + dy2), "●", MONO(18), (140, 180, 245, 235), anchor="mm")
        ctext(d, (t[0] + dx, t[1] + dy), "●", MONO(20), (150, 190, 250, 245), anchor="mm")
    ctext(d, (820, 500), "Tree / Heap  O(log n)", SANS(20), (255, 255, 255, 150), anchor="mm")
    # graph
    pts = [(1020, 330), (1100, 380), (1010, 440), (1150, 320), (1120, 470)]
    for a in range(len(pts)):
        for b in range(a + 1, len(pts)):
            d.line([pts[a], pts[b]], fill=(255, 255, 255, 42), width=2)
    for p in pts:
        ctext(d, (p[0], p[1]), "●", MONO(22), ACCENT_PURPLE + (235,), anchor="mm")
    ctext(d, (1080, 500), "Graph  BFS / DFS", SANS(20), (255, 255, 255, 150), anchor="mm")
    save(img, "wall-4.png")


# ---------------------------------------------------------------- wall-5 K8s
def wall5():
    img, d = new_card((14, 10, 12), (24, 16, 20), ACCENT_PINK)
    card_header(img, d, "Kubernetes 核心架构", "控制平面 · 工作负载 · 服务发现", ACCENT_PINK)
    # control plane
    rrect(d, (110, 290, 560, 640), 16, fill=(255, 255, 255, 12), outline=(255, 255, 255, 70), width=2)
    ctext(d, (335, 318), "Control Plane", SANS_B(24), (240, 244, 250, 255), anchor="mm")
    comps = ["API Server", "etcd", "Scheduler", "Controller"]
    for i, n in enumerate(comps):
        x = 140 + (i % 2) * 210
        y = 350 + (i // 2) * 120
        rrect(d, (x, y, x + 190, y + 92), 12, fill=ACCENT_PINK + (36,), outline=ACCENT_PINK + (170,), width=2)
        ctext(d, (x + 95, y + 46), n, SANS(22), (248, 240, 246, 255), anchor="mm")
    # worker nodes
    rrect(d, (700, 290, 1170, 640), 16, fill=(255, 255, 255, 12), outline=(255, 255, 255, 70), width=2)
    ctext(d, (935, 318), "Worker Nodes", SANS_B(24), (240, 244, 250, 255), anchor="mm")
    for i in range(3):
        x = 730 + i * 145
        rrect(d, (x, 350, x + 130, 610), 12, fill=(255, 255, 255, 10), outline=(255, 255, 255, 60), width=2)
        ctext(d, (x + 65, 376), f"Node {i + 1}", SANS(18), (235, 238, 244, 220), anchor="mm")
        for p in range(3):
            rrect(d, (x + 18, 410 + p * 62, x + 112, 464 + p * 62), 8, fill=ACCENT_BLUE + (44,), outline=ACCENT_BLUE + (160,), width=2)
            ctext(d, (x + 65, 437 + p * 62), "Pod", MONO(18), (226, 238, 255, 235), anchor="mm")
    arrow(d, (562, 460), (698, 460), (255, 255, 255, 120), width=3)
    save(img, "wall-5.png")


# ---------------------------------------------------------------- wall-6 Git
def wall6():
    img, d = new_card((8, 14, 24), (14, 24, 40), ACCENT_AMBER)
    card_header(img, d, "Git 分支模型图解", "main · feature · release · hotfix", ACCENT_AMBER)
    y_main = 560
    x = 110
    # main line
    d.line([(90, y_main), (1190, y_main)], fill=(255, 255, 255, 90), width=4)
    commits_main = [150, 360, 570, 830, 1090]
    for cx in commits_main:
        ctext(d, (cx, y_main), "●", MONO(24), ACCENT_AMBER + (240,), anchor="mm")
    ctext(d, (70, y_main), "main", MONO(20), (255, 255, 255, 170), anchor="rm")
    # feature branch
    f0, f1 = 360, 570
    d.line([(f0, y_main), (f0, y_main - 150), (f1, y_main - 150), (f1, y_main)], fill=ACCENT_BLUE + (200,), width=3)
    for cx in (f0, (f0 + f1) // 2):
        ctext(d, (cx + 40, y_main - 150), "●", MONO(22), ACCENT_BLUE + (240,), anchor="mm")
    ctext(d, (f0, y_main - 190), "feature/*", MONO(20), (160, 195, 250, 235), anchor="lm")
    # release branch
    r0, r1 = 830, 990
    d.line([(r0, y_main), (r0, y_main - 260), (r1, y_main - 260)], fill=ACCENT_GREEN + (200,), width=3)
    for cx in (r0 + 50, r0 + 110):
        ctext(d, (cx, y_main - 260), "●", MONO(22), ACCENT_GREEN + (240,), anchor="mm")
    ctext(d, (r0, y_main - 300), "release/*", MONO(20), (170, 235, 185, 235), anchor="lm")
    # hotfix
    h0 = 1090
    d.line([(h0, y_main), (h0, y_main + 120), (h0 + 60, y_main + 120)], fill=ACCENT_PINK + (220,), width=3)
    ctext(d, (h0 + 20, y_main + 120), "●", MONO(22), ACCENT_PINK + (240,), anchor="mm")
    ctext(d, (h0, y_main + 160), "hotfix/*", MONO(20), (250, 190, 210, 235), anchor="lm")
    # tag
    ctext(d, (1090, y_main - 36), "v1.0.0", MONO(18), (255, 255, 255, 150), anchor="mm")
    save(img, "wall-6.png")


# ---------------------------------------------------------------- wall-7 B+ 树
def wall7():
    img, d = new_card((9, 13, 28), (15, 21, 42), ACCENT_BLUE)
    card_header(img, d, "SQL 索引与 B+ 树", "为什么范围查询走得快", ACCENT_BLUE)
    root = (640, 330)
    rrect(d, (root[0] - 150, root[1] - 36, root[0] + 150, root[1] + 36), 10, fill=(255, 255, 255, 14), outline=(255, 255, 255, 80), width=2)
    ctext(d, root, "[ 17 | 35 ]", MONO(24), (238, 242, 250, 255), anchor="mm")
    leaves_y = 500
    leaves = [(240, "10 | 13 | 15"), (510, "18 | 23 | 27"), (780, "36 | 40 | 44"), (1050, "51 | 62 | 78")]
    for i, (x, keys) in enumerate(leaves):
        c = ACCENT_BLUE if i % 2 == 0 else ACCENT_PURPLE
        rrect(d, (x - 105, leaves_y - 34, x + 105, leaves_y + 34), 10, fill=c + (40,), outline=c + (180,), width=2)
        ctext(d, (x, leaves_y), keys, MONO(20), (240, 242, 250, 255), anchor="mm")
        d.line([root, (x, leaves_y - 40)], fill=(255, 255, 255, 60), width=2)
        if i < 3:
            nx = leaves[i + 1][0]
            arrow(d, (x + 107, leaves_y), (nx - 107, leaves_y), ACCENT_GREEN + (200,), width=2, head=7)
    ctext(d, (640, 600), "叶子节点成链 —— 范围扫描只在最底层横向滑动", SANS(22), (255, 255, 255, 160), anchor="mm")
    ctext(d, (640, 648), "SELECT * FROM users WHERE id BETWEEN 18 AND 44;", MONO(20), (150, 200, 250, 220), anchor="mm")
    save(img, "wall-7.png")


# ---------------------------------------------------------------- wall-8 正则
def wall8():
    img, d = new_card((12, 12, 18), (20, 20, 30), ACCENT_PURPLE)
    card_header(img, d, "正则表达式速查卡", "匹配 · 分组 · 常用模式", ACCENT_PURPLE)
    rows = [
        ("日期", r"^\d{4}-\d{2}-\d{2}$", "2026-11-14"),
        ("邮箱", r"[\w.+-]+@[\w-]+\.[\w.]+", "dev@jedi.io"),
        ("IPv4", r"^(\d{1,3}\.){3}\d{1,3}$", "10.50.128.32"),
        ("十六进制色", r"^#[0-9a-fA-F]{6}$", "#539bf5"),
        ("分组提取", r"(?P<year>\d{4})-(?P<month>\d{2})", "2026-11"),
        ("惰性匹配", r"<h(\d)>.*?</h\1>", "<h2>标题</h2>"),
    ]
    y = 300
    for name, pattern, sample in rows:
        rrect(d, (120, y, 1160, y + 68), 12, fill=(255, 255, 255, 12), outline=(255, 255, 255, 46), width=2)
        ctext(d, (152, y + 34), name, SANS_B(22), (244, 240, 250, 255), anchor="lm")
        ctext(d, (330, y + 34), pattern, MONO(22), ACCENT_CYAN + (240,), anchor="lm")
        ctext(d, (880, y + 34), sample, MONO(20), (170, 230, 210, 230), anchor="lm")
        y += 84
    save(img, "wall-8.png")


# ---------------------------------------------------------------- podcast covers
def cover(name, ch, title, top, bottom, accent):
    img = vgrad((600, 600), top, bottom).convert("RGBA")
    img = dot_grid(img, step=48, alpha=12)
    d = ImageDraw.Draw(img, "RGBA")
    img = radial_glow(img, (300, 210), 280, accent, 90)
    d = ImageDraw.Draw(img, "RGBA")
    ctext(d, (300, 285), ch, KAI(300), (248, 249, 252, 255), anchor="mm")
    d.rounded_rectangle([210, 470, 390, 480], radius=5, fill=accent + (230,))
    ctext(d, (300, 528), title, SANS_B(30), (255, 255, 255, 215), anchor="mm")
    save(img, name)


# ---------------------------------------------------------------- gallery items
def styled_api_docs():
    # window on a dark desk background
    bg = vgrad((1600, 1000), (14, 18, 32), (24, 30, 50)).convert("RGBA")
    bg = dot_grid(bg, step=64, alpha=8)
    win = Image.new("RGBA", (1360, 860), (0, 0, 0, 0))
    d = ImageDraw.Draw(win, "RGBA")
    rrect(d, (0, 0, 1360, 860), 18, fill=(18, 20, 27, 255), outline=(255, 255, 255, 40), width=2)
    rrect(d, (0, 0, 1360, 56), 18, fill=(30, 33, 43, 255))
    for i, c in enumerate(((255, 95, 87), (255, 189, 47), (40, 201, 78))):
        d.ellipse([26 + i * 34, 20, 46 + i * 34, 40], fill=c)
    ctext(d, (680, 28), "verify_hosts.py — jedi", SANS(20), (200, 206, 218, 255), anchor="mm")
    code = [
        ("import ", (255, 123, 114)), ("socket", (210, 168, 255)), ("", (160, 168, 186)),
        ("", (160, 168, 186)),
        ("DOMAINS = [", (160, 168, 186)), ('"api.dev.io"', (168, 223, 142)), (", ", (160, 168, 186)), ('"grafana.dev.io"', (168, 223, 142)), ("]", (160, 168, 186)),
        ("EXPECT  = ", (160, 168, 186)), ('"10.50.128.32"', (168, 223, 142)),
        ("", (160, 168, 186)),
        ("for ", (255, 123, 114)), ("domain ", (210, 168, 255)), ("in ", (255, 123, 114)), ("DOMAINS:", (160, 168, 186)),
        ("    ip = socket.gethostbyname(domain)", (160, 168, 186)),
        ("    flag = ", (160, 168, 186)), ('"OK" ', (168, 223, 142)), ("if ", (255, 123, 114)), ("ip == EXPECT ", (160, 168, 186)), ("else ", (255, 123, 114)), ('"FAIL"', (168, 223, 142)),
        ("    print(", (160, 168, 186)), ('f"{flag:<4} {domain:<18} -> {ip}"', (233, 196, 138)), (")", (160, 168, 186)),
    ]
    y = 100
    x = 60
    for text, color in code:
        if text == "":
            y += 40
            continue
        ctext(d, (x, y), text, MONO(26), color + (255,))
        x += d.textlength(text, font=MONO(26))
        if x > 1200:
            x = 60
            y += 40
    out = Image.new("RGBA", (1600, 1000), (0, 0, 0, 0))
    out.alpha_composite(bg)
    out.alpha_composite(win, (120, 70))
    save(out, "styled-api-docs.png")


def styled_terminal():
    bg = vgrad((1600, 1000), (10, 14, 22), (20, 26, 40)).convert("RGBA")
    bg = dot_grid(bg, step=64, alpha=8)
    win = Image.new("RGBA", (1360, 760), (0, 0, 0, 0))
    d = ImageDraw.Draw(win, "RGBA")
    rrect(d, (0, 0, 1360, 760), 18, fill=(13, 15, 20, 252), outline=(255, 255, 255, 40), width=2)
    rrect(d, (0, 0, 1360, 52), 18, fill=(24, 27, 36, 255))
    for i, c in enumerate(((255, 95, 87), (255, 189, 47), (40, 201, 78))):
        d.ellipse([24 + i * 32, 18, 42 + i * 32, 36], fill=c)
    ctext(d, (680, 26), "pwsh — verify", SANS(18), (190, 196, 210, 255), anchor="mm")
    lines = [
        ("PS C:\\tools>", (120, 200, 255), " python verify_hosts.py", (220, 226, 238)),
        ("OK   api.dev.io         -> 10.50.128.32", (120, 220, 140), "", None),
        ("OK   grafana.dev.io     -> 10.50.128.32", (120, 220, 140), "", None),
        ("", None, "", None),
        ("PS C:\\tools>", (120, 200, 255), " jedi hosts switch --env prod", (220, 226, 238)),
        ("✔ 已切换到「生产环境」，2 条记录暂停生效", (150, 210, 250), "", None),
        ("✔ hosts 文件已写回（JEDI 标记区）", (150, 210, 250), "", None),
    ]
    y = 96
    for a, ca, b, cb in lines:
        if a:
            ctext(d, (60, y), a, MONO(26), ca + (255,))
            if b:
                ctext(d, (60 + d.textlength(a, font=MONO(26)), y), b, MONO(26), (cb or (220, 226, 238)) + (255,))
        y += 52
    out = Image.new("RGBA", (1600, 1000), (0, 0, 0, 0))
    out.alpha_composite(bg)
    out.alpha_composite(win, (120, 120))
    save(out, "styled-terminal.png")


def longpage():
    img = vgrad((1200, 1500), (16, 18, 26), (26, 30, 44)).convert("RGBA")
    d = ImageDraw.Draw(img, "RGBA")
    rrect(d, (80, 70, 1120, 1430), 16, fill=(24, 27, 36, 255), outline=(255, 255, 255, 36), width=2)
    ctext(d, (130, 140), "Jedi v0.8.1 发布说明", KAI(56), (245, 246, 250, 255))
    ctext(d, (132, 230), "2026-11-08 · 滚动长图截取", SANS(24), (255, 255, 255, 150))
    d.line([(130, 280), (1070, 280)], fill=(255, 255, 255, 40), width=2)
    secs = [
        ("✨ 新特性", ["滚动截长图：选区固定，滚动窗口自动拼接", "截图美化：背景、圆角、水印一步到位", "OCR 面板：识别结果直接复制/发送到 AI"]),
        ("🛠 修复", ["快捷键在部分输入法下失效", "长图拼接时的重复行检测", "多显示器 DPI 缩放下的选区偏移"]),
        ("📝 其他", ["文档重写，截图全部换新", "依赖升级与性能优化"]),
    ]
    y = 330
    for title, items in secs:
        ctext(d, (130, y), title, SANS_B(30), ACCENT_BLUE + (255,))
        y += 58
        for it in items:
            d.ellipse([142, y + 14, 152, y + 24], fill=(255, 255, 255, 120))
            ctext(d, (170, y + 6), it, SANS(24), (225, 230, 240, 240))
            y += 54
        y += 30
    save(img, "release-notes-longpage.png")


def whiteboard():
    img = vgrad((1600, 900), (238, 240, 244), (224, 228, 236)).convert("RGBA")
    d = ImageDraw.Draw(img, "RGBA")
    notes = [
        (120, 120, " hosts 切换\n 一键生效", ACCENT_BLUE),
        (420, 160, "滚动长图\n 自动拼接", ACCENT_AMBER),
        (760, 110, "本地 OCR\n 隐私安全", ACCENT_GREEN),
        (1080, 170, "MCP + 技能\n 扩展能力", ACCENT_PURPLE),
        (300, 460, "多引擎\n Claude/Codex", ACCENT_PINK),
        (700, 500, "远程订阅\n k8s ingress", ACCENT_CYAN),
        (1050, 480, "知识壁纸\n 每天新知", (219, 88, 96)),
    ]
    for x, y, text, c in notes:
        rrect(d, (x, y, x + 240, y + 190), 10, fill=c + (210,), outline=(0, 0, 0, 30), width=1)
        ctext(d, (x + 120, y + 95), text, SANS_B(28), (255, 255, 255, 245), anchor="mm")
        d.polygon([(x + 110, y + 190), (x + 130, y + 190), (x + 116, y + 214)], fill=(60, 60, 70, 160))
    ctext(d, (800, 800), "团队头脑风暴 · 白板截图 OCR 后直接进周报", SANS(26), (90, 96, 110, 255), anchor="mm")
    save(img, "meeting-whiteboard.png")


# ---------------------------------------------------------------- wall-9 HTTP 状态码
def wall9():
    img, d = new_card((10, 16, 18), (18, 28, 32), ACCENT_GREEN)
    card_header(img, d, "HTTP 状态码速查", "2xx 成功 · 3xx 重定向 · 4xx/5xx 异常", ACCENT_GREEN)
    rows = [
        ("200 OK", "请求成功，返回体在响应里", ACCENT_GREEN),
        ("301 / 302", "永久 / 临时重定向，跟着 Location 走", ACCENT_BLUE),
        ("304 Not Modified", "缓存仍然有效，省一次传输", ACCENT_CYAN),
        ("401 / 403", "未登录 / 无权限，先看鉴权链路", ACCENT_AMBER),
        ("404 Not Found", "资源不存在，检查路由与参数", ACCENT_PINK),
        ("429 Too Many Requests", "触发限流，退避重试", ACCENT_AMBER),
        ("500 / 502 / 503", "服务端异常 / 网关错误 / 过载", ACCENT_PURPLE),
    ]
    y = 300
    for code, desc, c in rows:
        rrect(d, (120, y, 1160, y + 60), 10, fill=c + (34,), outline=c + (150,), width=2)
        ctext(d, (150, y + 30), code, MONO(24), (245, 246, 250, 255), anchor="lm")
        ctext(d, (520, y + 30), desc, SANS(21), (255, 255, 255, 165), anchor="lm")
        y += 76
    save(img, "wall-9.png")


# ---------------------------------------------------------------- wall-10 Docker
def wall10():
    img, d = new_card((9, 12, 30), (16, 20, 44), ACCENT_BLUE)
    card_header(img, d, "Docker 命令速查", "镜像 · 容器 · 日志 · 网络", ACCENT_BLUE)
    cols = [
        ("镜像", ["docker pull nginx:alpine", "docker images", "docker build -t app .", "docker rmi <image>"]),
        ("容器", ["docker run -d -p 8080:80 app", "docker ps -a", "docker exec -it <c> sh", "docker rm -f <c>"]),
        ("运维", ["docker logs -f <c>", "docker stats", "docker compose up -d", "docker system prune"]),
    ]
    x = 110
    for title, lines in cols:
        rrect(d, (x, 300, x + 340, 660), 14, fill=(255, 255, 255, 12), outline=(255, 255, 255, 56), width=2)
        rrect(d, (x + 24, 322, x + 130, 356), 8, fill=ACCENT_BLUE + (60,))
        ctext(d, (x + 77, 339), title, SANS_B(22), (226, 238, 255, 255), anchor="mm")
        yy = 392
        for line in lines:
            ctext(d, (x + 26, yy), "$ " + line, MONO(19), (168, 223, 200, 240))
            yy += 64
        x += 366
    save(img, "wall-10.png")


# ---------------------------------------------------------------- wall-11 设计模式
def wall11():
    img, d = new_card((14, 10, 26), (24, 16, 40), ACCENT_PURPLE)
    card_header(img, d, "设计模式三巨头", "工厂 · 观察者 · 策略", ACCENT_PURPLE)
    panels = [
        ("工厂模式", "把「创建什么」交给子类决定", "Client → Factory → Product", ACCENT_BLUE),
        ("观察者模式", "状态一变，订阅者自动收到通知", "Subject ⇄ Observer ×N", ACCENT_PURPLE),
        ("策略模式", "算法可替换，调用方无感知", "Context → Strategy A/B/C", ACCENT_GREEN),
    ]
    x = 110
    for title, desc, flow, c in panels:
        rrect(d, (x, 300, x + 360, 620), 14, fill=c + (36,), outline=c + (170,), width=2)
        ctext(d, (x + 180, 360), title, KAI(44), (246, 248, 252, 255), anchor="mm")
        ctext(d, (x + 180, 428), desc, SANS(21), (255, 255, 255, 170), anchor="mm")
        rrect(d, (x + 40, 500, x + 320, 548), 10, fill=(255, 255, 255, 12))
        ctext(d, (x + 180, 524), flow, MONO(19), (226, 232, 246, 235), anchor="mm")
        x += 386
    save(img, "wall-11.png")


# ---------------------------------------------------------------- wall-12 CSS
def wall12():
    img, d = new_card((8, 14, 24), (14, 26, 40), ACCENT_CYAN)
    card_header(img, d, "Flex / Grid 速查", "一屏搞定常见布局", ACCENT_CYAN)
    # flex row
    rrect(d, (110, 300, 610, 480), 14, fill=(255, 255, 255, 10), outline=(255, 255, 255, 56), width=2)
    ctext(d, (130, 330), "display: flex; gap: 16px;", MONO(21), (160, 226, 232, 245))
    for i, c in enumerate((ACCENT_BLUE, ACCENT_PURPLE, ACCENT_GREEN)):
        rrect(d, (140 + i * 140, 380, 140 + i * 140 + 120, 440), 8, fill=c + (80,))
    ctext(d, (360, 462), "justify-content / align-items", SANS(19), (255, 255, 255, 150), anchor="mm")
    # grid
    rrect(d, (670, 300, 1170, 480), 14, fill=(255, 255, 255, 10), outline=(255, 255, 255, 56), width=2)
    ctext(d, (690, 330), "grid-template-columns: repeat(3, 1fr);", MONO(21), (160, 226, 232, 245))
    for r in range(2):
        for c in range(3):
            x = 700 + c * 150
            y = 380 + r * 46
            rrect(d, (x, y, x + 130, y + 34), 6, fill=(255, 255, 255, 18))
    ctext(d, (920, 462), "gap 同时作用于行列", SANS(19), (255, 255, 255, 150), anchor="mm")
    # cheats
    tips = [
        ("垂直居中", "align-items: center + justify-content: center"),
        ("等分列", "repeat(auto-fit, minmax(220px, 1fr))"),
        ("吸底栏", "flex-direction: column + margin-top: auto"),
        (" sticky 表头", "position: sticky; top: 0;"),
    ]
    y = 530
    for name, code in tips:
        ctext(d, (130, y), name, SANS_B(22), (244, 248, 252, 245), anchor="lm")
        ctext(d, (420, y), code, MONO(20), (170, 226, 234, 235), anchor="lm")
        y += 62
    save(img, "wall-12.png")


# ---------------------------------------------------------------- gallery extras
def styled_code_card():
    bg = vgrad((1600, 1000), (12, 16, 26), (22, 28, 44)).convert("RGBA")
    bg = dot_grid(bg, step=64, alpha=8)
    win = Image.new("RGBA", (1360, 760), (0, 0, 0, 0))
    d = ImageDraw.Draw(win, "RGBA")
    rrect(d, (0, 0, 1360, 760), 18, fill=(17, 19, 26, 252), outline=(255, 255, 255, 40), width=2)
    rrect(d, (0, 0, 1360, 52), 18, fill=(26, 29, 38, 255))
    for i, c in enumerate(((255, 95, 87), (255, 189, 47), (40, 201, 78))):
        d.ellipse([24 + i * 32, 18, 42 + i * 32, 36], fill=c)
    ctext(d, (680, 26), "nginx.conf — upstream", SANS(18), (190, 196, 210, 255), anchor="mm")
    lines = [
        ("upstream jedi_api ", (255, 123, 114)), ("{", (200, 206, 218)),
        ("    server ", (160, 168, 186)), ("10.50.128.32:8000 ", (168, 223, 142)), ("weight=3;", (160, 168, 186)),
        ("    server ", (160, 168, 186)), ("10.50.128.33:8000 ", (168, 223, 142)), ("backup;", (160, 168, 186)),
        ("}", (200, 206, 218)),
        ("", None),
        ("location ", (255, 123, 114)), ("/api/ ", (168, 223, 142)), ("{", (200, 206, 218)),
        ("    proxy_pass ", (160, 168, 186)), ("http://jedi_api;", (233, 196, 138)),
        ("}", (200, 206, 218)),
    ]
    y = 100
    x = 60
    for text, color in lines:
        if text == "":
            y += 40
            x = 60
            continue
        ctext(d, (x, y), text, MONO(26), (color or (200, 206, 218)) + (255,))
        x += d.textlength(text, font=MONO(26))
    out = Image.new("RGBA", (1600, 1000), (0, 0, 0, 0))
    out.alpha_composite(bg)
    out.alpha_composite(win, (120, 120))
    save(out, "styled-code-card.png")


def styled_standup():
    img = vgrad((1600, 1000), (20, 22, 30), (30, 34, 46)).convert("RGBA")
    d = ImageDraw.Draw(img, "RGBA")
    rrect(d, (110, 80, 1490, 920), 18, fill=(28, 31, 41, 252), outline=(255, 255, 255, 36), width=2)
    ctext(d, (170, 150), "周一站会 · 同步看板", KAI(54), (245, 246, 250, 255))
    cols = [
        ("昨天", ["hosts 远程同步合入", "滚动长图拼接修复"], ACCENT_BLUE),
        ("今天", ["截图美化模板", "hosts 权限错误文案"], ACCENT_GREEN),
        ("阻塞", ["macOS 签名证书待续"], ACCENT_AMBER),
    ]
    x = 170
    for title, items, c in cols:
        rrect(d, (x, 260, x + 400, 640), 14, fill=(255, 255, 255, 10), outline=c + (140,), width=2)
        rrect(d, (x + 24, 286, x + 120, 322), 8, fill=c + (60,))
        ctext(d, (x + 72, 304), title, SANS_B(22), (244, 248, 252, 250), anchor="mm")
        yy = 350
        for it in items:
            rrect(d, (x + 24, yy, x + 376, yy + 90), 10, fill=(255, 255, 255, 12))
            ctext(d, (x + 44, yy + 24), it, SANS(23), (226, 232, 242, 240))
            yy += 104
        x += 436
    ctext(d, (800, 760), "OCR 到会话 → 自动生成周报草稿", SANS(24), (255, 255, 255, 150), anchor="mm")
    save(img, "styled-standup.png")


def main():
    wall1()
    wall2()
    wall3()
    wall4()
    wall5()
    wall6()
    wall7()
    wall8()
    wall9()
    wall10()
    wall11()
    wall12()
    cover("cover-1.png", "内", "内核脉搏", (16, 18, 34), (30, 26, 52), ACCENT_BLUE)
    cover("cover-2.png", "码", "代码与茶", (10, 26, 24), (16, 44, 38), ACCENT_GREEN)
    cover("cover-3.png", "设", "设计迷宫", (30, 14, 24), (48, 22, 36), ACCENT_PINK)
    cover("cover-4.png", "增", "增长手记", (28, 20, 8), (46, 34, 12), ACCENT_AMBER)
    cover("cover-5.png", "片", "片场休息室", (20, 12, 30), (34, 18, 48), ACCENT_PURPLE)
    cover("cover-6.png", "云", "云端夜话", (8, 18, 30), (14, 32, 48), ACCENT_CYAN)
    cover("cover-7.png", "核", "硬核观察", (24, 10, 10), (42, 16, 16), (219, 88, 96))
    cover("cover-8.png", "车", "车间电台", (22, 22, 10), (36, 36, 14), ACCENT_AMBER)
    cover("cover-9.png", "笔", "笔耕不辍", (12, 22, 16), (18, 36, 26), ACCENT_GREEN)
    cover("cover-10.png", "谣", "音乐拾遗", (18, 10, 26), (30, 16, 40), ACCENT_PINK)
    styled_api_docs()
    styled_terminal()
    longpage()
    whiteboard()
    styled_code_card()
    styled_standup()
    print("all assets generated ->", OUT)


if __name__ == "__main__":
    sys.exit(main())
