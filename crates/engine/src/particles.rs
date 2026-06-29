//! Particle system — 6 themed particle types with pool allocation.
//!
//! Particles are spawned by effects, updated each frame, and rendered
//! into the framebuffer. The pool uses a fixed-size array (no heap alloc
//! per frame) for the zero-alloc invariant.

use std::cell::Cell;

use crate::color::{hsv_to_rgb, lerp_palette};
use crate::dna::ParticleType;
use crate::framebuffer::{Cell as FbCell, Color, FrameBuffer};

/// Maximum particles in the pool.
const MAX_PARTICLES: usize = 512;

/// A single particle.
#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub ch: char,
    pub color: Color,
    pub active: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.0,
            max_life: 1.0,
            ch: ' ',
            color: Color::WHITE,
            active: false,
        }
    }
}

/// Fixed-size particle pool.
#[derive(Debug)]
pub struct ParticlePool {
    particles: [Particle; MAX_PARTICLES],
    count: usize,
}

impl ParticlePool {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            particles: std::array::from_fn(|_| Particle::default()),
            count: 0,
        }
    }

    /// Spawn a particle. Returns false if pool is full.
    pub fn spawn(&mut self, p: Particle) -> bool {
        for i in 0..MAX_PARTICLES {
            if !self.particles[i].active {
                self.particles[i] = p;
                self.count += 1;
                return true;
            }
        }
        false
    }

    /// Update all active particles by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        self.count = 0;
        for p in &mut self.particles {
            if !p.active {
                continue;
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
            if p.life <= 0.0 {
                p.active = false;
            } else {
                self.count += 1;
            }
        }
    }

    /// Render active particles into the framebuffer.
    pub fn render(&self, fb: &mut FrameBuffer, _time: f32, alpha_fn: fn(f32) -> f32) {
        for p in &self.particles {
            if !p.active {
                continue;
            }
            let xi = p.x as i32;
            let yi = p.y as i32;
            if xi < 0 || yi < 0 {
                continue;
            }
            let xi = xi as usize;
            let yi = yi as usize;
            if xi >= fb.width || yi >= fb.height {
                continue;
            }
            let life_ratio = (p.life / p.max_life).clamp(0.0, 1.0);
            let alpha = (alpha_fn(1.0 - life_ratio) * 255.0) as u8;
            let mut c = p.color;
            c.a = alpha;
            let _ = fb.set(
                xi,
                yi,
                FbCell {
                    ch: p.ch,
                    fg: c,
                    bg: Color::TRANSPARENT,
                    alpha,
                },
            );
        }
    }

    /// Number of active particles.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.count
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        for p in &mut self.particles {
            p.active = false;
        }
        self.count = 0;
    }
}

// ── Particle spawners per type ─────────────────────────────────────

/// Spawn fire particles (dragons, daemons).
pub fn spawn_fire(pool: &mut ParticlePool, x: f32, y: f32, palette: &[(u8, u8, u8)], time: f32) {
    let glyphs = ['*', '^', '.', '~', '*'];
    for _ in 0..3 {
        let glyph = glyphs[(time * 10.0) as usize % glyphs.len()];
        let (r, g, b) = if palette.len() >= 2 {
            lerp_palette(palette, rand_01())
        } else {
            (255, 128, 0)
        };
        let _ = pool.spawn(Particle {
            x: x + rand_range(-1.0, 1.0),
            y,
            vx: rand_range(-2.0, 2.0),
            vy: rand_range(-15.0, -5.0),
            life: rand_range(0.6, 1.4),
            max_life: 1.4,
            ch: glyph,
            color: Color::rgb(r, g, b),
            active: true,
        });
    }
}

/// Spawn bubble particles (dolphins, whales).
pub fn spawn_bubbles(pool: &mut ParticlePool, x: f32, y: f32, palette: &[(u8, u8, u8)], time: f32) {
    let glyphs = ['o', 'O', '°', '.'];
    let glyph = glyphs[(time * 3.0) as usize % glyphs.len()];
    let (r, g, b) = if palette.len() >= 2 {
        lerp_palette(palette, rand_01())
    } else {
        (100, 200, 255)
    };
    let _ = pool.spawn(Particle {
        x: x + rand_range(-0.5, 0.5),
        y,
        vx: (time * 2.0).sin() * 1.5,
        vy: rand_range(-3.0, -1.0),
        life: rand_range(1.5, 3.0),
        max_life: 3.0,
        ch: glyph,
        color: Color::rgb(r, g, b),
        active: true,
    });
}

/// Spawn star particles (nyan, wizard).
pub fn spawn_stars(pool: &mut ParticlePool, x: f32, y: f32, time: f32) {
    let glyphs = ['*', '+', '✦', '✧'];
    let glyph = glyphs[(time * 8.0) as usize % glyphs.len()];
    let hue = ((x * 0.01 + time * 2.0) % 1.0 + 1.0) % 1.0;
    let (r, g, b) = hsv_to_rgb(hue * 360.0, 0.9, 1.0);
    let _ = pool.spawn(Particle {
        x: x + rand_range(-1.0, 1.0),
        y: y + rand_range(-0.5, 0.5),
        vx: rand_range(-3.0, 3.0),
        vy: rand_range(-8.0, -3.0),
        life: rand_range(0.4, 0.8),
        max_life: 0.8,
        ch: glyph,
        color: Color::rgb(r, g, b),
        active: true,
    });
}

/// Spawn zzz particles (sleeping animals).
pub fn spawn_zzz(pool: &mut ParticlePool, x: f32, y: f32, time: f32) {
    let glyphs = ['Z', 'z', 'z'];
    let glyph = glyphs[(time * 2.0) as usize % glyphs.len()];
    let _ = pool.spawn(Particle {
        x: x + rand_range(-0.5, 0.5),
        y,
        vx: (time * 1.5).sin() * 2.0,
        vy: rand_range(-2.0, -1.0),
        life: rand_range(2.0, 4.0),
        max_life: 4.0,
        ch: glyph,
        color: Color::rgb(180, 160, 220),
        active: true,
    });
}

/// Spawn glitch particles (skeletons, doge).
pub fn spawn_glitch(pool: &mut ParticlePool, _x: f32, _y: f32, width: usize, height: usize) {
    let glyphs = ['0', '1', '#', '@', '█', '▓', '░'];
    let glyph = glyphs[(rand_01() * glyphs.len() as f32) as usize % glyphs.len()];
    let rx = rand_01() * width as f32;
    let ry = rand_01() * height as f32;
    let _ = pool.spawn(Particle {
        x: rx,
        y: ry,
        vx: 0.0,
        vy: 0.0,
        life: rand_range(0.1, 0.3),
        max_life: 0.3,
        ch: glyph,
        color: if rand_01() > 0.5 {
            Color::rgb(0, 255, 0)
        } else {
            Color::rgb(255, 0, 0)
        },
        active: true,
    });
}

/// Spawn particles based on type.
#[allow(clippy::too_many_arguments)]
pub fn spawn_for_type(
    pool: &mut ParticlePool,
    ptype: ParticleType,
    x: f32,
    y: f32,
    palette: &[(u8, u8, u8)],
    time: f32,
    width: usize,
    height: usize,
) {
    match ptype {
        ParticleType::Fire => spawn_fire(pool, x, y, palette, time),
        ParticleType::Bubbles => spawn_bubbles(pool, x, y, palette, time),
        ParticleType::Stars => spawn_stars(pool, x, y, time),
        ParticleType::Zzz => spawn_zzz(pool, x, y, time),
        ParticleType::Pulse => {} // Pulse is color-only, no particles
        ParticleType::Glitch => spawn_glitch(pool, x, y, width, height),
    }
}

// ── Simple PRNG (cell-based, no unsafe) ────────────────────────────

/// Simple xorshift32 — not cryptographic, but fast.
fn xorshift32(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

// Thread-local frame seed (no unsafe needed).
thread_local! {
    static FRAME_SEED: Cell<u32> = const { Cell::new(0x1234_5678) };
}

/// Get a pseudo-random f32 in [0.0, 1.0).
fn rand_01() -> f32 {
    FRAME_SEED.with(|cell| {
        let mut s = cell.get();
        xorshift32(&mut s);
        cell.set(s);
        (s >> 8) as f32 / (1 << 24) as f32
    })
}

/// Get a pseudo-random f32 in [lo, hi).
fn rand_range(lo: f32, hi: f32) -> f32 {
    lo + rand_01() * (hi - lo)
}

/// Seed the frame PRNG. Call once per frame.
pub fn seed_frame_rng(seed: u32) {
    FRAME_SEED.with(|cell| cell.set(seed));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_spawn_and_update() {
        let mut pool = ParticlePool::new();
        assert!(pool.spawn(Particle {
            x: 5.0,
            y: 3.0,
            vx: 1.0,
            vy: -2.0,
            life: 1.0,
            max_life: 1.0,
            ch: '*',
            color: Color::WHITE,
            active: true,
        }));
        assert_eq!(pool.active_count(), 1);
        pool.update(0.5);
        assert_eq!(pool.active_count(), 1);
        pool.update(0.6);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn pool_full_returns_false() {
        let mut pool = ParticlePool::new();
        for _ in 0..MAX_PARTICLES {
            let _ = pool.spawn(Particle {
                active: true,
                ..Default::default()
            });
        }
        assert!(!pool.spawn(Particle {
            active: true,
            ..Default::default()
        }));
    }

    #[test]
    fn pool_clear() {
        let mut pool = ParticlePool::new();
        for _ in 0..10 {
            let _ = pool.spawn(Particle {
                active: true,
                ..Default::default()
            });
        }
        pool.clear();
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn spawn_fire_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_fire(&mut pool, 10.0, 5.0, &[], 0.0);
        assert!(pool.active_count() > 0);
    }

    #[test]
    fn spawn_bubbles_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_bubbles(&mut pool, 10.0, 5.0, &[], 0.0);
        assert!(pool.active_count() > 0);
    }

    #[test]
    fn spawn_stars_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_stars(&mut pool, 10.0, 5.0, 0.0);
        assert!(pool.active_count() > 0);
    }

    #[test]
    fn spawn_zzz_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_zzz(&mut pool, 10.0, 5.0, 0.0);
        assert!(pool.active_count() > 0);
    }

    #[test]
    fn spawn_glitch_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_glitch(&mut pool, 10.0, 5.0, 80, 24);
        assert!(pool.active_count() > 0);
    }

    #[test]
    fn seed_frame_rng_is_deterministic() {
        seed_frame_rng(42);
        let a = rand_01();
        seed_frame_rng(42);
        let b = rand_01();
        assert_eq!(a, b);
    }
}
