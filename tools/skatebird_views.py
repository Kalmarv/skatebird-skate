"""SkateBIRD reference shots at exact camera poses, one per view of data/skatebird-views.json.

  python -I tools/skatebird_views.py LEVEL OUTDIR [--eye H] [--fov F] [--pitch P] [--settle S] [--keep]

(Run it through tools/skatebird_views.sh inside `kit/run game`: it opens a game window.)

Launches the lab copy (tools/skatebird_lab.sh makes it) windowed at 1024x576, loads the level's scene through the lab
plugin, and for each view locks Camera.main at (view position + H metres up), yaw = the view's yaw_deg (Unity: degrees
from +Z towards +X), pitch P (Unity Euler x, positive looks down), vertical FOV F, with the HUD/menus and the player's
renderers hidden. After the settle it saves OUTDIR/<view index>_<name>.png (the rendered frame, read back in game) and
writes OUTDIR/poses.json: per shot the requested pose, the plugin's read-back of the frame that was saved, a read-back
after the save, and the checks. OUTDIR/checks/player_{shown,hidden}.png are the control for the bird hiding (one pose
looking at the bird, with and without it). OUTDIR/plugin.log is the run's BepInEx log. Exit 1 if any check failed.

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
        os.replace(tmp, COMMAND)
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
            self.proc.wait(timeout=10)


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
    return bad


def pose(r):
    keys = ("frame", "time", "cam_name", "cam_position", "cam_euler", "cam_forward", "cam_bearing", "cam_fov",
            "cam_fov_projection", "cam_near", "cam_far", "screen_width", "screen_height", "player_position",
            "player_yaw", "gameplay_obstructed", "hidden_canvases", "hidden_renderers", "player_roots",
            "kept_canvases", "other_cameras", "post_effects_off", "post_effects_on")
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


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("level")
    ap.add_argument("outdir")
    ap.add_argument("--eye", type=float, default=1.6, help="camera height above the view position, metres")
    ap.add_argument("--fov", type=float, default=60.0, help="vertical field of view, degrees")
    ap.add_argument("--pitch", type=float, default=0.0, help="Unity pitch, degrees (positive looks down)")
    ap.add_argument("--settle", type=float, default=2.0, help="seconds each camera pose is held before the shot")
    ap.add_argument("--load-settle", type=float, default=8.0, help="seconds after the scene is active")
    ap.add_argument("--keep-post", action="store_true",
                    help="keep the game's depth of field, lens distortion, chromatic aberration, vignette, motion blur")
    ap.add_argument("--keep", action="store_true", help="leave the game running")
    args = ap.parse_args()

    level = json.loads((ROOT / "data/skatebird-views.json").read_text(encoding="utf-8")).get(args.level)
    if level is None:
        sys.exit(f"{args.level} is not in data/skatebird-views.json")
    if not (LAB / "BepInEx/plugins/SkateBIRDTeleport.dll").exists():
        sys.exit(f"no lab plugin under {LAB}; run tools/skatebird_lab.sh")
    out = Path(args.outdir)
    out.mkdir(parents=True, exist_ok=True)

    record = {"level": args.level, "scene": level["name"], "eye": args.eye, "fov": args.fov, "pitch": args.pitch,
              "post_effects_kept": args.keep_post,
              "settle_s": args.settle, "screen": [WIDTH, HEIGHT], "lab": str(LAB), "started": time.strftime("%Y-%m-%dT%H:%M:%S"),
              "shots": []}
    failed = 0
    game = Game()
    try:
        game.start()
        game.send("screen", timeout=120, width=WIDTH, height=HEIGHT)
        r = game.send("load", timeout=150, scene=level["name"], settle_frames=30, settle_seconds=args.load_settle)
        print(f"loaded {r['scene']}; player at {r.get('player_position')}", flush=True)
        for i, view in enumerate(level["views"]):
            x, y, z = view["position"]
            want = {"position": [x, y + args.eye, z], "yaw": view["yaw_deg"], "pitch": args.pitch, "fov": args.fov}
            name = re.sub(r"[^A-Za-z0-9]+", "_", view["name"]).strip("_")
            path = (out / f"{i}_{name}.png").resolve()
            game.send("camera", position=want["position"], yaw=want["yaw"], pitch=want["pitch"], fov=want["fov"],
                      settle_frames=30, settle_seconds=args.settle, keep_post=args.keep_post)
            shot = game.send("shot", path=str(path))
            after = game.send("readback")
            bad = checks(want, shot, keep_post=args.keep_post) + [
                f"after the shot: {b}" for b in checks(want, after, keep_post=args.keep_post)]
            failed += bool(bad)
            record["shots"].append({"index": i, "name": view["name"], "file": path.name,
                                    "view": {"position": view["position"], "yaw_deg": view["yaw_deg"]},
                                    "requested": want, "shot": pose(shot), "after": pose(after), "failed": bad})
            print(f"{path.name}: cam {shot['cam_position']} euler {shot['cam_euler']} fov {shot['cam_fov']:.2f}"
                  f" {'FAILED ' + '; '.join(bad) if bad else 'ok'}", flush=True)
        record["player_check"] = player_check(game, out / "checks", level["views"][0]["yaw_deg"], args)
        failed += bool(record["player_check"]["failed"])
        game.send("release")
    except RuntimeError as e:
        record["error"] = str(e)
        print("ERROR", e, flush=True)
        failed += 1
    finally:
        game.stop(args.keep)
        (out / "poses.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        if LOG.exists():
            shutil.copyfile(LOG, out / "plugin.log")
    print(f"{len(record['shots'])} shots, {failed} failed -> {out / 'poses.json'}")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
