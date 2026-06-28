# G12 — Get-ForgumConfigPath honors $env:FORGUM_CONFIG; Initialize/Set/Get
# round-trip correctly and reject path traversal (security).

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force
}

AfterAll { Cleanup-ForgumPester }

Describe 'Get-ForgumConfigPath (G12)' {
    It 'returns the override when $env:FORGUM_CONFIG is set' {
        $custom = '/tmp/forgum-test-explicit-config.json'
        $env:FORGUM_CONFIG = $custom
        try {
            (Get-ForgumConfigPath) | Should -Be $custom
        } finally {
            Remove-Item Env:FORGUM_CONFIG -ErrorAction SilentlyContinue
        }
    }

    It 'falls back to a platform-appropriate default' {
        Remove-Item Env:FORGUM_CONFIG -ErrorAction SilentlyContinue
        $path = Get-ForgumConfigPath
        $path | Should -Not -BeNullOrEmpty
        if ($IsWindows) {
            $path | Should -Match 'Forgum'
        } else {
            $path | Should -Match 'Forgum'
        }
    }
}

Describe 'Initialize/Set/Get-ForgumConfig' {
    BeforeEach {
        $script:configPath = Join-Path ([System.IO.Path]::GetTempPath()) ('forgum-it-' + [guid]::NewGuid() + '.json')
        $env:FORGUM_CONFIG = $script:configPath
        Remove-Item -LiteralPath $script:configPath -ErrorAction SilentlyContinue
    }
    AfterEach {
        Remove-Item -LiteralPath $script:configPath -ErrorAction SilentlyContinue
        Remove-Item Env:FORGUM_CONFIG -ErrorAction SilentlyContinue
    }

    It 'creates a default config when none exists' {
        Test-Path -LiteralPath $script:configPath | Should -BeFalse
        Initialize-ForgumConfig | Out-Null
        Test-Path -LiteralPath $script:configPath | Should -BeTrue
        $cfg = Get-Content -LiteralPath $script:configPath -Raw | ConvertFrom-Json
        $cfg.cow | Should -Be 'default'
        $cfg.effect | Should -Be 'static'
        $cfg.fps   | Should -Be 30
    }

    It 'round-trips user preferences via Set-ForgumConfig' {
        Set-ForgumConfig -Values @{ cow = 'tux'; fps = 60 } | Out-Null
        $cfg = Get-ForgumConfig
        $cfg.cow | Should -Be 'tux'
        $cfg.fps | Should -Be 60
    }
}