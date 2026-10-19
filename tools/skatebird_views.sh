#!/usr/bin/env bash
# SkateBIRD reference shots of one level's views (data/skatebird-views.json) at exact camera poses.
#
#   tools/skatebird_views.sh <level> <outdir> [--chase] [--back D] [--eye H] [--fov F] [--pitch P] [--yaw-offset Y]
#                            [--settle S] [--keep-post] [--keep]
#
# It opens a game window, so run it under the screen lock:
#   C:/Users/kalmarv/modding/kit/run game --job -- "$BASH" tools/skatebird_views.sh level2 C:/Users/kalmarv/modding/cache/skatebird/ref/level2
#
# All eight levels in one launch, each into <root>/<level>:
#   ... tools/skatebird_views.sh level1,level2,level3,level4,level5,level7,level12,level13 <root> --chase
#
# <level> is a data/skatebird-views.json key (level1 .. level13), or several joined by commas. Camera = view position - D m along the yaw + H m up,
# yaw = the view's yaw_deg + Y, pitch P (positive looks down), vertical FOV F; defaults D 0, H 1.6, P 0, F 60, or with
# --chase skate.'s fitted third-person camera (D -0.3, H 1.5, P 1.2, F 55.3); 1024x576 windowed, HUD and bird hidden.
# Writes <outdir>/<view index>_<name>.png, <outdir>/poses.json (read-back pose per shot) and <outdir>/plugin.log.
# Needs the lab copy: tools/skatebird_lab.sh. Details: tools/skatebird_views.py, the plugin's protocol:
# tools/skatebird-teleport-plugin/Plugin.cs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
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
exec python -I "$ROOT/tools/skatebird_views.py" "$@"
