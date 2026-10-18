"""Install a compiled ReSkate Studio stage as a mod folder (SkateBIRD port workflow).

    python tools/install_mod.py <stage dir> <ReSkate game dir> <mod folder name> --display "..." [--version 0.1.0]

Copies <stage>/Patch into <game>/Mods/<mod folder name>, writes manifest.json, sets the level's
displayName, and enables the mod in Mods/mods.json (backing it up first). No writes outside the game
folder and C:/Users/kalmarv/ds-modding.
"""
import argparse
import json
import shutil
import time
from pathlib import Path


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("stage")
    ap.add_argument("game")
    ap.add_argument("name")
    ap.add_argument("--display", required=True)
    ap.add_argument("--version", default="0.1.0")
    ap.add_argument("--description", default="A SkateBIRD level ported to skate. through primase. Unofficial fan port.")
    a = ap.parse_args()

    patch = Path(a.stage) / "Patch"
    if not (patch / "layout.toc").exists():
        raise SystemExit(f"no compiled stage in {patch}")
    mods = Path(a.game) / "Mods"
    dest = mods / a.name
    if dest.exists():
        backup = dest.with_name(f"{dest.name}.backup-{time.strftime('%Y%m%d-%H%M%S')}")
        dest.rename(backup)
        print("backed up", dest, "->", backup)
    shutil.copytree(patch, dest)

    levels = json.loads((dest / "reskate-levels.json").read_text(encoding="utf-8"))
    for lv in levels["levels"]:
        lv["displayName"] = a.display
    (dest / "reskate-levels.json").write_text(json.dumps(levels, indent=2) + "\n", encoding="utf-8")

    manifest = {
        "name": a.name,
        "version_number": a.version,
        "website_url": "",
        "description": a.description,
        "dependencies": [],
        "author": "Kalmarv",
    }
    (dest / "manifest.json").write_text(json.dumps(manifest, indent=4) + "\n", encoding="utf-8")

    registry = mods / "mods.json"
    data = json.loads(registry.read_text(encoding="utf-8")) if registry.exists() else {"schema": 1, "mods": []}
    (mods / f"mods.json.bak-skatebird-{time.strftime('%Y%m%d-%H%M%S')}").write_text(
        registry.read_text(encoding="utf-8") if registry.exists() else "", encoding="utf-8"
    )
    entries = [m for m in data.get("mods", []) if m.get("name") != a.name]
    entries.append({"enabled": True, "name": a.name})
    data["mods"] = entries
    registry.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print("installed", dest, "levels", [(l["asset"], l["displayName"]) for l in levels["levels"]])


if __name__ == "__main__":
    main()
