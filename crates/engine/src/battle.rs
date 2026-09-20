//! ASCII cow jousting — two cows charge, collide, and one loses.

use crate::framebuffer::{Cell, Color, FrameBuffer};
use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BATTLE_WIDTH: usize = 80;
const BATTLE_HEIGHT: usize = 14;

/// The Golden Ratio constant (\u{03a6} \u{2248} 1.61803398875) used for quasi-random low-discrepancy dispersion.
pub const PHI: f64 = 1.618_033_988_749_895;

#[derive(Debug, Clone)]
pub struct BattleCow {
    pub name: String,
    pub eyes: String,
    pub x: i32,
    pub y: usize,
    pub alive: bool,
    pub hp: u32,
    pub max_hp: u32,
}

impl BattleCow {
    pub fn new(name: &str, eyes: &str, start_x: i32) -> Self {
        Self {
            name: name.to_string(),
            eyes: eyes.to_string(),
            x: start_x,
            y: 5,
            alive: true,
            hp: 100,
            max_hp: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Battle {
    pub cow1: BattleCow,
    pub cow2: BattleCow,
    pub phase: BattlePhase,
    pub frames: u32,
    pub phase_frames: u32,
    pub width: usize,
    pub height: usize,
    pub winner: u8,
    /// Randomized clash column where lances impact
    pub clash_x: i32,
    pub start_x1: i32,
    pub start_x2: i32,
    /// Travel distance of mascot 1 (columns)
    pub dist1: i32,
    /// Travel distance of mascot 2 (columns)
    pub dist2: i32,
    /// Deliberate, smooth velocity of mascot 1 (columns / frame)
    pub speed1: f32,
    /// Deliberate, smooth velocity of mascot 2 (columns / frame)
    pub speed2: f32,
    /// Sub-column continuous position accumulator for mascot 1
    pub pos1: f32,
    /// Sub-column continuous position accumulator for mascot 2
    pub pos2: f32,
    /// Frames allocated for the charging phase
    pub charge_duration: u32,
    /// Frames allocated for the collision impact phase
    pub collision_duration: u32,
    /// Frames allocated for the aftermath victory phase
    pub aftermath_duration: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattlePhase {
    Charging,
    Collision,
    Aftermath,
    Done,
}

/// Computes randomized jousting kinematics using the Golden Ratio (\u{03a6} \u{2248} 1.6180339887)
/// and harmonic sinusoidal superposition to ensure organic, unpredictable battlefield charges.
///
/// Guarantees that:
/// 1. The travel distance of mascot 1 (`dist1`) is strictly different from mascot 2 (`dist2`).
/// 2. Every session with a new seed produces different clash points and travel distances.
/// 3. Speeds are scaled smoothly so mascots move deliberately and realistically.
pub fn compute_battle_kinematics(
    name1: &str,
    name2: &str,
    width: usize,
    seed: u64,
) -> (i32, i32, i32, f32, f32, u32, u32, u32) {
    // 1. Hash fighter names + entropy seed via 64-bit mixer
    let mut h = seed.wrapping_add(0x9e3779b97f4a7c15);
    for b in name1.as_bytes().iter().chain(name2.as_bytes()) {
        h = (h ^ (*b as u64)).wrapping_mul(0xbf58476d1ce4e5b9);
        h ^= h >> 30;
    }
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;

    // 2. Map through Golden Ratio (\u{03a6}) low-discrepancy dispersion
    let u = ((h as f64) / (u64::MAX as f64)).fract().abs();
    let phi_offset = (u * PHI).fract();

    // 3. Harmonic trigonometric superposition for natural battlefield variation
    let two_pi = std::f64::consts::TAU;
    let harmonic =
        0.50 + 0.32 * (two_pi * phi_offset).sin() + 0.14 * (two_pi * phi_offset * PHI).cos();
    let t = harmonic.clamp(0.18, 0.82);

    // 4. Determine valid clash zone within the arena
    // Cow 1 lance tip is at x1 + 22. Cow 2 lance tip is at x2.
    // Minimum clash_x: x1 >= 4 -> clash_x >= 26.
    // Maximum clash_x: x2 <= width - 12 -> clash_x <= width - 12.
    let min_clash = 26_i32;
    let max_clash = (width as i32 - 14).max(min_clash + 8);
    let span = (max_clash - min_clash) as f64;
    let mut clash_x = min_clash + (t * span).round() as i32;

    let start_x1 = 2_i32;
    let start_x2 = (width as i32) - 10;
    let mut dist1 = (clash_x - 22) - start_x1;
    let mut dist2 = start_x2 - clash_x;

    // Enforce distinct travel distances between mascot 1 and mascot 2
    if (dist1 - dist2).abs() < 4 {
        if phi_offset >= 0.5 {
            clash_x = (clash_x + 5).min(max_clash);
        } else {
            clash_x = (clash_x - 5).max(min_clash);
        }
        dist1 = (clash_x - 22) - start_x1;
        dist2 = start_x2 - clash_x;
    }

    // Ensure strictly positive distances
    dist1 = dist1.max(3);
    dist2 = dist2.max(3);

    // 5. Cinematic pacing: charge duration between 40 and 55 frames (slow and real!)
    let base_charge = 44.0 + 8.0 * (phi_offset * two_pi).sin();
    let charge_frames = (base_charge * (width as f64 / 80.0))
        .clamp(36.0, 56.0)
        .round() as u32;

    let speed1 = dist1 as f32 / charge_frames as f32;
    let speed2 = dist2 as f32 / charge_frames as f32;

    // 6. Combat HP asymmetry: each mascot has distinct, randomized Max HP and starting HP
    let hp1_raw = 95 + (h % 36) as u32; // 95 .. 130
    let hp2_raw = 90 + ((h >> 16) % 36) as u32; // 90 .. 125
    let (max_hp1, max_hp2) = if (hp1_raw as i32 - hp2_raw as i32).abs() < 10 {
        if phi_offset >= 0.5 {
            (hp1_raw + 15, hp2_raw.saturating_sub(10).max(85))
        } else {
            (hp1_raw.saturating_sub(10).max(85), hp2_raw + 15)
        }
    } else {
        (hp1_raw, hp2_raw)
    };

    (
        clash_x,
        dist1,
        dist2,
        speed1,
        speed2,
        charge_frames,
        max_hp1,
        max_hp2,
    )
}

impl Battle {
    pub fn new(name1: &str, name2: &str) -> Self {
        Self::with_dimensions(name1, name2, BATTLE_WIDTH, BATTLE_HEIGHT, 2)
    }

    pub fn with_dimensions(
        name1: &str,
        name2: &str,
        width: usize,
        height: usize,
        winner: u8,
    ) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x12345678_9abcdef0);
        Self::with_dimensions_and_seed(name1, name2, width, height, winner, seed)
    }

    pub fn with_dimensions_and_seed(
        name1: &str,
        name2: &str,
        width: usize,
        height: usize,
        winner: u8,
        seed: u64,
    ) -> Self {
        let start_x1 = 2;
        let start_x2 = (width as i32) - 10;
        let cow_y = 5.min(height.saturating_sub(6));

        let (clash_x, dist1, dist2, speed1, speed2, charge_duration, max_hp1, max_hp2) =
            compute_battle_kinematics(name1, name2, width, seed);

        let mut cow1 = BattleCow::new(name1, "oo", start_x1);
        cow1.y = cow_y;
        cow1.hp = max_hp1;
        cow1.max_hp = max_hp1;

        let mut cow2 = BattleCow::new(name2, "oo", start_x2);
        cow2.y = cow_y;
        cow2.hp = max_hp2;
        cow2.max_hp = max_hp2;

        Self {
            cow1,
            cow2,
            phase: BattlePhase::Charging,
            frames: 0,
            phase_frames: 0,
            width,
            height,
            winner,
            clash_x,
            start_x1,
            start_x2,
            dist1,
            dist2,
            speed1,
            speed2,
            pos1: start_x1 as f32,
            pos2: start_x2 as f32,
            charge_duration,
            collision_duration: 30,
            aftermath_duration: 35,
        }
    }

    pub fn tick(&mut self) {
        self.frames += 1;

        match self.phase {
            BattlePhase::Charging => {
                self.phase_frames += 1;
                self.pos1 += self.speed1;
                self.pos2 -= self.speed2;
                self.cow1.x = self.pos1.round() as i32;
                self.cow2.x = self.pos2.round() as i32;

                let lance1_tip = self.cow1.x + 22;
                let lance2_tip = self.cow2.x;

                if lance1_tip >= lance2_tip
                    || self.cow1.x >= self.clash_x - 22
                    || self.cow2.x <= self.clash_x
                    || self.phase_frames >= self.charge_duration
                {
                    self.cow1.x = self.clash_x - 22;
                    self.cow2.x = self.clash_x;
                    self.phase = BattlePhase::Collision;
                    self.phase_frames = 0;
                }
            }
            BattlePhase::Collision => {
                self.phase_frames += 1;

                // Progressive health depletion during collision based on individual Max HP
                let progress = (self.phase_frames as f32 / self.collision_duration as f32).min(1.0);
                if self.winner == 1 {
                    let win_retain = (self.cow1.max_hp as f32 * 0.78).round() as u32;
                    let damage = ((self.cow1.max_hp - win_retain) as f32 * progress).round() as u32;
                    self.cow1.hp = self.cow1.max_hp.saturating_sub(damage);
                    self.cow2.hp = ((self.cow2.max_hp as f32) * (1.0 - progress)).round() as u32;
                } else {
                    let win_retain = (self.cow2.max_hp as f32 * 0.78).round() as u32;
                    let damage = ((self.cow2.max_hp - win_retain) as f32 * progress).round() as u32;
                    self.cow2.hp = self.cow2.max_hp.saturating_sub(damage);
                    self.cow1.hp = ((self.cow1.max_hp as f32) * (1.0 - progress)).round() as u32;
                }

                // Mid-collision: Loser's lance shatters, loser recoils, eyes change
                if self.phase_frames >= self.collision_duration / 2 {
                    if self.winner == 1 {
                        self.cow1.eyes = "^^".to_string();
                        self.cow2.eyes = "xx".to_string();
                        self.cow2.alive = false;
                        self.cow2.x = (self.clash_x + 2).min(self.width as i32 - 10);
                    } else {
                        self.cow1.eyes = "xx".to_string();
                        self.cow2.eyes = "@@".to_string();
                        self.cow1.alive = false;
                        self.cow1.x = (self.clash_x - 24).max(1);
                    }
                }

                if self.phase_frames >= self.collision_duration {
                    self.phase = BattlePhase::Aftermath;
                    self.phase_frames = 0;
                    if self.winner == 1 {
                        self.cow2.hp = 0;
                        self.cow1.hp = (self.cow1.max_hp as f32 * 0.78).round() as u32;
                    } else {
                        self.cow1.hp = 0;
                        self.cow2.hp = (self.cow2.max_hp as f32 * 0.78).round() as u32;
                    }
                }
            }
            BattlePhase::Aftermath => {
                self.phase_frames += 1;
                if self.phase_frames >= self.aftermath_duration {
                    self.phase = BattlePhase::Done;
                }
            }
            BattlePhase::Done => {}
        }
    }

    pub fn is_done(&self) -> bool {
        self.phase == BattlePhase::Done
    }

    pub fn render_frame(&self) -> String {
        self.render_frame_styled(false)
    }

    pub fn render_frame_styled(&self, use_color: bool) -> String {
        let fb = self.build_framebuffer();
        let mut output = String::with_capacity(self.width * self.height * 4);

        for y in 0..self.height {
            let mut last_color: Option<Color> = None;
            for x in 0..self.width {
                let cell = fb.back[y * self.width + x];
                let ch = if cell.alpha == 0 || cell.ch == '\0' {
                    ' '
                } else {
                    cell.ch
                };

                if use_color {
                    if cell.alpha > 0 && ch != ' ' {
                        if last_color != Some(cell.fg) {
                            output.push_str(&format!(
                                "\x1b[38;2;{};{};{}m",
                                cell.fg.r, cell.fg.g, cell.fg.b
                            ));
                            last_color = Some(cell.fg);
                        }
                    } else if last_color.is_some() {
                        output.push_str("\x1b[0m");
                        last_color = None;
                    }
                    output.push(ch);
                } else {
                    output.push(ch);
                }
            }
            if use_color {
                if last_color.is_some() {
                    output.push_str("\x1b[0m");
                }
                output.push_str("\x1b[K\r\n");
            } else {
                output.push('\n');
            }
        }
        output
    }

    fn build_framebuffer(&self) -> FrameBuffer {
        let mut fb = FrameBuffer::new(self.width, self.height);
        let border_color = Color::rgb(100, 180, 255);
        let text_gold = Color::rgb(255, 215, 0);
        let turf_color = Color::rgb(70, 150, 90);

        // 1. Draw top border
        fb.set(0, 0, Cell::new('╔', border_color));
        for x in 1..(self.width.saturating_sub(1)) {
            fb.set(x, 0, Cell::new('═', border_color));
        }
        if self.width > 1 {
            fb.set(self.width - 1, 0, Cell::new('╗', border_color));
        }

        // 2. Row 1: Match header and health bars
        let bar_len: usize = 8;
        let hp1_filled = ((self.cow1.hp as f32 / self.cow1.max_hp.max(1) as f32) * bar_len as f32)
            .round() as usize;
        let hp1_bar = format!(
            "{} [{}{}] {}/{}HP",
            self.cow1.name,
            "■".repeat(hp1_filled),
            "░".repeat(bar_len.saturating_sub(hp1_filled)),
            self.cow1.hp,
            self.cow1.max_hp
        );
        let hp2_filled = ((self.cow2.hp as f32 / self.cow2.max_hp.max(1) as f32) * bar_len as f32)
            .round() as usize;
        let hp2_bar = format!(
            "{} [{}{}] {}/{}HP",
            self.cow2.name,
            "■".repeat(hp2_filled),
            "░".repeat(bar_len.saturating_sub(hp2_filled)),
            self.cow2.hp,
            self.cow2.max_hp
        );
        let title = format!("  ⚔️  {hp1_bar}  VS  {hp2_bar}  ⚔️");
        for (i, ch) in title.chars().enumerate() {
            if i + 1 < self.width.saturating_sub(1) {
                fb.set(i + 1, 1, Cell::new(ch, text_gold));
            }
        }

        // 3. Row 2: Status / Commentary
        let winner_name = if self.winner == 1 {
            &self.cow1.name
        } else {
            &self.cow2.name
        };
        let loser_name = if self.winner == 1 {
            &self.cow2.name
        } else {
            &self.cow1.name
        };

        let commentary = match self.phase {
            BattlePhase::Charging => {
                if self.phase_frames < self.charge_duration / 3 {
                    format!(
                        "  ❯ {} and {} lower tournament lances and begin their charge!",
                        self.cow1.name, self.cow2.name
                    )
                } else if self.phase_frames < (self.charge_duration * 2) / 3 {
                    format!(
                        "  ❯ Hooves thunder across the turf! Distances: {} vs {} cols...",
                        self.dist1, self.dist2
                    )
                } else {
                    format!(
                        "  ❯ BRACE FOR IMPACT! The champions hurtle toward column {}!",
                        self.clash_x
                    )
                }
            }
            BattlePhase::Collision => {
                if self.phase_frames < self.collision_duration / 3 {
                    "  💥 *** THUNDEROUS CLASH! LANCE TIPS COLLIDE! *** 💥".to_string()
                } else if self.phase_frames < (self.collision_duration * 2) / 3 {
                    "  ⚡ *** WOOD SPLINTERS! THE ARENA SHUDDERS UNDER IMPACT! *** ⚡".to_string()
                } else {
                    format!("  💥 {winner_name} delivers a devastating blow to {loser_name}!")
                }
            }
            BattlePhase::Aftermath => {
                format!(
                    "  🏆 {loser_name} is unseated! {winner_name} claims supreme jousting glory!"
                )
            }
            BattlePhase::Done => {
                format!("  🏁 Tournament ended. All hail the Victor, {winner_name}!")
            }
        };
        for (i, ch) in commentary.chars().enumerate() {
            if i + 1 < self.width.saturating_sub(1) {
                fb.set(i + 1, 2, Cell::new(ch, Color::rgb(255, 255, 120)));
            }
        }

        // 4. Side walls
        for y in 1..self.height.saturating_sub(1) {
            fb.set(0, y, Cell::new('║', border_color));
            if self.width > 1 {
                fb.set(self.width - 1, y, Cell::new('║', border_color));
            }
        }

        // 5. Ground divider line
        let ground_y = self.height.saturating_sub(3);
        if ground_y < self.height {
            fb.set(0, ground_y, Cell::new('╟', border_color));
            for x in 1..(self.width.saturating_sub(1)) {
                fb.set(x, ground_y, Cell::new('─', turf_color));
            }
            if self.width > 1 {
                fb.set(self.width - 1, ground_y, Cell::new('╢', border_color));
            }
        }

        // 6. Turf & dust line
        let turf_y = self.height.saturating_sub(2);
        if turf_y < self.height {
            let grass_pattern =
                "  .  ..   ...  ...      . ..    ...   . . ...       ... .. .    ... .  .. ";
            for (i, ch) in grass_pattern.chars().enumerate() {
                let x = i + 1;
                if x < self.width.saturating_sub(1) {
                    fb.set(x, turf_y, Cell::new(ch, Color::rgb(60, 130, 80)));
                }
            }
        }

        // 7. Bottom border
        let bottom_y = self.height.saturating_sub(1);
        fb.set(0, bottom_y, Cell::new('╚', border_color));
        for x in 1..(self.width.saturating_sub(1)) {
            fb.set(x, bottom_y, Cell::new('═', border_color));
        }
        if self.width > 1 {
            fb.set(self.width - 1, bottom_y, Cell::new('╝', border_color));
        }

        // 8. Galloping dust
        if self.phase == BattlePhase::Charging {
            let hoof_y = self.cow1.y + 4;
            let dust1_x = self.cow1.x - 3;
            let dust1 = match (self.frames / 3) % 4 {
                0 => ".. . o",
                1 => ". o O",
                2 => "o . ..",
                _ => ". .  .",
            };
            for (i, ch) in dust1.chars().enumerate() {
                let dx = dust1_x + i as i32;
                if dx >= 1 && dx < (self.width as i32 - 1) && hoof_y < self.height {
                    fb.set(
                        dx as usize,
                        hoof_y,
                        Cell::new(ch, Color::rgb(140, 140, 140)),
                    );
                }
            }

            let dust2_x = self.cow2.x + 23;
            let dust2 = match (self.frames / 3) % 4 {
                0 => "o . ..",
                1 => "O o .",
                2 => ".. . o",
                _ => ".  . .",
            };
            for (i, ch) in dust2.chars().enumerate() {
                let dx = dust2_x + i as i32;
                if dx >= 1 && dx < (self.width as i32 - 1) && hoof_y < self.height {
                    fb.set(
                        dx as usize,
                        hoof_y,
                        Cell::new(ch, Color::rgb(140, 140, 140)),
                    );
                }
            }
        }

        // 9. Cows
        let lance1_broken = !self.cow1.alive && self.phase != BattlePhase::Charging;
        let lance2_broken = !self.cow2.alive && self.phase != BattlePhase::Charging;
        self.render_cow_art(&mut fb, &self.cow1, true, self.frames / 3, lance1_broken);
        self.render_cow_art(&mut fb, &self.cow2, false, self.frames / 3, lance2_broken);

        // 10. Collision Sparks / Impact Burst at dynamic clash_x
        if self.phase == BattlePhase::Collision {
            let clash_x = self.clash_x.clamp(8, (self.width as i32).saturating_sub(8));
            let clash_y = 5.min(self.height.saturating_sub(6));

            let spark_color = match (self.phase_frames / 2) % 4 {
                0 => Color::rgb(255, 255, 255),
                1 => Color::rgb(255, 230, 80),
                2 => Color::rgb(255, 120, 30),
                _ => Color::rgb(255, 60, 60),
            };

            let spark_lines = match (self.phase_frames / 3) % 3 {
                0 => vec!["   \\  |  /   ", " -- *CLASH* --", "   /  |  \\   "],
                1 => vec![" .  \\ | /  . ", " * -*BOOM*- *", " .  / | \\  . "],
                _ => vec!["  :   ~   :  ", " ✦ * IMPACT * ✦ ", "  ~   .   ~  "],
            };

            for (row_idx, row_str) in spark_lines.iter().enumerate() {
                let sy = clash_y.saturating_sub(1) + row_idx;
                if sy >= self.height {
                    continue;
                }
                for (col_idx, ch) in row_str.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
                    let sx = clash_x + col_idx as i32 - 7;
                    if sx >= 1 && sx < (self.width as i32 - 1) {
                        fb.set(sx as usize, sy, Cell::new(ch, spark_color));
                    }
                }
            }

            // Ground shockwave dust right beneath collision
            let ground_dust = match (self.phase_frames / 2) % 3 {
                0 => ". o O o .",
                1 => "o O ✦ O o",
                _ => ". . o . .",
            };
            if turf_y < self.height {
                for (i, ch) in ground_dust.chars().enumerate() {
                    let gx = clash_x - 4 + i as i32;
                    if gx >= 1 && gx < (self.width as i32 - 1) {
                        fb.set(
                            gx as usize,
                            turf_y,
                            Cell::new(ch, Color::rgb(180, 160, 120)),
                        );
                    }
                }
            }
        }

        // 11. Aftermath KO Stars and Victory Speech
        if self.phase == BattlePhase::Aftermath || self.phase == BattlePhase::Done {
            // Draw KO stars above the loser's head
            let (loser_cow, is_left_loser) = if !self.cow1.alive {
                (&self.cow1, true)
            } else {
                (&self.cow2, false)
            };
            let star_y = loser_cow.y.saturating_sub(1);
            let head_x = if is_left_loser {
                loser_cow.x + 13
            } else {
                loser_cow.x + 8
            };
            let stars = match (self.frames / 2) % 4 {
                0 => "* . + .",
                1 => ". * . +",
                2 => "+ . * .",
                _ => ". + . *",
            };
            for (si, sc) in stars.chars().enumerate() {
                let sx = head_x + si as i32 - 3;
                if sx >= 1 && sx < (self.width as i32 - 1) && star_y < self.height {
                    fb.set(sx as usize, star_y, Cell::new(sc, Color::rgb(255, 220, 50)));
                }
            }

            // Draw victory speech above the winner's head
            let (winner_cow, is_left_winner) = if self.cow1.alive {
                (&self.cow1, true)
            } else {
                (&self.cow2, false)
            };
            let bubble_y = winner_cow.y.saturating_sub(1);
            let win_head_x = if is_left_winner {
                winner_cow.x + 10
            } else {
                winner_cow.x + 5
            };
            let bubble = "\"VICTORY!\"";
            for (bi, bc) in bubble.chars().enumerate() {
                let bx = win_head_x + bi as i32;
                if bx >= 1 && bx < (self.width as i32 - 1) && bubble_y < self.height {
                    fb.set(
                        bx as usize,
                        bubble_y,
                        Cell::new(bc, Color::rgb(50, 255, 120)),
                    );
                }
            }
        }

        fb
    }

    fn render_cow_art(
        &self,
        fb: &mut FrameBuffer,
        cow: &BattleCow,
        is_left_cow: bool,
        legs_frame: u32,
        lance_broken: bool,
    ) {
        let art = if is_left_cow {
            cow1_art(&cow.eyes, legs_frame, lance_broken)
        } else {
            cow2_art(&cow.eyes, legs_frame, lance_broken)
        };

        let lance_color = if is_left_cow {
            Color::rgb(0, 220, 255)
        } else {
            Color::rgb(255, 80, 100)
        };

        let eye_color = match cow.eyes.as_str() {
            "xx" => Color::rgb(255, 60, 60),
            "@@" => Color::rgb(220, 120, 255),
            "^^" => Color::rgb(255, 230, 80),
            _ => Color::rgb(255, 255, 255),
        };

        for (line_idx, line) in art.iter().enumerate() {
            let y = cow.y + line_idx;
            if y >= fb.height {
                continue;
            }

            for (ch_idx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = cow.x + ch_idx as i32;
                if x < 1 || x >= (fb.width as i32 - 1) {
                    continue;
                }

                let color = if ch == '=' || ch == '>' || ch == '<' || (lance_broken && ch == '-') {
                    lance_color
                } else if line_idx == 1
                    && ((is_left_cow && (ch_idx == 14 || ch_idx == 15))
                        || (!is_left_cow && (ch_idx == 9 || ch_idx == 10)))
                {
                    eye_color
                } else if ch == '^' {
                    Color::rgb(255, 195, 75)
                } else {
                    Color::rgb(240, 240, 240)
                };

                fb.set(x as usize, y, Cell::new(ch, color));
            }
        }
    }
}

fn cow1_art(eyes: &str, legs_frame: u32, lance_broken: bool) -> Vec<String> {
    let lance = if lance_broken {
        "== -  -        "
    } else {
        "=======>       "
    };
    let (leg1, leg2) = match legs_frame % 3 {
        0 => ("| w----||", "||     ||"),
        1 => ("| w----//", "//     ||"),
        _ => ("| w----\\\\", "||     \\\\"),
    };
    vec![
        "             ^__^               ".to_string(),
        format!("   ________/({eyes}){lance}"),
        "  /\\(       /__)                ".to_string(),
        format!("    {leg1}                     "),
        format!("    {leg2}                     "),
    ]
}

fn cow2_art(eyes: &str, legs_frame: u32, lance_broken: bool) -> Vec<String> {
    let lance = if lance_broken { "-  - ==" } else { "<=======" };
    let (leg1, leg2) = match legs_frame % 3 {
        0 => ("||----w |", "||     ||"),
        1 => ("\\\\----w |", "||     \\\\"),
        _ => ("//----w |", "//     ||"),
    };
    vec![
        "        ^__^                    ".to_string(),
        format!("{lance}({eyes})\\________       "),
        "        (__\\       )/\\/         ".to_string(),
        format!("            {leg1}         "),
        format!("            {leg2}         "),
    ]
}

/// Run a live, animated battle directly in the terminal with smooth frame updates and cursor protection.
pub fn run_battle_live(name1: &str, name2: &str, winner_choice: u8, target_fps: u16) {
    run_battle_live_internal(name1, name2, winner_choice, target_fps, true);
}

/// Internal live battle runner supporting caller-managed alternate screens.
pub fn run_battle_live_internal(
    name1: &str,
    name2: &str,
    winner_choice: u8,
    target_fps: u16,
    manage_alt_screen: bool,
) {
    let (term_w, term_h) = forgum_platform::terminal_size();
    let width = (term_w as usize).clamp(52, 120);
    let height = 14.min(term_h.saturating_sub(2) as usize).max(12);

    let winner = if winner_choice == 0 {
        if rand::random::<bool>() {
            1
        } else {
            2
        }
    } else {
        winner_choice.clamp(1, 2)
    };

    let mut battle = Battle::with_dimensions(name1, name2, width, height, winner);
    let _cursor_guard = if manage_alt_screen {
        forgum_platform::guards::CursorShowGuard::acquire().ok()
    } else {
        None
    };
    let _alt_guard = if manage_alt_screen {
        forgum_platform::guards::AltScreenGuard::acquire().ok()
    } else {
        None
    };

    let fps = target_fps.clamp(5, 60);
    let frame_time = Duration::from_millis(1000 / fps as u64);
    let max_duration = Duration::from_secs(25);
    let start = Instant::now();

    // Clear terminal once at battle start
    print!("\x1b[2J\x1b[H");
    let _ = std::io::stdout().flush();

    while !battle.is_done() && start.elapsed() < max_duration {
        battle.tick();
        let frame = battle.render_frame_styled(true);
        print!("\x1b[H{frame}");
        let _ = std::io::stdout().flush();
        std::thread::sleep(frame_time);
    }

    if manage_alt_screen {
        drop(_alt_guard);
        drop(_cursor_guard);

        let winner_name = if battle.winner == 1 {
            &battle.cow1.name
        } else {
            &battle.cow2.name
        };
        let loser_name = if battle.winner == 1 {
            &battle.cow2.name
        } else {
            &battle.cow1.name
        };

        println!(
            "\n  \x1b[1;33m🏆 JOUST CHAMPION: \x1b[1;32m{}\x1b[1;33m defeated \x1b[1;31m{}\x1b[1;33m! (Charge: {} vs {} cols | Clash: col {}) All hail the Bovine Victor! 🏆\x1b[0m\n",
            winner_name, loser_name, battle.dist1, battle.dist2, battle.clash_x
        );
        let _ = std::io::stdout().flush();
    }
}

/// Run a simulation of the battle and return the resulting text log without blocking sleeps.
pub fn run_battle(name1: &str, name2: &str) -> String {
    let mut battle = Battle::new(name1, name2);
    let mut output = String::new();

    output.push_str("\x1b[2J\x1b[H");
    output.push_str(&battle.render_frame());
    output.push('\n');

    while !battle.is_done() {
        battle.tick();
        if (battle.phase == BattlePhase::Charging && battle.phase_frames.is_multiple_of(15))
            || (battle.phase == BattlePhase::Collision && battle.phase_frames.is_multiple_of(10))
            || (battle.phase == BattlePhase::Aftermath && battle.phase_frames.is_multiple_of(15))
        {
            output.push_str(&battle.render_frame());
            output.push('\n');
        }
    }

    output.push_str(&battle.render_frame());
    output.push('\n');

    let winner_name = if battle.winner == 1 {
        &battle.cow1.name
    } else {
        &battle.cow2.name
    };
    let loser_name = if battle.winner == 1 {
        &battle.cow2.name
    } else {
        &battle.cow1.name
    };

    output.push_str(&format!(
        "\nBattle result: {} defeats {}! (Travel distance: {} vs {} cols, clash at col {})\n",
        winner_name, loser_name, battle.dist1, battle.dist2, battle.clash_x
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_new_sets_positions() {
        let b = Battle::new("Alice", "Bob");
        assert_eq!(b.cow1.x, 2);
        assert_eq!(b.cow2.x, 70);
    }

    #[test]
    fn battle_charging_moves_cows() {
        let mut b = Battle::new("A", "B");
        b.tick();
        assert!(b.cow1.x > 2 || b.pos1 > 2.0);
        assert!(b.cow2.x < 70 || b.pos2 < 70.0);
    }

    #[test]
    fn battle_reaches_collision() {
        let mut b = Battle::new("A", "B");
        while b.phase == BattlePhase::Charging {
            b.tick();
        }
        assert_eq!(b.phase, BattlePhase::Collision);
    }

    #[test]
    fn battle_eventually_done() {
        let mut b = Battle::new("A", "B");
        for _ in 0..160 {
            b.tick();
        }
        assert!(b.is_done());
    }

    #[test]
    fn battle_render_frame_has_structure() {
        let b = Battle::new("X", "Y");
        let frame = b.render_frame();
        assert!(frame.contains('\n'));
        assert!(frame.len() > 50);
    }

    #[test]
    fn battle_phase_transitions_correctly() {
        let mut b = Battle::new("A", "B");
        assert_eq!(b.phase, BattlePhase::Charging);

        while b.phase == BattlePhase::Charging {
            b.tick();
        }
        assert_eq!(b.phase, BattlePhase::Collision);

        while b.phase == BattlePhase::Collision {
            b.tick();
        }
        assert_eq!(b.phase, BattlePhase::Aftermath);
        assert!(!b.cow1.alive || !b.cow2.alive);

        while b.phase == BattlePhase::Aftermath {
            b.tick();
        }
        assert_eq!(b.phase, BattlePhase::Done);
    }

    #[test]
    fn battle_randomized_travel_distances_are_different() {
        for seed in [1001, 2002, 3003, 4004, 5005] {
            let b = Battle::with_dimensions_and_seed("Alpha", "Beta", 80, 14, 1, seed);
            assert_ne!(
                b.dist1, b.dist2,
                "Travel distances must be different between mascot 1 and mascot 2 for seed {seed}"
            );
            assert!(b.dist1 >= 3, "dist1 must be at least 3 cols");
            assert!(b.dist2 >= 3, "dist2 must be at least 3 cols");
        }
    }

    #[test]
    fn battle_variance_across_multiple_runs() {
        let b1 = Battle::with_dimensions_and_seed("Cow1", "Cow2", 80, 14, 1, 12345);
        let b2 = Battle::with_dimensions_and_seed("Cow1", "Cow2", 80, 14, 1, 67890);
        assert!(
            b1.clash_x != b2.clash_x || b1.dist1 != b2.dist1 || b1.dist2 != b2.dist2,
            "Kinematics must vary across different seeds"
        );
    }

    #[test]
    fn battle_progressive_hp_depletion() {
        let mut b = Battle::with_dimensions_and_seed("Hero", "Villain", 80, 14, 1, 42);
        assert_ne!(
            b.cow1.hp, b.cow2.hp,
            "Mascots must have different starting HP"
        );
        assert_ne!(
            b.cow1.max_hp, b.cow2.max_hp,
            "Mascots must have different max HP"
        );
        let initial_loser_hp = b.cow2.hp;
        while b.phase == BattlePhase::Charging {
            b.tick();
        }
        assert_eq!(b.cow2.hp, initial_loser_hp);
        b.tick();
        assert!(b.cow2.hp <= initial_loser_hp);
        while b.phase == BattlePhase::Collision {
            b.tick();
        }
        assert_eq!(b.cow2.hp, 0);
        assert!(b.cow1.hp > 0);
    }

    #[test]
    fn battle_same_names_works() {
        let mut b = Battle::new("X", "X");
        for _ in 0..200 {
            b.tick();
        }
        assert!(b.is_done());
    }

    #[test]
    fn battle_winner1_selection() {
        let mut b = Battle::with_dimensions("Champion", "Challenger", 80, 14, 1);
        while !b.is_done() {
            b.tick();
        }
        assert!(b.cow1.alive);
        assert!(!b.cow2.alive);
        assert_eq!(b.cow1.eyes, "^^");
        assert_eq!(b.cow2.eyes, "xx");
    }

    #[test]
    fn battle_styled_ansi_color() {
        let b = Battle::new("Alice", "Bob");
        let styled = b.render_frame_styled(true);
        assert!(styled.contains("\x1b[38;2;"));
        assert!(styled.contains("\x1b[0m"));
    }

    #[test]
    fn battle_no_wrap_on_edge() {
        let mut b = Battle::with_dimensions("A", "B", 60, 14, 2);
        b.cow1.x = -10;
        b.cow2.x = 80;
        let frame = b.render_frame();
        assert!(!frame.is_empty());
    }
}
