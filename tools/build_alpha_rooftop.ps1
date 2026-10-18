$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding/primase'
$env:CARGO_TARGET_DIR = 'C:/Users/kalmarv/ds-modding/target'
git rev-parse HEAD
cargo build --release -p primase-unity --example unity_export -p primase-reskate --example build_map
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Set-Location 'C:/Users/kalmarv/ds-modding'
$env:BIRD_OUTPUT_ROOT = 'C:/Users/kalmarv/ds-modding/out/alpha-fix'
& 'C:/Program Files/Git/bin/bash.exe' tools/export_level.sh level2
exit $LASTEXITCODE

