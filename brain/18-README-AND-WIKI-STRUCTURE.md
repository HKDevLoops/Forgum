# Forgum — README & Wiki Structure Plan (the documentation architecture)

> **The "eliminate config scripting, but document everything" blueprint.** Forgum's documentation has one north star: **a user should never need to write a config file by hand** — the interactive TUI (`17-…`) is the sanctioned path — **but every detail of what the config does, what APIs/powering it, and how to implement it must be exhaustively documented** for the users who want depth, and for the contributors building on top.
>
> This document specifies the **README** (the repo's front door), the **GitHub Wiki** page tree (the deep reference), the **per-page content spec** (what each page must contain: *what it is, how to use it, what API/mechanism powers it, how to set it in config, sample config + expected output*), the **sample-config library** (10+ annotated examples with outputs), and the **"expert configuration" corner** for power users. It is the map of the public docs; the internal engineering docs are the `00–17` planning set.
>
> Read after `16-CLI-DESIGN-…` and `17-TUI-CONFIG-MENU.md`.

---

## 0. The documentation constitution

1. **The TUI is the tutorial.** The README's quickstart is "install, run `forgum`, follow the cow." Everything else is reference.
2. **Every feature has a wiki page.** Not a section — a page. Discoverable, linkable, searchable. The wiki is the source of truth for *how things work*; the README is the *front door*.
3. **Every wiki page has the same 6-part skeleton:** (a) *What it is*, (b) *How to use it*, (c) *What powers it* (the API/mechanism/engineering), (d) *How to configure it* (TUI path + the config key), (e) *Sample config + expected output*, (f) *Troubleshooting*. A page missing any section fails the doc-CI gate.
4. **Sample configs always show output.** No "set this key" without "and here's what the terminal looks like after." Outputs are captured from real runs, ASCII-art-escaped, and versioned.
5. **Expert config is documented but de-emphasized.** The wiki has an "Expert configuration" section with the full `config.toml` schema, every key annotated, every value's effect shown — but the prose always says "or just toggle this in the TUI." Power users get depth; everyone else gets the toggle.
6. **Docs are versioned with the code.** The wiki's `_Sidebar.md` shows the current Forgum version; pages are tagged `[since v2.0]` / `[changed v2.1]` / `[deprecated v3.0]`. The README badges the latest release.
7. **Docs are testable.** Every sample config in the wiki is run through `forgum config validate` + a render in CI. A config that fails validation or renders an error fails doc-CI. No stale examples.

---

## 1. The README (`README.md`)

The README is the repo's front door. It must answer "what is this, how do I get it, how do I use it, where do I go deeper" in one scroll. Structure:

### 1.1 README outline

```
1.  Hero banner (animated GIF/Sixel of the flagship: dragon overlay above a live prompt)
2.  Badges (CI, crates.io, version, downloads, license, platform matrix, "made with Rust")
3.  One-paragraph pitch ("cowsay + fortune + lolcat, reimagined…")
4.  The flagship in one screenshot (the eye-tracking-cow TUI)
5.  Install (one-liner per platform — the 30-second path)
6.  Quickstart (3 commands: install, `forgum`, see the cow)
7.  Features (the headline list, with tiny GIFs)
8.  The interactive menu (one paragraph: "run `forgum` to configure everything")
9.  Shell integration (bash/zsh/fish/pwsh, one line each)
10. Platforms & install methods (table linking to wiki/Installation)
11. CLI cheat-sheet (the 15 most-used commands)
12. Where to go next (links to wiki pages: Getting Started, CLI Reference, TUI Guide, AI Features)
13. Contributing (link to CONTRIBUTING.md, one paragraph)
14. License + acknowledgements (cowsay, fortune, lolcat, the Rust ecosystem)
```

### 1.2 The install one-liners (section 5)

The README shows the **single fastest path per platform**, with a link to the wiki for the full menu:

```bash
# macOS (Homebrew)
brew tap forgum/forgum && brew install forgum

# Linux (universal — the interactive installer)
curl -fsSL https://forgum.dev/install.sh | bash

# Linux (Debian)
sudo apt install forgum            # after adding the apt repo (see wiki)

# Linux (Arch)
yay -S forgum-bin                  # AUR

# Linux (Nix)
nix run github:harish2222/Forgum

# Windows (winget)
winget install Forgum.Forgum

# Windows (scoop)
scoop bucket add forgum https://github.com/forgum/scoop-forgum && scoop install forgum

# Windows (PowerShell installer)
iex (irm https://forgum.dev/install.ps1)

# Any (cargo)
cargo install forgum
```

Each links to `wiki/Installation` for the deep dive (the interactive script, MSI/EXE, .deb/.rpm, etc.).

### 1.3 The quickstart (section 6)

```
1. Install (above).
2. Run:  forgum            # → opens the interactive TUI; follow the cow to configure
3. Done. Your prompt now has a cow. Type `forgum` again anytime to change things.
```

That's it. The README explicitly says: *"You do not need to edit any config file. Run `forgum` and use the menu."*

### 1.4 The CLI cheat-sheet (section 11)

A compact table of the 15 most-used commands (the full reference is `wiki/CLI-Reference`):

```
forgum                          # open the interactive config menu (TUI)
forgum init                     # first-run setup / reconfigure / change shell
forgum --cow dragon             # render a dragon once
forgum --cow dragon --effect rainbow --duration 5   # 5-second rainbow dragon
forgum fortune                  # a fortune
forgum herd 5                   # a 5-cow herd
forgum daemon start             # start the overlay daemon
forgum doctor                   # verify install integrity
forgum upgrade                  # upgrade via your install method
forgum ai why                   # why was this cow chosen? (AI)
forgum find "sleepy dragon"     # semantic cow search (AI)
forgum tone doom                # switch tone (AI)
forgum chat                     # conversational cow REPL (AI)
forgum tmux demo                # the showcase reel
forgum --help                   # everything else
```

### 1.5 README badges

```
CI: GitHub Actions status
crates.io: version + downloads
license: MIT
platforms: Linux · macOS · Windows (with arch icons)
made with: Rust (with version)
discussions / discord link
```

---

## 2. The GitHub Wiki — page tree

The wiki is the deep reference. Pages (the `_Sidebar.md` groups them):

### Home
- **Home** — the wiki front door: "new here? start with Getting Started." Links to the 4 entry points.

### Getting Started
- **Getting-Started** — install → `forgum` → first cow, in 5 minutes. Screenshots.
- **Installation** — the full matrix: every platform, every method, the interactive installer, MSI/EXE, .deb/.rpm, AUR, Nix, Homebrew, winget, scoop, cargo. Troubleshooting per method. (Mirrors `15-…`.)
- **Quickstart** — the 10 commands you'll use 90% of the time.
- **The-Interactive-Menu** — the TUI: what it is, how to navigate, the eye-tracking cow, the toggle mechanism, lolcat. (Mirrors `17-…`.) Heavy on screenshots/GIFs.

### Reference
- **CLI-Reference** — every subcommand + every flag, one section each, with examples. (Mirrors `16-…`.) Auto-generated from `forgum --help` markup + hand-written examples.
- **Configuration** — the full `config.toml` schema, every key annotated, with "set in TUI: Settings → …" callouts for each. The expert corner lives here.
- **Cows** — the 109-cow gallery, each with its ASCII art + 9-axis mood DNA badge + archetype. Auto-generated from `cow_dna.json`.
- **Effects** — the 19 effects, each with a preview GIF + tunable parameters.
- **Themes** — the built-in themes + how to create/import custom themes.
- **Shells** — bash/zsh/fish/pwsh integration: what the hook does, where it's installed, how to customize.

### The Animation Engine
- **Animation-Engine** — overview: the 3-thread model, the `Renderer` trait, hardware vs software. (Mirrors `11-…`, user-facing tone.)
- **Overlay-Mode** — the flagship: animation above the prompt, how it works, why it doesn't steal keystrokes, how to troubleshoot. (Mirrors `03-…`.)
- **Per-Animal-Animation** — the 7-axis DNA, why every cow animates uniquely, how to read the DNA table. (Mirrors `12-…`.)

### AI Features
- **AI-Overview** — the v3 vision: classification, command-aware selection, contextual thoughts, sentiment-adaptive animation, voice, the REPL. (Mirrors `14-…`, user-facing.)
- **AI-Privacy** — the local-first model, the redactor, what's stored, how to purge. **Prominent.**
- **AI-Models** — how to install local models, what each does, size/RAM requirements, cloud opt-in.
- **Tone-Modes** — the 6 tones with example thoughts.
- **Voice** — TTS moos + STT commands.
- **Conversational-REPL** — `forgum chat`, personas, memory.

### Integrations
- **tmux** — the plugin, the 4 surfaces, `forgum demo`. (Mirrors `05-…`.)
- **rmux** — remote/follow-me sessions.
- **herdr** — the daemon fleet manager.
- **Multiplexers** — zellij, screen, wezterm.

### For Package Maintainers & Contributors
- **Packaging** — how to build .deb/.rpm/MSI/AUR/Nix, the manifest templates, the PR process for winget/brew/nixpkgs. (Mirrors `15-…` §8.)
- **Contributing** — pointer to `CONTRIBUTING.md` + the dev setup deep dive.
- **Architecture** — the workspace layout, the 10 principles, the crate graph. (Mirrors `06-…` + `10-…`.)
- **Testing** — the 8-tier pyramid, how to run tests, how to add a golden. (Mirrors `13-…` + `14-…` §11.)

### Misc
- **Troubleshooting** — `forgum doctor` output decoded, common fixes, "my prompt is broken" recovery.
- **FAQ** — the questions that come up in issues.
- **Changelog** — per-version user-facing changes (link to `CHANGELOG.md`).
- **Deprecations** — the active deprecation list + timeline.
- **Glossary** — cow, archetype, DNA, overlay, daemon, redactor, intent cluster, etc.

**Total: ~35 pages.** Each follows the 6-part skeleton.

---

## 3. The per-page content skeleton (the doc-CI gate)

Every wiki page must contain these 6 sections, in order. A doc-CI script greps each page for the section headers and fails if any are missing.

```markdown
# <Page Title>

> One-sentence summary of what this page covers.

## What it is
<2–4 paragraphs: the concept, the why, the user-facing mental model.>

## How to use it
<Step-by-step, copy-pasteable commands. Screenshots/GIFs where visual.>

## What powers it
<The engineering: the crate, the trait, the API, the algorithm. For curious users
and contributors. Names the exact functions/types. Links to source.>

## How to configure it
<Two paths, always:
  1. In the TUI: Settings → <Section> → <control>. (screenshot)
  2. Via config.toml: the key, its type, its default, its range.>

## Sample config + output
<An annotated config.toml snippet + the exact terminal output it produces.
Captured from a real run. Versioned.>

## Troubleshooting
<3–6 common problems + fixes. Links to FAQ/Troubleshooting page.>
```

### 3.1 Example: the "Tone Modes" page (skeleton filled)

```markdown
# Tone Modes

> How to make your cow speak in character — sarcastic, zen, hype, doom, or pirate.

## What it is
Every cow's thought is generated in-character by default. Tone modes override the
*persona* the LLM adopts, giving you 6 switchable voices for the same cow. A dragon
in `doom` tone is bleak and funny; the same dragon in `hype` is maximal enthusiasm.

## How to use it
  forgum tone doom        # switch for this session
  forgum tone default     # back to cow-DNA-driven
  # or in the TUI: Settings → AI → Tone → [doom ▾]

## What powers it
Tone is injected into the LLM system prompt (see `crates/ai/src/thought.rs`,
`ThoughtContext::system_prompt()`). The 6 tones are enum variants in
`ai::ToneMode`; each prepends a persona clause. The cow's 9-axis mood DNA still
constrains the voice — a turtle in `hype` is gentler than a dragon in `hype`.
Powered by the local Llama-3.2-1B by default; cloud if opted in.

## How to configure it
TUI:  Settings → AI → Tone → select from dropdown
CLI:  forgum config set ai.tone doom
TOML: ai.tone = "doom"   # one of: default|sarcastic|zen|hype|doom|pirate

## Sample config + output
  # ~/.config/forgum/config.toml
  [ai]
  enabled = true
  tone = "doom"

  $ git push
  (exit 0)
  ┌─────────────────────────────────┐
  │ you shipped. the heat death is  │
  │ unchanged.                      │
  └─────────────────────────────────┘
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||

  $ forgum tone pirate
  $ git push
  ┌─────────────────────────────────┐
  │ ye cargo's aloft, cap'n. fair   │
  │ winds.                          │
  └─────────────────────────────────┘

## Troubleshooting
- "Tone has no effect" → AI is off; run `forgum` → AI → enable.
- "Tone feels wrong for my cow" → the cow DNA constrains it; try a different cow.
- "Tone repeats" → cache; `forgum thought --regenerate` or wait for rotation.
```

This is the template every page follows. Doc-CI validates the headers + that the sample config runs clean.

---

## 4. The sample-config library (10+ annotated examples)

A dedicated wiki page **Sample-Configs** collects 10+ complete, annotated `config.toml` files, each with the terminal output it produces. These are the "expert corner" — power users copy-paste; everyone else uses the TUI. Examples:

1. **Minimal** — defaults, just installed.
2. **The dragon prompt** — overlay on, dragon, rainbow, 60 fps.
3. **The zen turtle** — turtle, slow, muted palette, reduced motion.
4. **The AI companion** — AI on, local models, doom tone, command-aware.
5. **The cloud-powered** — AI on, cloud opt-in, multi-language (es).
6. **The privacy-hardened** — AI on, cloud off, blocklist, intent log off.
7. **The tmux herder** — tmux plugin, herd of 5, rmux follow-me.
8. **The accessible** — narrator on, high contrast, reduced motion, large font.
9. **The kid's terminal** — nyan, confetti, hype tone, voice on.
10. **The CI runner** — `render.quiet`, no overlay, single-frame render for logs.
11. **The developer** — debug logging, predictive pre-render, pair-programming on.

Each is a full `config.toml` + a screenshot/GIF of the resulting prompt. **Every one is validated by doc-CI** (`forgum config validate` + a render snapshot).

---

## 5. The "Expert configuration" corner

For the 20-year veterans (the user's own profile), the wiki's **Configuration** page has a deeply-annotated schema reference:

```toml
# ~/.config/forgum/config.toml — full annotated reference
schema_version = 3            # auto-migrated; do not edit

[render]
cow = "default"               # any of the 109 cow IDs; `forgum find "..."` to search
eyes = "default"              # default|happy|borg|dead|greedy|paranoid|stoned|wired|tongue
effect = "rainbow"            # see wiki/Effects; "none" for static
duration = 0                  # 0 = daemon infinite / fg single-frame; seconds otherwise
width = 40                    # speech-bubble width in cols
think = false                 # true = think-bubbles (( )) instead of speech (< >)
color = true                  # false respects NO_COLOR
gpu = "auto"                  # auto|on|off; auto-probes kitty-graphics/wgpu
fps = 60                      # target; daemon adaptive 15/30/60
quiet = false                 # suppress non-essential output

[animation]
mode = "breathing"            # the v2 base animation; see wiki/Per-Animal-Animation
speed_mult = 1.0              # multiplies the per-cow DNA speed
palette = "default"           # default|victory|ember|ocean|forest|custom (see [theme])

# ... every key, annotated, with the TUI path callout:
# (TUI: Settings → Animation → Speed)
```

Every key gets: type, default, range, the TUI path to change it, and a one-line effect description. The page opens with: *"You usually don't need to read this — run `forgum` and use the menu. This page is for users who prefer to manage config as code, for CI/CD, and for contributors."*

---

## 6. Doc-CI (the gate that keeps docs honest)

A `docs-ci` GitHub Actions job:

1. **Skeleton check**: for each wiki page, assert the 6 section headers exist.
2. **Config validation**: for each sample config in `Sample-Configs` + page samples, run `forgum config validate --file <sample>`; fail on error.
3. **Render snapshot**: for each sample, run `forgum render --config <sample> --output frame.txt` and assert the blake3 matches the committed snapshot (catches "the example drifted from reality").
4. **Link check**: all internal wiki links resolve; all external links return 200.
5. **Version tag check**: every page has a `[since vX.Y]` tag in its frontmatter.
6. **Help sync**: `forgum --help` + every subcommand's `--help` is diffed against `wiki/CLI-Reference`; fail on drift (forces docs to track CLI changes).

This runs on every PR touching `docs/` or the CLI, and nightly.

---

## 7. The README ↔ wiki ↔ CONTRIBUTING relationship

- **README.md** (in repo): front door, install, quickstart, cheat-sheet, links to wiki. Kept short — one scroll.
- **CONTRIBUTING.md** (in repo, see `19-…`): how to contribute, one-go-understandable.
- **GitHub Wiki**: the deep reference. Every feature, every key, every command, with samples + outputs.
- **`docs/` directory** (in repo): the source for diagrams, the sample configs, the doc-CI fixtures. The wiki is generated/mirrored from here where possible (so docs are versioned with code, not just on the wiki host).

The flow: a user lands on README → installs → runs `forgum` → the TUI teaches them. When they want depth, they hit the wiki. When they want to contribute, they hit CONTRIBUTING. Three doors, one building.

---

## 8. The one-paragraph summary

Forgum's documentation is a three-door building: a short **README** (install one-liners per platform, quickstart, CLI cheat-sheet, links deeper), a deep **GitHub Wiki** (~35 pages, each following a strict 6-part skeleton — *what it is, how to use it, what powers it, how to configure it, sample config + output, troubleshooting*), and a **CONTRIBUTING.md** for contributors. The north star is "eliminate config scripting": the interactive TUI is the sanctioned configuration path, and every doc page says so — but the wiki's "Expert configuration" corner and the **Sample-Configs** library (10+ annotated full configs with their terminal outputs) give power users and CI/CD the depth they need. A **doc-CI** job validates every page's skeleton, runs every sample config through `forgum config validate`, snapshots every render, checks links, and diffs `--help` against the CLI reference — so the docs can never silently drift from reality. The README is the front door, the wiki is the reference, the TUI is the tutorial; together they make Forgum a project where the user never needs to write a config file by hand, but can find out exactly how every byte of one works if they want to.
