"""The fixed views each level is compared at, the same in SkateBIRD and in skate.

  python -I tools/views.py make                 -> data/skatebird-views.json (spawn + 3 of the game's travel anchors)
  python -I tools/views.py skate-commands LEVEL -> RESKATE_STARTUP_COMMANDS for one run of that level

A view is a place on the ground in Unity coordinates (metres) and a yaw. skate's own coordinates are Unity's (the map
writes scene (x, y, -z) and skate's z is the scene's -north), so `trainer tp` takes the Unity position as it is.
skate gives no console way to turn the skater: a teleport keeps the spawn's heading, so in skate every view of a level
looks along the spawn's yaw, in first person (`firstperson 1`: the camera at the skater's eyes), and SkateBIRD's camera
is put at the same eye point and yaw.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEVELS = ["level1", "level2", "level3", "level4", "level5", "level7", "level12", "level13"]
EXTRA = 3  # travel anchors per level besides the spawn
LIFT = 0.3  # metres above the anchor a teleport lands (skate drops the skater onto the ground)


def make():
    scan = json.loads((ROOT / "data/skatebird-levels.json").read_text(encoding="utf-8"))["levels"]
    out = {}
    for level in LEVELS:
        l = scan[level]
        spawn = l.get("spawn") or l["player_birb_instance"]
        views = [{"name": "spawn", "position": spawn["position"], "yaw_deg": spawn["yaw_deg"]}]
        others = [p for p in l["travel_points"] if p["position"] != spawn["position"]]
        step = max(1, len(others) // EXTRA)
        for p in others[::step][:EXTRA]:
            views.append({"name": p["name"], "position": p["position"], "yaw_deg": spawn["yaw_deg"]})
        out[level] = {"name": l["name"], "views": views}
    path = ROOT / "data/skatebird-views.json"
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print(f"{path}: {sum(len(v['views']) for v in out.values())} views over {len(out)} levels")


def skate_commands(level):
    views = json.loads((ROOT / "data/skatebird-views.json").read_text(encoding="utf-8"))[level]["views"]
    asset = f"levels/game/dingolevel_reskate_skatebird_{level}/dingolevel_reskate_skatebird_{level}"
    # settle at the spawn, one third-person shot, then first person: one shot per view (each `trainer where` is a shot)
    c = ["wait 100", f"load {asset}", "wait 45", "trainer option hud 0", "hideui 1", "trainer where", "wait 6",
         "firstperson 1", "wait 6"]
    for v in views:
        x, y, z = v["position"]
        c += [f"trainer tp {x} {y + LIFT:.3f} {z}", "wait 10", "trainer where", "wait 6"]
    print(";".join(c))


if __name__ == "__main__":
    if sys.argv[1:] == ["make"]:
        make()
    elif len(sys.argv) == 3 and sys.argv[1] == "skate-commands":
        skate_commands(sys.argv[2])
    else:
        sys.exit(__doc__)
