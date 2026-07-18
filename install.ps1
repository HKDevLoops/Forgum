<#>
.SYNOPSIS
    One-command installer for forgum-engine (Phase 7).

.DESCRIPTION
    Detects the Windows architecture, downloads the correct prebuilt
    forgum-<version>-windows-<arch>.zip from the GitHub releases page (tag
    derived from Cargo.toml, or override with -Version), extracts
    forgum-engine.exe into a sensible location, and adds that location to the
    current user's PATH.

.PARAMETER Version
    Release version to install (e.g. 0.4.0). Defaults to the version in
    Cargo.toml at the current directory, then to the latest GitHub release.

.PARAMETER Repo
    owner/name of the GitHub repo. Defaults to HKDEVS/forgum.

.PARAMETER FirstRun
    If set, prints first-run guidance after install.

.EXAMPLE
    ./install.ps1
    ./install.ps1 -Version 0.4.0
#>
[CmdletBinding()]
param(
    [string] $Version,
    [string] $Repo = 'HKDEVS/forgum',
    [switch] $FirstRun
)

$ErrorActionPreference = 'Stop'

# --- resolve version --------------------------------------------------------
if (-not $Version) {
    if (Test-Path -LiteralPath (Join-Path $PWD 'Cargo.toml')) {
        $versionLine = (Get-Content -LiteralPath (Join-Path $PWD 'Cargo.toml') -Raw) `
            -split "`n" | Where-Object { $_ -match '^version' } | Select-Object -First 1
        if ($versionLine -match '"([^"]+)"') { $Version = $Matches[1] }
    }
}
if (-not $Version) {
    # Fall back to the latest GitHub release tag.
    try {
        $latest = (Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" `
            -Headers @{ 'User-Agent' = 'forgum-install' }).tag_name
        $Version = $latest.TrimStart('v')
    } catch {
        Write-Error "Could not determine version. Pass -Version, or run from a repo with Cargo.toml. $_"
        exit 1
    }
}

$Tag = "v$Version"
Write-Host ">> Installing forgum-engine $Version from $Repo ($Tag)"

# --- detect architecture ----------------------------------------------------
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { 'arm64' } `
        elseif ($env:PROCESSOR_ARCHITECTURE -eq 'AMD64') { 'x64' } `
        else { $env:PROCESSOR_ARCHITECTURE }

switch ($arch) {
    'x64'   { $asset = "forgum-$Version-windows-x64.zip" }
    'arm64' { $asset = "forgum-$Version-windows-arm64.zip" }
    default {
        Write-Error "Unsupported Windows architecture '$arch'. Expected x64 or arm64."
        exit 1
    }
}

$url = "https://github.com/$Repo/releases/download/$Tag/$asset"
Write-Host ">> Downloading $asset"

# --- install location -------------------------------------------------------
$installDir = if ($env:FORGUM_INSTALL_DIR) { $env:FORGUM_INSTALL_DIR } `
              else { Join-Path $env:LOCALAPPDATA 'Forgum' }
if (-not (Test-Path -LiteralPath $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}
$binPath = Join-Path $installDir 'forgum-engine.exe'

# --- download + extract (no external deps beyond Invoke-WebRequest/Shell) ----
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
$zipPath = Join-Path $tmp $asset

try {
    Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing -Headers @{ 'User-Agent' = 'forgum-install' }
    if (-not (Test-Path -LiteralPath $zipPath)) {
        Write-Error "Download failed: $url"
        exit 1
    }

    # Use the Shell.Application COM object so we don't depend on Expand-Archive
    # quirks; it handles the zip natively on all supported Windows builds.
    $shell = New-Object -ComObject Shell.Application
    $zipNs = $shell.NameSpace($zipPath)
    $destNs = $shell.NameSpace($tmp)
    $destNs.CopyHere($zipNs.Items(), 0x10)

    $extracted = Join-Path $tmp 'forgum-engine.exe'
    if (-not (Test-Path -LiteralPath $extracted)) {
        Write-Error "forgum-engine.exe not found inside $asset"
        exit 1
    }

    Copy-Item -LiteralPath $extracted -Destination $binPath -Force
    Write-Host ">> Installed: $binPath"
} finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

# --- add to User PATH (idempotent) -----------------------------------------
$userPath = [Environment]::GetEnvironmentVariable('PATH', 'User') -split ';' | Where-Object { $_ }
if ($installDir -notin $userPath) {
    $userPath += $installDir
    [Environment]::SetEnvironmentVariable('PATH', ($userPath -join ';'), 'User')
    Write-Host ">> Added $installDir to User PATH (reopen your terminal to use 'forgum-engine')"
} else {
    Write-Host ">> $installDir already on User PATH"
}

# --- next steps -------------------------------------------------------------
Write-Host
Write-Host "Done. Reopen your terminal, then try: forgum-engine --help"
if ($FirstRun) {
    Write-Host "First run: forgum-engine --daemon start"
}
