"""The fixed views each level is compared at, the same in SkateBIRD and in skate.

  python -I tools/views.py make                 -> data/skatebird-views.json (spawn + 3 of the game's travel anchors)
  python -I tools/views.py skate-commands LEVEL -> RESKATE_STARTUP_COMMANDS for one run of that level

A view is a place on the ground in Unity coordinates (metres) and the Unity yaw the camera looks along. skate's own
coordinates are the scene's: Unity (x, y, z) is skate (x, y, -z) (`trainer where` at level2's spawn, Unity
(-18.7, 1.189, 84.8), reads (-18.70, 1.95, -84.80)). skate gives no console way to turn the skater: the spawn view is
shot where the level starts (facing the spawn's yaw), and `trainer tp` leaves the skater at heading 0 (skate +z,
Unity -z: Unity yaw 180), so every other view looks along Unity yaw 180. All in first person (`firstperson 1`: the
camera at the skater's eyes); SkateBIRD's camera is put at the same eye point and yaw.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEVELS = ["level1", "level2", "level3", "level4", "level5", "level7", "level12", "level13"]
EXTRA = 3  # travel anchors per level besides the spawn
LIFT = 0.3  # metres above the anchor a teleport lands (skate drops the skater onto the ground)
TELEPORT_YAW = 180.0  # the Unity yaw a skater faces after `trainer tp` (skate heading 0)


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
            views.append({"name": p["name"], "position": p["position"], "yaw_deg": TELEPORT_YAW})
        out[level] = {"name": l["name"], "views": views}
    path = ROOT / "data/skatebird-views.json"
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print(f"{path}: {sum(len(v['views']) for v in out.values())} views over {len(out)} levels")


def skate_commands(level):
    views = json.loads((ROOT / "data/skatebird-views.json").read_text(encoding="utf-8"))[level]["views"]
    asset = f"levels/game/dingolevel_reskate_skatebird_{level}/dingolevel_reskate_skatebird_{level}"
    # settle at the spawn, one third-person shot, then first person: the spawn view where the level starts, then one
    # teleport per other view (each `trainer where` is a shot)
    c = ["wait 100", f"load {asset}", "wait 45", "trainer option hud 0", "hideui 1", "trainer where", "wait 6",
         "firstperson 1", "wait 6", "trainer where", "wait 6"]
    for v in views[1:]:
        x, y, z = v["position"]
        c += [f"trainer tp {x} {y + LIFT:.3f} {-z}", "wait 10", "trainer where", "wait 6"]
    print(";".join(c))


if __name__ == "__main__":
    if sys.argv[1:] == ["make"]:
        make()
    elif len(sys.argv) == 3 and sys.argv[1] == "skate-commands":
        skate_commands(sys.argv[2])
    else:
        sys.exit(__doc__)
