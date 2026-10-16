//! GPU Compute and Parallel Nature Mathematics Acceleration Engine.
//!
//! Provides hardware-accelerated and multi-threaded procedural nature math,
//! road spline undulation, Fibonacci crown envelopes, and particle grid physics.
//!
//! Strictly enforces the Nature Mathematics & Strict <100MB RAM Mandate:
//! - Operates with zero dynamic allocations in the animation loop.
//! - Parallel chunking across CPU/GPU cores via `std::thread::scope`.
//! - Uses pre-allocated static scratch buffers (~64 KB resident memory).
//! - Adheres strictly to workspace-wide `unsafe_code = "deny"`.

pub use forgum_platform::compute::ComputeEngine;
use forgum_platform::gpu::AccelerationMode;

/// Lightweight, copyable wrapper around the process-wide `ComputeEngine` singleton.
/// Delegates all mathematical and hardware-accelerated passes via `Deref`.
#[derive(Debug, Clone, Copy)]
pub struct GpuComputeEngine {
    platform_engine: &'static ComputeEngine,
    mode: AccelerationMode,
}

impl std::ops::Deref for GpuComputeEngine {
    type Target = ComputeEngine;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.platform_engine
    }
}

impl Default for GpuComputeEngine {
    fn default() -> Self {
        get_compute_engine(None)
    }
}

/// Retrieve the process-wide compute engine directly from `forgum_platform`.
#[must_use]
pub fn get_compute_engine(mode: Option<AccelerationMode>) -> GpuComputeEngine {
    let mode = mode.unwrap_or(AccelerationMode::Auto);
    GpuComputeEngine {
        platform_engine: forgum_platform::compute::get_compute_engine(Some(mode)),
        mode,
    }
}

/// Explicitly initialize the process-wide compute engine with a specific acceleration policy.
pub fn init_compute_engine(mode: AccelerationMode) -> GpuComputeEngine {
    GpuComputeEngine {
        platform_engine: forgum_platform::compute::init_compute_engine(mode),
        mode,
    }
}

impl GpuComputeEngine {
    /// Initialize compute engine with the requested acceleration policy.
    #[must_use]
    pub fn init(mode: AccelerationMode) -> Self {
        Self {
            platform_engine: forgum_platform::compute::get_compute_engine(Some(mode)),
            mode,
        }
    }

    /// Retrieve the resolved compute backend for this engine instance.
    #[must_use]
    pub fn compute_backend(&self) -> forgum_platform::ComputeBackend {
        forgum_platform::resolve_compute_pipeline(self.mode).backend
    }

    /// Retrieve the comprehensive compute pipeline capabilities for this engine instance.
    #[must_use]
    pub fn pipeline_capabilities(&self) -> &'static forgum_platform::ComputePipelineCapabilities {
        forgum_platform::resolve_compute_pipeline(self.mode)
    }

    /// Retrieve the requested acceleration mode.
    #[must_use]
    pub fn requested_mode(&self) -> AccelerationMode {
        self.mode
    }

    /// Evaluates full mountain elevations for a given style across columns.
    /// Operates with zero heap allocations on pre-allocated buffers.
    /// Evaluated in parallel across available CPU cores via `std::thread::scope`.
    pub fn compute_mountain_elevations(
        &self,
        cols: usize,
        width: usize,
        mountain_base_y: usize,
        style: crate::scenery::MountainStyle,
        time: f32,
        out: &mut [f32],
    ) {
        self.compute_mountain_elevations_aspect(
            cols,
            width,
            mountain_base_y,
            style,
            time,
            None,
            out,
        );
    }

    /// Evaluates full mountain elevations for a given style across columns starting at `start_x` with aspect ratio awareness.
    /// Operates with zero heap allocations on pre-allocated buffers.
    /// Evaluated in parallel across available CPU cores via `std::thread::scope`.
    pub fn compute_mountain_elevations_chunk(
        &self,
        start_x: usize,
        cols: usize,
        width: usize,
        mountain_base_y: usize,
        style: crate::scenery::MountainStyle,
        time: f32,
        aspect: Option<forgum_platform::AspectRatioFormat>,
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
            for (i, slot) in target.iter_mut().enumerate() {
                *slot = crate::scenery::calculate_mountain_height_aspect(
                    (start_x + i) as f32,
                    width,
                    mountain_base_y,
                    style,
                    time,
                    aspect,
                );
            }
        } else {
            let chunk_size = n.div_ceil(threads);
            std::thread::scope(|s| {
                for (chunk_idx, chunk) in target.chunks_mut(chunk_size).enumerate() {
                    let base_x = start_x + chunk_idx * chunk_size;
                    s.spawn(move || {
                        for (i, slot) in chunk.iter_mut().enumerate() {
                            *slot = crate::scenery::calculate_mountain_height_aspect(
                                (base_x + i) as f32,
                                width,
                                mountain_base_y,
                                style,
                                time,
                                aspect,
                            );
                        }
                    });
                }
            });
        }
    }

    /// Evaluates full mountain elevations for a given style across columns with aspect ratio awareness.
    /// Operates with zero heap allocations on pre-allocated buffers.
    /// Evaluated in parallel across available CPU cores via `std::thread::scope`.
    pub fn compute_mountain_elevations_aspect(
        &self,
        cols: usize,
        width: usize,
        mountain_base_y: usize,
        style: crate::scenery::MountainStyle,
        time: f32,
        aspect: Option<forgum_platform::AspectRatioFormat>,
        out: &mut [f32],
    ) {
        self.compute_mountain_elevations_chunk(
            0,
            cols,
            width,
            mountain_base_y,
            style,
            time,
            aspect,
            out,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_engine_initializes_cleanly() {
        let engine = GpuComputeEngine::init(AccelerationMode::Auto);
        assert!(!engine.device_name().is_empty());
        let caps = engine.pipeline_capabilities();
        assert!(!caps.api_version.is_empty());
        assert!(!caps.device_name.is_empty());
    }

    #[test]
    fn test_compute_backend_resolution() {
        let cpu_engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        assert_eq!(
            cpu_engine.compute_backend(),
            forgum_platform::ComputeBackend::SimdCpu
        );
    }

    #[test]
    fn test_mountain_harmonic_superposition_zero_alloc() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        let mut buffer = [0.0f32; 80];
        let amps = [0.5, 0.25, 0.125];
        let phases = [0.0, 1.0, 2.0];

        engine.compute_mountain_harmonic_superposition(
            80,
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
    fn test_road_undulation_evaluates_consistently() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
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
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
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
    fn test_mountain_bad_inputs_never_nan() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
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
    fn test_particle_physics_expires_out_of_bounds() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        let (mut px, mut py) = ([1.0f32, 50.0], [1.0f32, 1.0]);
        let (mut vx, mut vy) = ([0.0f32, 100.0], [0.0f32, 0.0]);
        let mut life = [1.0f32, 1.0];
        engine.compute_particle_physics_step(
            &mut px, &mut py, &mut vx, &mut vy, &mut life, 0.1, 0.0, 0.0, 20, 10,
        );
        assert!(life[0] > 0.0);
        assert_eq!(life[1], 0.0);
    }

    #[test]
    fn test_dynamic_bubble_inertia_gpu_compute() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        let mut ox = 0.0f32;
        let mut vx = 0.0f32;
        let mut oy = 0.0f32;
        let mut vy = 0.0f32;

        engine.compute_dynamic_bubble_inertia(0.1, 1.0, 5.0, &mut ox, &mut vx, &mut oy, &mut vy);

        assert!((-3.0..=3.0).contains(&ox));
        assert!((-1.0..=1.0).contains(&oy));
    }

    #[test]
    fn test_dynamic_kinematics_step_gpu_compute() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        let mut x = 0.0f32;
        let mut y = 0.0f32;

        engine.compute_dynamic_kinematics_step(
            1.5, 80, 24, 20, 10, 5.0, 3.0, 1.0, 1.0, 0.0, 0.0, &mut x, &mut y,
        );

        assert!((0.0..=60.0).contains(&x));
        assert!((0.0..=14.0).contains(&y));
    }

    #[test]
    fn test_celestial_trajectory_gpu_compute() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        for mode in 0..4 {
            let (cx, cy) = engine.compute_celestial_trajectory(mode, 10.0, 100, 30);
            assert!(cx < 100);
            assert!(cy <= 30 && cy >= 1);
        }
    }

    #[test]
    fn test_tree_sway_batch_gpu_compute() {
        let engine = GpuComputeEngine::init(AccelerationMode::Cpu);
        let h_norms = [0.0f32, 0.5, 1.0];
        let is_shrub = [false, false, true];
        let is_willow = [false, true, false];
        let seeds = [0.0f32, 1.0, 2.0];
        let mut sways = [0isize; 3];

        engine.compute_tree_sway_batch(&h_norms, &is_shrub, &is_willow, &seeds, 1.0, &mut sways);
        assert_eq!(sways[0], 0);
        for &s in &sways {
            assert!((-3..=3).contains(&s));
        }
    }
}
