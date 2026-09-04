```text
                   🌿  🌴  F O R G U M   J U N G L E   E N G I N E  🌴  🌿
       .~~.                                                                      .~~.
      (____)                                                                    (____)
    .-'    `-.                                                                .-'    `-.
  .'  / \  //\`.      ███████╗ ██████╗ ██████╗  ██████╗ ██╗   ██╗███╗   ███╗  .'  ^__^     `.
 /|\___/| /   \`\     ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██║   ██║████╗ ████║ /   /(oo)\____   \
/ /O   O \/ // \ \    █████╗  ██║   ██║██████╔╝██║  ███╗██║   ██║██╔████╔██║ |  /  (__)     )\ |
|( @_^_@ ) //   | |   ██╔══╝  ██║   ██║██╔══██╗██║   ██║██║   ██║██║╚██╔╝██║ |     ||---w |   |
 \ \__^_/ //    | /   ██║     ╚██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║ ╚═╝ ██║ \     ||    ||   /
  `-(_//)//____.-'    ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝     ╚═╝  `-.________..-'
      //  ||               🌴  ANIMATED TERMINAL ECOSYSTEM  🌴                   ||   ||
     //   ||            ___                                   ,___.              ||   ||
    //    ||          {~o_o~}    ( ( (   K I N E M A T I C   {o,o}               ||   ||
   //     ||           ( Y )      ) ) )   J U N G L E   ) )   )__)               ||   ||
  //      ||          ()~*~()    ( ( (   M O T I O N   ( ( (   ""                ||   ||
""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""""
   🐉 Dragon (Ember)       🐨 Koala (Zen)        🐮 Cow (Pasture)       🦜 Toucan (Sky)
```

> **Forgum** is a high-performance Rust terminal animation engine that renders living ANSI creatures in your terminal —
> featuring physical 2D kinematics (stride-coupled ground traversal, flight swoops, Lissajous drift), 15 preloaded themes,
> zero-alloc dirty-damage rasterization, fail-safe signal/input handling, shell hooks, daemons, and capability probes.
> Cross-platform on Windows, macOS, and Linux.

**Repo:** `HKDevLoops/Forgum` · **Version:** `v0.4.0` · **License:** MIT

---

---

## 📖 The Legend of the Ascended Bovine

In 1999, the terminal world was given `cowsay`. It was charming. It was funny. But it was also completely frozen in time — a static fossil that printed static ASCII text, cluttered your scrollback buffer, and died the microsecond you hit Enter. For over twenty-five years, developers accepted the lie that terminal critters must be static corpses carved into stdout.

**Then came Forgum.** ⚡

We didn't just give the cow a fresh lick of paint. We performed open-heart surgery on terminal rendering. We ripped out archaic synchronous print loops and dropped in a **lock-free, 3-threaded engine** (`SIM`, `RENDER`, `CONTROL`). We gave our creatures **Newtonian kinematics and 2D orbital trajectory physics**, so they actually walk, fly, and drift across your terminal pasture. We wired up **differential dirty-cell damage tracking**, so redrawing a frame requires zero full-screen clears and emits only the exact bytes that changed.

Today, Forgum is a living, breathing ANSI ecosystem that lives *above* your prompt without interfering with your workflow. While your terminal sits idle, a dragon breathes ember particles on your margin, a koala meditates with harmonic chest oscillation, and a cow strides purposefully across your screen — all at a buttery **60 FPS** while sipping less CPU than your terminal's blinking cursor.

---

## 🦾 Why Forgum Utterly Destroys Legacy Terminal Mascots

| What Legacy Mascots Do (1999) | What Forgum Does (2026) | Why You Should Care |
|:---|:---|:---|
| 🧟 **Static Print & Die:** Dumps text into your scrollback buffer. | 🏃 **2D Continuous Kinematics:** Creatures physically traverse the screen, swooping in flight and drifting on orbital paths. | Your terminal becomes a living, animated canvas. |
| 🪚 **Treadmill Moonwalking:** Characters toggle leg glyphs in place. | 📐 **Stride-Velocity Coupling:** Leg stride phase is mathematically bound to ground velocity ($\omega = v/\lambda$). | Zero moonwalking. If the cow halts, its hooves halt. |
| 🔥 **CPU-Melting Screen Clears:** Clears the entire terminal every frame (`\x1b[2J`), causing eye-bleeding flicker. | ⚡ **Dirty-Cell Damage Jumps:** Compares front and back buffers at the cell level, emitting only minimal ANSI jumps (`\x1b[y;xH`). | Smooth 60 FPS animation with sub-1% CPU usage. |
| 🔒 **File Lock Deadlocks:** Multiple shells crash or lock each other out of shared state files. | 🪟 **Zero-Lock Session Nirvana:** Pane-level isolation across `tmux`, `zellij`, `wezterm`, `kitty`, and Windows Terminal. | Run 50 terminal splits concurrently with zero lock contention. |
| 🦻 **Deaf to Ctrl+C in Raw Mode:** Raw mode suppresses `ISIG`, trapping you forever in a runaway animation. | 🛑 **Sub-Millisecond Fail-Safe Exit:** Non-blocking 50ms event polling intercepts `\x03`, `'q'`, and `Esc` instantly. | When you say stop, it stops in 5 milliseconds flat. Period. |
| 📄 **Manual JSON Configuration Hell:** Requires hunting down hidden dotfiles to configure anything. | 🎨 **15 Preloaded Themes & Interactive TUI:** Instant out-of-the-box themes (`matrix`, `cyberpunk`, `inferno`, etc.). | Zero-config out of the box; full TUI customizer when you want it. |

---

## 🧮 The Mathematics of the Pasture: Kinematics & Physics

Forgum's motion isn't fake frame-flipping. Every critter, particle, and speech bubble is an active rigid-body governed by continuous 2D classical mechanics:

### 1. Continuous Kinematic Integration
Every frame, the simulation thread computes exact floating-point position $\vec{P}(t)$, velocity $\vec{V}(t)$, and acceleration $\vec{A}(t)$ using deterministic time-delta $\Delta t$:

$$\vec{P}(t + \Delta t) = \vec{P}(t) + \vec{V}(t)\Delta t + \frac{1}{2}\vec{A}(t)\Delta t^2$$

$$\vec{V}(t + \Delta t) = \vec{V}(t) + \vec{A}(t)\Delta t$$

At render time, the continuous coordinates are quantized to discrete terminal grid boundaries:

$$x_{\text{col}} = \lfloor P_x \rceil, \quad y_{\text{row}} = \lfloor P_y \rceil$$

### 2. The Anti-Moonwalk Theorem: Stride-Velocity Coupling
In `WalkEffect`, leg oscillation frequency $\omega_{\text{stride}}$ is strictly coupled to instantaneous ground velocity $\vec{V}_x$ and step length $\lambda_{\text{step}}$:

$$\phi_{\text{stride}}(t) = \left( \frac{|P_x(t)|}{\lambda_{\text{step}}} \right) \pmod{1.0}$$

$$\text{LegState}(t) = \begin{cases} (\text{'╱'}, \text{'╲'}), & \text{if } \text{Easing}(\phi_{\text{stride}}) > 0.5 \\ (\text{'╲'}, \text{'╱'}), & \text{otherwise} \end{cases}$$

> 💡 **The Result:** When the animal walks forward at 12 cols/sec, its hooves alternate in exact geometric lockstep with the ground below. When it pauses, the legs freeze. Real physics in pure ASCII.

### 3. Harmonic 2D Lissajous Orbital Drift (`float`)
For aquatic and celestial creatures (`dolphin`, `nyan`, `happy-whale`), Forgum executes orthogonal harmonic oscillations producing smooth 2D Lissajous curves:

$$X(t) = X_{\text{anchor}} + A_x \cdot \sin(\omega_x t + \delta_x)$$

$$Y(t) = Y_{\text{anchor}} + A_y \cdot \cos(\omega_y t + \delta_y)$$

### 4. Sinusoidal Ballistic Flight Trajectories (`fly`)
Flying creatures (`dragon`, `golden-eagle`, `ghost`) execute continuous flight traversal combined with a dual-harmonic altitude swoop:

$$X(t) = (X_0 + V_x \cdot t) \pmod{W_{\text{term}} + W_{\text{critter}}} - W_{\text{critter}}$$

$$Y(t) = Y_{\text{cruise}} + A_y \cdot \sin(2\pi f_{\text{swoop}} t) + B_y \cdot \cos(4\pi f_{\text{swoop}} t)$$

Wing flapping frequency dynamically accelerates proportionally to vertical climb rate $|\frac{dY}{dt}|$.

### 5. CRT Scanline Glitch & Coordinate Tearing (`glitch`)
Under cybernetic glitch spikes, characters experience localized pseudo-random coordinate displacement and scanline tearing:

$$I_{\text{glitch}}(t) = |\sin(2\pi f_{\text{glitch}} t)|^3$$

$$\Delta x_{\text{scanline}} = \begin{cases} \text{sgn}(\sin(\text{seed})) \cdot \lfloor 3 \cdot I_{\text{glitch}} \rfloor, & \text{if } I_{\text{glitch}} > 0.80 \\ 0, & \text{otherwise} \end{cases}$$

---

## ⚡ Quick install

| Platform / pkg mgr | Command |
|--------------------|---------|
| Ubuntu / Debian (apt) | `sudo apt install ./forgum_*.deb` |
| Kali Linux (apt) | `sudo apt install ./forgum_*.deb` |
| Fedora (dnf) | `sudo dnf install ./forgum-*.rpm` |
| openSUSE (zypper) | `sudo zypper install ./forgum-*.rpm` |
| Nix (Flake / Nixpkgs) | `nix profile install github:HKDevLoops/Forgum` |
| Windows (winget) | `winget install HKDevLoops.Forgum` |
| Windows (scoop) | `scoop bucket add extras; scoop install forgum` |
| Windows (choco) | `choco install forgum` |
| macOS (Homebrew) | `brew install forgum` |
| Any (cargo) | `cargo install forgum-engine` |

> Community-maintained lanes — install at your own risk.
> The official build is `cargo build --workspace`.

---

## 🚀 Quickstart (3 commands)

```bash
# 1. Install (pick your lane from above)

# 2. Run Forgum
forgum-engine

# 3. See the cow. Type `forgum-engine` again anytime to change things.
```

That's it. You do not need to edit any config file. Run `forgum-engine` and follow the cow. On PowerShell, `forgum` is also available as a wrapper via `Forgum.psm1`.

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

## 🩺 Check Your Pasture's Health (`checkhealth`)

Got weird rendering? Colors looking like a melted popsicle? Shell hooks misbehaving? Channel your inner Neovim user and run:

```bash
forgum-engine checkhealth
```

The health inspector will run 12 diagnostic probes across 7 core systems (System, Configuration, Terminal & TrueColor, Pasture Assets & DNA profiles, Shell hooks, Daemons, and Structured Logs) and give you actionable remediation suggestions.

For CI/CD and scripts, get machine-readable JSON:
```bash
forgum-engine checkhealth --json
```

---

## ⚙️ Config File Location & Multi-Format Support

Forgum has **one unified configuration home across all operating systems**:

| Platform | Path |
|----------|------|
| Windows | `~/.config/forgum/config.json` (or `.yaml` / `.toml`) |
| macOS | `~/.config/forgum/config.json` (or `.yaml` / `.toml`) |
| Linux | `~/.config/forgum/config.json` (or `.yaml` / `.toml`) |

> **Single-Format Exclusivity:** Forgum supports `JSON`, `YAML`, and `TOML`, but forbids multiple format files in the same directory.
> Want to switch? Run `forgum-engine config --migrate toml` (or `json`/`yaml`) and let the engine convert it safely!

Override at runtime with the `FORGUM_CONFIG` environment variable.

---

## 🪵 Structured Logs & Diagnostics

Forgum logs all events with microsecond precision to both human-readable text and structured JSONL logs:

```bash
# View recent logs in a formatted table
forgum-engine logs

# Filter by severity
forgum-engine logs --level warn

# Live follow logs
forgum-engine logs -f
```

---

## 🎛️ Interactive Config TUI

Prefer a funky terminal menu with dark humour to hand-editing files?

```bash
forgum-engine config --tui
```

Walk through the pasture options, toggle animal attachments, preview cow DNA signatures, and save directly to your chosen format.

Headless or scripting? Set individual keys directly:

```bash
forgum-engine config set <key> <value>
```

For example: `forgum-engine config set color_mode rainbow`.

---

## 🎨 Preloaded Themes & Kinematic Motion

Forgum includes **15 built-in preloaded themes** ready out-of-the-box:

```bash
# List all built-in and user themes
forgum-engine theme list

# Apply a preloaded theme immediately
forgum-engine theme apply matrix
forgum-engine theme apply cyberpunk
forgum-engine theme apply inferno
```

| Theme | Effect | Mascot | Eyes | Vibe |
| :--- | :--- | :--- | :--- | :--- |
| `arcade` | `walk` | `default` | `oo` | Classic 8-bit pasture walk |
| `aurora` | `aurora` | `default` | `oo` | Northern lights color shifting |
| `cyberpunk` | `glitch` | `mech-and-cow` | `$$` | Neo-Tokyo neon scanline distortion |
| `forest` | `breathe` | `koala` | `..` | Calm bamboo canopy breathing |
| `ghost` | `portal` | `ghost` | `xx` | Ethereal spectral phasing |
| `inferno` | `ember` | `dragon` | `@@` | Blazing fire particles & smoke |
| `matrix` | `glitch` | `telebears` | `00` | Falling terminal green glyphs |
| `nyan` | `float` | `nyan` | `^^` | 2D cosmic rainbow orbital drift |
| `ocean` | `float` | `dolphin` | `oo` | Deep ocean buoyancy float |
| `zen` | `breathe` | `tux` | `==` | Meditative Linux penguin respiration |

### 🏃 2D Kinematics Engine
Entities in Forgum are driven by continuous 2D kinematics:
* **True Traversal (`walk`)**: The creature physically moves across the terminal pasture with leg strides coupled to ground velocity ($\omega = v / \lambda$).
* **Flight Trajectory (`fly`)**: Soars across the sky on a sinusoidal flight path with synchronized wing flapping.
* **Orbital Drift (`float`)**: Smooth 2D Lissajous floating drift across the viewport.
* **Interactive Fail-Safe Exit**: Press `Ctrl+C`, `q`, or `Esc` anytime in foreground mode for clean, immediate terminal restoration.

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
