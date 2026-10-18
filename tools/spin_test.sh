#!/bin/bash
# Spin the skate camera around the spawn in 90-degree-ish steps (relative mouse) and capture each view.
#   bash tools/spin_test.sh <levelN> <mod name> <out dir> [dx per turn]
set -u
ROOT=C:/Users/kalmarv/ds-modding
G="$ROOT/game"; A="$ROOT/appdata"
L=${1:?level}; MOD=${2:?mod}; OUT=${3:?out dir}; DX=${4:-800}
mkdir -p "$OUT"
if tasklist | grep -qi '^Skate.exe'; then echo "ABORT: Skate.exe running"; exit 2; fi
rm -rf "$G/Mods"/*; cp -r "$G/Mods.installed/$MOD" "$G/Mods/$MOD"
printf '{"schema": 1, "mods": [{"enabled": true, "name": "%s"}]}\n' "$MOD" > "$G/Mods/mods.json"
rm -rf "$A/run"; cp -r "$A/seed" "$A/run"; rm -f "$G/logs/ReSkate.log"
LOCALAPPDATA="$(cygpath -w "$A/run")" RESKATE_CRASH_REPORTING=0 \
  RESKATE_STARTUP_COMMANDS="load levels/game/dingolevel_reskate_skatebird_$L/dingolevel_reskate_skatebird_$L; wait 3; trainer option hud 0" \
  "$G/ReSkateLauncher.exe" --no-gui --no-update --offline --windowed --width=1920 --height=1080 > "$OUT/launcher.txt" 2>&1 &
SPID=
for i in $(seq 1 90); do sleep 2; SPID=$(powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='Skate.exe'\" | Where-Object { \$_.ExecutablePath -eq '$(cygpath -w "$G")\\Skate.exe' } | ForEach-Object { \$_.ProcessId }" | tr -d '\r' | head -1); [ -n "$SPID" ] && break; done
[ -n "$SPID" ] || { echo "no game"; exit 3; }
sleep 30
FF='C:\Users\kalmarv\AppData\Local\universal-modder\ffmpeg\bin\ffmpeg.exe'
shot() { powershell -NoProfile -Command "& '$FF' -hide_banner -loglevel error -y -f lavfi -i 'gfxcapture=window_exe=Skate.exe:capture_cursor=0:max_framerate=60,hwdownload,format=bgra' -frames:v 1 '$(cygpath -w "$OUT/$1.png")'" >/dev/null 2>&1; echo "shot $1 $(date +%T)"; }
shot view0
um win drive --proc Skate.exe "focus" "scanmode on" >/dev/null 2>&1 || true
sleep 1
for n in 1 2 3; do
  um win drive --proc Skate.exe "rel $DX 0" >/dev/null 2>&1 || true
  sleep 3
  shot "view$n"
done
um win drive --proc Skate.exe "scanmode off" >/dev/null 2>&1 || true
powershell -NoProfile -Command "Stop-Process -Id $SPID -Force" 2>/dev/null || true
sleep 3
echo done
