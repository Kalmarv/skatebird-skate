#!/usr/bin/env python3
"""Capture SkateBIRD reference shots for every level.

    # spawn + first travel point >= 20 m
    orca job run --lock screen -- python tools/skatebird_reference_shots.py

    # spawn only, with the W-nudge camera settle (rewrites skatebird_<name>_spawn.png)
    orca job run --lock screen -- python tools/skatebird_reference_shots.py --spawn-only --nudge

Reads data/skatebird-levels.json, drives tools/skatebird_teleport.py, and writes
out/skatebird_<name>_spawn.png / _tp1.png plus a JSON summary and log.
level7 (flashback) has no GameLevels/console name; it is loaded by scene name through
the BepInEx plugin.
"""
import argparse
import json
import math
import os
import subprocess
import sys
import time

ROOT = r"C:\Users\kalmarv\ds-modding"
RUNNER = os.path.join(ROOT, "tools", "skatebird_teleport.py")
DATA = os.path.join(ROOT, "data", "skatebird-levels.json")
OUT = os.path.join(ROOT, "out")

LEVELS = [
    ("level1", "bedroom_1", "bedroom_1", None),
    ("level2", "rooftop", "rooftop", None),
    ("level3", "office", "office", None),
    ("level4", "server_room", "server_room", None),
    ("level5", "bedroom_2", "bedroom_2", None),
    ("level7", "flashback", "flashback", "flashback"),
    ("level12", "playground", "playground", None),
    ("level13", "skate_heaven", "skate_heaven", None),
]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--spawn-only", action="store_true")
    ap.add_argument("--nudge", action="store_true")
    ap.add_argument("--camera-lock", action="store_true")
    args = ap.parse_args()

    suffix = "spawn_refs" if args.spawn_only else "reference_shots"
    log_path = os.path.join(OUT, "skatebird_%s.log" % suffix)
    summary_path = os.path.join(OUT, "skatebird_%s.json" % suffix)

    levels = json.load(open(DATA, encoding="utf-8"))["levels"]
    summary = {}
    failures = []
    os.makedirs(OUT, exist_ok=True)
    with open(log_path, "a", encoding="utf-8") as log:
        for level, console, stem, scene in LEVELS:
            data = levels[level]
            spawn = data["spawn"]
            sp = spawn["position"]
            sy = spawn.get("yaw_deg", 0.0)
            tp = None
            if not args.spawn_only:
                tp = next((t for t in data["travel_points"]
                           if math.dist(sp, t["position"]) >= 20.0), None)
                if tp is None:
                    failures.append((level, "no travel point >= 20 m"))
                    continue
            point1 = "%r,%r,%r,%r" % (sp[0], sp[1], sp[2], sy)
            points = [point1]
            if tp is not None:
                points.append("%r,%r,%r,%r" % (tp["position"][0], tp["position"][1],
                                               tp["position"][2], tp.get("yaw_deg", 0.0)))
            prefix = os.path.join(OUT, "skatebird_%s_spawn" % stem)
            cmd = [sys.executable, RUNNER, console, prefix]
            if scene:
                cmd += ["--scene", scene]
            if args.nudge:
                cmd += ["--nudge"]
            if args.camera_lock:
                cmd += ["--camera-lock"]
            cmd += ["--"] + points
            header = "\n===== %s (%s) %s -> %s\nspawn %s\n" % (
                level, stem, console, scene or "console", point1)
            if tp is not None:
                header += "tp1   %s (%s, %.2f m)\n" % (
                    points[1], tp["name"], math.dist(sp, tp["position"]))
            print(header, end="")
            log.write(header)
            log.flush()
            t0 = time.time()
            p = subprocess.run(cmd, capture_output=True, text=True,
                               encoding="utf-8", errors="replace")
            output = (p.stdout or "") + (p.stderr or "")
            print(output, end="")
            log.write(output)
            log.write("\nrc=%d seconds=%.1f\n" % (p.returncode, time.time() - t0))
            log.flush()

            spawn_png = os.path.join(OUT, "skatebird_%s_spawn.png" % stem)
            tp_png = os.path.join(OUT, "skatebird_%s_tp1.png" % stem)
            readback = [line.strip() for line in output.splitlines()
                        if any(k in line for k in ("TELEPORT", "PLAYER ", "CAMERA", "BEARING"))]
            camera_checks = [line.strip() for line in output.splitlines()
                             if "CAMERA CHECK" in line or "WARNING" in line]
            record = {
                "level": level,
                "console": console,
                "scene": scene,
                "spawn": {"position": sp, "yaw_deg": sy},
                "readback": readback,
                "camera_checks": camera_checks,
                "files": [],
                "rc": p.returncode,
            }
            if tp is not None:
                record["tp1"] = {"name": tp["name"], "position": tp["position"],
                                 "yaw_deg": tp.get("yaw_deg", 0.0),
                                 "distance_m": round(math.dist(sp, tp["position"]), 2)}
            if p.returncode == 0:
                try:
                    os.replace(prefix + "_1.png", spawn_png)
                    record["files"] = [spawn_png]
                    if tp is not None:
                        os.replace(prefix + "_2.png", tp_png)
                        record["files"].append(tp_png)
                except FileNotFoundError as e:
                    failures.append((level, "missing captured file: %s" % e))
            else:
                failures.append((level, "runner rc=%d" % p.returncode))
            summary[level] = record

    with open(summary_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)
    print("\nwrote", summary_path)
    if failures:
        print("FAILURES:")
        for level, why in failures:
            print(" ", level, why)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
