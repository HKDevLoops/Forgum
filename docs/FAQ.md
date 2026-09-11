# ❓ Frequently Asked Questions (FAQ)

Welcome to the **Forgum FAQ**. Here you'll find answers to the most common questions about the terminal pasture, performance, configuration, shell attachments, and diagnostics.

---

## 🐮 General & Philosophy

### Q: What is Forgum?
**A:** Forgum is a modern, cross-platform terminal animation engine and cowsay/fortune/lolcat successor written in pure Rust. It renders living, breathing, color-graded ANSI creatures above or beside your prompt at up to 60 FPS with microsecond latency.

### Q: Why does my cow blink and breathe even when the effect is `"static"`?
**A:** Because in Forgum, even "static" cows are alive! Phase 8.1 keep-alive micro-animations introduce a subtle 1px vertical thoracic oscillation (~0.15 Hz) and an occasional eye-blink (~every 4.5s) to make your terminal feel alive without consuming CPU.

### Q: How is Forgum so lightweight on CPU and memory?
**A:** Forgum uses:
1. **Dirty-Cell Tracking:** Only cells that changed between frames are diffed and sent to the terminal.
2. **Zero-Allocation Hot Loops:** Precomputed lookup tables (LUTs) for blinks, pre-allocated scratch buffers, and in-place particle updates avoid all heap allocations during rendering.
3. **Double-Buffered Framebuffers:** Vectorized slice filling (`fill()`) enables LLVM to emit SIMD/memset assembly instructions.

---

## ⚙️ Configuration & Formats

### Q: Where is the configuration file located?
**A:** Forgum uses **`~/.config/forgum/`** as its unified configuration home across **all** operating systems:
- **Windows:** `%USERPROFILE%\.config\forgum\config.json` (or `.yaml` / `.toml`)
- **macOS:** `$HOME/.config/forgum/config.json` (or `.yaml` / `.toml`)
- **Linux:** `$HOME/.config/forgum/config.json` (or `.yaml` / `.toml`)

You can override this anytime by setting the `FORGUM_CONFIG` environment variable.

### Q: Which configuration formats are supported?
**A:** Forgum supports **JSON**, **YAML**, and **TOML**.

### Q: Why did Forgum give me a `ConfigConflict` error?
**A:** Forgum strictly enforces the **Law of Single-Format Exclusivity**. You cannot have both `config.json` and `config.toml` in the same directory.
To safely switch formats without losing any settings, use the built-in migration command:
```bash
forgum config --migrate toml    # or yaml / json
```

### Q: Can I configure Forgum using a GUI/TUI menu?
**A:** Yes! Run:
```bash
forgum config --tui
```
This launches an interactive terminal interface where you can browse options, cycle through 109+ cows, preview color palettes, and save your settings.

---

## 🐚 Shell Hooks & Attachment Modes

### Q: What are the 4 shell attachment modes?
**A:**
1. **Banner Burst (`banner`):** Renders a quick, non-blocking splash of your chosen creature right above the prompt upon opening a terminal or running a command.
2. **Split-Scroll Margin (`split` / DECSTBM):** Pins the cow in a dedicated top or bottom viewport margin while your shell scrolls underneath smoothly.
3. **PSReadLine Reactive Overlay (`overlay`):** Uses an idle timer to gracefully fade the cow in when you pause typing, and immediately yields the screen when you type.
4. **Manual CLI (`manual`):** Only displays when explicitly called via `forgum`, `forgum say`, `forgum fortune`, or `forgum timer`.

### Q: How do I hook Forgum into my shell?
**A:**
- **PowerShell / pwsh:**
  ```powershell
  forgum init pwsh | Out-String | Invoke-Expression
  ```
- **Bash:**
  ```bash
  eval "$(forgum init bash)"
  ```
- **Zsh:**
  ```bash
  eval "$(forgum init zsh)"
  ```
- **Fish:**
  ```fish
  forgum init fish | source
  ```
- **CMD:**
  ```cmd
  forgum init cmd
  ```

---

## 🩺 Diagnostics & Health Inspection

### Q: How do I verify my setup or debug terminal rendering issues?
**A:** Run Neovim-style health checks:
```bash
forgum checkhealth
```
This inspects:
- **System:** OS, architecture, PID, binary paths.
- **Config:** Syntax validation, file exclusivity, permissions.
- **Terminal:** Viewport size, TTY, TrueColor (24-bit RGB), DEC Mode 2026 Sync Update support, Sixel/Kitty graphics, and tmux/Zellij multiplexers.
- **Assets:** 109+ cow files and 10 dynamic DNA animation profiles.
- **Shell Hooks:** Active shell detection and profile integration snippets.
- **Daemon Herd:** Running background daemons and dead-daemon sweep status.
- **Structured Logs:** Text and JSONL log file accessibility and sizes.

For CI/CD or automation, pass `--json`:
```bash
forgum checkhealth --json
```

---

## 🪵 Logging & Daemon Life-Cycle

### Q: Can background daemons leak memory or linger forever?
**A:** No. Forgum implements a **Precmd Reaper** that runs via your shell's precmd hook (installed by `forgum init <shell>`). Every time your shell displays a new prompt, it checks for dead daemons, stale socket pipes, and orphaned lock files in under 1 millisecond and cleans them up automatically.

### Q: How do I inspect logs?
**A:**
```bash
# View recent logs in a formatted table
forgum logs

# Filter by severity
forgum logs --level warn

# Follow logs in real-time
forgum logs -f
```
Logs are stored in `~/.config/forgum/logs/` (or OS state directory) as human-readable `forgum.log` and machine-parseable `forgum.jsonl`.

---

## 🎨 Cows, DNA & Effects

### Q: How many creatures are available?
**A:** Over 109 creatures are embedded directly into the binary (including `default`, `tux`, `dragon`, `dolphin`, `koala`, `nyan`, `daemon`, `stegosaurus`, and many more). You can list them with:
```bash
forgum herd list
```

### Q: How do I add my own custom cow?
**A:** Drop any `.cow` ASCII template into `~/.config/forgum/cows/mycow.cow`. Forgum will automatically discover it!

---

## 📜 License

Forgum is free and open-source software released under the **[MIT License](../LICENSE)**. You are free to use, modify, distribute, and embed it in your own terminal configurations and tools.
