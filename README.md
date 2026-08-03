# ╔══════════════════════════════════════════════════════════════════════════╗
# ║                        🐮  F O R G U M  🐮                            ║
# ║                                                                        ║
# ║   The terminal cow that refuses to be ordinary.                        ║
# ║   Cowsay's cooler, smarter, more animated cousin.                      ║
# ╚══════════════════════════════════════════════════════════════════════════╝

> **Forgum** is a Rust CLI that renders ANSI cows in your live terminal —
> with a render loop, animation effects, shell hooks, a daemon, and a
> capability probe. Cross-platform on Windows, macOS, and Linux.

**Repo:** `HKDevLoops/Forgum` · **Version:** `v0.4.0` · **License:** MIT

---

## 🎯 Why Forgum?

Forgum is not just another cowsay. It is a tiny, opinionated animation engine that
lives *above* your prompt without getting in the way. Think of it as the cow
that went to art school and came back with a portfolio.

| Feature | What it means |
|---------|---------------|
| 🎨 **Dirty-tracking renderer** | Only redraws pixels that changed — cheap even at high FPS |
| ⚡ **Zero-alloc hot path** | No per-frame heap churn; the engine reuses its buffers |
| 🔒 **Synchronized-update safe** | Opt-in only where the terminal proves it can handle it |
| 🔍 **Capability probe** | Detects terminal features at runtime; degrades gracefully |
| 🐜 **Leak-proofed daemon** | Cleans up pipes, PID, and socket on exit |
| 🐚 **6-shell hooks** | bash, zsh, fish, pwsh, powershell, cmd — first-class |
| 📦 **9 package managers** | winget, scoop, choco, brew, apt, dnf, pacman, emerge, nix |
| 🌍 **Cross-platform** | Windows, macOS, Linux — consistent behavior everywhere |

---

## ⚡ Quick install

| Platform / pkg mgr | Command |
|--------------------|---------|
| Windows (winget) | `winget install HKDevLoops.Forgum` |
| Windows (scoop) | `scoop bucket add extras; scoop install forgum` |
| Windows (choco) | `choco install forgum` |
| macOS (Homebrew) | `brew install forgum` |
| Debian / apt | `sudo apt install forgum` |
| Fedora (dnf) | `sudo dnf install forgum` |
| Arch (pacman) | `sudo pacman -S forgum` |
| Gentoo (emerge) | `sudo emerge forgum` |
| Nix | `nix-env -iA nixpkgs.forgum` |
| Any (cargo) | `cargo install forgum` |

> Community-maintained lanes — install at your own risk.
> The official build is `cargo build --workspace`.

---

## 🚀 Quickstart (3 commands)

```bash
# 1. Install (pick your lane from above)

# 2. Run Forgum
forgum

# 3. See the cow. Type `forgum` again anytime to change things.
```

That's it. You do not need to edit any config file. Run `forgum` and follow the cow.

---

## 🎪 The Forgum Farm — A Tour in Animal Voices

Every Forgum scene is described by a `SceneConfig`. Think of the config as a
little farm, and each option as one of the animals that lives there. Here is
who you'll meet:

### 🐮 The Cow says:

> "Moo. I am the star of the show — the ANSI cow (or whatever critter) that gets
> rendered. Set `cow` to pick your beast, and I'll moo it across the terminal.
> Set `cow` to `"random"` and I'll pick a different cow from `data/Cows/` each time."

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

## 🐚 Shell Integration

Forgum hooks into your shell so the cow shows up automatically. The easiest path:

```bash
forgum-engine init <shell>
```

…where `<shell>` is one of `bash`, `zsh`, `fish`, `pwsh`, `cmd`, `powershell`.

### Manual integration

| Shell | Manual snippet |
|-------|----------------|
| bash | Add `eval "$(forgum-engine init bash)"` to `~/.bashrc` |
| zsh | Add `eval "$(forgum-engine init zsh)"` to `~/.zshrc` |
| fish | `forgum-engine init fish \| source` in `~/.config/fish/config.fish` |
| pwsh (PowerShell 7+) | `forgum-engine init pwsh \| Out-String \| Invoke-Expression` in `$PROFILE` |
| powershell (5.1) | Same as pwsh, in Windows PowerShell's `$PROFILE` |
| cmd | `forgum-engine init cmd` prints a registry/AutoRun snippet |

---

## 🖥️ Terminal Compatibility

| Terminal | Sync (DEC 2026) | Graphics | Notes |
|----------|-----------------|----------|-------|
| Windows Terminal | ✓ (when supported) | ✗ | sync gated by capability probe |
| Ghostty | ✓ | ✓ (Sixel) | full modern support |
| kitty | ✓ | ✓ (Kitty graphics) | native graphics protocol |
| iTerm2 | ✓ | via imgcat (out of scope) | sync supported |
| Alacritty | ✓ | ✗ | sync only |
| Konsole | ✓ | ✓ (Sixel) | sync + sixel |
| gnome-terminal / xterm | varies | Sixel via xterm sometimes | conservative |
| Terminal.app (macOS) | ✗ | ✗ | ANSI only |

All advanced features are capability-probed and OFF by default; Forgum emits
conservative ANSI so it never breaks on an unknown terminal.

---

## ⚙️ Config File Location

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\Forgum\config.json` |
| macOS | `~/.config/Forgum/config.json` |
| Linux | `~/.config/Forgum/config.json` |

Override at runtime with the `FORGUM_CONFIG` environment variable.

---

## 🎛️ Interactive Config

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

## 🎨 Sample Configs

Want a head start? Browse the ready-made scenes in `docs/samples/`:

| Config | Description |
|--------|-------------|
| `config.rainbow.json` | Full-color, effect-heavy joy |
| `config.minimal.json` | Just the cow, nothing else |
| `config.solid.json` | Solid background, calm and clean |

See `docs/samples/README.md` for the full tour of each sample.

---

## 🏗️ Build from Source

```bash
cargo build --workspace
cargo test --workspace
```

### Bubble Format

The speech bubble is rendered with a consistent width guarantee: all rows (top,
content, and bottom) have identical visible width. This ensures the bubble looks
correct regardless of text length or line count.

### Platform Notes

- **i686 (32-bit Windows)**: Build-only lane — the binary builds but is not
  tested or packaged for release. Use at your own risk.

---

## 🍀 Fortune

Need a little wisdom from the farm?

```bash
forgum-engine fortune
```

---

## 📚 Further Reading

| Document | What it covers |
|----------|----------------|
| `CONTRIBUTING.md` | How to contribute, and the current status of each package-manager lane |
| `ADVANCED.md` | Deep dives into the engine, daemon, and capability probe |
| `docs/TALES.md` | Longer stories from the Forgum menagerie |
| `docs/samples/README.md` | The sample config catalog |

---

## 📜 License

MIT. See `LICENSE`.

---

<div align="center">

```
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||
```

*Made with ❤️ and Rust · For the terminal cow in all of us*

</div>
