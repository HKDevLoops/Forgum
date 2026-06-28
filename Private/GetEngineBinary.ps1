# Private/GetEngineBinary.ps1
# Resolve the path to the forgum-engine binary.
#
# Precedence:
#   1. $env:FORGUM_ENGINE (explicit override)
#   2. Module-relative bin/forgum-engine[.exe] (bundled)
#   3. forgum-engine on $env:PATH (system install)
#
# **The fix for BUG-P30**: we never auto-rebuild. If the binary is missing,
# we return a clear error pointing the user at install.ps1 / install.sh.

function Get-ForgumEngineBinary {
    [CmdletBinding()]
    [OutputType([string])]
    param()

    $exeName = if ($IsWindows) { 'forgum-engine.exe' } else { 'forgum-engine' }

    # 1. Explicit override.
    if ($env:FORGUM_ENGINE -and (Test-Path -LiteralPath $env:FORGUM_ENGINE)) {
        return (Resolve-Path -LiteralPath $env:FORGUM_ENGINE).Path
    }

    # 2. Module-relative.
    $moduleRoot = $PSScriptRoot | Split-Path -Parent
    $candidate = Join-Path (Join-Path $moduleRoot 'bin') $exeName
    if (Test-Path -LiteralPath $candidate) {
        return (Resolve-Path -LiteralPath $candidate).Path
    }

    # 3. System PATH.
    $onPath = Get-Command $exeName -ErrorAction SilentlyContinue
    if ($onPath) {
        return $onPath.Source
    }

    # Clear failure.
    throw "forgum-engine not found. Tried:`n" +
          "  - `$env:FORGUM_ENGINE = '$($env:FORGUM_ENGINE)'`n" +
          "  - $candidate`n" +
          "  - $exeName on PATH`n" +
          "Run install.ps1 / install.sh, or set `$env:FORGUM_ENGINE."
}