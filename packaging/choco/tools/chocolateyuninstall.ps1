# chocolateyuninstall.ps1 - Chocolatey uninstaller for Forgum
$ErrorActionPreference = 'Continue'
$packageName = 'forgum'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

# Execute forgum uninstaller to cleanly excise shell hooks and completions
$engine = Join-Path $toolsDir 'forgum-engine.exe'
if (-not (Test-Path -LiteralPath $engine)) {
    $engine = Join-Path $toolsDir 'forgum.exe'
}
if (Test-Path -LiteralPath $engine) {
    & "$engine" uninstall --method soft --yes
}

# Remove unpacked directory
Remove-Item -LiteralPath $toolsDir -Recurse -Force -ErrorAction SilentlyContinue
