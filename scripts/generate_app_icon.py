#!/usr/bin/env python3
from __future__ import annotations

import binascii
import math
import struct
import subprocess
import zlib
from pathlib import Path
from tempfile import TemporaryDirectory

SIZE = 1024
ICONSET_SPECS = [
    (16, "16x16"),
    (32, "16x16@2x"),
    (32, "32x32"),
    (64, "32x32@2x"),
    (128, "128x128"),
    (256, "128x128@2x"),
    (256, "256x256"),
    (512, "256x256@2x"),
    (512, "512x512"),
    (1024, "512x512@2x"),
]

OUTER_TOP = (24, 33, 47, 255)
OUTER_BOTTOM = (13, 18, 29, 255)
WINDOW_BG = (15, 17, 26, 255)
WINDOW_TOP = (29, 39, 55, 255)
BORDER = (82, 96, 120, 255)
TEXT = (197, 209, 235, 255)
MUTED = (112, 128, 150, 255)
BLUE = (122, 162, 247, 255)
CYAN = (127, 219, 202, 255)
PURPLE = (192, 153, 255, 255)
RED = (240, 113, 120, 255)
YELLOW = (235, 203, 139, 255)
GREEN = (166, 218, 149, 255)
SHADOW = (0, 0, 0, 72)
GLOW = (122, 162, 247, 38)


def clamp_channel(value: float) -> int:
    return max(0, min(255, round(value)))


class Canvas:
    def __init__(self, width: int, height: int) -> None:
        self.width = width
        self.height = height
        self.pixels = bytearray(width * height * 4)

    def blend_pixel(self, x: int, y: int, color: tuple[int, int, int, int]) -> None:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return
        index = (y * self.width + x) * 4
        src_r, src_g, src_b, src_a = color
        if src_a <= 0:
            return

        dst_r = self.pixels[index]
        dst_g = self.pixels[index + 1]
        dst_b = self.pixels[index + 2]
        dst_a = self.pixels[index + 3]

        src_af = src_a / 255.0
        dst_af = dst_a / 255.0
        out_af = src_af + dst_af * (1.0 - src_af)
        if out_af <= 0:
            return

        out_r = (src_r * src_af + dst_r * dst_af * (1.0 - src_af)) / out_af
        out_g = (src_g * src_af + dst_g * dst_af * (1.0 - src_af)) / out_af
        out_b = (src_b * src_af + dst_b * dst_af * (1.0 - src_af)) / out_af

        self.pixels[index] = clamp_channel(out_r)
        self.pixels[index + 1] = clamp_channel(out_g)
        self.pixels[index + 2] = clamp_channel(out_b)
        self.pixels[index + 3] = clamp_channel(out_af * 255.0)

    def save_png(self, path: Path) -> None:
        rows = []
        stride = self.width * 4
        for y in range(self.height):
            start = y * stride
            rows.append(b"\x00" + bytes(self.pixels[start : start + stride]))
        raw = b"".join(rows)
        data = png_bytes(self.width, self.height, raw)
        path.write_bytes(data)


def png_bytes(width: int, height: int, raw: bytes) -> bytes:
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", binascii.crc32(tag + data) & 0xFFFFFFFF)
    )


def lerp(a: int, b: int, t: float) -> int:
    return round(a + (b - a) * t)


def mix(c1: tuple[int, int, int, int], c2: tuple[int, int, int, int], t: float) -> tuple[int, int, int, int]:
    return tuple(lerp(c1[i], c2[i], t) for i in range(4))  # type: ignore[return-value]


def rounded_rect_mask(x: int, y: int, left: int, top: int, right: int, bottom: int, radius: float) -> float:
    if x < left or y < top or x >= right or y >= bottom:
        return 0.0
    inner_left = left + radius
    inner_right = right - radius
    inner_top = top + radius
    inner_bottom = bottom - radius
    if inner_left <= x < inner_right or inner_top <= y < inner_bottom:
        return 1.0
    corner_x = inner_left if x < inner_left else inner_right
    corner_y = inner_top if y < inner_top else inner_bottom
    distance = math.hypot(x - corner_x, y - corner_y)
    if distance <= radius - 1.0:
        return 1.0
    if distance >= radius + 1.0:
        return 0.0
    return max(0.0, min(1.0, radius + 1.0 - distance))


def draw_rounded_rect(
    canvas: Canvas,
    left: int,
    top: int,
    right: int,
    bottom: int,
    radius: float,
    color_fn,
) -> None:
    for y in range(top, bottom):
        for x in range(left, right):
            alpha = rounded_rect_mask(x, y, left, top, right, bottom, radius)
            if alpha <= 0:
                continue
            r, g, b, a = color_fn(x, y)
            canvas.blend_pixel(x, y, (r, g, b, round(a * alpha)))


def draw_circle(canvas: Canvas, cx: float, cy: float, radius: float, color: tuple[int, int, int, int]) -> None:
    left = max(0, math.floor(cx - radius - 1))
    right = min(canvas.width, math.ceil(cx + radius + 1))
    top = max(0, math.floor(cy - radius - 1))
    bottom = min(canvas.height, math.ceil(cy + radius + 1))
    for y in range(top, bottom):
        for x in range(left, right):
            distance = math.hypot((x + 0.5) - cx, (y + 0.5) - cy)
            if distance <= radius - 1.0:
                alpha = 1.0
            elif distance >= radius + 1.0:
                continue
            else:
                alpha = radius + 1.0 - distance
            canvas.blend_pixel(x, y, (color[0], color[1], color[2], round(color[3] * alpha)))


def draw_soft_shadow(canvas: Canvas, left: int, top: int, right: int, bottom: int, radius: float) -> None:
    draw_rounded_rect(
        canvas,
        left,
        top,
        right,
        bottom,
        radius,
        lambda _x, _y: SHADOW,
    )


def draw_icon(output_path: Path) -> None:
    canvas = Canvas(SIZE, SIZE)

    draw_soft_shadow(canvas, 142, 170, 882, 900, 170)
    draw_rounded_rect(
        canvas,
        110,
        110,
        914,
        914,
        190,
        lambda _x, y: mix(OUTER_TOP, OUTER_BOTTOM, (y - 110) / 804.0),
    )
    draw_rounded_rect(
        canvas,
        170,
        188,
        854,
        836,
        120,
        lambda _x, _y: GLOW,
    )
    draw_rounded_rect(
        canvas,
        166,
        178,
        858,
        846,
        120,
        lambda _x, _y: BORDER,
    )
    draw_rounded_rect(
        canvas,
        182,
        194,
        842,
        830,
        104,
        lambda _x, y: WINDOW_TOP if y < 318 else WINDOW_BG,
    )

    for offset, color in enumerate((RED, YELLOW, GREEN)):
        draw_circle(canvas, 278 + offset * 62, 258, 20, color)

    draw_rounded_rect(
        canvas,
        248,
        388,
        314,
        448,
        20,
        lambda _x, _y: TEXT,
    )
    draw_rounded_rect(
        canvas,
        344,
        388,
        694,
        432,
        22,
        lambda _x, _y: BLUE,
    )
    draw_rounded_rect(
        canvas,
        344,
        462,
        620,
        506,
        22,
        lambda _x, _y: CYAN,
    )
    draw_rounded_rect(
        canvas,
        344,
        536,
        740,
        580,
        22,
        lambda _x, _y: TEXT,
    )
    draw_rounded_rect(
        canvas,
        344,
        610,
        566,
        654,
        22,
        lambda _x, _y: MUTED,
    )
    draw_rounded_rect(
        canvas,
        586,
        610,
        618,
        654,
        14,
        lambda _x, _y: CYAN,
    )

    swatch_y = 706
    swatch_w = 116
    swatch_gap = 28
    swatch_x = 248
    for color in (BLUE, CYAN, PURPLE, TEXT):
        draw_rounded_rect(
            canvas,
            swatch_x,
            swatch_y,
            swatch_x + swatch_w,
            swatch_y + 78,
            28,
            lambda _x, _y, color=color: color,
        )
        swatch_x += swatch_w + swatch_gap

    output_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save_png(output_path)


def render_icns(png_path: Path, icns_path: Path) -> None:
    with TemporaryDirectory() as temp_dir:
        iconset_dir = Path(temp_dir) / "app-icon.iconset"
        iconset_dir.mkdir()

        for size, suffix in ICONSET_SPECS:
            subprocess.run(
                [
                    "sips",
                    "-z",
                    str(size),
                    str(size),
                    str(png_path),
                    "--out",
                    str(iconset_dir / f"icon_{suffix}.png"),
                ],
                check=True,
                stdout=subprocess.DEVNULL,
            )

        subprocess.run(
            ["iconutil", "-c", "icns", str(iconset_dir), "-o", str(icns_path)],
            check=True,
            stdout=subprocess.DEVNULL,
        )


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[1]
    png_output = root / "assets" / "app-icon.png"
    icns_output = root / "assets" / "app-icon.icns"

    draw_icon(png_output)
    render_icns(png_output, icns_output)

    print(png_output)
    print(icns_output)
