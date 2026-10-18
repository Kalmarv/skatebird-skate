$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding/library-worktrees/unity-v21'
$env:CARGO_TARGET_DIR = 'C:/Users/kalmarv/ds-modding/library-targets/unity-v21'
$env:PRIMASE_UNITY_V21_DATA = 'C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD/SkateBIRD_Data'
$env:PRIMASE_UNITY_V21_CENSUS = 'C:/Users/kalmarv/ds-modding/data/skatebird-census.json'
git rev-parse HEAD
cargo fmt --all --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p primase-unity --test serialized_headers --test serialized_census -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy -p primase-unity --test serialized_headers --test serialized_census -- -D warnings
exit $LASTEXITCODE
