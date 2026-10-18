"""Render SkateBIRD level data: data/skatebird-levels.json + docs/levels.md."""
import json
import os

ROOT = r"C:\Users\kalmarv\ds-modding"
SCAN = os.path.join(ROOT, "out", "skatebird-scan.json")
OUT_JSON = os.path.join(ROOT, "data", "skatebird-levels.json")
OUT_MD = os.path.join(ROOT, "docs", "levels.md")

GAME_ROOT = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD"
DATA_DIR = os.path.join(GAME_ROOT, "SkateBIRD_Data")
UNITY = "2019.4.20f1"


def f3(p):
    return "(" + ", ".join(f"{v:.3f}" for v in p) + ")"


def f2(p):
    return "(" + ", ".join(f"{v:.2f}" for v in p) + ")"


def main():
    scan = json.load(open(SCAN, encoding="utf-8"))
    levels = scan["levels"]

    out = {
        "game": {
            "title": "SkateBIRD",
            "install_root": GAME_ROOT,
            "data_dir": DATA_DIR,
            "unity_version": UNITY,
            "inspected": "2026-10-18",
            "units": "metres, Unity left-handed Y-up; yaw_deg = atan2(forward.x, forward.z)",
            "read_only": True,
            "method": "UnityPy 1.25.4 over the loose levelN + sharedassetsN serialized files; "
                      "spawn semantics confirmed against Assembly-CSharp.dll",
        },
        "levels": {},
    }

    md = [
        "# SkateBIRD level data (spawn, bounds, travel points, lighting)",
        "",
        "Read-only inspection of `C:\\Program Files (x86)\\Steam\\steamapps\\common\\SkateBIRD\\SkateBIRD_Data`",
        "(Unity 2019.4.20f1, Mono, built-in render pipeline). Scene files are the loose `levelN` serialized",
        "files; meshes/materials live in `sharedassetsN`. Nothing in the install was modified.",
        "",
        "## Method",
        "",
        "- **spawn**: `Skatebirb.Gameplay.ResetPosition.ResetToStart` (called from `ControlBoard` on the first",
        "  play frame) teleports to the active `ResetPosition` whose serialized `startingPosition` flag is true.",
        "  That transform is the spawn. The flag is read from the raw MonoBehaviour tail (the player build has",
        "  no script type trees). When no flag is set, `ResetToStart` does nothing and the serialized `Player Birb`",
        "  placement is the effective spawn (both cases below).",
        "- **bounds**: every enabled, non-trigger, active-in-hierarchy collider in the scene. `MeshCollider`",
        "  bounds are the mesh's local AABB (`m_LocalAABB`) transformed by the collider's world matrix (8 corners).",
        "  Any collider whose **world-space** AABB exceeds 100 m on any axis is excluded (the same filter is",
        "  applied to box/sphere/capsule colliders because oversized scene-boundary volumes would otherwise",
        "  dominate). Excluded colliders are listed in the JSON; the unfiltered min/max is also recorded.",
        "- **travel points**: curated from the scene's reset anchors (the game's own teleport destinations)",
        "  and `MissionGiver` objects, farthest-point sampled so they spread across the level. All positions are",
        "  world-space metres in Unity's left-handed Y-up frame; yaw is `atan2(forward.x, forward.z)` in degrees.",
        "- **lighting**: `Light` objects (`m_Type == 1` directional) and the scene `RenderSettings` object.",
        "  Euler angles use Unity's ZXY `Quaternion.Euler` convention.",
        "",
        "## Summary",
        "",
        "| level | scene | spawn (x, y, z) | yaw | bounds min | bounds max | travel points |",
        "|---|---|---|---|---|---|---|",
    ]

    for level, d in levels.items():
        sp = d["spawn"]
        b = d["bounds"]
        lighting = dict(d["lighting"])
        lighting["directional_lights"] = d.get("directional_lights", [])
        lighting["lights_total"] = len(d.get("lights", []))
        out["levels"][level] = {
            "scene": level,
            "name": d["name"],
            "spawn": sp,
            "player_birb_instance": d["player_birb_instance"],
            "bounds": b,
            "travel_points": d["travel_points"],
            "reset_positions": d["reset_positions"],
            "lighting": lighting,
            "notes": d["notes"],
        }
        md.append(
            f"| {level} | {d['name']} | {f2(sp['position'])} | {sp['yaw_deg']:.2f}° | "
            f"{f2(b['min'])} | {f2(b['max'])} | {len(d['travel_points'])} |"
        )

    md += ["", "## Levels", ""]

    for level, d in levels.items():
        sp = d["spawn"]
        pb = d["player_birb_instance"]
        b = d["bounds"]
        li = d["lighting"]
        rs = li.get("render_settings", {})
        sun = li.get("directional")
        md += [
            f"### {level} — {d['name']}",
            "",
            f"Scene file: `{level}`.",
            "",
            "**Spawn**",
            "",
            f"- source: `{sp['source']}`",
            f"- GameObject: `{sp['game_object']}` (`{sp['path']}`)",
            f"- position: `{f3(sp['position'])}` m; yaw `{sp['yaw_deg']:.2f}°`; "
            f"quaternion `({', '.join(f'{q:.6f}' for q in sp['quaternion'])})` (x, y, z, w)",
            f"- active in hierarchy: `{sp['active_in_hierarchy']}`",
        ]
        if pb:
            md.append(f"- serialized Player Birb instance: `{f3(pb['position'])}`, yaw `{pb['yaw_deg']:.2f}°`")
        if d["notes"]:
            md.append(f"- notes: {'; '.join(d['notes'])}")
        md += [
            "",
            "**Bounds** (colliders, world-space, colliders > 100 m excluded)",
            "",
            f"- min `{f3(b['min'])}` m; max `{f3(b['max'])}` m; size `{f3(b['size'])}` m; "
            f"center `{f3(b['center'])}` m",
            f"- colliders used: " + ", ".join(f"{k} {v}" for k, v in sorted(b["colliders_used"].items())),
            f"- excluded: " + (", ".join(f"{k} {v}" for k, v in sorted(b["excluded"].items())) or "none"),
            f"- skipped: " + ", ".join(f"{k} {v}" for k, v in sorted(b["skipped"].items())),
            f"- mesh-collider-only bounds: `{f3(b['mesh_collider_min'])}` .. `{f3(b['mesh_collider_max'])}`",
            f"- unfiltered (including >100 m colliders): `{f3(b['unfiltered_min'])}` .. `{f3(b['unfiltered_max'])}`",
            "",
            "**Travel points**",
            "",
            "| kind | name | x | y | z | yaw |",
            "|---|---|---|---|---|---|",
        ]
        for t in d["travel_points"]:
            yaw = "" if t["yaw_deg"] is None else f"{t['yaw_deg']:.2f}°"
            md.append(f"| {t['kind']} | {t['name']} | {t['position'][0]:.3f} | {t['position'][1]:.3f} | "
                      f"{t['position'][2]:.3f} | {yaw} |")
        md += ["", "**Lighting**", ""]
        if sun:
            md += [
                f"- sun: `{sun['name']}` (`{sun['path']}`), intensity `{sun['intensity']}`, color "
                f"`({', '.join(f'{c:.4f}' for c in sun['color'][:3])})`, shadows `{sun['shadows']}`",
                f"  - Unity euler `{f2(sun['euler_deg'])}°`; forward `{f3(sun['forward'])}`; "
                f"quaternion `({', '.join(f'{q:.6f}' for q in sun['rotation_quaternion'])})`",
            ]
        else:
            md.append("- sun: none")
        active_dirs = [l for l in d.get("directional_lights", []) if l["active_in_hierarchy"] and l["enabled"]]
        if len(active_dirs) > 1:
            md.append(f"- other active directional lights: " +
                      ", ".join(f"`{l['name']}` (int {l['intensity']}, euler {f2(l['euler_deg'])})"
                                for l in active_dirs if l is not sun))
        if rs:
            md += [
                f"- ambient: `{rs['ambient_mode']}`, sky `({', '.join(f'{c:.4f}' for c in rs['ambient_sky_color'][:3])})`, "
                f"equator `({', '.join(f'{c:.4f}' for c in rs['ambient_equator_color'][:3])})`, "
                f"ground `({', '.join(f'{c:.4f}' for c in rs['ambient_ground_color'][:3])})`, "
                f"intensity `{rs['ambient_intensity']}`",
                f"- skybox material: `{rs['skybox_material']}`",
                f"- fog: `{rs['fog']}`, color `({', '.join(f'{c:.4f}' for c in rs['fog_color'][:3])})`, "
                f"mode `{rs['fog_mode']}`, density `{rs['fog_density']}`, linear "
                f"`{rs['linear_fog_start']}`..`{rs['linear_fog_end']}`",
                f"- reflections: `{rs['default_reflection_mode']}`, intensity `{rs['reflection_intensity']}`, "
                f"subtractive shadow color `({', '.join(f'{c:.4f}' for c in rs['subtractive_shadow_color'][:3])})`",
            ]
        md += [""]

    md += [
        "## Caveats",
        "",
        "- `level7` (flashback) has **no** `startingPosition` reset flag; `ResetToStart` is a no-op there and the",
        "  serialized `Player Birb` transform `(0, 1.189, 0)` is the effective spawn. Nearest named reset anchor:",
        "  `The Alley` `(4.75, 0.296, 5.29)`.",
        "- `level12` (playground) and `level5` (bedroom 2) place the `Player Birb` instance at the origin; the game",
        "  teleports it to the flagged ResetPosition at scene start, so the flagged reset is the real spawn.",
        "- Bounds are collider coverage, not render coverage. Some real level structure (e.g. 130 x 160 m room",
        "  floors/walls in bedroom 1) exceeds the 100 m filter and is excluded from the bounds but still present",
        "  in the level. The JSON keeps the unfiltered range and the list of excluded oversized colliders.",
        "- Scene root names are stale in places (the office scene root says `Missions - Server Room`); level",
        "  identity above comes from the scene file, not the root names.",
        "",
    ]

    os.makedirs(os.path.dirname(OUT_JSON), exist_ok=True)
    os.makedirs(os.path.dirname(OUT_MD), exist_ok=True)
    with open(OUT_JSON, "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1, ensure_ascii=True)
    with open(OUT_MD, "w", encoding="utf-8") as f:
        f.write("\n".join(md))
    print("wrote", OUT_JSON)
    print("wrote", OUT_MD)


if __name__ == "__main__":
    main()
