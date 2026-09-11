# ╔══════════════════════════════════════════════════════════════════════════╗
# ║               🎨  F O R G U M   S A M P L E   C O N F I G S  🎨        ║
# ║                                                                        ║
# ║   Ready-made scenes to get you started — copy, paste, and enjoy.       ║
# ╚══════════════════════════════════════════════════════════════════════════╝

These sample `SceneConfig` files validate against the **v0.4.0** schema.
The schema uses `#[serde(deny_unknown_fields)]`, so every key must be exactly
one of the 12 known fields:

`cow`, `text`, `effect`, `background`, `duration`, `fps`, `eyes`, `tongue`,
`default_shell`, `auto_render_on_prompt`, `color_mode`, `shell_attach_mode`
(valid `color_mode`: `rainbow` | `solid` | `none`; valid `shell_attach_mode`: `banner` | `split` | `reactive` | `manual`).

---

## 🌈 Rainbow (`config.rainbow.json`)

```json
{
  "cow": "default",
  "text": "Moo from the rainbow farm!",
  "effect": "rainbow",
  "background": false,
  "duration": 0,
  "fps": 30,
  "eyes": "oo",
  "tongue": " ",
  "default_shell": "",
  "auto_render_on_prompt": true,
  "color_mode": "rainbow"
}
```

The full-color, effect-heavy joyride. This is what happens when the meadow
throws a festival and every animal wears its brightest coat.

---

## 🤏 Minimal (`config.minimal.json`)

```json
{
  "cow": "default",
  "effect": "static",
  "color_mode": "none"
}
```

Just the cow, nothing else. The purist's choice — a single static frame,
no animation, no color. Sometimes less is more.

---

## 🎨 Solid (`config.solid.json`)

```json
{
  "cow": "tux",
  "text": "Penguin says hi.",
  "effect": "static",
  "background": true,
  "duration": 0,
  "fps": 12,
  "eyes": "$$",
  "tongue": " ",
  "default_shell": "bash",
  "auto_render_on_prompt": false,
  "color_mode": "solid"
}
```

Solid background, calm and clean. Tux the penguin with dollar-sign eyes
— a creature of commerce and cheer.

---

## 🚀 Usage

Pass a sample via the global `--config` flag, then a render command:

```bash
forgum --config docs/samples/config.rainbow.json say "hello"
```

Or just copy a sample to your config location:

- **Windows:** `~/.config/forgum/config.json` (i.e. `%USERPROFILE%\.config\forgum\config.json`)
- **macOS / Linux:** `~/.config/forgum/config.json`

---

<div align="center">

```
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||
```

*Find the config that speaks to you! 🐮*

</div>
