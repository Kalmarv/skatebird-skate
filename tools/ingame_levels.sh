#!/usr/bin/env bash
# Every level in one skate. launch, through the shared runner (reskate-tools ingame.sh in copy mode with several mods:
# each level's mod copied in, every other mod disabled, all restored after; crash reporting off). Each level is loaded
# in turn, shot at the spawn and at each of its views (data/skatebird-views.json; every `trainer where` is a shot).
#   kit/run game --job -- "$BASH" tools/ingame_levels.sh TAG [LEVEL ...]     (default: every level built)
# BUILD: the program's build --out folder (default <modding>/cache/skatebird/build). Output: cache/skatebird/runs/TAG.
set -u
TAG=$1; shift
W=$(cd "$(dirname "$0")/.." && pwd)
E=/c/Users/kalmarv/modding/reskate/reskate-tools/ports/gta/engine
B=${BUILD:-/c/Users/kalmarv/modding/cache/skatebird/build}
R=/c/Users/kalmarv/modding/cache/skatebird/runs/$TAG
LEVELS=("$@"); [ ${#LEVELS[@]} -gt 0 ] || LEVELS=(level1 level2 level3 level4 level5 level7 level12 level13)
SRCS=; NAMES=; CMDS="wait 100"; SECS=100
for L in "${LEVELS[@]}"; do
  [ -d "$B/$L/stage/Patch" ] || { echo "$L: no stage in $B"; exit 2; }
  SRCS+=${SRCS:+,}$B/$L/stage/Patch; NAMES+=${NAMES:+,}SkateBIRD_$L
  C=$(python -I "$W/tools/views.py" skate-commands "$L") || exit 2
  CMDS+=";${C#wait 100;}"
  # load + settle + 4 views (wait 45 + 6 + 3 x 18), with room for the load itself under load
  SECS=$((SECS + 160))
done
mkdir -p "$R"
printf '%s\n' "${LEVELS[@]}" > "$R/levels.txt"
NO_DLL=1 "$BASH" "$E/ingame.sh" "$R/run" copy "$SRCS" "$NAMES" "$CMDS" "$SECS" > "$R/run.txt" 2>&1; e=$?
echo "runner exit $e; shots $(ls "$R"/run/where_*.png 2>/dev/null | grep -vc _small); crash dumps: $(grep -c 'crash dump' "$R/run.txt")"
# the console lines in order: per level, the load and its `trainer where` readings (one per shot)
grep -a "load levels/game\|\[Console\] \[INFO\] (" "$R/run/ReSkate.log" 2>/dev/null | cut -c1-200
exit $e
