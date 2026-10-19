#!/usr/bin/env bash
# Fit the SkateBIRD chase camera to skate. third-person shots (tools/skatebird_chase_fit.py has the method).
#
#   C:/Users/kalmarv/modding/kit/run game --job -- "$BASH" tools/skatebird_chase_fit.sh run <level> <outdir> <target.png>...
#   tools/skatebird_chase_fit.sh fit <outdir> <target.png>...      (offline, on <outdir>'s renders)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [ "${1:-}" = "run" ]; then
  bash "$ROOT/tools/skatebird_lab.sh" --check >/dev/null || { echo "the lab copy is incomplete; run tools/skatebird_lab.sh" >&2; exit 2; }
fi
exec python -I "$ROOT/tools/skatebird_chase_fit.py" "$@"
