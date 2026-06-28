# Private/Animation/InvokeEngine.ps1
# Launch the forgum-engine binary with graceful shutdown semantics.
#
# **The fix for BUG-T3**: SIGTERM (Unix) / CloseMainWindow (Windows) is sent
# first, and only after a 2-second grace period is Kill() called as a last
# resort. Also raises the timeout from the buggy 30 s to duration+5 s so the
# watchdog never fires during a normal foreground animation.
#
# **The fix for BUG-D2**: the temp JSON file is removed in `finally`, and
# any `.err` companion is also cleaned up.

function Invoke-ForgumEngine {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [string] $EnginePath,
        [Parameter(Mandatory)] [string] $JsonFile,
        [ValidateSet('render', 'status')] [string] $Command = 'render',
        [switch] $Background,
        [int]    $DurationSeconds = 0,
        [int]    $TimeoutSeconds  = 0,
        [int]    $KillGracePeriodMs = 2000
    )

    if (-not (Test-Path -LiteralPath $EnginePath)) {
        throw "forgum-engine not found at: $EnginePath (set `$env:FORGUM_ENGINE or run install.ps1)"
    }

    # Derive timeout from duration if not explicit. Minimum 35 s for
    # foreground to give the user time to read the cow before SIGTERM.
    if ($TimeoutSeconds -le 0) {
        $TimeoutSeconds = [Math]::Max(35, $DurationSeconds + 5)
    }

    $argList = @($Command, '--file', "`"$JsonFile`"")
    if ($Background) {
        $argList += @('--background', '--duration', "$DurationSeconds")
    }

    $errFile = "$JsonFile.err"

    try {
        $proc = Start-Process -FilePath $EnginePath `
                              -ArgumentList $argList `
                              -NoNewWindow `
                              -PassThru `
                              -RedirectStandardError $errFile

        $exited = $proc.WaitForExit($TimeoutSeconds * 1000)

        if (-not $exited) {
            # Graceful shutdown first.
            try {
                if ($IsWindows) {
                    # CloseMainWindow sends WM_CLOSE to the main window. The
                    # engine's signal handler (SetConsoleCtrlHandler) won't
                    # see this; for full graceful shutdown we'd need to
                    # send CTRL_C_EVENT via GenerateConsoleCtrlEvent, but
                    # that's invasive from a child. CloseMainWindow is the
                    # best portable option.
                    $proc.CloseMainWindow() | Out-Null
                } else {
                    Stop-Process -Id $proc.Id -Signal SIGTERM -ErrorAction Stop
                }
                $proc.WaitForExit($KillGracePeriodMs) | Out-Null
            } catch {
                Write-Verbose "Forgum: graceful shutdown failed: $_"
            }

            # Last resort.
            if (-not $proc.HasExited) {
                try { $proc.Kill() } catch { }
            }

            # Force-restore the terminal no matter what.
            # ESC[?25h = show cursor, ESC[0m = reset attrs, ESC[?1049l = leave alt screen
            [Console]::Out.Write("`e[?25h`e[0m`e[?1049l`e[?25h")
        }

        return $proc.ExitCode
    } finally {
        # Clean up the temp files regardless of exit path.
        foreach ($f in @($JsonFile, $errFile)) {
            if (Test-Path -LiteralPath $f) {
                try { Remove-Item -LiteralPath $f -Force -ErrorAction Stop } catch { }
            }
        }
    }
}