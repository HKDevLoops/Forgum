# G-StopDaemon — Stop-ForgumDaemon handles a stale PID file gracefully.

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force

    $script:savedConfig = $env:FORGUM_CONFIG
    $script:tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-stop-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $script:tmpDir -Force | Out-Null
    $env:FORGUM_CONFIG = Join-Path $script:tmpDir 'config.json'

    # Create a daemon state file pointing at a PID that is not running.
    # Stop-ForgumDaemon derives the session via Get-ForgumSessionId, so we
    # use the same real session id here.
    $script:sid = Get-ForgumSessionId
    $script:daemonPath = Get-ForgumDaemonStatePath -SessionId $script:sid
    $stale = [pscustomobject]@{ pid = 999999; socket_path = $null }
    $stale | ConvertTo-Json | Set-Content -LiteralPath $script:daemonPath -Encoding utf8
}

AfterAll {
    if (Test-Path -LiteralPath $script:daemonPath) {
        Remove-Item -LiteralPath $script:daemonPath -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $script:tmpDir) {
        Remove-Item -LiteralPath $script:tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    if ($script:savedConfig) { $env:FORGUM_CONFIG = $script:savedConfig }
    else { Remove-Item Env:FORGUM_CONFIG -ErrorAction SilentlyContinue }
    Cleanup-ForgumPester
}

Describe 'Stop-ForgumDaemon with stale PID (G-StopDaemon)' {
    It 'does not throw when the PID process is not running' {
        { Stop-ForgumDaemon -ErrorAction Stop } | Should -Not -Throw
    }

    It 'removes the stale state file after cleanup' {
        Stop-ForgumDaemon -ErrorAction SilentlyContinue
        Test-Path -LiteralPath $script:daemonPath | Should -BeFalse
    }
}
