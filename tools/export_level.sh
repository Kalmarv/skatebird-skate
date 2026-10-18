#!/bin/bash
# One level end to end: Unity -> skate scene -> normalized map -> Studio compile -> mod folder install.
#   bash tools/export_level.sh <levelN> [name] [install|noinstall]
# Spawn/yaw and display names are the game's own (data/skatebird-levels.json, frozen here: job shells have no python).
# Unity (x, y, z) -> scene (x, y, -z); scene yaw = (180 - unity yaw) mod 360.
set -euo pipefail
ROOT=C:/Users/kalmarv/ds-modding
DATA="C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD/SkateBIRD_Data"
GAME="$ROOT/game"
LEVEL=${1:?level name (level1..level13)}
NAME=${2:-SkateBIRD_$LEVEL}
INSTALL=${3:-install}
EXE="$ROOT/target/release/examples"
OUT="${BIRD_OUTPUT_ROOT:-$ROOT/out}/$LEVEL"
MAP="$OUT/map/$NAME"
STAGE="$OUT/stage"

case "$LEVEL" in
  level1)  SPAWN="45.4,16.796,-55.77,234.82";       TITLE="bedroom 1" ;;
  level2)  SPAWN="-18.7,1.189,-84.8,7.0659";       TITLE="rooftop" ;;
  level3)  SPAWN="-34.026,11.98,78.359,180.0";     TITLE="office" ;;
  level4)  SPAWN="28.36,-1.0,16.86,238.66";       TITLE="server room" ;;
  level5)  SPAWN="-26.68,15.2,-10.05,234.95";     TITLE="bedroom 2" ;;
  level7)  SPAWN="0.0,1.189,0.0,7.0659";           TITLE="flashback" ;;
  level12) SPAWN="-59.09,1.1,-62.76,73.85";      TITLE="playground" ;;
  level13) SPAWN="30.7,0.2,23.6,194.09";         TITLE="skate heaven" ;;
  *) echo "unknown level $LEVEL"; exit 2 ;;
esac

if [ -e "$OUT" ]; then echo "output already exists: $OUT (use a fresh BIRD_OUTPUT_ROOT)"; exit 2; fi
mkdir -p "$OUT"
echo "spawn: --spawn $SPAWN"

powershell -NoProfile -Command "\$ErrorActionPreference = 'Stop'; & '$EXE/unity_export.exe' '$DATA' '$ROOT/data/typetrees.tpk' '$LEVEL' '$OUT/unity' --surfaces '$ROOT/data/skatebird-surfaces.json' --spawn $SPAWN --travel '$ROOT/data/skatebird-levels.json'; exit \$LASTEXITCODE" 2>&1 | tee "$OUT/export.log" | tail -n 3
powershell -NoProfile -Command "\$ErrorActionPreference = 'Stop'; & '$EXE/build_map.exe' '$OUT/unity/skate/scene' --out '$OUT/map' --name '$NAME'; exit \$LASTEXITCODE" 2>&1 | tee "$OUT/build.log" | tail -n 3
mkdir -p "$STAGE"
powershell -NoProfile -Command "\$ErrorActionPreference = 'Stop'; & '$ROOT/tools/ReSkateStudio-20261012/reskate_cli.exe' compile-map '$GAME' '$MAP' '$STAGE' --mod-folder '$NAME' --no-gi --time-of-day noon; exit \$LASTEXITCODE" 2>&1 | tee "$OUT/compile.log" | tail -n 8
if [ "$INSTALL" = "install" ]; then
  powershell -NoProfile -Command "\$ErrorActionPreference = 'Stop'; python '$ROOT/tools/install_mod.py' '$STAGE' '$GAME' '$NAME' --display 'SkateBIRD - $TITLE'; exit \$LASTEXITCODE"
fi
echo "done: $OUT"
