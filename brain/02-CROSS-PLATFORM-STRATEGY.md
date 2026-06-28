# Forgum — Cross-Platform Strategy

> **Goal:** Forgum must run, render, and exit cleanly on **Windows 10/11, macOS (Intel + Apple Silicon), and Linux (x86_64 + aarch64)**, inside **PowerShell 5.1, PowerShell 7+, bash, zsh, fish**, and under **tmux, zellij, GNU screen, wezterm, Windows Terminal, iTerm2, Alacritty, kitty, ConHost**. No "works on my machine."

This document defines the platform matrix, the abstraction boundaries, the build/distribution plan, the runtime detection logic, and the CI verification gates.

---

## 1. Platform matrix

| Axis | Tier-1 (must work, CI-gated) | Tier-2 (best effort, CI-informed) |
|------|------------------------------|-----------------------------------|
| **OS** | Windows 11, macOS 14+, Ubuntu 22.04/24.04, Debian 12, Fedora 40, Arch | FreeBSD, Alpine (musl), WSL2, Windows 10 |
| **Arch** | x86_64, aarch64 (Apple Silicon + Linux ARM) | armv7, riscv64 |
| **Shell** | pwsh 7, bash 5, zsh 5.8, fish 3 | pwsh 5.1 (Windows-only), bash 3.2 (macOS default) |
| **Terminal** | Windows Terminal, iTerm2, Alacritty, kitty, gnome-terminal, tmux 3.3+ | ConHost, Terminology, xterm, screen, wezterm |
| **Multiplexer** | tmux 3.3+ | zellij, GNU screen, wezterm-mux |
| **PowerShell** | 7.4 LTS | 7.2 (EOL soon), 5.1 (Windows-only fallback) |

**Rule:** a change is not "done" until Tier-1 is green in CI. Tier-2 regressions block a release only if they're Tier-1 regressions in disguise.

---

## 2. Abstraction boundaries (where the platform splits live)

The single biggest source of cross-platform bugs today is platform logic **scattered** across `main.rs`, `CrossPlatform.ps1`, `InvokeEngine.ps1`, and the shell hooks. We centralize it.

```
┌─────────────────────────────────────────────────────────────┐
│  Platform-abstraction crate:  forgum-platform  (new)         │
│  ───────────────────────────────────────────────────────     │
│  • trait TerminalHandle  { size, write_raw, open_tty, … }    │
│  • trait SignalHandler    { on_shutdown(F) }                  │
│  • trait ProcessSpawn     { detach(cmd) -> Child }            │
│  • fn config_path()  -> PathBuf   (XDG / Windows Known Folder)│
│  • fn runtime_dir()  -> PathBuf   (XDG_RUNTIME_DIR / %TEMP%)  │
│  • fn shell()        -> ShellKind (bash/zsh/fish/pwsh/… )     │
│  • fn multiplexer()  -> MuxKind    (none/tmux/zellij/screen)  │
│                                                              │
│  impls:  unix.rs · windows.rs · macos.rs (FSEvents if needed)│
└─────────────────────────────────────────────────────────────┘
        ▲                              ▲
        │                              │
   forgum-engine                Forgum PowerShell module
   (Rust binary)                (thin wrapper, no platform ifs)
```

**Principle:** the PowerShell module should contain **zero** `if ($IsWindows)` outside of `CrossPlatform.ps1`. The Rust engine should contain **zero** `#[cfg(windows)]` outside `forgum-platform`. Everything else programs against traits.

---

## 3. The five cross-platform seams (and how each is handled)

### Seam 1 — Terminal size & TTY detection

| Platform | API | Notes |
|----------|-----|-------|
| Unix | `ioctl(STDOUT, TIOCGWINSZ)` via crossterm → `crossterm::terminal::size()` | Fails when stdout is a pipe → fall back to `ioctl(/dev/tty)`. |
| Windows | `GetConsoleScreenBufferInfo` on `CONOUT$` | Fails under `head`/pipe → open `CONOUT$` directly. |

**Detection rule (replaces the dead `console_out`):**
```rust
fn open_output() -> Box<dyn Write> {
    if io::stdout().is_terminal() {
        return Box::new(io::BufWriter::new(io::stdout()));
    }
    #[cfg(unix)]    { if let Ok(f) = fs::OpenOptions::new().write(true).open("/dev/tty") { return Box::new(io::BufWriter::new(f)); } }
    #[cfg(windows)] { if let Some(f) = open_console_out() { return Box::new(io::BufWriter::new(f)); } }
    Box::new(io::BufWriter::new(io::stdout())) // last resort: pipe (no animation)
}
```
This single helper kills BUG-B9, BUG-C1, and the `cmd /c` workaround.

### Seam 2 — Raw mode & terminal-state restoration

- **Unix:** `tcgetattr/tcsetattr` via crossterm. RAII guard restores on drop (BUG-T2). `stty sane` is the manual escape hatch — document it.
- **Windows:** `SetConsoleMode` to enable `VIRTUAL_TERMINAL_INPUT` + disable `ENABLE_ECHO_INPUT`/`LINE_INPUT` for raw mode. Restore on drop. `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on the output handle for ANSI.

**Guard stack (always created in this order, dropped in reverse):**
```rust
let _sig = SignalGuard::install();           // BUG-T1
let _out = OutputHandle::open();             // Seam 1
let _raw = RawModeGuard;                     // foreground only
let _alt = AltScreenGuard;                   // foreground only
let _cur = CursorShowGuard;
```
A `panic` (unwind) or a caught signal runs all `Drop`s → terminal always restored. `SIGKILL` is the only uncatchable case; for that, the next shell prompt should detect a stale state via a sentinel and run `reset`.

### Seam 3 — Process detachment (the daemon)

| Platform | Detach recipe |
|----------|---------------|
| **Unix** | `pre_exec(|| { libc::setsid(); libc::umask(0); libc::close(STDIN); Ok(()) })`; redirect stdout/stderr to a log file or `/dev/null`. Child survives parent shell exit. |
| **Windows** | `CommandExt::creation_flags(DETACHED_PROCESS \| CREATE_NEW_PROCESS_GROUP)`. Redirect handles. Child survives console close only if not in the console's process group — `CREATE_NEW_PROCESS_GROUP` helps for Ctrl events. |

Write the PID to `runtime_dir()/forgum-<session>.pid`. `session` = `tmux session id` if inside tmux, else the parent shell PID. This lets `StopDaemon` target one instance (BUG-D3).

### Seam 4 — Signals

| Signal | Unix meaning | Windows equivalent | Engine action |
|--------|--------------|--------------------|---------------|
| `SIGINT` | Ctrl+C | `CTRL_C_EVENT` | graceful shutdown (foreground) |
| `SIGTERM` | `kill` | `Stop-Process` (graceful) | graceful shutdown |
| `SIGHUP` | terminal close / SSH drop | (none — use `CTRL_CLOSE_EVENT`) | graceful shutdown + clear overlay |
| `SIGWINCH` | terminal resize | `WINDOW_BUFFER_SIZE_EVENT` | re-query size, resize fb (BUG-B3) |
| `SIGUSR1` | user-defined | (none) | toggle pause/resume (control socket alt) |

Implementation: `signal-hook` on Unix, `SetConsoleCtrlHandler` on Windows, both flip one `Arc<AtomicBool>`. `SIGWINCH` is handled by crossterm's `Event::Resize` (foreground) — but background mode must also install a `SIGWINCH` handler because it doesn't read events (BUG-B1). On Windows, spawn a background thread that calls `ReadConsoleInput` filtered to `WINDOW_BUFFER_SIZE_EVENT` to synthesize resizes in background mode.

### Seam 5 — Config & data paths

Use the `directories` crate (or hand-rolled equivalents) so we never hard-code `~/.config`:

| Artifact | macOS | Linux | Windows |
|----------|-------|-------|---------|
| config | `~/Library/Application Support/Forgum/config.json` | `$XDG_CONFIG_HOME/Forgum/config.json` (~/.config) | `%APPDATA%\Forgum\config.json` |
| cows/data | `…/Forgum/cows/` | `…/Forgum/cows/` | `…\Forgum\cows\` |
| runtime (pid/socket) | `$TMPDIR/forgum-<uid>/` | `$XDG_RUNTIME_DIR/forgum/` | `%TEMP%\Forgum\` |
| logs | `~/Library/Logs/Forgum/` | `$XDG_STATE_HOME/Forgum/` | `%LOCALAPPDATA%\Forgum\Logs\` |

**Honor `$FORGUM_CONFIG` and `$FORGUM_DATA` overrides** in one place (BUG-C3).

---

## 4. Build & distribution

### 4.1 Rust engine — one workspace, many targets

`engine/` becomes a Cargo workspace with two crates:
- `forgum-platform` (the abstraction, `#[cfg]`-heavy, fully unit-tested per platform).
- `forgum-engine` (the binary; depends on `forgum-platform`, contains **no** `#[cfg]`).

### 4.2 Cross-compilation matrix

Produce prebuilt binaries for every Tier-1 target so users without a Rust toolchain get a working engine.

| Target triple | Build host | Toolchain | Notes |
|---------------|-----------|-----------|-------|
| `x86_64-pc-windows-msvc` | `windows-latest` | stable msvc | `.exe`, sign with sigstore |
| `aarch64-pc-windows-msvc` | `windows-latest` | stable + `aarch64` target | ARM64 Windows |
| `x86_64-apple-darwin` | `macos-13` | stable | Intel mac |
| `aarch64-apple-darwin` | `macos-14` | stable | Apple Silicon; universal2 binary via `lipo` |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | stable | glibc 2.35+ |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04` + `cross` | stable | `cross` uses QEMU/binfmt |
| `x86_64-unknown-linux-musl` | `ubuntu-22.04` | stable + `musl-tools` | fully static, for Alpine/scratch |

**Tooling:** prefer [`cross`](https://github.com/cross-rs/cross) for Linux ARM/musl (Docker-based, reproducible). For Apple Silicon, build natively on a `macos-14` runner. For Windows ARM64, build on `windows-latest` with the `aarch64` component.

### 4.3 Universal macOS binary

```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create \
  target/x86_64-apple-darwin/release/forgum-engine \
  target/aarch64-apple-darwin/release/forgum-engine \
  -output target/universal/forgum-engine
```
The Homebrew formula ships the universal binary → no per-arch bottles needed initially.

### 4.4 Static linking considerations

- `crossterm` is pure Rust → fine on musl.
- `libc`/`signal-hook` → fine.
- `winapi`/`windows-sys` → fine (no C deps).
- No OpenSSL, no sqlite → fully static musl build works. ✔

### 4.5 PowerShell module packaging

- Publish to **PowerShell Gallery** (`Publish-Module -Path ./Forgum -NuGetApiKey …`). The module manifest embeds the engine binary as a `NestedModule` or ships it in `bin/` and `GetEngineBinary.ps1` resolves it.
- The module must **not** require a Rust toolchain. Prebuilt binaries per OS/arch are downloaded by `install.ps1`/`install.sh` from GitHub release assets, with `sha256` verification.

### 4.6 Package managers

| Manager | Platform | Recipe |
|---------|----------|--------|
| **Homebrew** | macOS/Linux | `package-managers/homebrew/forgum.rb` — universal binary, `depends_on :rust` only as build-time. Add `on_arm` / `on_intel` stanzas if per-arch bottles. |
| **Scoop** | Windows | `package-managers/scoop/forgum.json` — `architecture` map with `64bit`/`arm64` `url`+`hash`. |
| **winget** | Windows | `HKDEVS.Forgum.yaml` + installer YAML; `InstallerType: zip` with per-arch `Architecture: x64`/`arm64`. |
| **Cargo** | All | `cargo install forgum-engine` (engine only; for users who don't want the PowerShell layer). |
| **APT/RPM** | Linux | Generate `.deb`/`.rpm` via `cargo-deb`/`cargo-generate-rpm` in CI. |

---

## 5. Runtime detection logic (the "what am I running in?" decision tree)

```text
forgum-engine starts
  │
  ├─ stdout is a tty?
  │    yes → render to stdout
  │    no  → open /dev/tty (unix) or CONOUT$ (win); else print cow_text & exit
  │
  ├─ inside tmux?        ($TMUX env, or ppid chain contains tmux)
  │    yes → MuxKind::Tmux
  │         ├─ passthrough escapes? tmux 3.3+ supports `set -g allow-passthrough on`
  │         │   needed only if we use the alternate screen inside a pane
  │         ├─ overlay above prompt: safe (tmux panes are independent ttys)
  │         └─ background daemon must be per-pane (PID keyed on $TMUX_PANE)
  │
  ├─ inside zellij?      ($ZELLIJ env)
  │    yes → MuxKind::Zellij — no passthrough by default; overlay still safe
  │
  ├─ inside screen?      ($STY env)
  │    yes → MuxKind::Screen — avoid alternate screen (screen's is quirky)
  │
  ├─ terminal supports 24-bit color?  ($COLORTERM=truecolor, or query `DA1`)
  │    no  → downgrade effects to 8-color / mono
  │
  └─ terminal width < 20 or height < 5?
       yes → print cow_text statically (engine refuses to animate on tiny terms)
```

**Shell detection (for `forgum init`):**
```rust
fn detect_shell() -> ShellKind {
    // 1. explicit override
    if let Ok(s) = env::var("FORGUM_SHELL") { return ShellKind::from(&s); }
    // 2. parent process name (most reliable)
    if let Some(name) = parent_process_name() {  // /proc/$PPID/comm or ps
        return match name.as_str() {
            "bash" => Bash, "zsh" => Zsh, "fish" => Fish,
            "pwsh" | "powershell" => Pwsh,
            _ => {}
        };
    }
    // 3. $SHELL fallback (unix) / assume pwsh (windows)
    env::var("SHELL").map(ShellKind::from).unwrap_or(if cfg!(windows) { Pwsh } else { Bash })
}
```
This fixes BUG-S8 (WSL/Git Bash misdetected) — `parent_process_name()` returns `bash` even on Windows.

---

## 6. CI matrix (the gate)

See `08-TESTING-STRATEGY.md` for the full plan. The cross-platform-relevant slice:

```yaml
strategy:
  fail-fast: false
  matrix:
    os: [ubuntu-22.04, ubuntu-24.04, macos-13, macos-14, windows-latest]
    target:
      - x86_64-unknown-linux-gnu
      - aarch64-unknown-linux-gnu
      - x86_64-unknown-linux-musl
      - x86_64-apple-darwin
      - aarch64-apple-darwin
      - x86_64-pc-windows-msvc
      - aarch64-pc-windows-msvc
    exclude:
      - { os: macos-13, target: aarch64-apple-darwin }
      - { os: macos-14, target: x86_64-apple-darwin }
      - { os: windows-latest, target: [linux, apple, musl]* }
      # …
steps:
  - uses: dtolnay/rust-toolchain@stable
    with: { targets: ${{ matrix.target }} }
  - run: cargo install cross --git https://github.com/cross-rs/cross
  - run: cross test --target ${{ matrix.target }} --release
  - run: cross build --target ${{ matrix.target }} --release
  - uses: actions/upload-artifact@v4
    with: { name: forgum-engine-${{ matrix.target }}, path: target/*/release/forgum-engine* }
```

**Gates that block merge:**
1. `cargo test` green on all 7 targets.
2. `cargo clippy -- -D warnings` clean.
3. `cargo fmt --check`.
4. Pester suite green on Windows (`pwsh`), Linux (`pwsh`).
5. A headless render-smoke test under `xvfb-run` + `tmux` that pipes a JSON config, captures 2 s of ANSI output with `asciinema`, and asserts the overlay region was written and the prompt row (last 3 lines) was **not** touched. (This single test would have caught BUG-B1, BUG-B3, BUG-B5.)

---

## 7. The "works on my machine" killers — a checklist for reviewers

Every PR that touches the engine or shell hooks must answer these:

- [ ] Does it compile on `aarch64-unknown-linux-gnu` and `x86_64-pc-windows-msvc`?
- [ ] Does it handle stdout-not-a-tty (piped)? (Seam 1)
- [ ] Does it restore the terminal on panic, SIGINT, SIGTERM, SIGHUP? (Seam 2/4)
- [ ] Does it use `cfg!` only inside `forgum-platform`?
- [ ] Does it avoid `cmd`, `Start-Process -Verb`, `MoveTo` beyond bounds?
- [ ] Does it use `saturating_*`/`checked_*` math on user-supplied ints?
- [ ] Does it escape JSON via a real serializer, not string substitution?
- [ ] Does it respect `$XDG_RUNTIME_DIR` / `%TEMP%` for runtime files?
- [ ] Does the new test run under `xvfb-run -a tmux new-session -d -s test …`?

---

## 8. Known cross-platform incompatibilities we explicitly accept

| Limitation | Reason | Mitigation |
|-----------|--------|------------|
| PowerShell 5.1 (Windows PowerShell) can't run the engine via `pwsh` | 5.1 has no `$IsLinux`/`$IsMacOS`; some .NET API differences | Supported only as a Windows host; `CrossPlatform.ps1` gates on `$PSVersionTable.PSEdition`. New features target pwsh 7. |
| ConHost (legacy console) has no truecolor | Pre-Windows Terminal | Detect `$env:WT_SESSION` absence → degrade to 8-color. |
| GNU screen's alternate screen is flaky | Historical | In `screen`, force background-overlay mode (no alt screen) and document it. |
| Alpine musl + `signal-hook` | Needs `signal-hook-registry` which is fine on musl | Verified; musl build is static. |
| macOS bash 3.2 | Licensed bash | The bash hook uses only 3.2-compatible substitutions (no `mapfile`, no `${var,,}`). CI runs a 3.2-equivalent check via `shellcheck --shell=bash`. |

---

## 9. Migration path from the current codebase

1. **Extract `forgum-platform`** (1–2 days): move `open_console_out`, `get_terminal_size`, `get_config_path` into the new crate; add Unix `/dev/tty`, signal, detach impls.
2. **RAII guards** (½ day): BUG-T2.
3. **Signal handler** (½ day): BUG-T1.
4. **`open_output()` unification** (½ day): BUG-B9, BUG-C1.
5. **Detach fix + PID file** (1 day): BUG-B7, BUG-D3.
6. **Config paths via `directories`** (½ day): BUG-C3.
7. **CI matrix** (1 day): section 6.

Total: ~1 week to reach "honestly cross-platform." Everything after that is features and polish.

---

**Next:** `03-ANIMATION-ENGINE-ABOVE-PROMPT.md` — the precise design for the overlay engine that renders above the prompt without breaking it.
