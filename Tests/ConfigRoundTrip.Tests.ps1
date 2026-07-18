# G-ConfigRoundTrip — no-orphan-config invariant + engine config schema.
#
# Verifies that a config written via the module can be read back with
# consistent values, and that the engine's SceneConfig (deny_unknown_fields)
# accepts only the allowed keys and rejects stray keys.

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force

    $script:savedConfig = $env:FORGUM_CONFIG
    $script:tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-cfg-" + [guid]::NewGuid())
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

Describe 'Config round-trip (G-ConfigRoundTrip)' {
    It 'writes and reads back consistent values' {
        Set-ForgumConfig -Values @{ cow = 'tux'; effect = 'rain'; fps = 24 }
        $cfg = Get-ForgumConfig
        $cfg.cow    | Should -Be 'tux'
        $cfg.effect | Should -Be 'rain'
        $cfg.fps    | Should -Be 24
    }

    It 'engine rejects a config with an unknown key (exit 65)' {
        $stray = Join-Path $script:tmpDir 'stray.json'
        '{"cow":"default","bogus_field":42}' | Set-Content -LiteralPath $stray -Encoding utf8
        $code = Invoke-ForgumEngine -EnginePath (Get-ForgumEnginePath) `
                                     -JsonFile $stray `
                                     -Command status `
                                     -TimeoutSeconds 5
        $code | Should -Be 65
    }

    It 'engine accepts a config with only allowed keys (exit 0)' {
        $allowed = Join-Path $script:tmpDir 'allowed.json'
        $obj = [pscustomobject]@{
            cow        = 'default'
            text       = 'hi'
            effect     = 'static'
            background = $false
            duration   = 0
            fps        = 30
            eyes       = 'oo'
            tongue     = ' '
        }
        $obj | ConvertTo-Json -Compress | Set-Content -LiteralPath $allowed -Encoding utf8
        $code = Invoke-ForgumEngine -EnginePath (Get-ForgumEnginePath) `
                                     -JsonFile $allowed `
                                     -Command status `
                                     -TimeoutSeconds 5
        $code | Should -Be 0
    }
}
