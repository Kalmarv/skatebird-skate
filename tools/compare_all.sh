#!/bin/bash
# Side-by-side sheets: SkateBIRD reference vs the skate. spawn shot, for every level.
#   bash tools/compare_all.sh [shot index, default 2]
set -u
ROOT=C:/Users/kalmarv/ds-modding
N=${1:-2}
declare -A REF=( [level1]=bedroom_1 [level2]=rooftop [level3]=office [level4]=server_room [level5]=bedroom_2 [level7]=flashback [level12]=playground [level13]=skate_heaven )
for L in level1 level2 level3 level4 level5 level7 level12 level13; do
  R="$ROOT/out/skatebird_${REF[$L]}_spawn.png"
  S="$ROOT/out/run_$L/shot_$N.png"
  if [ -f "$R" ] && [ -f "$S" ]; then
    powershell -NoProfile -Command "python '$ROOT/tools/compare_shots.py' '$ROOT/out/compare_$L.png' '$(cygpath -w "$R")' '$(cygpath -w "$S")' --labels 'SkateBIRD $L' 'skate. $L'"
  else
    echo "missing for $L: ref=$( [ -f "$R" ] && echo yes || echo no ) shot=$( [ -f "$S" ] && echo yes || echo no )"
  fi
done
