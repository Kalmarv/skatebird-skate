$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding'
python tools/prove_uv_loss.py C:/Users/kalmarv/ds-modding/out/uv-fix/level3 --expect-fixed
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$env:BIRD_MOD_SOURCE = 'C:/Users/kalmarv/ds-modding/out/uv-fix/level3/stage/Patch'
& 'C:/Program Files/Git/bin/bash.exe' tools/run_skate.sh levels/game/dingolevel_reskate_skatebird_level3/dingolevel_reskate_skatebird_level3 C:/Users/kalmarv/ds-modding/out/uv-fix/run_level3 60 SkateBIRD_level3 1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
python tools/compare_shots.py out/uv-fix/compare_level3.png out/skatebird_office_spawn.png out/uv-fix/run_level3/shot_1.png
exit $LASTEXITCODE
