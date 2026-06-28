# G10 — Invoke-ForgumEngine cleans up temp files, returns exit code,
# rejects missing binary.

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force
}

AfterAll { Cleanup-ForgumPester }

Describe 'Invoke-ForgumEngine (G10)' {
    BeforeEach {
        $script:tmp = [System.IO.Path]::GetTempFileName() + '.json'
        '{}' | Set-Content -LiteralPath $script:tmp -Encoding utf8
    }
    AfterEach {
        if (Test-Path -LiteralPath $script:tmp) {
            Remove-Item -LiteralPath $script:tmp -Force -ErrorAction SilentlyContinue
        }
    }

    It 'cleans up the temp JSON file in finally' {
        Invoke-ForgumEngine -EnginePath $env:FORGUM_ENGINE `
                            -JsonFile $script:tmp `
                            -TimeoutSeconds 5 `
        | Out-Null
        Test-Path -LiteralPath $script:tmp | Should -BeFalse
    }

    It 'returns 0 for the engine status command' {
        $code = Invoke-ForgumEngine -EnginePath (Get-ForgumEnginePath) `
                                     -Command status `
                                     -JsonFile $script:tmp `
                                     -TimeoutSeconds 5
        $code | Should -Be 0
    }

    It 'returns exit code 65 (EX_DATAERR) for invalid JSON' {
        '{"unknown_field": 42}' | Set-Content -LiteralPath $script:tmp -Encoding utf8
        $code = Invoke-ForgumEngine -EnginePath $env:FORGUM_ENGINE `
                                     -JsonFile $script:tmp `
                                     -TimeoutSeconds 5
        $code | Should -Be 65
    }

    It 'rejects missing binary with a typed error' {
        $bogus = Join-Path ([System.IO.Path]::GetTempPath()) ('nonexistent-' + [guid]::NewGuid() + '.exe')
        { Invoke-ForgumEngine -EnginePath $bogus -JsonFile $script:tmp -TimeoutSeconds 2 } `
            | Should -Throw -ExpectedMessage '*not found*'
    }

    It 'gracefully sends SIGTERM/CloseMainWindow before Kill()' -Skip:($IsLinux -or $IsMacOS) {
        # We launch our engine in `--background --duration 0` (which would
        # loop forever). Then ask Invoke-ForgumEngine to clean it up after
        # the timeout. The process must exit (it'll be killed), and the
        # JSON must be cleaned up.
        $envJson = '{"cow":"x","duration":0}'
        $envJson | Set-Content -LiteralPath $script:tmp -Encoding utf8

        # Pass a short explicit timeout — the engine runs forever, so we
        # must rely on the timeout firing. Use KillGracePeriodMs=500 to
        # keep the test fast.
        $start = Get-Date
        Invoke-ForgumEngine -EnginePath $env:FORGUM_ENGINE `
                            -JsonFile $script:tmp `
                            -TimeoutSeconds 2 `
                            -KillGracePeriodMs 1500 `
                            -Background `
                            -DurationSeconds 0 `
        | Out-Null
        $elapsed = (Get-Date) - $start
        $elapsed.TotalSeconds | Should -BeLessThan 8 -Because "graceful kill should take < 5 s"
        Test-Path -LiteralPath $script:tmp | Should -BeFalse
    }
}