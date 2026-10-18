#!/bin/bash
# Merge the eight level scenes into one combined level and compile it.
set -eu
ROOT=C:/Users/kalmarv/ds-modding
EXE="$ROOT/target/release/examples"
OUT="$ROOT/out/all-levels"
GAME="$ROOT/game"
rm -rf "$OUT"; mkdir -p "$OUT"
SCENES=()
for L in level1 level2 level3 level4 level5 level7 level12 level13; do
  SCENES+=("'$ROOT/out/$L/unity/skate/scene'")
done
powershell -NoProfile -Command "& '$EXE/merge_scenes.exe' '$OUT/scene' 2500,2500 ${SCENES[*]}" 2>&1 | tail -n 12
powershell -NoProfile -Command "& '$EXE/build_map.exe' '$OUT/scene' --out '$OUT/map' --name SkateBIRD_AllLevels" 2>&1 | tail -n 3
mkdir -p "$OUT/stage"
powershell -NoProfile -Command "& '$ROOT/tools/ReSkateStudio-20261012/reskate_cli.exe' compile-map '$GAME' '$OUT/map/SkateBIRD_AllLevels' '$OUT/stage' --mod-folder SkateBIRD_AllLevels --no-gi" 2>&1 | tail -n 8
echo "merged map at $OUT"
