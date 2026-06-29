# Forgum PowerShell module loader.
#
# This file is intentionally minimal. All real logic lives in Public/ and
# Private/. We dot-source the public functions and the private helpers.

# Dot-source private helpers in a deterministic order.
$privateFiles = @(
    'Private\GetEngineBinary.ps1',
    'Private\GetConfigPath.ps1',
    'Private\GetDaemonStatePath.ps1',
    'Private\Animation\InvokeEngine.ps1'
)

foreach ($file in $privateFiles) {
    $path = Join-Path $PSScriptRoot $file
    if (Test-Path -LiteralPath $path) {
        . $path
    } else {
        Write-Warning "Forgum: missing private file: $file"
    }
}

# Dot-source public functions.
$publicFiles = Get-ChildItem -LiteralPath (Join-Path $PSScriptRoot 'Public') -Filter '*.ps1' -ErrorAction SilentlyContinue
foreach ($file in $publicFiles) {
    . $file.FullName
}

# Set strict mode for safety.
Set-StrictMode -Version Latest

# Module loaded successfully.