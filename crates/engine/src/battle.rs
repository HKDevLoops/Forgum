//! ASCII cow jousting — two cows charge, collide, and one loses.

use crate::framebuffer::{Color, FrameBuffer};
use std::time::{Duration, Instant};

const BATTLE_WIDTH: usize = 80;
const BATTLE_HEIGHT: usize = 12;
const CHARGE_SPEED: i32 = 4;
const COLLISION_X: i32 = 38;

#[derive(Debug, Clone)]
pub struct BattleCow {
    pub name: String,
    pub eyes: String,
    pub x: i32,
    pub y: usize,
    pub alive: bool,
}

impl BattleCow {
    pub fn new(name: &str, eyes: &str, start_x: i32) -> Self {
        Self {
            name: name.to_string(),
            eyes: eyes.to_string(),
            x: start_x,
            y: 4,
            alive: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Battle {
    pub cow1: BattleCow,
    pub cow2: BattleCow,
    pub phase: BattlePhase,
    pub frames: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BattlePhase {
    Charging,
    Collision,
    Aftermath,
    Done,
}

impl Battle {
    pub fn new(name1: &str, name2: &str) -> Self {
        Self {
            cow1: BattleCow::new(name1, "oo", 2),
            cow2: BattleCow::new(name2, "oo", (BATTLE_WIDTH as i32) - 10),
            phase: BattlePhase::Charging,
            frames: 0,
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
                    self.cow1.eyes = "xx".to_string();
                    self.cow2.eyes = "@@".to_string();
                    self.cow1.alive = false;
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
        let mut fb = FrameBuffer::new(BATTLE_WIDTH, BATTLE_HEIGHT);

        let status = match self.phase {
            BattlePhase::Charging => format!("  {} charges at {}!", self.cow1.name, self.cow2.name),
            BattlePhase::Collision => "  *** COLLISION! ***".to_string(),
            BattlePhase::Aftermath => {
                format!("  {} is defeated! {} wins!", self.cow2.name, self.cow1.name)
            }
            BattlePhase::Done => format!("  Battle complete. {} is victorious!", self.cow1.name),
        };

        for (i, ch) in status.chars().enumerate() {
            fb.set(
                i,
                0,
                crate::framebuffer::Cell::new(ch, Color::rgb(255, 255, 0)),
            );
        }

        self.render_cow_art(&mut fb, &self.cow1);
        self.render_cow_art(&mut fb, &self.cow2);

        if self.phase == BattlePhase::Collision {
            let impact = "* * * BOOM * * *";
            for (i, ch) in impact.chars().enumerate() {
                fb.set(
                    (COLLISION_X + i as i32 - 8) as usize,
                    3,
                    crate::framebuffer::Cell::new(ch, Color::rgb(255, 100, 0)),
                );
            }
        }

        let mut output = String::new();
        for y in 0..BATTLE_HEIGHT {
            for x in 0..BATTLE_WIDTH {
                let cell = fb.back[y * BATTLE_WIDTH + x];
                if cell.alpha == 0 || cell.ch == ' ' {
                    output.push(' ');
                } else {
                    output.push(cell.ch);
                }
            }
            output.push('\n');
        }
        output
    }

    fn render_cow_art(&self, fb: &mut FrameBuffer, cow: &BattleCow) {
        if cow.x < 0 || cow.x >= BATTLE_WIDTH as i32 {
            return;
        }

        let art = format!(
            "  \\   ^__^  \n   \\  ({})\\_______\n      (__\\       )\\/\\\n          ||----w |\n          ||     ||",
            cow.eyes
        );

        for (line_idx, line) in art.lines().enumerate() {
            let y = cow.y + line_idx;
            for (ch_idx, ch) in line.chars().enumerate() {
                let x = (cow.x as usize + ch_idx) % BATTLE_WIDTH;
                fb.set(
                    x,
                    y,
                    crate::framebuffer::Cell::new(ch, Color::rgb(255, 255, 255)),
                );
            }
        }
    }
}

pub fn run_battle(name1: &str, name2: &str) -> String {
    let mut battle = Battle::new(name1, name2);
    let start = Instant::now();
    let max_duration = Duration::from_secs(8);

    let mut output = String::new();
    output.push_str("\x1b[2J\x1b[H");

    while !battle.is_done() && start.elapsed() < max_duration {
        battle.tick();
        let frame = battle.render_frame();
        output.push_str(&frame);
        output.push('\n');
        std::thread::sleep(Duration::from_millis(100));
    }

    output.push_str(&format!(
        "\nBattle result: {} defeats {}!\n",
        battle.cow1.name, battle.cow2.name
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
}
