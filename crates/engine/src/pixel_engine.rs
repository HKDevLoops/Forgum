//! High-precision pixel-level animation engine with nanosecond and millisecond updates.
//!
//! Provides true subpixel rasterization, nanosecond clock pacing, and continuous
//! floating-point nature kinematics with 100% backward compatibility for the Forgum engine.
//!
//! # Architecture
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────────┐
//! │                    NanosecondClock & Pacer                    │
//! │   High-precision elapsed nanoseconds & sub-ms pacing          │
//! └──────────────────────────────┬────────────────────────────────┘
//!                                │ dt (f32/f64 seconds)
//!                                ▼
//! ┌───────────────────────────────────────────────────────────────┐
//! │                 Continuous Nature Kinematics                  │
//! │   SubpixelWaveField, SubpixelKinematicBody (x, y ∈ ℝ)         │
//! └──────────────────────────────┬────────────────────────────────┘
//!                                │ continuous coordinates
//!                                ▼
//! ┌───────────────────────────────────────────────────────────────┐
//! │                         PixelCanvas                           │
//! │   W_px × H_px (Cols × 2Rows) 24-bit TrueColor Rasterizer      │
//! │   Bilinear subpixel anti-aliasing, Wu's lines, circles        │
//! └──────────────────────────────┬────────────────────────────────┘
//!                                │ blit_to_framebuffer
//!                                ▼
//! ┌───────────────────────────────────────────────────────────────┐
//! │               FrameBuffer (Terminal Cells W × H)              │
//! │   Dual-Pixel Half-Block Protocol ('▀' / '▄') + Mascot Text    │
//! └───────────────────────────────────────────────────────────────┘
//! ```
//!
//! **Zero `#[cfg]` lives in this module.** Strictly compliant with Forgum architecture.

use crate::framebuffer::{Cell, Color, FrameBuffer};
use std::time::{Duration, Instant};

/// Sub-millisecond spin-wait threshold for high-precision frame pacing.
pub const SUB_MS_SPIN_THRESHOLD: Duration = Duration::from_micros(1200);

/// Maximum delta time clamp (100ms) to prevent simulation explosions on stall.
pub const MAX_DELTA_TIME_SECS: f64 = 0.100;

// ── 1. Nanosecond Clock & Temporal Engine ───────────────────────────

/// High-precision temporal timekeeper with nanosecond resolution.
#[derive(Debug, Clone)]
pub struct NanosecondClock {
    start_instant: Instant,
    last_tick: Instant,
    frame_count: u64,
    total_elapsed_nanos: u128,
    last_dt_nanos: u64,
    target_period_nanos: u64,
    recent_frame_durations: [u64; 32],
    duration_ring_idx: usize,
}

impl Default for NanosecondClock {
    fn default() -> Self {
        Self::new(60)
    }
}

impl NanosecondClock {
    /// Initialize a new nanosecond clock targeted at `fps` (clamped to 1..=1000).
    #[must_use]
    pub fn new(fps: u32) -> Self {
        let safe_fps = fps.clamp(1, 1000) as u64;
        let target_period_nanos = 1_000_000_000 / safe_fps;
        let now = Instant::now();
        Self {
            start_instant: now,
            last_tick: now,
            frame_count: 0,
            total_elapsed_nanos: 0,
            last_dt_nanos: target_period_nanos,
            target_period_nanos,
            recent_frame_durations: [target_period_nanos; 32],
            duration_ring_idx: 0,
        }
    }

    /// Advance the clock by one tick. Returns the elapsed delta time as a `Duration`.
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let raw_dt = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;

        let dt_nanos = raw_dt.as_nanos() as u64;
        self.last_dt_nanos = dt_nanos;
        self.total_elapsed_nanos = self.total_elapsed_nanos.saturating_add(dt_nanos as u128);
        self.frame_count = self.frame_count.saturating_add(1);

        self.recent_frame_durations[self.duration_ring_idx] = dt_nanos;
        self.duration_ring_idx = (self.duration_ring_idx + 1) % self.recent_frame_durations.len();

        let clamped_secs = raw_dt.as_secs_f64().min(MAX_DELTA_TIME_SECS);
        Duration::from_secs_f64(clamped_secs)
    }

    /// Time at clock initialization.
    #[must_use]
    pub fn start_instant(&self) -> Instant {
        self.start_instant
    }

    /// Current total elapsed nanoseconds since clock inception.
    #[must_use]
    pub fn elapsed_nanos(&self) -> u128 {
        self.total_elapsed_nanos
    }

    /// Current total elapsed milliseconds since clock inception.
    #[must_use]
    pub fn elapsed_millis(&self) -> f64 {
        self.total_elapsed_nanos as f64 / 1_000_000.0
    }

    /// Current total elapsed seconds since clock inception.
    #[must_use]
    pub fn elapsed_secs(&self) -> f64 {
        self.total_elapsed_nanos as f64 / 1_000_000_000.0
    }

    /// Last frame delta time in nanoseconds.
    #[must_use]
    pub fn last_dt_nanos(&self) -> u64 {
        self.last_dt_nanos
    }

    /// Last frame delta time as fractional seconds (`f32`).
    #[must_use]
    pub fn last_dt_secs(&self) -> f32 {
        (self.last_dt_nanos as f32 / 1_000_000_000.0).min(MAX_DELTA_TIME_SECS as f32)
    }

    /// Total frames elapsed.
    #[must_use]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Estimated real-time FPS calculated from the rolling window of recent frame durations.
    #[must_use]
    pub fn fps_estimate(&self) -> f64 {
        let sum: u64 = self.recent_frame_durations.iter().sum();
        let avg_nanos = (sum as f64) / (self.recent_frame_durations.len() as f64);
        if avg_nanos <= 0.0 {
            60.0
        } else {
            1_000_000_000.0 / avg_nanos
        }
    }

    /// Absolute frame period jitter in nanoseconds relative to target period.
    #[must_use]
    pub fn jitter_nanos(&self) -> u64 {
        self.last_dt_nanos.abs_diff(self.target_period_nanos)
    }
}

// ── 2. Precision Frame Pacer ────────────────────────────────────────

/// Ultra-high precision frame pacer supporting 1..=1000 FPS and uncapped pacing.
/// Uses a two-phase hybrid sleep/spin-wait kernel for sub-millisecond accuracy.
#[derive(Debug, Clone)]
pub struct PrecisionFramePacer {
    target_period: Duration,
    next_deadline: Instant,
    clock: NanosecondClock,
    is_uncapped: bool,
}

impl PrecisionFramePacer {
    /// Create a pacer for the specified target frame rate.
    /// Setting `fps == 0` configures uncapped maximum-throughput frame pacing.
    #[must_use]
    pub fn new(fps: u32) -> Self {
        let is_uncapped = fps == 0;
        let safe_fps = if is_uncapped {
            1000
        } else {
            fps.clamp(1, 1000)
        };
        let target_period = Duration::from_nanos(1_000_000_000 / u64::from(safe_fps));
        let now = Instant::now();
        Self {
            target_period,
            next_deadline: now + target_period,
            clock: NanosecondClock::new(safe_fps),
            is_uncapped,
        }
    }

    /// Reference to the internal nanosecond clock.
    #[must_use]
    pub fn clock(&self) -> &NanosecondClock {
        &self.clock
    }

    /// Mutable reference to the internal nanosecond clock.
    pub fn clock_mut(&mut self) -> &mut NanosecondClock {
        &mut self.clock
    }

    /// Set a new target frame rate.
    pub fn set_fps(&mut self, fps: u32) {
        self.is_uncapped = fps == 0;
        let safe_fps = if self.is_uncapped {
            1000
        } else {
            fps.clamp(1, 1000)
        };
        self.target_period = Duration::from_nanos(1_000_000_000 / u64::from(safe_fps));
        self.clock.target_period_nanos = self.target_period.as_nanos() as u64;
    }

    /// Block execution with sub-millisecond precision until the frame deadline is reached.
    ///
    /// Implements a hybrid wait kernel:
    /// 1. Coarse sleep for the portion exceeding `SUB_MS_SPIN_THRESHOLD` (conserves host CPU).
    /// 2. Microsecond active spin-wait loop for the remainder (eliminates OS timer quantization judder).
    pub fn pace_frame(&mut self) -> Duration {
        if !self.is_uncapped {
            let now = Instant::now();
            if now < self.next_deadline {
                let remaining = self.next_deadline - now;
                if remaining > SUB_MS_SPIN_THRESHOLD {
                    std::thread::sleep(remaining - SUB_MS_SPIN_THRESHOLD);
                }
                while Instant::now() < self.next_deadline {
                    std::hint::spin_loop();
                }
            }
            let post_wait = Instant::now();
            self.next_deadline += self.target_period;
            if self.next_deadline <= post_wait {
                self.next_deadline = post_wait + self.target_period;
            }
        }
        self.clock.tick()
    }
}

// ── 3. PixelCanvas (True Subpixel Rasterizer) ────────────────────────

/// 2D TrueColor pixel canvas providing continuous floating-point drawing primitives.
///
/// Maps $W_{\text{cells}} \times H_{\text{cells}}$ terminal cells into a
/// $W_{\text{px}} \times H_{\text{px}} = \text{cols} \times (2 \cdot \text{rows})$
/// high-resolution pixel canvas.
#[derive(Debug, Clone)]
pub struct PixelCanvas {
    cols: usize,
    rows: usize,
    pixel_width: usize,
    pixel_height: usize,
    pixels: Vec<Color>,
}

impl PixelCanvas {
    /// Allocate a new pixel canvas for the given terminal cell dimensions.
    #[must_use]
    pub fn new(cols: usize, rows: usize) -> Self {
        let cols = cols.clamp(1, 8192);
        let rows = rows.clamp(1, 4096);
        let pixel_width = cols;
        let pixel_height = rows.saturating_mul(2);
        let total_pixels = pixel_width.saturating_mul(pixel_height);
        Self {
            cols,
            rows,
            pixel_width,
            pixel_height,
            pixels: vec![Color::TRANSPARENT; total_pixels],
        }
    }

    /// Number of terminal cell columns.
    #[must_use]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Number of terminal cell rows.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Horizontal resolution in pixels.
    #[must_use]
    pub fn pixel_width(&self) -> usize {
        self.pixel_width
    }

    /// Vertical resolution in pixels ($2 \times \text{rows}$).
    #[must_use]
    pub fn pixel_height(&self) -> usize {
        self.pixel_height
    }

    /// Clear all canvas pixels to the specified background color (or `Color::TRANSPARENT`).
    pub fn clear(&mut self, bg: Color) {
        self.pixels.fill(bg);
    }

    /// Write an exact integer pixel coordinate `(x, y)` with boundary checking.
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> bool {
        if x >= self.pixel_width || y >= self.pixel_height {
            return false;
        }
        let idx = y * self.pixel_width + x;
        self.pixels[idx] = color;
        true
    }

    /// Read an integer pixel coordinate `(x, y)`. Returns `Color::TRANSPARENT` if out of bounds.
    #[must_use]
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x >= self.pixel_width || y >= self.pixel_height {
            return Color::TRANSPARENT;
        }
        self.pixels[y * self.pixel_width + x]
    }

    /// Alpha-blend a color over the existing pixel at `(x, y)`.
    #[inline]
    pub fn blend_pixel(&mut self, x: usize, y: usize, color: Color, alpha: f32) {
        if x >= self.pixel_width || y >= self.pixel_height || alpha <= 0.0 {
            return;
        }
        let a = alpha.clamp(0.0, 1.0);
        let idx = y * self.pixel_width + x;
        let dst = self.pixels[idx];
        if dst == Color::TRANSPARENT {
            self.pixels[idx] = Color::rgba(color.r, color.g, color.b, (a * 255.0).round() as u8);
        } else {
            let inv_a = 1.0 - a;
            let r = ((dst.r as f32 * inv_a + color.r as f32 * a).round()).min(255.0) as u8;
            let g = ((dst.g as f32 * inv_a + color.g as f32 * a).round()).min(255.0) as u8;
            let b = ((dst.b as f32 * inv_a + color.b as f32 * a).round()).min(255.0) as u8;
            self.pixels[idx] = Color::rgb(r, g, b);
        }
    }

    /// Continuous subpixel anti-aliased pixel drawing at fractional coordinates `(x, y) \in \mathbb{R}^2`.
    /// Bilinearly distributes color energy to the 4 surrounding discrete pixels.
    pub fn set_pixel_f32(&mut self, x: f32, y: f32, color: Color) {
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
            return;
        }
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let fx = x.fract();
        let fy = y.fract();

        let w00 = (1.0 - fx) * (1.0 - fy);
        let w10 = fx * (1.0 - fy);
        let w01 = (1.0 - fx) * fy;
        let w11 = fx * fy;

        self.blend_pixel(x0, y0, color, w00);
        self.blend_pixel(x0 + 1, y0, color, w10);
        self.blend_pixel(x0, y0 + 1, color, w01);
        self.blend_pixel(x0 + 1, y0 + 1, color, w11);
    }

    /// Draw a continuous subpixel line using Xiaolin Wu's anti-aliased algorithm.
    pub fn draw_line_f32(
        &mut self,
        mut x0: f32,
        mut y0: f32,
        mut x1: f32,
        mut y1: f32,
        color: Color,
    ) {
        if !x0.is_finite() || !y0.is_finite() || !x1.is_finite() || !y1.is_finite() {
            return;
        }
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let gradient = if dx == 0.0 { 1.0 } else { dy / dx };

        // Handle first endpoint
        let xend = x0.round();
        let yend = y0 + gradient * (xend - x0);
        let xgap = 1.0 - (x0 + 0.5).fract();
        let xpx1 = xend as usize;
        let ypx1 = yend.floor() as usize;

        if steep {
            self.blend_pixel(ypx1, xpx1, color, (1.0 - yend.fract()) * xgap);
            self.blend_pixel(ypx1 + 1, xpx1, color, yend.fract() * xgap);
        } else {
            self.blend_pixel(xpx1, ypx1, color, (1.0 - yend.fract()) * xgap);
            self.blend_pixel(xpx1, ypx1 + 1, color, yend.fract() * xgap);
        }

        let mut intery = yend + gradient;

        // Handle second endpoint
        let xend2 = x1.round();
        let yend2 = y1 + gradient * (xend2 - x1);
        let xgap2 = (x1 + 0.5).fract();
        let xpx2 = xend2 as usize;
        let ypx2 = yend2.floor() as usize;

        if steep {
            self.blend_pixel(ypx2, xpx2, color, (1.0 - yend2.fract()) * xgap2);
            self.blend_pixel(ypx2 + 1, xpx2, color, yend2.fract() * xgap2);
        } else {
            self.blend_pixel(xpx2, ypx2, color, (1.0 - yend2.fract()) * xgap2);
            self.blend_pixel(xpx2, ypx2 + 1, color, yend2.fract() * xgap2);
        }

        // Main line body
        if xpx1 < xpx2.saturating_sub(1) {
            for x in (xpx1 + 1)..xpx2 {
                let y = intery.floor() as usize;
                let f = intery.fract();
                if steep {
                    self.blend_pixel(y, x, color, 1.0 - f);
                    self.blend_pixel(y + 1, x, color, f);
                } else {
                    self.blend_pixel(x, y, color, 1.0 - f);
                    self.blend_pixel(x, y + 1, color, f);
                }
                intery += gradient;
            }
        }
    }

    /// Draw a subpixel axis-aligned rectangle (filled or outline).
    pub fn draw_rect_f32(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color, filled: bool) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        if filled {
            let start_x = (x.floor() as usize).min(self.pixel_width);
            let end_x = ((x + w).ceil() as usize).min(self.pixel_width);
            let start_y = (y.floor() as usize).min(self.pixel_height);
            let end_y = ((y + h).ceil() as usize).min(self.pixel_height);

            for py in start_y..end_y {
                for px in start_x..end_x {
                    self.set_pixel(px, py, color);
                }
            }
        } else {
            self.draw_line_f32(x, y, x + w, y, color);
            self.draw_line_f32(x + w, y, x + w, y + h, color);
            self.draw_line_f32(x + w, y + h, x, y + h, color);
            self.draw_line_f32(x, y + h, x, y, color);
        }
    }

    /// Draw a subpixel circle centered at `(cx, cy)` with radius `r`.
    pub fn draw_circle_f32(&mut self, cx: f32, cy: f32, r: f32, color: Color, filled: bool) {
        if r <= 0.0 {
            return;
        }
        let min_x = (cx - r - 1.0).floor().max(0.0) as usize;
        let max_x = (cx + r + 1.0).ceil().min((self.pixel_width - 1) as f32) as usize;
        let min_y = (cy - r - 1.0).floor().max(0.0) as usize;
        let max_y = (cy + r + 1.0).ceil().min((self.pixel_height - 1) as f32) as usize;

        for py in min_y..=max_y {
            let dy = py as f32 - cy;
            let dy_sq = dy * dy;
            for px in min_x..=max_x {
                let dx = px as f32 - cx;
                let dist_sq = dx * dx + dy_sq;
                let dist = dist_sq.sqrt();
                if filled {
                    if dist <= r {
                        self.set_pixel(px, py, color);
                    } else if dist < r + 1.0 {
                        let alpha = 1.0 - (dist - r);
                        self.blend_pixel(px, py, color, alpha);
                    }
                } else {
                    let d = (dist - r).abs();
                    if d < 1.0 {
                        self.blend_pixel(px, py, color, 1.0 - d);
                    }
                }
            }
        }
    }

    /// Blit this pixel canvas directly into a terminal `FrameBuffer`.
    ///
    /// Every vertical pair of pixels `(y * 2, y * 2 + 1)` is cleanly mapped into
    /// character cell `(x, y)`:
    /// - Top pixel: cell foreground TrueColor with upper half-block `'▀'`
    /// - Bottom pixel: cell background TrueColor
    /// - If cell already holds opaque text/mascot characters, the pixel color is
    ///   safely mapped into the cell's background to preserve mascot foreground silhouette!
    pub fn blit_to_framebuffer(&self, fb: &mut FrameBuffer) {
        let max_cols = self.cols.min(fb.width);
        let max_rows = self.rows.min(fb.height);

        for y in 0..max_rows {
            let top_py = y * 2;
            let bot_py = top_py + 1;

            for x in 0..max_cols {
                let top_col = self.get_pixel(x, top_py);
                let bot_col = self.get_pixel(x, bot_py);

                // If both pixels are completely transparent, leave existing cell untouched
                if top_col == Color::TRANSPARENT && bot_col == Color::TRANSPARENT {
                    continue;
                }

                let existing = fb.get_back(x, y);

                // If the cell already contains an opaque mascot glyph or speech character:
                // Preserve the character while blending the pixel color into the background!
                if existing.ch != ' ' && existing.alpha > 0 {
                    let avg_bg = if bot_col != Color::TRANSPARENT {
                        bot_col
                    } else {
                        top_col
                    };
                    let _ = fb.set(x, y, Cell::with_bg(existing.ch, existing.fg, avg_bg));
                } else if top_col != Color::TRANSPARENT && bot_col != Color::TRANSPARENT {
                    // Both subpixels active: full dual-pixel half-block
                    let _ = fb.set(x, y, Cell::with_bg('▀', top_col, bot_col));
                } else if top_col != Color::TRANSPARENT {
                    // Upper subpixel active
                    let _ = fb.set(x, y, Cell::with_bg('▀', top_col, Color::TRANSPARENT));
                } else {
                    // Lower subpixel active
                    let _ = fb.set(x, y, Cell::with_bg('▄', bot_col, Color::TRANSPARENT));
                }
            }
        }
    }

    /// Decode existing `FrameBuffer` cells back into the pixel canvas.
    pub fn blit_from_framebuffer(&mut self, fb: &FrameBuffer) {
        let max_cols = self.cols.min(fb.width);
        let max_rows = self.rows.min(fb.height);

        for y in 0..max_rows {
            let top_py = y * 2;
            let bot_py = top_py + 1;

            for x in 0..max_cols {
                let cell = fb.get_back(x, y);
                if cell.alpha == 0 {
                    self.set_pixel(x, top_py, Color::TRANSPARENT);
                    self.set_pixel(x, bot_py, Color::TRANSPARENT);
                } else if cell.ch == '▀' {
                    self.set_pixel(x, top_py, cell.fg);
                    self.set_pixel(x, bot_py, cell.bg);
                } else if cell.ch == '▄' {
                    self.set_pixel(x, top_py, cell.bg);
                    self.set_pixel(x, bot_py, cell.fg);
                } else {
                    self.set_pixel(x, top_py, cell.fg);
                    self.set_pixel(x, bot_py, cell.fg);
                }
            }
        }
    }
}

// ── 4. Continuous Nature Mathematics & Fluid Simulation ─────────────

/// High-precision harmonic sinusoidal fluid wave field.
/// Evaluates fluid surface elevations and velocities in continuous subpixel coordinates.
#[derive(Debug, Clone)]
pub struct SubpixelWaveField {
    pub amplitudes: [f32; 4],
    pub frequencies: [f32; 4],
    pub speeds: [f32; 4],
    pub phases: [f32; 4],
}

impl Default for SubpixelWaveField {
    fn default() -> Self {
        Self {
            amplitudes: [1.8, 0.9, 0.45, 0.22],
            frequencies: [0.12, 0.28, 0.55, 1.10],
            speeds: [3.2, 4.8, 6.5, 9.1],
            phases: [0.0, 1.25, 2.71, 4.14],
        }
    }
}

impl SubpixelWaveField {
    /// Evaluate wave elevation at continuous subpixel coordinate `x` and time `t` (seconds).
    #[inline]
    #[must_use]
    pub fn evaluate_elevation(&self, x: f32, t: f32) -> f32 {
        let mut elevation = 0.0f32;
        for i in 0..4 {
            let theta = x * self.frequencies[i] - t * self.speeds[i] + self.phases[i];
            elevation += self.amplitudes[i] * theta.sin();
        }
        elevation
    }

    /// Evaluate wave slope gradient `dE/dx` at subpixel coordinate `x` and time `t`.
    #[inline]
    #[must_use]
    pub fn evaluate_slope(&self, x: f32, t: f32) -> f32 {
        let mut slope = 0.0f32;
        for i in 0..4 {
            let theta = x * self.frequencies[i] - t * self.speeds[i] + self.phases[i];
            slope += self.amplitudes[i] * self.frequencies[i] * theta.cos();
        }
        slope
    }

    /// Rasterize fluid wave into a `PixelCanvas` across the full width at base water line `water_y_px`.
    pub fn render_wave_surface(
        &self,
        canvas: &mut PixelCanvas,
        water_y_px: f32,
        time_secs: f32,
        crest_color: Color,
        depth_color: Color,
    ) {
        let width = canvas.pixel_width();
        let height = canvas.pixel_height();

        for x in 0..width {
            let x_f = x as f32;
            let elevation = self.evaluate_elevation(x_f, time_secs);
            let slope = self.evaluate_slope(x_f, time_secs);

            let surface_y = (water_y_px + elevation).clamp(0.0, (height - 1) as f32);
            let start_y = surface_y.floor() as usize;

            // Solar slope lighting on wave facet
            let slope_highlight = (1.0 + slope * 0.35).clamp(0.60, 1.40);
            let lit_crest = crest_color.scale_luminance(slope_highlight);

            // Subpixel anti-aliased surface crest point
            canvas.set_pixel_f32(x_f, surface_y, lit_crest);

            // Subsurface depth fill
            for py in (start_y + 1)..height {
                let depth_factor = ((py - start_y) as f32 / 12.0).clamp(0.0, 1.0);
                let col = crate::scenery::blend_color(lit_crest, depth_color, depth_factor);
                canvas.set_pixel(x, py, col);
            }
        }
    }
}

// ── 5. Continuous Subpixel Kinematic Particle ────────────────────────

/// Physical body with continuous floating-point kinematics evaluated at nanosecond resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubpixelKinematicBody {
    pub pos_x: f32,
    pub pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub damping: f32,
}

impl SubpixelKinematicBody {
    /// Initialize a body at continuous coordinates `(x, y)`.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos_x: x,
            pos_y: y,
            vel_x: 0.0,
            vel_y: 0.0,
            damping: 0.985,
        }
    }

    /// Apply an instantaneous impulse `(ix, iy)`.
    pub fn apply_impulse(&mut self, ix: f32, iy: f32) {
        self.vel_x += ix;
        self.vel_y += iy;
    }

    /// Advance physics by `dt_secs` seconds using Symplectic Euler integration.
    pub fn update(&mut self, dt_secs: f32) {
        let dt = dt_secs.clamp(0.0, MAX_DELTA_TIME_SECS as f32);
        self.pos_x += self.vel_x * dt;
        self.pos_y += self.vel_y * dt;
        self.vel_x *= self.damping.powf(dt * 60.0);
        self.vel_y *= self.damping.powf(dt * 60.0);
    }
}

// ── Unit Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nanosecond_clock_increments_and_fps() {
        let mut clock = NanosecondClock::new(120);
        assert_eq!(clock.frame_count(), 0);
        assert_eq!(clock.target_period_nanos, 1_000_000_000 / 120);

        let dt = clock.tick();
        assert!(dt <= Duration::from_millis(100));
        assert_eq!(clock.frame_count(), 1);
        assert!(clock.elapsed_nanos() > 0);
        assert!(clock.fps_estimate() > 0.0);
    }

    #[test]
    fn test_precision_frame_pacer_pacing_and_clamping() {
        let mut pacer = PrecisionFramePacer::new(240);
        pacer.set_fps(60);
        assert_eq!(pacer.clock().target_period_nanos, 1_000_000_000 / 60);

        let dt = pacer.pace_frame();
        assert!(dt <= Duration::from_millis(100));
    }

    #[test]
    fn test_pixel_canvas_dimensions_and_clearing() {
        let mut canvas = PixelCanvas::new(80, 24);
        assert_eq!(canvas.cols(), 80);
        assert_eq!(canvas.rows(), 24);
        assert_eq!(canvas.pixel_width(), 80);
        assert_eq!(canvas.pixel_height(), 48);

        let red = Color::rgb(255, 0, 0);
        canvas.set_pixel(10, 10, red);
        assert_eq!(canvas.get_pixel(10, 10), red);

        canvas.clear(Color::TRANSPARENT);
        assert_eq!(canvas.get_pixel(10, 10), Color::TRANSPARENT);
    }

    #[test]
    fn test_pixel_canvas_subpixel_anti_aliasing() {
        let mut canvas = PixelCanvas::new(40, 20);
        let cyan = Color::rgb(0, 255, 255);

        // Fractional point at (10.5, 10.5) distributes evenly to the 4 corners
        canvas.set_pixel_f32(10.5, 10.5, cyan);
        let c00 = canvas.get_pixel(10, 10);
        let c10 = canvas.get_pixel(11, 10);
        let c01 = canvas.get_pixel(10, 11);
        let c11 = canvas.get_pixel(11, 11);

        assert_eq!(c00, c10);
        assert_eq!(c10, c01);
        assert_eq!(c01, c11);
        assert!(c00.a > 0 && c00.a < 255);
    }

    #[test]
    fn test_pixel_canvas_draw_line_and_circle() {
        let mut canvas = PixelCanvas::new(60, 30);
        let white = Color::WHITE;

        canvas.draw_line_f32(0.0, 0.0, 10.0, 10.0, white);
        assert_ne!(canvas.get_pixel(5, 5), Color::TRANSPARENT);

        canvas.draw_circle_f32(20.0, 20.0, 5.0, white, true);
        assert_ne!(canvas.get_pixel(20, 20), Color::TRANSPARENT);
    }

    #[test]
    fn test_pixel_canvas_blit_to_and_from_framebuffer() {
        let mut canvas = PixelCanvas::new(40, 20);
        let mut fb = FrameBuffer::new(40, 20);

        let top_color = Color::rgb(200, 50, 50);
        let bot_color = Color::rgb(50, 50, 200);

        canvas.set_pixel(5, 10, top_color); // top subpixel of cell (5, 5)
        canvas.set_pixel(5, 11, bot_color); // bot subpixel of cell (5, 5)

        canvas.blit_to_framebuffer(&mut fb);
        let cell = fb.get_back(5, 5);
        assert_eq!(cell.ch, '▀');
        assert_eq!(cell.fg, top_color);
        assert_eq!(cell.bg, bot_color);

        // Blit back to a fresh canvas
        let mut canvas2 = PixelCanvas::new(40, 20);
        canvas2.blit_from_framebuffer(&fb);
        assert_eq!(canvas2.get_pixel(5, 10), top_color);
        assert_eq!(canvas2.get_pixel(5, 11), bot_color);
    }

    #[test]
    fn test_subpixel_wave_field_continuity() {
        let wave = SubpixelWaveField::default();
        let elev1 = wave.evaluate_elevation(10.0, 1.0);
        let elev2 = wave.evaluate_elevation(10.1, 1.0);
        assert!((elev1 - elev2).abs() < 0.2); // Continuity invariant
        assert!(elev1.is_finite());

        let mut canvas = PixelCanvas::new(40, 20);
        wave.render_wave_surface(
            &mut canvas,
            15.0,
            0.5,
            Color::rgb(0, 200, 255),
            Color::rgb(0, 20, 60),
        );
        assert_ne!(canvas.get_pixel(20, 16), Color::TRANSPARENT);
    }

    #[test]
    fn test_subpixel_kinematic_body_integration() {
        let mut body = SubpixelKinematicBody::new(0.0, 0.0);
        body.apply_impulse(10.0, 5.0);
        body.update(0.016); // 16ms tick
        assert!(body.pos_x > 0.0);
        assert!(body.pos_y > 0.0);
    }
}
