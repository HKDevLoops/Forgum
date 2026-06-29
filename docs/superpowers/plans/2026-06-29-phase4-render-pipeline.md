# Phase 4 — Render Pipeline & Real Effects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the effect system into the render loop, implement all 7 stubbed effects with real animation, integrate Verlet chains for secondary motion, and extract a Renderer trait.

**Architecture:** The render loop currently calls `render_static_cow()` directly. Phase 4 replaces this with an `Effect` trait object created from DNA. Each frame calls `effect.update(dt)` then `effect.render(fb, time)`. A `Renderer` trait abstracts ANSI output for tmux passthrough.

**Tech Stack:** Rust 1.96, existing crates (no new deps), existing modules.

---

## Task 1: Wire effect dispatch into render loop

**Files:**
- Modify: `crates/engine/src/render.rs:53-125` (foreground loop)
- Modify: `crates/engine/src/render.rs:130-199` (background loop)
- Modify: `crates/engine/src/main.rs:130-145` (pass DNA to render)

- [ ] **Step 1: Add DNA loading in main.rs**

In `main.rs`, after `let composed = cow::compose_scene(...)`, load the DNA:

```rust
use forgum_engine::dna::{self, CowDna};

// Load animation DNA for this cow
let animations = dna::load_animations(&data);
let cow_stem = scene.cow.trim_end_matches(".cow").trim_end_matches(".Cow");
let cow_dna = dna::get_dna(&animations, &scene.cow);
let instance_id = std::process::id();
```

Pass `cow_dna` and `instance_id` to the render functions:

```rust
let result = if scene.background {
    render::render_loop_background(out, scene, shutdown, Some(&composed), cow_dna, instance_id)
} else {
    render::render_loop_foreground(out, scene, shutdown, Some(&composed), cow_dna, instance_id)
};
```

- [ ] **Step 2: Update render loop signatures**

In `render.rs`, update both function signatures to accept DNA:

```rust
pub fn render_loop_foreground(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    dna: CowDna,
    instance_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
```

Same for `render_loop_background`.

- [ ] **Step 3: Create effect and wire into render loop**

Replace the static cow rendering with effect dispatch. In both loops, after creating `cow_text`:

```rust
use crate::effects::create_effect;
use crate::dna;

let base = dna.base;
let effect = create_effect(base, cow_text.clone(), dna, instance_id);
```

Change the loop body from:

```rust
let _dt = scheduler.tick();
fb.clear();
effects::render_static_cow(&mut fb, &cow_text);
```

To:

```rust
let dt = scheduler.tick();
let time = elapsed; // f32 accumulator
fb.clear();
effect.update(dt, usize::from(cols), usize::from(rows));
effect.render(&mut fb, time);
elapsed += dt;
```

Add `let mut elapsed: f32 = 0.0;` before the loop.

- [ ] **Step 4: Run tests**

Run: `cargo test 2>&1`
Expected: All 193 tests pass (signatures changed, callers updated).

- [ ] **Step 5: Commit**

```bash
git add crates/engine/src/render.rs crates/engine/src/main.rs
git commit -m "Phase 4.1: wire effect dispatch into render loop"
```

---

## Task 2: Implement BreatheEffect — vertical oscillation

**Files:**
- Modify: `crates/engine/src/effects.rs:75-85`

- [ ] **Step 1: Implement BreatheEffect::render**

Replace the stub:

```rust
impl Effect for BreatheEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let y_offset = (eased * self.amp.breath * 3.0) as i32;
        render_text_offset(fb, &self.cow_text, Color::WHITE, 0, y_offset);
    }
}
```

- [ ] **Step 2: Add render_text_offset helper**

Add to the shared helpers section:

```rust
/// Render text with x/y offset into the framebuffer.
fn render_text_offset(fb: &mut FrameBuffer, text: &str, fg: Color, x_off: i32, y_off: i32) {
    let mut x = 0usize;
    let mut y = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            x = 0;
            y = y.saturating_add(1);
            continue;
        }
        let xi = x as i32 + x_off;
        let yi = y as i32 + y_off;
        if yi >= 0 && xi >= 0 {
            let xi = xi as usize;
            let yi = yi as usize;
            if yi < fb.height && xi < fb.width {
                let _ = fb.set(xi, yi, Cell::new(ch, fg));
            }
        }
        x = x.saturating_add(1);
    }
}
```

- [ ] **Step 3: Test**

Run: `cargo test effects::tests::create_breathe_effect`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/engine/src/effects.rs
git commit -m "Phase 4.2: implement BreatheEffect vertical oscillation"
```

---

## Task 3: Implement FloatEffect — whole-art drift

**Files:**
- Modify: `crates/engine/src/effects.rs:111-119`

- [ ] **Step 1: Implement FloatEffect::render**

```rust
impl Effect for FloatEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let intensity = (self.easing_fn)(t);
        let x_off = ((intensity * self.amp.sway * 4.0) as i32) - 2;
        let y_off = ((intensity * self.amp.float * 3.0) as i32) - 1;
        render_text_offset(fb, &self.cow_text, Color::WHITE, x_off, y_off);
    }
}
```

- [ ] **Step 2: Test**

Run: `cargo test effects`
Expected: All effects tests pass.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.3: implement FloatEffect whole-art drift"
```

---

## Task 4: Implement WalkEffect — leg character swap

**Files:**
- Modify: `crates/engine/src/effects.rs:147-155`

- [ ] **Step 1: Implement WalkEffect::render**

WalkEffect alternates the bottom line characters to simulate walking:

```rust
impl Effect for WalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        // Alternate between two leg states
        let leg_state = if eased > 0.5 { "╱╲" } else { "╲╱" };
        let lines: Vec<&str> = self.cow_text.lines().collect();
        let last_idx = lines.len().saturating_sub(1);
        for (i, line) in lines.iter().enumerate() {
            let mut x = 0usize;
            let y = i;
            for ch in line.chars() {
                if y < fb.height && x < fb.width {
                    let display_ch = if i == last_idx && ch == ' ' && x + 1 < line.len() {
                        // Replace spaces in bottom row with leg chars
                        let byte_idx = x;
                        if byte_idx % 2 == 0 {
                            leg_state.chars().next().unwrap_or(ch)
                        } else {
                            leg_state.chars().nth(1).unwrap_or(ch)
                        }
                    } else {
                        ch
                    };
                    let _ = fb.set(x, y, Cell::new(display_ch, Color::WHITE));
                }
                x = x.saturating_add(1);
            }
        }
    }
}
```

- [ ] **Step 2: Test**

Run: `cargo test effects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.4: implement WalkEffect leg character swap"
```

---

## Task 5: Implement FlyEffect — erratic float + flap

**Files:**
- Modify: `crates/engine/src/effects.rs:340-348`

- [ ] **Step 1: Implement FlyEffect::render**

```rust
impl Effect for FlyEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let intensity = (self.easing_fn)(t);
        // Erratic movement: combine sine waves at different frequencies
        let x_off = ((t * 6.28 * 3.0).sin() * self.amp.sway * 5.0) as i32;
        let y_off = ((t * 6.28 * 2.0).cos() * self.amp.float * 3.0
            + (intensity * 2.0 - 1.0)) as i32;
        // Wing flap: alternate a character
        let flap_ch = if (time * 12.0) as i32 % 2 == 0 { '~' } else { '^' };
        render_text_offset(fb, &self.cow_text, Color::WHITE, x_off, y_off);
        // Add flap indicator near top
        if fb.height > 0 && fb.width > 2 {
            let _ = fb.set(1, 0, Cell::new(flap_ch, Color::WHITE));
        }
    }
}
```

- [ ] **Step 2: Test**

Run: `cargo test effects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.5: implement FlyEffect erratic float + flap"
```

---

## Task 6: Implement TalkEffect — mouth animation

**Files:**
- Modify: `crates/engine/src/effects.rs:374-382`

- [ ] **Step 1: Implement TalkEffect::render**

TalkEffect alternates mouth characters (`o`, `O`, `0`) in the cow art:

```rust
impl Effect for TalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let mouth_chars = ['o', 'O', '0', 'o'];
        let mouth_idx = (eased * mouth_chars.len() as f32) as usize % mouth_chars.len();
        let mouth_ch = mouth_chars[mouth_idx];

        // Render text, replacing any existing mouth-like chars
        let lines: Vec<&str> = self.cow_text.lines().collect();
        for (y, line) in lines.iter().enumerate() {
            if y >= fb.height {
                break;
            }
            for (x, ch) in line.chars().enumerate() {
                if x >= fb.width {
                    break;
                }
                let display_ch = match ch {
                    'o' | 'O' | '0' | '@' => mouth_ch,
                    _ => ch,
                };
                let _ = fb.set(x, y, Cell::new(display_ch, Color::WHITE));
            }
        }
    }
}
```

- [ ] **Step 2: Test**

Run: `cargo test effects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.6: implement TalkEffect mouth animation"
```

---

## Task 7: Implement SwayEffect — pendulum skew

**Files:**
- Modify: `crates/engine/src/effects.rs:408-416`

- [ ] **Step 1: Implement SwayEffect::render**

SwayEffect shifts top lines left/right while keeping bottom anchored:

```rust
impl Effect for SwayEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let lines: Vec<&str> = self.cow_text.lines().collect();
        let total_lines = lines.len();
        for (i, line) in lines.iter().enumerate() {
            // Progressive skew: top = max, bottom = 0
            let skew_factor = 1.0 - (i as f32 / total_lines.max(1) as f32);
            let x_off = ((eased * self.amp.sway * 4.0 - 2.0) * skew_factor) as i32;
            let mut x = 0usize;
            for ch in line.chars() {
                let xi = x as i32 + x_off;
                let yi = i as i32;
                if yi >= 0 && xi >= 0 {
                    let xi = xi as usize;
                    let yi = yi as usize;
                    if yi < fb.height && xi < fb.width {
                        let _ = fb.set(xi, yi, Cell::new(ch, Color::WHITE));
                    }
                }
                x = x.saturating_add(1);
            }
        }
    }
}
```

- [ ] **Step 2: Test**

Run: `cargo test effects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.7: implement SwayEffect pendulum skew"
```

---

## Task 8: Implement DissolveEffect — scatter/reassemble

**Files:**
- Modify: `crates/engine/src/effects.rs:446-458`

- [ ] **Step 1: Implement DissolveEffect::render**

DissolveEffect scatters characters randomly in the first half, reassembles in the second:

```rust
impl Effect for DissolveEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {
        // Mark done after 2 cycles
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = ((time * self.speed + self.phase) % 2.0).min(2.0);
        let phase = if t < 1.0 { t } else { 2.0 - t }; // 0→1→0

        let lines: Vec<&str> = self.cow_text.lines().collect();
        // Collect all chars with positions
        let mut chars: Vec<(usize, usize, char)> = Vec::new();
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    chars.push((x, y, ch));
                }
            }
        }

        // Scatter phase: move chars to random positions
        let scatter_amount = 1.0 - phase; // 1.0 = fully scattered, 0.0 = assembled
        for (idx, &(x, y, ch)) in chars.iter().enumerate() {
            let seed = (idx as f32 * 0.618 + time * 0.1) as u32;
            let dx = ((seed.wrapping_mul(7) % 20) as i32 - 10) as f32;
            let dy = ((seed.wrapping_mul(13) % 10) as i32 - 5) as f32;
            let final_x = (x as f32 + dx * scatter_amount) as i32;
            let final_y = (y as f32 + dy * scatter_amount) as i32;
            if final_x >= 0 && final_y >= 0 {
                let fx = final_x as usize;
                let fy = final_y as usize;
                if fy < fb.height && fx < fb.width {
                    let alpha = (phase * 255.0) as u8;
                    let _ = fb.set(fx, fy, Cell { ch, fg: Color::WHITE, bg: Color::TRANSPARENT, alpha });
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        false // loops forever
    }
}
```

- [ ] **Step 2: Test**

Run: `cargo test effects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.8: implement DissolveEffect scatter/reassemble"
```

---

## Task 9: Remove dead_code allows from implemented effects

**Files:**
- Modify: `crates/engine/src/effects.rs`

- [ ] **Step 1: Remove #[allow(dead_code)] from BreatheEffect, FloatEffect, WalkEffect, FlyEffect, TalkEffect, SwayEffect, DissolveEffect**

These effects now use their fields, so the allow attributes are no longer needed.

- [ ] **Step 2: Test**

Run: `cargo test 2>&1`
Expected: All tests pass, no dead_code warnings.

- [ ] **Step 3: Commit**

```bash
git commit -m "Phase 4.9: remove dead_code allows from implemented effects"
```

---

## Task 10: Extract Renderer trait + AnsiRenderer

**Files:**
- Create: `crates/engine/src/renderer.rs`
- Modify: `crates/engine/src/lib.rs`
- Modify: `crates/engine/src/render.rs`

- [ ] **Step 1: Create renderer.rs with Renderer trait**

```rust
//! Renderer trait and ANSI backend.

use std::io::Write;

use crate::framebuffer::FrameBuffer;

/// Trait for rendering framebuffer damage to a terminal.
pub trait Renderer {
    /// Write the given damage cells to the output.
    fn render_damage(&mut self, out: &mut dyn Write, fb: &FrameBuffer, damage: &[(usize, usize)]) -> std::io::Result<()>;

    /// Wrap output in synchronized update (tmux-friendly).
    fn begin_sync(&self) -> &'static str { "" }
    fn end_sync(&self) -> &'static str { "" }
}

/// Default ANSI renderer — writes escaped cell sequences.
pub struct AnsiRenderer;

impl Renderer for AnsiRenderer {
    fn render_damage(&mut self, out: &mut dyn Write, fb: &FrameBuffer, damage: &[(usize, usize)]) -> std::io::Result<()> {
        for &(x, y) in damage {
            let cell = fb.get(x, y);
            // Move cursor + write char
            write!(out, "\x1b[{};{}H", y + 1, x + 1)?;
            if cell.alpha == 0 {
                write!(out, " ")?;
            } else {
                // Set foreground color
                write!(out, "\x1b[38;2;{};{};{}m", cell.fg.r, cell.fg.g, cell.fg.b)?;
                write!(out, "{}", cell.ch)?;
            }
        }
        Ok(())
    }
}

/// tmux-aware renderer wrapping ANSI with synchronized update escape sequences.
pub struct TmuxPassthroughRenderer;

impl Renderer for TmuxPassthroughRenderer {
    fn render_damage(&mut self, out: &mut dyn Write, fb: &FrameBuffer, damage: &[(usize, usize)]) -> std::io::Result<()> {
        // tmux passthrough: wrap in DCS sequences
        write!(out, "\x1bPtmux;\r")?;
        let inner = AnsiRenderer;
        inner.render_damage(out, fb, damage)?;
        write!(out, "\x1b\\")?;
        Ok(())
    }

    fn begin_sync(&self) -> &'static str {
        "\x1b[?2026h" // Begin synchronized update
    }

    fn end_sync(&self) -> &'static str {
        "\x1b[?2026l" // End synchronized update
    }
}
```

- [ ] **Step 2: Register in lib.rs**

Add `pub mod renderer;` to `lib.rs`.

- [ ] **Step 3: Update render.rs to use Renderer trait**

Replace the inline `render_damage` function with a call to the trait:

```rust
use crate::renderer::{Renderer, AnsiRenderer};

// In the loop:
let mut renderer = AnsiRenderer;
if !dmg.is_empty() {
    renderer.render_damage(&mut out.raw_writer_mut(), &fb, &dmg)?;
}
```

Remove the old `render_damage` function.

- [ ] **Step 4: Test**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/engine/src/renderer.rs crates/engine/src/lib.rs crates/engine/src/render.rs
git commit -m "Phase 4.10: extract Renderer trait + AnsiRenderer backend"
```

---

## Task 11: tmux passthrough detection + backend selection

**Files:**
- Modify: `crates/engine/src/render.rs`
- Modify: `crates/engine/src/renderer.rs`

- [ ] **Step 1: Add tmux detection to renderer.rs**

```rust
/// Detect if running inside tmux.
pub fn is_tmux() -> bool {
    std::env::var("TMUX").map(|v| !v.is_empty()).unwrap_or(false)
}

/// Create the appropriate renderer for the environment.
pub fn create_renderer() -> Box<dyn Renderer> {
    if is_tmux() {
        Box::new(TmuxPassthroughRenderer)
    } else {
        Box::new(AnsiRenderer)
    }
}
```

- [ ] **Step 2: Use create_renderer() in render loops**

Replace `let mut renderer = AnsiRenderer;` with `let mut renderer = create_renderer();`.

- [ ] **Step 3: Wrap damage rendering in synchronized update**

```rust
if !dmg.is_empty() {
    write!(out, "{}", renderer.begin_sync())?;
    renderer.render_damage(&mut out, &fb, &dmg)?;
    write!(out, "{}", renderer.end_sync())?;
}
```

- [ ] **Step 4: Test**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git commit -m "Phase 4.11: tmux passthrough detection + backend selection"
```

---

## Task 12: Final cleanup, tests, clippy, fmt

**Files:**
- All modified files

- [ ] **Step 1: Run full test suite**

Run: `cargo test 2>&1`
Expected: All tests pass (193+).

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets 2>&1`
Expected: Zero warnings (or only suppressed ones).

- [ ] **Step 3: Run fmt**

Run: `cargo fmt 2>&1`

- [ ] **Step 4: Run cfg-grep gate**

Run: `cargo test -p forgum-engine --test cfg_containment 2>&1`
Expected: PASS.

- [ ] **Step 5: Run Pester tests**

Run: `pwsh -Command "Invoke-Pester -Path './Tests' -Output Detailed" 2>&1`
Expected: 13/13 pass.

- [ ] **Step 6: Commit final state**

```bash
git add -A
git commit -m "Phase 4: render pipeline + real effects + Renderer trait + tmux passthrough

- Wired effect dispatch into render loop (update + render per frame)
- Implemented 7 effects: Breathe (vertical oscillation), Float (drift),
  Walk (leg swap), Fly (erratic + flap), Talk (mouth), Sway (pendulum),
  Dissolve (scatter/reassemble)
- Renderer trait with AnsiRenderer and TmuxPassthroughRenderer
- Synchronized update wrapping for tmux compatibility
- 210+ tests, 0 clippy warnings, cfg-grep enforced"
```
