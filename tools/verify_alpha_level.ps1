param([Parameter(Mandatory=$true)][string]$Level, [ValidateSet('export','capture','both')][string]$Mode = 'both')
$ErrorActionPreference = 'Stop'
if ($Level -notin @('level1','level2','level3','level4','level5','level7','level12','level13')) { throw 'Unknown level' }
Set-Location 'C:/Users/kalmarv/ds-modding'
$root = 'C:/Users/kalmarv/ds-modding/out/alpha-fix'
$env:BIRD_OUTPUT_ROOT = $root
if ($Mode -ne 'capture') {
    & 'C:/Program Files/Git/bin/bash.exe' tools/export_level.sh $Level "SkateBIRD_$Level" noinstall
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
if ($Mode -eq 'export') { exit 0 }
$env:BIRD_MOD_SOURCE = "$root/$Level/stage/Patch"
& 'C:/Program Files/Git/bin/bash.exe' tools/run_skate.sh "levels/game/dingolevel_reskate_skatebird_$Level/dingolevel_reskate_skatebird_$Level" "$root/run_$Level" 60 "SkateBIRD_$Level" 1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
if (!(Test-Path "$root/run_$Level/shot_1.png")) { throw 'Capture missing' }
if (!(Select-String -LiteralPath "$root/run_$Level/ReSkate.log" -SimpleMatch "the level itself is active. Level: Levels/Game/DingoLevel_Root/DingoLevel_Root + levels/game/dingolevel_reskate_skatebird_$Level/" -Quiet)) { throw 'Target active state missing' }
$names = @{level1='bedroom_1'; level2='rooftop'; level3='office'; level4='server_room'; level5='bedroom_2'; level7='flashback'; level12='playground'; level13='skate_heaven'}
python tools/compare_shots.py "$root/compare_$Level.png" "out/skatebird_$($names[$Level])_spawn.png" "$root/run_$Level/shot_1.png"
exit $LASTEXITCODE
