# Forgum — The TUI Config Menu (modern, 3D, with an eye-tracking cow)

> **The flagship interactive surface.** Running `forgum` with no arguments (or `forgum tui`, or `forgum init`) launches a full-screen **terminal UI** that is the single, sanctioned way to configure every aspect of Forgum — appearance, animation, cows, effects, AI, shell, integrations, voice, privacy, and install state. It is built with **`ratatui`** on top of the v2 render engine, styled to feel like a **3D-rendered page** (depth, shadows, parallax, beveled panels, glow), saturated with **lolcat rainbow gradients**, and anchored by a **hero cow whose eyes track the focused widget and the terminal cursor**, blink on selection, and shift mood per section.
>
> This document specifies the **design language**, the **eye-tracking cow**, the **toggle mechanism** (no free-text unless validating), the **menu tree** (every config key reachable — the "no orphan config" invariant), **lolcat integration**, **accessibility**, **persistence** (atomic writes, diff-before-apply, undo), and the **expert escape hatch** (`e` → `$EDITOR` on the raw TOML). It is the user's primary interaction surface; the CLI (`16-…`) is the contract beneath it.
>
> Read after `11-ENGINE-INTERNALS-V2.md`, `12-PER-ANIMAL-ANIMATION-DESIGN.md`, and `16-CLI-DESIGN-…`.

---

## 0. The design constitution of the TUI

1. **`forgum` opens the menu.** The bare keyword, with no args, launches the TUI. This is the "interactive menu is enough" rule made literal — the user's muscle memory is `forgum` → menu → change a thing → done.
2. **Toggle, don't type.** Every setting is a **toggle, a selectable list, a slider, or a multi-select**. Free-text entry exists only for paths/URLs/identifiers and is validated on commit. No user ever needs to know TOML syntax to configure Forgum. (Expert escape hatch in §8.)
3. **Every config key is reachable.** The "no orphan config" invariant: for every key in `config.toml`, there is a TUI control. CI enumerates the schema and asserts each key has a widget. If a key is added to the schema without a TUI control, CI fails.
4. **The cow is the host.** A live, animated cow occupies a dedicated region of every screen. Its **eyes track** the currently-focused widget (and, when the terminal cursor moves within a text field, the cursor). It **blinks** on selection, **shifts mood** per section (a dragon in the "AI" section, a turtle in "Privacy"), and **reacts** to errors with a confused expression. The cow is not decoration; it is the feedback channel.
5. **3D feel, 2D terminal.** Depth is faked with: drop shadows (half-block chars at 50% alpha), beveled panel borders (light top-left / dark bottom-right), parallax on scroll (background stars drift slower than foreground), and a subtle perspective skew on the hero cow. No actual 3D pipeline — this is the v2 ANSI/kitty renderer doing what it does best, plus careful shading.
6. **Lolcat is always present.** The title, the focused widget's accent, the selected cow's preview, and the hero cow itself cycle through a **lolcat rainbow** (the classic HSV hue rotation, implemented as the v2 OKLCH pipeline's hue-shift mode). `--no-color` / `NO_COLOR` / accessibility "reduced motion" modes flatten it to the theme's accent.
7. **Atomic persistence.** "Apply" writes `config.toml` to a temp file, `fsync`s, renames over the original. A `.bak.{timestamp}` is kept. Undo/redo within the session (Ctrl-Z / Ctrl-Y) operates on the in-memory draft. Nothing is written until the user confirms a diff.
8. **Keyboard-first, mouse-optional.** Every action has a keybinding. Mouse is supported (click to focus, scroll to navigate) but never required. This is accessibility table-stakes.

---

## 1. The visual design language

### 1.1 Layout (the 3-zone shell)

Every screen of the TUI shares a 3-zone layout:

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ★ FORGUM                                                  [?] help  [q] quit │  ← header (lolcat title, context keys)
├────────────────────┬─────────────────────────────────────────────────────┤
│                    │                                                     │
│   SIDEBAR          │              MAIN PANEL                             │
│   (sections)       │              (the active section's controls)        │
│                    │                                                     │
│   ▸ Appearance     │   ┌───────────────────────────────────────────┐     │
│     Animation      │   │  Cow:      [dragon          ▾]            │     │
│     Cows           │   │  Eyes:     [happy           ▾]            │     │
│     Effects        │   │  Effect:   [rainbow         ▾]            │     │
│     AI             │   │  Speed:    ━━━━●━━━━━  1.0x               │     │
│     Shell          │   │  Palette:  [victory gold    ▾]            │     │
│     Integrations   │   │  GPU:      [● on]                         │     │
│     Voice          │   │  FPS cap:  ━━━━━━●━━  60                  │     │
│     Privacy        │   │                                             │     │
│     Accessibility  │   │  [ live preview: the cow renders here ]   │     │
│   ▸ About          │   │                                             │     │
│                    │   └───────────────────────────────────────────┘     │
│   ┌──────────────┐ │                                                     │
│   │              │ │                                                     │
│   │  HERO COW    │ │   (footer: keybindings)                             │
│   │  eyes track  │ │   ↑↓ navigate · enter toggle · e edit raw · apply  │
│   │  the cursor  │ │                                                     │
│   └──────────────┘ │                                                     │
└────────────────────┴─────────────────────────────────────────────────────┘
```

- **Header**: the lolcat-rainbow `★ FORGUM` title (animated hue drift), the current section name, and the global keybindings (`?` help, `q` quit, `Ctrl-S` apply, `Ctrl-Z` undo).
- **Sidebar**: the section list, navigable with ↑↓. The focused section's icon animates (the cow's eyes track it).
- **Main panel**: the active section's controls — toggles, dropdowns, sliders, multi-selects, and a **live preview** region that re-renders the cow as you change settings (using the v2 engine, throttled to 30 fps in the TUI).
- **Hero cow**: a persistent cow in the sidebar's lower third. Its eyes follow the focused widget in the main panel. Its mood/archetype matches the active section (see §2.3). Its body idles with the v2 breathing animation. The hero cow is rendered by the **same engine** that does the overlay — no separate renderer.
- **Footer**: contextual keybindings.

### 1.2 The 3D feel (depth without a GPU)

| Technique | Implementation | Where |
|-----------|----------------|-------|
| **Drop shadow** | half-block `▀`/`▄` chars at the panel's bottom-right edge, foreground = panel color × 0.5 alpha (the v2 OKLCH pipeline supports alpha) | every panel |
| **Beveled border** | `┌┐└┘` corners + light horizontal/verticals on top-left, dark on bottom-right (simulated emboss) | every panel |
| **Parallax** | a starfield/dot-grid background in the empty regions; on sidebar scroll, background shifts by 1 cell for every 3 the foreground moves | sidebar + main panel |
| **Panel stack depth** | the focused panel is drawn at full brightness; unfocused panels dim to 70% and shift 1 column right (fake Z-offset) | multi-panel screens |
| **Hero cow perspective** | a subtle 2-3% horizontal squash + top-edge highlight to imply the cow is "in front of" the panel | hero cow region |
| **Glow on focus** | the focused toggle gets a 1-cell OKLCH glow halo (the v2 Gaussian-glow particle, repurposed) | focused widget |
| **Selection pulse** | the selected item pulses (breathing easing) at 0.5 Hz — never fast enough to distract | list items |

These are all achievable with the v2 software renderer (ANSI 24-bit + the kitty-graphics backend for the glow if available). The "3D" is an aesthetic, not a real pipeline — which keeps it working on every terminal.

### 1.3 Color & lolcat

- **Base palette**: the user's theme (default "forgum-dark": near-black bg, warm amber accent). Defined in OKLCH.
- **Lolcat accent**: the title, the focused-widget glow, the selected cow preview, and the hero cow's body cycle hue over 2 s (classic lolcat: `hue = (frame * 2.5) % 360`). Implemented as the v2 pipeline's `effect = "rainbow"` mode applied selectively.
- **`--no-color` / `NO_COLOR`**: lolcat disabled, glow becomes a solid amber outline, depth becomes solid borders. The TUI remains fully usable.
- **Accessibility "high contrast"** (`forgum config set accessibility.high_contrast true`): pure black/white/yellow, no gradients, thicker borders.

---

## 2. The eye-tracking cow (the hero)

This is the signature interaction. The hero cow is **alive** in the menu.

### 2.1 Eye mechanics

The cow's eyes are two characters in its ASCII art (the standard `OO` or `oo`). In the TUI, the eye region is **dynamically replaced** based on where the user's focus is:

- The TUI computes the **screen-space position of each eye** (fixed, from the cow layout).
- It computes the **screen-space position of the focused widget's center** (or the terminal cursor, if in a text field).
- It computes the **direction vector** from eye to target.
- It selects the eye glyph from a 9-direction set: `•` (center), `˙` (up), `.` (down-left), `›` (right), `‹` (left), `⌐` `¬` (diag), etc. — drawn from a small bitmap atlas so they render cleanly in any font.
- Both eyes look at the **same** target (mammalian vergence), with a tiny per-eye offset for liveliness.

```
   focused widget is              focused widget is              terminal cursor
   up-and-right:                  down-and-left:                 in a text field,
                                                                  eyes track the caret:
        ____                        ____
       /    \                      /    \
      | ˙  ˙ |                    | .  . |
      |      |                    |      |
       \____/                      \____/
```

### 2.2 Blink & react

- **Blink**: every 4–7 s (jittered, so it feels organic), the eyes close for 120 ms (rendered as `--`/`==`). On **selection** (enter pressed on a toggle), the cow does a single deliberate blink as confirmation.
- **Mood reaction**: 
  - On **success** (apply saved): cow does a happy eyes `^^` + a tiny bounce (2-particle burst).
  - On **error** (validation failed): cow does confused eyes `¿¿` + a head-tilt (the ASCII shifts 1 col).
  - On **undo**: cow does a "hmm" `··` eyes.
  - On **section change**: the cow morphs to the section's archetype (see 2.3).
- **Idle**: between interactions, the cow breathes (the v2 base animation, slow scale on the body) and occasionally glances around (eyes drift to random widgets for 800 ms, then back).

### 2.3 Section archetypes (the cow matches the page)

| Section | Hero cow archetype | Why |
|---------|-------------------|-----|
| Appearance | **Dragon** (majestic, slow) | "you are shaping the look" |
| Animation | **Dragon** (energetic) | motion is the dragon's domain |
| Cows | **Cat** (curious, eyes darting) | browsing the herd |
| Effects | **Nyan** (chaotic, rainbow) | effects = particles = nyan |
| AI | **Sage** (owl, slow blink, wise) | "the cow is thinking" |
| Shell | **Tux** (penguin, formal) | shell = the penguin's home |
| Integrations | **Octopus** (multi-armed) | reaching into tmux/rmux/herdr |
| Voice | **Whale** (singing) | voice = song |
| Privacy | **Turtle** (hides in shell) | "safety" |
| Accessibility | **Dog** (guide dog, attentive) | "helpful" |
| About | **Default cow** (the classic) | home base |

The archetype is selected from the cow library's `archetype` axis (the v3 9-axis DNA, `14-…` §2.1). The transition is a 300 ms crossfade (two cows overlaid, alpha shifting) so it never feels like a jump cut.

### 2.4 The cow is the feedback channel

Because the cow reacts to every action, the user doesn't need to read a status bar — the cow *is* the status. This is the design principle: **the host character is the UI feedback**, not a separate toast/log region. (Errors that need text still print to a small log line in the footer, but the cow's expression is the primary signal.)

---

## 3. The toggle mechanism (the interaction primitive)

Every setting is one of five widget types. **No free-text** unless it's a path/URL/identifier, and those validate on blur.

| Widget | For | Interaction | Example |
|--------|-----|-------------|---------|
| **Toggle** | boolean settings | `enter` flips ●○ | `GPU: [● on]` |
| **Select** | enum / single-choice | `enter` opens dropdown, ↑↓ + enter | `Cow: [dragon ▾]` |
| **Multi-select** | sets | `space` toggles membership | `Cows in herd: [✓ dragon ✓ turtle □ cat]` |
| **Slider** | numeric ranges | `←→` or `h l`, `enter` to type | `Speed: ━━━━●━━━━━ 1.0x` |
| **Validated text** | paths/urls/ids | type, validated on blur, red border on error | `Library path: ~/.local/share/forgum/cows` |

**Toggle navigation model:**
- `↑`/`↓` (or `k`/`j`): move focus between widgets.
- `enter` (or `space`): activate the focused widget (toggle / open dropdown / start slider drag / focus text).
- `←`/`→` (or `h`/`l`): change value (for toggle: wrap; for slider: step; for select: cycle).
- `esc`: close dropdown / cancel text edit.
- `Ctrl-S`: apply (write config).
- `Ctrl-Z` / `Ctrl-Y`: undo / redo (in-session draft).
- `e`: expert escape — open the raw TOML for this section in `$EDITOR` (§8).
- `/`: fuzzy-search the controls (jumps focus to the match).
- `?`: help overlay (all keybindings).

**Every toggle has a live preview.** Change `Cow` to `dragon`, the preview region (and the hero cow, if applicable) re-renders immediately. Change `Speed`, the animation visibly speeds up. This closes the feedback loop — the user never wonders "what did that do?"

---

## 4. The menu tree (every config key reachable)

The sidebar's 11 sections, each a screen of toggles. **Every key from the config schema (`16-…` §4.1) appears here.** This is the "no orphan config" invariant made concrete.

### Appearance
`render.color`, `render.quiet`, `theme.name`, `theme.custom.*`, accessibility high-contrast toggle, lolcat on/off, lolcat speed.

### Animation
`animation.effect`, `animation.mode`, `animation.speed_mult`, `animation.palette`, `render.fps`, `render.gpu`, reduced-motion toggle. Live preview front-and-center.

### Cows
`cows.library_path`, `cows.blocklist[]` (multi-select from the 109), `cows.favorites[]`, a browsable gallery of all 109 cows (each with its 9-axis mood DNA badge from `14-…`), `forgum new "<desc>"` launcher (F15).

### Effects
Per-effect toggles + tuning (the 19 effects): rainbow, bounce, wave, fade, glitch, shake, zoom, typewriter, matrix, fire, bubbles, stars, snow, confetti, lightning, pulse, scanline, static, none. Each with a mini-preview.

### AI (the v3 horizon, `14-…`)
`ai.enabled`, `ai.default_backend`, `ai.cloud`, `ai.lang`, `ai.tone`, `ai.voice`, `ai.max_thought_words`, `ai.features.*` (error_explain, suggest_next, predictive_prerender, pair_programming, fortune_feed), `ai.models.*` (install/verify/list), the 24 intent clusters (view/disable per cluster), a "classify all cows now" button (F01).

### Shell
Detected shell(s), rc file path(s), hook-injected status (toggle re-inject), completions installed (toggle reinstall), a "test the hook" button that runs a precmd and shows the sweep output.

### Integrations
`integrations.tmux` (install/uninstall plugin), `integrations.rmux`, `integrations.herdr` (daemon fleet manager), `integrations.wezterm`, `forgum demo` launcher.

### Voice
`ai.voice` master, TTS backend (Piper local / z-ai cloud), voice per archetype, STT wake-word sensitivity, `forgum voice test` button, accessibility narrator toggle (F24).

### Privacy (the most important screen, paired with the turtle)
`ai.privacy.redact_home_paths`, `ai.privacy.blocklist_commands[]` (add/remove patterns), `ai.privacy.intent_log` (toggle + purge button), `ai.privacy.telemetry` (off by default), a "show me exactly what's stored" button (`forgum privacy show`), a "purge everything" button (`forgum privacy purge`), the redactor pattern list (read-only, for transparency).

### Accessibility
`accessibility.narrator` (F24, screen-reader alt-text + TTS), `accessibility.high_contrast`, `accessibility.reduced_motion` (disables lolcat drift, parallax, pulse; slows animations), `accessibility.font_scale`, keyboard-only confirmation (all mouse actions have keybindings, tested).

### About (read-only)
`install.method`, `install.version`, `install.canonical_path`, binary/engine/AI versions, the active renderer backend, a "run `forgum doctor`" button, a "check for updates" button (`forgum upgrade --check`), links to the wiki + contributing guide + issue tracker.

**The CI test for "no orphan config":** a test loads the config schema, walks the TUI's widget tree (via the `ratatui` test harness), and asserts every schema key has a widget. Missing widget → fail. This is how the invariant stays true as the schema grows.

---

## 5. Lolcat integration (always present, the user asked)

The classic `lolcat` effect (HSV hue rotation across the text) is Forgum's house style. In the TUI:

- **Title** `★ FORGUM`: hue-drifts continuously (2 s cycle).
- **Focused widget accent**: the glow + border cycles hue.
- **Selected cow preview**: the cow's body is lolcat'd.
- **Hero cow**: the body is lolcat'd at a slower cycle (4 s) so it doesn't compete with the title.
- **Apply success flash**: on save, the whole screen does a 400 ms lolcat sweep as confirmation.
- **`forgum demo`**: the showcase reel is full lolcat.

**Implementation:** the v2 OKLCH pipeline's `effect = "rainbow"` is the engine. For the classic lolcat look (per-character hue offset by column), the renderer exposes a `LolcatMode::PerColumn { speed, spread }`. This is the same code path that powers `forgum render --effect rainbow`, so the TUI gets it for free. `NO_COLOR` flattens it; `accessibility.reduced_motion` slows it to a static gradient.

---

## 6. Persistence & safety

### 6.1 The draft model

The TUI edits an **in-memory draft** (a clone of the loaded config). Nothing is written until the user presses `Ctrl-S` (Apply) or navigates away with unsaved changes (which prompts "save? y/n"). This means exploration is free — flip toggles, see previews, walk away, nothing persisted.

### 6.2 Atomic write

On Apply:
1. Serialize the draft to TOML.
2. Write to `config.toml.tmp`.
3. `fsync` the temp file.
4. `rename` over `config.toml` (atomic on all 3 platforms).
5. Copy the prior `config.toml` to `config.toml.bak.{timestamp}` (keep last 5).

### 6.3 Diff-before-apply

Apply shows a **diff screen** first:

```
   ┌─ Apply changes? ──────────────────────────────┐
   │                                                │
   │   render.cow          = "default"  → "dragon"  │
   │   animation.effect    = "rainbow"  → "bounce"  │
   │   animation.speed_mult= 1.0        → 1.3       │
   │   ai.enabled          = false      → true      │
   │                                                │
   │   [Y] apply   [N] cancel   [E] edit raw        │
   └────────────────────────────────────────────────┘
```

`Y` writes; `N` returns to the menu; `E` opens the raw TOML diff in `$EDITOR` for the truly expert.

### 6.4 Undo / redo

In-session: `Ctrl-Z` reverts the last toggle change, `Ctrl-Y` redoes. Stack is unbounded for the session. (Cross-session undo = the `.bak` files; `forgum config restore --bak <ts>`.)

### 6.5 Validation

Every change is validated against the schema **before** it enters the draft. An invalid value (e.g., `fps: 0`) is rejected at the widget level — the toggle won't accept it, the cow does its confused face, and a footer message explains why. The draft is therefore always valid; Apply never fails on schema.

---

## 7. Accessibility (non-negotiable)

- **Keyboard-first**: every action has a keybinding, documented in `?`. Mouse is additive.
- **Screen reader**: the TUI emits ARIA-equivalent announcements to stderr (the `ratatui` `ScreenReader` bridge, or via the platform's accessibility API) describing focus changes. The accessibility narrator (F24) can speak them via TTS.
- **High contrast** mode: pure black/white/yellow, no gradients.
- **Reduced motion**: disables lolcat drift, parallax, pulse, blink; slows animations to 0.5×; the eye-tracking cow still works (it's informational, not decorative) but without the breathing.
- **Font scale**: the TUI respects `accessibility.font_scale` (1.0–2.0) by scaling the layout grid.
- **No flashing**: no animation exceeds 3 Hz (photosafety). The selection pulse is 0.5 Hz.
- **`forgum` with no TTY** (piped): the TUI detects non-interactive stdout and falls back to printing the config as TOML (so `forgum | cat` works in scripts).

---

## 8. The expert escape hatch

Power users (the 20-year veterans) want raw access. The TUI honors this without compromising the "toggle, don't type" principle for everyone else:

- **`e` on any section** opens that section's TOML in `$EDITOR`. Save + quit → the TUI re-loads, validates, and shows the diff. If invalid, the cow does its confused face and the edit is rejected with the parse error.
- **`forgum config edit`** opens the whole `config.toml` in `$EDITOR` (the CLI escape hatch, `16-…`).
- **`forgum config import <url|path>`** imports a config (validated, diffed).
- **`forgum config export <path>`** exports the current effective config.

The escape hatches are documented in the wiki (`18-…`) under "Expert configuration" — clearly marked as "for users who know TOML and want full control," with sample configs and their outputs.

---

## 9. First-run vs. reconfigure vs. `forgum` (no args)

| Entry | What it does |
|-------|--------------|
| `forgum init --first-run` (called by installers) | Detect env → TUI with "Welcome" screen → quick-start or custom → apply → inject shell hook → `forgum doctor` |
| `forgum init` (user re-runs) | Same TUI, but "Welcome" is skipped; current config is the draft; used to reconfigure / change settings |
| `forgum` (no args) | Launches the TUI at the last-visited section (or Appearance on first open). This is the everyday "I want to change a thing" entry. |
| `forgum tui` | Explicit alias for `forgum` (no args). |
| `forgum-init` | Redirects to `forgum init` (shim binary + shell function, `16-…` §1.1). |

All four are the same TUI binary path; they differ only in the initial screen and whether the shell-hook injection runs.

---

## 10. Implementation notes

- **Renderer**: the TUI uses `ratatui` for layout/widgets, but the **hero cow and previews** are rendered by the v2 engine (the `Renderer` trait, `11-…`) into a `ratatui` buffer region. This is the same engine as the overlay — no second renderer to maintain.
- **Threading**: the TUI runs the render loop on the main thread (60 fps target, throttled to 30 for the preview to save battery). The eye-tracking computation is <0.1 ms per frame. The hero cow's idle animation uses the engine's sim thread.
- **DCL singleton**: the TUI reads config via the `config()` singleton (`OnceLock`, v2 principle #5) and writes via `config::set()` which updates the singleton + the file atomically.
- **Test harness**: `ratatui`'s `TestBackend` drives a headless test that walks every screen, asserts every config key has a widget (the "no orphan config" test), and snapshots golden frames (blake3-hashed, the v2 visual-regression tier).
- **Size budget**: the TUI binary adds ~400 KB to the engine (ratatui + unicode-width). Acceptable.

---

## 11. The one-paragraph summary

Running `forgum` (no args) launches a full-screen `ratatui` TUI — the single sanctioned way to configure Forgum. It is a 3-zone layout (lolcat-rainbow header, section sidebar, controls + live-preview main panel) styled to feel 3D via drop shadows, beveled borders, parallax, and glow, with a persistent **hero cow whose eyes track the focused widget and the terminal cursor**, blink on selection, and morph to a section-appropriate archetype (dragon for Appearance, owl for AI, turtle for Privacy). Every setting is a **toggle, dropdown, slider, or multi-select** — no free-text unless validated — and **every config key has a widget** (CI-enforced "no orphan config" invariant). Lolcat rainbow gradients saturate the title, focused accents, and the cow. Changes edit an in-memory draft; Apply shows a diff, then writes atomically with a `.bak` backup; undo/redo within the session. It's keyboard-first (mouse optional), screen-reader-friendly, with high-contrast and reduced-motion accessibility modes. An expert escape hatch (`e` → `$EDITOR` on the raw TOML, or `forgum config edit`) serves power users without compromising the toggle-first experience for everyone else. `forgum init`, `forgum-init`, and `forgum tui` all reach this same surface. The cow is the host, the feedback channel, and the soul of the menu — when you change a setting, it blinks; when you save, it smiles; when something's wrong, it looks confused. The menu isn't a settings dialog; it's a conversation with a small, attentive animal who happens to live in your terminal.
