//! ASCII cow jousting — two cows charge, collide, and one loses.

use crate::framebuffer::{Cell, Color, FrameBuffer};
use std::io::Write;
use std::time::{Duration, Instant};

const BATTLE_WIDTH: usize = 80;
const BATTLE_HEIGHT: usize = 14;
const CHARGE_SPEED: i32 = 4;
const COLLISION_X: i32 = 38;

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
    pub width: usize,
    pub height: usize,
    pub winner: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattlePhase {
    Charging,
    Collision,
    Aftermath,
    Done,
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
        let cow_y = 5.min(height.saturating_sub(6));
        let mut cow1 = BattleCow::new(name1, "oo", 2);
        cow1.y = cow_y;
        let mut cow2 = BattleCow::new(name2, "oo", (width as i32) - 10);
        cow2.y = cow_y;
        Self {
            cow1,
            cow2,
            phase: BattlePhase::Charging,
            frames: 0,
            width,
            height,
            winner,
        }
    }

    pub fn tick(&mut self) {
        self.frames += 1;

        match self.phase {
            BattlePhase::Charging => {
                self.cow1.x += CHARGE_SPEED;
                self.cow2.x -= CHARGE_SPEED;

                if self.cow1.x >= COLLISION_X - 4 || self.cow2.x <= COLLISION_X + 2 {
                    self.phase = BattlePhase::Collision;
                }
            }
            BattlePhase::Collision => {
                if self.frames % 8 == 0 {
                    self.phase = BattlePhase::Aftermath;
                    if self.winner == 1 {
                        self.cow1.eyes = "^^".to_string();
                        self.cow2.eyes = "xx".to_string();
                        self.cow2.alive = false;
                        self.cow2.hp = 0;
                        self.cow1.hp = 85;
                    } else {
                        self.cow1.eyes = "xx".to_string();
                        self.cow2.eyes = "@@".to_string();
                        self.cow1.alive = false;
                        self.cow1.hp = 0;
                        self.cow2.hp = 85;
                    }
                }
            }
            BattlePhase::Aftermath => {
                if self.frames % 20 == 0 {
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
            if use_color && last_color.is_some() {
                output.push_str("\x1b[0m");
            }
            output.push('\n');
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
            "{} [{}{}] {}HP",
            self.cow1.name,
            "■".repeat(hp1_filled),
            "░".repeat(bar_len.saturating_sub(hp1_filled)),
            self.cow1.hp
        );
        let hp2_filled = ((self.cow2.hp as f32 / self.cow2.max_hp.max(1) as f32) * bar_len as f32)
            .round() as usize;
        let hp2_bar = format!(
            "{} [{}{}] {}HP",
            self.cow2.name,
            "■".repeat(hp2_filled),
            "░".repeat(bar_len.saturating_sub(hp2_filled)),
            self.cow2.hp
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
                format!(
                    "  ❯ {} and {} lower their lances and charge!",
                    self.cow1.name, self.cow2.name
                )
            }
            BattlePhase::Collision => {
                "  ❯ *** CLASH OF LANCES! THE MEADOW SHAKES! ***".to_string()
            }
            BattlePhase::Aftermath => {
                format!("  {loser_name} is defeated! {winner_name} wins!")
            }
            BattlePhase::Done => {
                format!("  Battle complete. {winner_name} is victorious!")
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
            let grass_pattern = "  .  ..   ...  ...      . ..    ...   . . ...       ... .. .    ... .  .. ";
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
            let dust1 = match (self.frames / 2) % 3 {
                0 => ".. . o",
                1 => ". o O",
                _ => "o . ..",
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
            let dust2 = match (self.frames / 2) % 3 {
                0 => "o . ..",
                1 => "O o .",
                _ => ".. . o",
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
        self.render_cow_art(
            &mut fb,
            &self.cow1,
            true,
            self.frames / 2,
            lance1_broken,
        );
        self.render_cow_art(
            &mut fb,
            &self.cow2,
            false,
            self.frames / 2,
            lance2_broken,
        );

        // 10. Collision Sparks / Impact Burst
        if self.phase == BattlePhase::Collision {
            let clash_x = COLLISION_X.min(self.width as i32 - 10).max(10);
            let clash_y = 5.min(self.height.saturating_sub(6));

            let spark_color = match self.frames % 3 {
                0 => Color::rgb(255, 255, 100),
                1 => Color::rgb(255, 120, 30),
                _ => Color::rgb(255, 255, 255),
            };

            let spark_lines = match (self.frames / 2) % 3 {
                0 => vec!["   \\  |  /   ", " -- *CLASH* --", "   /  |  \\   "],
                1 => vec![" .  \\ | /  . ", " * -*BOOM*- *", " .  / | \\  . "],
                _ => vec!["  :   .   :  ", " ✦ * IMPACT * ✦ ", "  :   .   :  "],
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
                    fb.set(
                        sx as usize,
                        star_y,
                        Cell::new(sc, Color::rgb(255, 220, 50)),
                    );
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
    let lance = if lance_broken {
        "-  - =="
    } else {
        "<======="
    };
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
    let _cursor_guard = forgum_platform::guards::CursorShowGuard::acquire();

    let fps = target_fps.clamp(5, 60);
    let frame_time = Duration::from_millis(1000 / fps as u64);
    let max_duration = Duration::from_secs(15);
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
        "\n  \x1b[1;33m🏆 JOUST CHAMPION: \x1b[1;32m{}\x1b[1;33m defeated \x1b[1;31m{}\x1b[1;33m! All hail the Bovine Victor! 🏆\x1b[0m\n",
        winner_name, loser_name
    );
    let _ = std::io::stdout().flush();
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
        if battle.phase == BattlePhase::Collision && battle.frames % 4 == 0 {
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
        "\nBattle result: {} defeats {}!\n",
        winner_name, loser_name
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
        assert!(b.cow1.x > 2);
        assert!(b.cow2.x < 70);
    }

    #[test]
    fn battle_reaches_collision() {
        let mut b = Battle::new("A", "B");
        for _ in 0..8 {
            b.tick();
        }
        assert_ne!(b.phase, BattlePhase::Charging);
    }

    #[test]
    fn battle_eventually_done() {
        let mut b = Battle::new("A", "B");
        for _ in 0..100 {
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

        loop {
            b.tick();
            if b.phase == BattlePhase::Collision {
                break;
            }
        }
        assert_eq!(b.phase, BattlePhase::Collision);

        loop {
            b.tick();
            if b.phase == BattlePhase::Aftermath {
                break;
            }
        }
        assert_eq!(b.phase, BattlePhase::Aftermath);
        assert!(!b.cow1.alive);
        assert_eq!(b.cow1.eyes, "xx");
        assert_eq!(b.cow2.eyes, "@@");

        loop {
            b.tick();
            if b.phase == BattlePhase::Done {
                break;
            }
        }
        assert_eq!(b.phase, BattlePhase::Done);
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
