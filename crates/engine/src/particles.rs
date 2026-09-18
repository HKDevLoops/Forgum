//! Particle system — 6 themed particle types with pool allocation.
//!
//! Particles are spawned by effects, updated each frame, and rendered
//! into the framebuffer. Phase 1.7: uses `slotmap::SlotMap` for O(1)
//! spawn/kill with ABA-safe generational keys (replaces O(n) linear scan).

use std::cell::Cell;

use crate::color::{hsv_to_rgb, lerp_palette};
use crate::dna::ParticleType;
use crate::framebuffer::{Cell as FbCell, Color, FrameBuffer};

/// Maximum particles in the pool.
const MAX_PARTICLES: usize = 512;

slotmap::new_key_type! {
    /// Generational key for a particle in the pool.
    pub struct ParticleKey;
}

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
        }
    }
}

/// Slotmap-backed particle pool. O(1) spawn/kill, ABA-safe.
#[derive(Debug)]
pub struct ParticlePool {
    particles: slotmap::SlotMap<ParticleKey, Particle>,
    /// Reusable buffer for dead particle keys — avoids per-frame Vec allocation.
    dead_keys: Vec<ParticleKey>,
}

impl ParticlePool {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            particles: slotmap::SlotMap::with_capacity_and_key(MAX_PARTICLES),
            dead_keys: Vec::with_capacity(MAX_PARTICLES),
        }
    }

    /// Spawn a particle. Returns the key if pool has capacity, None if full.
    /// O(1) — slotmap reuses dead slots instantly.
    pub fn spawn(&mut self, p: Particle) -> Option<ParticleKey> {
        if self.particles.len() >= MAX_PARTICLES {
            return None;
        }
        Some(self.particles.insert(p))
    }

    /// Kill a particle by key. O(1).
    pub fn kill(&mut self, key: ParticleKey) {
        self.particles.remove(key);
    }

    /// Update all active particles by `dt` seconds. Dead particles are
    /// removed from the slotmap automatically.
    pub fn update(&mut self, dt: f32) {
        self.dead_keys.clear();
        for (key, p) in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
            if p.life <= 0.0 {
                self.dead_keys.push(key);
            }
        }
        for key in &self.dead_keys {
            self.particles.remove(*key);
        }
    }

    /// Render active particles into the framebuffer.
    pub fn render(&self, fb: &mut FrameBuffer, time: f32, alpha_fn: fn(f32) -> f32) {
        self.render_with_cutoff(fb, time, alpha_fn, 0);
    }

    /// Render active particles into the framebuffer with a vertical exclusion cutoff.
    /// Particles located at `yi < min_y` are occluded and not drawn (strictly protects speech bubbles).
    pub fn render_with_cutoff(
        &self,
        fb: &mut FrameBuffer,
        _time: f32,
        alpha_fn: fn(f32) -> f32,
        min_y: usize,
    ) {
        for (_key, p) in self.particles.iter() {
            let xi = p.x as i32;
            let yi = p.y as i32;
            if xi < 0 || yi < 0 {
                continue;
            }
            let xi = xi as usize;
            let yi = yi as usize;
            if xi >= fb.width || yi >= fb.height || yi < min_y {
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
        self.particles.len()
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        self.particles.clear();
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
        life: 0.3,
        max_life: 0.3,
        ch: glyph,
        color: Color::rgb(0, 255, 0),
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
        ParticleType::Glitch => spawn_glitch(pool, x, y, width, height),
        ParticleType::Pulse => { /* No particles for pulse — it colors text */ }
    }
}

// ── Internal PRNG ──────────────────────────────────────────────────

thread_local! {
    static FRAME_SEED: Cell<u32> = const { Cell::new(0) };
}

fn xorshift32(state: &mut u32) {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
}

/// Get a pseudo-random f32 in [0, 1).
pub fn rand_01() -> f32 {
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
        let key = pool.spawn(Particle {
            x: 5.0,
            y: 3.0,
            vx: 1.0,
            vy: -2.0,
            life: 1.0,
            max_life: 1.0,
            ch: '*',
            color: Color::WHITE,
        });
        assert!(key.is_some());
        assert_eq!(pool.active_count(), 1);
        pool.update(0.5);
        assert_eq!(pool.active_count(), 1);
        pool.update(0.6);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn pool_full_returns_none() {
        let mut pool = ParticlePool::new();
        for _ in 0..MAX_PARTICLES {
            let _ = pool.spawn(Particle::default());
        }
        assert!(pool.spawn(Particle::default()).is_none());
    }

    #[test]
    fn pool_clear() {
        let mut pool = ParticlePool::new();
        for _ in 0..10 {
            let _ = pool.spawn(Particle::default());
        }
        pool.clear();
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn spawn_fire_adds_particles() {
        let mut pool = ParticlePool::new();
        let empty: &[(u8, u8, u8)] = &[];
        spawn_fire(&mut pool, 10.0, 5.0, empty, 0.0);
        assert_eq!(pool.active_count(), 3);

        for (_key, p) in pool.particles.iter() {
            assert!(
                ['*', '^', '.', '~'].contains(&p.ch),
                "unexpected char: {:?}",
                p.ch
            );
            assert!(p.life > 0.0);
            assert!(p.color.r > 0);
        }
    }

    #[test]
    fn spawn_bubbles_adds_particles() {
        let mut pool = ParticlePool::new();
        let empty: &[(u8, u8, u8)] = &[];
        spawn_bubbles(&mut pool, 10.0, 5.0, empty, 0.0);
        assert_eq!(pool.active_count(), 1);

        let (_key, p) = pool.particles.iter().next().unwrap();
        assert!(['o', 'O', '°', '.'].contains(&p.ch));
        assert!(p.life > 0.0);
    }

    #[test]
    fn spawn_stars_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_stars(&mut pool, 10.0, 5.0, 0.0);
        assert_eq!(pool.active_count(), 1);

        let (_key, p) = pool.particles.iter().next().unwrap();
        assert!(['*', '+', '✦', '✧'].contains(&p.ch));
        assert!(p.life > 0.0);
    }

    #[test]
    fn spawn_zzz_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_zzz(&mut pool, 10.0, 5.0, 0.0);
        assert_eq!(pool.active_count(), 1);

        let (_key, p) = pool.particles.iter().next().unwrap();
        assert!(['Z', 'z'].contains(&p.ch));
        assert!(p.vy < 0.0);
    }

    #[test]
    fn spawn_glitch_adds_particles() {
        let mut pool = ParticlePool::new();
        spawn_glitch(&mut pool, 10.0, 5.0, 80, 24);
        assert_eq!(pool.active_count(), 1);

        let (_key, p) = pool.particles.iter().next().unwrap();
        assert!(['0', '1', '#', '@', '█', '▓', '░'].contains(&p.ch));
    }

    #[test]
    fn seed_frame_rng_is_deterministic() {
        seed_frame_rng(42);
        let a = rand_01();
        seed_frame_rng(42);
        let b = rand_01();
        assert_eq!(a, b);
    }

    #[test]
    fn update_moves_particles_by_velocity() {
        let mut pool = ParticlePool::new();
        let _ = pool.spawn(Particle {
            x: 0.0,
            y: 0.0,
            vx: 10.0,
            vy: 0.0,
            life: 2.0,
            max_life: 2.0,
            ch: '*',
            color: Color::WHITE,
        });
        pool.update(0.5);
        let (_key, p) = pool.particles.iter().next().unwrap();
        assert!((p.x - 5.0).abs() < 0.01);
    }

    #[test]
    fn update_kills_expired_particles() {
        let mut pool = ParticlePool::new();
        let _ = pool.spawn(Particle {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.1,
            max_life: 0.1,
            ch: 'X',
            color: Color::WHITE,
        });
        assert_eq!(pool.active_count(), 1);
        pool.update(0.2);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn render_writes_to_framebuffer() {
        let mut pool = ParticlePool::new();
        let _ = pool.spawn(Particle {
            x: 5.0,
            y: 3.0,
            vx: 0.0,
            vy: 0.0,
            life: 1.0,
            max_life: 1.0,
            ch: 'Z',
            color: Color::WHITE,
        });
        let mut fb = FrameBuffer::new(80, 24);
        pool.render(&mut fb, 0.0, |v| v);
        fb.swap();
        let cell = fb.get(5, 3);
        assert_eq!(cell.ch, 'Z');
    }

    #[test]
    fn pool_render_with_cutoff_occludes_speech_bubbles() {
        let mut pool = ParticlePool::new();
        // Particle 1 in bubble zone (y = 2)
        pool.spawn(Particle {
            x: 10.0,
            y: 2.0,
            vx: 0.0,
            vy: 0.0,
            life: 1.0,
            max_life: 1.0,
            ch: '*',
            color: Color::WHITE,
        });
        // Particle 2 in creature zone (y = 6)
        pool.spawn(Particle {
            x: 10.0,
            y: 6.0,
            vx: 0.0,
            vy: 0.0,
            life: 1.0,
            max_life: 1.0,
            ch: '#',
            color: Color::WHITE,
        });

        let mut fb = FrameBuffer::new(80, 24);
        // Bubble cutoff at y = 4 (bubble occupies rows 0..4)
        pool.render_with_cutoff(&mut fb, 0.0, |v| v, 4);
        fb.swap();

        // Bubble zone particle at y=2 must NOT be rendered
        assert_eq!(fb.get(10, 2).ch, ' ');
        // Creature zone particle at y=6 MUST be rendered
        assert_eq!(fb.get(10, 6).ch, '#');
    }

    #[test]
    fn spawn_pulse_type_adds_no_particles() {
        let mut pool = ParticlePool::new();
        let empty: &[(u8, u8, u8)] = &[];
        spawn_for_type(
            &mut pool,
            ParticleType::Pulse,
            10.0,
            5.0,
            empty,
            0.0,
            80,
            24,
        );
        assert_eq!(
            pool.active_count(),
            0,
            "Pulse type should not spawn particles"
        );
    }

    #[test]
    fn clear_deactivates_all_particles() {
        let mut pool = ParticlePool::new();
        for _ in 0..5 {
            let _ = pool.spawn(Particle {
                life: 10.0,
                ..Default::default()
            });
        }
        assert_eq!(pool.active_count(), 5);
        pool.clear();
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn kill_removes_particle_by_key() {
        let mut pool = ParticlePool::new();
        let key = pool.spawn(Particle::default()).unwrap();
        assert_eq!(pool.active_count(), 1);
        pool.kill(key);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn spawn_returns_distinct_keys() {
        let mut pool = ParticlePool::new();
        let k1 = pool.spawn(Particle::default()).unwrap();
        let k2 = pool.spawn(Particle::default()).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn seed_frame_rng_deterministic_sequence() {
        seed_frame_rng(42);
        let seq_a: Vec<f32> = (0..5).map(|_| rand_01()).collect();

        seed_frame_rng(42);
        let seq_b: Vec<f32> = (0..5).map(|_| rand_01()).collect();

        assert_eq!(
            seq_a, seq_b,
            "same seed must produce identical PRNG sequence"
        );
    }

    #[test]
    fn particle_velocity_zero_stays_put() {
        let mut pool = ParticlePool::new();
        let _ = pool.spawn(Particle {
            x: 10.0,
            y: 5.0,
            vx: 0.0,
            vy: 0.0,
            life: 10.0,
            max_life: 10.0,
            ch: '*',
            color: Color::WHITE,
        });
        pool.update(1.0);
        let (_key, p) = pool.particles.iter().next().unwrap();
        assert!(
            (p.y - 5.0).abs() < 0.01,
            "particle with zero vy should stay at spawn Y, got y={}",
            p.y
        );
    }

    #[test]
    fn dead_keys_reused_across_ticks() {
        let mut pool = ParticlePool::new();
        let mut keys = Vec::new();

        // Fill the pool.
        for _ in 0..10 {
            if let Some(k) = pool.spawn(Particle {
                x: 0.0,
                y: 0.0,
                vx: 1.0,
                vy: 0.0,
                life: 0.1,
                max_life: 0.1,
                ch: '*',
                color: Color::WHITE,
            }) {
                keys.push(k);
            }
        }

        // Kill some particles manually.
        pool.kill(keys[0]);
        pool.kill(keys[1]);
        pool.kill(keys[2]);

        // Update to process dead keys.
        pool.update(0.01);

        // Re-spawn — should reuse the freed slots.
        for _ in 0..3 {
            let k = pool.spawn(Particle {
                x: 5.0,
                y: 5.0,
                vx: 0.0,
                vy: 0.0,
                life: 100.0,
                max_life: 100.0,
                ch: '+',
                color: Color::WHITE,
            });
            assert!(k.is_some(), "reused dead_keys should allow re-spawning");
        }
    }

    #[test]
    fn pool_stays_within_capacity() {
        let mut pool = ParticlePool::new();
        let mut count = 0;
        for _ in 0..MAX_PARTICLES + 10 {
            if pool
                .spawn(Particle {
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    life: 100.0,
                    max_life: 100.0,
                    ch: 'x',
                    color: Color::WHITE,
                })
                .is_some()
            {
                count += 1;
            }
        }
        assert_eq!(count, MAX_PARTICLES);
    }
}
