#!/bin/bash
# Test three spawn-yaw conventions for one level and capture each at the spawn.
#   bash tools/yaw_test.sh <levelN> <mod name> <yaw1> <yaw2> <yaw3>
set -u
ROOT=C:/Users/kalmarv/ds-modding
G="$ROOT/game"
L=${1:?level}
MODSRC=${2:?installed mod name}
shift 2
EXE="$ROOT/target/release/examples"
STUDIO="$ROOT/tools/ReSkateStudio-20261012/reskate_cli.exe"
for YAW in "$@"; do
  TAG="yaw${YAW}"   # decimal point free enough for a folder name
  D="$ROOT/out/yawtest/$TAG"
  rm -rf "$D"; mkdir -p "$D"
  cp -r "$ROOT/out/$L/unity/skate/scene" "$D/scene"
  python - "$D/scene/scene.json" "$YAW" <<'PYEOF'
import json, sys
p, yaw = sys.argv[1], float(sys.argv[2])
d = json.load(open(p, encoding="utf-8"))
for m in d["markers"]:
    if m.get("kind") == "spawn":
        m["yaw"] = yaw
json.dump(d, open(p, "w", encoding="utf-8"), indent=1)
PYEOF
  powershell -NoProfile -Command "& '$EXE/build_map.exe' '$(cygpath -w "$D/scene")' --out '$(cygpath -w "$D/map")' --name "$MODSRC""
  mkdir -p "$D/stage"
  powershell -NoProfile -Command "& '$STUDIO' compile-map '$G' '$(cygpath -w "$D/map/$MODSRC")' '$(cygpath -w "$D/stage")' --mod-folder "$MODSRC" --no-gi" > "$D/compile.log" 2>&1
  rm -rf "$G/Mods.installed/$MODSRC"; cp -r "$D/stage/Patch" "$G/Mods.installed/$MODSRC"
  bash "$ROOT/tools/run_skate.sh" "levels/game/dingolevel_reskate_skatebird_$L/dingolevel_reskate_skatebird_$L" "$D/run" 30 "$MODSRC" 1
done
