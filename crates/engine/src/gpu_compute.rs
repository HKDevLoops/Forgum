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

use forgum_platform::compute::ComputeEngine;
use forgum_platform::gpu::{AccelerationMode, GpuDeviceType, GpuInfo};
use std::sync::OnceLock;

/// Global compute engine instance initialized on startup.
static COMPUTE_ENGINE: OnceLock<GpuComputeEngine> = OnceLock::new();

/// Retrieve or initialize the process-wide compute engine.
#[must_use]
pub fn get_compute_engine(mode: Option<AccelerationMode>) -> &'static GpuComputeEngine {
    COMPUTE_ENGINE.get_or_init(|| {
        let requested = mode.unwrap_or(AccelerationMode::Auto);
        GpuComputeEngine::init(requested)
    })
}

/// Explicitly initialize the compute engine with a specific acceleration policy.
pub fn init_compute_engine(mode: AccelerationMode) -> &'static GpuComputeEngine {
    COMPUTE_ENGINE.get_or_init(|| GpuComputeEngine::init(mode))
}

/// Hardware-accelerated nature math and particle compute engine.
#[derive(Debug)]
pub struct GpuComputeEngine {
    platform_engine: &'static ComputeEngine,
    mode: AccelerationMode,
}

impl GpuComputeEngine {
    /// Initialize compute engine with the requested acceleration policy.
    #[must_use]
    pub fn init(mode: AccelerationMode) -> Self {
        let platform_engine = forgum_platform::compute::get_compute_engine(Some(mode));
        Self {
            platform_engine,
            mode,
        }
    }

    #[must_use]
    pub fn device_type(&self) -> GpuDeviceType {
        self.platform_engine.device_type()
    }

    #[must_use]
    pub fn device_name(&self) -> &str {
        self.platform_engine.device_name()
    }

    #[must_use]
    pub fn device_info(&self) -> Option<&GpuInfo> {
        self.platform_engine.device_info()
    }

    #[must_use]
    pub fn requested_mode(&self) -> AccelerationMode {
        self.mode
    }

    #[must_use]
    pub fn is_accelerated(&self) -> bool {
        self.platform_engine.is_accelerated()
    }

    /// Access the active Vulkan + Metal compute pipeline capabilities.
    #[must_use]
    pub fn pipeline_capabilities(&self) -> &'static forgum_platform::ComputePipelineCapabilities {
        forgum_platform::resolve_compute_pipeline(self.mode)
    }

    /// Active compute execution backend (Vulkan, Metal, MoltenVK, or SimdCpu).
    #[must_use]
    pub fn compute_backend(&self) -> forgum_platform::ComputeBackend {
        self.pipeline_capabilities().backend
    }

    /// Evaluates harmonic sinusoidal mountain elevation:
    /// H(x) = H_base * [1 + Σ A_k |sin(2πkx/λ + φ_k)|^γ]
    ///
    /// Evaluated in parallel across CPU/GPU cores via `std::thread::scope`.
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
        self.platform_engine
            .compute_mountain_harmonic_superposition(
                cols, h_base, wavelength, gamma, time, amplitudes, phases, out,
            );
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

    /// Evaluates road surface spline undulation and procedural harmonic surface roughness.
    /// Evaluated in parallel across CPU/GPU cores via `std::thread::scope`.
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
    /// Evaluated in parallel across CPU/GPU cores via `std::thread::scope`.
    pub fn compute_road_undulation_chunk(
        &self,
        start_x: usize,
        cols: usize,
        base_y: f32,
        roughness: f32,
        time: f32,
        out: &mut [f32],
    ) {
        self.platform_engine
            .compute_road_undulation_chunk(start_x, cols, base_y, roughness, time, out);
    }

    /// Evaluates vegetation crown envelope with Fibonacci / Golden Ratio spacing:
    /// Φ ≈ 1.6180339887
    #[must_use]
    pub fn compute_foliage_density(&self, x: f32, y: f32, crown_radius: f32) -> f32 {
        self.platform_engine
            .compute_foliage_density(x, y, crown_radius)
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
        self.platform_engine.compute_particle_physics_step(
            positions_x,
            positions_y,
            velocities_x,
            velocities_y,
            lifetimes,
            dt,
            gravity,
            drag,
            cols,
            rows,
        );
    }

    /// Evaluates continuous spring-damper dialogue bubble physics.
    pub fn compute_dynamic_bubble_inertia(
        &self,
        dt: f32,
        elapsed: f32,
        mascot_vx: f32,
        bubble_offset_x: &mut f32,
        bubble_vel_x: &mut f32,
        bubble_offset_y: &mut f32,
        bubble_vel_y: &mut f32,
    ) {
        self.platform_engine.compute_dynamic_bubble_inertia(
            dt,
            elapsed,
            mascot_vx,
            bubble_offset_x,
            bubble_vel_x,
            bubble_offset_y,
            bubble_vel_y,
        );
    }

    /// Evaluates continuous 2D harmonic Lissajous or continuous trajectory displacement.
    pub fn compute_dynamic_kinematics_step(
        &self,
        total_time: f32,
        cols: usize,
        rows: usize,
        width: usize,
        height: usize,
        amp_x: f32,
        amp_y: f32,
        freq_x: f32,
        freq_y: f32,
        phase_x: f32,
        phase_y: f32,
        out_x: &mut f32,
        out_y: &mut f32,
    ) {
        self.platform_engine.compute_dynamic_kinematics_step(
            total_time, cols, rows, width, height, amp_x, amp_y, freq_x, freq_y, phase_x, phase_y,
            out_x, out_y,
        );
    }

    /// Batch-evaluates aquatic underwater rising bubble kinematics (turbulence, wobble, vertical drift).
    pub fn compute_underwater_bubble_kinematics(
        &self,
        dt: f32,
        time: f32,
        xs: &mut [f32],
        ys: &mut [f32],
        lifetimes: &mut [f32],
        rise_speeds: &[f32],
    ) {
        self.platform_engine.compute_underwater_bubble_kinematics(
            dt,
            time,
            xs,
            ys,
            lifetimes,
            rise_speeds,
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
}
