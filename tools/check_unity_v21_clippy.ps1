$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding/library-worktrees/unity-v21'
$env:CARGO_TARGET_DIR = 'C:/Users/kalmarv/ds-modding/library-targets/unity-v21'
git rev-parse HEAD
cargo fmt --all --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy -p primase-unity --test serialized_headers --test serialized_census -- -D warnings
exit $LASTEXITCODE
