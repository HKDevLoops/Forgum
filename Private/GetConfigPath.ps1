# Private/GetConfigPath.ps1
# Resolve the Forgum config path, honoring overrides.
#
# Precedence:
#   1. $env:FORGUM_CONFIG (explicit override)
#   2. Platform default: %APPDATA%\Forgum\config.json (Windows)
#                       $XDG_CONFIG_HOME/Forgum/config.json (Linux/macOS)
#
# **The fix for BUG-C3**: $env:FORGUM_CONFIG is honored everywhere.

function Get-ForgumConfigPath {
    [CmdletBinding()]
    [OutputType([string])]
    param()

    if ($env:FORGUM_CONFIG) {
        return $env:FORGUM_CONFIG
    }

    if ($IsWindows -or $PSVersionTable.PSEdition -eq 'Desktop') {
        $appdata = $env:APPDATA
        if (-not $appdata) {
            throw "Cannot resolve Forgum config: neither `$env:FORGUM_CONFIG nor `$env:APPDATA is set."
        }
        return (Join-Path $appdata 'Forgum' 'config.json')
    } else {
        $xdg = $env:XDG_CONFIG_HOME
        if ($xdg) {
            return (Join-Path $xdg 'Forgum' 'config.json')
        }
        $home = $env:HOME
        if (-not $home) {
            throw "Cannot resolve Forgum config: neither `$env:FORGUM_CONFIG nor `$env:XDG_CONFIG_HOME nor `$env:HOME is set."
        }
        return (Join-Path $home '.config' 'Forgum' 'config.json')
    }
}

function Initialize-ForgumConfig {
    [CmdletBinding()]
    param(
        [string] $Path
    )

    if (-not $Path) { $Path = Get-ForgumConfigPath }

    if (Test-Path -LiteralPath $Path) {
        return Get-Item -LiteralPath $Path
    }

    $parent = Split-Path -Path $Path -Parent
    if ($parent -and -not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    # Minimal default config. Phase 2 will flesh this out.
    $default = [pscustomobject]@{
        cow       = 'default'
        effect    = 'static'
        background = $false
        duration  = 0
        fps       = 30
        eyes      = 'oo'
        tongue    = ' '
        animation = [pscustomobject]@{
            mode    = 'static'
            lolcat  = $false
            palette = 'default'
        }
    }
    $default | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $Path -Encoding utf8
    return Get-Item -LiteralPath $Path
}

function Get-ForgumConfig {
    [CmdletBinding()]
    param(
        [string] $Path
    )

    if (-not $Path) { $Path = Get-ForgumConfigPath }

    if (-not (Test-Path -LiteralPath $Path)) {
        Initialize-ForgumConfig -Path $Path | Out-Null
    }
    return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}

function Set-ForgumConfig {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [hashtable] $Values,
        [string] $Path
    )

    if (-not $Path) { $Path = Get-ForgumConfigPath }

    if (-not (Test-Path -LiteralPath $Path)) {
        Initialize-ForgumConfig -Path $Path | Out-Null
    }
    $cfg = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    foreach ($k in $Values.Keys) {
        $cfg.$k = $Values[$k]
    }
    $cfg | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $Path -Encoding utf8
}