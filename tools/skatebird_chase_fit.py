"""Fit SkateBIRD's chase camera (D back, H up, pitch P, vertical FOV F) to skate. third-person shots of the same spawn.

  python -I tools/skatebird_chase_fit.py run LEVEL OUTDIR TARGET.png [TARGET.png ...]   (in kit/run game: opens the game)
  python -I tools/skatebird_chase_fit.py fit OUTDIR TARGET.png [TARGET.png ...]          (offline, on OUTDIR's renders)
  python -I tools/skatebird_chase_fit.py views OUTDIR                                    (remake the images of fit.json)

A rotation or zoom of a pinhole camera only re-maps pixels, so the game is needed for the camera *position* only: `run`
renders the level's spawn view (data/skatebird-views.json, view 0) from a grid of positions (view point - D m along
the yaw + H m up + L m to the right) at pitch 0 and a wide vertical FOV (RENDER_FOV), sky flat black, HUD/bird/lens effects off, 16:9.
Each target is then fitted per render by warping that render to pitch P, a yaw offset dY and vertical FOV F and
comparing edges. The score is the correlation of blurred gradient-orientation fields (doubled angle, contrast
normalised), so colour, lighting and sky differences between the games count little; the centre-bottom (skate's
skater) and the top-right corner (skate's FPS counter) are masked. `run` renders a coarse grid, fits, renders a fine
grid around each target's best and fits again, in one launch. Writes OUTDIR/fit.json and, per target,
OUTDIR/<run>_<target stem>_skatebird.png (the best render warped to the fitted pose), _overlay.png (edges: target
green, SkateBIRD magenta, both white) and _pair.png (tools/compare_shots.py of the two).

Camera convention (Unity): rotation Euler(P, yaw + dY, 0), positive P looks down; position = p - D*fwd + H*up with fwd
the horizontal unit vector of the yaw and p the view point on the ground.
"""
import argparse
import json
import math
import os
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage, optimize

sys.path.insert(0, str(Path(__file__).resolve().parent))
import skatebird_views as sv  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
RENDER_W, RENDER_H = 1920, 1080
RENDER_FOV = 100.0  # vertical; covers F <= 70 at |P| <= 15
# Position grids (metres): D back along the yaw, H up, L right of the yaw. The coarse grid brackets the first level2
# fit (2026-10-19: H 1.5 sharp, D <= 0 best on a 0..1 m strip, L untested); the fine grid is around each target's best.
COARSE = {"D": [round(-1.5 + 0.25 * i, 2) for i in range(13)], "H": [round(1.2 + 0.1 * i, 2) for i in range(7)],
          "L": [round(-0.6 + 0.3 * i, 2) for i in range(5)]}
FINE = {"D": (0.1, 2), "H": (0.05, 2), "L": (0.1, 2)}  # step, steps each side
BOUNDS = {"P": (-20.0, 30.0), "dY": (-15.0, 15.0), "F": (30.0, 90.0)}


# ---- geometry -------------------------------------------------------------------------------------------------------

def rot_x(a):
    c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])


def rot_y(a):
    c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
    return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])


def chase_position(view, d, h, l=0.0):
    """view point - d along the yaw's horizontal forward + h up + l along its right."""
    x, y, z = view["position"]
    a = math.radians(view["yaw_deg"])
    return [x - d * math.sin(a) + l * math.cos(a), y + h, z - d * math.cos(a) - l * math.sin(a)]


def warp_coords(render, w, h, p, dy, f):
    """Render pixel coordinates (rows, cols) of each pixel of a w x h view at pitch p, yaw offset dy, vertical FOV f.

    The render was taken at pitch 0 and yaw offset 0 from the same position, with vertical FOV render.fov."""
    ft = (h / 2) / math.tan(math.radians(f) / 2)
    fr = (render.h / 2) / math.tan(math.radians(render.fov) / 2)
    u, v = np.meshgrid(np.arange(w) + 0.5 - w / 2, np.arange(h) + 0.5 - h / 2)
    c = np.stack([u / ft, -v / ft, np.ones_like(u)], axis=-1)
    r = c @ (rot_y(dy) @ rot_x(p)).T
    z = np.where(r[..., 2] > 1e-6, r[..., 2], np.nan)
    cols = render.w / 2 + fr * r[..., 0] / z - 0.5
    rows = render.h / 2 - fr * r[..., 1] / z - 0.5
    return rows, cols


# ---- features -------------------------------------------------------------------------------------------------------

def gray(path):
    return np.asarray(Image.open(path).convert("L"), dtype=np.float32) / 255.0


def features(img, sigma):
    """Doubled-angle gradient orientation, contrast normalised, blurred by sigma; NaN pixels give no edges."""
    valid = np.isfinite(img)
    img = np.where(valid, img, 0.0)
    g = ndimage.gaussian_filter(img, 1.0)
    gx, gy = ndimage.sobel(g, 1), ndimage.sobel(g, 0)
    m2 = gx * gx + gy * gy
    m = np.sqrt(m2)
    ref = np.percentile(m[valid], 95) + 1e-6 if valid.any() else 1.0
    w = np.minimum(m / ref, 1.0) * ndimage.binary_erosion(valid, iterations=2)
    with np.errstate(invalid="ignore", divide="ignore"):
        c2 = np.where(m2 > 0, (gx * gx - gy * gy) / m2, 0.0)
        s2 = np.where(m2 > 0, 2 * gx * gy / m2, 0.0)
    return np.stack([ndimage.gaussian_filter(w * c2, sigma), ndimage.gaussian_filter(w * s2, sigma)])


def target_mask(w, h):
    m = np.ones((h, w), bool)
    m[int(0.55 * h):, int(0.35 * w):int(0.65 * w)] = False  # skate's skater
    m[: int(0.06 * h), int(0.9 * w):] = False  # skate's FPS counter
    return m


class Target:
    def __init__(self, path, w, sigma):
        img = Image.open(path).convert("L")
        h = round(w * img.height / img.width)
        self.w, self.h = w, h
        self.img = np.asarray(img.resize((w, h), Image.LANCZOS), dtype=np.float32) / 255.0
        self.mask = target_mask(w, h)
        self.feat = features(self.img, sigma) * self.mask
        self.norm = float(np.sqrt((self.feat ** 2).sum()))
        self.sigma = sigma


class Render:
    def __init__(self, path, fov):
        self.full = gray(path)
        self.h, self.w = self.full.shape
        self.fov = fov
        self.cache = {}

    def at(self, blur):
        if blur not in self.cache:
            self.cache[blur] = ndimage.gaussian_filter(self.full, blur) if blur > 0 else self.full
        return self.cache[blur]


def view_image(render, t, p, dy, f):
    rows, cols = warp_coords(render, t.w, t.h, p, dy, f)
    fr = (render.h / 2) / math.tan(math.radians(render.fov) / 2)
    ft = (t.h / 2) / math.tan(math.radians(f) / 2)
    blur = round(max(0.0, 0.5 * fr / ft - 0.3), 1)
    inside = np.isfinite(rows) & (rows >= 0) & (rows <= render.h - 1) & (cols >= 0) & (cols <= render.w - 1)
    out = ndimage.map_coordinates(render.at(blur), [np.nan_to_num(rows), np.nan_to_num(cols)], order=1)
    return np.where(inside, out, np.nan)


def score(render, t, p, dy, f):
    fc = features(view_image(render, t, p, dy, f), t.sigma) * t.mask
    n = float(np.sqrt((fc ** 2).sum()))
    return float((fc * t.feat).sum() / (n * t.norm + 1e-9))


def fit_render(render, t, starts):
    """Best (score, P, dY, F) by Nelder-Mead from each start."""
    best = None
    for x0 in starts:
        def neg(x):
            p, dy, f = x
            if not (BOUNDS["P"][0] <= p <= BOUNDS["P"][1] and BOUNDS["dY"][0] <= dy <= BOUNDS["dY"][1]
                    and BOUNDS["F"][0] <= f <= BOUNDS["F"][1]):
                return 1.0
            return -score(render, t, p, dy, f)
        r = optimize.minimize(neg, x0, method="Nelder-Mead",
                              options={"xatol": 0.05, "fatol": 1e-4, "maxfev": 150,
                                       "initial_simplex": [x0, [x0[0] + 3, x0[1], x0[2]], [x0[0], x0[1] + 3, x0[2]],
                                                           [x0[0], x0[1], x0[2] + 6]]})
        cand = (-float(r.fun), *map(float, r.x))
        if best is None or cand[0] > best[0]:
            best = cand
    return best


def grid_start(render, t):
    """Coarse exhaustive (P, dY, F) for one render: the start for the per-render searches."""
    best = None
    for p in np.arange(-10, 22.5, 2.5):
        for dy in np.arange(-8, 9, 2):
            for f in np.arange(40, 85, 5):
                s = score(render, t, p, dy, f)
                if best is None or s > best[0]:
                    best = (s, float(p), float(dy), float(f))
    return best


# ---- fitting over the renders ---------------------------------------------------------------------------------------

def load_renders(outdir):
    index = json.loads((outdir / "renders.json").read_text(encoding="utf-8"))
    return index, [r for r in index["renders"] if not r["failed"]]


_targets = {}


def _fit_one(job):
    """One render against one target (a worker process; targets are cached per process)."""
    render_path, fov, target_path, w, sigma, starts = job
    key = (target_path, w, sigma)
    if key not in _targets:
        _targets[key] = Target(target_path, w, sigma)
    return fit_render(Render(render_path, fov), _targets[key], starts)


def fit_target(target_path, outdir, renders, fov, log):
    """Per target: coarse features (320 px, sigma 3) over all renders, then fine (640 px, sigma 1.5) on the best 8."""
    coarse = Target(target_path, 320, 3.0)
    md, mh = np.median([m["D"] for m in renders]), np.median([m["H"] for m in renders])
    mid = min(renders, key=lambda m: abs(m["D"] - md) + abs(m["H"] - mh) + abs(m.get("L", 0.0)))
    g = grid_start(Render(outdir / mid["file"], fov), coarse)
    log(f"  grid start on D {mid['D']} H {mid['H']}: score {g[0]:.4f} P {g[1]} dY {g[2]} F {g[3]}")
    workers = max(1, min(12, (os.cpu_count() or 2) - 2))
    with ProcessPoolExecutor(workers) as pool:
        jobs = [(str(outdir / m["file"]), fov, str(target_path), 320, 3.0, [g[1:]]) for m in renders]
        results = [{"D": m["D"], "H": m["H"], "L": m.get("L", 0.0), "file": m["file"], "coarse": list(r)}
                   for m, r in zip(renders, pool.map(_fit_one, jobs))]
        results.sort(key=lambda r: -r["coarse"][0])
        top = results[:8]
        jobs = [(str(outdir / r["file"]), fov, str(target_path), 640, 1.5, [r["coarse"][1:]]) for r in top]
        for r, fine in zip(top, pool.map(_fit_one, jobs)):
            r["fine"] = list(fine)
    best = max(top, key=lambda r: r["fine"][0])
    return best, results


def outputs(target_path, render_path, fov, best, outdir, stem):
    """<stem>_skatebird.png: the best render warped to the fitted pose (the pinhole view a camera at that pose takes;
    sky flat black); <stem>_overlay.png: edges, target green, SkateBIRD magenta, both white; <stem>_pair.png:
    tools/compare_shots.py of the two."""
    t = Target(target_path, 1280, 1.0)
    s, p, dy, f = best["fine"]
    rows, cols = warp_coords(Render(render_path, fov), t.w, t.h, p, dy, f)
    rgb = np.asarray(Image.open(render_path).convert("RGB"), dtype=np.float32)
    inside = np.isfinite(rows)
    coords = [np.nan_to_num(rows), np.nan_to_num(cols)]
    view = np.stack([ndimage.map_coordinates(ndimage.gaussian_filter(rgb[..., c], 0.6), coords, order=1)
                     for c in range(3)], axis=-1)
    view = np.where(inside[..., None], view, 0).clip(0, 255).astype(np.uint8)
    Image.fromarray(view).save(outdir / f"{stem}_skatebird.png")

    def edges(img):
        g = ndimage.gaussian_filter(img, 1.0)
        m = np.hypot(ndimage.sobel(g, 1), ndimage.sobel(g, 0))
        return m > np.percentile(m, 90)

    et, ev = edges(t.img), edges(np.asarray(Image.fromarray(view).convert("L"), dtype=np.float32) / 255.0)
    over = np.zeros((t.h, t.w, 3), np.uint8)
    over[..., 1] = et * 255
    over[..., 0] = ev * 255
    over[..., 2] = ev * 255
    Image.fromarray(over).save(outdir / f"{stem}_overlay.png")
    subprocess.run([sys.executable, "-I", str(ROOT / "tools/compare_shots.py"), str(outdir / f"{stem}_pair.png"),
                    str(outdir / f"{stem}_skatebird.png"), str(target_path), "--labels",
                    f"SkateBIRD fit: D {best['D']} H {best['H']} L {best['L']} P {p:.1f} dY {dy:.1f} F {f:.1f}",
                    "skate. target"], check=True, capture_output=True)


def fit_all(outdir, targets, log):
    index, renders = load_renders(outdir)
    fits = {}
    for tp in targets:
        log(f"fit {tp} over {len(renders)} renders")
        best, results = fit_target(tp, outdir, renders, index["render_fov"], log)
        s, p, dy, f = best["fine"]
        log(f"  best D {best['D']} H {best['H']} L {best['L']} P {p:.2f} dY {dy:.2f} F {f:.2f} score {s:.4f}")
        fits[str(tp)] = {"best": best, "top": results[:8]}
    return index, renders, fits


# ---- the game run ---------------------------------------------------------------------------------------------------

def render_grid(game, view, outdir, grid, index, log):
    done = {r["key"] for r in index["renders"]}
    cells = [(d, h, l) for d in grid["D"] for h in grid["H"] for l in grid["L"]]
    for n, (d, h, l) in enumerate(cells):
        key = f"D{d:+.2f}_H{h:.2f}_L{l:+.2f}"
        if key in done:
            continue
        done.add(key)
        pos = chase_position(view, d, h, l)
        path = (outdir / "renders" / f"{key}.png").resolve()
        game.send("camera", position=pos, yaw=view["yaw_deg"], pitch=0.0, fov=RENDER_FOV,
                  settle_frames=8, settle_seconds=0.0, flat_sky=True)
        shot = game.send("shot", path=str(path))
        bad = sv.checks({"position": pos, "yaw": view["yaw_deg"], "pitch": 0.0, "fov": RENDER_FOV}, shot)
        bad = [b for b in bad if not b.startswith("screen ")]
        if (shot["screen_width"], shot["screen_height"]) != (RENDER_W, RENDER_H):
            bad.append(f"screen {shot['screen_width']}x{shot['screen_height']}")
        index["renders"].append({"key": key, "D": d, "H": h, "L": l, "file": f"renders/{key}.png",
                                 "position": pos, "readback": sv.pose(shot), "failed": bad})
        if bad:
            log(f"  {key}: FAILED {'; '.join(bad)}")
        if n % 25 == 0:
            (outdir / "renders.json").write_text(json.dumps(index, indent=1) + "\n", encoding="utf-8")
    (outdir / "renders.json").write_text(json.dumps(index, indent=1) + "\n", encoding="utf-8")


def run(args, log):
    level = json.loads((ROOT / "data/skatebird-views.json").read_text(encoding="utf-8"))[args.level]
    view = level["views"][0]
    outdir = Path(args.outdir)
    (outdir / "renders").mkdir(parents=True, exist_ok=True)
    index = {"level": args.level, "scene": level["name"], "view": view, "render_fov": RENDER_FOV,
             "render_size": [RENDER_W, RENDER_H], "renders": []}
    game = sv.Game()
    error = None
    try:
        game.start()
        game.send("screen", timeout=120, width=RENDER_W, height=RENDER_H)
        game.send("load", timeout=150, scene=level["name"], settle_frames=30, settle_seconds=8.0)
        t0 = time.time()
        render_grid(game, view, outdir, COARSE, index, log)
        log(f"coarse grid: {len(index['renders'])} renders in {time.time() - t0:.0f} s")
        _, _, fits = fit_all(outdir, args.targets, log)
        for fit in fits.values():
            b = fit["best"]
            grid = {k: [round(b[k] + step * i, 2) for i in range(-n, n + 1)] for k, (step, n) in FINE.items()}
            render_grid(game, view, outdir, grid, index, log)
        log(f"all renders: {len(index['renders'])}")
        game.send("release")
    except RuntimeError as e:
        error = e
        log(f"ERROR {e}; fitting the {len(index['renders'])} renders made so far")
        (outdir / "renders.json").write_text(json.dumps(index, indent=1) + "\n", encoding="utf-8")
    finally:
        game.stop(False)
        if sv.LOG.exists():
            (outdir / "plugin.log").write_bytes(sv.LOG.read_bytes())
    if index["renders"]:
        finish(outdir, args.targets, log)
    if error is not None:
        sys.exit(1)


def finish(outdir, targets, log):
    index, renders, fits = fit_all(outdir, targets, log)
    for tp, fit in fits.items():
        stem = Path(tp).parent.parent.name + "_" + Path(tp).stem
        outputs(tp, outdir / fit["best"]["file"], index["render_fov"], fit["best"], outdir, stem)
    (outdir / "fit.json").write_text(json.dumps({"targets": fits, "render_fov": index["render_fov"]}, indent=1) + "\n",
                                     encoding="utf-8")
    failed = sum(bool(r["failed"]) for r in index["renders"])
    log(f"{len(index['renders'])} renders ({failed} with failed read-backs) -> {outdir / 'fit.json'}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("level")
    r.add_argument("outdir")
    r.add_argument("targets", nargs="+")
    f = sub.add_parser("fit")
    f.add_argument("outdir")
    f.add_argument("targets", nargs="+")
    v = sub.add_parser("views", help="remake the pair/overlay images from OUTDIR/fit.json without refitting")
    v.add_argument("outdir")
    args = ap.parse_args()

    def log(msg):
        print(msg, flush=True)

    if args.cmd == "run":
        run(args, log)
    elif args.cmd == "fit":
        finish(Path(args.outdir), args.targets, log)
    else:
        outdir = Path(args.outdir)
        fit = json.loads((outdir / "fit.json").read_text(encoding="utf-8"))
        for tp, t in fit["targets"].items():
            stem = Path(tp).parent.parent.name + "_" + Path(tp).stem
            outputs(tp, outdir / t["best"]["file"], fit["render_fov"], t["best"], outdir, stem)
            log(f"{outdir / (stem + '_pair.png')}")


if __name__ == "__main__":
    main()
