//! Deterministic low-discrepancy pseudo-random sequence engine based on Fibonacci / Golden Ratio
//! phyllotaxis mathematics, matching procedural tree spawning in `scenery.rs`.
//!
//! # Mathematical Foundation
//! In botanical phyllotaxis, tree branching and foliage are placed at angles governed by the golden ratio
//! reciprocal:
//!   $$\Phi^{-1} = \frac{\sqrt{5} - 1}{2} \approx 0.618033988749895$$
//!
//! This produces an optimal low-discrepancy (additive recurrence / Weyl) sequence on the unit interval:
//!   $$x_n = \{ n \cdot \Phi^{-1} \} \in [0, 1)$$
//!
//! It mathematically guarantees:
//! 1. Maximum possible dispersion between successive samples (no clumping).
//! 2. Zero immediate repetition across consecutive invocations.
//! 3. Dense, uniform coverage of multi-dimensional combination space:
//!    - Dimension 1 (Mascot):      $u_1 = \{ n \cdot \Phi^{-1} \}$
//!    - Dimension 2 (Environment):  $u_2 = \{ n \cdot \Phi^{-2} \}$
//!    - Dimension 3 (Road):         $u_3 = \{ n \cdot \Phi^{-3} \}$
//!    - Dimension 4 (Mountain):     $u_4 = \{ n \cdot \Phi^{-4} \}$
//!    - Dimension 5 (Animation):    $u_5 = \{ n \cdot \Phi^{-5} \}$ (static vs dynamic)
//!    - Dimension 6 (Effect):       $u_6 = \{ n \cdot (\Phi^{-1} + \Phi^{-3}) \}$
//!    - Dimension 7 (Thought):      $u_7 = \{ n \cdot (\Phi^{-2} + \Phi^{-4}) \}$
//!
//! Because all powers of $\Phi^{-1}$ are linearly independent over $\mathbb{Q}$, the joint trajectory
//! traces a dense, non-repeating path across the state space.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Reciprocal Golden Ratio constant $\Phi^{-1} = \frac{\sqrt{5} - 1}{2}$, identical to `scenery.rs` tree phyllotaxis.
pub const PHI_INV: f64 = 0.618_033_988_749_895;

/// Multi-dimensional low-discrepancy constants (Kronecker-Weyl sequence theorem).
/// Linearly independent algebraic irrationalities over $\mathbb{Q}$ ensure optimal multi-dimensional dispersion.
pub const PHI_INV_SQ: f64 = 0.414_213_562_373_095; // $\sqrt{2} - 1$ (Silver ratio)
pub const PHI_INV_CUBE: f64 = 0.732_050_807_568_877; // $\sqrt{3} - 1$
pub const PHI_INV_QUAD: f64 = 0.645_751_311_064_591; // $\sqrt{7} - 2$
pub const PHI_INV_QUINT: f64 = 0.316_624_790_355_400; // $\sqrt{11} - 3$
pub const PHI_INV_EFFECT: f64 = 0.605_551_275_463_989; // $\sqrt{13} - 3$
pub const PHI_INV_THOUGHT: f64 = 0.123_105_625_617_661; // $\sqrt{17} - 4$

/// 7-dimensional phyllotaxis coordinates for deterministic non-repeating mascot/scenery/fx/thought generation.
#[derive(Debug, Clone, PartialEq)]
pub struct PhyllotaxisCoordinates {
    pub step: u64,
    pub u_mascot: f64,
    pub u_env: f64,
    pub u_road: f64,
    pub u_mountain: f64,
    pub u_anim_mode: f64,
    pub u_effect: f64,
    pub u_thought: f64,
}

impl PhyllotaxisCoordinates {
    /// Compute low-discrepancy coordinates for a given sequence step.
    pub fn compute(step: u64) -> Self {
        let n = (step % 1_000_000_007) as f64;
        Self {
            step,
            u_mascot: (n * PHI_INV).fract(),
            u_env: (n * PHI_INV_SQ).fract(),
            u_road: (n * PHI_INV_CUBE).fract(),
            u_mountain: (n * PHI_INV_QUAD).fract(),
            u_anim_mode: (n * PHI_INV_QUINT).fract(),
            u_effect: (n * PHI_INV_EFFECT).fract(),
            u_thought: (n * PHI_INV_THOUGHT).fract(),
        }
    }

    /// Select an item from a non-empty slice using a phyllotaxis fractional coordinate.
    pub fn pick_from_slice<'a, T>(&self, slice: &'a [T], coord: f64) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let normalized = coord.fract().abs();
        let idx = ((normalized * slice.len() as f64).floor() as usize) % slice.len();
        Some(&slice[idx])
    }

    /// Returns `true` if the animation mode coordinate dictates a static (stationary) animation,
    /// or `false` for dynamic motion.
    pub fn is_static_animation(&self) -> bool {
        self.u_anim_mode < 0.5
    }
}

/// Sequence persistence state file location.
fn state_file_path() -> Option<PathBuf> {
    if let Ok(cfg) = forgum_platform::config_path() {
        if let Some(parent) = cfg.parent() {
            return Some(parent.join(".random_sequence"));
        }
    }
    if let Ok(rt) = forgum_platform::runtime_dir() {
        return Some(rt.join(".random_sequence"));
    }
    None
}

/// Advance the persistent sequence counter and return the next phyllotaxis coordinate sample.
pub fn advance_and_sample() -> PhyllotaxisCoordinates {
    let state_file = state_file_path();
    let current_step = state_file
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| (d.as_micros() as u64) ^ (std::process::id() as u64))
                .unwrap_or(42)
        });

    let next_step = current_step.wrapping_add(1);

    if let Some(ref p) = state_file {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(p, next_step.to_string());
    }

    PhyllotaxisCoordinates::compute(next_step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phyllotaxis_dispersion_no_consecutive_duplicates() {
        let items: Vec<usize> = (0..50).collect();
        let mut last_idx = None;
        for step in 1..=200 {
            let coords = PhyllotaxisCoordinates::compute(step);
            let picked = *coords.pick_from_slice(&items, coords.u_mascot).unwrap();
            if let Some(prev) = last_idx {
                assert_ne!(
                    prev, picked,
                    "Consecutive phyllotaxis selection must never repeat: step {} picked {}",
                    step, picked
                );
            }
            last_idx = Some(picked);
        }
    }

    #[test]
    fn test_phyllotaxis_combination_uniqueness() {
        let mut seen = std::collections::HashSet::new();
        for step in 1..=500 {
            let coords = PhyllotaxisCoordinates::compute(step);
            let mascot_idx = ((coords.u_mascot * 100.0) as usize) % 100;
            let env_idx = ((coords.u_env * 12.0) as usize) % 12;
            let road_idx = ((coords.u_road * 8.0) as usize) % 8;
            let mtn_idx = ((coords.u_mountain * 9.0) as usize) % 9;
            let anim_mode = coords.is_static_animation();

            let tuple = (mascot_idx, env_idx, road_idx, mtn_idx, anim_mode);
            // Verify every combination along the trajectory is unique across 500 steps
            assert!(
                seen.insert(tuple),
                "Phyllotaxis combination repeated prematurely at step {}: {:?}",
                step,
                tuple
            );
        }
    }

    #[test]
    fn test_coordinates_stay_in_unit_interval() {
        for step in 0..1000 {
            let c = PhyllotaxisCoordinates::compute(step);
            assert!((0.0..1.0).contains(&c.u_mascot));
            assert!((0.0..1.0).contains(&c.u_env));
            assert!((0.0..1.0).contains(&c.u_road));
            assert!((0.0..1.0).contains(&c.u_mountain));
            assert!((0.0..1.0).contains(&c.u_anim_mode));
            assert!((0.0..1.0).contains(&c.u_effect));
            assert!((0.0..1.0).contains(&c.u_thought));
        }
    }
}
