"""Side-by-side comparison sheet for SkateBIRD vs ReSkate screenshots.

    python tools/compare_shots.py out.png <left.png> <right.png> [--labels "SkateBIRD" "skate."]

Both images are scaled to the same height; the result is left|right with labels. Used as the visual
verification artifact for a level port.
"""
import argparse
from pathlib import Path

from PIL import Image, ImageDraw


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("left")
    ap.add_argument("right")
    ap.add_argument("--labels", nargs=2, default=["SkateBIRD", "skate. (ReSkate)"])
    ap.add_argument("--height", type=int, default=720)
    a = ap.parse_args()

    ims = []
    for p in (a.left, a.right):
        im = Image.open(p).convert("RGB")
        w = round(im.width * a.height / im.height)
        ims.append(im.resize((w, a.height), Image.LANCZOS))
    lab = 34
    out = Image.new("RGB", (sum(i.width for i in ims) + 8, a.height + lab), (16, 16, 16))
    d = ImageDraw.Draw(out)
    x = 4
    for im, name in zip(ims, a.labels):
        out.paste(im, (x, lab))
        d.text((x + 8, 10), name, fill=(240, 240, 240))
        x += im.width + 8
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    out.save(a.out)
    print(a.out, out.size)


if __name__ == "__main__":
    main()
