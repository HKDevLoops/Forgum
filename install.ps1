<#
.SYNOPSIS
    One-command celestial installer for Forgum.

.DESCRIPTION
    Detects Windows architecture, runs preflight terminal checks (TrueColor, UTF-8),
    detects existing package manager installations (Scoop, WinGet, Chocolatey, Cargo),
    reconciles conflicts interactively, installs forgum to %LOCALAPPDATA%\Forgum,
    ensures PATH integration, and launches the rich Celestial Terminal UI Wizard for interactive
    configuration, shell integration, channel selection, and transparent motivation telemetry consent.

.PARAMETER Version
    Release version to install. Defaults to the version in Cargo.toml at the
    current directory, then to the latest GitHub release.

.PARAMETER Channel
    Release channel stream: 'stable', 'nightly', or 'dev'. Defaults to 'stable'.

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
    ./install.ps1 -Channel nightly
    ./install.ps1 -Headless -Telemetry decline
#>
[CmdletBinding()]
param(
    [string] $Version,
    [ValidateSet('stable', 'nightly', 'dev', '')]
    [string] $Channel = 'stable',
    [string] $Repo = 'HKDevLoops/Forgum',
    [switch] $Headless,
    [ValidateSet('allow', 'decline', '')]
    [string] $Telemetry = '',
    [switch] $FirstRun
)

$ErrorActionPreference = 'Stop'

# --- banner -----------------------------------------------------------------
Write-Host "✦ FORGUM CELESTIAL INSTALLER ✦" -ForegroundColor Magenta
Write-Host "Idempotent, non-destructive installer with conflict reconciliation" -ForegroundColor DarkGray

# --- terminal preflight diagnostics -----------------------------------------
$hasTrueColor = $false
if ($env:WT_SESSION -or ($env:COLORTERM -in @('truecolor', '24bit')) -or ($env:TERM_PROGRAM -in @('vscode', 'warp', 'tabby', 'hyper', 'WezTerm', 'ghostty'))) {
    $hasTrueColor = $true
}

$hasUtf8 = $false
try {
    if ([Console]::OutputEncoding.CodePage -eq 65001 -or $env:WT_SESSION) {
        $hasUtf8 = $true
    }
} catch {
    $hasUtf8 = $true
}

Write-Host ">> Preflight Terminal Diagnostics:" -ForegroundColor Cyan
if ($hasTrueColor) {
    Write-Host "  [✓] 24-bit TrueColor display detected" -ForegroundColor Green
} else {
    Write-Host "  [!] TrueColor not detected (256-color fallback will be used)" -ForegroundColor Yellow
}
if ($hasUtf8) {
    Write-Host "  [✓] UTF-8 encoding active" -ForegroundColor Green
} else {
    Write-Host "  [!] Code page is not UTF-8 (Run 'chcp 65001' or use Windows Terminal for full cosmic art)" -ForegroundColor Yellow
}

# --- ffmpeg optional recording check ----------------------------------------
$hasFfmpeg = [bool](Get-Command ffmpeg -ErrorAction SilentlyContinue)
if ($hasFfmpeg) {
    Write-Host "  [✓] ffmpeg detected (mascot recording and video exports active)" -ForegroundColor Green
} else {
    Write-Host "  [-] ffmpeg not detected (optional)" -ForegroundColor DarkGray
    Write-Host "      Tip: Install via 'winget install Gyan.FFmpeg' or 'scoop install ffmpeg' to record animations." -ForegroundColor DarkGray
}

# --- install location -------------------------------------------------------
$installDir = if ($env:FORGUM_INSTALL_DIR) { $env:FORGUM_INSTALL_DIR } `
              else { Join-Path $env:LOCALAPPDATA 'Forgum' }
if (-not (Test-Path -LiteralPath $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}
$binPath = Join-Path $installDir 'forgum.exe'
$legacyPath = Join-Path $installDir 'forgum-engine.exe'

# --- detect existing package manager installations & shadow binaries --------
$existingInstallations = @()

# Check Scoop
$scoopShim = if ($env:SCOOP) { Join-Path $env:SCOOP 'shims\forgum.exe' } else { Join-Path $HOME 'scoop\shims\forgum.exe' }
if (Test-Path -LiteralPath $scoopShim) {
    $existingInstallations += [PSCustomObject]@{
        Manager = 'Scoop'
        Path = $scoopShim
        UpdateCmd = 'scoop update forgum'
    }
}

# Check WinGet
$wingetCmd = Get-Command winget -ErrorAction SilentlyContinue
if ($wingetCmd) {
    try {
        $wingetCheck = winget list --id HKDevLoops.Forgum 2>$null
        if ($wingetCheck -and $LASTEXITCODE -eq 0 -and ($wingetCheck | Select-String "HKDevLoops.Forgum")) {
            $existingInstallations += [PSCustomObject]@{
                Manager = 'WinGet'
                Path = 'WinGet Package (HKDevLoops.Forgum)'
                UpdateCmd = 'winget upgrade HKDevLoops.Forgum'
            }
        }
    } catch {}
}

# Check Chocolatey
$chocoCmd = Get-Command choco -ErrorAction SilentlyContinue
if ($chocoCmd) {
    try {
        $chocoCheck = choco list --local-only forgum 2>$null
        if ($chocoCheck -and ($chocoCheck | Select-String "1 packages installed")) {
            $existingInstallations += [PSCustomObject]@{
                Manager = 'Chocolatey'
                Path = 'Chocolatey Package (forgum)'
                UpdateCmd = 'choco upgrade forgum'
            }
        }
    } catch {}
}

# Check Cargo
$cargoBin = Join-Path $HOME '.cargo\bin\forgum.exe'
if (Test-Path -LiteralPath $cargoBin) {
    $existingInstallations += [PSCustomObject]@{
        Manager = 'Cargo'
        Path = $cargoBin
        UpdateCmd = 'cargo install --force forgum-cli'
    }
}

# Check on-PATH active binary
$activeCmd = Get-Command forgum -ErrorAction SilentlyContinue
if ($activeCmd -and $activeCmd.Source) {
    $isKnown = $false
    foreach ($inst in $existingInstallations) {
        if ($inst.Path -eq $activeCmd.Source) { $isKnown = $true; break }
    }
    if (-not $isKnown -and $activeCmd.Source -ne $binPath) {
        $existingInstallations += [PSCustomObject]@{
            Manager = 'PATH Shadow'
            Path = $activeCmd.Source
            UpdateCmd = "Replace $activeCmd.Source"
        }
    }
}

# --- interactive conflict reconciliation menu -------------------------------
if ($existingInstallations.Count -gt 0 -and -not $Headless) {
    Write-Host ""
    Write-Host "⚠ Package Manager Conflict Detected!" -ForegroundColor Yellow
    foreach ($inst in $existingInstallations) {
        Write-Host "  • Found existing Forgum via $($inst.Manager): $($inst.Path)" -ForegroundColor Cyan
    }
    Write-Host ""
    Write-Host "How would you like to proceed?" -ForegroundColor White
    Write-Host "  [1] Delegate to package manager (Run '$($existingInstallations[0].UpdateCmd)' and exit)" -ForegroundColor Green
    Write-Host "  [2] Switch to standalone channel build (Install to $installDir and update PATH)" -ForegroundColor Cyan
    Write-Host "  [3] Clean reinstall / overwrite" -ForegroundColor Yellow
    Write-Host ""
    
    $choice = Read-Host "Select option [1/2/3] (default: 2)"
    if (-not $choice) { $choice = '2' }
    
    if ($choice.Trim() -eq '1') {
        Write-Host ">> Delegating update to $($existingInstallations[0].Manager)..." -ForegroundColor Green
        Invoke-Expression $existingInstallations[0].UpdateCmd
        exit 0
    }
}

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
Write-Host ">> Installing Forgum $Version ($Channel channel) from $Repo ($Tag)" -ForegroundColor Cyan

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

# Check if local build binary exists first (fast developer install)
$localDebug = Join-Path $PWD 'target\debug\forgum.exe'
$localRelease = Join-Path $PWD 'target\release\forgum.exe'
if (Test-Path -LiteralPath $localRelease) {
    Copy-Item -LiteralPath $localRelease -Destination $binPath -Force
    Copy-Item -LiteralPath $localRelease -Destination $legacyPath -Force
    Write-Host ">> Installed from local release build: $binPath" -ForegroundColor Green
} elseif (Test-Path -LiteralPath $localDebug) {
    Copy-Item -LiteralPath $localDebug -Destination $binPath -Force
    Copy-Item -LiteralPath $localDebug -Destination $legacyPath -Force
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

        $extracted = Join-Path $tmp 'forgum.exe'
        if (-not (Test-Path -LiteralPath $extracted)) {
            $extracted = Join-Path $tmp 'forgum-engine.exe'
        }
        if (-not (Test-Path -LiteralPath $extracted)) {
            throw "forgum binary not found inside $asset"
        }

        Copy-Item -LiteralPath $extracted -Destination $binPath -Force
        Copy-Item -LiteralPath $extracted -Destination $legacyPath -Force
        Write-Host ">> Installed: $binPath" -ForegroundColor Green
    } catch {
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Write-Host ">> Release archive download unavailable. Compiling via Cargo..." -ForegroundColor Yellow
            if (Test-Path -LiteralPath (Join-Path $PWD 'Cargo.toml')) {
                cargo build --release --bin forgum --bin forgum-engine
                Copy-Item -LiteralPath 'target\release\forgum.exe' -Destination $binPath -Force
                Copy-Item -LiteralPath 'target\release\forgum.exe' -Destination $legacyPath -Force
            } else {
                cargo install forgum-cli
                $cargoBin = Join-Path $HOME '.cargo\bin\forgum.exe'
                if (Test-Path -LiteralPath $cargoBin) {
                    Copy-Item -LiteralPath $cargoBin -Destination $binPath -Force
                    Copy-Item -LiteralPath $cargoBin -Destination $legacyPath -Force
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
    $userPath = @($installDir) + $userPath
    [Environment]::SetEnvironmentVariable('PATH', ($userPath -join ';'), 'User')
    Write-Host ">> Prepended $installDir to User PATH" -ForegroundColor Green
} else {
    Write-Host ">> $installDir already on User PATH"
}
if ($installDir -notin ($env:PATH -split ';')) {
    $env:PATH = "$installDir;$env:PATH"
}

# --- record initial receipt if channel is specified -------------------------
try {
    $receiptDir = Join-Path $HOME '.config\forgum'
    if (-not (Test-Path -LiteralPath $receiptDir)) {
        New-Item -ItemType Directory -Path $receiptDir -Force | Out-Null
    }
    $receiptPath = Join-Path $receiptDir 'install_receipt.json'
    $epochSeconds = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $receiptContent = @"
{
  "installer_source": "standalone",
  "channel": "$Channel",
  "version": "$Version",
  "bin_path": "$($binPath -replace '\\', '\\')",
  "installed_at": $epochSeconds,
  "auto_update": true
}
"@
    Set-Content -Path $receiptPath -Value $receiptContent -Encoding UTF8 -Force
} catch {}

# --- launch wizard or headless setup ---------------------------------------
if ($Headless) {
    $telemetryArg = if ($Telemetry) { @("--telemetry", $Telemetry) } else { @() }
    & "$binPath" install --headless @telemetryArg
} else {
    # Launch celestial Terminal UI wizard
    & "$binPath" install
}
