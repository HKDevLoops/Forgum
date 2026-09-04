# ╔══════════════════════════════════════════════════════════════════════════╗
# ║                   🔧  A D V A N C E D   G U I D E  🔧                  ║
# ║                                                                        ║
# ║   For the curious, the brave, and the terminally ambitious.             ║
# ╚══════════════════════════════════════════════════════════════════════════╝

This guide is for advanced users who want to extend Forgum: add cows, effects,
shell hooks, or config keys, or hack on the engine internals. Every `file:line`
reference below points at the **real** workspace code. Nothing here describes the fictional `brain/19` flow.

- Binary name: `forgum-engine`
- Real crates: `engine`, `platform`, `tui`
- Config schema is **strict** (`#[serde(deny_unknown_fields)]`) — unknown
  fields are rejected at parse time, so a new option must be added end-to-end.

---

## 🐄 Adding a Cow

Cows are plain files in `data/Cows/`. There is **no `.cow` enumerator** in the
engine — a cow is referenced by its basename (free text), and the engine builds
the path on demand:

- `crates/engine/src/cow.rs:29` joins `data_dir.join("Cows").join("{name}.cow")`.

### Steps

1. Drop a new file into `data/Cows/`, e.g. `data/Cows/mycow.cow`. Use the
   classic cowsay `.cow` format (a `$the_cow =` block with `$eyes`, `$tongue`,
   `$thoughts` placeholders).
2. Reference it by name via the `cow` config key (no code change required):
   - `forgum-engine --config ... say "hi"` with `"cow": "mycow"` in the JSON, or
   - `cargo run -p forgum-engine -- say "hi"` (uses `cow: "default"` unless overridden).
3. If a `cow_dna.json` exists (a DNA/catalog file referenced by older tools),
   add an entry for `mycow`. **Verified: `data/cow_dna.json` does NOT currently
   exist in this workspace**, so there is nothing to update — skip this step.

There are 100+ cows already (e.g. `default.cow`, `tux.cow`, `dragon.cow`,
`nyan.cow`). The cow name is free-text; just make sure the `.cow` file exists.

---

## ✨ Adding an Effect

Effects live in `crates/engine/src/effects.rs`:

- The `Effect` trait is defined there. Implement it for your new effect type.
- `create_effect` (~line 699, test mod) is the factory: add your effect to its
  match arm so it can be selected by the `effect` config string.

### Steps

1. Implement the `Effect` trait in `crates/engine/src/effects.rs`.
2. Register it in `create_effect` (add a match arm mapping `"youreffect"` →
   `Box::new(YourEffect)`).
3. Add a golden frame test: render one frame and `assert!` the output bytes
   match an expected snapshot. Put it in the effects test module (~line 699).

Because `deny_unknown_fields` is on `SceneConfig`, the `effect` field already
accepts any string; you only need the factory registration to make a new value
actually do something.

---

## 🐚 Adding a Shell Hook

Shell hooks are generated in `crates/engine/src/init.rs`. Six shells are
supported (bash, zsh, fish, pwsh 7+, powershell.exe 5.1, cmd.exe):

- `Shell` enum — `crates/engine/src/init.rs:13`
- `Shell::parse` — `crates/engine/src/init.rs:25`
- `generate_hook` match — `crates/engine/src/init.rs:62`
- `default_config_path` per shell — `crates/engine/src/init.rs:38`

### Steps

1. Add a variant to the `Shell` enum in `crates/engine/src/init.rs:13`.
2. Add an arm to `Shell::parse` (`init.rs:25`) so the name resolves.
3. Add an arm to `generate_hook` (`init.rs:62`) that emits the hook snippet for
   the new shell.
4. Add a per-shell `default_config_path` entry if the shell needs its own config
   location (`init.rs:38`).
5. Add a test asserting `generate_hook` emits the expected snippet for the new
   variant, and that `Shell::parse` round-trips.

Do **not** confuse engine `Shell` (`init.rs`) with platform `ShellKind`
(`crates/platform/src/paths.rs:28`) — they are different types.

---

## ⚙️ Adding a Config Key (The No-Orphan-Config Dance)

Because `SceneConfig` uses `#[serde(deny_unknown_fields)]`, a new field must be
wired through **every** layer or it is silently dropped/orphaned. The real flow
(not the fictional `brain/19` flow) is:

### Step-by-step

1. **Schema** — `crates/engine/src/protocol.rs`:
   - Add the field to `SceneConfig` (struct ~line 15, after `color_mode`).
   - Add a `default_<name>()` helper (see `default_color_mode` at line 91).
   - Add the field to the `Default` impl (`protocol.rs:95`, the struct literal
     at line 97).
   - Keep `#[serde(deny_unknown_fields)]` at `protocol.rs:14`.

2. **Merge sentinel** — `crates/engine/src/config.rs`:
   - Add a sentinel in `merge()` (`config.rs:42` / function at line 45) so the
     overlay can be "unset" (e.g. `== ""` or `== 0` keeps the base). Mirror the
     existing field sentinels (lines 26–40).

3. **CLI mirror** — `crates/engine/src/cli.rs`:
   - Add a key to the `config set` subcommand so users can set it from the CLI.
   - `Commands` enum ~line 85; `Config` variant ~line 108; `ShellArg` ValueEnum
     ~line 275. `main.rs` dispatch ~line 33.

4. **TUI widget** — `crates/tui/src/app.rs`:
   - Add a widget in the config TUI so the field is editable in
     `run_config_tui` (`crates/tui/src/lib.rs`). The engine call site is
     `crates/engine/src/config_tui.rs`, gated by `cfg!(feature = "tui")`.

5. **Test** — add a unit test covering: default value, merge sentinel behavior,
   and (optionally) a TUI round-trip.

Skip any layer and the value becomes an orphan (ignored or rejected).

---

## 🗺️ API / Hacking Map

Subsystem → file:line (all verified against this workspace):

| Subsystem | Location |
|-----------|----------|
| **SceneConfig schema** | `crates/engine/src/protocol.rs` (struct ~line 15, `Default` ~line 97) |
| **Config load / merge** | `crates/engine/src/config.rs` (`read_config_file` ~line 16, `merge` ~line 42) |
| **FrameBuffer dirty-tracking** | `crates/engine/src/framebuffer.rs` (dirty-tracking logic; test mod ~line 207) |
| **Renderer trait** | `crates/engine/src/renderer.rs` (test mod ~line 231; `cfg!` macro ~line 206 and ~line 366) |
| **Effects** | `crates/engine/src/effects.rs` (`Effect` trait + `create_effect` factory; test mod ~line 699) |
| **Shell hooks** | `crates/engine/src/init.rs` (`Shell` enum ~line 13, `Shell::parse` ~line 25, `generate_hook` ~line 62) |
| **Capability probe** | `crates/platform/src/terminal.rs` (terminal capability detection) |
| **Cows** | `data/Cows/*.cow` (100+ files); path built at `crates/engine/src/cow.rs:29` |
| **Fortunes** | `data/Fortunes/fortunes.txt`, reader `crates/engine/src/fortune.rs` |
| **CLI** | `crates/engine/src/cli.rs` (`Commands` enum ~line 85, `Config` variant ~line 108) |
| **TUI** | `crates/tui/src/lib.rs` (`run_config_tui`), `crates/tui/src/app.rs` |
| **Platform paths** | `crates/platform/src/paths.rs` (`ShellKind` ~line 28 — NOT the engine `Shell`) |

---

## 🚩 Feature Flags

| Flag | Crate | Description |
|------|-------|-------------|
| `synchronized-update` | engine | Checked at runtime via `cfg!(feature = "synchronized-update")` |
| `sixel` | platform | Platform crate feature for sixel-capable terminals |
| `tui` | engine | Optional `forgum-tui` dependency; gated by `cfg!(feature = "tui")` |

### Running feature-gated tests

```sh
cargo test --workspace --features forgum-engine/synchronized-update
cargo test -p forgum-platform --features forgum-platform/sixel
```

---

## ⚠️ Important

**The crate names `config`, `tui`, `cli`, `cowsay`, `ai`, and `shell` referenced
in `brain/19-CONTRIBUTING-GUIDE.md` DO NOT EXIST.** The real crates in this
workspace are:

- `engine` (`crates/engine`)
- `platform` (`crates/platform`)
- `tui` (`crates/tui`)

If you follow `brain/19`, you will edit files and modules that are not present.
Use the file:line map in this document instead.

---

<div align="center">

```
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||
```

*Now go build something amazing! 🔧*

</div>
