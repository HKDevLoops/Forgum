# Public/forgum.ps1
# Main entry point for the Forgum PowerShell module.
#
# Usage:
#   forgum hello world               # foreground
#   forgum -Background -Duration 0   # background until signal
#   forgum -Cow tux -Text "hi" -Background
#
# In Phase 0 this is a thin shell-out to the Rust engine. Phase 2 will add
# native cow rendering for the PowerShell-only path.

function forgum {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromRemainingArguments = $true, Position = 0)]
        [string[]] $Text,

        [string] $Cow     = 'default',
        [string] $Effect  = 'static',
        [switch] $Background,
        [int]    $Duration = 0,
        [int]    $Fps     = 30,
        [string[]] $Eyes   = @('oo'),
        [string[]] $Tongue = @(' ')
    )

    $engine = Get-ForgumEngineBinary
    $joinedText = if ($Text) { ($Text -join ' ') } else { '' }

    $scene = @{
        cow        = $Cow
        text       = $joinedText
        effect     = $Effect
        background = [bool]$Background
        duration   = $Duration
        fps        = $Fps
        eyes       = ($Eyes   -join '')
        tongue     = ($Tongue -join '')
    }

    $json = $scene | ConvertTo-Json -Compress -Depth 5
    $tmp = [System.IO.Path]::GetTempFileName() + '.json'

    try {
        # UTF-8 with no BOM. PowerShell 5.1 would default to UTF-16; on 7+
        # this is fine. The engine accepts either, but we standardize.
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($tmp, $json, $utf8NoBom)

        $exitCode = Invoke-ForgumEngine -EnginePath $engine `
                                         -JsonFile $tmp `
                                         -Background:$Background `
                                         -DurationSeconds $Duration

        if ($exitCode -ne 0) {
            Write-Warning "forgum-engine exited with code $exitCode"
        }
    } finally {
        if (Test-Path -LiteralPath $tmp) {
            try { Remove-Item -LiteralPath $tmp -Force -ErrorAction Stop } catch { }
        }
    }
}