"""Draws the SM7 app icon (the in-app mark: a ring with a core on a rounded accent square)
and writes assets/sm7.ico (16-256 px, PNG-compressed entries) and assets/sm7-256.png.
Pure Python (no Pillow): each pixel is supersampled 4x4 for smooth edges.
Run: python -I tools/make_icon.py
"""
import os, struct, zlib

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "assets")
ACC = (0x4C, 0x8D, 0xFF)
INK = (0xFF, 0xFF, 0xFF)


def coverage(x, y, n):
    """(square alpha, ink alpha) at a sample point in a unit square scaled to n px."""
    u, v = x / n, y / n
    r = 0.22  # corner radius
    m = 0.04  # margin
    lo, hi = m, 1 - m
    cx = min(max(u, lo + r), hi - r)
    cy = min(max(v, lo + r), hi - r)
    inside = lo <= u <= hi and lo <= v <= hi and (u - cx) ** 2 + (v - cy) ** 2 <= r * r
    d = ((u - 0.5) ** 2 + (v - 0.5) ** 2) ** 0.5
    ring = abs(d - 0.26) <= 0.055
    core = d <= 0.10
    return inside, inside and (ring or core)


def render(n):
    ss = 4
    rows = []
    for y in range(n):
        row = bytearray()
        for x in range(n):
            a = ink = 0
            for sy in range(ss):
                for sx in range(ss):
                    i, k = coverage(x + (sx + 0.5) / ss, y + (sy + 0.5) / ss, n)
                    a += i
                    ink += k
            a /= ss * ss
            k = ink / (ss * ss) / a if a else 0
            col = [round(ACC[j] + (INK[j] - ACC[j]) * k) for j in range(3)]
            row += bytes(col + [round(a * 255)])
        rows.append(bytes(row))
    return rows


def png(n, rows):
    raw = b"".join(b"\0" + r for r in rows)
    chunk = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", n, n, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def main():
    os.makedirs(OUT, exist_ok=True)
    sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    imgs = [(n, png(n, render(n))) for n in sizes]
    head = struct.pack("<HHH", 0, 1, len(imgs))
    off = 6 + 16 * len(imgs)
    dirs, data = b"", b""
    for n, b in imgs:
        dirs += struct.pack("<BBBBHHII", n % 256, n % 256, 0, 0, 1, 32, len(b), off + len(data))
        data += b
    open(os.path.join(OUT, "sm7.ico"), "wb").write(head + dirs + data)
    open(os.path.join(OUT, "sm7-256.png"), "wb").write(dict(imgs)[256])
    print("wrote assets/sm7.ico and assets/sm7-256.png")


if __name__ == "__main__":
    main()
