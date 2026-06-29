# Public/Stop-ForgumDaemon.ps1
# Stop a running Forgum daemon for the current session.

function Stop-ForgumDaemon {
    <#
    .SYNOPSIS
        Stops a running Forgum daemon for the current session.
    .DESCRIPTION
        Reads daemon.json, sends the stop signal, and cleans up state files.
    #>
    [CmdletBinding()]
    param()

    $session = Get-ForgumSessionId
    $daemonPath = Get-ForgumDaemonStatePath -SessionId $session

    if (-not (Test-Path -LiteralPath $daemonPath)) {
        Write-Verbose "No daemon state file found at $daemonPath"
        return
    }

    try {
        $daemon = Get-Content -Raw -LiteralPath $daemonPath | ConvertFrom-Json
    } catch {
        Write-Warning "Failed to read daemon state: $_"
        Remove-Item -LiteralPath $daemonPath -Force -ErrorAction SilentlyContinue
        return
    }

    if ($null -eq $daemon.pid) {
        Write-Warning "Daemon state has no PID"
        Remove-Item -LiteralPath $daemonPath -Force -ErrorAction SilentlyContinue
        return
    }

    # Check if process is still running.
    $proc = Get-Process -Id $daemon.pid -ErrorAction SilentlyContinue
    if ($null -eq $proc) {
        Write-Verbose "Daemon PID $($daemon.pid) is not running"
        Remove-Item -LiteralPath $daemonPath -Force -ErrorAction SilentlyContinue
        return
    }

    # Stop the process.
    try {
        Stop-Process -Id $daemon.pid -Force -ErrorAction Stop
        Write-Verbose "Stopped daemon PID $($daemon.pid)"
    } catch {
        Write-Warning "Failed to stop daemon PID $($daemon.pid): $_"
    }

    # Clean up state file.
    Remove-Item -LiteralPath $daemonPath -Force -ErrorAction SilentlyContinue

    # Clean up control socket (Unix).
    if ($daemon.socket_path) {
        Remove-Item -LiteralPath $daemon.socket_path -Force -ErrorAction SilentlyContinue
    }
}
