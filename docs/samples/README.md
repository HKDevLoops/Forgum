# Forgum Sample Configs

These sample `SceneConfig` files validate against the **v0.4.0** schema.
The schema uses `#[serde(deny_unknown_fields)]`, so every key must be exactly
one of the 11 known fields:

`cow`, `text`, `effect`, `background`, `duration`, `fps`, `eyes`, `tongue`,
`default_shell`, `auto_render_on_prompt`, `color_mode`
(valid `color_mode`: `rainbow` | `solid` | `none`).

The three files include the 3 newer fields: `default_shell`,
`auto_render_on_prompt`, and `color_mode`.

## Samples

- `config.rainbow.json` — full happy rainbow config.
- `config.minimal.json` — minimal/sane defaults; unset fields take defaults on merge.
- `config.solid.json` — solid-color, overlay-style config.

## Usage

Pass a sample via the global `--config` flag, then a render command:

```
forgum-engine --config docs/samples/config.rainbow.json say "hello"
```

Or just copy a sample to your config location:

- Windows: `%APPDATA%\Forgum\config.json`
- macOS / Linux: `~/.config/Forgum/config.json`
