# Private/GetDaemonStatePath.ps1
# Resolve per-session daemon state file path.
#
# The daemon state file stores the PID and optional control-socket path
# for a background daemon launched by the current shell session.

function Get-ForgumSessionId {
    [CmdletBinding()]
    [OutputType([string])]
    param()

    if ($env:TMUX_PANE) { return $env:TMUX_PANE }
    if ($env:ZELLIJ_SESSION_ID) { return $env:ZELLIJ_SESSION_ID }
    return "shell-$PID"
}

function Get-ForgumDaemonStatePath {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)] [string] $SessionId
    )

    if ($IsWindows -or $env:OS -eq 'Windows_NT') {
        $base = if ($env:LOCALAPPDATA) { $env:LOCALAPPDATA } else { Join-Path $env:USERPROFILE "AppData\Local" }
        $base = Join-Path $base "Forgum"
    } else {
        $base = if ($env:XDG_RUNTIME_DIR) { $env:XDG_RUNTIME_DIR } else { "/tmp" }
        $base = Join-Path $base "Forgum"
    }

    if (-not (Test-Path -LiteralPath $base)) {
        New-Item -ItemType Directory -Path $base -Force | Out-Null
    }

    return Join-Path $base "daemon-$SessionId.json"
}
