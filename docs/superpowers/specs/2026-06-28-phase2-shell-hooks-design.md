# Phase 2 — Shell Hooks That Work + Native Cow Renderer

> **Goal:** `forgum init bash/zsh/fish/pwsh` produces a hook that renders the right cow,
> eyes, effect, lolcat — read live from config — without breaking the prompt.
> Engine renders cows natively (no system `cowsay`). `[dep: 1]` `[par: 3]`

**Invariants hardened:** #2 (RAII on shell side), #9 (tests are spec).

---

## 1. Architecture

```
forgum init bash
    │
    ▼
forgum-engine init bash  (single source of truth, in Rust)
    │
    ├── emits bash hook:
    │   1. resolves engine path (baked at init time)
    │   2. forgum() calls engine with argv flags (no JSON in shell)
    │   3. precmd sweeps dead daemons
    │
    └── stdout = hook script → user pastes into ~/.bashrc
```

**Decisions:**
1. `forgum-engine init <shell>` is the only hook generator. Kills BUG-S6.
2. Engine renders cows natively — loads `.cow`, expands `$eyes/$tongue/$thoughts`, wraps in speech bubble. Kills BUG-S4.
3. Engine reads config via `--config` flag. Hook never parses JSON. Kills BUG-S3/S7.
4. Hook passes `--cow`, `--eyes`, `--text` as argv flags. Engine assembles internally. Kills BUG-S1/S2.

---

## 2. Task Breakdown

| Task | Bug | Deliverable |
|------|-----|-------------|
| 2.1 Add clap + clap_complete | (new) | Structured CLI with subcommands |
| 2.2 Native cow renderer | S4 | Load `.cow`, expand `$eyes/$tongue/$thoughts`, speech bubble |
| 2.3 Config file reading | S3,S7 | Engine reads config.json; argv > file > defaults |
| 2.4 Fortune subcommand | (new) | `forgum-engine fortune` picks random line |
| 2.5 Shell hook generation | S1,S2,S5,S6 | `forgum-engine init bash/zsh/fish/pwsh` |
| 2.6 Completions subcommand | I3 | `forgum-engine completions <shell>` via clap_complete |
| 2.7 Precmd sweep | (new) | Dead daemon cleanup in all hooks |
| 2.8 Sample data | (new) | Default cow + fortune files in data/ |

---

## 3. Cow File Format

Standard cowsay `.cow` format:

```
$the_cow = <<EOC;
        $eyes
   ─────$thoughts─────
  / $tongue            \\
  │                     │
  \\____   U    ____/
       \\_______/
           (   )
           (   )
           U U
EOC
```

The engine:
1. Reads the `.cow` file
2. Extracts `$the_cow = <<EOC;...EOC;`
3. Replaces `$eyes` → user's eyes (default `oo`)
4. Replaces `$tongue` → user's tongue (default `  `)
5. Replaces `$thoughts` → user's thoughts (default `\\`)
6. Wraps the `--text` in a speech bubble above the cow
7. Renders the combined output into the framebuffer

---

## 4. Config Merge Chain

```
argv flags  >  config.json  >  built-in defaults
   (highest)      (middle)        (lowest)
```

Config file path priority:
1. `--config <path>` flag
2. `$FORGUM_CONFIG` env var
3. `$XDG_CONFIG_HOME/Forgum/config.json` (Linux/macOS)
4. `%APPDATA%\Forgum\config.json` (Windows)

---

## 5. Test Gate (Phase 2 DoD)

- [ ] `forgum-engine init bash` emits valid bash hook
- [ ] `forgum-engine init zsh` emits valid zsh hook
- [ ] `forgum-engine init fish` emits valid fish hook
- [ ] `forgum-engine init pwsh` emits valid pwsh hook
- [ ] `forgum-engine fortune` returns a non-empty string
- [ ] `forgum-engine render --cow default --text "hi"` renders cow + bubble
- [ ] `forgum-engine completions bash` emits valid completion script
- [ ] `forgum-engine --version` works
- [ ] All 99+ Rust tests pass
- [ ] Clippy clean, fmt clean
