# Forgum — Target Architecture

> The end-state design that ties together the engine, the platform crate, the shell hooks, the herder, and the PowerShell module. This is the picture every contributor should hold in their head.

---

## 1. Component map

```
┌───────────────────────────────────────────────────────────────────────────┐
│                         USER SHELL (bash/zsh/fish/pwsh)                     │
│                                                                            │
│   `forgum` function ──► forgum-engine render --cow … --background …        │
│   `precmd` sweep ────► checks daemon.json, clears overlay if daemon dead   │
└──────────────────────────────┬────────────────────────────────────────────┘
                               │ argv / control socket / signal
                               ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          forgum-engine  (Rust binary)                       │
│                                                                            │
│   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌────────────┐ │
│   │  CLI (clap)  │──►│ Config layer │──►│ Cow renderer │──►│  Effects   │ │
│   │ render/      │   │ merge:       │   │ load .cow,   │   │ (19+) +    │ │
│   │ fortune/     │   │ argv>file>   │   │ $eyes/tongue,│   │ particles  │ │
│   │ daemon/      │   │ defaults     │   │ speech bubble│   │            │ │
│   │ herd/theme/  │   └──────────────┘   └──────────────┘   └─────┬──────┘ │
│   │ tmux/init/   │                                              │        │
│   │ completions  │   ┌──────────────────────────────────────────▼──────┐ │
│   └──────┬───────┘   │            Render core                           │ │
│          │           │  FrameBuffer (double-buffered) ─► damage ─► run  │ │
│          │           │  Scheduler (adaptive fps)                         │ │
│          │           │  RegionAllocator (overlay clip)                   │ │
│          │           └──────────────────────┬───────────────────────────┘ │
│          │                                  │                            │
│          │           ┌──────────────────────▼───────────────────────────┐ │
│          │           │     forgum-platform  (cross-platform crate)      │ │
│          │           │  OutputHandle · SignalGuard · RawModeGuard       │ │
│          │           │  DetachSpawn · ConfigPath · RuntimeDir           │ │
│          │           │  DetectShell · DetectMux · ControlSocket         │ │
│          │           │  unix.rs / windows.rs / macos.rs                 │ │
│          │           └──────────────────────────────────────────────────┘ │
│          │                                                                 │
│   ┌──────▼──────────────────────────────────────────────────────────────┐ │
│   │  Daemon mode:  detached child + control socket + daemon.json state   │ │
│   │  Herder:       scans daemon.json files, multiplexes control cmds     │ │
│   │  Remote:       JSON-RPC over control socket (follow-me / shared)     │ │
│   └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────┬────────────────────────────────────────────┘
                               │ bundled data (include_dir!)
                               ▼
                  Data/Cows/*.cow  Data/Fortunes/*.txt  Data/Templates/

┌───────────────────────────────────────────────────────────────────────────┐
│            Forgum PowerShell module  (thin, optional, Windows-native UX)    │
│   Public/forgum.ps1 ──► InvokeEngine ──► forgum-engine (same binary)        │
│   Get-CFConfig / Set-CFConfig ──► config.json (shared with engine)          │
│   InvokeForgumTUI ──► interactive config wizard                             │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Workspace layout (proposed)

```
Forgum/
├── Cargo.toml                    # [workspace]
├── crates/
│   ├── platform/                 # forgum-platform (no #[cfg] outside this)
│   │   ├── src/{lib,unix,windows,macos,signal,output,paths,shell,mux,socket}.rs
│   │   └── tests/
│   ├── engine/                   # forgum-engine binary
│   │   ├── src/{main,cli,config,cow,bubble,effects,particles,framebuffer,
│   │   │         region,scheduler,color,style_matcher,terminal,render,daemon,
│   │   │         herd,remote,theme,tmux,init,completions}.rs
│   │   └── tests/
│   └── protocol/                 # forgum-protocol (SceneConfig, shared types)
├── data/Cows data/Fortunes data/Templates   # bundled via include_dir!
├── Forgum.psd1 Forgum.psm1 Public/ Private/  # PowerShell module (thin)
├── scripts/completions/                       # generated, never hand-edited
├── install.sh install.ps1 setup.ps1
├── package-managers/{homebrew,scoop,winget,deb,rpm}
├── Tests/                                     # Pester
└── .github/workflows/                         # CI matrix
```

**Rule:** the `engine/src/*.rs` files contain **zero** `#[cfg]` and **zero** `#[cfg(windows)]`. All platform branching lives in `crates/platform/`. This makes the engine logic portable and testable on a single host.

---

## 3. Data flow: `forgum "hello"` (background)

```
1. shell function `forgum` runs:
   forgum-engine render --cow default --text "hello" --background --duration 0 --config ~/.config/Forgum/config.json

2. engine CLI (clap) parses argv.

3. config layer:
   - load --config path (platform default if absent)
   - merge: argv flags > config.json > built-in defaults (default-config.json)
   - result: SceneConfig { effect, cow_file, eyes, tongue, text, background, duration, overlay_height, fps, speed, particles, lolcat, … }

4. cow renderer:
   - load Data/Cows/<cow_file>.cow (bundled)
   - expand $eyes/$tongue/$thoughts placeholders
   - wrap --text in a speech/thought bubble (word-wrap to max_width)
   - result: cow_text (the full ASCII art string)

5. effect resolution:
   - if effect == "auto": style_matcher::get_cow_style(cow_file) -> { base, speed, particles }
   - create_effect(base, cow_text, speed, particles)

6. render dispatch:
   - background=true && daemon not already running for this pane/session:
       spawn_daemon(config) -> setsid, writes daemon.json, returns
       (shell prompt returns immediately — the daemon renders in the background)
   - daemon child: render_loop_background(config)
       - open_render_output() (stdout or /dev/tty/CONOUT$)
       - install signal guards
       - loop: dt → effect.update → fb.clear → effect.render(clip) → compute_damage
               → if should_render: ESC7 / render_region (coalesced) / ESC8 / flush
               → check shutdown flag, control socket, max_frames
               → scheduler.wait_if_needed
       - on exit: OverlayGuard::drop clears the overlay, restores cursor, resets color

7. control socket (concurrent thread): accepts STOP/EFFECT/SPEED/PAUSE/STATUS,
   pushes onto an mpsc channel consumed each frame.

8. shell prompt: untouched. User types commands normally. `precmd` sweep runs
   on each prompt; if daemon died (kill -9), sweeps the overlay.
```

---

## 4. Key invariants (enforced by tests, see `08-TESTING-STRATEGY.md`)

1. **Prompt guard** — engine never writes row ≥ `ob_y1`.
2. **Cursor balance** — every `ESC7` has a matching `ESC8` before the next frame.
3. **No input reads in background** — `event::poll`/`event::read` absent from `render_loop_background` (grep-enforced).
4. **`#[cfg]` containment** — `engine/src/` has zero `#[cfg]` (CI grep-enforced).
5. **JSON via serde** — no string-concatenated JSON anywhere (CI grep for `format!(.*\"\\{` patterns + a JSON-validity fuzz test).
6. **RAII cleanup** — `RawModeGuard`/`AltScreenGuard`/`CursorShowGuard`/`OverlayGuard` exist and are created in the right scope (review gate).
7. **Version parity** — `Cargo.toml`, `Forgum.psd1`, manifests share one version; CI asserts equality.
8. **Completion parity** — committed completion scripts equal `forgum-engine completions <shell>` output (CI diff-enforced).

---

## 5. The platform crate contract (`forgum-platform`)

```rust
pub trait TerminalHandle {
    fn size(&self) -> (u16, u16);
    fn open_output(&self) -> Box<dyn Write>;
    fn enable_raw_mode(&self) -> RawModeGuard;
    fn enter_alt_screen(&self) -> AltScreenGuard;
    fn hide_cursor(&self) -> CursorShowGuard;
}

pub trait Spawner {
    fn detach(&mut self) -> io::Result<u32>;     // setsid/DETACHED_PROCESS
}

pub trait Paths {
    fn config_path(&self) -> PathBuf;            // XDG / AppData / Library
    fn data_dir(&self) -> PathBuf;
    fn runtime_dir(&self) -> PathBuf;            // XDG_RUNTIME_DIR / %TEMP%
    fn log_dir(&self) -> PathBuf;
}

pub fn detect_shell() -> ShellKind;
pub fn detect_mux() -> MuxKind;
pub fn install_shutdown_handler(flag: Arc<AtomicBool>) -> io::Result<()>;
pub fn install_resize_handler(flag: Arc<AtomicBool>) -> io::Result<()>;
pub fn control_socket_path(session: &str) -> PathBuf;   // mode 0600
```

Every other crate programs against these traits. Platform bugs (BUG-C1/C2/C3, BUG-B7/B9, BUG-T1/T2) are fixed **once** here.

---

## 6. The PowerShell module's role (deliberately shrinking)

Today the PowerShell module owns cow rendering, fortune, lolcat, animation, config, and the shell hooks. In the target architecture it becomes a **thin native-UX layer**:

| Responsibility | Today | Target |
|---------------|-------|--------|
| Cow rendering | `InvokeCowsay.ps1` + `ReadCowFile.ps1` + `FormatCowMessage.ps1` | **Engine** (native) |
| Fortune | `GetFortune.ps1` + `ReadFortuneFile.ps1` | **Engine** (`forgum-engine fortune`) |
| Lolcat | `FormatLolcat.ps1` | **Engine** (color module) |
| Animation | `PhysicsCow.ps1` + 10 legacy files | **Engine** (effects) — legacy PS animations kept only as a fallback when the binary is absent |
| Config | `Get/Set-CFConfig.ps1` | Shared `config.json` (engine is the writer of record for daemon state; PowerShell reads/writes user prefs) |
| Shell hooks | `GetForgumShellHook.ps1` | **Engine** (`forgum-engine init`) |
| TUI wizard | `InvokeForgumTUI.ps1` | Stays PowerShell (nice `PSReadLine`-powered UX) but calls the engine for preview |
| Tab completion | `ArgumentCompleters` | Stays PowerShell (engine provides `--list-effects`) |

The PowerShell module remains the **best Windows experience** (profile integration, `PSReadLine`, `Out-ConsoleGridView`), but it's no longer load-bearing for correctness. A bash user with zero PowerShell installed gets the full experience. This also kills the "PowerShell startup hangs" class of bugs permanently — the engine has no module-import cost.

---

## 7. Versioning & release

- Single source of truth: `crates/engine/Cargo.toml` `version`.
- CI generates `Forgum.psd1`, manifests, and the Homebrew formula version from it (`cargo run -- gen-version`).
- Semantic versioning: engine `0.x` → `1.0` when the §4 invariants hold green on all Tier-1 platforms for one release cycle.
- Release artifacts (per `02-…` §4.2): 7 prebuilt binaries + universal macOS + PowerShell module `.nupkg` + Homebrew bottle + scoop/winget manifests + `.deb`/`.rpm`.

---

## 8. Security model

- **Control socket** — mode `0600`, under per-user runtime dir; commands validated against an allowlist.
- **Cow file loading** — path-traversal guard (repo audit #3, already fixed) + symlink resolution confined to the data dir.
- **Fortune files** — same confinement.
- **stdin** — bounded to 4 MB (BUG-D4).
- **`--config` path** — must be inside config dir or an explicit user path; no following symlinks outside.
- **No network** — the engine never opens a socket except the local control socket and (optionally) the SSH-forwarded remote peer (user-initiated). No telemetry.

---

**Next:** `07-DEVELOPMENT-ROADMAP.md` — the phased plan with concrete tasks, owners, and dependencies.
