#!/bin/bash
# One screenshot pass over every exported level, in the isolated skate. install.
#   bash tools/run_levels.sh [settle] [shots]
# Needs out/<level>/stage/Patch (tools/export_level.sh) for each level.
set -u
ROOT=C:/Users/kalmarv/ds-modding
G="$ROOT/game"
SETTLE=${1:-45}
SHOTS=${2:-3}
for L in level1 level2 level3 level4 level5 level7 level12 level13; do
  echo "=== $L ==="
  rm -rf "$G/Mods.installed/SkateBIRD_$L"
  cp -r "$ROOT/out/$L/stage/Patch" "$G/Mods.installed/SkateBIRD_$L" || { echo "no stage for $L"; continue; }
  bash "$ROOT/tools/run_skate.sh" \
    "levels/game/dingolevel_reskate_skatebird_$L/dingolevel_reskate_skatebird_$L" \
    "$ROOT/out/run_$L" "$SETTLE" "SkateBIRD_$L" "$SHOTS"
done
echo "all levels done"
