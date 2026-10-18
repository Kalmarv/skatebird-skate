$ErrorActionPreference = 'Continue'
Set-Location 'C:/Users/kalmarv/ds-modding'
$env:CARGO_TARGET_DIR = 'C:/Users/kalmarv/ds-modding/target'
cargo build --release --manifest-path program/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$env:BIRD_UV_DIAGNOSTIC = '1'
& './target/release/skatebird-export.exe' 'C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD/SkateBIRD_Data' data/typetrees.tpk level3 out/uv-diagnostic --surfaces data/skatebird-surfaces.json *> out/uv-diagnostic.log
$result = $LASTEXITCODE
Get-Content out/uv-diagnostic.log -Tail 12
exit $result
