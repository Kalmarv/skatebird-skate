$ErrorActionPreference = 'Continue'
python tools/native_stderr_fixture.py 0 *> 'out/native-stderr-preflight.log'
$nativeExit = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($nativeExit -ne 0) { throw "Success fixture failed: $nativeExit" }
if (!(Select-String -LiteralPath 'out/native-stderr-preflight.log' -SimpleMatch 'native-progress' -Quiet)) { throw 'Progress missing' }
$ErrorActionPreference = 'Continue'
python tools/native_stderr_fixture.py 7 *> 'out/native-stderr-failure.log'
$nativeExit = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($nativeExit -ne 7) { throw "Failure code lost: $nativeExit" }
Write-Output 'Native stderr captured; exit 0 succeeds, exit 7 remains failure.'
