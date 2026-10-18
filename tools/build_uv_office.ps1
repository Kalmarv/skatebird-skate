$ErrorActionPreference = 'Continue'
Set-Location 'C:/Users/kalmarv/ds-modding'
$env:CARGO_TARGET_DIR = 'C:/Users/kalmarv/ds-modding/target'
cargo build --release --manifest-path program/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$data = 'C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD/SkateBIRD_Data'
$out = 'C:/Users/kalmarv/ds-modding/out/uv-fix-v2/level3'
if (Test-Path $out) { throw "Output already exists: $out" }
New-Item -ItemType Directory -Path $out | Out-Null
& './target/release/skatebird-export.exe' $data data/typetrees.tpk level3 "$out/unity" --surfaces data/skatebird-surfaces.json --spawn '-34.026,11.98,78.359,180.0' --travel data/skatebird-levels.json *> "$out/export.log"
if ($LASTEXITCODE -ne 0) { Get-Content "$out/export.log" -Tail 10; exit $LASTEXITCODE }
python tools/prove_uv_loss.py $out --expect-fixed
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& './target/release/examples/build_map.exe' "$out/unity/skate/scene" --out "$out/map" --name SkateBIRD_level3 *> "$out/build.log"
if ($LASTEXITCODE -ne 0) { Get-Content "$out/build.log" -Tail 10; exit $LASTEXITCODE }
& './tools/ReSkateStudio-20261012/reskate_cli.exe' compile-map './game' "$out/map/SkateBIRD_level3" "$out/stage" --mod-folder SkateBIRD_level3 --no-gi --time-of-day noon *> "$out/compile.log"
if ($LASTEXITCODE -ne 0) { Get-Content "$out/compile.log" -Tail 10; exit $LASTEXITCODE }
Write-Output "Office UV candidate compiled: $out"
