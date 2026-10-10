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
    pub last_cols: usize,
    pub last_rows: usize,
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
            last_cols: 0,
            last_rows: 0,
        }
    }

    /// Notify of terminal or pane resize.
    ///
    /// Immediately recomputes bounds limits and clamps coordinates
    /// so the creature snaps inside the visible viewport immediately.
    pub fn on_resize(&mut self, cols: usize, rows: usize) {
        self.last_cols = cols;
        self.last_rows = rows;
        let max_x = cols.saturating_sub(self.width) as f32;
        let max_y = rows.saturating_sub(self.height) as f32;
        self.x = self.x.clamp(0.0, max_x.max(0.0));
        self.y = self.y.clamp(0.0, max_y.max(0.0));
    }

    /// Advance kinematic physics by `dt` seconds given terminal dimensions.
    pub fn update(&mut self, dt: f32, total_time: f32, cols: usize, rows: usize) {
        let max_x = cols.saturating_sub(self.width) as f32;
        let max_y = rows.saturating_sub(self.height) as f32;

        if (self.last_cols > 0 && cols < self.last_cols)
            || (self.last_rows > 0 && rows < self.last_rows)
        {
            self.x = self.x.clamp(0.0, max_x.max(0.0));
            self.y = self.y.clamp(0.0, max_y.max(0.0));
        }
        self.last_cols = cols;
        self.last_rows = rows;

        match self.bounds_mode {
            BoundsMode::Lissajous {
                amp_x,
                amp_y,
                freq_x,
                freq_y,
                phase_x,
                phase_y,
            } => {
                let center_x = (max_x / 2.0).max(0.0);
                let center_y = (max_y / 2.0).max(0.0);

                let offset_x =
                    (total_time * freq_x * std::f32::consts::TAU + phase_x).sin() * amp_x;
                let offset_y =
                    (total_time * freq_y * std::f32::consts::TAU + phase_y).cos() * amp_y;

                self.x = (center_x + offset_x).clamp(0.0, max_x.max(0.0));
                self.y = (center_y + offset_y).clamp(0.0, max_y.max(0.0));
            }
            BoundsMode::Wrap => {
                self.vx += self.ax * dt;
                self.vy += self.ay * dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;

                self.facing_right = self.vx >= 0.0;

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

                self.y = self.y.clamp(0.0, max_y.max(0.0));
            }
            BoundsMode::Bounce => {
                self.vx += self.ax * dt;
                self.vy += self.ay * dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;

                let limit_x = max_x.max(0.0);
                let limit_y = max_y.max(0.0);

                if self.x <= 0.0 {
                    self.x = 0.0;
                    self.vx = self.vx.abs();
                    self.facing_right = true;
                } else if self.x >= limit_x {
                    self.x = limit_x;
                    self.vx = -self.vx.abs();
                    self.facing_right = false;
                }

                if self.y <= 0.0 {
                    self.y = 0.0;
                    self.vy = self.vy.abs();
                } else if self.y >= limit_y {
                    self.y = limit_y;
                    self.vy = -self.vy.abs();
                }
            }
            BoundsMode::Clamp => {
                self.vx += self.ax * dt;
                self.vy += self.ay * dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;

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
        let step = if step_length.abs() < 0.001 {
            1.0
        } else {
            step_length.abs()
        };
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

/// Anatomical segmentation landmarks for an ASCII character mesh.
/// Divides the creature into functional kinetic zones:
/// - Dialogue/Bubble Zone: lines before the animal art
/// - Cranial Zone: head, ears, horns, snout (bobs, rotates, nods)
/// - Thoracic/Spinal Zone: chest, spine, belly, flanks (breathes, undulates with spine wave)
/// - Caudal Zone: tail, wings, fins (follows through with phase-delayed pendulum inertia)
/// - Pedal Zone: legs, paws, hooves (coupled directly to ground displacement)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnatomicalSegments {
    pub bubble_lines: usize,
    pub cow_start: usize,
    pub head_row: usize,
    pub head_col: usize,
    pub chest_row: usize,
    pub leg_row: usize,
    pub tail_col: usize,
    pub width: usize,
    pub height: usize,
    pub faces_right: bool,
}

impl AnatomicalSegments {
    /// Detects anatomical segments from the composed ASCII scene and landmarks.
    #[must_use]
    pub fn detect(
        text: &str,
        cow_start: usize,
        eye_pos: Option<(usize, usize)>,
        mouth_pos: Option<(usize, usize)>,
        tail_pos: Option<(usize, usize)>,
        leg_row: usize,
    ) -> Self {
        let (mascot_canvas_width, mascot_canvas_height) = ascii_dimensions(text);
        let head_row = eye_pos
            .map(|(landmark_row, _)| landmark_row)
            .unwrap_or(cow_start.saturating_add(1));
        let head_col = eye_pos
            .map(|(_, landmark_col)| landmark_col)
            .unwrap_or(mascot_canvas_width / 4);
        let chest_row = mouth_pos
            .map(|(landmark_row, _)| landmark_row.saturating_add(1))
            .unwrap_or_else(|| (head_row + leg_row) / 2);
        let tail_col = tail_pos
            .map(|(_, landmark_col)| landmark_col)
            .unwrap_or(mascot_canvas_width.saturating_sub(4));
        let faces_right = head_col > tail_col;

        Self {
            bubble_lines: cow_start,
            cow_start,
            head_row,
            head_col,
            chest_row,
            leg_row,
            tail_col,
            width: mascot_canvas_width,
            height: mascot_canvas_height,
            faces_right,
        }
    }
}

/// Deformation offset for a single character cell at (x, y).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CellDeformation {
    /// Horizontal displacement in columns (-2 to +2).
    pub dx: i32,
    /// Vertical displacement in rows (-1 to +1).
    pub dy: i32,
    /// Character transformation (e.g. stretching / swelling glyph).
    pub alt_glyph: Option<char>,
    /// Subtle luminance / glow boost factor (1.0 = normal, 1.25 = peak, etc.).
    pub luminance_mult: f32,
}

/// Dynamic 2D cartoon rigging engine providing continuous multi-segment degrees of freedom.
/// Implements Disney 2D cartoon animation principles:
/// 1. Squash & Stretch: volumetric compression on ground contact and extension in air.
/// 2. Follow-Through: appendages (tail, ears) lag behind body motion with wave inertia.
/// 3. Secondary Action: spine harmonic wave and chest expansion.
/// 4. Decoupled Dialogue Physics: speech/thought bubbles float with elastic spring-damping.
#[derive(Debug, Clone)]
pub struct CartoonRig {
    pub segments: AnatomicalSegments,
    /// Spring-damper displacement for buoyant dialogue bubble
    pub bubble_offset_x: f32,
    pub bubble_vel_x: f32,
    pub bubble_offset_y: f32,
    pub bubble_vel_y: f32,
    /// Continuous phase variables
    pub elapsed: f32,
    pub squash_amplitude: f32,
    pub spine_amplitude: f32,
    pub tail_amplitude: f32,
    pub head_bob_amplitude: f32,
}

impl CartoonRig {
    #[must_use]
    pub fn new(segments: AnatomicalSegments) -> Self {
        Self {
            segments,
            bubble_offset_x: 0.0,
            bubble_vel_x: 0.0,
            bubble_offset_y: 0.0,
            bubble_vel_y: 0.0,
            elapsed: 0.0,
            squash_amplitude: 0.35,
            spine_amplitude: 0.60,
            tail_amplitude: 0.85,
            head_bob_amplitude: 0.45,
        }
    }

    /// Advance spring-damper bubble physics and harmonic phases by `dt` seconds.
    pub fn update(&mut self, dt: f32, mascot_vx: f32) {
        self.elapsed += dt;

        // 1. Elastic spring-damper for dialogue bubble:
        // When mascot moves with velocity mascot_vx, the bubble experiences inertial drag
        let target_x = -(mascot_vx * 0.08).clamp(-2.0, 2.0);
        let spring_k = 12.0; // Spring stiffness
        let damping_c = 4.5; // Damping ratio
        let force_x = spring_k * (target_x - self.bubble_offset_x) - damping_c * self.bubble_vel_x;
        self.bubble_vel_x += force_x * dt;
        self.bubble_offset_x += self.bubble_vel_x * dt;
        self.bubble_offset_x = self.bubble_offset_x.clamp(-3.0, 3.0);

        // Buoyant aerodynamic vertical floating on bubble:
        let float_freq = 2.4;
        let target_y = (self.elapsed * float_freq).sin() * 0.4;
        let force_y = spring_k * (target_y - self.bubble_offset_y) - damping_c * self.bubble_vel_y;
        self.bubble_vel_y += force_y * dt;
        self.bubble_offset_y += self.bubble_vel_y * dt;
        self.bubble_offset_y = self.bubble_offset_y.clamp(-1.0, 1.0);
    }

    /// Computes cartoon deformation for cell at (x, y) given animation state.
    #[must_use]
    pub fn deform_cell(
        &self,
        x: usize,
        y: usize,
        time: f32,
        stride: f32,
        is_walking: bool,
        is_floating: bool,
        is_aquatic: bool,
    ) -> CellDeformation {
        if time <= 0.0 && self.elapsed <= 0.0 {
            return CellDeformation {
                dx: 0,
                dy: 0,
                alt_glyph: None,
                luminance_mult: 1.0,
            };
        }

        // Zone 0: Dialogue Bubble (lines before cow_start)
        if y < self.segments.cow_start {
            return CellDeformation {
                dx: self.bubble_offset_x.round() as i32,
                dy: self.bubble_offset_y.round() as i32,
                alt_glyph: None,
                luminance_mult: 1.0,
            };
        }

        let animal_y = y.saturating_sub(self.segments.cow_start);
        let total_animal_h = self
            .segments
            .height
            .saturating_sub(self.segments.cow_start)
            .max(1);
        let vertical_height_ratio = (animal_y as f32 / total_animal_h as f32).clamp(0.0, 1.0);

        let mut dx_accum = 0.0f32;
        let mut dy_accum = 0.0f32;
        let alt_glyph = None;
        let mut luminance_boost = 1.0f32;

        if is_walking {
            // Principle 1: Squash and Stretch
            let cycle = stride * std::f32::consts::TAU * 2.0;
            let bounce_wave = cycle.sin();

            // Height squash/stretch: grounded at feet (vertical_height_ratio=1.0 has dy=0, vertical_height_ratio=0.0 has max dy)
            let vert_scale = (1.0 - vertical_height_ratio) * self.squash_amplitude;
            dy_accum += bounce_wave * vert_scale;

            // Width stretch: flanks expand when squashing down
            let x_from_center = (x as f32 - self.segments.width as f32 * 0.5)
                / (self.segments.width as f32 * 0.5).max(1.0);
            let horiz_stretch =
                -bounce_wave * 0.45 * x_from_center * (1.0 - vertical_height_ratio * 0.5);
            dx_accum += horiz_stretch;

            // Principle 2: Spine Secondary Action Wave
            let spine_phase = self.elapsed * 4.0 - animal_y as f32 * 0.35;
            let spine_wave =
                spine_phase.sin() * self.spine_amplitude * (1.0 - vertical_height_ratio * 0.4);
            dx_accum += spine_wave;

            // Principle 3: Cranial Zone (Head Bobbing)
            if y <= self.segments.head_row.saturating_add(1) {
                let head_bob =
                    (cycle + std::f32::consts::FRAC_PI_2).cos() * self.head_bob_amplitude;
                dy_accum += head_bob;
                let head_nod = (cycle + std::f32::consts::FRAC_PI_4).sin() * 0.3;
                dx_accum += head_nod;
            }

            // Principle 4: Caudal Zone (Tail Follow-Through)
            let is_near_tail = if self.segments.faces_right {
                x < self.segments.tail_col.saturating_add(3)
            } else {
                x > self.segments.tail_col.saturating_sub(3)
            };
            if is_near_tail && y >= self.segments.head_row {
                let tail_dist = (x as f32 - self.segments.tail_col as f32).abs();
                let tail_lag_phase = self.elapsed * 5.0 - tail_dist * 0.45;
                let tail_sway = tail_lag_phase.sin() * self.tail_amplitude;
                dx_accum += tail_sway;
            }
        } else if is_floating || is_aquatic {
            // Fluid/Buoyant Locomotion:
            let wave_freq = if is_aquatic { 3.2 } else { 2.0 };
            let undulation = (time * wave_freq + animal_y as f32 * 0.4).sin() * 0.65;
            dx_accum += undulation;

            let float_bob = (time * (wave_freq * 0.8) + (x as f32 * 0.15)).cos() * 0.4;
            dy_accum += float_bob;

            luminance_boost = 1.0 + (undulation.abs() * 0.25);
        } else {
            // Breathing / Idle Keep-Alive:
            let breath_phase = time * 2.2;
            let is_chest = y >= self.segments.head_row && y <= self.segments.leg_row;
            if is_chest {
                let chest_swell = breath_phase.sin() * 0.4;
                dx_accum += chest_swell * (1.0 - vertical_height_ratio * 0.5);
                luminance_boost = 1.0 + (breath_phase.sin() * 0.15);
            }
        }

        CellDeformation {
            dx: dx_accum.round() as i32,
            dy: dy_accum.round() as i32,
            alt_glyph,
            luminance_mult: luminance_boost,
        }
    }
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

    #[test]
    fn on_resize_clamps_position_immediately() {
        let mut body = KinematicBody::new(10, 5, BoundsMode::Wrap);
        body.x = 85.0;
        body.y = 20.0;
        body.on_resize(50, 15);
        // max_x = 50 - 10 = 40; max_y = 15 - 5 = 10
        assert_eq!(body.x, 40.0);
        assert_eq!(body.y, 10.0);
        assert_eq!(body.last_cols, 50);
        assert_eq!(body.last_rows, 15);
    }

    #[test]
    fn update_shrinking_clamps_across_all_modes() {
        // Wrap mode shrinking
        let mut body_wrap = KinematicBody::new(10, 5, BoundsMode::Wrap);
        body_wrap.update(0.1, 0.1, 100, 30);
        body_wrap.x = 80.0;
        body_wrap.y = 22.0;
        body_wrap.update(0.1, 0.2, 50, 15);
        assert!(body_wrap.x <= 40.0);
        assert!(body_wrap.y <= 10.0);

        // Lissajous mode shrinking
        let mut body_lis = KinematicBody::new(
            10,
            5,
            BoundsMode::Lissajous {
                amp_x: 20.0,
                amp_y: 10.0,
                freq_x: 1.0,
                freq_y: 1.0,
                phase_x: 0.0,
                phase_y: 0.0,
            },
        );
        body_lis.update(0.1, 0.1, 100, 30);
        body_lis.update(0.1, 0.2, 20, 8);
        assert!(body_lis.x <= 10.0);
        assert!(body_lis.y <= 3.0);
    }

    #[test]
    fn bounce_clamps_when_max_zero() {
        let mut body = KinematicBody::new(10, 5, BoundsMode::Bounce);
        body.x = 25.0;
        body.y = 12.0;
        body.vx = 5.0;
        body.vy = 5.0;
        // Viewport smaller than entity -> max_x = 0, max_y = 0
        body.update(0.1, 0.1, 8, 4);
        assert_eq!(body.x, 0.0);
        assert_eq!(body.y, 0.0);
    }

    #[test]
    fn test_cartoon_rig_segment_detection_and_deformation() {
        let text = " ______\n< hello >\n ------\n   \\   ^__^\n       (oo)\\_______\n        (__)\\       )\\/\\\n            ||----w |\n            ||     ||";
        let segs =
            AnatomicalSegments::detect(text, 3, Some((4, 8)), Some((5, 9)), Some((5, 20)), 7);
        assert_eq!(segs.cow_start, 3);
        assert_eq!(segs.head_row, 4);
        assert_eq!(segs.head_col, 8);
        assert_eq!(segs.leg_row, 7);

        let mut rig = CartoonRig::new(segs);
        rig.update(0.1, 10.0);
        // Bubble experiences inertial offset opposite to mascot movement direction
        assert!(rig.bubble_offset_x <= 0.0);

        // Dialogue bubble cell (y = 1) has spring displacement
        let b_def = rig.deform_cell(2, 1, 0.1, 0.25, true, false, false);
        assert_eq!(b_def.dx, rig.bubble_offset_x.round() as i32);

        // Animal cell (y = 4, head) deforms with head bob and secondary action
        let h_def = rig.deform_cell(8, 4, 0.1, 0.25, true, false, false);
        assert!(h_def.dx.abs() <= 2);
        assert!(h_def.dy.abs() <= 1);
    }
}
