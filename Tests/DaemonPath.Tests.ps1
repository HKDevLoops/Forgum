# G-DaemonPath — daemon state path + session id.

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force
}

AfterAll { Cleanup-ForgumPester }

Describe 'Get-ForgumDaemonStatePath / Get-ForgumSessionId' {
    It 'Get-ForgumSessionId returns a non-empty string' {
        $sid = Get-ForgumSessionId
        $sid | Should -Not -BeNullOrEmpty
        $sid | Should -BeOfType ([string])
    }

    It 'Get-ForgumDaemonStatePath returns a path ending in daemon-<session>.json' {
        $sid = Get-ForgumSessionId
        $path = Get-ForgumDaemonStatePath -SessionId $sid
        $path | Should -BeOfType ([string])
        $path | Should -Not -BeNullOrEmpty
        $leaf = Split-Path -Path $path -Leaf
        $leaf | Should -Be "daemon-$sid.json"
    }

    It 'Get-ForgumDaemonStatePath is deterministic for the same session' {
        $sid = 'test-session-123'
        $a = Get-ForgumDaemonStatePath -SessionId $sid
        $b = Get-ForgumDaemonStatePath -SessionId $sid
        $a | Should -Be $b
    }
}
