# Forgum.PesterHelpers.psm1
# Shared helpers for Pester tests. Import this at the top of each
# *.Tests.ps1 file:
#
#     BeforeAll {
#         Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
#         Initialize-ForgumPester
#         Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force
#     }

$script:ForgumRepoRoot = $null
$script:ForgumBinPath = $null

function Initialize-ForgumPester {
    if ($script:ForgumRepoRoot) { return }
    $script:ForgumRepoRoot = (Resolve-Path "$PSScriptRoot/..").Path
    $script:ForgumBinPath = Join-Path $script:ForgumRepoRoot 'target' 'debug' 'forgum-engine.exe'

    if (-not (Test-Path -LiteralPath $script:ForgumBinPath)) {
        Write-Host "[ForgumPester] Building forgum-engine (one-time)..."
        Push-Location $script:ForgumRepoRoot
        try {
            & cargo build --quiet 2>&1 | Out-Null
        } finally {
            Pop-Location
        }
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
    }
    $env:FORGUM_ENGINE = $script:ForgumBinPath
}

function Get-ForgumEnginePath {
    if (-not $script:ForgumBinPath) {
        Initialize-ForgumPester
    }
    return $script:ForgumBinPath
}

function Get-ForgumRepoRoot {
    if (-not $script:ForgumRepoRoot) {
        Initialize-ForgumPester
    }
    return $script:ForgumRepoRoot
}

function Cleanup-ForgumPester {
    Remove-Item Env:FORGUM_ENGINE -ErrorAction SilentlyContinue
    Remove-Item Env:FORGUM_CONFIG -ErrorAction SilentlyContinue
}

Export-ModuleMember -Function Initialize-ForgumPester, Get-ForgumEnginePath, Get-ForgumRepoRoot, Cleanup-ForgumPester