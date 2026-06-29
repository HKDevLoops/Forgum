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
#[allow(dead_code)]
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
        let _t = (time * self.speed + self.phase) % 1.0;
        let _eased = (self.easing_fn)(_t);
        render_text(fb, &self.cow_text, Color::WHITE);
    }
}

// ── Float / Bob ────────────────────────────────────────────────────

/// Whole-art vertical/horizontal drift.
#[derive(Debug)]
#[allow(dead_code)]
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
        let _t = (time * self.speed + self.phase) % 1.0;
        let _intensity = (self.easing_fn)(_t);
        // TODO: apply _intensity to vertical shift via self.amp.float
        render_text(fb, &self.cow_text, Color::WHITE);
    }
}

// ── Walk / Trot ────────────────────────────────────────────────────

/// Bottom-row leg character swap.
#[derive(Debug)]
#[allow(dead_code)]
pub struct WalkEffect {
    cow_text: String,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl WalkEffect {
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

impl Effect for WalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let _t = (time * self.speed + self.phase) % 1.0;
        let _eased = (self.easing_fn)(_t);
        render_text(fb, &self.cow_text, Color::WHITE);
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
#[allow(dead_code)]
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
        let _t = (time * self.speed + self.phase) % 1.0;
        let _eased = (self.easing_fn)(_t);
        render_text(fb, &self.cow_text, Color::WHITE);
    }
}

// ── Talk / Chew ────────────────────────────────────────────────────

/// Mouth + eye region animation.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TalkEffect {
    cow_text: String,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
}

impl TalkEffect {
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

impl Effect for TalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let _t = (time * self.speed + self.phase) % 1.0;
        let _eased = (self.easing_fn)(_t);
        render_text(fb, &self.cow_text, Color::WHITE);
    }
}

// ── Sway / Pendulum ────────────────────────────────────────────────

/// Top-half skew, bottom anchored.
#[derive(Debug)]
#[allow(dead_code)]
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
        let _t = (time * self.speed + self.phase) % 1.0;
        let _eased = (self.easing_fn)(_t);
        render_text(fb, &self.cow_text, Color::WHITE);
    }
}

// ── Dissolve ───────────────────────────────────────────────────────

/// Break art into falling chars, reassemble.
#[derive(Debug)]
#[allow(dead_code)]
pub struct DissolveEffect {
    cow_text: String,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    done: bool,
}

impl DissolveEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            done: false,
        }
    }
}

impl Effect for DissolveEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let _t = ((time * self.speed + self.phase) % 1.0).min(1.0);
        let _eased = (self.easing_fn)(_t);
        // TODO: apply dissolve effect (scatter chars when _t < 0.5)
        render_text(fb, &self.cow_text, Color::WHITE);
    }

    fn is_done(&self) -> bool {
        self.done
    }
}

// ── Shared helpers ─────────────────────────────────────────────────

/// Render text into the framebuffer at row 0.
fn render_text(fb: &mut FrameBuffer, text: &str, fg: Color) {
    let mut x = 0usize;
    let mut y = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            x = 0;
            y = y.saturating_add(1);
            continue;
        }
        if y >= fb.height {
            break;
        }
        if x < fb.width {
            let _ = fb.set(x, y, Cell::new(ch, fg));
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

    #[test]
    fn static_cow_does_not_overflow() {
        let mut fb = FrameBuffer::new(40, 10);
        render_static_cow(&mut fb, default_cow_text());
        fb.swap();
        assert_eq!(fb.get(8, 0).ch, '\\');
    }

    #[test]
    fn empty_text_is_safe() {
        let mut fb = FrameBuffer::new(10, 10);
        render_static_cow(&mut fb, "");
        assert!(fb.compute_damage().is_empty());
    }

    #[test]
    fn create_breathe_effect() {
        let dna = CowDna::default();
        let effect = create_effect(BaseAnim::Breathe, "test".to_string(), dna, 0);
        assert!(!effect.is_done());
    }

    #[test]
    fn create_particles_effect() {
        let dna = CowDna::default();
        let mut effect = create_effect(BaseAnim::Particles, "test".to_string(), dna, 0);
        effect.update(0.1, 80, 24);
        assert!(!effect.is_done());
    }

    #[test]
    fn create_glitch_effect() {
        let dna = CowDna::default();
        let mut effect = create_effect(BaseAnim::Glitch, "test".to_string(), dna, 0);
        effect.update(0.1, 80, 24);
        assert!(!effect.is_done());
    }

    #[test]
    fn all_base_types_create_effects() {
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
            let effect = create_effect(*base, "test".to_string(), dna.clone(), 0);
            assert!(!effect.is_done());
        }
    }
}
