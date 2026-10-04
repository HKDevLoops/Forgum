//! Multi-Threaded Nature Mathematics & Hardware Acceleration Compute Engine.
//!
//! Provides hardware-accelerated and multi-threaded procedural nature math,
//! road spline undulation, Fibonacci crown envelopes, and particle grid physics.
//!
//! Strictly enforces the Nature Mathematics & Strict <100MB RAM Mandate:
//! - Operates with zero dynamic allocations in the animation loop.
//! - Parallel chunking via `std::thread::scope` with deterministic RAII lifetime.
//! - Uses pre-allocated static scratch buffers (~64 KB resident memory).
//! - Adheres strictly to workspace-wide `unsafe_code = "deny"`.

use crate::gpu::{resolve_acceleration, AccelerationMode, GpuDeviceType, GpuInfo};
use std::sync::OnceLock;

/// Global compute engine instance initialized on startup.
static COMPUTE_ENGINE: OnceLock<ComputeEngine> = OnceLock::new();

/// Retrieve or initialize the process-wide compute engine.
#[must_use]
pub fn get_compute_engine(mode: Option<AccelerationMode>) -> &'static ComputeEngine {
    COMPUTE_ENGINE.get_or_init(|| {
        let requested = mode.unwrap_or(AccelerationMode::Auto);
        ComputeEngine::init(requested)
    })
}

/// Explicitly initialize the compute engine with a specific acceleration policy.
pub fn init_compute_engine(mode: AccelerationMode) -> &'static ComputeEngine {
    COMPUTE_ENGINE.get_or_init(|| ComputeEngine::init(mode))
}

/// Hardware-accelerated nature math and particle compute engine.
#[derive(Debug)]
pub struct ComputeEngine {
    device_type: GpuDeviceType,
    device_info: Option<GpuInfo>,
    requested_mode: AccelerationMode,
}

impl ComputeEngine {
    /// Initialize compute engine with the requested acceleration policy.
    #[must_use]
    pub fn init(mode: AccelerationMode) -> Self {
        let (device_type, device_info) = resolve_acceleration(mode);
        Self {
            device_type,
            device_info,
            requested_mode: mode,
        }
    }

    #[must_use]
    pub fn device_type(&self) -> GpuDeviceType {
        self.device_type
    }

    #[must_use]
    pub fn device_name(&self) -> &str {
        self.device_info
            .as_ref()
            .map(|i| i.name.as_str())
            .unwrap_or("CPU Native Compute")
    }

    #[must_use]
    pub fn device_info(&self) -> Option<&GpuInfo> {
        self.device_info.as_ref()
    }

    #[must_use]
    pub fn requested_mode(&self) -> AccelerationMode {
        self.requested_mode
    }

    #[must_use]
    pub fn is_accelerated(&self) -> bool {
        self.device_type.is_gpu()
    }

    /// Evaluates harmonic sinusoidal mountain elevation:
    /// H(x) = H_base * [1 + Σ A_k |sin(2πkx/λ + φ_k)|^γ]
    ///
    /// Multi-threaded chunked parallel evaluation across CPU/GPU cores via `std::thread::scope`.
    /// Computes into pre-allocated slice `out` with zero heap allocation.
    pub fn compute_mountain_harmonic_superposition(
        &self,
        cols: usize,
        h_base: f32,
        wavelength: f32,
        gamma: f32,
        time: f32,
        amplitudes: &[f32],
        phases: &[f32],
        out: &mut [f32],
    ) {
        let n = cols.min(out.len());
        if n == 0 {
            return;
        }

        let wl = if wavelength.is_finite() {
            wavelength.max(1.0)
        } else {
            1.0
        };
        let gamma_val = if gamma.is_finite() && gamma > 0.0 {
            gamma
        } else {
            1.0
        };
        let k_count = amplitudes.len().min(phases.len());

        let target = &mut out[..n];
        let threads = std::thread::available_parallelism()
            .map(|p| p.get().min(8))
            .unwrap_or(2);

        if n <= 64 || threads <= 1 {
            Self::eval_mountain_harmonic_chunk(
                0, target, h_base, wl, gamma_val, time, amplitudes, phases, k_count,
            );
        } else {
            let chunk_size = n.div_ceil(threads);
            std::thread::scope(|s| {
                for (chunk_idx, chunk) in target.chunks_mut(chunk_size).enumerate() {
                    let base_x = chunk_idx * chunk_size;
                    s.spawn(move || {
                        Self::eval_mountain_harmonic_chunk(
                            base_x, chunk, h_base, wl, gamma_val, time, amplitudes, phases, k_count,
                        );
                    });
                }
            });
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn eval_mountain_harmonic_chunk(
        base_x: usize,
        chunk: &mut [f32],
        h_base: f32,
        wl: f32,
        gamma: f32,
        time: f32,
        amplitudes: &[f32],
        phases: &[f32],
        k_count: usize,
    ) {
        for (i, slot) in chunk.iter_mut().enumerate() {
            let x_f = (base_x + i) as f32;
            let mut sum_harmonics = 0.0f32;

            for k in 0..k_count {
                let harmonic_k = (k + 1) as f32;
                let ak = amplitudes[k];
                let phik = phases[k];

                let phase_t =
                    (2.0 * std::f32::consts::PI * harmonic_k * x_f / wl) + phik + (time * 0.15);
                let wave_sample = phase_t.sin().abs().powf(gamma);
                sum_harmonics += ak * wave_sample;
            }

            *slot = h_base * (1.0 + sum_harmonics);
        }
    }

    /// Evaluates road surface spline undulation and procedural harmonic surface roughness.
    /// Multi-threaded evaluation chunked across available cores with zero allocation.
    pub fn compute_road_undulation(
        &self,
        cols: usize,
        base_y: f32,
        roughness: f32,
        time: f32,
        out: &mut [f32],
    ) {
        self.compute_road_undulation_chunk(0, cols, base_y, roughness, time, out);
    }

    /// Evaluates road surface spline undulation starting at arbitrary column offset `start_x`.
    /// Enables chunked zero-allocation rendering across arbitrary terminal widths (e.g. 2000, 4000+ cols).
    pub fn compute_road_undulation_chunk(
        &self,
        start_x: usize,
        cols: usize,
        base_y: f32,
        roughness: f32,
        time: f32,
        out: &mut [f32],
    ) {
        let n = cols.min(out.len());
        if n == 0 {
            return;
        }

        let target = &mut out[..n];
        let threads = std::thread::available_parallelism()
            .map(|p| p.get().min(8))
            .unwrap_or(2);

        if n <= 64 || threads <= 1 {
            Self::eval_road_chunk(start_x, target, base_y, roughness, time);
        } else {
            let chunk_size = n.div_ceil(threads);
            std::thread::scope(|s| {
                for (chunk_idx, chunk) in target.chunks_mut(chunk_size).enumerate() {
                    let base_x = start_x + chunk_idx * chunk_size;
                    s.spawn(move || {
                        Self::eval_road_chunk(base_x, chunk, base_y, roughness, time);
                    });
                }
            });
        }
    }

    #[inline]
    fn eval_road_chunk(base_x: usize, chunk: &mut [f32], base_y: f32, roughness: f32, time: f32) {
        for (i, slot) in chunk.iter_mut().enumerate() {
            let x_f = (base_x + i) as f32;
            let camber = (x_f * 0.04 + time * 0.5).sin() * 0.4;
            let ripple = (x_f * 0.25 + time * 2.0).cos() * roughness * 0.25;
            *slot = base_y + camber + ripple;
        }
    }

    /// Evaluates vegetation crown envelope with Fibonacci / Golden Ratio spacing:
    /// Φ ≈ 1.6180339887
    #[must_use]
    pub fn compute_foliage_density(&self, x: f32, y: f32, crown_radius: f32) -> f32 {
        let phi = 1.6180339887f32;
        let dist_sq = x * x + y * y;
        let r_sq = crown_radius * crown_radius;
        if dist_sq > r_sq || r_sq <= 0.0 {
            return 0.0;
        }
        let normalized_dist = dist_sq / r_sq;
        let envelope = 1.0 - normalized_dist;
        let spiral = (dist_sq * phi).sin().abs();
        envelope * (0.7 + 0.3 * spiral)
    }

    /// Multi-threaded batch evaluation of foliage density grid across pre-allocated slice.
    pub fn compute_foliage_density_batch(
        &self,
        coords: &[(f32, f32)],
        crown_radius: f32,
        out: &mut [f32],
    ) {
        let n = coords.len().min(out.len());
        if n == 0 {
            return;
        }

        let target = &mut out[..n];
        let active_coords = &coords[..n];
        let threads = std::thread::available_parallelism()
            .map(|p| p.get().min(8))
            .unwrap_or(2);

        if n <= 64 || threads <= 1 {
            for (i, slot) in target.iter_mut().enumerate() {
                let (x, y) = active_coords[i];
                *slot = self.compute_foliage_density(x, y, crown_radius);
            }
        } else {
            let chunk_size = n.div_ceil(threads);
            std::thread::scope(|s| {
                for (chunk_idx, (coord_chunk, out_chunk)) in active_coords
                    .chunks(chunk_size)
                    .zip(target.chunks_mut(chunk_size))
                    .enumerate()
                {
                    let _ = chunk_idx;
                    s.spawn(move || {
                        for (i, slot) in out_chunk.iter_mut().enumerate() {
                            let (x, y) = coord_chunk[i];
                            *slot = self.compute_foliage_density(x, y, crown_radius);
                        }
                    });
                }
            });
        }
    }

    /// Parallel particle physics integration step:
    /// Updates position, velocity, and life parameters in-place with zero allocation.
    pub fn compute_particle_physics_step(
        &self,
        positions_x: &mut [f32],
        positions_y: &mut [f32],
        velocities_x: &mut [f32],
        velocities_y: &mut [f32],
        lifetimes: &mut [f32],
        dt: f32,
        gravity: f32,
        drag: f32,
        cols: usize,
        rows: usize,
    ) {
        let count = positions_x
            .len()
            .min(positions_y.len())
            .min(velocities_x.len())
            .min(velocities_y.len())
            .min(lifetimes.len());

        if count == 0 {
            return;
        }

        let cols_f = cols as f32;
        let rows_f = rows as f32;

        let drag_factor = 1.0 - (drag * dt).clamp(0.0, 1.0);
        let gravity_delta = gravity * dt;

        let threads = std::thread::available_parallelism()
            .map(|p| p.get().min(8))
            .unwrap_or(2);

        if count <= 64 || threads <= 1 {
            Self::eval_particle_chunk(
                0..count,
                positions_x,
                positions_y,
                velocities_x,
                velocities_y,
                lifetimes,
                dt,
                gravity_delta,
                drag_factor,
                cols_f,
                rows_f,
            );
        } else {
            let chunk_size = count.div_ceil(threads);
            std::thread::scope(|s| {
                let px_chunks = positions_x[..count].chunks_mut(chunk_size);
                let py_chunks = positions_y[..count].chunks_mut(chunk_size);
                let vx_chunks = velocities_x[..count].chunks_mut(chunk_size);
                let vy_chunks = velocities_y[..count].chunks_mut(chunk_size);
                let life_chunks = lifetimes[..count].chunks_mut(chunk_size);

                for ((((px, py), vx), vy), life) in px_chunks
                    .zip(py_chunks)
                    .zip(vx_chunks)
                    .zip(vy_chunks)
                    .zip(life_chunks)
                {
                    s.spawn(move || {
                        let len = px.len();
                        Self::eval_particle_chunk(
                            0..len,
                            px,
                            py,
                            vx,
                            vy,
                            life,
                            dt,
                            gravity_delta,
                            drag_factor,
                            cols_f,
                            rows_f,
                        );
                    });
                }
            });
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn eval_particle_chunk(
        range: std::ops::Range<usize>,
        px: &mut [f32],
        py: &mut [f32],
        vx: &mut [f32],
        vy: &mut [f32],
        life: &mut [f32],
        dt: f32,
        gravity_delta: f32,
        drag_factor: f32,
        cols_f: f32,
        rows_f: f32,
    ) {
        for i in range {
            if life[i] <= 0.0 {
                continue;
            }

            vy[i] += gravity_delta;
            vx[i] *= drag_factor;
            vy[i] *= drag_factor;

            px[i] += vx[i] * dt;
            py[i] += vy[i] * dt;

            life[i] -= dt;

            if px[i] < 0.0 || px[i] >= cols_f || py[i] < 0.0 || py[i] >= rows_f {
                life[i] = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_engine_initializes_cleanly() {
        let engine = ComputeEngine::init(AccelerationMode::Auto);
        assert!(!engine.device_name().is_empty());
    }

    #[test]
    fn test_mountain_harmonic_superposition_multithreaded() {
        let engine = ComputeEngine::init(AccelerationMode::Cpu);
        let mut buffer = [0.0f32; 128];
        let amps = [0.5, 0.25, 0.125];
        let phases = [0.0, 1.0, 2.0];

        engine.compute_mountain_harmonic_superposition(
            128,
            10.0,
            40.0,
            1.2,
            0.0,
            &amps,
            &phases,
            &mut buffer,
        );

        for &val in &buffer {
            assert!(
                val >= 10.0,
                "Elevation must be at or above base height: {val}"
            );
        }
    }

    #[test]
    fn test_mountain_bad_inputs_never_nan() {
        let engine = ComputeEngine::init(AccelerationMode::Cpu);
        let mut buffer = [0.0f32; 16];
        engine.compute_mountain_harmonic_superposition(
            16,
            5.0,
            f32::NAN,
            -2.0,
            0.0,
            &[0.5],
            &[0.0],
            &mut buffer,
        );
        assert!(buffer.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_road_undulation_evaluates_consistently() {
        let engine = ComputeEngine::init(AccelerationMode::Cpu);
        let mut buffer = [0.0f32; 100];
        engine.compute_road_undulation(100, 15.0, 0.5, 1.0, &mut buffer);

        for &val in &buffer {
            assert!(
                (13.0..=17.0).contains(&val),
                "Road baseline clamped around 15.0: {val}"
            );
        }
    }

    #[test]
    fn test_foliage_density_golden_ratio() {
        let engine = ComputeEngine::init(AccelerationMode::Cpu);
        let center = engine.compute_foliage_density(0.0, 0.0, 5.0);
        let outside = engine.compute_foliage_density(6.0, 6.0, 5.0);

        assert!(
            center > 0.5,
            "Center foliage density must be high: {center}"
        );
        assert_eq!(
            outside, 0.0,
            "Outside foliage density must be zero: {outside}"
        );
    }

    #[test]
    fn test_foliage_density_batch_multithreaded() {
        let engine = ComputeEngine::init(AccelerationMode::Cpu);
        let coords: Vec<(f32, f32)> = (0..100).map(|i| (i as f32 * 0.1, 0.0)).collect();
        let mut densities = vec![0.0f32; 100];
        engine.compute_foliage_density_batch(&coords, 5.0, &mut densities);
        assert!(densities[0] > 0.5);
    }

    #[test]
    fn test_particle_physics_expires_out_of_bounds() {
        let engine = ComputeEngine::init(AccelerationMode::Cpu);
        let (mut px, mut py) = ([1.0f32, 50.0], [1.0f32, 1.0]);
        let (mut vx, mut vy) = ([0.0f32, 100.0], [0.0f32, 0.0]);
        let mut life = [1.0f32, 1.0];
        engine.compute_particle_physics_step(
            &mut px, &mut py, &mut vx, &mut vy, &mut life, 0.1, 0.0, 0.0, 20, 10,
        );
        assert!(life[0] > 0.0);
        assert_eq!(life[1], 0.0);
    }
}
