# Forgum — Per-Animal Animation Design (the 7-Axis DNA)

> **The "every cow is unique and looks amazing" document.** This specifies the animation system that makes all 109 cows animate distinctly — each with its own speed, movement type, easing, color palette, particle behavior, and phase randomization. It realizes `docs/Cow_Animation_Manifesto.md` and fixes BUG-E3 (cow-specific styling ignored) structurally.
>
> **Audience:** the Rust engineer implementing `crates/engine/src/animation/` and curating `data/animations.json`.

---

## 1. The problem (why the current engine feels cheap)

Today, `style_matcher::get_cow_style` returns `{ base, particles, speed }` for each cow — but `create_effect` in `main.rs` **discards `speed` and `particles`**, instantiating every effect with identical parameters. The result: a dragon, a dolphin, a nyan cat, and a ghost all animate the same way. The manifesto (`docs/Cow_Animation_Manifesto.md`) defines 10 base animation types and 6 particle types with rich per-cow assignments, but the engine ignores them.

This document defines the **7-axis DNA profile** that makes each cow unique, the **easing functions** that make motion feel alive, the **OKLCH color system** that makes palettes look professional, the **Verlet physics** for secondary motion (tails, capes), and the **phase randomization** that prevents herds from syncing.

---

## 2. The 7-axis DNA profile

Every cow gets a profile in `data/animations.json`:

```json
{
  "dragon.cow": {
    "base": "Fire",
    "particles": {
      "type": "Fire",
      "rate": 18,
      "life": [0.6, 1.4],
      "speed": [0.3, 0.8],
      "palette": ["#ff8800", "#ff2200", "#440000"]
    },
    "speed": 1.0,
    "amplitude": {
      "breath": 0.6,
      "sway": 0.2,
      "float": 0.1
    },
    "palette": ["#ff8800", "#ff2200", "#440000"],
    "easing": {
      "base": "sine_inout",
      "particle_alpha": "expo_out",
      "particle_velocity": "cubic_out"
    },
    "phase_seed": 4242,
    "glow": {
      "color": "#ff6600",
      "radius": 6.0,
      "falloff": "gaussian"
    },
    "region_overrides": {
      "eyes": { "blink_rate": 0.2, "blink_char": "--" },
      "mouth": { "chew": true, "chew_chars": ["W", "w", "W", " "] }
    }
  }
}
```

### 2.1 The 7 axes

| Axis | Type | What it controls | Example |
|------|------|------------------|---------|
| **1. base** | enum (10 types) | The primary motion algorithm | `"Fire"` — fire particles + breathe + hover |
| **2. particles** | struct | Particle emitter config | `{type: "Fire", rate: 18, life: [0.6,1.4], ...}` |
| **3. speed** | f32 | Global time multiplier | `1.0` = normal, `0.3` = tortoise-slow, `2.0` = nyan-fast |
| **4. amplitude** | struct | Motion depth per channel | `{breath: 0.6, sway: 0.2, float: 0.1}` |
| **5. palette** | `Vec<HexColor>` | Color gradient stops | `["#ff8800", "#ff2200", "#440000"]` |
| **6. easing** | struct | Motion curve per channel | `{base: "sine_inout", particle_alpha: "expo_out"}` |
| **7. phase_seed** | u32 | Per-instance randomization seed | `4242` — hashed with instance_id to desync herds |

Plus optional **`glow`** (radial light source) and **`region_overrides`** (per-region behavior for eyes/mouth/wings/tail).

### 2.2 The 10 base animation types

| Base | Motion | Default easing | Use for |
|------|--------|----------------|---------|
| **Breathe** | Expands/contracts whitespace gaps (chest/belly morph) | `sine_inout` | Cows, sheep, koalas — living creatures at rest |
| **Float / Bob** | Whole-art vertical/horizontal drift | `sine_inout` | Ghosts, jellyfish, aquatic floaters |
| **Walk / Trot** | Bottom-row leg char swap (`\|`→`/`→`\|`→`\`) | `cubic_inout` | Bovines, moose, cats walking |
| **Particles** | Contextual emitter (fire/bubbles/stars/zzz/pulse/glitch) | per-particle | Dragon fire, dolphin bubbles, nyan stars |
| **Pulse / Glow** | Color cycling (rainbow sweep or localized glow) | `expo_out` | Aliens, cthulhu, periodic-table |
| **Glitch** | Random char swap with binary/hex | `linear` + jitter | Skeletons, doge, GLaDOS |
| **Fly / Hover** | Fast erratic float + wing flap (`>`/`<`/`v` swap) | `sine_in`/`sine_out` alt | Birds, bats, nyan, bees |
| **Talk / Chew** | Mouth + eye region animation (synced to text) | `back_out` | Default cow, eyes cow |
| **Sway / Pendulum** | Top-half skew, bottom anchored | `sine_inout` | Trees, tux, hellokitty |
| **Dissolve** | Break art into falling chars, reassemble | `cubic_in` | Hedgehogs, ghosts (exit transition) |

### 2.3 The 6 particle types

| Particle | Glyphs | Velocity | Color palette | Lifetime | Use for |
|----------|--------|----------|---------------|----------|---------|
| **Fire** | `*`, `^`, `.`, `~` | Upward (-5 to -15 vy), slight horizontal drift | Orange→Red→Dark red | 0.6–1.4s | Dragons, daemons, flaming sheep |
| **Bubbles** | `o`, `O`, `°`, `.` | Upward (-1 to -3 vy), sine-wave horizontal | Cyan/light-blue, translucent | 1.5–3.0s | Dolphins, whales, seahorses |
| **Stars** | `*`, `+`, `✦`, `✧` | Trailed behind flyer | Rainbow (lolcat), bright | 0.4–0.8s | Nyan, wizard |
| **Zzz** | `Z`, `z` | Upward + sine drift | Soft purple/blue, fading | 2.0–4.0s | Sleeping koalas, snoopysleep |
| **Pulse** | (color-only, no glyph) | Static position, alpha pulse | Cow's palette, pulsing | 0.5–1.0s | Doge, GLaDOS, kosh |
| **Glitch** | `0`, `1`, `#`, `@`, random | Random teleport within bounds | Green/red glitch | 0.1–0.3s | Ghosts, skeletons, surgery |

---

## 3. Easing functions (the "alive" factor)

### 3.1 Why easing matters

Linear motion looks robotic. Real creatures accelerate and decelerate — a bunny's hop starts slow, speeds up, slows at the peak, accelerates down, and squashes on landing. Easing functions encode this.

### 3.2 The 8 functions Forgum uses

Vendored in `crates/engine/src/animation/easing.rs` (~60 lines, no deps), or via the `ezing` crate:

```rust
pub fn linear(t: f32) -> f32 { t }
pub fn sine_inout(t: f32) -> f32 { -(std::f32::consts::PI * (t - 0.5)).cos() * 0.5 + 0.5 }
pub fn cubic_inout(t: f32) -> f32 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}
pub fn cubic_out(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
pub fn cubic_in(t: f32) -> f32 { t * t * t }
pub fn back_out(t: f32) -> f32 {
    let c1 = 1.70158; let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}
pub fn expo_out(t: f32) -> f32 { if t == 1.0 { 1.0 } else { 1.0 - 2.0.powf(-10.0 * t) } }
pub fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625; let d1 = 2.75;
    if t < 1.0 / d1 { n1 * t * t }
    else if t < 2.0 / d1 { n1 * (t - 1.5 / d1) * (t - 1.5 / d1) + 0.75 }
    else if t < 2.5 / d1 { n1 * (t - 2.25 / d1) * (t - 2.25 / d1) + 0.9375 }
    else { n1 * (t - 2.625 / d1) * (t - 2.625 / d1) + 0.984375 }
}
```

### 3.3 Per-base easing defaults (overridable per cow)

| Base anim | Default easing | Why |
|-----------|----------------|-----|
| Breathe | `sine_inout` | Smooth periodic chest rise |
| Float | `sine_inout` | Ghostly bobbing |
| Walk | `cubic_inout` | Slow start/stop = weight |
| Fly | `sine_in` + `sine_out` alternating | Wing flap cadence |
| Talk | `back_out` (mouth open) | Slight overshoot = chew |
| Sway | `sine_inout` | Pendulum |
| Pulse | `expo_out` | Fast bright, slow fade |
| Glitch | `linear` + jitter | Robotic = intentional |
| Dissolve | `cubic_in` | Slow start, accelerating scatter |
| Particles (Fire) | `cubic_out` (velocity) + `expo_out` (alpha) | Fast burst, slow fade |
| Particles (Bubbles) | `sine_inout` (y-velocity) | Buoyant wobble |
| Particles (Zzz) | `cubic_out` (y) + `sine` (x drift) | Drift up, sway sideways |
| Particles (Stars) | `linear` (trailing) | Constant velocity |
| Particles (Glitch) | `linear` + random | Digital jitter |

### 3.4 Disney's 12 principles applied to ASCII

| Principle | How Forgum applies it | Example |
|-----------|----------------------|---------|
| **Squash & Stretch** | Scale Y of cow body during hop/land | Bunny: ×0.9 dip → ×1.15 jump → ×0.85 land |
| **Anticipation** | 80ms ease-in dip before a jump | Bunny hop, dragon firebreath glow ramp |
| **Staging** | One focal point per frame (the cow, not the particles) | Particles are secondary, lower alpha |
| **Follow-Through** | Verlet tail continues moving after body stops | Cat tail flick, dragon tail whip |
| **Slow In / Slow Out** | Easing functions (never linear except Glitch) | All base anims use easing |
| **Arcs** | Particles follow parabolic arcs (gravity) | Fire particles arc, bubbles float straight |
| **Secondary Action** | Particles, glow, tail flick support the primary | Dragon: breathe (primary) + fire particles (secondary) |
| **Timing** | 80ms blink, 200ms hop, 400ms tail flick, 4s breath cycle | Per-cow `blink_rate` |
| **Exaggeration** | Dragon fire is bigger than realistic | `amplitude.breath: 0.6` not `0.1` |
| **Solid Drawing** | Consistent light source (glow position) | `glow.color` + `glow.radius` per cow |
| **Appeal** | OKLCH palettes, harmonious colors | Dragon: orange→red→black, not random |

---

## 4. Color: OKLCH (the professional palette system)

### 4.1 Why OKLCH, not HSV

HSV is non-uniform: equal hue steps produce unequal perceived steps (yellow looks lighter than blue at same V). OKLCH (polar form of Oklab) is perceptually uniform — equal numerical hue steps produce equal perceived hue steps. It's the 2025 industry standard (Photoshop, CSS Color Level 4/5, Unity, Godot).

### 4.2 The `palette` crate

```rust
use palette::{Lch, Srgb, IntoColor, ShiftHue};

// Gradient interpolation in OKLCH space
fn lerp_palette(palette: &[Srgb<u8>], t: f32) -> Srgb<u8> {
    let lch: Vec<Lch> = palette.iter()
        .map(|c| Srgb::from(*c).into_color())
        .collect();
    let segs = lch.len() - 1;
    let seg = (t * segs as f32).floor() as usize;
    let local_t = t * segs as f32 - seg as f32;
    let a = lch[seg.min(segs)];
    let b = lch[(seg + 1).min(segs)];
    // Interpolate L, C, h independently (h via shortest arc)
    lch_lerp(a, b, local_t).into_color()
}
```

### 4.3 The lolcat rainbow (ported to engine)

The canonical lolcat formula (from `equa.space/sh/lolcat`):

```rust
pub fn lolcat_hsv(x: f32, y: f32, t: f32, offset: f32) -> Rgb {
    let angle = 45f32.to_radians();
    let hue = ((x * angle.cos() + y * angle.sin()) / 100.0 + offset / 360.0 + t) % 1.0;
    hsv_to_rgb(hue * 360.0, 0.8, 0.9)
}
```

This eliminates the PowerShell-only `FormatLolcat.ps1` and lets `--lolcat` work in background daemon mode.

### 4.4 Gaussian radial glow (replacing linear falloff)

The current `apply_radial_glow` uses linear `1.0 - distance/radius` which produces a hard edge. Replace with Gaussian:

```rust
pub fn gaussian_glow(dx: f32, dy: f32, radius: f32) -> f32 {
    let sigma = radius / 2.5;  // intensity ≈ 0.135 at the radius boundary
    let d2 = dx * dx + dy * dy;
    (-d2 / (2.0 * sigma * sigma)).exp()
}
```

For Pulse cores (bright center), use inverse-square:

```rust
pub fn inverse_square_glow(dx: f32, dy: f32, k: f32) -> f32 {
    1.0 / (1.0 + (dx * dx + dy * dy) * k)
}
```

### 4.5 256-color dithering fallback

When `COLORTERM != truecolor`, quantize to xterm-256:

```rust
pub fn rgb_to_xterm256(r: u8, g: u8, b: u8) -> u8 {
    // xterm-256 cube: 16 + 36*(r/51) + 6*(g/51) + b/51
    16 + 36 * (r / 51) + 6 * (g / 51) + (b / 51)
}
```

Apply 4×4 Bayer dithering on the per-cell alpha to mask banding:

```rust
const BAYER_4X4: [[f32; 4]; 4] = [
    [-0.5,  0.0, -0.375, 0.125],
    [ 0.25,-0.25, 0.375,-0.125],
    [-0.375,0.125,-0.5,  0.0],
    [ 0.375,-0.125,0.25,-0.25],
];

pub fn dithered_quantize(r: u8, g: u8, b: u8, x: usize, y: usize, alpha: f32) -> u8 {
    let bias = BAYER_4X4[y % 4][x % 4] * 16.0;  // ±8 levels
    let biased_r = (r as f32 + bias).clamp(0.0, 255.0) as u8;
    // ... apply to g, b
    rgb_to_xterm256(biased_r, g, b)
}
```

---

## 5. Verlet physics for secondary motion (tails, capes)

### 5.1 Why Verlet (not Euler)

Euler integration (`pos += vel * dt`) is unstable for spring/constraint systems — it explodes with large `dt`. Verlet integration stores `position` and `prev_position` (not velocity); the implicit velocity is `pos - prev_pos`. It's unconditionally stable, energy-conserving, and trivially constraint-solvable.

### 5.2 The `VerletChain<N>` generic

```rust
pub struct VerletChain<const N: usize> {
    pos: [[f32; 2]; N],       // current positions
    prev: [[f32; 2]; N],      // previous positions (implicit velocity)
    rest_len: [f32; N - 1],   // rest distances between adjacent links
}

impl<const N: usize> VerletChain<N> {
    pub fn new(anchor: [f32; 2], link_len: f32) -> Self {
        let mut pos = [[0.0; 2]; N];
        let mut prev = [[0.0; 2]; N];
        for i in 0..N {
            pos[i] = [anchor[0], anchor[1] + i as f32 * link_len];
            prev[i] = pos[i];
        }
        let mut rest_len = [0.0; N - 1];
        for r in &mut rest_len { *r = link_len; }
        Self { pos, prev, rest_len }
    }

    pub fn step(&mut self, dt: f32, gravity: f32, anchor: [f32; 2], wind: [f32; 2]) {
        // 1. Verlet integrate (anchor link 0 is pinned)
        for i in 1..N {
            let vel = [self.pos[i][0] - self.prev[i][0],
                       self.pos[i][1] - self.prev[i][1]];
            self.prev[i] = self.pos[i];
            self.pos[i][0] += vel[0] * 0.98 + wind[0] * dt * dt;  // 0.98 = damping
            self.pos[i][1] += vel[1] * 0.98 + gravity * dt * dt;
        }
        // Pin link 0 to anchor
        self.pos[0] = anchor;
        self.prev[0] = anchor;

        // 2. Distance-constraint relaxation (4 iterations)
        for _ in 0..4 {
            for i in 0..(N - 1) {
                let dx = self.pos[i + 1][0] - self.pos[i][0];
                let dy = self.pos[i + 1][1] - self.pos[i][1];
                let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                let diff = (dist - self.rest_len[i]) / dist * 0.5;
                if i != 0 {  // don't move the pinned anchor
                    self.pos[i][0] += dx * diff;
                    self.pos[i][1] += dy * diff;
                }
                self.pos[i + 1][0] -= dx * diff;
                self.pos[i + 1][1] -= dy * diff;
            }
        }
    }
}
```

### 5.3 Tail flick (impulse application)

To trigger a tail flick (e.g. cat swats a fly):

```rust
pub fn impulse(&mut self, link_idx: usize, force: [f32; 2]) {
    if link_idx < N {
        self.pos[link_idx][0] += force[0];
        self.pos[link_idx][1] += force[1];
    }
}
// Triggered randomly every 5-15s (Poisson), force scaled by amplitude.sway
```

### 5.4 Per-cow Verlet config

| Cow | Chain length | Link len | Gravity | Damping | Wind | Flick rate |
|-----|--------------|----------|---------|---------|------|------------|
| cat/kitty/kitten | 4 | 1.0 | 9.8 | 0.98 | 0.2 | 5-15s |
| dragon | 6 | 1.5 | 9.8 | 0.95 | 0.5 | 3-8s (agitated) |
| nyan | 0 (rainbow trail instead) | — | — | — | — | — |
| batman | 3 (cape) | 2.0 | 12.0 | 0.92 | 1.0 | on movement |
| bunny | 0 (ears instead — 2-link vertical) | 1.0 | 9.8 | 0.99 | 0.1 | on hop |

---

## 6. Per-instance phase randomization (herds don't sync)

### 6.1 The problem

If 5 `default.cow` instances run in 5 tmux panes with the same config, they breathe in lockstep — looks mechanical. Real herds breathe at slightly different rates.

### 6.2 The golden-ratio offset

```rust
pub fn instance_phase(phase_seed: u32, instance_id: u32) -> f32 {
    // Hash seed + instance_id, then multiply by golden ratio fractional part
    // for maximal low-discrepancy sequence (no clustering).
    let hash = phase_seed.wrapping_mul(2654435761).wrapping_add(instance_id);
    let normalized = (hash as f32 / u32::MAX as f32).fract();
    normalized * 0.61803398875  // golden ratio fractional part
}

// In AnimState:
pub struct AnimState {
    pub breath_phase: f32,       // + instance_phase offset
    pub blink_t: f32,            // + offset
    pub tail_flick_next: f32,    // + offset
    // ...
}

impl AnimState {
    pub fn new(dna: &CowDna, instance_id: u32) -> Self {
        let offset = instance_phase(dna.phase_seed, instance_id);
        Self {
            breath_phase: offset * std::f32::consts::TAU,
            blink_t: offset * 4.0,  // staggered blink timing
            tail_flick_next: 5.0 + offset * 10.0,  // 5-15s, staggered
            // ...
        }
    }
}
```

The golden ratio fractional part (0.618...) produces a low-discrepancy sequence — N instances are maximally spread across the phase space, no clustering.

### 6.3 The `instance_id`

Derived from the daemon's session context:
- In tmux: `$TMUX_PANE` (e.g. `%3`)
- In herbstluftwm: the frame index
- Standalone: the daemon PID

This guarantees two `default.cow` daemons in two panes get different `instance_id`s and desync.

---

## 7. The layered animation system

Each cow composes N independent phase tracks, each driving a different aspect:

```rust
pub struct AnimState {
    // Track A: breath (always on)
    pub breath_phase: f32,       // radians, advances at speed * 0.8 Hz
    pub breath_amplitude: f32,   // from DNA

    // Track B: blink (Poisson-triggered)
    pub blink_t: f32,            // 0..1 countdown
    pub blink_rate: f32,         // per-cow (owls 0.2/s, cats 0.5/s)
    pub blink_active: bool,

    // Track C: tail flick (Verlet + impulse)
    pub tail: Option<VerletChain<4>>,
    pub tail_flick_next: f32,    // seconds until next flick

    // Track D: color cycle
    pub color_hue: f32,          // degrees, advances at speed * 30°/s

    // Track E: base motion (walk/fly/sway/etc.)
    pub base_phase: f32,

    // Track F: particle emission
    pub particle_accum: f32,     // accumulates until next spawn

    // Instance identity
    pub instance_seed: u64,
}
```

Each track has its own phase accumulator; the per-instance seed offsets them so identical cows don't sync. The renderer composes all tracks additively into the framebuffer.

---

## 8. The full 109-cow DNA table

This is the curated assignments. Cows are grouped by the manifesto's 5-class taxonomy. Within each class, cows sharing a base get distinct `phase_seed`s and slight amplitude/palette tweaks.

> **Note:** `speed` is the global time multiplier (0.3 = slow, 2.0 = fast). `amplitude` is per-channel depth (0.0–1.0). `palette` is OKLCH gradient stops. `easing` names the function per channel. `particles` omitted = none.

### 8.1 Bovines & Farm Animals (Breathe + Walk + Talk)

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| default.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ffffff] | sine_inout | — | 1001 |
| cower.cow | Breathe | 0.8 | {0.4, 0.0, 0.0} | [#dddddd] | sine_inout | — | 1002 |
| fat-cow.cow | Breathe | 0.7 | {0.7, 0.0, 0.0} | [#ffffff] | sine_inout | — | 1003 |
| supermilker.cow | Breathe | 2.0 | {0.6, 0.0, 0.0} | [#ffffff, #ffaaaa] | sine_inout | Pulse(udder) | 1004 |
| moose.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#8B4513] | cubic_inout | — | 1005 |
| mule.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#A0522D] | cubic_inout | — | 1006 |
| sheep.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#f5f5f5] | sine_inout | — | 1007 |
| flaming-sheep.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ffffff] | sine_inout | Fire(rate=8) | 1008 |
| lamb.cow | Walk | 1.5 | {0.4, 0.2, 0.0} | [#fafafa] | cubic_inout | — | 1009 |
| lamb2.cow | Walk | 1.5 | {0.4, 0.2, 0.0} | [#f0f0f0] | cubic_inout | — | 1010 |
| goat.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#d4d4d4] | sine_inout | — | 1011 |
| goat2.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#c8c8c8] | sine_inout | — | 1012 |
| bud-frogs.cow | Breathe | 1.2 | {0.6, 0.0, 0.0} | [#4caf50] | sine_inout | — | 1013 |
| moofasa.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ffffff] | sine_inout | — | 1014 |
| mooghidjirah.cow | Breathe | 0.9 | {0.5, 0.0, 0.0} | [#f5f5dc] | sine_inout | — | 1015 |
| moojira.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#4287f5, #ffffff] | sine_inout | Pulse(rate=2) | 1016 |

### 8.2 Aquatic Creatures (Float + Bubbles)

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| dolphin.cow | Float | 0.5 | {0.0, 0.0, 0.6} | [#80d8ff] | sine_inout | Bubbles(rate=6) | 2001 |
| whale.cow | Float | 0.4 | {0.0, 0.0, 0.5} | [#4a90a4] | sine_inout | Bubbles(rate=4) | 2002 |
| happy-whale.cow | Float | 0.5 | {0.0, 0.0, 0.6} | [#5fb3d4] | sine_inout | Bubbles(rate=6) | 2003 |
| docker-whale.cow | Float | 0.5 | {0.0, 0.0, 0.5} | [#2496ed, #ffffff] | sine_inout | Bubbles(rate=5) | 2004 |
| octopus.cow | Sway | 1.0 | {0.0, 0.3, 0.3} | [#ff6b6b] | sine_inout | — | 2005 |
| smiling-octopus.cow | Sway | 1.0 | {0.0, 0.3, 0.3} | [#ff8e8e] | sine_inout | — | 2006 |
| jellyfish.cow | Float | 1.0 | {0.0, 0.0, 0.4} | [#00ffff, #ff00ff] | sine_inout | Pulse(rate=3) | 2007 |
| seahorse.cow | Float | 1.5 | {0.0, 0.0, 0.5} | [#ff9800] | sine_inout | Bubbles(rate=8) | 2008 |
| seahorse-big.cow | Float | 1.5 | {0.0, 0.0, 0.5} | [#ffb74d] | sine_inout | Bubbles(rate=8) | 2009 |
| lobster.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#d32f2f] | cubic_inout | — | 2010 |
| ebi_furai.cow | Float | 1.0 | {0.0, 0.0, 0.4} | [#ffa726] | sine_inout | Bubbles(rate=4) | 2011 |

### 8.3 Birds & Avians (Fly + Flap)

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| turkey.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#8d6e63] | sine_inout | — | 3001 |
| owl.cow | Talk | 1.0 | {0.3, 0.0, 0.0} | [#9e9e9e] | back_out | — (eyes Glitch) | 3002 |
| golden-eagle.cow | Fly | 0.8 | {0.0, 0.0, 0.5} | [#bcaaa4] | sine_in/sine_out | — | 3003 |
| tweety-bird.cow | Fly | 2.0 | {0.0, 0.0, 0.7} | [#ffeb3b] | sine_in/sine_out | — | 3004 |
| pterodactyl.cow | Fly | 1.5 | {0.0, 0.0, 0.6} | [#795548] | sine_in/sine_out | — | 3005 |
| batman.cow | Fly | 1.5 | {0.0, 0.0, 0.5} | [#212121, #f44336] | sine_in/sine_out | — (cape Verlet) | 3006 |

### 8.4 Felines, Canines & Forest Critters (Walk + Breathe)

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| cat.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#ffa726] | cubic_inout | — (tail Verlet N=4) | 4001 |
| cat2.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#ffcc80] | cubic_inout | — (tail Verlet) | 4002 |
| kitty.cow | Walk | 1.1 | {0.3, 0.2, 0.0} | [#ffb74d] | cubic_inout | — (tail Verlet) | 4003 |
| kitten.cow | Walk | 1.2 | {0.3, 0.2, 0.0} | [#ffcc80] | cubic_inout | — (tail Verlet) | 4004 |
| meow.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#ffa726] | cubic_inout | — (tail Verlet) | 4005 |
| catfence.cow | Sway | 1.0 | {0.0, 0.3, 0.0} | [#ffa726] | sine_inout | — | 4006 |
| doge.cow | Glitch | 1.0 | {0.3, 0.0, 0.0} | [#fff176, #ff8a65] | linear + jitter | Pulse(rate=4) | 4007 |
| fox.cow | Walk | 0.5 | {0.3, 0.2, 0.0} | [#ff7043] | cubic_inout | — | 4008 |
| bunny.cow | Float | 2.0 | {0.5, 0.0, 0.8} | [#fafafa] | bounce_out | — (ears Verlet) | 4009 |
| squirrel.cow | Glitch | 1.5 | {0.4, 0.3, 0.0} | [#a1887f] | linear + jitter | — | 4010 |
| hedgehog.cow | Dissolve | 1.0 | {0.0, 0.0, 0.0} | [#8d6e63] | cubic_in | — | 4011 |
| koala.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#b0bec5] | sine_inout | Zzz(rate=1) | 4012 |
| luke-koala.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#90a4ae] | sine_inout | Zzz(rate=1) | 4013 |
| bearface.cow | Breathe | 1.0 | {0.6, 0.0, 0.0} | [#6d4c41] | sine_inout | — | 4014 |
| cowfee.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#8d6e63] | sine_inout | Fire(rate=3, steam) | 4015 |

### 8.5 Sci-Fi, Fantasy & Monsters (Glitch + Pulse + Fly)

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| dragon.cow | Fire | 1.0 | {0.6, 0.2, 0.1} | [#ff8800, #ff2200, #440000] | sine_inout + expo_out | Fire(rate=18) | 5001 |
| dragon-and-cow.cow | Fly | 1.0 | {0.0, 0.2, 0.4} | [#ff8800, #440000] | sine_in/sine_out | Fire(rate=12) | 5002 |
| charizardvice.cow | Fly | 1.0 | {0.0, 0.2, 0.4} | [#ff6600, #6644ff] | sine_in/sine_out | Fire(rate=14) | 5003 |
| ghost.cow | Dissolve | 1.0 | {0.0, 0.0, 0.5} | [#e0e0e0, #ffffff] | cubic_in | Glitch(rate=5) | 5004 |
| ghostbusters.cow | Float | 1.0 | {0.0, 0.0, 0.4} | [#e0e0e0] | sine_inout | Glitch(rate=3) | 5005 |
| weeping-angel.cow | (static) | 0.0 | {0.0, 0.0, 0.0} | [#cfd8dc] | — | — (moves on resize) | 5006 |
| cthulhu-mini.cow | Pulse | 1.0 | {0.0, 0.2, 0.2} | [#4a148c, #00e676] | expo_out | — (tentacle Sway) | 5007 |
| daemon.cow | Float | 1.0 | {0.0, 0.0, 0.3} | [#b71c1c] | sine_inout | Fire(rate=10) | 5008 |
| satanic.cow | Float | 1.0 | {0.0, 0.0, 0.3} | [#7f0000] | sine_inout | Fire(rate=12) | 5009 |
| minotaur.cow | Walk | 1.5 | {0.5, 0.3, 0.0} | [#5d4037] | cubic_inout | — | 5010 |
| glados.cow | Glitch | 1.0 | {0.3, 0.0, 0.0} | [#e0e0e0, #f44336] | linear + jitter | Pulse(rate=2, eye) | 5011 |
| personality-sphere.cow | Glitch | 1.0 | {0.3, 0.0, 0.0} | [#e0e0e0, #2196f3] | linear + jitter | Pulse(rate=2, eye) | 5012 |
| kosh.cow | Pulse | 0.5 | {0.0, 0.0, 0.3} | [#9c27b0] | expo_out | Pulse(rate=3) | 5013 |
| alien.cow | Pulse | 1.0 | {0.0, 0.0, 0.2} | [#76ff03] | expo_out | — | 5014 |
| spidercow.cow | Walk | 1.0 | {0.2, 0.2, 0.3} | [#424242] | cubic_inout | — (web drop) | 5015 |

### 8.6 Pop Culture & Characters

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| snoopy.cow | Breathe | 1.0 | {0.4, 0.0, 0.0} | [#ffffff] | sine_inout | — | 6001 |
| snoopyhouse.cow | Breathe | 1.0 | {0.4, 0.0, 0.0} | [#ffffff] | sine_inout | — | 6002 |
| snoopysleep.cow | Breathe | 1.0 | {0.4, 0.0, 0.0} | [#ffffff] | sine_inout | Zzz(rate=2) | 6003 |
| ren.cow | Glitch | 1.5 | {0.4, 0.3, 0.0} | [#8d6e63] | linear + jitter | — | 6004 |
| stimpy.cow | Glitch | 1.5 | {0.4, 0.3, 0.0} | [#ffcc80] | linear + jitter | — | 6005 |
| beavis.zen.cow | Glitch | 1.5 | {0.3, 0.2, 0.0} | [#ffeb3b] | linear + jitter | — | 6006 |
| nyan.cow | Fly | 2.0 | {0.0, 0.0, 0.5} | [#ff0000, #ff8800, #ffff00, #00ff00, #0088ff, #8800ff] | sine_in/sine_out | Stars(rate=20, rainbow trail) | 6007 |
| hellokitty.cow | Sway | 1.0 | {0.0, 0.3, 0.0} | [#ffffff, #ff69b4] | sine_inout | — | 6008 |
| bees.cow | Fly | 2.0 | {0.0, 0.0, 0.6} | [#ffeb3b, #212121] | sine_in/sine_out | — (swarm) | 6009 |
| bill-the-cat.cow | Glitch | 1.5 | {0.4, 0.3, 0.0} | [#a1887f] | linear + jitter | — | 6010 |
| charlie.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#a1887f] | cubic_inout | — | 6011 |
| shikato.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#a1887f] | cubic_inout | — | 6012 |
| shrug.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ffffff] | sine_inout | — | 6013 |
| kiss.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ff69b4] | sine_inout | — | 6014 |
| hippie.cow | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ff8800, #ffff00, #00ff00] | sine_inout | — (rainbow) | 6015 |
| hiya.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#ffffff] | sine_inout | — | 6016 |

### 8.7 Objects, Insects & Misc

| Cow | base | speed | amplitude{breath,sway,float} | palette | easing | particles | phase_seed |
|-----|------|-------|------------------------------|---------|--------|-----------|------------|
| stegosaurus.cow | Walk | 0.3 | {0.3, 0.2, 0.0} | [#558b2f] | cubic_inout | — | 7001 |
| tortoise.cow | Walk | 0.3 | {0.3, 0.2, 0.0} | [#6d4c41] | cubic_inout | — | 7002 |
| turtle.cow | Walk | 0.3 | {0.3, 0.2, 0.0} | [#4caf50] | cubic_inout | — | 7003 |
| mona-lisa.cow | Talk | 1.0 | {0.1, 0.0, 0.0} | [#d4af37] | back_out | — (eyes only) | 7004 |
| periodic-table.cow | Pulse | 1.0 | {0.0, 0.0, 0.0} | [rainbow per element] | expo_out | — | 7005 |
| world.cow | Pulse | 1.0 | {0.0, 0.0, 0.0} | [#2196f3, #4caf50] | expo_out | — | 7006 |
| king.cow | Float | 0.5 | {0.0, 0.0, 0.3} | [#ffd700] | sine_inout | — | 7007 |
| queen.cow | Float | 0.5 | {0.0, 0.0, 0.3} | [#ffd700] | sine_inout | — | 7008 |
| knight.cow | Float | 0.5 | {0.0, 0.0, 0.3} | [#c0c0c0] | sine_inout | — | 7009 |
| rook.cow | Float | 0.5 | {0.0, 0.0, 0.3} | [#c0c0c0] | sine_inout | — | 7010 |
| pawn.cow | Float | 0.5 | {0.0, 0.0, 0.3} | [#c0c0c0] | sine_inout | — | 7011 |
| surgery.cow | Glitch | 1.0 | {0.3, 0.2, 0.0} | [#f44336, #ffffff] | linear + jitter | Glitch(rate=8) | 7012 |
| mutilated.cow | Glitch | 1.0 | {0.3, 0.2, 0.0} | [#b71c1c] | linear + jitter | Glitch(rate=10) | 7013 |
| tux.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#212121, #ffffff] | sine_inout | — (waddle) | 7014 |
| tux-big.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#212121, #ffffff] | sine_inout | — | 7015 |
| armadillo.cow | Walk | 1.0 | {0.3, 0.2, 0.0} | [#8d6e63] | cubic_inout | — | 7016 |
| atat.cow | Walk | 0.5 | {0.3, 0.2, 0.0} | [#9e9e9e] | cubic_inout | — | 7017 |
| elephant.cow | Walk | 0.5 | {0.4, 0.2, 0.0} | [#9e9e9e] | cubic_inout | — (trunk Verlet) | 7018 |
| elephant2.cow | Walk | 0.5 | {0.4, 0.2, 0.0} | [#757575] | cubic_inout | — (trunk Verlet) | 7019 |
| elephant-in-snake.cow | Breathe | 0.5 | {0.6, 0.0, 0.0} | [#4caf50] | sine_inout | — | 7020 |
| eyes.cow | Talk | 1.0 | {0.3, 0.0, 0.0} | [#ffffff] | back_out | — (blink-heavy) | 7021 |
| fat-banana.cow | Sway | 1.0 | {0.0, 0.4, 0.0} | [#ffeb3b] | sine_inout | — | 7022 |
| fence.cow | Sway | 1.0 | {0.0, 0.3, 0.0} | [#8d6e63] | sine_inout | — | 7023 |
| hypno.cow | Pulse | 1.0 | {0.0, 0.0, 0.0} | [spiral rainbow] | expo_out | — (hypnotic spiral) | 7024 |
| lollerskates.cow | Float | 2.0 | {0.0, 0.0, 0.5} | [#00ffff] | sine_inout | — | 7025 |
| small.cow | Float | 1.0 | {0.0, 0.0, 0.3} | [#ffffff] | sine_inout | — | 7026 |
| skeleton.cow | Glitch | 1.0 | {0.2, 0.1, 0.0} | [#ffffff] | linear + jitter | — | 7027 |
| wizard.cow | Pulse | 1.0 | {0.0, 0.2, 0.2} | [#7b1fa2, #ffd700] | expo_out | Stars(rate=5) | 7028 |
| claw-arm.cow | Sway | 1.0 | {0.0, 0.3, 0.0} | [#9e9e9e] | sine_inout | — | 7029 |
| knight.cow | Float | 0.5 | {0.0, 0.0, 0.3} | [#c0c0c0] | sine_inout | — | (dup above) |
| abomination (animate.cow fallback) | Breathe | 1.0 | {0.5, 0.0, 0.0} | [#ffffff] | sine_inout | — | 9999 |

### 8.8 Default fallback (unknown cows)

```json
{
  "_default": {
    "base": "Breathe",
    "speed": 1.0,
    "amplitude": { "breath": 0.5, "sway": 0.0, "float": 0.0 },
    "palette": ["#ffffff"],
    "easing": { "base": "sine_inout" },
    "phase_seed": 0,
    "glow": null
  }
}
```

Unknown cows (user-supplied `.cow` files without a DNA entry) get the default Breathe + a `phase_seed` derived from `hash(cow_filename)` so even unknown cows desync.

---

## 9. The `animations.json` schema (formal)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "patternProperties": {
    "^[a-z0-9_-]+\\.cow$": {
      "type": "object",
      "required": ["base", "speed", "amplitude", "palette", "easing", "phase_seed"],
      "properties": {
        "base": { "enum": ["Breathe", "Float", "Walk", "Particles", "Pulse",
                            "Glitch", "Fly", "Talk", "Sway", "Dissolve"] },
        "particles": {
          "type": "object",
          "properties": {
            "type": { "enum": ["Fire", "Bubbles", "Stars", "Zzz", "Pulse", "Glitch"] },
            "rate": { "type": "number", "minimum": 0 },
            "life": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 },
            "speed": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 },
            "palette": { "type": "array", "items": { "type": "string", "pattern": "^#[0-9a-fA-F]{6}$" } }
          }
        },
        "speed": { "type": "number", "minimum": 0.1, "maximum": 4.0 },
        "amplitude": {
          "type": "object",
          "properties": {
            "breath": { "type": "number", "minimum": 0, "maximum": 1 },
            "sway": { "type": "number", "minimum": 0, "maximum": 1 },
            "float": { "type": "number", "minimum": 0, "maximum": 1 }
          }
        },
        "palette": { "type": "array", "items": { "type": "string", "pattern": "^#[0-9a-fA-F]{6}$" }, "minItems": 1 },
        "easing": {
          "type": "object",
          "properties": {
            "base": { "enum": ["linear", "sine_inout", "cubic_inout", "cubic_out", "cubic_in", "back_out", "expo_out", "bounce_out"] },
            "particle_alpha": { "enum": ["linear", "sine_inout", "cubic_inout", "cubic_out", "cubic_in", "back_out", "expo_out", "bounce_out"] },
            "particle_velocity": { "enum": ["linear", "sine_inout", "cubic_inout", "cubic_out", "cubic_in", "back_out", "expo_out", "bounce_out"] }
          }
        },
        "phase_seed": { "type": "integer", "minimum": 0, "maximum": 4294967295 },
        "glow": {
          "type": ["object", "null"],
          "properties": {
            "color": { "type": "string", "pattern": "^#[0-9a-fA-F]{6}$" },
            "radius": { "type": "number", "minimum": 1, "maximum": 20 },
            "falloff": { "enum": ["gaussian", "linear", "inverse_square"] }
          }
        },
        "region_overrides": {
          "type": "object",
          "properties": {
            "eyes": { "type": "object", "properties": { "blink_rate": {"type":"number"}, "blink_char": {"type":"string"} } },
            "mouth": { "type": "object", "properties": { "chew": {"type":"boolean"}, "chew_chars": {"type":"array","items":{"type":"string"}} } },
            "wings": { "type": "object", "properties": { "flap_rate": {"type":"number"}, "flap_chars": {"type":"array","items":{"type":"string"}} } },
            "tail": { "type": "object", "properties": { "verlet_links": {"type":"integer"}, "flick_rate": {"type":"number"} } }
          }
        }
      }
    }
  },
  "additionalProperties": false
}
```

CI validates `animations.json` against this schema on every PR.

---

## 10. The curation process (how to fine-tune a cow)

A senior artist/engineer fine-tunes a cow in 5 steps:

1. **Assign the base** from the taxonomy (§8).
2. **Pick the palette** in OKLCH space (use a tool like oklch.com to choose harmonious stops).
3. **Set speed + amplitude** by eye: render at 60fps, adjust until it "feels right" (dragon fire is fast and intense; tortoise walk is slow and deliberate).
4. **Choose easing** per the §3.3 defaults, override only if the motion feels wrong.
5. **Set phase_seed** to any unique u32 (the hash + golden-ratio offset handles desync; the exact value doesn't matter, only uniqueness).

The `forgum gallery` command (Phase 8) renders all 109 cows in a grid for side-by-side comparison and tuning.

---

## 11. Verification (how we prove every cow is unique)

### 11.1 Golden framebuffer hash (per-cow visual regression)

For each cow, render 60 frames at 60fps into a `Vec<Cell>` framebuffer, then `blake3::hash(framebuffer.as_bytes())`. Store in `Tests/golden/<cow>.blake3`. CI fails if the hash changes — catches "the dragon stopped breathing" or "lolcat rainbow collapsed."

```rust
#[test]
fn golden_framebuffer_per_cow() {
    for cow in all_cows() {
        let dna = load_dna(cow);
        let mut sim = SimState::new(dna, instance_id=0);
        let mut fb = FrameBuffer::new(80, 24);
        for _ in 0..60 { sim.tick(16_666_667); sim.render(&mut fb); }
        let hash = blake3::hash(fb.as_bytes());
        let golden = std::fs::read_to_string(format!("Tests/golden/{}.blake3", cow)).unwrap();
        assert_eq!(hash.to_hex().as_str(), golden.trim(),
            "{}: visual regression — golden hash mismatch", cow);
    }
}
```

Re-golden after intentional changes: `cargo test --features regolden`.

### 11.2 Perceptual hash (weekly, "does it look right")

Rasterize the framebuffer to PNG via `tiny-skia`, compute pHash (8×8 grayscale DCT, 64-bit hash), compare to reference with Hamming distance ≤ 5. Catches regressions golden-hash misses (1-pixel shifts) and avoids false positives from trivial refactors.

### 11.3 Herd desync test

Render 5 `default.cow` instances with `instance_id` 0–4 for 60 frames. Assert the 5 framebuffers are **not** byte-identical (the phase randomization worked):

```rust
#[test]
fn herd_desyncs() {
    let mut hashes = std::collections::HashSet::new();
    for id in 0..5 {
        let mut sim = SimState::new(load_dna("default.cow"), instance_id=id);
        // ... render 60 frames ...
        hashes.insert(blake3::hash(fb.as_bytes()).to_hex().to_string());
    }
    assert_eq!(hashes.len(), 5, "5 instances produced identical frames — desync failed");
}
```

### 11.4 Palette uniqueness test

Assert no two cows in the same taxonomy class share an identical palette (catches copy-paste laziness):

```rust
#[test]
fn palettes_unique_within_class() {
    for class in taxonomy_classes() {
        let palettes: Vec<_> = class.cows().map(|c| load_dna(c).palette).collect();
        let unique: HashSet<_> = palettes.iter().cloned().collect();
        assert_eq!(palettes.len(), unique.len(), "duplicate palettes in {}", class.name);
    }
}
```

---

## 12. The "amazing" factor — what makes it look professional

| Aspect | Amateur | Professional (Forgum v2) |
|--------|---------|--------------------------|
| Easing | Linear everywhere | Per-base easing; Disney 12 principles |
| Color | HSV (muddy gradients) | OKLCH (perceptually uniform) |
| Glow | Linear falloff (hard edge) | Gaussian (soft, natural) |
| Motion | Uniform phase (lockstep herds) | Golden-ratio phase randomization |
| Tails | Static or stiff | Verlet physics (whip, lag, overshoot) |
| Idle | Dead until triggered | Keep-alive: breath + blink + tail-flick always on |
| Particles | One glyph, one color | Bucketed by color, multi-glyph, life-based color shift |
| 256-color | Banding | 4×4 Bayer dithering |
| Per-cow | All animate identically | 7-axis DNA, 109 unique profiles |
| Transitions | Hard cuts | nms-style dissolve/reveal |

This is the difference between "a cowsay clone with colors" and "a terminal companion that feels alive."

---

**Companion documents:**
- `10-FINE-TUNED-MASTER-PLAN.md` — the phased plan (Phase 3 implements this).
- `11-ENGINE-INTERNALS-V2.md` — the engine that runs this (sim thread, `AnimState`, `VerletChain`).
- `13-TEST-COVERAGE-MATRIX.md` — the golden-hash + pHash tests that verify this.
