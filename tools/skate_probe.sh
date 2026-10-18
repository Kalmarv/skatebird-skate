#!/bin/bash
# Place the skater at an exact spot and heading in the isolated skate. install, then screenshot.
#   bash tools/skate_probe.sh <mod name> <level asset> <x> <y> <z> <yaw> <out.png> [settle seconds]
# Console: ReSkate trainer (trainer tp <x> <y> <z>, trainer where, trainer option hud 0).
set -u
G=C:/Users/kalmarv/ds-modding/game
A=C:/Users/kalmarv/ds-modding/appdata
MOD=${1:?mod}
LEVEL=${2:?level asset}
X=${3:?x}; Y=${4:?y}; Z=${5:?z}; YAW=${6:?yaw}
OUT=${7:?out.png}
SETTLE=${8:-12}
OUTDIR=$(dirname "$OUT"); mkdir -p "$OUTDIR"
LOG="$G/logs/ReSkate.log"

if tasklist | grep -qi '^Skate.exe'; then echo "ABORT: Skate.exe already running"; exit 2; fi
rm -rf "$G/Mods"/*; cp -r "$G/Mods.installed/$MOD" "$G/Mods/$MOD"
printf '{"schema": 1, "mods": [{"enabled": true, "name": "%s"}]}\n' "$MOD" > "$G/Mods/mods.json"
rm -rf "$A/run"; cp -r "$A/seed" "$A/run"
rm -f "$LOG"

CMDS="load $LEVEL; wait 4; trainer option hud 0; trainer tp $X $Y $Z; wait 2; trainer where; echo PROBE_TP; wait 3; trainer where; echo PROBE_WHERE"
LOCALAPPDATA="$(cygpath -w "$A/run")" RESKATE_CRASH_REPORTING=0 RESKATE_STARTUP_COMMANDS="$CMDS" \
  "$G/ReSkateLauncher.exe" --no-gui --no-update --offline --windowed --width=1920 --height=1080 > "$OUTDIR/launcher.txt" 2>&1 &
SPID=
for i in $(seq 1 90); do
  sleep 2
  SPID=$(powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='Skate.exe'\" | Where-Object { \$_.ExecutablePath -eq '$(cygpath -w "$G")\\Skate.exe' } | ForEach-Object { \$_.ProcessId }" | tr -d '\r' | head -1)
  [ -n "$SPID" ] && break
done
[ -n "$SPID" ] || { echo "no game"; tail -20 "$OUTDIR/launcher.txt"; exit 3; }
echo "pid=$SPID; waiting for PROBE_WHERE"
for i in $(seq 1 120); do
  sleep 2
  grep -a -q "PROBE_WHERE" "$LOG" 2>/dev/null && break
  tasklist //FI "PID eq $SPID" 2>/dev/null | grep -qi skate || { echo "game exited"; break; }
done

# heading: the trainer reports 'heading N'; turn with relative mouse until it is within 2 degrees of YAW
heading() { grep -a -o "heading [-0-9.]*" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}'; }
H=$(heading); echo "heading now ${H:-none}, want $YAW"
for i in 1 2 3 4 5 6; do
  [ -n "${H:-}" ] || break
  D=$(awk -v h="$H" -v y="$YAW" 'BEGIN{d=(y-h+540)%360-180; printf "%d", d*4}')
  [ "$D" = 0 ] && break
  [ "${D#-}" -lt 8 ] && break
  um win drive --proc Skate.exe "focus" "rel $D 0" > /dev/null 2>&1 || true
  sleep 1
  # re-query through the console (backtick opens it; typing goes to the game only while focused)
  um win drive --proc Skate.exe "focus" "key 0xC0" > /dev/null 2>&1 || true
  sleep 1
  um win drive --proc Skate.exe "type trainer where" "key 0x0D" "key 0x1B" > /dev/null 2>&1 || true
  sleep 2
  H=$(heading); echo "heading now ${H:-none}"
done

sleep "$SETTLE"
powershell -NoProfile -Command "& 'C:\Users\kalmarv\AppData\Local\universal-modder\ffmpeg\bin\ffmpeg.exe' -hide_banner -loglevel error -y -f lavfi -i 'gfxcapture=window_exe=Skate.exe:capture_cursor=0:max_framerate=60,hwdownload,format=bgra' -frames:v 1 '$(cygpath -w "$OUT")'" > "$OUTDIR/shot.err" 2>&1
echo "shot $OUT"
grep -a "PROBE_\|heading" "$LOG" | tail -4
powershell -NoProfile -Command "Stop-Process -Id $SPID -Force" 2>/dev/null || true
sleep 4
echo "done"
