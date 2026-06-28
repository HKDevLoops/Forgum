# G11 — Get-ForgumEngineBinary: env override > module-relative > PATH.
# Critically, **no auto-rebuild** (BUG-P30).

BeforeAll {
    Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
    Initialize-ForgumPester
    Import-Module (Join-Path (Get-ForgumRepoRoot) 'Forgum.psd1') -Force
}

AfterAll { Cleanup-ForgumPester }

Describe 'Get-ForgumEngineBinary (G11)' {
    It 'honors $env:FORGUM_ENGINE when set' {
        $custom = Join-Path ([System.IO.Path]::GetTempPath()) ('forgum-engine-custom-' + [guid]::NewGuid() + '.exe')
        'fake-binary' | Set-Content -LiteralPath $custom -Encoding ascii
        try {
            $env:FORGUM_ENGINE = $custom
            (Get-ForgumEngineBinary) | Should -Be (Resolve-Path $custom).Path
        } finally {
            Remove-Item Env:FORGUM_ENGINE -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $custom -Force -ErrorAction SilentlyContinue
        }
    }

    It 'falls back to module bin/ when env is unset and PATH is empty' {
        Remove-Item Env:FORGUM_ENGINE -ErrorAction SilentlyContinue
        $binDir = Join-Path (Get-ForgumRepoRoot) 'bin'
        $dest = Join-Path $binDir ('forgum-engine' + $(if ($IsWindows) { '.exe' } else { '' }))
        if (-not (Test-Path -LiteralPath $binDir)) {
            New-Item -ItemType Directory -Path $binDir | Out-Null
        }
        if (-not (Test-Path -LiteralPath $dest)) {
            Copy-Item -LiteralPath (Get-ForgumEnginePath) -Destination $dest
        }
        try {
            Get-ForgumEngineBinary | Should -Be (Resolve-Path $dest).Path
        } finally {
            Remove-Item -LiteralPath $dest -Force -ErrorAction SilentlyContinue
        }
    }

    It 'throws a clear error when no binary can be found' {
        $env:FORGUM_ENGINE = Join-Path ([System.IO.Path]::GetTempPath()) ('nobinary-' + [guid]::NewGuid() + '.exe')
        $binDir = Join-Path (Get-ForgumRepoRoot) 'bin'
        $preserved = $null
        if (Test-Path -LiteralPath $binDir) {
            $preserved = @(Get-ChildItem -LiteralPath $binDir -Force -ErrorAction SilentlyContinue)
            Remove-Item -LiteralPath $binDir -Recurse -Force
        }
        try {
            # If it's still resolvable via PATH, we can't test this case.
            $resolveEnv = Get-Command 'forgum-engine' -ErrorAction SilentlyContinue
            if ($null -eq $resolveEnv) {
                { Get-ForgumEngineBinary } | Should -Throw -ExpectedMessage '*not found*'
            } else {
                Set-ItResult -Pending -Because 'forgum-engine is on PATH and would mask this test'
            }
        } finally {
            if ($preserved) {
                New-Item -ItemType Directory -Path $binDir -Force | Out-Null
                foreach ($f in $preserved) {
                    Copy-Item -LiteralPath $f.FullName -Destination (Join-Path $binDir $f.Name) -Force
                }
            }
            Remove-Item Env:FORGUM_ENGINE -ErrorAction SilentlyContinue
        }
    }

    It 'does NOT auto-rebuild when called from a non-build context' {
        $marker = Join-Path ([System.IO.Path]::GetTempPath()) ('forgum-build-marker-' + [guid]::NewGuid())
        $env:FORGUM_BUILD_MARKER = $marker
        $env:FORGUM_ENGINE = $marker  # nonexistent on purpose
        try {
            { Get-ForgumEngineBinary -ErrorAction SilentlyContinue } | Should -Throw
            Test-Path -LiteralPath $marker | Should -BeFalse -Because "must not have created the file"
        } finally {
            Remove-Item Env:FORGUM_ENGINE -ErrorAction SilentlyContinue
            Remove-Item Env:FORGUM_BUILD_MARKER -ErrorAction SilentlyContinue
        }
    }
}