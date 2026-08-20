# Public/forgum.ps1
# Main entry point for the Forgum PowerShell module.
#
# Auto-detects OS, shell, terminal capabilities. Proxies engine subcommands
# directly. Renders cows with env-aware defaults when no subcommand given.
#
# Usage:
#   forgum hello world               # foreground
#   forgum -Cow tux -Text "hi" -Background
#   forgum init pwsh                 # proxy to engine
#   forgum fortune                   # proxy to engine
#   forgum config set cow tux        # proxy to engine
#   forgum status                    # proxy to engine

function forgum {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromRemainingArguments = $true, Position = 0)]
        [string[]] $Text,

        [string] $Cow       = 'default',
        [string] $Effect    = 'static',
        [string] $ColorMode = '',
        [switch] $Background,
        [int]    $Duration  = 0,
        [int]    $Fps       = 30,
        [string[]] $Eyes    = @('oo'),
        [string[]] $Tongue  = @(' ')
    )

    $engine = Get-ForgumEngineBinary

    # --- Subcommand proxy ---
    # Known engine subcommands that should be forwarded directly, not
    # treated as cow text.
    $engineSubcommands = @(
        'init', 'fortune', 'config', 'status', 'completions',
        'tmux', 'status-line', 'herd', 'theme', 'demo', 'showcase',
        'remote', 'say', 'timer', 'battle', 'render', 'logs', 'doctor', 'checkhealth', 'health', 'help'
    )

    if ($Text -and $Text.Count -gt 0 -and $engineSubcommands -contains $Text[0]) {
        # Proxy entire arg list verbatim to forgum-engine (subcommand + its args).
        & $engine @Text
        return
    }

    # --- Environment detection ---
    $isWindows  = $PSVersionTable.Platform -eq 'Win32NT' -or
                  (-not (Test-Path Variable:\IsWindows) -or $IsWindows)
    $isSSH      = $env:SSH_CLIENT -or $env:SSH_TTY -or $env:SSH_CONNECTION
    $isCI       = $env:CI -or $env:GITHUB_ACTIONS -or $env:TF_BUILD -or $env:JENKINS_URL
    $isTmux     = $env:TMUX -or $env:TMUX_PANE
    $isScreen   = $env:STY
    $isWezTerm  = $env:TERM_PROGRAM -eq 'WezTerm' -or $env:WEZTERM_PANE
    $isKitty    = $env:TERM -eq 'xterm-kitty' -or $env:KITTY_PID
    $isSixel    = $env:TERM -match 'sixel' -or $env:KITTY_WINDOW_ID
    $hasTrueColor = $env:COLORTERM -eq 'truecolor' -or $env:COLORTERM -eq '24bit'

    # Detect interactive vs piped.
    $isInteractive = [Console]::IsInputRedirected -eq $false -and
                     [Console]::IsOutputRedirected -eq $false

    # --- Smart defaults ---
    # Background rendering only in interactive terminals; piped output
    # falls back to static to avoid garbled escape sequences.
    if (-not $isInteractive -and -not $Background) {
        # Pipe/redirect detected — force static, no background.
        $Effect = 'static'
    }

    # SSH/tmux: prefer lower FPS to reduce bandwidth/latency.
    if ($isSSH -or $isTmux -or $isScreen) {
        if ($Fps -eq 30) { $Fps = 15 }
    }

    # No true color: downgrade rainbow effects to static.
    if (-not $hasTrueColor -and $Effect -eq 'rainbow') {
        $Effect = 'static'
    }

    # CI environments: static only, short duration.
    if ($isCI) {
        $Effect   = 'static'
        $Duration = 3
    }

    # --- Render ---
    $joinedText = if ($Text) { ($Text -join ' ') } else { '' }

    # Resolve color_mode: explicit param > saved config > default "rainbow".
    if (-not $ColorMode) {
        $configPath = if ($env:FORGUM_CONFIG) { $env:FORGUM_CONFIG }
                      else { Join-Path $env:APPDATA "Forgum\config.json" }
        $ColorMode = 'rainbow'
        if (Test-Path -LiteralPath $configPath) {
            try {
                $savedCfg = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
                if ($savedCfg.color_mode) { $ColorMode = $savedCfg.color_mode }
            } catch { }
        }
    }

    $scene = @{
        cow        = $Cow
        text       = $joinedText
        effect     = $Effect
        color_mode = $ColorMode
        background = [bool]$Background
        duration   = $Duration
        fps        = $Fps
        eyes       = ($Eyes   -join '')
        tongue     = ($Tongue -join '')
    }

    $json = $scene | ConvertTo-Json -Compress -Depth 5
    $tmp = [System.IO.Path]::GetTempFileName() + '.json'

    try {
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
