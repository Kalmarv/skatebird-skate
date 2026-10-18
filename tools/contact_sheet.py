"""Contact sheet of per-level comparison images (2 x 4 grid)."""
import sys
from pathlib import Path
from PIL import Image

levels = ["level1", "level2", "level3", "level4", "level5", "level7", "level12", "level13"]
root = Path("out")
ims = []
for l in levels:
    p = root / f"compare_{l}.png"
    if not p.exists():
        print("missing", p); continue
    im = Image.open(p).convert("RGB")
    w = 760
    ims.append(im.resize((w, round(im.height * w / im.width)), Image.LANCZOS))
if not ims:
    sys.exit("no comparisons")
cols = 2
rows = (len(ims) + cols - 1) // cols
cw, ch = ims[0].size
sheet = Image.new("RGB", (cw * cols + 8 * (cols + 1), ch * rows + 8 * (rows + 1)), (16, 16, 16))
for i, im in enumerate(ims):
    r, c = divmod(i, cols)
    sheet.paste(im, (8 + c * (cw + 8), 8 + r * (ch + 8)))
sheet.save(root / "compare_contact.png")
print(root / "compare_contact.png", sheet.size)
