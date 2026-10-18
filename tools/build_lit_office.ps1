$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding'
$out = 'C:/Users/kalmarv/ds-modding/out/lighting-no-bam/level3'
$map = 'C:/Users/kalmarv/ds-modding/out/alpha-fix/level3/map/SkateBIRD_level3'
if (Test-Path $out) { throw "Output already exists: $out" }
New-Item -ItemType Directory -Path $out | Out-Null
python tools/hash_inputs.py "$out/input-hashes.json" "$map/map.json" './tools/ReSkateStudio-20261012/reskate_cli.exe'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
# Same map and Enlighten as b26f0559; only disable inherited BAM lighting.
$ErrorActionPreference = 'Continue'
& './tools/ReSkateStudio-20261012/reskate_cli.exe' compile-map './game' $map "$out/stage" --mod-folder SkateBIRD_level3 --time-of-day noon --enlighten --no-bam-lighting *> "$out/compile.log"
$compileExit = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($compileExit -ne 0) { Get-Content "$out/compile.log" -Tail 20; exit $compileExit }
Get-Content "$out/compile.log" -Tail 20
