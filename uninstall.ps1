<#>
.SYNOPSIS
    One-command celestial uninstaller for Forgum.

.DESCRIPTION
    Safely de-orbits Forgum from your environment with two distinct methods:
    Method 1: Soft Uninstall (Default)
      - Removes binary from disk and PATH
      - Strips shell hooks and completions from profiles (PowerShell, Pwsh, Bash, Zsh, Fish, Nushell)
      - Deletes completions directory (~/.config/forgum/completions)
      - PRESERVES user configuration (~/.config/forgum) and custom cows
    Method 2: Purge Uninstall
      - Complete clean slate removal
      - Deletes binary, PATH, shell hooks, completions
      - PERMANENTLY deletes configuration, cache, logs, daemons, and state

.PARAMETER Method
    Uninstallation mode: 'Soft' (default) or 'Purge'.

.PARAMETER Yes
    Bypass interactive confirmation prompt.

.PARAMETER Tui
    Launch the interactive Celestial De-Orbit TUI wizard.

.EXAMPLE
    ./uninstall.ps1
    ./uninstall.ps1 -Method Purge -Yes
#>
[CmdletBinding()]
param(
    [ValidateSet('Soft', 'Purge')]
    [string] $Method = 'Soft',
    [switch] $Yes,
    [switch] $Tui
)

$ErrorActionPreference = 'Stop'

# Locate binary if available
$installDir = if ($env:FORGUM_INSTALL_DIR) { $env:FORGUM_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Forgum' }
$binPath = Join-Path $installDir 'forgum.exe'
if (-not (Test-Path -LiteralPath $binPath)) {
    $binPath = Join-Path $installDir 'forgum-engine.exe'
}

# If TUI requested or interactive without -Yes, launch interactive TUI wizard if binary exists
$isInteractive = [Environment]::UserInteractive -and -not [Console]::IsInputRedirected
if (($Tui -or (-not $Yes)) -and $isInteractive -and (Test-Path -LiteralPath $binPath)) {
    & "$binPath" uninstall --tui
    exit $LASTEXITCODE
}

Write-Host "━━━ Forgum De-Orbit Uninstaller ━━━" -ForegroundColor Cyan
Write-Host "Method: $Method" -ForegroundColor Yellow

if (-not $Yes) {
    $confirm = Read-Host "Are you sure you want to proceed with $Method uninstallation? [y/N]"
    if ($confirm -notmatch '^[Yy]') {
        Write-Host "Uninstallation cancelled." -ForegroundColor Yellow
        exit 0
    }
}

# If binary is available, delegate to engine uninstaller for complete platform-native de-orbit
if (Test-Path -LiteralPath $binPath) {
    & "$binPath" uninstall --method "$Method" --yes
    exit $LASTEXITCODE
}

# Fallback: Binary was already removed, perform manual script-based cleanup across all shells
# 1. Clean shell profiles
$profilePaths = @(
    $PROFILE.CurrentUserCurrentHost,
    $PROFILE.CurrentUserAllHosts,
    (Join-Path $HOME '.bashrc'),
    (Join-Path $HOME '.zshrc'),
    (Join-Path $env:APPDATA 'fish\config.fish'),
    (Join-Path $env:APPDATA 'nushell\config.nu')
)

foreach ($p in $profilePaths) {
    if ($p -and (Test-Path -LiteralPath $p)) {
        try {
            $content = Get-Content -LiteralPath $p -Raw
            $origLen = $content.Length
            
            # Remove all variants of forgum delimited blocks
            $patterns = @(
                '(?ms)# >>> forgum >>>.*?# <<< forgum <<<[\r\n]*',
                '(?ms)# >>> forgum completions >>>.*?# <<< forgum completions <<<[\r\n]*',
                '(?ms)# >>> forgum \(.*?\) >>>.*?# <<< forgum <<<[\r\n]*',
                '(?ms)rem >>> forgum \(cmd\) >>>.*?rem <<< forgum \(cmd\) <<<[\r\n]*'
            )
            foreach ($pat in $patterns) {
                $content = $content -replace $pat, ''
            }
            if ($content.Length -ne $origLen) {
                Set-Content -LiteralPath $p -Value $content.TrimEnd() -NoNewline
                Write-Host "✓ Removed hooks from $p" -ForegroundColor Green
            }
        } catch {
            Write-Warning "Could not update profile $p: $_"
        }
    }
}

# 2. Remove completions directory
$compDir = Join-Path $HOME '.config\forgum\completions'
if (Test-Path -LiteralPath $compDir) {
    Remove-Item -LiteralPath $compDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "✓ Removed completions directory $compDir" -ForegroundColor Green
}

# 3. Remove from User PATH
$currPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($currPath) {
    $parts = $currPath -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne $installDir.TrimEnd('\') }
    [Environment]::SetEnvironmentVariable('PATH', ($parts -join ';'), 'User')
    Write-Host "✓ Removed $installDir from User PATH" -ForegroundColor Green
}

# 4. Remove binaries
if (Test-Path -LiteralPath $binPath) {
    try {
        Remove-Item -LiteralPath $binPath -Force
        Write-Host "✓ Deleted $binPath" -ForegroundColor Green
    } catch {
        # If open, rename and schedule delete
        $delPath = "$binPath.del"
        Move-Item -LiteralPath $binPath -Destination $delPath -Force -ErrorAction SilentlyContinue
        Start-Process -FilePath "cmd.exe" -ArgumentList "/C ping 127.0.0.1 -n 2 > nul & del /F /Q `"$delPath`"" -WindowStyle Hidden
        Write-Host "✓ Scheduled deletion of $binPath" -ForegroundColor Green
    }
}
$aliasExe = Join-Path $installDir 'forgum.exe'
if (Test-Path -LiteralPath $aliasExe) {
    Remove-Item -LiteralPath $aliasExe -Force -ErrorAction SilentlyContinue
}

# 5. Method handling: Soft vs Purge
if ($Method -eq 'Purge') {
    Write-Host ">> Executing Purge: Deleting all user configuration, logs, and cache..." -ForegroundColor Yellow
    $dirsToPurge = @(
        $installDir,
        (Join-Path $env:APPDATA 'Forgum'),
        (Join-Path $HOME '.config\forgum'),
        (Join-Path $env:LOCALAPPDATA 'Forgum')
    )
    foreach ($d in $dirsToPurge) {
        if (Test-Path -LiteralPath $d) {
            Remove-Item -LiteralPath $d -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "✓ Permanently deleted $d" -ForegroundColor Green
        }
    }
    Write-Host "`n✓ Purge complete. Zero trace of Forgum remains." -ForegroundColor Green
} else {
    Write-Host "`n★ Soft uninstall complete." -ForegroundColor Green
    Write-Host "✓ Preserved user configuration in ~/.config/forgum/ so your custom settings are saved if you reinstall." -ForegroundColor Cyan
}
