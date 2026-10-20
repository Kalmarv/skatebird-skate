#!/usr/bin/env bash
# SkateBIRD reference shots of levels' views (data/skatebird-views.json) at exact camera poses: the program's
# `skatebird-shots` (program/src/shots.rs) after a check of the lab copy.
#
#   tools/skatebird_views.sh <level>[,<level>...] <outdir> [--chase] [--back D] [--eye H] [--fov F] [--pitch P]
#                            [--yaw-offset Y] [--settle S] [--load-settle S] [--keep-post] [--keep-npcs] [--keep]
#
# It opens a game window, so run it under the screen lock:
#   C:/Users/kalmarv/modding/kit/run game --job -- "$BASH" tools/skatebird_views.sh level2 C:/Users/kalmarv/modding/cache/skatebird/ref/level2 --chase
# All eight levels in one launch, each into <root>/<level>:
#   ... tools/skatebird_views.sh level1,level2,level3,level4,level5,level7,level12,level13 <root> --chase
#
# Camera = view position - D m along the yaw + H m up, yaw = the view's yaw_deg + Y, pitch P (positive looks down),
# vertical FOV F; defaults D 0, H 1.6, P 0, F 60, or with --chase skate.'s fitted third-person camera (D -0.3, H 1.5,
# P 1.2, F 55.3); 1024x576 windowed, HUD, bird and NPCs (--keep-npcs keeps them) hidden, lens effects off
# (--keep-post keeps them). Writes <outdir>/<view index>_<name>.png, <outdir>/poses.json (read-back pose and checks
# per shot) and <outdir>/plugin.log. Needs the lab copy (tools/skatebird_lab.sh) and the tooling (cargo build --release
# -p skatebird-tools; SKATEBIRD_TOOLS to use another build). The plugin's protocol: tools/skatebird-teleport-plugin/Plugin.cs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
X=${SKATEBIRD_TOOLS:-/c/Users/kalmarv/modding/games/skatebird/target/release/skatebird-tools.exe}
if [ $# -lt 2 ]; then
  awk 'NR > 1 && /^set -euo/ { exit } NR > 1 { sub(/^# ?/, ""); print }' "$0"
  exit 2
fi
if [ -z "${KIT_RUN_KIND:-}" ]; then
  echo "warning: not under kit/run; another game may hold the screen" >&2
fi
bash "$ROOT/tools/skatebird_lab.sh" --check >/dev/null || {
  echo "the lab copy is incomplete; run tools/skatebird_lab.sh" >&2
  exit 2
}
exec "$X" skatebird-shots "$@" --data "$ROOT/data"
