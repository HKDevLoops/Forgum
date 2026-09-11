<#>
.SYNOPSIS
    One-command celestial installer for Forgum.

.DESCRIPTION
    Detects the Windows architecture, installs forgum-engine and forgum into
    %LOCALAPPDATA%\Forgum, ensures the directory is on the current user's PATH,
    and launches the rich Celestial Terminal UI Wizard for interactive configuration,
    shell integration, and transparent motivation telemetry consent.

.PARAMETER Version
    Release version to install. Defaults to the version in Cargo.toml at the
    current directory, then to the latest GitHub release.

.PARAMETER Repo
    owner/name of the GitHub repo. Defaults to HKDevLoops/Forgum.

.PARAMETER Headless
    Perform non-interactive installation without launching the Celestial TUI.

.PARAMETER Telemetry
    In headless mode, 'allow' (yes) or 'decline' (no) motivation telemetry.

.PARAMETER FirstRun
    If set, prints first-run guidance after install.

.EXAMPLE
    ./install.ps1
    ./install.ps1 -Headless -Telemetry decline
#>
[CmdletBinding()]
param(
    [string] $Version,
    [string] $Repo = 'HKDevLoops/Forgum',
    [switch] $Headless,
    [ValidateSet('allow', 'decline', '')]
    [string] $Telemetry = '',
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
Write-Host ">> Installing Forgum $Version from $Repo ($Tag)" -ForegroundColor Cyan

# --- detect architecture ----------------------------------------------------
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { 'arm64' } `
        elseif ($env:PROCESSOR_ARCHITECTURE -eq 'AMD64') { 'x64' } `
        elseif ($env:PROCESSOR_ARCHITECTURE -eq 'x86') { 'x86' } `
        else { $env:PROCESSOR_ARCHITECTURE }

switch ($arch) {
    'x64'   { $asset = "forgum-$Version-windows-x64.zip" }
    'arm64' { $asset = "forgum-$Version-windows-arm64.zip" }
    'x86'   { $asset = "forgum-$Version-windows-x86.zip" }
    default {
        Write-Error "Unsupported Windows architecture '$arch'. Expected x64, arm64, or x86."
        exit 1
    }
}

# --- install location -------------------------------------------------------
$installDir = if ($env:FORGUM_INSTALL_DIR) { $env:FORGUM_INSTALL_DIR } `
              else { Join-Path $env:LOCALAPPDATA 'Forgum' }
if (-not (Test-Path -LiteralPath $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}
$binPath = Join-Path $installDir 'forgum-engine.exe'
$aliasPath = Join-Path $installDir 'forgum.exe'

# Check if local build binary exists first (fast developer install)
$localDebug = Join-Path $PWD 'target\debug\forgum.exe'
$localRelease = Join-Path $PWD 'target\release\forgum.exe'
if (Test-Path -LiteralPath $localRelease) {
    Copy-Item -LiteralPath $localRelease -Destination $binPath -Force
    Copy-Item -LiteralPath $localRelease -Destination $aliasPath -Force
    Write-Host ">> Installed from local release build: $binPath" -ForegroundColor Green
} elseif (Test-Path -LiteralPath $localDebug) {
    Copy-Item -LiteralPath $localDebug -Destination $binPath -Force
    Copy-Item -LiteralPath $localDebug -Destination $aliasPath -Force
    Write-Host ">> Installed from local debug build: $binPath" -ForegroundColor Green
} else {
    $url = "https://github.com/$Repo/releases/download/$Tag/$asset"
    Write-Host ">> Downloading $asset" -ForegroundColor Cyan

    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-" + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null
    $zipPath = Join-Path $tmp $asset

    try {
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing -Headers @{ 'User-Agent' = 'forgum-install' }
        if (-not (Test-Path -LiteralPath $zipPath)) {
            throw "Download failed: $url"
        }

        $shell = New-Object -ComObject Shell.Application
        $zipNs = $shell.NameSpace($zipPath)
        $destNs = $shell.NameSpace($tmp)
        $destNs.CopyHere($zipNs.Items(), 0x10)

        $extracted = Join-Path $tmp 'forgum-engine.exe'
        if (-not (Test-Path -LiteralPath $extracted)) {
            $extracted = Join-Path $tmp 'forgum.exe'
        }
        if (-not (Test-Path -LiteralPath $extracted)) {
            throw "forgum binary not found inside $asset"
        }

        Copy-Item -LiteralPath $extracted -Destination $binPath -Force
        Copy-Item -LiteralPath $extracted -Destination $aliasPath -Force
        Write-Host ">> Installed: $binPath" -ForegroundColor Green
    } catch {
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Write-Host ">> Release archive download unavailable. Compiling via Cargo..." -ForegroundColor Yellow
            if (Test-Path -LiteralPath (Join-Path $PWD 'Cargo.toml')) {
                cargo build --release --bin forgum-engine --bin forgum
                Copy-Item -LiteralPath 'target\release\forgum.exe' -Destination $binPath -Force
                Copy-Item -LiteralPath 'target\release\forgum.exe' -Destination $aliasPath -Force
            } else {
                cargo install forgum-cli
                $cargoBin = Join-Path $HOME '.cargo\bin\forgum.exe'
                if (Test-Path -LiteralPath $cargoBin) {
                    Copy-Item -LiteralPath $cargoBin -Destination $binPath -Force
                    Copy-Item -LiteralPath $cargoBin -Destination $aliasPath -Force
                }
            }
            Write-Host ">> Installed via Cargo: $binPath" -ForegroundColor Green
        } else {
            Write-Error "Failed to install Forgum: $_"
            exit 1
        }
    } finally {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# --- add to User PATH (idempotent) & activate in current session ------------
$userPath = [Environment]::GetEnvironmentVariable('PATH', 'User') -split ';' | Where-Object { $_ }
if ($installDir -notin $userPath) {
    $userPath += $installDir
    [Environment]::SetEnvironmentVariable('PATH', ($userPath -join ';'), 'User')
    Write-Host ">> Added $installDir to User PATH" -ForegroundColor Green
} else {
    Write-Host ">> $installDir already on User PATH"
}
if ($installDir -notin ($env:PATH -split ';')) {
    $env:PATH = "$installDir;$env:PATH"
}

# --- launch wizard or headless setup ---------------------------------------
if ($Headless) {
    $telemetryArg = if ($Telemetry) { @("--telemetry", $Telemetry) } else { @() }
    & "$binPath" install --headless @telemetryArg
} else {
    # Launch celestial Terminal UI wizard
    & "$binPath" install
}
