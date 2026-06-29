# Forgum — CLI Design: Subcommands + Args (the dual interface)

> **The "fast for humans, scriptable for shells" blueprint.** Forgum exposes **two equivalent interfaces to the same engine**: a rich **subcommand tree** (`forgum init`, `forgum render --cow dragon`, `forgum ai classify`) optimized for interactive terminal use, and a **flag surface** (`forgum --cow dragon --effect rainbow`) optimized for shell-rc scripting and one-liners. Both resolve to the same config + the same render path. **`forgum init` and `forgum-init` are the same command** — the project ships both the spaced and hyphenated forms so that shell autocompletion, muscle memory, and `eval`-style init blocks all work.
>
> This document specifies the **dual-interface principle**, the **complete subcommand tree**, the **complete flag surface**, the **config-as-single-source-of-truth** precedence rules, the **shell-startup integration** pattern, **completion generation**, the **`forgum init` first-run flow**, and the **CLI versioning/deprecation policy**. It is the user-facing contract; the TUI in `17-TUI-CONFIG-MENU.md` is the friendly face on top of this contract.
>
> Read after `04-PROMPT-INTEGRATION.md`, `06-ARCHITECTURE.md`, and `15-INSTALLATION-AND-PACKAGING.md`.

---

## 0. The dual-interface principle (the constitution of the CLI)

1. **Subcommands are for humans.** `forgum render --cow dragon` is what you type at a prompt. They are discoverable (`forgum --help`, `forgum <cmd> --help`), autocompleteable, and self-documenting. They are the **primary** interface.
2. **Flags are for scripts.** `forgum --cow dragon --effect rainbow --duration 0` is what goes in a `.zshrc` precmd or a CI script. They are terse, positional, and pipeline-friendly. They are the **scripting** interface.
3. **They are equivalent.** Every subcommand has a flag equivalent and vice versa, resolved by the same `clap` parser. `forgum render --cow dragon` ≡ `forgum --render --cow dragon` ≡ `forgum --cow dragon`. The parser normalizes all three to the same internal `Args` struct. No feature is subcommand-only or flag-only.
4. **`forgum init` == `forgum-init`.** Both the spaced-subcommand form and the hyphenated-external-command form launch the same first-run/configure flow. A `forgum-init` shim binary (or shell function) redirects to `forgum init`. This is the rule the user explicitly required.
5. **Config is the single source of truth.** The TUI writes `config.toml`. Flags *override* config at runtime (never write back unless `--save`). Env vars are the lowest-priority override. The precedence: **CLI flags > config.toml > env vars > built-in defaults.** This is invariant #1 of the config system.
6. **The interactive menu is the sanctioned way to change config.** Users should run `forgum` (no args → the TUI) to change settings, not hand-edit `config.toml`. Expert users *may* edit the TOML directly (documented in `18-README-AND-WIKI-STRUCTURE.md`), but the TUI is the supported path. Every config key is reachable from the TUI — the "no orphan config" invariant (see `17-…`).
7. **No config without a CLI mirror.** If a setting exists in `config.toml`, a `forgum config set <key> <value>` (and a `--<key> <value>` flag) exists to change it. There is no setting reachable only by editing TOML. This is testable: CI enumerates config keys and asserts each has a `config set` path.

---

## 1. The parser: `clap` with derive, both surfaces in one struct

```rust
// crates/cli/src/args.rs
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "forgum", version, about = "Cross-platform cowsay + fortune + lolcat")]
#[command(propagate_version = true)]
pub struct Args {
    /// Render a cow to stdout (flag form of `forgum render`)
    #[arg(long, global = true)]
    pub render: bool,

    /// Which cow to show
    #[arg(long, short = 'c', global = true, env = "FORGUM_COW")]
    pub cow: Option<String>,

    /// Eye style
    #[arg(long, short = 'e', global = true)]
    pub eyes: Option<String>,

    /// Animation effect
    #[arg(long, global = true)]
    pub effect: Option<String>,

    /// Duration in seconds (0 = infinite for daemon; for foreground, 0 = single frame)
    #[arg(long, short = 'd', global = true)]
    pub duration: Option<u64>,

    /// Path to a config file (overrides the default lookup)
    #[arg(long, short = 'C', global = true, env = "FORGUM_CONFIG")]
    pub config: Option<PathBuf>,

    /// Save the current flag overrides back to config.toml
    #[arg(long, global = true)]
    pub save: bool,

    /// Quiet: suppress non-essential output (for scripting)
    #[arg(long, short = 'q', global = true, env = "FORGUM_QUIET")]
    pub quiet: bool,

    /// The subcommand (None → launch the TUI)
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Launch the interactive config menu (TUI). Default when no args.
    Tui,
    /// First-run setup + shell-hook injection. `forgum-init` redirects here.
    Init {
        #[arg(long)] first_run: bool,
        #[arg(long)] sh: Option<String>,        // bash|zsh|fish|pwsh
        #[arg(long)] silent: bool,              // emit hook lines only (for sourcing)
        #[arg(long)] uninstall: bool,           // strip the managed shell block
        #[arg(long)] yes: bool,                 // non-interactive, accept defaults
        #[arg(long)] no_shell_hook: bool,
    },
    /// Render a cow (subcommand form of --render)
    Render { /* cow/eyes/effect inherited as global flags */ },
    /// Print a fortune
    Fortune { #[arg(long)] long: bool, #[arg(long)] offensive: bool },
    /// Daemon lifecycle
    Daemon { #[command(subcommand)] action: DaemonAction },
    /// Multi-cow herd
    Herd { #[arg(default_value = "3")] count: usize, #[arg(long)] ai: bool },
    /// Theme/color management
    Theme { #[command(subcommand)] action: ThemeAction },
    /// AI features (see 14-AI-INTEGRATION-PLAN.md)
    Ai { #[command(subcommand)] action: AiAction },
    /// Config get/set/edit/validate/reset/import/export/diff
    Config { #[command(subcommand)] action: ConfigAction },
    /// Shell integration helpers (emit hook, completions)
    Shell { #[arg(long)] sh: String, #[arg(long)] completions: bool, #[arg(long)] hook: bool },
    /// Generate shell completions
    Completions { shell: ShellKind },
    /// tmux / multiplexer integration
    Tmux { #[command(subcommand)] action: TmuxAction },
    /// Verify install integrity
    Doctor { #[arg(long)] fix: bool },
    /// Upgrade via the active install method
    Upgrade { #[arg(long)] check: bool },
    /// Install or switch install method
    Install { #[arg(long)] switch_to: Option<String>, #[arg(long)] version: Option<String> },
    /// Uninstall (surgical; --purge for full removal)
    Uninstall { #[arg(long)] purge: bool },
    /// Voice (TTS/STT) controls
    Voice { #[command(subcommand)] action: VoiceAction },
    /// Conversational cow REPL
    Chat,
    /// Why was this cow chosen? (AI explainability)
    Why { #[arg(long)] verbose: bool },
}
```

**Note the global flags:** `--cow`, `--eyes`, `--effect`, `--duration`, `--config`, `--save`, `--quiet` are `global = true`, so they work on *every* subcommand. This is how `forgum render --cow dragon` and `forgum --cow dragon` (with `--render` implicit when no subcommand and a cow is given) both work.

### 1.1 The `forgum-init` redirect

Two mechanisms, both shipped:

1. **A `forgum-init` binary** (a separate cargo target, ~20 lines) that `exec`s `forgum init "$@"`. Built and installed alongside `forgum`. This guarantees `forgum-init` works even outside a shell (e.g., from a desktop launcher).
2. **A shell function** in the managed hook block: `forgum-init() { forgum init "$@"; }` — for users who installed only the main binary.

The `forgum init --first-run` flow is the **canonical** entry point for the interactive installer scripts (see `15-…` §2) and for users running `forgum init` themselves to reconfigure.

### 1.2 The no-args default

Running `forgum` with no arguments launches the **TUI config menu** (`17-…`). This is the "interactive menu is enough" rule made literal: the bare keyword opens the configuration interface. Running `forgum` with only global flags (e.g., `forgum --cow dragon`) renders once to stdout (the cowsay-like behavior). The disambiguation: **no subcommand + no render-implicating flags → TUI; no subcommand + a render flag → render.**

---

## 2. The full subcommand tree (reference)

```
forgum                                  # → TUI (interactive config menu)
forgum tui                              # → TUI (explicit)
forgum init [--first-run] [--sh <s>] [--silent] [--uninstall] [--yes] [--no-shell-hook]
forgum-init                             # ≡ forgum init

forgum render [--cow <c>] [--eyes <e>] [--effect <f>] [--duration <d>]   # foreground render to stdout
forgum fortune [--long] [--offensive]
forgum herd [N] [--ai]
forgum chat                             # conversational REPL (F25)

forgum daemon start|stop|status|restart|logs
forgum theme list|apply <name>|show|create|import <path>
forgum voice on|off|test|list
forgum why [--verbose]                  # AI: explain the last cow selection

forgum ai classify [--all|--cow <c>]
forgum ai show <cow>
forgum ai models list|install <id>|verify|remove <id>
forgum ai cloud on|off|status
forgum ai cloud <feature> on|off
forgum find "<natural language>"        # semantic cow search (F06)
forgum new "<description>"              # generate a cow (F15)
forgum tone <default|sarcastic|zen|hype|doom|pirate>
forgum star                             # star the last thought (F18)
forgum thought --regenerate
forgum day                              # daily digest (F21)
forgum pair on|off|status               # tmux pair-programming (F23)

forgum config get <key>
forgum config set <key> <value>
forgum config unset <key>
forgum config list [--prefix <p>]
forgum config edit                      # open config.toml in $EDITOR (expert)
forgum config validate
forgum config reset [--yes]
forgum config import <path>
forgum config export <path>
forgum config diff                      # show runtime overrides vs file
forgum config migrate [--dry-run]
forgum config path                      # print the config file path

forgum shell --sh <bash|zsh|fish|pwsh> --hook         # emit the precmd hook (for sourcing)
forgum shell --sh <...> --completions                  # emit completion script
forgum completions <bash|zsh|fish|powershell|elvish>

forgum tmux install|uninstall|status                   # tmux plugin management
forgum tmux demo                                       # the showcase reel

forgum doctor [--fix]
forgum upgrade [--check]
forgum install [--switch-to <method>] [--version <v>]
forgum uninstall [--purge]

forgum --help | -h
forgum --version | -V
```

**Every leaf is documented** with `--help` (clap derive generates it), and the wiki (`18-…`) has a page per subcommand with examples.

---

## 3. The full flag surface (reference)

### 3.1 Global flags (work on every subcommand)

| Flag | Short | Env | Config key | Meaning |
|------|-------|-----|------------|---------|
| `--cow <id>` | `-c` | `FORGUM_COW` | `render.cow` | Which cow to render |
| `--eyes <style>` | `-e` | `FORGUM_EYES` | `render.eyes` | Eye style (`default`, `happy`, `borg`, `dead`, `greedy`, `paranoid`, `stoned`, `wired`, `tongue`) |
| `--effect <name>` | | `FORGUM_EFFECT` | `animation.effect` | Animation effect (rainbow, bounce, wave, fade, glitch, …) |
| `--duration <sec>` | `-d` | `FORGUM_DURATION` | `render.duration` | 0 = infinite (daemon) / single frame (fg) |
| `--config <path>` | `-C` | `FORGUM_CONFIG` | (the file itself) | Override config file path |
| `--save` | | | | Write current flag overrides back to config.toml |
| `--quiet` | `-q` | `FORGUM_QUIET` | `render.quiet` | Suppress non-essential output |
| `--no-color` | | `NO_COLOR` | `render.color` | Disable all color (respect `NO_COLOR` convention) |
| `--gpu` | | `FORGUM_GPU` | `render.gpu` | Prefer hardware rendering (kitty/wgpu) |
| `--lang <code>` | | `FORGUM_LANG`/`LANG` | `ai.lang` | Thought language (F17) |
| `--tone <mode>` | | `FORGUM_TONE` | `ai.tone` | Tone mode (F07) |
| `--theme <name>` | | `FORGUM_THEME` | `theme.name` | Color theme |
| `--verbose` | `-v` | | | Debug logging |

### 3.2 Render-specific flags

| Flag | Config key | Meaning |
|------|------------|---------|
| `--width <cols>` | `render.width` | Speech-bubble width |
| `--think` | `render.think` | Use think-bubbles (`( )`) instead of speech (`< >`) |
| `--bubble-only` | `render.bubble_only` | Print only the bubble (no cow) |
| `--cow-only` | `render.cow_only` | Print only the cow (no bubble) |
| `--text <msg>` | | The message (positional alternative to stdin) |
| `--file <path>` | `-f` | Read message from file |
| `--fortune` | | Use a fortune as the message |

### 3.3 Daemon/overlay flags

| Flag | Config key | Meaning |
|------|------------|---------|
| `--daemon` | | Run as the background overlay daemon |
| `--overlay` | `overlay.enabled` | Render above the prompt (the flagship, see `03-…`) |
| `--fps <n>` | `render.fps` | Target framerate |
| `--port <n>` | `daemon.control_port` | Control socket port |

### 3.4 AI flags

| Flag | Config key | Meaning |
|------|------------|---------|
| `--ai`/`--no-ai` | `ai.enabled` | Master AI switch |
| `--cloud`/`--no-cloud` | `ai.cloud` | Cloud LLM opt-in |
| `--max-thought-words <n>` | `ai.max_thought_words` | Thought length cap |
| `--predict`/`--no-predict` | `ai.features.predictive_prerender` | Predictive pre-render |

**The mirror invariant (CI-enforced):** a test enumerates every key in `config.toml`'s schema and asserts that either (a) a `--<key>` flag exists, or (b) a `forgum config set <key>` path exists. No orphan config. This is what makes "everything configurable from the CLI/TUI" real, not aspirational.

---

## 4. Config-as-single-source-of-truth: the precedence ladder

```
   ┌─────────────────────────────────────────────┐
   │  1. CLI flags (--cow dragon)        highest  │  ephemeral, this invocation only
   │  2. config.toml                             │  persistent, what the TUI edits
   │  3. environment variables (FORGUM_COW)      │  session-scoped
   │  4. built-in defaults              lowest   │  compiled in
   └─────────────────────────────────────────────┘
```

- **`--save`** promotes #1 into #2: `forgum --cow dragon --effect rainbow --save` writes those values to `config.toml` and they persist. Without `--save`, flags are ephemeral.
- **`forgum config set <key> <value>`** writes directly to #2 (the sanctioned scripting way to persist a change without the TUI).
- **`forgum config get <key>`** reads the *effective* value (after the full ladder), not just the file — so you can see what a flag override would produce.
- **`forgum config diff`** shows the delta between the file and the current effective config (i.e., what flags/env are overriding).

### 4.1 The config schema (top-level keys)

```toml
schema_version = 3

[render]          # cow, eyes, effect, duration, width, think, color, gpu, fps, quiet
[animation]       # effect, mode, speed_mult, palette (the v2 7-axis DNA is per-cow, not here)
[overlay]         # enabled, region_height, z_order
[daemon]          # control_port, pid_file, watchdog_secs
[shell]           # detected, rc_files[], hook_injected
[theme]           # name, custom {}
[cows]            # library_path, blocklist[], favorites[]
[ai]              # enabled, default_backend, cloud, lang, tone, voice, max_thought_words
[ai.features]     # error_explain, suggest_next, predictive_prerender, pair_programming, fortune_feed
[ai.privacy]      # redact_home_paths, blocklist_commands[], intent_log, telemetry
[ai.models]       # llm, embeddings, vlm, tts, stt
[integrations]    # tmux, rmux, herdr, wezterm
[accessibility]   # narrator, high_contrast, reduced_motion
[install]         # method, version, canonical_path (written by installer, not user)
```

Every key here is reachable via the TUI (`17-…`) and via `forgum config set`. The `[install]` block is the one exception: it's written by the installer, read-only for users (the TUI shows it in "About" but doesn't let you edit it).

---

## 5. Shell-startup integration (the `eval` pattern)

The managed block in the shell rc (injected by `forgum init`, see `15-…` §4) sources the output of `forgum shell --sh <s> --hook`. That command emits:

### 5.1 bash
```bash
# emitted by `forgum shell --sh bash --hook`
_forgum_precmd() {
  forgum daemon sweep          # reap dead daemons (v2 fix)
  # the precmd render is done by the daemon overlay, not here, to keep the prompt fast
}
PROMPT_COMMAND="_forgum_precmd;${PROMPT_COMMAND}"
```

### 5.2 zsh
```zsh
# emitted by `forgum shell --sh zsh --hook`
autoload -Uz add-zsh-hook
_forgum_precmd() { forgum daemon sweep }
add-zsh-hook precmd _forgum_precmd
```

### 5.3 fish
```fish
# emitted by `forgum shell --sh fish --hook`
function _forgum_precmd --on-event fish_prompt
  forgum daemon sweep
end
```

### 5.4 pwsh
```powershell
# emitted by `forgum shell --sh pwsh --hook`
$global:__forgum_OriginalPrompt = $function:prompt
function global:prompt {
  forgum daemon sweep *> $null
  & $global:__forgum_OriginalPrompt
}
```

**Why `--hook` emits the *sweep* and not the render:** the render is done by the **daemon overlay** (the flagship, `03-…`), which runs continuously above the prompt. The precmd hook's only job is to sweep dead daemons and (optionally) trigger a one-shot render if the daemon isn't running. This keeps `PROMPT_COMMAND` cheap (<2 ms) and is the v2 fix for BUG-B1 (keystroke theft) — the hook never reads `/dev/tty`.

### 5.5 The completions

`forgum completions zsh` (or `forgum shell --sh zsh --completions`) emits the completion script, which `forgum init` installs to the right location (`~/.zsh/completions/_forgum` or `/usr/share/zsh/vendor-completions/`). Completions cover every subcommand, every flag, and **dynamic value completion** for `--cow` (lists the 109 cows), `--effect` (lists effects), `--theme` (lists themes), `--eyes` (lists eye styles). This is the `clap_complete` + custom dynamic completer pattern.

---

## 6. The `forgum init` first-run flow (detailed)

`forgum init --first-run` is what the install scripts call and what users run to (re)configure. The flow:

```
1. Detect environment
   - OS, arch, shell(s) present, terminal capabilities (kitty-graphics? wgpu?)
   - existing config? (if yes, this is a reconfigure; load current values as defaults)

2. Launch the TUI (17-TUI-CONFIG-MENU.md)
   ┌─────────────────────────────────────────────────────┐
   │            ★  Welcome to Forgum  ★                  │
   │                                                     │
   │        (animated cow, eyes tracking the cursor)     │
   │                                                     │
   │   Detected shell: zsh  (/Users/you/.zshrc)          │
   │                                                     │
   │   ▸ Quick start  (sensible defaults)                │
   │     Custom setup  (walk through every option)       │
   │     Import config  (from a file or URL)             │
   │                                                     │
   └─────────────────────────────────────────────────────┘
   (toggle with ↑↓, enter to select; the cow's eyes follow the highlight)

3. On "Quick start":
   - write config.toml with defaults + the detected shell
   - inject the managed hook block into the detected rc file
   - install completions
   - run `forgum doctor` and show the report
   - render a welcome cow ("forgum is ready. type 'forgum' to configure anytime.")

4. On "Custom setup": walk every config section as toggle menus (Appearance,
   Animation, Cows, Effects, AI, Shell, Integrations, Voice, Privacy, About).
   Each toggle writes to a draft config; "Apply" writes atomically.

5. On reconfigure (config exists):
   - load current config as the draft
   - same TUI, but every field shows its current value
   - "Apply" shows a diff before writing
```

`--yes` skips the TUI entirely (accepts all defaults + detected shell) — for non-interactive installs. `--no-shell-hook` does everything except the rc injection. `--sh bash` forces a specific shell. `--uninstall` reverses: strips the hook block, leaves config/data (unless `--purge`).

---

## 7. CLI versioning & deprecation

- **Semver.** `forgum --version` prints `forgum 2.0.0 (engine 0.4.0, ai 1.0.0)`. The CLI, engine, and AI crate version independently but ship together.
- **No breaking subcommand/flag removals in a major version.** A flag renamed in v3 keeps the old name as a hidden alias for one major version, printing a deprecation warning on stderr (once per session).
- **`forgum config migrate --dry-run`** shows what a version upgrade will change in the config. Migrations are always backward-compatible (additive); a v3 config works on v2 (ignored keys), a v2 config is migrated up by v3.
- **Deprecation policy:** a deprecated flag is documented in `CHANGELOG.md` and the wiki "Deprecations" page; it remains for 2 minor versions, then is removed in the next major.

---

## 8. The one-paragraph summary

Forgum's CLI is a single `clap` parser exposing **two equivalent surfaces**: a subcommand tree (`forgum init`, `forgum render --cow dragon`, `forgum ai classify`) for interactive use, and a global-flag surface (`forgum --cow dragon --effect rainbow`) for shell-rc scripting — both resolving to the same `Args` struct, so `forgum render --cow dragon` and `forgum --cow dragon` are identical. **`forgum init` and `forgum-init` are the same command** (a shim binary + a shell function both redirect), launching the first-run/configure TUI. Running bare `forgum` launches the config menu; running `forgum` with a render flag renders once. Config is the single source of truth with a strict precedence ladder (flags > config.toml > env > defaults), `--save` persists overrides, and `forgum config set` is the sanctioned scripting path — but the **TUI is the supported way to change settings**. Every config key has both a flag mirror and a TUI toggle (CI-enforced "no orphan config"). Shell integration is an `eval`-able `forgum shell --sh <s> --hook` that injects an idempotent managed block, with completions installed per shell. The CLI is semver-stable, deprecations span two minor versions, and config migrations are automatic and backward-compatible. This is the contract the TUI sits on top of, and the contract every integration (tmux, AI, voice) is built against.
