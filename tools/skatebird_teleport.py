#!/usr/bin/env python3
"""Launch the SkateBIRD lab instance, load a level, teleport, and capture.

    python tools/skatebird_teleport.py <level> <out-prefix> <x,y,z,yaw> [<x,y,z,yaw> ...]
        [--dismiss] [--keep] [--settle S] [--log PATH]

Each point is written to data/skatebird-teleport.json, the BepInEx plugin applies it,
and the window is captured to <out-prefix>_<n>.png with ffmpeg gfxcapture.
The lab folder (skatebird-lab) has SkateBIRD.exe and BepInEx; SkateBIRD_Data and
MonoBleedingEdge are junctions to the Steam install, so the install is only read.
"""
import argparse
import os
import re
import subprocess
import sys
import time

ROOT = r"C:\Users\kalmarv\ds-modding"
LAB = os.path.join(ROOT, "skatebird-lab")
EXE = os.path.join(LAB, "SkateBIRD.exe")
PROC = "SkateBIRD"
TELEPORT_JSON = os.path.join(ROOT, "data", "skatebird-teleport.json")
SCENE_JSON = os.path.join(ROOT, "data", "skatebird-load-scene.json")
LOG = os.path.join(LAB, "BepInEx", "LogOutput.log")
FFMPEG = r"C:/Users/kalmarv/AppData/Local/universal-modder/ffmpeg/bin/ffmpeg.exe"
UM = "um"


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, encoding="utf-8", errors="replace", **kw)


def um_ps():
    """[(pid, name, title)] from `um win ps`; tolerates um's own crashes."""
    p = run([UM, "win", "ps"])
    out = []
    for line in (p.stdout or "").splitlines():
        parts = line.split(None, 2)
        if len(parts) >= 2 and parts[0].isdigit():
            out.append((int(parts[0]), parts[1], parts[2] if len(parts) > 2 else ""))
    return out


def write_json(x, y, z, yaw, cam_yaw=None, cam_pitch=12.0):
    os.makedirs(os.path.dirname(TELEPORT_JSON), exist_ok=True)
    body = '"x": %r, "y": %r, "z": %r, "yaw": %r' % (x, y, z, yaw)
    if cam_yaw is not None:
        body += ', "cam_yaw": %r, "cam_pitch": %r' % (cam_yaw, cam_pitch)
    body += ', "nonce": %d' % time.time_ns()
    tmp = TELEPORT_JSON + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write("{" + body + "}\n")
    os.replace(tmp, TELEPORT_JSON)


def write_scene(scene):
    os.makedirs(os.path.dirname(SCENE_JSON), exist_ok=True)
    tmp = SCENE_JSON + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write('{"scene": "%s", "nonce": %d}\n' % (scene, time.time_ns()))
    os.replace(tmp, SCENE_JSON)


def read_log():
    try:
        with open(LOG, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    except FileNotFoundError:
        return ""


def wait_for_teleport(offset, point, timeout=20.0, require_lock=False):
    """Wait for a TELEPORT OK after offset whose read-back PLAYER POS matches point."""
    end = time.time() + timeout
    while time.time() < end:
        text = read_log()
        if len(text) < offset:
            offset = 0  # a run that truncates the log
        new = text[offset:]
        if require_lock and "CAMERA LOCK BEARING" not in new:
            time.sleep(0.2)
            continue
        if "TELEPORT OK" in new and "CAMERA" in new:
            poses = re.findall(
                r"PLAYER POS \(([-0-9.]+),([-0-9.]+),([-0-9.]+)\) ROT", new)
            if poses:
                p = [float(v) for v in poses[-1]]
                if max(abs(p[j] - point[j]) for j in range(3)) < 0.01:
                    return new.rsplit("TELEPORT REQUEST", 1)[-1]
        time.sleep(0.2)
    return None


def camera_bearing(block):
    """Last BEARING in the block (the plugin logs the camera for 1.5 s after a teleport)."""
    found = re.findall(r"BEARING (-?[0-9]+(?:\.[0-9]+)?)", block)
    return float(found[-1]) if found else None


def last_camera_line(block):
    lines = [l for l in block.splitlines() if "CAMERA POS" in l]
    return lines[-1].strip() if lines else None


def wait_for_line(needle, offset, timeout=40.0):
    end = time.time() + timeout
    while time.time() < end:
        text = read_log()
        if len(text) < offset:
            offset = 0
        if needle in text[offset:]:
            return text[offset:]
        time.sleep(0.2)
    return None


def capture(path):
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    ff = FFMPEG
    graph = ("gfxcapture=window_exe=SkateBIRD.exe:capture_cursor=0:max_framerate=60,"
             "hwdownload,format=bgra")
    p = run([ff, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", graph,
             "-frames:v", "1", "-y", path])
    if p.returncode != 0 or not os.path.exists(path):
        raise RuntimeError("capture failed: " + (p.stderr or "").strip())
    return path


def parse_point(text):
    vals = [float(v) for v in text.split(",")]
    if len(vals) != 4:
        raise argparse.ArgumentTypeError("point must be x,y,z,yaw")
    return vals


def main():
    global LOG
    ap = argparse.ArgumentParser()
    ap.add_argument("level")
    ap.add_argument("out_prefix")
    ap.add_argument("points", nargs="+", type=parse_point)
    ap.add_argument("--settle", type=float, default=12.0, help="seconds after LoadLevel")
    ap.add_argument("--scene", default=None,
                    help="load this scene through the plugin (e.g. flashback) instead of the console")
    ap.add_argument("--dismiss", action="store_true",
                    help="send Esc, Esc, Enter for the fresh-save popup chain")
    ap.add_argument("--nudge", action="store_true",
                    help="hold W, wait, re-teleport to the exact pose, wait before capture")
    ap.add_argument("--nudge-hold", type=float, default=0.8)
    ap.add_argument("--nudge-wait", type=float, default=2.0)
    ap.add_argument("--nudge-settle", type=float, default=1.0)
    ap.add_argument("--camera-lock", action="store_true",
                    help="write cam_yaw=pose yaw and cam_pitch, then wait for the camera-lock check")
    ap.add_argument("--cam-pitch", type=float, default=12.0)
    ap.add_argument("--keep", action="store_true")
    ap.add_argument("--log", default=LOG)
    args = ap.parse_args()

    LOG = args.log

    if any(name == PROC for _, name, _ in um_ps()):
        sys.exit("SkateBIRD is already running; close it first")

    print("launching", EXE)
    for stale in (TELEPORT_JSON, SCENE_JSON):
        try:
            os.remove(stale)
        except FileNotFoundError:
            pass
    proc = subprocess.Popen(
        [EXE, "-screen-fullscreen", "0", "-screen-width", "1920", "-screen-height", "1080",
         "-devconsole"],
        cwd=LAB,
    )
    pid = proc.pid
    try:
        deadline = time.time() + 90
        while time.time() < deadline:
            if any(p == pid and n == PROC for p, n, _ in um_ps()):
                break
            if proc.poll() is not None:
                raise RuntimeError("game exited early, code %s" % proc.returncode)
            time.sleep(0.5)
        else:
            raise RuntimeError("window did not appear")
        print("pid", pid)

        if args.scene:
            before = len(read_log())
            write_scene(args.scene)
            if wait_for_line("SCENE LOADED " + args.scene, before) is None:
                raise RuntimeError("scene '%s' did not load; log tail:\n%s"
                                   % (args.scene, read_log()[-2000:]))
            print("scene loaded:", args.scene)
            time.sleep(4.0)
        else:
            # title -> console -> level
            drive = run([UM, "win", "drive", "--proc", PROC, "focus", "sleep 10000",
                         "key 0x70", "sleep 1200", "type MissionManager.LoadLevel " + args.level,
                         "key 0x0D"])
            if drive.returncode != 0:
                print("drive failed:", (drive.stderr or drive.stdout).strip())
            print("load issued; settling", args.settle, "s")
            time.sleep(args.settle)

        if args.dismiss:
            run([UM, "win", "drive", "--proc", PROC, "focus",
                 "key 0x1B", "sleep 2000", "key 0x1B", "sleep 2500", "key 0x0D", "sleep 4000"])

        for i, (x, y, z, yaw) in enumerate(args.points, 1):
            before = len(read_log())
            cam_yaw = yaw if args.camera_lock else None
            write_json(x, y, z, yaw, cam_yaw, args.cam_pitch)
            new = wait_for_teleport(before, (x, y, z), require_lock=args.camera_lock)
            if new is None:
                raise RuntimeError("no matching PLAYER POS in BepInEx log after teleport; log tail:\n"
                                   + read_log()[-2000:])
            for line in new.splitlines():
                if ("TELEPORT" in line or "PLAYER " in line
                        or "CAMERA" in line or "BEARING" in line):
                    print(line)
            if args.camera_lock:
                m = re.search(
                    r"CAMERA LOCK BEARING ([-0-9.]+) TARGET ([-0-9.]+) DIFF ([-0-9.]+) (OK|FAIL)",
                    new)
                if m is None or m.group(4) != "OK":
                    raise RuntimeError("camera lock check failed: %s"
                                       % (m.group(0) if m else "no CAMERA LOCK BEARING line"))
                print("CAMERA LOCK VERIFIED bearing=%s target=%s diff=%s"
                      % (m.group(1), m.group(2), m.group(3)))

            if args.nudge:
                # Move forward so the follow camera swings behind the heading, then put the
                # player back on the exact pose and let the camera settle without input.
                run([UM, "win", "drive", "--proc", PROC, "focus",
                     "hold 0x57 %d" % int(args.nudge_hold * 1000)])
                time.sleep(args.nudge_wait)
                before2 = len(read_log())
                write_json(x, y, z, yaw, cam_yaw, args.cam_pitch)
                again = wait_for_teleport(before2, (x, y, z),
                                          require_lock=args.camera_lock)
                if again is None:
                    raise RuntimeError("re-teleport after nudge produced no read-back")
                for line in again.splitlines():
                    if ("TELEPORT" in line or "PLAYER " in line
                            or "CAMERA" in line or "BEARING" in line):
                        print(line)
            time.sleep(args.nudge_settle)  # let the camera settle on the pose
            base = before2 if args.nudge else before
            settled = read_log()
            if len(settled) < base:
                base = 0
            settled = settled[base:]
            line = last_camera_line(settled)
            bearing = camera_bearing(settled)
            if line:
                print(line)
            if bearing is None:
                print("WARNING: no settled camera bearing")
            else:
                diff = ((bearing - yaw + 180.0) % 360.0) - 180.0
                print("CAMERA CHECK pose_yaw=%.2f camera_bearing=%.2f diff=%.2f"
                      % (yaw, bearing, diff))
                if abs(diff) > 10.0:
                    print("WARNING: camera forward differs from pose yaw by %.2f deg" % diff)
            out = "%s_%d.png" % (args.out_prefix, i)
            capture(out)
            print("wrote", out)
    finally:
        if not args.keep:
            run([UM, "win", "kill", str(pid)])
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
            print("closed", pid)


if __name__ == "__main__":
    main()
