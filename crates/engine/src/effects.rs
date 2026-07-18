//! Effect system — trait + 10 base animation types.
//!
//! Each effect owns its state, updates per-frame, and renders into the
//! framebuffer. The render loop calls `effect.update(dt)` then
//! `effect.render(fb)` each frame.

use crate::color::{self, gaussian_glow, lerp_palette, parse_hex};
use crate::dna::{instance_phase, Amplitude, BaseAnim, CowDna};
use crate::easing;
use crate::framebuffer::{Cell, Color, FrameBuffer};
use crate::particles::{seed_frame_rng, spawn_for_type, ParticlePool};

/// Trait for all animation effects.
pub trait Effect: Send + Sync {
    /// Advance the effect by `dt` seconds.
    fn update(&mut self, dt: f32, cols: usize, rows: usize);

    /// Render the current frame into `fb`.
    fn render(&self, fb: &mut FrameBuffer, time: f32);

    /// Returns true when a one-shot effect is finished.
    fn is_done(&self) -> bool {
        false
    }

    /// Notify of terminal resize.
    fn on_resize(&mut self, _cols: usize, _rows: usize) {}
}

// ── Static (no animation) ──────────────────────────────────────────

/// The Phase 0 static cow — no animation, just the text.
#[derive(Debug)]
pub struct StaticEffect {
    cow_text: String,
}

impl StaticEffect {
    pub fn new(cow_text: String) -> Self {
        Self { cow_text }
    }
}

impl Effect for StaticEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, _time: f32) {
        render_text(fb, &self.cow_text, Color::WHITE);
    }
}

// ── Breathe ────────────────────────────────────────────────────────

/// Subtle chest/belly expansion and contraction.
#[derive(Debug)]
pub struct BreatheEffect {
    cow_text: String,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl BreatheEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for BreatheEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let y_offset = (eased * self.amp.breath * 3.0) as i32;
        render_text_offset(fb, &self.cow_text, Color::WHITE, 0, y_offset);
    }
}

// ── Float / Bob ────────────────────────────────────────────────────

/// Whole-art vertical/horizontal drift.
#[derive(Debug)]
pub struct FloatEffect {
    cow_text: String,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl FloatEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

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

// ── Walk / Trot ────────────────────────────────────────────────────

/// Bottom-row leg character swap.
#[derive(Debug)]
pub struct WalkEffect {
    cow_text: String,
    _amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl WalkEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            _amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for WalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        // Alternate between two leg states
        let (leg_l, leg_r) = if eased > 0.5 {
            ('╱', '╲')
        } else {
            ('╲', '╱')
        };
        let lines: Vec<&str> = self.cow_text.lines().collect();
        let last_idx = lines.len().saturating_sub(1);
        for (i, line) in lines.iter().enumerate() {
            let mut x = 0usize;
            for ch in line.chars() {
                let display_ch = if i == last_idx && ch == ' ' {
                    if x % 2 == 0 {
                        leg_l
                    } else {
                        leg_r
                    }
                } else {
                    ch
                };
                if i < fb.height && x < fb.width {
                    let _ = fb.set(x, i, Cell::new(display_ch, Color::WHITE));
                }
                x = x.saturating_add(1);
            }
            let _ = last_idx; // suppress unused warning
        }
    }
}

// ── Particles (Fire/Bubbles/Stars/Zzz/Pulse/Glitch) ────────────────

/// Contextual particle emitter overlay.
#[derive(Debug)]
pub struct ParticlesEffect {
    cow_text: String,
    dna: CowDna,
    pool: ParticlePool,
    spawn_timer: f32,
    phase: f32,
    instance_id: u32,
}

impl ParticlesEffect {
    pub fn new(cow_text: String, dna: CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            dna,
            pool: ParticlePool::new(),
            spawn_timer: 0.0,
            phase,
            instance_id,
        }
    }
}

impl Effect for ParticlesEffect {
    fn update(&mut self, dt: f32, cols: usize, rows: usize) {
        seed_frame_rng(self.dna.phase_seed.wrapping_add(self.instance_id));
        self.spawn_timer += dt;
        let interval = if self.dna.particles.rate > 0 {
            1.0 / self.dna.particles.rate as f32
        } else {
            1.0
        };
        if self.spawn_timer >= interval {
            self.spawn_timer -= interval;
            let palette = color::parse_palette(&self.dna.particles.palette);
            spawn_for_type(
                &mut self.pool,
                self.dna.particles.r#type,
                cols as f32 / 2.0,
                rows as f32 * 0.3,
                &palette,
                self.phase + dt,
                cols,
                rows,
            );
        }
        self.pool.update(dt);
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        render_text(fb, &self.cow_text, Color::WHITE);
        self.pool.render(fb, time, easing::expo_out);
    }
}

// ── Pulse / Glow ───────────────────────────────────────────────────

/// Color cycling (rainbow sweep or localized glow).
#[derive(Debug)]
pub struct PulseEffect {
    cow_text: String,
    palette: Vec<(u8, u8, u8)>,
    glow_color: Color,
    glow_radius: f32,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl PulseEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let palette = color::parse_palette(&dna.palette);
        let glow_color = parse_hex(&dna.glow.color)
            .map(|(r, g, b)| Color::rgb(r, g, b))
            .unwrap_or(Color::WHITE);
        Self {
            cow_text,
            palette,
            glow_color,
            glow_radius: dna.glow.radius,
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for PulseEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let intensity = (self.easing_fn)(t);

        // Render cow text with color from palette
        let color = if self.palette.is_empty() {
            Color::WHITE
        } else {
            let (r, g, b) = lerp_palette(&self.palette, intensity);
            Color::rgb(r, g, b)
        };
        render_text(fb, &self.cow_text, color);

        // Apply glow at center of cow
        let cx = fb.width as f32 / 2.0;
        let cy = fb.height as f32 * 0.3;
        let glow_intensity = intensity * 0.5;
        apply_glow(
            fb,
            cx,
            cy,
            self.glow_radius,
            self.glow_color,
            glow_intensity,
        );
    }
}

// ── Glitch ─────────────────────────────────────────────────────────

/// Random character swap with binary/hex.
#[derive(Debug)]
pub struct GlitchEffect {
    cow_text: String,
    phase: f32,
    speed: f32,
}

impl GlitchEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for GlitchEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        render_text(fb, &self.cow_text, Color::WHITE);
        // Randomly overwrite some cells with glitch chars
        let glitch_chars = ['0', '1', '#', '@', '█', '▓'];
        let t = time * self.speed + self.phase;
        let intensity = (t * 3.0).sin() * 0.5 + 0.5;
        let count = (intensity * 10.0) as usize;
        if fb.width == 0 || fb.height == 0 {
            return;
        }
        for i in 0..count {
            let seed = (t * 100.0 + i as f32) as u32;
            let x = ((seed.wrapping_mul(7)) as usize) % fb.width;
            let y = ((seed.wrapping_mul(13)) as usize) % fb.height;
            let ch = glitch_chars[(seed as usize) % glitch_chars.len()];
            let c = if seed % 2 == 0 {
                Color::rgb(0, 255, 0)
            } else {
                Color::rgb(255, 0, 0)
            };
            let _ = fb.set(x, y, Cell::new(ch, c));
        }
    }
}

// ── Fly / Hover ────────────────────────────────────────────────────

/// Fast erratic float + wing flap.
#[derive(Debug)]
pub struct FlyEffect {
    cow_text: String,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl FlyEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for FlyEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let intensity = (self.easing_fn)(t);
        // Erratic movement: combine sine waves at different frequencies
        let x_off = ((t * std::f32::consts::TAU * 3.0).sin() * self.amp.sway * 5.0) as i32;
        let y_off = ((t * std::f32::consts::TAU * 2.0).cos() * self.amp.float * 3.0
            + (intensity * 2.0 - 1.0)) as i32;
        render_text_offset(fb, &self.cow_text, Color::WHITE, x_off, y_off);
        // Wing flap indicator near top
        let flap_ch = if (time * 12.0) as i32 % 2 == 0 {
            '~'
        } else {
            '^'
        };
        if fb.height > 0 && fb.width > 2 {
            let _ = fb.set(1, 0, Cell::new(flap_ch, Color::WHITE));
        }
    }
}

// ── Talk / Chew ────────────────────────────────────────────────────

/// Mouth + eye region animation.
#[derive(Debug)]
pub struct TalkEffect {
    cow_text: String,
    _amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl TalkEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            _amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for TalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let mouth_chars = ['o', 'O', '0', 'o'];
        let mouth_idx = (eased * mouth_chars.len() as f32) as usize % mouth_chars.len();
        let mouth_ch = mouth_chars[mouth_idx];

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

// ── Sway / Pendulum ────────────────────────────────────────────────

/// Top-half skew, bottom anchored.
#[derive(Debug)]
pub struct SwayEffect {
    cow_text: String,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl SwayEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
        }
    }
}

impl Effect for SwayEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let lines: Vec<&str> = self.cow_text.lines().collect();
        let total_lines = lines.len().max(1);
        for (i, line) in lines.iter().enumerate() {
            // Progressive skew: top = max, bottom = 0
            let skew_factor = 1.0 - (i as f32 / total_lines as f32);
            let x_off = ((eased * self.amp.sway * 4.0 - 2.0) * skew_factor) as i32;
            let mut x = 0usize;
            for ch in line.chars() {
                let xi = x as i32 + x_off;
                if i < fb.height && xi >= 0 {
                    let xi = xi as usize;
                    if xi < fb.width {
                        let _ = fb.set(xi, i, Cell::new(ch, Color::WHITE));
                    }
                }
                x = x.saturating_add(1);
            }
        }
    }
}

// ── Dissolve ───────────────────────────────────────────────────────

/// Break art into falling chars, reassemble.
#[derive(Debug)]
pub struct DissolveEffect {
    cow_text: String,
    _easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    _done: bool,
}

impl DissolveEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            _easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            _done: false,
        }
    }
}

impl Effect for DissolveEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let cycle = (time * self.speed + self.phase) % 2.0;
        let t = if cycle < 1.0 { cycle } else { 2.0 - cycle }; // 0→1→0
        let scatter = 1.0 - t; // 1.0 = scattered, 0.0 = assembled

        let lines: Vec<&str> = self.cow_text.lines().collect();
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let seed = ((x as f32 * 0.618 + y as f32 * 0.382 + time * 0.1) * 1000.0) as u32;
                let dx = ((seed.wrapping_mul(7) % 20) as i32 - 10) as f32;
                let dy = ((seed.wrapping_mul(13) % 10) as i32 - 5) as f32;
                let final_x = (x as f32 + dx * scatter) as i32;
                let final_y = (y as f32 + dy * scatter) as i32;
                if final_x >= 0 && final_y >= 0 {
                    let fx = final_x as usize;
                    let fy = final_y as usize;
                    if fy < fb.height && fx < fb.width {
                        let alpha = (t * 255.0) as u8;
                        let _ = fb.set(
                            fx,
                            fy,
                            Cell {
                                ch,
                                fg: Color::WHITE,
                                bg: Color::TRANSPARENT,
                                alpha,
                            },
                        );
                    }
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        false
    }
}

// ── Shared helpers ─────────────────────────────────────────────────

/// Render text into the framebuffer at row 0.
fn render_text(fb: &mut FrameBuffer, text: &str, fg: Color) {
    render_text_offset(fb, text, fg, 0, 0);
}

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

/// Apply a radial glow effect centered at (cx, cy).
fn apply_glow(fb: &mut FrameBuffer, cx: f32, cy: f32, radius: f32, color: Color, intensity: f32) {
    if intensity <= 0.0 || radius <= 0.0 {
        return;
    }
    let r = radius as i32;
    let cx_i = cx as i32;
    let cy_i = cy as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let x = cx_i + dx;
            let y = cy_i + dy;
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= fb.width || y >= fb.height {
                continue;
            }
            let glow = gaussian_glow(dx as f32, dy as f32, radius) * intensity;
            if glow < 0.05 {
                continue;
            }
            let alpha = (glow * 255.0) as u8;
            let existing = fb.get(x, y);
            if existing.alpha == 0 {
                // Empty cell — fill with glow
                let _ = fb.set(
                    x,
                    y,
                    Cell {
                        ch: '·',
                        fg: Color {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: alpha,
                        },
                        bg: Color::TRANSPARENT,
                        alpha,
                    },
                );
            }
        }
    }
}

/// Create an effect from a base animation type.
pub fn create_effect(
    base: BaseAnim,
    cow_text: String,
    dna: CowDna,
    instance_id: u32,
) -> Box<dyn Effect> {
    match base {
        BaseAnim::Breathe => Box::new(BreatheEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Float => Box::new(FloatEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Walk => Box::new(WalkEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Particles => Box::new(ParticlesEffect::new(cow_text, dna, instance_id)),
        BaseAnim::Pulse => Box::new(PulseEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Glitch => Box::new(GlitchEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Fly => Box::new(FlyEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Talk => Box::new(TalkEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Sway => Box::new(SwayEffect::new(cow_text, &dna, instance_id)),
        BaseAnim::Dissolve => Box::new(DissolveEffect::new(cow_text, &dna, instance_id)),
    }
}

// ── Legacy static cow text (backward compat) ───────────────────────

/// Write the cow text into `fb` at row 0 (Phase 0 compat).
pub fn render_static_cow(fb: &mut FrameBuffer, cow_text: &str) {
    render_text(fb, cow_text, Color::WHITE);
}

/// Returns the canonical "default" cow art for Phase 0.
#[must_use]
pub fn default_cow_text() -> &'static str {
    r#"        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::GlowDna;

    const COW: &str = "  ^__^  \n (oo)   \n(__)    ";

    /// Count non-space cells in the framebuffer.
    fn count_non_space(fb: &FrameBuffer) -> usize {
        let mut n = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if fb.get(x, y).ch != ' ' {
                    n += 1;
                }
            }
        }
        n
    }

    /// Count cells matching a predicate.
    #[allow(dead_code)]
    fn count_cells(fb: &FrameBuffer, pred: impl Fn(char) -> bool) -> usize {
        let mut n = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if pred(fb.get(x, y).ch) {
                    n += 1;
                }
            }
        }
        n
    }

    // ── StaticEffect ──────────────────────────────────────────────

    #[test]
    fn static_cow_renders_exact_art() {
        let mut fb = FrameBuffer::new(80, 24);
        render_static_cow(&mut fb, default_cow_text());
        fb.swap();
        // default_cow_text() starts with "        \   ^__^"
        // Row 0: 8 spaces, \, 3 spaces, ^__^ → ^ at column 12
        assert_eq!(fb.get(12, 0).ch, '^', "caret (^) must be at (12,0)");
        assert_eq!(fb.get(13, 0).ch, '_');
        assert_eq!(fb.get(14, 0).ch, '_');
        assert_eq!(fb.get(15, 0).ch, '^');
        // Row 1: "         \\  (oo)\\_______" → 9 spaces, \, 2 spaces, (oo) → ( at column 12
        assert_eq!(fb.get(12, 1).ch, '(');
        assert_eq!(fb.get(13, 1).ch, 'o');
        assert_eq!(fb.get(14, 1).ch, 'o');
        assert_eq!(fb.get(15, 1).ch, ')');
        // Row 3: must contain || for the legs
        let row3: String = (0..30).map(|x| fb.get(x, 3).ch).collect();
        assert!(
            row3.contains("||"),
            "row 3 must contain || for the legs: got {row3:?}"
        );
    }

    #[test]
    fn static_cow_all_chars_white() {
        let mut fb = FrameBuffer::new(80, 24);
        render_static_cow(&mut fb, default_cow_text());
        fb.swap();
        for y in 0..5 {
            for x in 0..20 {
                let cell = fb.get(x, y);
                if cell.ch != ' ' && cell.ch != '\0' {
                    assert_eq!(
                        cell.fg,
                        Color::WHITE,
                        "non-space cell at ({x},{y}) ch={:?} must be WHITE",
                        cell.ch
                    );
                }
            }
        }
    }

    #[test]
    fn empty_text_produces_no_damage() {
        let mut fb = FrameBuffer::new(10, 10);
        render_static_cow(&mut fb, "");
        assert!(
            fb.compute_damage().is_empty(),
            "rendering empty text should produce zero damage"
        );
    }

    #[test]
    fn render_cow_text_fits_in_framebuffer() {
        // A 1x1 framebuffer should only show the first character
        let mut fb = FrameBuffer::new(1, 1);
        render_static_cow(&mut fb, "ABC\nDEF");
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'A');
        // Second row is out of bounds for height=1
    }

    // ── BreatheEffect ─────────────────────────────────────────────

    #[test]
    fn breathe_at_different_times_produces_different_offsets() {
        let mut dna = CowDna::default();
        dna.amplitude.breath = 5.0;
        dna.speed = 1.0;
        let effect = BreatheEffect::new(COW.to_string(), &dna, 0);

        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.75);
        fb1.swap();

        // Find the first non-space cell in each frame
        let find_first_char = |fb: &FrameBuffer| -> Option<(usize, usize, char)> {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    let ch = fb.get(x, y).ch;
                    if ch != ' ' {
                        return Some((x, y, ch));
                    }
                }
            }
            None
        };

        let pos0 = find_first_char(&fb0);
        let pos1 = find_first_char(&fb1);
        assert!(pos0.is_some(), "breathe at t=0 must render something");
        assert!(pos1.is_some(), "breathe at t=0.75 must render something");

        // The Y position should differ due to breath amplitude
        let (_, y0, _) = pos0.unwrap();
        let (_, y1, _) = pos1.unwrap();
        assert_ne!(
            y0, y1,
            "breathe Y offset should differ between t=0 and t=0.75 (amplitude=5.0)"
        );
    }

    #[test]
    fn zero_amplitude_produces_no_offset() {
        let find_first = |fb: &FrameBuffer| -> Option<(usize, usize)> {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return Some((x, y));
                    }
                }
            }
            None
        };

        // Breathe: zero amplitude → no Y drift
        let mut dna = CowDna::default();
        dna.amplitude.breath = 0.0;
        dna.speed = 1.0;
        let effect = BreatheEffect::new(COW.to_string(), &dna, 0);
        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.5);
        fb1.swap();
        assert_eq!(
            find_first(&fb0),
            find_first(&fb1),
            "breathe zero amplitude = no Y drift"
        );

        // Float: zero amplitude → no drift
        let mut dna = CowDna::default();
        dna.amplitude.sway = 0.0;
        dna.amplitude.float = 0.0;
        dna.speed = 1.0;
        let effect = FloatEffect::new(COW.to_string(), &dna, 0);
        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.5);
        fb1.swap();
        assert_eq!(
            find_first(&fb0),
            find_first(&fb1),
            "float zero amplitude = no drift"
        );
    }

    #[test]
    fn breathe_instance_phase_offsets_multiple_instances() {
        let dna = CowDna::default();
        let e0 = BreatheEffect::new(COW.to_string(), &dna, 0);
        let e1 = BreatheEffect::new(COW.to_string(), &dna, 1);

        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        e0.render(&mut fb0, 0.3);
        fb0.swap();
        e1.render(&mut fb1, 0.3);
        fb1.swap();

        // Two instances with different IDs should render at different offsets
        let find_y = |fb: &FrameBuffer| -> usize {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return y;
                    }
                }
            }
            fb.height
        };
        assert_ne!(
            find_y(&fb0),
            find_y(&fb1),
            "different instance_ids should produce different Y offsets"
        );
    }

    // ── FloatEffect ───────────────────────────────────────────────

    #[test]
    fn float_produces_horizontal_and_vertical_drift() {
        let mut dna = CowDna::default();
        dna.amplitude.sway = 3.0;
        dna.amplitude.float = 3.0;
        dna.speed = 1.0;
        let effect = FloatEffect::new(COW.to_string(), &dna, 0);

        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.75);
        fb1.swap();

        let find_first = |fb: &FrameBuffer| -> Option<(usize, usize)> {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return Some((x, y));
                    }
                }
            }
            None
        };

        let p0 = find_first(&fb0).expect("float t=0 must render");
        let p1 = find_first(&fb1).expect("float t=0.75 must render");
        // Position should differ in at least one axis
        assert!(
            p0.0 != p1.0 || p0.1 != p1.1,
            "float position should differ: t=0 at {:?}, t=0.75 at {:?}",
            p0,
            p1
        );
    }

    // ── WalkEffect ────────────────────────────────────────────────

    #[test]
    fn walk_legs_alternate_between_frames() {
        let dna = CowDna::default();
        let effect_a = WalkEffect::new(COW.to_string(), &dna, 0);
        let effect_b = WalkEffect::new(COW.to_string(), &dna, 0);
        let mut fb_a = FrameBuffer::new(80, 24);
        let mut fb_b = FrameBuffer::new(80, 24);
        effect_a.render(&mut fb_a, 0.25);
        fb_a.swap();
        effect_b.render(&mut fb_b, 0.75);
        fb_b.swap();

        // Last row of the COW string is row 2 (3 lines: "  ^__^  ", " (oo)   ", "(__)    ")
        let last_row = 2;
        let legs_a: Vec<char> = (0..fb_a.width).map(|x| fb_a.get(x, last_row).ch).collect();
        let legs_b: Vec<char> = (0..fb_b.width).map(|x| fb_b.get(x, last_row).ch).collect();

        // At least one leg char must be present
        let has_slash = |v: &[char]| v.iter().any(|c| *c == '╱' || *c == '╲');
        assert!(
            has_slash(&legs_a),
            "walk t=0.25 must have leg chars: {legs_a:?}"
        );
        assert!(
            has_slash(&legs_b),
            "walk t=0.75 must have leg chars: {legs_b:?}"
        );

        // The legs must differ between the two frames
        assert_ne!(
            legs_a, legs_b,
            "walk legs must alternate between t=0.25 and t=0.75"
        );
    }

    #[test]
    fn walk_only_modifies_last_row() {
        let dna = CowDna::default();
        let effect = WalkEffect::new(COW.to_string(), &dna, 0);
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.25);
        fb.swap();

        // COW = "  ^__^  \n (oo)   \n(__)    " → ^ at x=2, _ at x=3
        assert_eq!(fb.get(2, 0).ch, '^', "walk should not modify row 0");
        assert_eq!(fb.get(3, 0).ch, '_', "walk should not modify row 0");
        // Row 1 should also be unchanged
        assert_eq!(fb.get(1, 1).ch, '(', "walk should not modify row 1");
    }

    // ── GlitchEffect ──────────────────────────────────────────────

    #[test]
    fn glitch_intensity_varies_with_time() {
        let dna = CowDna::default();
        let glitch_chars = ['0', '1', '#', '@', '█', '▓'];

        let count_glitch = |fb: &FrameBuffer| -> usize {
            let mut n = 0;
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if glitch_chars.contains(&fb.get(x, y).ch) {
                        n += 1;
                    }
                }
            }
            n
        };

        // peak: sin(t*3) ≈ 1 → intensity ≈ 1 → many glitch chars
        let effect_peak = GlitchEffect::new(COW.to_string(), &dna, 0);
        let mut fb_peak = FrameBuffer::new(80, 24);
        effect_peak.render(&mut fb_peak, std::f32::consts::FRAC_PI_6);
        fb_peak.swap();
        let peak_count = count_glitch(&fb_peak);
        assert!(
            peak_count > 0,
            "glitch at peak intensity should produce glitch characters"
        );

        // trough: sin(t*3) ≈ -1 → intensity ≈ 0 → fewer or zero
        let effect_trough = GlitchEffect::new(COW.to_string(), &dna, 0);
        let mut fb_trough = FrameBuffer::new(80, 24);
        effect_trough.render(&mut fb_trough, std::f32::consts::FRAC_PI_3);
        fb_trough.swap();
        let trough_count = count_glitch(&fb_trough);

        assert!(
            peak_count >= trough_count,
            "peak intensity ({peak_count}) should have >= trough ({trough_count})"
        );
    }

    // ── FlyEffect ─────────────────────────────────────────────────

    #[test]
    fn fly_renders_wing_flap_indicator() {
        let dna = CowDna::default();
        let effect = FlyEffect::new(COW.to_string(), &dna, 0);
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.0);
        fb.swap();

        // FlyEffect sets (1,0) to '~' or '^' as wing flap
        let ch = fb.get(1, 0).ch;
        assert!(
            ch == '~' || ch == '^',
            "fly wing flap at (1,0) must be '~' or '^', got '{ch}'"
        );
    }

    #[test]
    fn fly_wing_flap_toggles() {
        let dna = CowDna::default();
        let e1 = FlyEffect::new(COW.to_string(), &dna, 0);
        let e2 = FlyEffect::new(COW.to_string(), &dna, 0);
        let mut fb1 = FrameBuffer::new(80, 24);
        let mut fb2 = FrameBuffer::new(80, 24);
        e1.render(&mut fb1, 0.0);
        fb1.swap();
        e2.render(&mut fb2, 0.05);
        fb2.swap();
        // (time * 12.0) as i32 % 2 toggles between 0 and 1
        // at t=0: 0 % 2 = 0 => '~'; at t=0.05: (0.6) as i32 = 0 => '~' still
        // need enough time delta to toggle: t=0.1 => (1.2) as i32 = 1 => '^'
        let e3 = FlyEffect::new(COW.to_string(), &dna, 0);
        let mut fb3 = FrameBuffer::new(80, 24);
        e3.render(&mut fb3, 0.1);
        fb3.swap();
        assert_ne!(
            fb1.get(1, 0).ch,
            fb3.get(1, 0).ch,
            "wing flap should toggle between frames"
        );
    }

    // ── TalkEffect ────────────────────────────────────────────────

    #[test]
    fn talk_replaces_and_cycles_mouth_chars() {
        let dna = CowDna::default();
        let mouth_chars = ['o', 'O', '0', 'o'];

        // Render at 4 different times: verify mouth chars are replaced AND cycle
        let mut seen = std::collections::HashSet::new();
        for i in 0..4 {
            let effect = TalkEffect::new(COW.to_string(), &dna, 0);
            let mut fb = FrameBuffer::new(80, 24);
            let t = i as f32 / 4.0;
            effect.render(&mut fb, t);
            fb.swap();

            // COW = "  ^__^  \n (oo)   \n(__)    " → first 'o' at (2,1)
            let ch = fb.get(2, 1).ch;
            assert!(
                mouth_chars.contains(&ch),
                "talk must replace 'o' with mouth char, got '{ch}' at t={t}"
            );
            seen.insert(ch);
        }
        assert!(
            seen.len() >= 2,
            "talk mouth should cycle through multiple chars, saw: {:?}",
            seen
        );
    }

    // ── SwayEffect ────────────────────────────────────────────────

    #[test]
    fn sway_top_rows_shift_more_than_bottom() {
        let mut dna = CowDna::default();
        dna.amplitude.sway = 4.0;
        dna.speed = 1.0;
        let effect = SwayEffect::new(COW.to_string(), &dna, 0);

        let mut fb_t0 = FrameBuffer::new(80, 24);
        let mut fb_t1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb_t0, 0.0);
        fb_t0.swap();
        effect.render(&mut fb_t1, 0.25);
        fb_t1.swap();

        // Top row should shift more than bottom row
        let find_x_at_row = |fb: &FrameBuffer, row: usize| -> Option<usize> {
            (0..fb.width).find(|&x| fb.get(x, row).ch != ' ')
        };

        // Row 0 is the top of the cow (highest skew), row 2 is bottom (zero skew)
        if let (Some(x0_top), Some(x1_top)) = (find_x_at_row(&fb_t0, 0), find_x_at_row(&fb_t1, 0)) {
            if let (Some(x0_bot), Some(x1_bot)) =
                (find_x_at_row(&fb_t0, 2), find_x_at_row(&fb_t1, 2))
            {
                let top_shift = (x0_top as i32 - x1_top as i32).unsigned_abs();
                let bot_shift = (x0_bot as i32 - x1_bot as i32).unsigned_abs();
                // Top should shift at least as much as bottom
                assert!(
                    top_shift >= bot_shift,
                    "sway top shift ({top_shift}) should be >= bottom shift ({bot_shift})"
                );
            }
        }
    }

    // ── DissolveEffect ────────────────────────────────────────────

    #[test]
    fn dissolve_at_assembled_has_low_scatter() {
        let dna = CowDna::default();
        let effect = DissolveEffect::new(COW.to_string(), &dna, 0);
        let mut fb = FrameBuffer::new(80, 24);
        // At cycle midpoint (t=1.0 in 0→1→0 cycle), scatter is 0 (assembled)
        effect.render(&mut fb, 1.0);
        fb.swap();

        // When assembled, the cow art should be close to original positions
        // Count non-space cells — should be similar to cow character count
        let count = count_non_space(&fb);
        assert!(
            count > 10,
            "dissolve assembled (t=1.0) should render visible cow art, got {count} cells"
        );
    }

    #[test]
    fn dissolve_at_scattered_has_different_positions() {
        let dna = CowDna::default();
        let effect_assembled = DissolveEffect::new(COW.to_string(), &dna, 0);
        let effect_scattered = DissolveEffect::new(COW.to_string(), &dna, 0);

        let mut fb_assembled = FrameBuffer::new(80, 24);
        let mut fb_scattered = FrameBuffer::new(80, 24);

        // assembled: cycle=1.0 → t=1.0, scatter=0
        effect_assembled.render(&mut fb_assembled, 1.0);
        fb_assembled.swap();

        // scattered: cycle=0.0 → t=0.0, scatter=1.0
        effect_scattered.render(&mut fb_scattered, 0.0);
        fb_scattered.swap();

        // The cell positions should differ
        let mut differ = false;
        for y in 0..fb_assembled.height {
            for x in 0..fb_assembled.width {
                if fb_assembled.get(x, y).ch != fb_scattered.get(x, y).ch {
                    differ = true;
                    break;
                }
            }
            if differ {
                break;
            }
        }
        assert!(
            differ,
            "dissolve assembled vs scattered should produce different cell positions"
        );
    }

    // ── PulseEffect ───────────────────────────────────────────────

    #[test]
    fn pulse_with_palette_produces_colored_cells() {
        let dna = CowDna {
            palette: vec!["#ff0000".to_string(), "#0000ff".to_string()],
            ..CowDna::default()
        };
        let effect = PulseEffect::new(COW.to_string(), &dna, 0);
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        let mut has_non_white = false;
        for y in 0..fb.height {
            for x in 0..fb.width {
                let c = fb.get(x, y);
                if c.ch != ' ' && (c.fg.r != 255 || c.fg.g != 255 || c.fg.b != 255) {
                    has_non_white = true;
                    break;
                }
            }
            if has_non_white {
                break;
            }
        }
        assert!(
            has_non_white,
            "pulse with palette should produce non-white colored cells"
        );
    }

    #[test]
    fn pulse_without_palette_is_white() {
        let dna = CowDna::default();
        let effect = PulseEffect::new(COW.to_string(), &dna, 0);
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        // Without palette, pulse uses Color::WHITE
        for y in 0..5 {
            for x in 0..20 {
                let c = fb.get(x, y);
                if c.ch != ' ' {
                    assert_eq!(
                        c.fg,
                        Color::WHITE,
                        "pulse without palette should use WHITE at ({x},{y})"
                    );
                }
            }
        }
    }

    #[test]
    fn pulse_applies_glow_to_empty_cells() {
        let dna = CowDna {
            palette: vec!["#ff0000".to_string()],
            glow: GlowDna {
                radius: 10.0,
                color: "#ff0000".to_string(),
                ..GlowDna::default()
            },
            ..CowDna::default()
        };
        let effect = PulseEffect::new(COW.to_string(), &dna, 0);
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        // Glow should fill some empty cells with '·' character
        let mut glow_count = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if fb.get(x, y).ch == '·' {
                    glow_count += 1;
                }
            }
        }
        assert!(
            glow_count > 0,
            "pulse glow should fill empty cells with '·' characters"
        );
    }

    // ── ParticlesEffect ───────────────────────────────────────────

    #[test]
    fn particles_renders_cow_always() {
        // Cow text must be rendered regardless of particle rate
        let find_char = |fb: &FrameBuffer, ch: char| -> bool {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch == ch {
                        return true;
                    }
                }
            }
            false
        };

        for rate in [0u32, 5, 20] {
            let mut dna = CowDna::default();
            dna.particles.rate = rate;
            let mut effect = ParticlesEffect::new(COW.to_string(), dna, 0);
            effect.update(1.0, 80, 24);
            let mut fb = FrameBuffer::new(80, 24);
            effect.render(&mut fb, 1.0);
            fb.swap();
            // COW = "  ^__^  \n (oo)   \n(__)    " → ^ at (2,0)
            assert!(
                find_char(&fb, '^'),
                "particles with rate={rate} must render cow '^'"
            );
        }
    }

    // ── create_effect dispatch ─────────────────────────────────────

    #[test]
    fn create_effect_dispatches_correct_types() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0);
            effect.update(0.1, 40, 10);
            let mut fb = FrameBuffer::new(40, 10);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert!(
                count_non_space(&fb) > 0,
                "{base:?} should render non-empty output"
            );
        }
    }

    #[test]
    fn create_effect_with_different_dna_produces_different_output() {
        let dna_fast = CowDna {
            speed: 10.0,
            amplitude: Amplitude {
                breath: 5.0,
                ..Amplitude::default()
            },
            ..CowDna::default()
        };

        let dna_slow = CowDna {
            speed: 0.1,
            amplitude: Amplitude {
                breath: 0.1,
                ..Amplitude::default()
            },
            ..CowDna::default()
        };

        let e_fast = BreatheEffect::new(COW.to_string(), &dna_fast, 0);
        let e_slow = BreatheEffect::new(COW.to_string(), &dna_slow, 0);

        let mut fb_fast = FrameBuffer::new(80, 24);
        let mut fb_slow = FrameBuffer::new(80, 24);
        // Use t=0.25 so fast (speed=10) has t*10=2.5→0.5→eased≠0,
        // while slow (speed=0.1) has t*0.1=0.025→eased≈0
        e_fast.render(&mut fb_fast, 0.25);
        fb_fast.swap();
        e_slow.render(&mut fb_slow, 0.25);
        fb_slow.swap();

        // Different DNA should produce different visual output
        let find_y = |fb: &FrameBuffer| -> usize {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return y;
                    }
                }
            }
            fb.height
        };
        assert_ne!(
            find_y(&fb_fast),
            find_y(&fb_slow),
            "different DNA (speed/amplitude) should produce different Y offsets"
        );
    }

    // ── Edge cases: tiny terminal ──────────────────────────────────

    #[test]
    fn all_effects_survive_1x1_terminal() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0);
            effect.update(0.1, 1, 1);
            let mut fb = FrameBuffer::new(1, 1);
            effect.render(&mut fb, 0.5);
            fb.swap();
            let cell = fb.get(0, 0);
            assert!(
                !cell.ch.is_control(),
                "{base:?} should not write control chars to (0,0)"
            );
        }
    }

    #[test]
    fn all_effects_survive_zero_size_terminal() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0);
            effect.update(0.1, 0, 0);
            let mut fb = FrameBuffer::new(0, 0);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(
                fb.compute_damage().len(),
                0,
                "{base:?} zero-size framebuffer should produce empty damage"
            );
        }
    }

    #[test]
    fn all_effects_on_resize_no_panic() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0);
            effect.on_resize(1, 1);
            let mut fb = FrameBuffer::new(1, 1);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(fb.width, 1, "{base:?} width should remain 1 after render");

            effect.on_resize(100, 50);
            let mut fb = FrameBuffer::new(100, 50);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(fb.width, 100, "{base:?} width should be 100 after resize");

            effect.on_resize(0, 0);
            let mut fb = FrameBuffer::new(0, 0);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(
                fb.compute_damage().len(),
                0,
                "{base:?} zero-size should produce empty damage"
            );
        }
    }

    // ── Edge cases: extreme speed/amplitude ────────────────────────

    #[test]
    fn extreme_inputs_no_panic() {
        let dna = CowDna {
            speed: 1000.0,
            amplitude: Amplitude {
                breath: 1000.0,
                sway: 100.0,
                float: 100.0,
            },
            ..CowDna::default()
        };

        let mut fb = FrameBuffer::new(40, 10);
        let effect = BreatheEffect::new(COW.to_string(), &dna, 0);
        effect.render(&mut fb, 999.0);
        assert!(
            !fb.compute_damage().is_empty(),
            "breathe at extreme time should still produce damage (non-space cells)"
        );
        fb.swap();

        let effect = FloatEffect::new(COW.to_string(), &dna, 0);
        effect.render(&mut fb, 999.0);
        assert!(
            !fb.compute_damage().is_empty(),
            "float at extreme time should still produce damage (non-space cells)"
        );
        fb.swap();
    }

    // ── Invariants: render count ───────────────────────────────────

    #[test]
    fn char_count_invariant_across_effects() {
        let dna = CowDna::default();

        // Effects that only reposition (no add/remove chars) must preserve char count
        let effects_and_times: Vec<(&str, Vec<f32>)> = vec![
            ("breathe", vec![0.0, 0.25, 0.5]),
            ("walk", vec![0.25, 0.5, 0.75]),
        ];

        for (name, times) in &effects_and_times {
            let first_count = {
                let effect = match *name {
                    "breathe" => {
                        Box::new(BreatheEffect::new(COW.to_string(), &dna, 0)) as Box<dyn Effect>
                    }
                    "walk" => Box::new(WalkEffect::new(COW.to_string(), &dna, 0)),
                    _ => unreachable!(),
                };
                let mut fb = FrameBuffer::new(80, 24);
                effect.render(&mut fb, times[0]);
                fb.swap();
                count_non_space(&fb)
            };

            for &t in &times[1..] {
                let effect = match *name {
                    "breathe" => {
                        Box::new(BreatheEffect::new(COW.to_string(), &dna, 0)) as Box<dyn Effect>
                    }
                    "walk" => Box::new(WalkEffect::new(COW.to_string(), &dna, 0)),
                    _ => unreachable!(),
                };
                let mut fb = FrameBuffer::new(80, 24);
                effect.render(&mut fb, t);
                fb.swap();
                let c = count_non_space(&fb);
                assert_eq!(
                    first_count, c,
                    "{name} char count should be constant at t={t}"
                );
            }
        }
    }

    // ── Effect::is_done returns false for non-one-shot effects ────

    #[test]
    fn non_one_shot_effects_return_is_done_false() {
        let dna = CowDna::default();
        let breathe = BreatheEffect::new(COW.to_string(), &dna, 0);
        assert!(!breathe.is_done());
        let dissolve = DissolveEffect::new(COW.to_string(), &dna, 0);
        assert!(!dissolve.is_done());
    }
}
