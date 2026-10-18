#!/usr/bin/env bash
# SkateBIRD reproducible screenshot runner.
#
#   tools/skatebird_shot.sh <level> <out.png> [--first-run] [--no-ui] [--settle S] [--keep]
#   tools/skatebird_shot.sh capture <out.png> [--ffmpeg]
#
# <level> is a GameLevels enum name accepted by MissionManager.LoadLevel:
#   title, bedroom_1, rooftop, office, server_room, bedroom_2, credits, playground, skate_heaven
#
# Verified building blocks (2026-10-18):
#   - launch with -screen-fullscreen 0 -screen-width 1920 -screen-height 1080 -devconsole
#     gives a 1920x1080 CLIENT area (um win drive rect -> 1920x1080).
#   - F1 opens/closes the ReflexCLI console; Tab does nothing. Key comes from
#     resources.assets/ReflexConsoleSettings (_OpenConsoleKey = KeyCode.F1 = 282).
#   - "MissionManager.LoadLevel rooftop" typed into the console loads the level from the
#     title screen (observed: accessibility popup over rooftop).
#   - The first-run chain on a fresh save, in this order: Esc (first-options popup),
#     Esc (Create-a-Birb), Enter (pause menu -> RESUME SKATEBOARDING), then gameplay.
set -euo pipefail

GAME_DIR="C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD"
GAME_EXE="$GAME_DIR/SkateBIRD.exe"
PROC="SkateBIRD"
W=1920
H=1080
FF="C:/Users/kalmarv/AppData/Local/universal-modder/ffmpeg/bin/ffmpeg.exe"

usage() {
  awk 'NR > 1 && /^set -euo/ { exit } NR > 1 { sub(/^# ?/, ""); print }' "$0"
  exit "${1:-1}"
}

# ---- capture-only mode -------------------------------------------------------
if [ "${1:-}" = "capture" ]; then
  out="${2:-}"
  [ -n "$out" ] || usage
  shift 2
  mode="um"
  while [ $# -gt 0 ]; do
    case "$1" in
      --ffmpeg) mode="ffmpeg" ;;
      -h|--help) usage 0 ;;
      *) echo "unknown argument: $1" >&2; usage ;;
    esac
    shift
  done
  mkdir -p "$(dirname "$out")"
  if [ "$mode" = "ffmpeg" ]; then
    # Same input graph um win shot uses. The lavfi source emits d3d11 frames, hence
    # hwdownload + format=bgra before the png encoder.
    "$FF" -hide_banner -loglevel error -f lavfi \
      -i "gfxcapture=window_exe=${PROC}.exe:capture_cursor=0:max_framerate=60,hwdownload,format=bgra" \
      -frames:v 1 -y "$out"
  else
    um win shot --exe "${PROC}.exe" "$out"
  fi
  echo "$out"
  exit 0
fi

if [ $# -lt 2 ]; then usage; fi
LEVEL="$1"
OUT="$2"
shift 2

FIRST_RUN=0
NO_UI=0
SETTLE=12
KEEP=0
while [ $# -gt 0 ]; do
  case "$1" in
    --first-run) FIRST_RUN=1 ;;
    --no-ui) NO_UI=1 ;;
    --settle) SETTLE="${2:?}"; shift ;;
    --keep) KEEP=1 ;;
    -h|--help) usage 0 ;;
    *) echo "unknown argument: $1" >&2; usage ;;
  esac
  shift
done

case "$LEVEL" in
  title|bedroom_1|rooftop|office|server_room|bedroom_2|credits|playground|skate_heaven) ;;
  *) echo "refusing: '$LEVEL' is not a GameLevels enum name (flashback/loading have no LoadLevel path)" >&2; exit 2 ;;
esac

# The desktop is shared: never start a second game run behind another one.
if um win ps | awk '{print $2}' | grep -qx "$PROC"; then
  echo "SkateBIRD is already running; close it first (um win kill <pid>)" >&2
  exit 2
fi

lock_first=$(orca lock status heavy 2>/dev/null | head -1 || true)
if [ -n "$lock_first" ]; then
  echo "heavy lock: $lock_first"
  case "$lock_first" in
    *paused*|*"in use"*[1-9]*) echo "WARNING: heavy lock is not free; the run will share the display/GPU." >&2 ;;
  esac
fi

echo "launching $LEVEL at ${W}x${H} windowed"
# um win launch stays attached for the game's lifetime; background it and take the game pid from um.
um win launch "$GAME_EXE" -- -screen-fullscreen 0 -screen-width "$W" -screen-height "$H" -devconsole \
  >/dev/null 2>&1 &
PID=""

cleanup() {
  if [ "$KEEP" -eq 0 ] && [ -n "${PID:-}" ]; then
    um win kill "$PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for _ in $(seq 1 120); do
  um win ps | awk '{print $2}' | grep -qx "$PROC" && break
  sleep 0.5
done
if ! um win ps | awk '{print $2}' | grep -qx "$PROC"; then
  echo "window did not appear" >&2
  exit 3
fi
PID=$(um win ps | awk -v p="$PROC" '$2 == p { print $1; exit }')
echo "pid $PID"

um win drive --proc "$PROC" "focus" "sleep 3000" \
  "key 0x70" "sleep 800" \
  "type MissionManager.LoadLevel $LEVEL" "key 0x0D" >/dev/null
echo "load issued; settling ${SETTLE}s"
sleep "$SETTLE"

if [ "$FIRST_RUN" -eq 1 ]; then
  echo "dismissing fresh-save popups (Esc, Esc, Enter)"
  um win drive --proc "$PROC" "focus" \
    "key 0x1B" "sleep 2000" \
    "key 0x1B" "sleep 2500" \
    "key 0x0D" "sleep 4000" >/dev/null
fi

if [ "$NO_UI" -eq 1 ]; then
  echo "hiding gameplay HUD (Options.ToggleUI)"
  um win drive --proc "$PROC" "focus" \
    "key 0x70" "sleep 500" \
    "type Options.ToggleUI" "key 0x0D" "sleep 500" \
    "key 0x70" "sleep 1200" >/dev/null
fi

mkdir -p "$(dirname "$OUT")"
um win shot --exe "${PROC}.exe" "$OUT"
echo "wrote $OUT"

if [ "$NO_UI" -eq 1 ]; then
  # Restore the persisted option so later runs start with the HUD on.
  um win drive --proc "$PROC" "focus" \
    "key 0x70" "sleep 400" \
    "type Options.ToggleUI" "key 0x0D" "sleep 400" \
    "key 0x70" "sleep 600" >/dev/null || true
fi

if [ "$KEEP" -eq 0 ]; then
  um win kill "$PID" >/dev/null
  echo "closed $PID"
  PID=""
fi
