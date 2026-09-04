//! Kinematics and continuous 2D motion engine for terminal animations.
//!
//! Benchmarked against `sl` (train traversal across terminal), `cmatrix` (fluid stream velocity),
//! and `donut.c` (continuous coordinate projection).
//!
//! Provides:
//! - Continuous 2D position, velocity, and acceleration integration.
//! - Bounds modes: Wrap (full traversal), Bounce (elastic reflection), Lissajous (orbital drift).
//! - Sub-pixel coordinates with discrete integer terminal grid quantization.
//! - Leg stride frequency coupled directly to horizontal ground displacement.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsMode {
    /// Entity wraps around terminal margins (leaves right edge, re-enters left edge).
    Wrap,
    /// Entity bounces elastically when hitting viewport borders.
    Bounce,
    /// Entity remains clamped within pasture bounds.
    Clamp,
    /// Harmonic 2D floating/drift around an equilibrium anchor.
    Lissajous {
        amp_x: f32,
        amp_y: f32,
        freq_x: f32,
        freq_y: f32,
        phase_x: f32,
        phase_y: f32,
    },
}

#[derive(Debug, Clone)]
pub struct KinematicBody {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ax: f32,
    pub ay: f32,
    pub width: usize,
    pub height: usize,
    pub bounds_mode: BoundsMode,
    pub facing_right: bool,
}

impl KinematicBody {
    #[must_use]
    pub fn new(width: usize, height: usize, bounds_mode: BoundsMode) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            ax: 0.0,
            ay: 0.0,
            width,
            height,
            bounds_mode,
            facing_right: true,
        }
    }

    /// Advance kinematic physics by `dt` seconds given terminal dimensions.
    pub fn update(&mut self, dt: f32, total_time: f32, cols: usize, rows: usize) {
        match self.bounds_mode {
            BoundsMode::Lissajous {
                amp_x,
                amp_y,
                freq_x,
                freq_y,
                phase_x,
                phase_y,
            } => {
                let max_x = cols.saturating_sub(self.width) as f32;
                let max_y = rows.saturating_sub(self.height) as f32;
                let center_x = (max_x / 2.0).max(0.0);
                let center_y = (max_y / 2.0).max(0.0);

                let offset_x = (total_time * freq_x * std::f32::consts::TAU + phase_x).sin() * amp_x;
                let offset_y = (total_time * freq_y * std::f32::consts::TAU + phase_y).cos() * amp_y;

                self.x = (center_x + offset_x).clamp(0.0, max_x.max(0.0));
                self.y = (center_y + offset_y).clamp(0.0, max_y.max(0.0));
            }
            BoundsMode::Wrap => {
                self.vx += self.ax * dt;
                self.vy += self.ay * dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;

                let entity_w = self.width as f32;
                let term_w = cols as f32;

                if self.vx >= 0.0 {
                    // Moving right: wraps from right edge to left edge
                    if self.x > term_w {
                        self.x = -entity_w;
                    }
                } else {
                    // Moving left: wraps from left edge to right edge
                    if self.x < -entity_w {
                        self.x = term_w;
                    }
                }

                let max_y = rows.saturating_sub(self.height) as f32;
                self.y = self.y.clamp(0.0, max_y);
            }
            BoundsMode::Bounce => {
                self.vx += self.ax * dt;
                self.vy += self.ay * dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;

                let max_x = cols.saturating_sub(self.width) as f32;
                let max_y = rows.saturating_sub(self.height) as f32;

                if self.x <= 0.0 {
                    self.x = 0.0;
                    self.vx = self.vx.abs();
                    self.facing_right = true;
                } else if self.x >= max_x && max_x > 0.0 {
                    self.x = max_x;
                    self.vx = -self.vx.abs();
                    self.facing_right = false;
                }

                if self.y <= 0.0 {
                    self.y = 0.0;
                    self.vy = self.vy.abs();
                } else if self.y >= max_y && max_y > 0.0 {
                    self.y = max_y;
                    self.vy = -self.vy.abs();
                }
            }
            BoundsMode::Clamp => {
                self.vx += self.ax * dt;
                self.vy += self.ay * dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;

                let max_x = cols.saturating_sub(self.width) as f32;
                let max_y = rows.saturating_sub(self.height) as f32;

                self.x = self.x.clamp(0.0, max_x.max(0.0));
                self.y = self.y.clamp(0.0, max_y.max(0.0));
            }
        }
    }

    /// Discrete integer terminal column anchor.
    #[must_use]
    pub fn screen_x(&self) -> i32 {
        self.x.round() as i32
    }

    /// Discrete integer terminal row anchor.
    #[must_use]
    pub fn screen_y(&self) -> i32 {
        self.y.round() as i32
    }

    /// Calculate stride phase [0.0, 1.0) coupled to ground displacement.
    /// Eliminates treadmill slipping.
    #[must_use]
    pub fn stride_phase(&self, step_length: f32) -> f32 {
        let step = if step_length.abs() < 0.001 { 1.0 } else { step_length.abs() };
        let phase = (self.x.abs() / step) % 1.0;
        if phase < 0.0 {
            phase + 1.0
        } else {
            phase
        }
    }
}

/// Computes (width, height) of multi-line ASCII text.
#[must_use]
pub fn ascii_dimensions(text: &str) -> (usize, usize) {
    let mut max_width = 0;
    let mut height = 0;
    for line in text.lines() {
        height += 1;
        let line_len = line.chars().count();
        if line_len > max_width {
            max_width = line_len;
        }
    }
    (max_width, height.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_calculated_correctly() {
        let text = " /\\_/\\\n( o.o )\n > ^ < ";
        let (w, h) = ascii_dimensions(text);
        assert_eq!(w, 7);
        assert_eq!(h, 3);
    }

    #[test]
    fn linear_wrap_traversal() {
        let mut body = KinematicBody::new(10, 5, BoundsMode::Wrap);
        body.vx = 20.0; // 20 cols per second
        body.update(1.0, 1.0, 80, 24);
        assert_eq!(body.screen_x(), 20);

        // Advance 4 more seconds -> x = 100 > term_w 80 -> wraps to -10
        body.update(4.0, 5.0, 80, 24);
        assert_eq!(body.screen_x(), -10);
    }

    #[test]
    fn bounce_inverts_velocity_at_margins() {
        let mut body = KinematicBody::new(10, 5, BoundsMode::Bounce);
        body.x = 65.0;
        body.vx = 10.0;
        // At max_x = 70 (80 - 10), advancing 1 sec hits 75, bounces to 70 and inverts vx
        body.update(1.0, 1.0, 80, 24);
        assert!(body.vx < 0.0);
        assert!(!body.facing_right);
    }

    #[test]
    fn stride_phase_coupled_to_displacement() {
        let mut body = KinematicBody::new(10, 5, BoundsMode::Wrap);
        body.x = 0.0;
        assert_eq!(body.stride_phase(4.0), 0.0);
        body.x = 2.0;
        assert_eq!(body.stride_phase(4.0), 0.5);
        body.x = 4.0;
        assert_eq!(body.stride_phase(4.0), 0.0);
    }
}
