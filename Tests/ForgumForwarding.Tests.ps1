# G-ForgumForwarding — the `forgum` wrapper forwards parameters to the engine.

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force

    $script:savedConfig = $env:FORGUM_CONFIG
    $script:tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-fwd-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $script:tmpDir -Force | Out-Null
    $env:FORGUM_CONFIG = Join-Path $script:tmpDir 'config.json'
}

AfterAll {
    if (Test-Path -LiteralPath $script:tmpDir) {
        Remove-Item -LiteralPath $script:tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    if ($script:savedConfig) { $env:FORGUM_CONFIG = $script:savedConfig }
    else { Remove-Item Env:FORGUM_CONFIG -ErrorAction SilentlyContinue }
    Cleanup-ForgumPester
}

Describe 'forgum wrapper forwards parameters (G-ForgumForwarding)' {
    It 'forgum -Cow tux invokes the engine without error' {
        { forgum -Cow tux -Text 'hello' } | Should -Not -Throw
    }

    It 'accepts -Effect, -Eyes, -Tongue, -Background, -Duration, -Fps' {
        { forgum -Cow tux -Text 'x' -Effect 'rain' -Eyes 'oo' -Tongue 'U' -Background -Duration 0 -Fps 24 } | Should -Not -Throw
    }

    It 'leaves no temp JSON files behind' {
        $before = Get-ChildItem -LiteralPath $env:TEMP -Filter 'tmp*.json' -ErrorAction SilentlyContinue
        forgum -Cow tux -Text 'cleanup check' | Out-Null
        $after = Get-ChildItem -LiteralPath $env:TEMP -Filter 'tmp*.json' -ErrorAction SilentlyContinue
        $after.Count | Should -Be $before.Count
    }
}
