#!/usr/bin/env bash
# Make (or refresh) the SkateBIRD lab copy and build the lab plugin into it.
#
#   tools/skatebird_lab.sh            create/refresh the lab, (re)build and install the plugin
#   tools/skatebird_lab.sh --check    only report what the lab has
#
# The lab is C:/Users/kalmarv/modding/cache/skatebird/lab (override with SKATEBIRD_LAB): real copies of the install's
# root files (SkateBIRD.exe, UnityPlayer.dll, UnityCrashHandler64.exe), junctions SkateBIRD_Data and MonoBleedingEdge
# into the Steam install, BepInEx 5 (win x64, pinned release and sha256) and tools/skatebird-teleport-plugin in
# BepInEx/plugins. Nothing is written into the Steam install. Never recurse into or delete through the junctions:
# remove one with `cmd //c rmdir <path>` only.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GAME="C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD"
LAB="${SKATEBIRD_LAB:-C:/Users/kalmarv/modding/cache/skatebird/lab}"
DL="$(dirname "$LAB")/downloads"
BEPINEX_URL="https://github.com/BepInEx/BepInEx/releases/download/v5.4.23.5/BepInEx_win_x64_5.4.23.5.zip"
BEPINEX_SHA256="82f9878551030f54657792c0740d9d51a09500eeae1fba21106b0c441e6732c4"
ROOT_FILES=(SkateBIRD.exe UnityPlayer.dll UnityCrashHandler64.exe)
JUNCTIONS=(SkateBIRD_Data MonoBleedingEdge)
PLUGIN="$ROOT/tools/skatebird-teleport-plugin"

win() { cygpath -w "$1"; }

# Prints the junction target of $1, or nothing if it is not a junction.
junction_target() {
  powershell -NoProfile -Command \
    "\$i = Get-Item -LiteralPath '$(win "$1")' -Force -ErrorAction SilentlyContinue; if (\$i -and \$i.LinkType -eq 'Junction') { \$i.Target }; exit 0" \
    | tr -d '\r'
}

check() {
  local bad=0
  for f in "${ROOT_FILES[@]}"; do
    if [ -f "$LAB/$f" ] && cmp -s "$GAME/$f" "$LAB/$f"; then echo "ok   $f (copy)"; else echo "MISS $f"; bad=1; fi
  done
  for j in "${JUNCTIONS[@]}"; do
    local t
    t="$(junction_target "$LAB/$j")"
    if [ "$t" = "$(win "$GAME/$j")" ]; then echo "ok   $j -> $t"; else echo "MISS $j (junction target: '${t}')"; bad=1; fi
  done
  for f in winhttp.dll doorstop_config.ini BepInEx/core/BepInEx.dll BepInEx/plugins/SkateBIRDTeleport.dll; do
    if [ -f "$LAB/$f" ]; then echo "ok   $f"; else echo "MISS $f"; bad=1; fi
  done
  return $bad
}

if [ "${1:-}" = "--check" ]; then
  check
  exit $?
fi

[ -f "$GAME/SkateBIRD.exe" ] || { echo "no SkateBIRD install at $GAME" >&2; exit 2; }
if tasklist //FI "IMAGENAME eq SkateBIRD.exe" | grep -qi skatebird; then
  echo "SkateBIRD is running; close it before changing the lab" >&2
  exit 2
fi
mkdir -p "$LAB" "$DL"

for f in "${ROOT_FILES[@]}"; do
  cmp -s "$GAME/$f" "$LAB/$f" 2>/dev/null || cp -f "$GAME/$f" "$LAB/$f"
done

for j in "${JUNCTIONS[@]}"; do
  t="$(junction_target "$LAB/$j")"
  if [ "$t" = "$(win "$GAME/$j")" ]; then continue; fi
  if [ -e "$LAB/$j" ]; then
    echo "$LAB/$j exists and is not a junction to the install; fix it by hand (rmdir only if it is a junction)" >&2
    exit 2
  fi
  powershell -NoProfile -Command \
    "New-Item -ItemType Junction -Path '$(win "$LAB/$j")' -Target '$(win "$GAME/$j")' | Out-Null"
done

zip="$DL/$(basename "$BEPINEX_URL")"
if [ ! -f "$zip" ] || [ "$(sha256sum "$zip" | cut -d' ' -f1)" != "$BEPINEX_SHA256" ]; then
  curl -fsSL -o "$zip.part" "$BEPINEX_URL"
  [ "$(sha256sum "$zip.part" | cut -d' ' -f1)" = "$BEPINEX_SHA256" ] || { echo "BepInEx sha256 mismatch" >&2; exit 3; }
  mv -f "$zip.part" "$zip"
fi
# The archive must not reach into the junctions.
if unzip -Z1 "$zip" | grep -qiE '^(SkateBIRD_Data|MonoBleedingEdge)(/|$)|\.\.'; then
  echo "refusing: $zip has entries under the junction folders" >&2
  exit 3
fi
[ -f "$LAB/BepInEx/core/BepInEx.dll" ] || unzip -q -o "$zip" -d "$LAB"

dotnet build "$(win "$PLUGIN/SkateBIRDTeleport.csproj")" -c Release -nologo -v q "-p:LabDir=$(win "$LAB")"
mkdir -p "$LAB/BepInEx/plugins"
cp -f "$PLUGIN/bin/Release/SkateBIRDTeleport.dll" "$LAB/BepInEx/plugins/"
check
