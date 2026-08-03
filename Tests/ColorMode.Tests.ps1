# B4 — color_mode E2E: verify that color_mode "none" suppresses
# rainbow ANSI sequences in engine output.

Describe 'color_mode end-to-end (B4)' {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot 'Forgum.PesterHelpers.psm1') -Force
        Initialize-ForgumPester
    }

    AfterAll { Cleanup-ForgumPester }

    It 'produces no truecolor sequences when color_mode is none' {
        $engine = Get-ForgumEnginePath
        $tmpJson = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-b4-" + [guid]::NewGuid() + ".json")
        $tmpOut = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-b4-out-" + [guid]::NewGuid() + ".txt")
        $tmpErr = Join-Path ([System.IO.Path]::GetTempPath()) ("forgum-b4-err-" + [guid]::NewGuid() + ".txt")

        $scene = @{
            cow        = 'default'
            text       = 'color test'
            effect     = 'static'
            color_mode = 'none'
            background = $false
            duration   = 1
            fps        = 30
            eyes       = 'oo'
            tongue     = ' '
        } | ConvertTo-Json -Compress

        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($tmpJson, $scene, $utf8NoBom)

        try {
            $proc = Start-Process -FilePath $engine `
                                  -ArgumentList "render", '--file', "`"$tmpJson`"" `
                                  -NoNewWindow `
                                  -PassThru `
                                  -RedirectStandardOutput $tmpOut `
                                  -RedirectStandardError $tmpErr
            $proc.WaitForExit(10000) | Out-Null

            if (Test-Path -LiteralPath $tmpOut) {
                $output = [System.IO.File]::ReadAllText($tmpOut)
                # \x1b[38;2;R;G;B m is truecolor rainbow sequence
                $output | Should Not Match '\x1b\[38;2;'
            }
        } finally {
            foreach ($f in @($tmpJson, $tmpOut, $tmpErr)) {
                if (Test-Path -LiteralPath $f) {
                    try { Remove-Item -LiteralPath $f -Force -ErrorAction Stop } catch { }
                }
            }
        }
    }
}
