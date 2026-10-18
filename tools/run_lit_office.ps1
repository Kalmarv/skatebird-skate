$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding'
$root = 'C:/Users/kalmarv/ds-modding/out/lighting-no-bam'
$env:BIRD_MOD_SOURCE = "$root/level3/stage/Patch"
& 'C:/Program Files/Git/bin/bash.exe' tools/run_skate.sh levels/game/dingolevel_reskate_skatebird_level3/dingolevel_reskate_skatebird_level3 "$root/run_level3" 60 SkateBIRD_level3 1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
if (!(Test-Path "$root/run_level3/shot_1.png")) { throw 'Capture missing' }
if (!(Select-String -LiteralPath "$root/run_level3/ReSkate.log" -SimpleMatch 'the level itself is active. Level: Levels/Game/DingoLevel_Root/DingoLevel_Root + levels/game/dingolevel_reskate_skatebird_level3/' -Quiet)) { throw 'Target active state missing' }
python tools/compare_shots.py "$root/compare_level3.png" out/skatebird_office_spawn.png "$root/run_level3/shot_1.png"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
python tools/compare_shots.py "$root/gi_control_level3.png" out/lighting-enlighten/run_level3/shot_1.png "$root/run_level3/shot_1.png"
exit $LASTEXITCODE
