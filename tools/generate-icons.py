#!/usr/bin/env python3
"""Regenerate the packaged icon set from the SVG sources in ``assets/``.

The build itself never runs this script: the PNGs and ``app.ico`` it produces are
committed. Re-run it after editing ``assets/logo.svg`` or ``assets/logo-mono.svg``.

    python tools/generate-icons.py

Requires ``rsvg-convert`` (librsvg) on PATH, or the ``cairosvg`` module.
"""

from __future__ import annotations

import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"

BRAND_SIZES = (16, 20, 24, 32, 40, 48, 64, 128, 256, 512)
MONO_SIZES = (24, 32, 48, 64, 256)
ICO_SIZES = (16, 20, 24, 32, 48, 64, 128, 256)

# Below this size the topology mark turns to mush, so the compact artwork
# (cloud only) is used instead.
COMPACT_BELOW = 24


def render(svg: Path, size: int, out: Path) -> None:
    """Rasterise one square PNG at ``size`` pixels."""
    rsvg = shutil.which("rsvg-convert")
    if rsvg:
        subprocess.run(
            [rsvg, "-w", str(size), "-h", str(size), "-o", str(out), str(svg)],
            check=True,
        )
        return

    try:
        import cairosvg  # type: ignore[import-not-found]
    except ImportError as exc:  # pragma: no cover - developer tool
        raise SystemExit("need rsvg-convert or cairosvg to rasterise the SVGs") from exc

    cairosvg.svg2png(url=str(svg), write_to=str(out), output_width=size, output_height=size)


def write_ico(frames: list[Path], out: Path) -> None:
    """Pack PNG frames into a 32-bit ICO (PNG-compressed entries, Vista and later)."""
    payload = [(path, path.read_bytes()) for path in frames]
    header = struct.pack("<HHH", 0, 1, len(payload))
    offset = 6 + 16 * len(payload)
    entries = bytearray()
    for path, data in payload:
        size = int(path.stem.rsplit("-", 1)[1])
        entries += struct.pack(
            "<BBBBHHII",
            0 if size >= 256 else size,
            0 if size >= 256 else size,
            0,  # palette size
            0,  # reserved
            1,  # colour planes
            32,  # bits per pixel
            len(data),
            offset,
        )
        offset += len(data)
    out.write_bytes(header + bytes(entries) + b"".join(data for _, data in payload))


def main() -> int:
    brand = ASSETS / "logo.svg"
    compact = ASSETS / "logo-compact.svg"
    mono = ASSETS / "logo-mono.svg"
    for svg in (brand, compact, mono):
        if not svg.is_file():
            raise SystemExit(f"missing source artwork: {svg}")

    for size in BRAND_SIZES:
        target = ASSETS / f"icon-{size}.png"
        render(compact if size < COMPACT_BELOW else brand, size, target)
        print(f"wrote {target.relative_to(ROOT)}")

    for size in MONO_SIZES:
        target = ASSETS / f"mono-{size}.png"
        render(mono, size, target)
        print(f"wrote {target.relative_to(ROOT)}")

    # The ICO wants a stable, ordered frame list; 256 must be last.
    frames = [ASSETS / f"icon-{size}.png" for size in ICO_SIZES]
    missing = [path.name for path in frames if not path.is_file()]
    if missing:
        raise SystemExit(f"cannot build app.ico, missing: {', '.join(missing)}")

    write_ico(frames, ASSETS / "app.ico")
    print(f"wrote {(ASSETS / 'app.ico').relative_to(ROOT)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
