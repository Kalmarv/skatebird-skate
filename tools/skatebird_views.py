"""SkateBIRD reference shots at exact camera poses, one per view of data/skatebird-views.json.

  python -I tools/skatebird_views.py LEVEL OUTDIR [--chase] [--back D] [--eye H] [--fov F] [--pitch P] [--yaw-offset Y]
                                     [--settle S] [--keep-post] [--keep-npcs] [--keep]

(Run it through tools/skatebird_views.sh inside `kit/run game`: it opens a game window.) LEVEL may be several levels
joined by commas: one launch shoots them all, each into OUTDIR/<level>.

Launches the lab copy (tools/skatebird_lab.sh makes it) windowed at 1024x576, loads the level's scene through the lab
plugin, and for each view locks Camera.main at (view position - D metres along the yaw + H metres up), yaw = the view's
yaw_deg + Y (Unity: degrees from +Z towards +X), pitch P (Unity Euler x, positive looks down), vertical FOV F; defaults
from the preset, "eye" (D 0, H 1.6, P 0, F 60) or with --chase skate.'s third-person camera (MODES), with the
HUD/menus and the player's renderers hidden. After the settle it saves OUTDIR/<view index>_<name>.png (the rendered
frame, read back in game) and writes OUTDIR/poses.json: per shot the requested pose, the plugin's read-back of the
frame that was saved, a read-back after the save, and the checks (read-back = request; no menu, HUD, bird or lens
effects; no collider within 0.2 m of the camera; a collider below it). Unless --keep-npcs, the NPC birds (every
BirbDecorator but the player's) and every other SkinnedMeshRenderer are hidden too; poses.json counts them
(npcs_hidden, and per shot). OUTDIR/checks/player_{shown,hidden}.png are the
control for the bird hiding (one pose looking at the bird, with and without it). OUTDIR/plugin.log (the parent's, with
several levels) is the run's BepInEx log. Exit 1 if any check failed.

Unless --keep-post, the plugin turns off the game's depth of field (focused on the bird ~3 m away, it blurs
everything else), lens distortion, chromatic aberration, vignette and motion blur for the shots.
"""
import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAB = Path(os.environ.get("SKATEBIRD_LAB", "C:/Users/kalmarv/modding/cache/skatebird/lab"))
EXE = LAB / "SkateBIRD.exe"
CONTROL = LAB / "control"
COMMAND = CONTROL / "command.json"
READY = CONTROL / "ready.json"
REPLY = CONTROL / "reply.json"
LOG = LAB / "BepInEx" / "LogOutput.log"
WIDTH, HEIGHT = 1024, 576
# Camera presets: back = metres behind the view point along its yaw, eye = metres above it, pitch (Unity, positive looks
# down), fov (vertical), yaw_offset (added to the view's yaw). "chase" is skate.'s third-person camera as fitted at
# level2's spawn (tools/skatebird_chase_fit.py, job 0da1c4ce, cache/skatebird/chase-fit/level2/fit.json): two skate
# launches agree on the position (D -0.25/-0.35, H 1.5, L -0.2/0.0; the score is flat over +-0.15 m in D and L) and FOV
# (55.4/55.2) but not on the orientation (P +1.2, dY -0.3 in t1; P -6.2, dY +4.2 in t2); the preset takes t1, whose
# camera looks along the skater's heading.
MODES = {
    "eye": {"back": 0.0, "eye": 1.6, "pitch": 0.0, "fov": 60.0, "yaw_offset": 0.0},
    "chase": {"back": -0.3, "eye": 1.5, "pitch": 1.2, "fov": 55.3, "yaw_offset": 0.0},
}
POS_TOL = 1e-3  # metres
ANGLE_TOL = 0.05  # degrees
# What the plugin turns off unless --keep-post: they make a frame differ from a pinhole render of the scene.
POST_OFF = {"DepthOfField", "LensDistortion", "ChromaticAberration", "Vignette", "MotionBlur"}


class Game:
    def __init__(self):
        self.seq = 0
        self.proc = None

    def start(self):
        if running():
            sys.exit("SkateBIRD is already running; close it first")
        for f in (COMMAND, REPLY, READY):
            f.unlink(missing_ok=True)
        args = self.launch_args()
        print("launch", " ".join(args), flush=True)
        self.proc = subprocess.Popen(args, cwd=str(LAB))
        # The plugin clears the control folder when it starts; nothing may be sent before it says it is ready.
        end = time.time() + 120
        while time.time() < end:
            if self.proc.poll() is not None:
                raise RuntimeError(f"game exited (code {self.proc.returncode}) before the plugin was ready")
            try:
                if json.loads(READY.read_text(encoding="utf-8")).get("pid") == self.proc.pid:
                    return
            except (FileNotFoundError, PermissionError, json.JSONDecodeError):
                pass
            time.sleep(0.1)
        raise RuntimeError(f"the plugin did not write {READY} in 120 s (is BepInEx loading? see {LOG})")

    @staticmethod
    def launch_args():
        return [str(EXE), "-screen-fullscreen", "0", "-screen-width", str(WIDTH), "-screen-height", str(HEIGHT)]

    def send(self, op, timeout=30.0, **fields):
        self.seq += 1
        cmd = {"seq": self.seq, "op": op, **fields}
        CONTROL.mkdir(parents=True, exist_ok=True)
        tmp = COMMAND.with_suffix(".tmp")
        tmp.write_text(json.dumps(cmd), encoding="utf-8")
        for attempt in range(200):  # the plugin may be reading the old command (no delete sharing on Windows)
            try:
                os.replace(tmp, COMMAND)
                break
            except PermissionError:
                if attempt == 199:
                    raise
                time.sleep(0.01)
        end = time.time() + timeout
        while time.time() < end:
            if self.proc.poll() is not None:
                raise RuntimeError(f"game exited (code {self.proc.returncode}) during {op}")
            try:
                reply = json.loads(REPLY.read_text(encoding="utf-8"))
            except (FileNotFoundError, PermissionError, json.JSONDecodeError):
                reply = None
            if reply and reply.get("seq") == self.seq:
                if not reply.get("ok"):
                    raise RuntimeError(f"{op} failed: {reply.get('error')}")
                return reply
            time.sleep(0.05)
        raise RuntimeError(f"no reply to {op} (seq {self.seq}) in {timeout:.0f} s")

    def stop(self, keep):
        if self.proc is None or self.proc.poll() is not None or keep:
            return
        try:
            self.send("quit", timeout=10)
            self.proc.wait(timeout=20)
        except (RuntimeError, subprocess.TimeoutExpired):
            pass
        if self.proc.poll() is None:
            subprocess.run(["taskkill", "/PID", str(self.proc.pid), "/F"], capture_output=True)
            try:
                self.proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                print(f"warning: SkateBIRD pid {self.proc.pid} still running after taskkill", flush=True)


def running():
    out = subprocess.run(["tasklist", "/FI", "IMAGENAME eq SkateBIRD.exe"], capture_output=True, text=True).stdout
    return "skatebird.exe" in out.lower()


def angle_diff(a, b):
    return (a - b + 180.0) % 360.0 - 180.0


def checks(want, got, player_hidden=True, keep_post=False):
    """Failed checks of one read-back against the requested pose."""
    bad = []
    if len(got.get("cam_position") or []) != 3:
        return ["no camera"]
    d = max(abs(g - w) for g, w in zip(got["cam_position"], want["position"]))
    if d > POS_TOL:
        bad.append(f"position off by {d:.4f} m")
    yaw, pitch, roll = got["cam_euler"][1], got["cam_euler"][0], got["cam_euler"][2]
    for name, g, w in (("yaw", yaw, want["yaw"]), ("pitch", pitch, want["pitch"]), ("roll", roll, 0.0)):
        if abs(angle_diff(g, w)) > ANGLE_TOL:
            bad.append(f"{name} {g:.3f} != {w:.3f}")
    if abs(got["cam_fov"] - want["fov"]) > ANGLE_TOL or abs(got["cam_fov_projection"] - want["fov"]) > ANGLE_TOL:
        bad.append(f"fov {got['cam_fov']:.3f} (projection {got['cam_fov_projection']:.3f}) != {want['fov']}")
    if (got["screen_width"], got["screen_height"]) != (WIDTH, HEIGHT):
        bad.append(f"screen {got['screen_width']}x{got['screen_height']}")
    if not got["locked"]:
        bad.append("camera not locked")
    if got["gameplay_obstructed"]:
        bad.append("a game menu is open (UIManager.IsGameplayObstructed)")
    if player_hidden and (not got["player_roots"] or any("skipped" in r for r in got["player_roots"])
                          or got["hidden_renderers"] == 0):
        bad.append(f"player not hidden (roots {got['player_roots']}, {got['hidden_renderers']} renderers hidden)")
    screen_canvases = [c for c in got["kept_canvases"] if not c.endswith("WorldSpace")]
    if screen_canvases:
        bad.append(f"screen canvases still on: {screen_canvases}")
    post_on = POST_OFF & set(got.get("post_effects_on") or [])
    if post_on and not keep_post:
        bad.append(f"post effects still on: {sorted(post_on)}")
    if got.get("cam_overlaps"):
        bad.append(f"camera within 0.2 m of colliders {got['cam_overlaps']} (in or against geometry)")
    if got.get("cam_ground", 0) < 0:
        bad.append("no collider below the camera")
    return bad


def pose(r):
    keys = ("frame", "time", "cam_name", "cam_position", "cam_euler", "cam_forward", "cam_bearing", "cam_fov",
            "cam_fov_projection", "cam_near", "cam_far", "screen_width", "screen_height", "player_position",
            "player_yaw", "gameplay_obstructed", "hidden_canvases", "hidden_renderers", "player_roots",
            "kept_canvases", "other_cameras", "post_effects_off", "post_effects_on", "cam_overlaps", "cam_ground", "npc_birbs", "npc_skinned_other",
            "hidden_npc_renderers")
    return {k: r.get(k) for k in keys}


def player_check(game, outdir, yaw, args):
    """Control for the bird hiding: the same pose 3 m behind and 1.5 m above the player, shown and hidden.

    The views put the camera over the player's spawn, where the bird is usually out of frame anyway, so they alone do
    not show that hiding works."""
    p = game.send("readback")["player_position"]
    a = math.radians(yaw)
    want = {"position": [p[0] - 3 * math.sin(a), p[1] + 1.5, p[2] - 3 * math.cos(a)], "yaw": yaw,
            "pitch": math.degrees(math.atan2(1.1, 3.0)), "fov": args.fov}
    result = {"player_position": p, "requested": want, "failed": []}
    for shown in (True, False):
        name = "player_shown" if shown else "player_hidden"
        path = (outdir / f"{name}.png").resolve()
        game.send("camera", position=want["position"], yaw=want["yaw"], pitch=want["pitch"], fov=want["fov"],
                  settle_frames=30, settle_seconds=1.0, show_player=shown, keep_post=args.keep_post)
        shot = game.send("shot", path=str(path))
        result[name] = pose(shot)
        result["failed"] += [f"{name}: {b}" for b in checks(want, shot, player_hidden=not shown, keep_post=args.keep_post)]
    try:
        from PIL import Image, ImageChops, ImageStat
        a_img = Image.open(outdir / "player_shown.png").convert("RGB")
        b_img = Image.open(outdir / "player_hidden.png").convert("RGB")
        diff = ImageChops.difference(a_img, b_img).convert("L")
        changed = sum(diff.point(lambda v: 255 if v > 24 else 0).histogram()[255:])
        result["changed_pixels"] = changed
        if changed < 500:
            result["failed"].append(f"hiding the player changed only {changed} pixels (is the bird in the control frame?)")
    except ImportError:
        result["changed_pixels"] = None
    print(f"player check: {result.get('changed_pixels')} pixels change when the bird is hidden"
          f" {'FAILED ' + '; '.join(result['failed']) if result['failed'] else 'ok'}", flush=True)
    return result


def shoot_level(game, key, level, out, args):
    """Load one level and shoot its views into out; returns (record, number of failed shots/checks)."""
    out.mkdir(parents=True, exist_ok=True)
    record = {"level": key, "scene": level["name"], "mode": args.mode, "back": args.back, "eye": args.eye,
              "fov": args.fov, "pitch": args.pitch, "yaw_offset": args.yaw_offset, "post_effects_kept": args.keep_post,
              "npcs_kept": args.keep_npcs,
              "settle_s": args.settle, "screen": [WIDTH, HEIGHT], "lab": str(LAB),
              "started": time.strftime("%Y-%m-%dT%H:%M:%S"), "shots": []}
    failed = 0
    try:
        r = game.send("load", timeout=150, scene=level["name"], settle_frames=30, settle_seconds=args.load_settle)
        print(f"{key}: loaded {r['scene']}; player at {r.get('player_position')}", flush=True)
        for i, view in enumerate(level["views"]):
            x, y, z = view["position"]
            a = math.radians(view["yaw_deg"])
            want = {"position": [x - args.back * math.sin(a), y + args.eye, z - args.back * math.cos(a)],
                    "yaw": view["yaw_deg"] + args.yaw_offset, "pitch": args.pitch, "fov": args.fov}
            name = re.sub(r"[^A-Za-z0-9]+", "_", view["name"]).strip("_")
            path = (out / f"{i}_{name}.png").resolve()
            game.send("camera", position=want["position"], yaw=want["yaw"], pitch=want["pitch"], fov=want["fov"],
                      settle_frames=30, settle_seconds=args.settle, keep_post=args.keep_post,
                      hide_npcs=not args.keep_npcs)
            shot = game.send("shot", path=str(path))
            after = game.send("readback")
            bad = checks(want, shot, keep_post=args.keep_post) + [
                f"after the shot: {b}" for b in checks(want, after, keep_post=args.keep_post)]
            failed += bool(bad)
            record["shots"].append({"index": i, "name": view["name"], "file": path.name,
                                    "view": {"position": view["position"], "yaw_deg": view["yaw_deg"]},
                                    "requested": want, "shot": pose(shot), "after": pose(after), "failed": bad})
            print(f"{key} {path.name}: cam {[round(v, 3) for v in shot['cam_position']]} ground "
                  f"{shot.get('cam_ground', -1):.2f} m, NPC renderers hidden {shot.get('hidden_npc_renderers')}"
                  f" {'FAILED ' + '; '.join(bad) if bad else 'ok'}", flush=True)
        last = record["shots"][-1]["shot"] if record["shots"] else {}
        record["npcs_hidden"] = {"birbs": last.get("npc_birbs"), "other_skinned": last.get("npc_skinned_other"),
                                 "renderers": last.get("hidden_npc_renderers")}
        record["player_check"] = player_check(game, out / "checks", level["views"][0]["yaw_deg"], args)
        failed += bool(record["player_check"]["failed"])
        game.send("release")
    except RuntimeError as e:
        record["error"] = str(e)
        print(f"{key}: ERROR {e}", flush=True)
        failed += 1
    (out / "poses.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"{key}: {len(record['shots'])} shots, {failed} failed -> {out / 'poses.json'}", flush=True)
    return record, failed


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("level", help="a data/skatebird-views.json key, or several joined by commas (one launch)")
    ap.add_argument("outdir", help="the level's output folder; with several levels, their parent (OUTDIR/<level>)")
    ap.add_argument("--chase", action="store_true", help="skate.'s third-person camera preset instead of the eye")
    ap.add_argument("--back", type=float, help="camera distance behind the view point along its yaw, metres")
    ap.add_argument("--eye", type=float, help="camera height above the view point, metres")
    ap.add_argument("--fov", type=float, help="vertical field of view, degrees")
    ap.add_argument("--pitch", type=float, help="Unity pitch, degrees (positive looks down)")
    ap.add_argument("--yaw-offset", type=float, help="degrees added to each view's yaw")
    ap.add_argument("--settle", type=float, default=2.0, help="seconds each camera pose is held before the shot")
    ap.add_argument("--load-settle", type=float, default=8.0, help="seconds after the scene is active")
    ap.add_argument("--keep-post", action="store_true",
                    help="keep the game's depth of field, lens distortion, chromatic aberration, vignette, motion blur")
    ap.add_argument("--keep-npcs", action="store_true",
                    help="keep the NPC birds and other skinned meshes (hidden by default: skate. gets no skinned meshes)")
    ap.add_argument("--keep", action="store_true", help="leave the game running")
    args = ap.parse_args()
    args.mode = "chase" if args.chase else "eye"
    for k, v in MODES[args.mode].items():
        if getattr(args, k) is None:
            setattr(args, k, v)

    views = json.loads((ROOT / "data/skatebird-views.json").read_text(encoding="utf-8"))
    keys = [k for k in args.level.split(",") if k]
    unknown = [k for k in keys if k not in views]
    if unknown or not keys:
        sys.exit(f"not in data/skatebird-views.json: {unknown or args.level}")
    if not (LAB / "BepInEx/plugins/SkateBIRDTeleport.dll").exists():
        sys.exit(f"no lab plugin under {LAB}; run tools/skatebird_lab.sh")
    root = Path(args.outdir)
    root.mkdir(parents=True, exist_ok=True)

    failed = {}
    game = Game()
    try:
        game.start()
        game.send("screen", timeout=120, width=WIDTH, height=HEIGHT)
        for key in keys:
            if game.proc.poll() is not None:
                failed[key] = "game exited"
                continue
            _, failed[key] = shoot_level(game, key, views[key], root / key if len(keys) > 1 else root, args)
    except RuntimeError as e:
        print("ERROR", e, flush=True)
        failed["launch"] = str(e)
    finally:
        game.stop(args.keep)
        if LOG.exists():
            shutil.copyfile(LOG, root / "plugin.log")
    print("summary: " + ", ".join(f"{k} {v if isinstance(v, str) else (str(v) + ' failed')}" for k, v in failed.items()))
    sys.exit(1 if any(failed.values()) else 0)


if __name__ == "__main__":
    main()
