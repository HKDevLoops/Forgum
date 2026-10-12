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
                crate::gpu_compute::get_compute_engine(None).compute_dynamic_kinematics_step(
                    total_time,
                    cols,
                    rows,
                    self.width,
                    self.height,
                    amp_x,
                    amp_y,
                    freq_x,
                    freq_y,
                    phase_x,
                    phase_y,
                    &mut self.x,
                    &mut self.y,
                );
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

/// Richard Williams' 8-Phase Quadruped Gait Cycle.
///
/// Grounded in *The Animator's Survival Kit* (Quadruped & Biped Gait Cycles):
/// - Phase 0: Left Hind Strike & Right Fore Push-Off
/// - Phase 1: Left Hind Stance & Right Fore Swing
/// - Phase 2: Left Fore Strike & Left Hind Stance
/// - Phase 3: Left Fore Stance & Left Hind Push-Off
/// - Phase 4: Right Hind Strike & Left Fore Push-Off
/// - Phase 5: Right Hind Stance & Left Fore Swing
/// - Phase 6: Right Fore Strike & Right Hind Stance
/// - Phase 7: Right Fore Stance & Right Hind Push-Off
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadrupedPhase {
    Phase0LeftHindStrikeRightForePushOff,
    Phase1LeftHindStanceRightForeSwing,
    Phase2LeftForeStrikeLeftHindStance,
    Phase3LeftForeStanceLeftHindPushOff,
    Phase4RightHindStrikeLeftForePushOff,
    Phase5RightHindStanceLeftForeSwing,
    Phase6RightForeStrikeRightHindStance,
    Phase7RightForeStanceRightHindPushOff,
}

impl QuadrupedPhase {
    #[must_use]
    pub fn from_stride(stride: f32) -> Self {
        let normalized = stride.rem_euclid(1.0);
        let phase_idx = ((normalized * 8.0).floor() as usize) % 8;
        match phase_idx {
            0 => Self::Phase0LeftHindStrikeRightForePushOff,
            1 => Self::Phase1LeftHindStanceRightForeSwing,
            2 => Self::Phase2LeftForeStrikeLeftHindStance,
            3 => Self::Phase3LeftForeStanceLeftHindPushOff,
            4 => Self::Phase4RightHindStrikeLeftForePushOff,
            5 => Self::Phase5RightHindStanceLeftForeSwing,
            6 => Self::Phase6RightForeStrikeRightHindStance,
            _ => Self::Phase7RightForeStanceRightHindPushOff,
        }
    }

    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Phase0LeftHindStrikeRightForePushOff => 0,
            Self::Phase1LeftHindStanceRightForeSwing => 1,
            Self::Phase2LeftForeStrikeLeftHindStance => 2,
            Self::Phase3LeftForeStanceLeftHindPushOff => 3,
            Self::Phase4RightHindStrikeLeftForePushOff => 4,
            Self::Phase5RightHindStanceLeftForeSwing => 5,
            Self::Phase6RightForeStrikeRightHindStance => 6,
            Self::Phase7RightForeStanceRightHindPushOff => 7,
        }
    }

    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::Phase0LeftHindStrikeRightForePushOff => "Left Hind Strike & Right Fore Push-Off",
            Self::Phase1LeftHindStanceRightForeSwing => "Left Hind Stance & Right Fore Swing",
            Self::Phase2LeftForeStrikeLeftHindStance => "Left Fore Strike & Left Hind Stance",
            Self::Phase3LeftForeStanceLeftHindPushOff => "Left Fore Stance & Left Hind Push-Off",
            Self::Phase4RightHindStrikeLeftForePushOff => "Right Hind Strike & Left Fore Push-Off",
            Self::Phase5RightHindStanceLeftForeSwing => "Right Hind Stance & Left Fore Swing",
            Self::Phase6RightForeStrikeRightHindStance => "Right Fore Strike & Right Hind Stance",
            Self::Phase7RightForeStanceRightHindPushOff => {
                "Right Fore Stance & Right Hind Push-Off"
            }
        }
    }
}

/// Partitions detected leg columns into Left/Right Hind and Left/Right Fore limbs.
/// Implements mammalian quadruped limb decoupling with 90-degree phase shift (0.25 stride lag).
#[derive(Debug, Clone, PartialEq)]
pub struct QuadrupedLegPartition {
    pub hind_left: Vec<usize>,
    pub hind_right: Vec<usize>,
    pub fore_left: Vec<usize>,
    pub fore_right: Vec<usize>,
    pub is_quadruped: bool,
}

impl QuadrupedLegPartition {
    /// Partitions detected leg columns into Hind and Fore legs with lateral decoupling.
    #[must_use]
    pub fn partition(leg_cols: &[usize], head_col: usize, tail_col: usize) -> Self {
        if leg_cols.is_empty() {
            return Self {
                hind_left: Vec::new(),
                hind_right: Vec::new(),
                fore_left: Vec::new(),
                fore_right: Vec::new(),
                is_quadruped: false,
            };
        }

        let faces_right = head_col > tail_col;

        // If only 1 or 2 columns, treat as biped or simple leg pair
        if leg_cols.len() <= 2 {
            let left = if !leg_cols.is_empty() {
                vec![leg_cols[0]]
            } else {
                Vec::new()
            };
            let right = if leg_cols.len() > 1 {
                vec![leg_cols[1]]
            } else {
                Vec::new()
            };
            return Self {
                hind_left: left.clone(),
                hind_right: right.clone(),
                fore_left: left,
                fore_right: right,
                is_quadruped: false,
            };
        }

        // For 3+ columns, find the spatial gap separating Hind from Fore limbs
        let mut max_gap = 0;
        let mut split_idx = leg_cols.len() / 2;
        for i in 0..leg_cols.len().saturating_sub(1) {
            let gap = leg_cols[i + 1].saturating_sub(leg_cols[i]);
            if gap > max_gap && gap >= 2 {
                max_gap = gap;
                split_idx = i + 1;
            }
        }

        let group_low = &leg_cols[..split_idx];
        let group_high = &leg_cols[split_idx..];

        // If faces_right: lower columns are Hind (rear), higher columns are Fore (front)
        // If !faces_right: lower columns are Fore (front), higher columns are Hind (rear)
        let (hind_slice, fore_slice) = if faces_right {
            (group_low, group_high)
        } else {
            (group_high, group_low)
        };

        let split_group = |slice: &[usize]| -> (Vec<usize>, Vec<usize>) {
            if slice.is_empty() {
                (Vec::new(), Vec::new())
            } else if slice.len() == 1 {
                (vec![slice[0]], vec![slice[0]])
            } else {
                let mid = slice.len() / 2;
                (slice[..mid].to_vec(), slice[mid..].to_vec())
            }
        };

        let (hl, hr) = split_group(hind_slice);
        let (fl, fr) = split_group(fore_slice);

        Self {
            hind_left: hl,
            hind_right: hr,
            fore_left: fl,
            fore_right: fr,
            is_quadruped: true,
        }
    }

    /// Resolves which limb a column belongs to, returning (phase_offset, is_outer).
    /// Lateral sequence phase offsets:
    /// - Left Hind (LH): 0 (Phase 0)
    /// - Left Fore (LF): 2 (Phase 2, 90-degree / 0.25 stride lag after LH)
    /// - Right Hind (RH): 4 (Phase 4, 180-degree / 0.50 stride lag after LH)
    /// - Right Fore (RF): 6 (Phase 6, 270-degree / 0.75 stride lag after LH)
    #[must_use]
    pub fn resolve_limb_offset(&self, col: usize) -> (usize, bool) {
        if !self.is_quadruped {
            if self.hind_left.contains(&col) {
                return (0, false);
            }
            return (4, true);
        }

        if self.hind_left.contains(&col) {
            (0, false)
        } else if self.fore_left.contains(&col) {
            (2, false)
        } else if self.hind_right.contains(&col) {
            (4, true)
        } else if self.fore_right.contains(&col) {
            (6, true)
        } else {
            (0, false)
        }
    }
}

/// Articulated leg glyph resolution for 8-phase quadruped and biped locomotion.
///
/// Uses rich articulated characters: '|', '/', '\\', '_', ')', '(', 'J', 'L', 'd', 'b'
/// representing knee flexion, ankle push-off, and paw/hoof strike.
#[must_use]
pub fn articulated_leg_glyph(
    stage: usize,
    faces_right: bool,
    is_moonwalk: bool,
    is_outer_leg: bool,
) -> char {
    if is_moonwalk {
        return match stage % 4 {
            0 if is_outer_leg => '\\',
            2 if is_outer_leg => '/',
            _ => '|',
        };
    }

    if faces_right {
        match stage % 8 {
            0 => {
                if is_outer_leg {
                    'L'
                } else {
                    '\\'
                }
            } // Strike: hoof/paw contact forward
            1 => '|', // Landing: full vertical stance
            2 => '|', // Mid-Stance: vertical weight-bearing
            3 => '/', // Late Stance: ankle extending back
            4 => {
                if is_outer_leg {
                    'b'
                } else {
                    '/'
                }
            } // Push-Off: ankle drive / push-off
            5 => ')', // Lift/Recoil: knee flexion in air
            6 => '\\', // Mid-Swing: forward swing
            _ => {
                if is_outer_leg {
                    'L'
                } else {
                    '\\'
                }
            } // Terminal Swing: forward reach
        }
    } else {
        match stage % 8 {
            0 => {
                if is_outer_leg {
                    'J'
                } else {
                    '/'
                }
            } // Strike: hoof/paw contact forward
            1 => '|',  // Landing: full vertical stance
            2 => '|',  // Mid-Stance: vertical weight-bearing
            3 => '\\', // Late Stance: ankle extending back
            4 => {
                if is_outer_leg {
                    'd'
                } else {
                    '\\'
                }
            } // Push-Off: ankle drive / push-off
            5 => '(',  // Lift/Recoil: knee flexion in air
            6 => '/',  // Mid-Swing: forward swing
            _ => {
                if is_outer_leg {
                    'J'
                } else {
                    '/'
                }
            } // Terminal Swing: forward reach
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

        crate::gpu_compute::get_compute_engine(None).compute_dynamic_bubble_inertia(
            dt,
            self.elapsed,
            mascot_vx,
            &mut self.bubble_offset_x,
            &mut self.bubble_vel_x,
            &mut self.bubble_offset_y,
            &mut self.bubble_vel_y,
        );
    }

    /// Calculate continuous vertical center-of-mass bobbing: \Delta y = A \sin(4\pi \cdot \text{stride}).
    /// Applied uniformly across the mascot body to strictly eliminate integer cell tearing.
    #[must_use]
    pub fn pelvic_bounce(&self, stride: f32) -> f32 {
        (stride * 4.0 * std::f32::consts::PI).sin() * self.squash_amplitude
    }

    /// Calculate cranial head bobbing with pendular phase lag (\Delta \phi \approx \pi/4).
    /// Returns (horizontal_nod, vertical_lag).
    #[must_use]
    pub fn head_bob(&self, stride: f32) -> (f32, f32) {
        let pelvic_cycle = stride * 4.0 * std::f32::consts::PI;
        let head_lag = pelvic_cycle - std::f32::consts::FRAC_PI_4;
        let nod = head_lag.sin() * (self.head_bob_amplitude * 0.7);
        let dip = -head_lag.cos() * (self.head_bob_amplitude * 0.25);
        (nod, dip)
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
            // Principle 1: Continuous Non-Tearing Vertical Pelvic Bounce
            // \Delta y = A \sin(4\pi \cdot \text{stride})
            // Applied uniformly across the entire mascot body to strictly ELIMINATE integer rounding cell tearing.
            let pelvic_cycle = stride * 4.0 * std::f32::consts::PI;
            let pelvic_dy = pelvic_cycle.sin() * self.squash_amplitude;
            dy_accum += pelvic_dy;

            // Width stretch: subtle flank expansion when squashing down
            let x_from_center = (x as f32 - self.segments.width as f32 * 0.5)
                / (self.segments.width as f32 * 0.5).max(1.0);
            let horiz_stretch = -pelvic_dy * 0.35 * x_from_center;
            dx_accum += horiz_stretch;

            // Principle 2: Spine Secondary Action Wave
            let spine_phase = self.elapsed * 4.0 - animal_y as f32 * 0.25;
            let spine_wave = spine_phase.sin() * self.spine_amplitude * 0.25;
            dx_accum += spine_wave;

            // Principle 3: Cranial Zone (Head Bob with Inertia Lag)
            // Head bobs with pendular phase lag (\Delta \phi \approx \pi/4), gently dipping on recoil and lifting on push-off.
            if y <= self.segments.head_row.saturating_add(1) {
                let head_lag = pelvic_cycle - std::f32::consts::FRAC_PI_4;
                let head_nod = head_lag.sin() * (self.head_bob_amplitude * 0.7);
                dx_accum += head_nod;
            }

            // Principle 4: Caudal Zone (Tail Follow-Through)
            // Pendular wave trailing body motion
            let is_near_tail = if self.segments.faces_right {
                x <= self.segments.tail_col.saturating_add(3)
            } else {
                x >= self.segments.tail_col.saturating_sub(3)
            };
            if is_near_tail && y >= self.segments.head_row {
                let tail_dist = (x as f32 - self.segments.tail_col as f32).abs();
                let tail_wave_phase =
                    self.elapsed * 5.0 - tail_dist * 0.45 + (stride * std::f32::consts::TAU);
                let tail_sway = tail_wave_phase.sin() * self.tail_amplitude;
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

    #[test]
    fn test_quadruped_8_phase_gait_cycle() {
        for idx in 0..8 {
            let stride = idx as f32 / 8.0 + 0.01;
            let phase = QuadrupedPhase::from_stride(stride);
            assert_eq!(phase.index(), idx);
        }

        // Verify all 8 Richard Williams quadruped gait phases
        assert_eq!(
            QuadrupedPhase::Phase0LeftHindStrikeRightForePushOff.description(),
            "Left Hind Strike & Right Fore Push-Off"
        );
        assert_eq!(
            QuadrupedPhase::Phase1LeftHindStanceRightForeSwing.description(),
            "Left Hind Stance & Right Fore Swing"
        );
        assert_eq!(
            QuadrupedPhase::Phase2LeftForeStrikeLeftHindStance.description(),
            "Left Fore Strike & Left Hind Stance"
        );
        assert_eq!(
            QuadrupedPhase::Phase3LeftForeStanceLeftHindPushOff.description(),
            "Left Fore Stance & Left Hind Push-Off"
        );
        assert_eq!(
            QuadrupedPhase::Phase4RightHindStrikeLeftForePushOff.description(),
            "Right Hind Strike & Left Fore Push-Off"
        );
        assert_eq!(
            QuadrupedPhase::Phase5RightHindStanceLeftForeSwing.description(),
            "Right Hind Stance & Left Fore Swing"
        );
        assert_eq!(
            QuadrupedPhase::Phase6RightForeStrikeRightHindStance.description(),
            "Right Fore Strike & Right Hind Stance"
        );
        assert_eq!(
            QuadrupedPhase::Phase7RightForeStanceRightHindPushOff.description(),
            "Right Fore Stance & Right Hind Push-Off"
        );
    }

    #[test]
    fn test_quadruped_leg_partition_lateral_sequence() {
        // Standard quadruped with 4 leg strokes: columns 16, 17 (fore) and 23, 24 (hind), facing left
        let leg_cols = vec![16, 17, 23, 24];
        let partition = QuadrupedLegPartition::partition(&leg_cols, 8, 24);
        assert!(partition.is_quadruped);
        assert_eq!(partition.fore_left, vec![16]);
        assert_eq!(partition.fore_right, vec![17]);
        assert_eq!(partition.hind_left, vec![23]);
        assert_eq!(partition.hind_right, vec![24]);

        // Lateral walk sequence phase offsets (0.25 stride / 90 degree increments):
        // LH = 0, LF = 2, RH = 4, RF = 6
        assert_eq!(partition.resolve_limb_offset(23).0, 0); // LH
        assert_eq!(partition.resolve_limb_offset(16).0, 2); // LF (90 deg lag)
        assert_eq!(partition.resolve_limb_offset(24).0, 4); // RH (180 deg lag)
        assert_eq!(partition.resolve_limb_offset(17).0, 6); // RF (270 deg lag)

        // Articulated leg glyphs across phases
        let lh_strike = articulated_leg_glyph(0, false, false, false);
        let stance = articulated_leg_glyph(1, false, false, false);
        assert!(matches!(lh_strike, '/' | 'J'));
        assert_eq!(stance, '|');
    }

    #[test]
    fn test_uniform_pelvic_bounce_strictly_non_tearing() {
        let text = " ______\n< hello >\n ------\n   \\   ^__^\n       (oo)\\_______\n        (__)\\       )\\/\\\n            ||----w |\n            ||     ||";
        let segs =
            AnatomicalSegments::detect(text, 3, Some((4, 8)), Some((5, 9)), Some((5, 20)), 7);
        let rig = CartoonRig::new(segs);

        // Check across 16 points in the walk cycle
        for step in 0..16 {
            let stride = step as f32 / 16.0;
            let head_def = rig.deform_cell(8, 4, 1.0, stride, true, false, false);
            let torso_def = rig.deform_cell(14, 5, 1.0, stride, true, false, false);
            let flank_def = rig.deform_cell(18, 6, 1.0, stride, true, false, false);

            // Vertical pelvic bounce MUST be identical across body rows to eliminate cell tearing!
            assert_eq!(
                head_def.dy, torso_def.dy,
                "Head and torso dy must be equal to eliminate line tearing at stride {stride}"
            );
            assert_eq!(
                torso_def.dy, flank_def.dy,
                "Torso and flank dy must be equal to eliminate line tearing at stride {stride}"
            );
        }
    }
}
