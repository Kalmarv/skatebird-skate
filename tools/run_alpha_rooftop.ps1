$ErrorActionPreference = 'Stop'
Set-Location 'C:/Users/kalmarv/ds-modding'
$env:BIRD_MOD_SOURCE = 'C:/Users/kalmarv/ds-modding/out/alpha-fix/level2/stage/Patch'
& 'C:/Program Files/Git/bin/bash.exe' tools/run_skate.sh levels/game/dingolevel_reskate_skatebird_level2/dingolevel_reskate_skatebird_level2 C:/Users/kalmarv/ds-modding/out/alpha-fix/run_level2 60 SkateBIRD_level2 1
exit $LASTEXITCODE

