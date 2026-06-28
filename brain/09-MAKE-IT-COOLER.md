# Forgum — Make It Cooler (Feature Backlog)

> The user asked to "make it much more cooler." This is the idea backlog — features that take Forgum from "a working cowsay with animations" to "the terminal companion people show off in demos." Grouped by theme, each idea has a hook, a sketch, and a difficulty.

---

## 1. Reactive effects (the terminal feels alive)

### 1.1 CPU-load reactive ember 🔥
- **Hook:** the cow's fire burns hotter when your CPU is busy.
- **Sketch:** the engine polls system load every 500 ms (`sysinfo` crate). Ember particle spawn rate and brightness scale with load. At > 80 %, the cow sweats (`@@` eyes) and the flame turns white-blue.
- **Difficulty:** 🟢 easy. `sysinfo` is a 2-line add; map `load → spawn_rate`.

### 1.2 Memory-pressure liquid-chrome
- **Hook:** as RAM fills, the liquid-chrome surface gets more turbulent.
- **Sketch:** map `mem_used/mem_total → wave amplitude`. At > 90 %, droplets "leak" below `ob_y1` into the prompt guard band (clamped) — a visual "your memory is overflowing" gag.
- **Difficulty:** 🟢 easy.

### 1.3 Network-traffic matrix rain
- **Hook:** the matrix effect's glyph fall speed scales with bytes/sec on the default interface.
- **Sketch:** `sysinfo::Networks` delta → `matrix` fall speed. Idle = slow drizzle; `apt update` = downpour.
- **Difficulty:** 🟢 easy.

### 1.4 Audio-reactive aurora 🎵
- **Hook:** aurora hue and amplitude track the room's audio (via the mic) or a playing track's FFT.
- **Sketch:** `cpal` (cross-platform audio) captures the default input; a 64-bin FFT drives aurora band intensity + hue. Opt-in (`--audio`). On macOS, tap CoreAudio; on Linux, PipeWire via `pw-cat`; on Windows, WASAPI loopback.
- **Difficulty:** 🟠 medium. Audio capture perms differ per OS; provide a `--audio-device` flag and a graceful fallback.

### 1.5 Battery-aware palette
- **Hook:** on laptops, the whole palette warms (red shift) as battery drops; at < 20 %, effects slow to 15 fps to save power.
- **Sketch:** `sysinfo::Battery` → hue offset + fps cap. Plugs into the scheduler.
- **Difficulty:** 🟢 easy.

### 1.6 Git-status cow mood
- **Hook:** in a git repo, the cow's mood reflects repo state: clean = `oo` (happy), dirty = `..` (youthful), merge conflict = `@@` (paranoia), detached HEAD = `xx` (dead).
- **Sketch:** the shell hook runs `git status --porcelain` before `forgum` and passes `--eyes` accordingly. A `--git-aware` flag enables it.
- **Difficulty:** 🟢 easy. Pure shell-side.

---

## 2. The cow as a companion

### 2.1 `forgum say <cmd>` — the cow speaks command output
- **Hook:** `forgum say fortune` pipes a fortune; `forgum say ls -la` renders the directory listing inside the speech bubble (word-wrapped, scrolled).
- **Sketch:** the engine accepts `--text-command "ls -la"`, runs it, captures stdout, wraps it in the bubble. Long output paginates inside the bubble with a 1 fps scroll.
- **Difficulty:** 🟢 easy.

### 2.2 `forgum lang` — a cow-headed REPL
- **Hook:** a tiny REPL where every result is spoken by the cow. `forgum lang` → `forgum> 2+2` → cow says `4`.
- **Sketch:** `rustyline` for line editing; a minimal expression evaluator (or shell out to `python3`/`node`). Each result feeds `--text` and re-renders. The cow's eyes change with exit codes (0 = `oo`, non-zero = `xx`).
- **Difficulty:** 🟠 medium.

### 2.3 `forgum notify <msg>` — desktop notification + cow popup
- **Hook:** long-running command finishes? `make ; forgum notify "build done"` pops a tmux popup with a celebratory cow.
- **Sketch:** `forgum tmux popup --effect shatter --text "$1" --duration 3` + a native notification (`notify-rust`).
- **Difficulty:** 🟢 easy.

### 2.4 The cow that ages with your session
- **Hook:** the cow starts young (`..` eyes) at shell start and gets tired (`--`) after 2 hours; stoned (`**`) after 4.
- **Sketch:** the daemon tracks session start time; an `EyesEffect` overlay drifts the eyes on a schedule.
- **Difficulty:** 🟢 easy.

### 2.5 `forgum pet` — interact with the cow
- **Hook:** `forgum pet` opens a foreground animation where keystrokes "pet" the cow (it purrs, eyes become `^^`, hearts float up).
- **Sketch:** foreground mode + key event → particle burst of `♥` + eye state change. Esc to exit.
- **Difficulty:** 🟠 medium (needs the input path, which is legitimate in foreground).

---

## 3. Visuals & effects

### 3.1 3D-ish perspective cow (mode-7 floor)
- **Hook:** a "warp" effect that renders the cow on a receding perspective grid (SNES Mode-7 style).
- **Sketch:** each row's horizontal scale = `1 / (1 + row*k)`; rows further down compress. Pairs with `liquid-chrome`.
- **Difficulty:** 🟠 medium.

### 3.2 ASCII raymarched metaballs
- **Hook:** gooey metaballs that merge and split, rendered as `@%#*:. ` by density.
- **Sketch:** 2-field metaball SDF sampled on the grid → density → ASCII ramp. Smooth, hypnotic.
- **Difficulty:** 🟠 medium.

### 3.3 True 24-bit photographic cows
- **Hook:** `forgum --cow photo cat.png` converts an image to ANSI truecolor blocks (via `viuer`/`ratatui-image`) and animates it.
- **Sketch:** image → half-block ANSI; effects operate on the block grid. Kittens → actual kitten photos.
- **Difficulty:** 🟠 medium (image crates are heavy; gate behind a feature).

### 3.4 Particle text ("draw a fortune in sparks")
- **Hook:** the fortune text itself assembles from converging sparks, holds, then disperses.
- **Sketch:** `ShatterEffect` in reverse: particles fly *to* their target glyph positions, settle, hold for 2 s, then shatter out.
- **Difficulty:** 🟠 medium.

### 3.5 CRT/glow shader overlay
- **Hook:** a post-process that adds scanlines + bloom to any effect for retro vibes.
- **Sketch:** after compositing the frame, a second pass adds faint scanline cells and brightens high-luminance glyphs. `--shader crt` flag.
- **Difficulty:** 🟢 easy (it's a framebuffer post-pass).

### 3.6 Seasonal/auto-themes
- **Hook:** on Halloween, the default cow becomes a ghost + `portal` effect; on Dec 25, a santa cow + `ember` + snow particles.
- **Sketch:** `forgum theme auto` checks the date and applies a bundled seasonal theme. Users can override.
- **Difficulty:** 🟢 easy.

---

## 4. Social & multi-user

### 4.1 Pair-programming shared cow (rmux sync, §6.2 of `05-…`)
- **Hook:** on a shared tmux session, everyone sees the same cow; `forgum herd effect plasma` changes it for all.
- **Status:** designed in `05-…` §6.2; implement in Phase 6.

### 4.2 `forgum battle` — ASCII cow jousting
- **Hook:** two cows charge from opposite sides; on collision, a `shatter` burst; loser's eyes go `xx`.
- **Sketch:** two `PhysicsEffect` instances on opposite clips; a collision check; a 4-second scripted sequence. Pure delight for demos.
- **Difficulty:** 🟠 medium.

### 4.3 Leaderboard fortunes
- **Hook:** `forgum fortune --star` stars a fortune; an optional shared file (`~/.local/share/forgum/stars.json`) ranks them; `forgum fortune --top` shows the community favorites.
- **Sketch:** local-only by default; an opt-in `forgum remote sync-stars` can merge across hosts.
- **Difficulty:** 🟢 easy.

### 4.4 `forgum screenshot` — capture the current overlay as an image
- **Hook:** one command exports the animated overlay as a GIF or a styled PNG (with the terminal chrome).
- **Sketch:** render N frames to an offscreen framebuffer → `gif` crate / `image`. Great for READMEs and tweets.
- **Difficulty:** 🟠 medium.

---

## 5. AI-augmented fortunes

### 5.1 Local-LLM fortune generation
- **Hook:** `forgum fortune --ai "programming, dry humor"` generates a fresh fortune via a local model (llama.cpp / `mistral.rs`) — no network.
- **Sketch:** optional feature `ai`; spawns a local model with a fortune-themed prompt; caches by prompt-hash. Falls back to the bundled DB if no model.
- **Difficulty:** 🔴 hard (model deps), but high wow-factor.

### 5.2 Context-aware fortune
- **Hook:** the fortune references your current dir, last command, or git branch.
- **Sketch:** shell hook passes `--context "dir:~/proj, last:make, branch:main"`; the engine picks (or generates) a fortune that nods to it ("`make`: it's a trap.").
- **Difficulty:** 🟠 medium.

### 5.3 `forgum roast` — the cow roasts your last command
- **Hook:** `forgum roast` after a failed command → the cow delivers a gentle insult based on the command + exit code.
- **Sketch:** a template bank keyed on exit code + command pattern (`git` + non-zero → "committed to failure, I see"). Opt-in, never mean.
- **Difficulty:** 🟢 easy (templates) → 🟠 medium (LLM).

---

## 6. Workflow integration

### 6.1 `forgum hook preexec` / `postexec`
- **Hook:** wrap every command: preexec shows a "thinking" cow (`..` eyes, pulse effect); postexec shows the result mood.
- **Sketch:** `preexec`/`precmd` functions call `forgum daemon effect pulse` then `forgum daemon effect aurora` based on exit code.
- **Difficulty:** 🟢 easy.

### 6.2 `forgum timer <cmd>` — cow times your command
- **Hook:** `forgum timer cargo build` runs the build and, on completion, a cow announces the duration in a popup.
- **Sketch:** `time` wrapper → `forgum notify "built in 42s"`.
- **Difficulty:** 🟢 easy.

### 6.3 CI status in the status line
- **Hook:** `forgum-engine status-line` shows your last CI run's status (green/red) as a tiny cow emoji + the fortune.
- **Sketch:** the status-line subcommand reads a cached CI status file (updated by a GitHub Action webhook → local file). 
- **Difficulty:** 🟠 medium (needs the webhook side).

### 6.4 `forgum weather`
- **Hook:** the cow wears sunglasses if it's sunny at your location, an umbrella if raining; the effect matches (rain → `liquid`, sun → `ember`-warm).
- **Sketch:** `--weather` opt-in; calls `wttr.in` (or a configured endpoint) once per hour; maps to a theme. Offline → skip.
- **Difficulty:** 🟢 easy.

---

## 7. Accessibility & polish

### 7.1 `--reduce-motion` mode
- **Hook:** honors the OS reduce-motion setting; effects become gentle fades only, no shake/shatter.
- **Sketch:** detect `$NO_MOTION`/macOS `com.apple.universalaccess reduceMotion`; the engine clamps amplitude and disables one-shot effects.
- **Difficulty:** 🟢 easy. **Important for inclusivity.**

### 7.2 Colorblind-safe palettes
- **Hook:** `forgum theme cb-deutan` remaps the palette to deuteranopia-safe colors.
- **Sketch:** bundled palette transforms applied in the color module.
- **Difficulty:** 🟢 easy.

### 7.3 Screen-reader `--text-only`
- **Hook:** `forgum --text-only` prints just the fortune (no art) for screen-reader users.
- **Sketch:** already partly exists (piped stdout); make it an explicit flag with a clean newline-terminated fortune.
- **Difficulty:** 🟢 easy.

### 7.4 `forgum config` TUI glow-up
- **Hook:** the config wizard uses `ratatui` (Rust) for a gorgeous full-screen TUI with live preview, instead of the PowerShell `Out-ConsoleGridView`.
- **Sketch:** `forgum config` launches a ratatui app; left = settings tree, right = live mini-render of the current effect. Changes write `config.json` on save.
- **Difficulty:** 🟠 medium.

---

## 8. The "wow" demo reel (for the README)

A single `forgum showcase` command that runs a scripted 60-second reel:
1. (0–5s) `portal` cow materializes,
2. (5–15s) `aurora` with audio-reactive bands (if `--audio`),
3. (15–20s) `shatter` + reassemble as `forgum say "hello"`,
4. (20–30s) CPU-reactive `ember` while a `yes > /dev/null &` runs,
5. (30–40s) `forgum battle` joust,
6. (40–50s) `forgum herd list --watch` dashboard,
7. (50–60s) seasonal theme reveal.

Record with `asciinema` → GIF for the README. This is the marketing asset.

---

## 9. Prioritization for the "cool" backlog

| Tier | Ideas | Why |
|------|-------|-----|
| **Quick wins (Phase 8a, ~1 week)** | 1.1, 1.2, 1.3, 1.5, 1.6, 2.1, 2.3, 3.5, 3.6, 6.1, 6.2, 7.1, 7.3 | Mostly thin shells over existing effects; huge perceived value; some (reduce-motion, text-only) are accessibility musts. |
| **Demo drivers (Phase 8b, ~2 weeks)** | 2.2, 2.5, 3.1, 3.4, 4.2, 4.4, 8 (showcase) | The things that make a 30-second clip go viral. |
| **Deep cuts (Phase 8c, ongoing)** | 1.4, 3.2, 3.3, 5.1, 5.2, 5.3, 6.3, 7.4 | Heavy deps or research; ship when ready. |

---

**Next:** `00-README.md` — the index that ties all these documents together.
