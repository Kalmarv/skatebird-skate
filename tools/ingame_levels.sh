#!/usr/bin/env bash
# One skate. run per level through the shared runner (reskate-tools ingame.sh: copy mode, every other mod disabled and
# restored, crash reporting off): the level loaded, a third-person shot at the spawn, then a first-person shot at each
# of the level's views (data/skatebird-views.json). Run as `kit/run game --job -- "$BASH" tools/ingame_levels.sh TAG LEVELS...` (BUILD: the build --out folder).
set -u
TAG=$1; shift
W=$(cd "$(dirname "$0")/.." && pwd)
E=/c/Users/kalmarv/modding/reskate/reskate-tools/ports/gta/engine
B=${BUILD:-/c/Users/kalmarv/modding/cache/skatebird/build}
R=/c/Users/kalmarv/modding/cache/skatebird/runs/$TAG
mkdir -p $R
for L in "$@"; do
  S=$B/$L/stage/Patch
  [ -d "$S" ] || { echo "$L: no stage"; continue; }
  C=$(python -I $W/tools/views.py skate-commands $L)
  NO_DLL=1 "$BASH" $E/ingame.sh $R/$L copy "$S" SkateBIRD_$L "$C" 330 > $R/$L.txt 2>&1; e=$?
  act=$(grep -a -c "dingolevel_reskate_skatebird_$L" $R/$L/ReSkate.log 2>/dev/null)
  echo "$L: runner exit $e; level lines in log $act; shots $(ls $R/$L/where_*.png 2>/dev/null | wc -l); dumps $(grep -c 'crash dump' $R/$L.txt)"
  grep -a "\[Console\] \[INFO\] (" $R/$L/ReSkate.log 2>/dev/null | cut -c1-200 | sed 's/^/   /'
done
