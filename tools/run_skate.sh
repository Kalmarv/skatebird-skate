#!/bin/bash
# One isolated skate. run straight into a custom level, with screenshots.
#   bash tools/run_skate.sh <level asset> <out dir> [settle seconds]
# Isolated install: ds-modding/game (hardlinked/copied binaries, Data + __Installer junctions to the real
# install, its own Mods/ and LOCALAPPDATA). Caleb's install is never written. Run inside a job:
#   kit/run game --job -- bash tools/run_skate.sh <level> <out dir>
set -u
G=/c/Users/kalmarv/ds-modding/game
A=/c/Users/kalmarv/ds-modding/appdata
MOD=${4:-SkateBIRD_Rooftop}
MOD_SOURCE=${BIRD_MOD_SOURCE:-$G/Mods.installed/$MOD}
LEVEL=${1:?level asset}
OUT=${2:?output dir}
SETTLE=${3:-40}
SHOTS=${5:-4}
mkdir -p "$OUT"
GW=$(cygpath -w "$G")
LAUNCHER="$GW\\ReSkateLauncher.exe"
SKATE_EXE="$GW\\Skate.exe"

if tasklist | grep -qi '^Skate.exe'; then echo "ABORT: a Skate.exe is already running"; exit 2; fi

# only our mod, enabled
if [ ! -f "$MOD_SOURCE/layout.toc" ]; then echo "missing compiled patch: $MOD_SOURCE"; exit 2; fi
rm -rf "$G/Mods"/*
cp -r "$MOD_SOURCE" "$G/Mods/$MOD" || exit 2
printf '{"schema": 1, "mods": [{"enabled": true, "name": "%s"}]}\n' "$MOD" > "$G/Mods/mods.json"
# fresh LOCALAPPDATA from the seed (camera/settings profile)
rm -rf "$A/run"; cp -r "$A/seed" "$A/run"
rm -f "$G/logs/ReSkate.log"
mkdir -p "$G/logs"

CMD="load $LEVEL;wait 2;echo SKATEBIRD_RUN_READY"
echo "launch $(date +%T) level $LEVEL"
LOCALAPPDATA="$(cygpath -w "$A/run")" RESKATE_CRASH_REPORTING=0 RESKATE_STARTUP_COMMANDS="$CMD" \
  "$G/ReSkateLauncher.exe" --no-gui --no-update --offline --windowed --width=1920 --height=1080 > "$OUT/launcher.txt" 2>&1 &
SPID=
for i in $(seq 1 90); do
  sleep 2
  SPID=$(powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='Skate.exe'\" | Where-Object { \$_.ExecutablePath -eq '$SKATE_EXE' } | ForEach-Object { \$_.ProcessId }" | tr -d '\r' | head -1)
  [ -n "$SPID" ] && break
done
echo "Skate.exe pid=${SPID:-none} at $(date +%T)"
if [ -z "$SPID" ]; then echo "no game"; tail -20 "$OUT/launcher.txt"; exit 3; fi

FF='C:\Users\kalmarv\AppData\Local\universal-modder\ffmpeg\bin\ffmpeg.exe'
shot() {  # shot <name> - ffmpeg through PowerShell: this shell cannot execute .exe files directly
  powershell -NoProfile -Command "& '$FF' -hide_banner -loglevel error -y -f lavfi -i 'gfxcapture=window_exe=Skate.exe:capture_cursor=0:max_framerate=60,hwdownload,format=bgra' -frames:v 1 '$(cygpath -w "$OUT/$1.png")'" > "$OUT/shot.err" 2>&1
  echo "shot $1 at $(date +%T) rc=$?"
}
n=0
for i in $(seq 1 "$SHOTS"); do
  sleep "$SETTLE"
  n=$((n + 1))
  shot "shot_$n"
  grep -a "Custom-level\|Level\|level" "$G/logs/ReSkate.log" 2>/dev/null | tail -3
  if ! tasklist //FI "PID eq $SPID" 2>/dev/null | grep -qi skate; then echo "game exited"; break; fi
done
echo "=== log tail ==="; tail -30 "$G/logs/ReSkate.log" 2>/dev/null
cp "$G/logs/ReSkate.log" "$OUT/ReSkate.log"
# close only the instance this script started, and wait for it to be gone
powershell -NoProfile -Command "Stop-Process -Id $SPID -Force" 2>/dev/null || true
for i in $(seq 1 40); do
  tasklist 2>/dev/null | grep -qi '^Skate.exe' || break
  sleep 2
done
if tasklist 2>/dev/null | grep -qi '^Skate.exe'; then echo "WARNING: Skate.exe still running after 80 s"; fi
echo "done $(date +%T)"
