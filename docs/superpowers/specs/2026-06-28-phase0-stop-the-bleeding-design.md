# Forgum Phase 0 — Stop the bleeding (spec)

> Status: Phase 0 of the v1 stabilization plan in `brain/00-README.md`, `brain/06-ARCHITECTURE.md` §2, and `brain/07-DEVELOPMENT-ROADMAP.md` §Phase 0.
> Bug IDs fixed: BUG-T1, T2, B1, B2, B9, E1, D4, D5, D7 (per `brain/01-BUGS-AND-ISSUES.md`).
> Source note: the spec is written against the audit's `file:line` references, not against a live `repo/` checkout (none exists on disk). When the upstream Forgum v1.1.2 source is dropped in, this spec is the contract that pins its behavior.

## 1. Goal

By the end of Phase 0:

1. **Terminal never corrupts** — panic, `SIGINT`, `SIGTERM`, `SIGHUP`, or the foreground `q` exit all leave raw mode off, alt screen off, cursor visible, overlay cleared.
2. **Background overlay does not steal keystrokes** — `event::poll`/`event::read` is physically absent from the background render loop.
3. **`duration=0` is infinite** — not 150 frames.
4. **CPU idle when nothing changes** — `Cell::dirty` no longer participates in the equality check, so a static cow goes to the idle scheduler tier.
5. **`stdin` is bounded** — 4 MB cap, returns `Err` and `exit(1)` otherwise.
6. **Malformed JSON exits non-zero** — callers can detect failure.
7. **`open_output` works** when stdout is piped (`/dev/tty` on Unix, `CONOUT$` on Windows).
8. **All `#[cfg]` lives in `forgum-platform`** — enforced by CI grep on `engine/src/`.
9. **PowerShell module** shims `InvokeEngine.ps1` to do SIGTERM-first kill + force-restore escape.

## 2. Architecture

```
Forgum/                            ← workspace root
├── Cargo.toml                     ← [workspace] members = ["crates/*"]
├── Cargo.lock
├── rust-toolchain.toml            ← stable channel pin
├── crates/
│   ├── platform/                  ← forgum-platform (all #[cfg] lives here)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs             ← re-exports + module docs
│   │   │   ├── error.rs           ← PlatformError enum (thiserror)
│   │   │   ├── paths.rs           ← ConfigPaths (XDG/Roaming/TEMP)
│   │   │   ├── terminal.rs        ← TerminalHandle trait + size()
│   │   │   ├── output.rs          ← OutputHandle (open_output: stdout | /dev/tty | CONOUT$)
│   │   │   ├── guards.rs          ← RawModeGuard, AltScreenGuard, CursorShowGuard
│   │   │   ├── signal.rs          ← SignalGuard (signal-hook unix, SetConsoleCtrlHandler win)
│   │   │   ├── spawn.rs           ← Detached command helpers (unix setsid, win DETACHED_PROCESS)
│   │   │   └── platform/
│   │   │       ├── unix.rs        ← cfg(unix) impls
│   │   │       └── windows.rs     ← cfg(windows) impls
│   │   └── tests/
│   │       └── guards_drop_on_panic.rs
│   └── engine/                    ← forgum-engine binary (zero #[cfg])
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs            ← CLI parse (std::env::args), dispatch, render loops
│       │   ├── config.rs          ← SceneConfig + merge: argv > file > defaults
│       │   ├── protocol.rs        ← JSON stdin/--file read with 4 MB cap
│       │   ├── render.rs          ← foreground + background render loops
│       │   ├── framebuffer.rs     ← Cell + FrameBuffer (manual PartialEq on ch/fg/bg/alpha)
│       │   ├── scheduler.rs       ← adaptive fps + idle detection
│       │   └── effects.rs         ← minimal stub: render "default" cow text
│       └── tests/
│           ├── cell_partial_eq.rs
│           ├── duration_zero.rs
│           ├── bounded_stdin.rs
│           ├── no_input_reads.rs  ← assert via grep on source
│           └── cfg_containment.rs ← assert zero #[cfg in engine/src
├── Forgum.psd1
├── Forgum.psm1
├── Public/
│   └── forgum.ps1
├── Private/
│   ├── Animation/
│   │   └── InvokeEngine.ps1       ← SIGTERM-first, raise timeout, force-restore
│   ├── GetEngineBinary.ps1        ← resolve path, no auto-rebuild
│   └── GetConfigPath.ps1
├── Tests/                         ← Pester
│   ├── InvokeEngine.Tests.ps1
│   ├── GetEngineBinary.Tests.ps1
│   └── GetConfigPath.Tests.ps1
├── .github/
│   └── workflows/
│       ├── ci.yml                 ← rust + pester + cfg-grep + version-parity
│       └── release.yml            ← placeholder for Phase 7
└── docs/
    └── superpowers/
        └── specs/
            └── 2026-06-28-phase0-stop-the-bleeding-design.md
```

**Invariant (CI-grep enforced):** `rg '#\[cfg' crates/engine/src/` returns zero hits. All platform branching is `forgum_platform::cfg_unix!()` / `cfg_windows!()` macro calls re-exported from `forgum-platform` (so the engine never sees `#[cfg]` literally, but the platform crate does). This matches `06-ARCHITECTURE.md` §2.

## 3. The `forgum-platform` crate contract

```rust
// crates/platform/src/lib.rs

pub mod error;
pub mod paths;
pub mod terminal;
pub mod output;
pub mod guards;
pub mod signal;
pub mod spawn;

// Re-exports for callers (engine, future herdr, etc.)
pub use error::PlatformError;
pub use paths::{ConfigPaths, config_path, data_dir, runtime_dir, log_dir};
pub use terminal::TerminalHandle;
pub use output::{OutputHandle, open_output};
pub use guards::{RawModeGuard, AltScreenGuard, CursorShowGuard};
pub use signal::{SignalGuard, ShutdownFlag};
pub use spawn::spawn_detached;

/// Macro: callers that need platform-specific code path use this instead of
/// writing `#[cfg(unix)]` themselves. The literal `#[cfg]` attribute is
/// confined to `forgum-platform` so the CI grep on `engine/src/` passes.
#[macro_export]
macro_rules! cfg_unix { ($($tt:tt)*) => { #[cfg(unix)] { $($tt)* } }; }
#[macro_export]
macro_rules! cfg_windows { ($($tt:tt)*) => { #[cfg(windows)] { $($tt)* } }; }
```

### 3.1 Guards (`crates/platform/src/guards.rs`)

All three guards restore terminal state on `Drop`, even on panic unwind. They are constructed unconditionally in the render loop's local scope; `Drop` order is reverse construction order (Rust guarantees LIFO).

```rust
pub struct RawModeGuard { /* opaque */ }
impl RawModeGuard {
    pub fn acquire() -> Result<Self, PlatformError>;   // calls crossterm::terminal::enable_raw_mode
}
impl Drop for RawModeGuard {
    fn drop(&mut self) { let _ = crossterm::terminal::disable_raw_mode(); }
}
// AltScreenGuard, CursorShowGuard: analogous, calling EnterAlternateScreen/Show on acquire,
// LeaveAlternateScreen/Hide on Drop (NOT the reverse — we leave alt screen, then show cursor).
```

The fix for **BUG-T2** is structural: `Terminal::Drop` is no longer the cleanup point — the guards are.

### 3.2 Signal handler (`crates/platform/src/signal.rs`)

```rust
pub struct ShutdownFlag(Arc<AtomicBool>);
impl ShutdownFlag {
    pub fn new() -> Self;
    pub fn is_shutdown(&self) -> bool { self.0.load(Ordering::Relaxed) }
    pub fn handle(&self) -> Arc<AtomicBool> { Arc::clone(&self.0) }
}
pub struct SignalGuard { /* holds registered handlers */ }
impl SignalGuard {
/// Unix: signal-hook registers SIGINT, SIGTERM, SIGHUP → flag.
/// Windows: SetConsoleCtrlHandler with a `HandlerRoutine` callback that sets the
///          flag for CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT,
///          CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT. The callback runs on the
///          kernel's console-handler thread — no polling thread is needed.
///          (Docs note: a `TRUE` return tells the kernel we handled it; for
///          CTRL_CLOSE/SHUTDOWN/LOGOFF we have no choice but to return `TRUE`
///          to keep the process alive long enough to set the flag.)
pub fn install(flag: ShutdownFlag) -> Result<Self, PlatformError>;
}
impl Drop for SignalGuard { /* deregisters */ }
```

The fix for **BUG-T1**: signal disposition is now graceful; render loop checks `flag.is_shutdown()` at the top of each iteration.

### 3.3 OutputHandle (`crates/platform/src/output.rs`)

```rust
pub struct OutputHandle { inner: Box<dyn Write + Send> }
impl OutputHandle {
    /// If stdout is a tty → stdout (wrapped in BufWriter).
    /// Else on unix: open /dev/tty O_WRONLY; on windows: open CONOUT$.
    /// Last resort: stdout (the engine prints a warning and `cow_text` once).
    pub fn open() -> Result<Self, PlatformError>;
}
impl Write for OutputHandle { /* delegate to inner */ }
```

This single helper fixes **BUG-B9** + **BUG-C1** (no more `cmd /c`).

### 3.4 Paths (`crates/platform/src/paths.rs`)

Honors `$FORGUM_CONFIG` and `$FORGUM_DATA` overrides (fixes **BUG-C3**).

| Platform | Config | Data | Runtime | Log |
|----------|--------|------|---------|-----|
| Linux    | `$XDG_CONFIG_HOME/Forgum/config.json` (default `~/.config/Forgum`) | `…/Forgum/cows/` | `$XDG_RUNTIME_DIR/Forgum/` (fallback `$TMPDIR/Forgum-<uid>/`) | `$XDG_STATE_HOME/Forgum/` |
| macOS    | `~/Library/Application Support/Forgum/config.json` | same | `$TMPDIR/Forgum-<uid>/` | `~/Library/Logs/Forgum/` |
| Windows  | `%APPDATA%\Forgum\config.json` | same | `%TEMP%\Forgum\` | `%LOCALAPPDATA%\Forgum\Logs\` |

Override precedence: explicit env var > platform default. Errors returned only if the *override* path is unusable (e.g., env var set to a path we can't write); missing default dirs are auto-created.

## 4. The `forgum-engine` binary

### 4.1 CLI surface (Phase 0, std::env::args; clap in Phase 2)

```
forgum-engine <command> [options]

Commands:
  render    Render a cow (foreground by default; --background to overlay above prompt)
  status    Print "ok" and exit (for daemon health checks — Phase 1)

Options (render):
  --cow <name>            Cow file basename (default "default"). Phase 0: only "default" works.
  --text <s>              Text inside the speech bubble.
  --effect <name>         Effect name (default "static"). Phase 0: only "static" works.
  --background            Render above prompt, run until signal/control.
  --duration <u32>        Seconds; 0 = infinite (BUG-B2 fix). Default 0 when --background.
  --fps <u16>             Target fps (default 30). 0 = idle-only.
  --config <path>         Override config path.
  --file <path>           Read scene JSON from file (alternative to stdin).
  --daemon                Detach and write PID file (Phase 0: stub; real in Phase 1).
  --control-socket <path> Phase 1; ignored in Phase 0 with a warning.
```

### 4.2 Main loop structure

```
fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();          // Phase 0: hand-rolled
    let config = Config::load(&args); // merge: argv > --file > --config > defaults
    let shutdown = ShutdownFlag::new();
    let _signals = SignalGuard::install(shutdown.handle())?;

    match args.command {
        Command::Render => {
            if args.background {
                render_loop_background(config, shutdown)?;
            } else {
                render_loop_foreground(config, shutdown)?;
            }
        }
        Command::Status => println!("ok"),
    }
    Ok(())
}
```

### 4.3 `render_loop_foreground`

```
let out = OutputHandle::open()?;
let _raw = RawModeGuard::acquire()?;
let _alt = AltScreenGuard::acquire()?;
let _cur = CursorShowGuard::acquire()?;

let mut fb = FrameBuffer::new(cols, rows);
let mut scheduler = Scheduler::new(fps);

loop {
    if shutdown.is_shutdown() { break; }
    let dt = scheduler.tick();
    fb.clear();
    effects::render_static_cow(&mut fb, &config.cow_text); // Phase 0 stub
    let damaged = fb.commit_damage();
    if !damaged.is_empty() {
        render_region(&mut out, &fb, &damaged)?;
    }
    scheduler.sleep_until_next();
}
// _cur, _alt, _raw drop in LIFO order → cursor visible, alt screen left, raw off
```

### 4.4 `render_loop_background` (the critical one)

Identical to foreground EXCEPT:

- Does **not** acquire `RawModeGuard` or `AltScreenGuard`.
- Does **not** call `event::poll` or `event::read` anywhere. **CI grep enforces this** (test `no_input_reads.rs` greps `crates/engine/src/render.rs`).
- Uses `SignalGuard::install` + `ShutdownFlag` for exit (BUG-T1, BUG-B1).
- Honors `max_frames = 0` as infinite (BUG-B2).
- Writes to the overlay region only (rows `0..ob_y1`). Phase 1 implements the strict clip; Phase 0 uses a single-screen stub that writes nothing destructive (just moves cursor + writes cow to `ob_y1..rows-1`).

### 4.5 Cell PartialEq fix (BUG-E1)

```rust
// crates/engine/src/framebuffer.rs

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub alpha: u8,
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.ch == other.ch && self.fg == other.fg && self.bg == other.bg && self.alpha == other.alpha
    }
}
impl Eq for Cell {}
// No `dirty` field. Damage = positions where `back != front`.
```

The `scheduler.idle()` transition fires when 15 consecutive frames have zero damage — so a static cow goes to the idle tier within ~0.25 s.

### 4.6 Bounded stdin (BUG-D4)

```rust
// crates/engine/src/protocol.rs

const MAX_INPUT: usize = 4 * 1024 * 1024; // 4 MB

pub fn read_scene(args: &Args) -> Result<SceneConfig, PlatformError> {
    let raw = if let Some(p) = &args.file {
        std::fs::read(p).map_err(PlatformError::Io)?
    } else {
        let mut buf = Vec::with_capacity(8 * 1024);
        std::io::stdin().take(MAX_INPUT as u64).read_to_end(&mut buf)
            .map_err(PlatformError::Io)?;
        if buf.len() >= MAX_INPUT {
            return Err(PlatformError::InputTooLarge { cap: MAX_INPUT });
        }
        buf
    };
    serde_json::from_slice(&raw).map_err(|e| PlatformError::Parse(e.to_string()))
}
```

### 4.7 Non-zero exit on parse error (BUG-D5)

`main()` wraps `read_scene`; on `PlatformError::Parse` or `InputTooLarge`, prints to stderr and `std::process::exit(1)`.

### 4.8 Saturating math (BUG-D7)

`let max_frames = (config.duration as u64).saturating_mul(fps as u64);` — no `u32` overflow possible.

## 5. The PowerShell module

### 5.1 `Private/Animation/InvokeEngine.ps1`

Replaces the buggy implementation referenced in **BUG-T3**:

```powershell
function Invoke-Engine {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [string] $EnginePath,
        [Parameter(Mandatory)] [string] $JsonFile,
        [switch] $Background,
        [int]    $DurationSeconds = 0,
        [int]    $TimeoutSeconds  = 0,        # 0 → derived from $DurationSeconds + 5
    )

    if ($TimeoutSeconds -le 0) { $TimeoutSeconds = [Math]::Max(35, $DurationSeconds + 5) }

    $argList = @('render', '--file', "`"$JsonFile`"")
    if ($Background) { $argList += @('--background', '--duration', "$DurationSeconds") }

    $proc = Start-Process -FilePath $EnginePath -ArgumentList $argList `
                          -NoNewWindow -PassThru -RedirectStandardError "$JsonFile.err"

    $exited = $proc.WaitForExit($TimeoutSeconds * 1000)
    if (-not $exited) {
        # Graceful first
        try {
            if ($IsWindows) { $proc.CloseMainWindow() | Out-Null }
            else            { Stop-Process -Id $proc.Id -Signal SIGTERM -ErrorAction Stop }
            $proc.WaitForExit(2000) | Out-Null
        } catch { }
        if (-not $proc.HasExited) { $proc.Kill() }     # last resort only
        # Force-restore terminal (BUG-T3 belt + braces)
        [Console]::Out.Write("`e[?25h`e[0m`e[?1049l`e[?25h")
    }

    Remove-Item -LiteralPath $JsonFile -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath "$JsonFile.err") { Remove-Item -LiteralPath "$JsonFile.err" -ErrorAction SilentlyContinue }
    return $proc.ExitCode
}
```

### 5.2 `Private/GetEngineBinary.ps1`

Resolves the engine binary by:
1. `$env:FORGUM_ENGINE` if set
2. Module-relative `bin/forgum-engine[.exe]`
3. `forgum-engine` on `$env:PATH`

**No auto-rebuild** (fixes **BUG-P30**). If the binary is missing, return a clear error.

### 5.3 `Public/forgum.ps1`

Phase 0 surface (the minimum needed to exercise the engine contract):

```powershell
function forgum {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromRemainingArguments=$true)] [string[]] $Text,
        [string] $Cow = 'default',
        [string] $Effect = 'static',
        [switch] $Background,
        [int]    $Duration = 0,
    )

    $engine = Get-EngineBinary
    $scene = @{ cow = $Cow; text = ($Text -join ' '); effect = $Effect
                background = [bool]$Background; duration = $Duration } | ConvertTo-Json -Compress
    $tmp = [System.IO.Path]::GetTempFileName() + '.json'
    try {
        Set-Content -LiteralPath $tmp -Value $scene -Encoding utf8
        Invoke-Engine -EnginePath $engine -JsonFile $tmp -Background:$Background -DurationSeconds $Duration
    } finally {
        Remove-Item -LiteralPath $tmp -ErrorAction SilentlyContinue
    }
}
```

## 6. Test gates (every one of these must pass before Phase 0 is "done")

| ID | Test | Bug it locks down |
|----|------|-------------------|
| G1 | `tests/guards_drop_on_panic.rs`: construct a `RawModeGuard`, force a `panic!()`, assert `crossterm::terminal::is_raw_mode_enabled() == false` in the catch_unwind block | T2 |
| G2 | `tests/signal_graceful_exit.rs`: spawn engine with `--background --duration 0`, send SIGTERM after 100 ms, assert process exits within 200 ms with exit code 0 | T1 |
| G3 | `tests/no_input_reads.rs`: literal `assert!(!source.contains("event::poll") && !source.contains("event::read"))` on `crates/engine/src/render.rs` | B1 |
| G4 | `tests/duration_zero.rs`: spawn engine with `--background --duration 0`, sleep 5 s, assert still running, then SIGTERM | B2 |
| G5 | `tests/cell_partial_eq.rs`: two `Cell`s with same `ch/fg/bg/alpha` are `==`; differing `ch` are `!=`. Also: a framebuffer of identical cells has `commit_damage()` returning empty | E1 |
| G6 | `tests/bounded_stdin.rs`: feed 5 MB to engine stdin, assert non-zero exit + stderr contains "too large" | D4 |
| G7 | `tests/parse_error_exit.rs`: feed `{ not json`, assert exit code 1 | D5 |
| G8 | `tests/cfg_containment.rs`: walk `crates/engine/src/**/*.rs`, assert no file contains `#[cfg` | arch |
| G9 | `tests/open_output.rs`: redirect engine stdout to a pipe (`./forgum-engine render --text hi > out.txt`), assert either `/dev/tty` was opened or `out.txt` contains the cow text + a warning | B9 |
| G10 | `Tests/InvokeEngine.Tests.ps1`: spawn a fake engine that sleeps 60 s; call `Invoke-Engine` with `TimeoutSeconds=2`; assert SIGTERM was sent (Unix) / `CloseMainWindow` was called (Windows) before `Kill()`, and that the force-restore escape was emitted | T3 |
| G11 | `Tests/GetEngineBinary.Tests.ps1`: `$env:FORGUM_ENGINE` overrides everything; missing binary returns a clear error (no auto-rebuild) | P30 |
| G12 | `Tests/GetConfigPath.Tests.ps1`: `$env:FORGUM_CONFIG` overrides the platform default | C3 |

CI configuration: `.github/workflows/ci.yml` runs (a) `cargo test` on `ubuntu-latest` + `windows-latest`, (b) `cargo clippy -- -D warnings`, (c) `cargo fmt --check`, (d) Pester on both OSes, (e) the cfg-grep gate, (f) a `version-parity` job (Phase 0 stub: only checks that `Cargo.toml` has a version field).

## 7. Dependencies (Cargo.toml pin)

Workspace `Cargo.toml`:
- `resolver = "2"`
- `members = ["crates/*"]`
- `[workspace.package]`: `edition = "2024"`, `rust-version = "1.85"`, `license = "MIT"`, `version = "0.4.0"`

`crates/platform/Cargo.toml` runtime deps:
- `crossterm = "0.28"` (raw mode + alt screen + cursor; pure Rust, musl-safe)
- `signal-hook = "0.3"` (unix)
- `windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_System_Console"] }` (windows)

`crates/engine/Cargo.toml` runtime deps:
- `forgum-platform = { path = "../platform" }`
- `serde = { version = "1", features = ["derive"] }`
- `serde_json = "1"`
- `crossterm = "0.28"` (only for ANSI emit helpers — raw mode/alt screen calls go through `forgum-platform`)
- `thiserror = "1"`

Dev deps (workspace):
- `proptest = "1"`
- `tempfile = "3"`

Dev deps (engine):
- `assert_cmd = "2"` (CLI testing)

## 8. Out of scope for Phase 0 (deferred to later phases)

| Item | Phase | Notes |
|------|-------|-------|
| `clap` CLI | 2 | Hand-rolled `std::env::args` is sufficient for Phase 0 |
| Cow file loading (`.cow` parser, `$eyes/$tongue` expansion, speech bubble wrapping) | 2 | Phase 0 emits the literal text "hello\n<\\/>" for the "default" cow |
| Real effects (aurora, plasma, particle pool, etc.) | 3 | Phase 0 has only `effects::render_static_cow` |
| `include_dir!` cow bundling | 2 | — |
| Resize handling (`Event::Resize` / `SIGWINCH`) | 1 | Phase 0 still responds to SIGWINCH via `signal-hook` and re-queries terminal size — but the re-layout logic is "clear + redraw" |
| Daemon detach (`setsid`, PID file, control socket) | 1 | Phase 0 `--daemon` is a stub that prints "phase 1" and exits |
| Prebuilt binaries / installers / Homebrew / Scoop / winget | 7 | — |
| Multiplexer integration (tmux/zellij/wezterm/screen) | 5 | — |
| Herder / remote | 6 | — |
| `forgum init <shell>` hook generator | 2 | — |
| Tier-2/3 E2E under `xvfb-run tmux` | 1 (Tier-3) | Phase 0 ships Tier-1 unit + a Tier-2 integration test (G9 uses `assert_cmd`) |

## 9. Definition of done

- [ ] All 12 test gates (G1–G12) green on `ubuntu-latest` and `windows-latest`.
- [ ] `cargo build --release` produces `target/release/forgum-engine` on both OSes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] `pwsh -NoProfile -Command "Invoke-Pester Tests/ -CI"` exits 0 on both OSes.
- [ ] Manual smoke (Unix): `./target/release/forgum-engine render --text hi` exits 0; same with `--background --duration 0` then `kill -TERM` exits 0 and leaves terminal pristine.
- [ ] Manual smoke (Windows): same in PowerShell 7.
- [ ] The CI workflow file is committed at `.github/workflows/ci.yml` and is syntactically valid (CI dry-run on the next push will catch issues).
- [ ] No `#[cfg` literal anywhere in `crates/engine/src/` (G8 passes).
- [ ] `rg 'event::(poll|read)' crates/engine/src/` returns zero hits (G3 passes).
- [ ] This spec is committed to git.

## 10. Open questions (resolved by Phase 1 kickoff)

1. Should `Render` foreground mode have a default duration? Plan §Phase 0 says foreground is `q`/SIGTERM; current audit says it has a default. Decision deferred: Phase 0 foreground runs until SIGINT or `--duration` expires, whichever first.
2. Should the control-socket stub be `--control-socket <path>` even when ignored? Yes — keeps the CLI stable for Phase 1.
3. Does `--background` need `--duration 0` to be the default? Yes (BUG-B2 fix only makes sense if the obvious case is "run forever"); we enforce `if --background && !--duration { duration = 0 }` at parse time.