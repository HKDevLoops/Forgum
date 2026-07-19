# Forgum 🐮

> A Rust CLI that renders ANSI cows (cowsay-like) in your live terminal — with a
> render loop, effects, shell hooks, a daemon, and a capability probe. Cross-platform
> on Windows, macOS, and Linux; bash, zsh, fish, pwsh, powershell, and cmd.

**Repo:** `HKDevLoops/Forgum` · **Version:** `v0.4.0`

---

## Why Forgum?

Forgum is not just another cowsay. It is a tiny, opinionated animation engine that
lives *above* your prompt without getting in the way.

- **Dirty-tracking performance** — only redraws the pixels that changed, so the
  render loop stays cheap even at high FPS.
- **Zero-alloc renderer** — the hot path reuses buffers; no per-frame heap churn.
- **Synchronized-update off by default** — safe ANSI output everywhere; it opts in
  only where the terminal proves it can handle it.
- **Capability probe** — detects your terminal's features at runtime and degrades
  gracefully instead of corrupting your screen.
- **Leak-proofed daemon** — the background overlay manager cleans up its pipes, PID,
  and socket on exit.
- **6-shell hooks** — first-class integration for bash, zsh, fish, pwsh, powershell,
  and cmd.
- **9 package managers** — winget, scoop, choco, brew, apt, dnf, pacman, emerge, nix.
- **Cross-platform** — Windows, macOS, and Linux, with consistent behavior.

---

## Quick install

These are package-manager lanes. Community-maintained — install at your own risk;
the official build is `cargo build --workspace` (see [Build from source](#build-from-source)).
See `CONTRIBUTING.md` for the current PR/availability status of each lane.

| Platform / pkg mgr | Command                                            |
|--------------------|----------------------------------------------------|
| Windows (winget)   | `winget install HKDevLoops.Forgum`                 |
| Windows (scoop)    | `scoop bucket add extras; scoop install forgum`    |
| Windows (choco)    | `choco install forgum`                             |
| macOS (Homebrew)   | `brew install forgum` *(tap may vary)*             |
| Debian / apt       | `sudo apt install forgum`                          |
| Fedora (dnf)       | `sudo dnf install forgum`                          |
| Arch (pacman)      | `sudo pacman -S forgum`                            |
| Gentoo (emerge)    | `sudo emerge forgum`                               |
| Nix                | `nix-env -iA nixpkgs.forgum`                       |

---

## The Forgum farm — a tour in animal voices

Every Forgum scene is described by a `SceneConfig`. Think of the config as a little
farm, and each option as one of the animals that lives there. Here is who you'll meet:

### 🐮 The Cow says:

> "Moo. I am the star of the show — the ANSI cow (or whatever critter) that gets
> rendered. Set `cow` to pick your beast, and I'll moo it across the terminal."

### 💬 The Text says:

> "I'm the words in the speech bubble. Put your message in `text` and I'll carry it
> wherever the cow goes."

### ✨ The Effect says:

> "Watch me sparkle! `effect` chooses how I animate — rainbows, fades, and more.
> I'm the reason people stare at their terminal instead of working."

### 🎨 The Background says:

> "I'm the canvas behind everything. `background` tints the world so the cow pops.
> Subtle is classy; loud is fun. Your call."

### ⏱️ The Duration says:

> "Tick. Tock. `duration` is how long I let the scene play before it bows out. Set me
> to `0` and I'll linger until you say stop."

### 🎞️ The FPS says:

> "I'm the heartbeat of the animation. `fps` tells me how many frames per second to
> push. Too high and you'll exhaust the terminal; too low and I limp."

### 👀 The Eyes say:

> "Look at me. `eyes` sets the cow's gaze — the classic `oo`, the deadpan `??`, or
> something silly. I give every cow its attitude."

### 👅 The Tongue says:

> "Blep. `tongue` is the little flick of personality at the bottom of the muzzle.
> Pair me with the right eyes and the cow gets a whole mood."

### 🦉 The Owl says:

> "Who delivers the cow to your prompt? I decide. `default_shell` is the shell the
> engine assumes when it sets up hooks — I watch from the branch and whisper the
> right command."

### 🦫 The Beaver says:

> "I build dams, and I also build habits. `auto_render_on_prompt` is my switch — when
> on, I trigger a render every time your prompt appears. Busy terminal? Flip me off."

### 🦎 The Chameleon says:

> "I become whatever the room needs. `color_mode` controls how color is handled —
> full, reduced, or off — so the farm looks right on every terminal, bright or dim."

---

## Shell integration

Forgum hooks into your shell so the cow shows up automatically. The easiest path is:

```bash
forgum-engine init <shell>
```

…where `<shell>` is one of `bash`, `zsh`, `fish`, `pwsh`, `cmd`, `powershell`.
That command prints the exact snippet your shell needs — but if you'd rather wire it
by hand, here's the manual version for each:

| Shell                | Manual snippet                                                                                        |
|----------------------|-------------------------------------------------------------------------------------------------------|
| bash                 | Add `eval "$(forgum-engine init bash)"` to `~/.bashrc`                                                |
| zsh                  | Add `eval "$(forgum-engine init zsh)"` to `~/.zshrc`                                                  |
| fish                 | `forgum-engine init fish \| source` in `~/.config/fish/config.fish`                                   |
| pwsh (PowerShell 7+) | `forgum-engine init pwsh \| Out-String \| Invoke-Expression` in `$PROFILE`                            |
| powershell (5.1)     | `forgum-engine init pwsh \| Out-String \| Invoke-Expression` in `$PROFILE` for Windows PowerShell     |
| cmd                  | `forgum-engine init cmd` prints a registry/AutoRun snippet to add (manual reg edit required)          |

> `forgum-engine init <shell>` does all of the above automatically — the manual
> snippets are for when you want to see (or tweak) what gets injected.

---

## Terminal compatibility

| Terminal                 | Sync (DEC 2026)        | Graphics (Sixel/Kitty)      | Notes                                          |
|--------------------------|------------------------|-----------------------------|------------------------------------------------|
| Windows Terminal         | ✓ (when supported)     | ✗                           | sync gated by capability probe; graphics off by default |
| Ghostty                  | ✓                      | ✓ (Sixel)                   | full modern support                            |
| kitty                    | ✓                      | ✓ (Kitty graphics)          | native graphics protocol                        |
| iTerm2                   | ✓                      | via imgcat (out of scope)   | sync supported                                 |
| Alacritty                | ✓                      | ✗                           | sync only                                      |
| Konsole                  | ✓                      | ✓ (Sixel)                   | sync + sixel                                   |
| gnome-terminal / xterm   | varies                 | Sixel via xterm sometimes   | conservative                                   |
| Terminal.app (macOS)     | ✗                      | ✗                           | ANSI only (conservative default)               |

All advanced features are capability-probed and OFF by default; Forgum emits conservative ANSI so it never breaks on an unknown terminal.

---

## Config file location

Forgum stores its config as JSON. The path depends on your platform:

| Platform | Path                                |
|----------|-------------------------------------|
| Windows  | `%APPDATA%\Forgum\config.json`      |
| macOS    | `~/.config/Forgum/config.json`      |
| Linux    | `~/.config/Forgum/config.json`      |

You can also override the path at runtime with the `FORGUM_CONFIG` environment
variable.

---

## Interactive config

Prefer a menu to hand-editing JSON? If Forgum was built with the `tui` feature:

```bash
forgum-engine config --tui
```

That opens the interactive configuration menu — walk the fields, toggle the animals,
and save. No mouse required.

Headless or scripting? Set individual keys directly:

```bash
forgum-engine config set <key> <value>
```

For example: `forgum-engine config set color_mode none`.

---

## Sample configs

Want a head start? Browse the ready-made scenes in `docs/samples/`:

- `docs/samples/config.rainbow.json` — full-color, effect-heavy joy
- `docs/samples/config.minimal.json` — just the cow, nothing else
- `docs/samples/config.solid.json` — solid background, calm and clean

See `docs/samples/README.md` for the full tour of each sample.

---

## Build from source

```bash
cargo build --workspace
cargo test --workspace
```

## Fortune

Need a little wisdom from the farm?

```bash
forgum-engine fortune
```

---

## Further reading

- `CONTRIBUTING.md` — how to contribute, and the current status of each package-manager lane
- `ADVANCED.md` — deep dives into the engine, daemon, and capability probe
- `docs/TALES.md` — longer stories from the Forgum menagerie
- `docs/samples/README.md` — the sample config catalog

---

## License

MIT. See `LICENSE`.
