//! Rock Paper Scissors mascot battle engine (User vs Computer).
//!
//! Features:
//! - High-entropy zero-bias cryptographic PRNG with Lemire's debiased reduction.
//! - Interactive terminal input (`[r]ock`, `[p]aper`, `[s]cissors`).
//! - Full-color ASCII hand showdown arena.
//! - Automatic routing into physical cow jousting battle based on round outcome.

use crate::battle::run_battle;
use std::io::{self, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Possible weapon moves in Rock-Paper-Scissors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpsMove {
    Rock,
    Paper,
    Scissors,
}

impl RpsMove {
    /// Parse user input leniently from strings like "r", "rock", "p", "paper", "s", "scissors".
    pub fn from_str(s: &str) -> Option<Self> {
        let trimmed = s.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "r" | "rock" | "1" => Some(Self::Rock),
            "p" | "paper" | "2" => Some(Self::Paper),
            "s" | "scissors" | "scissor" | "3" => Some(Self::Scissors),
            _ => None,
        }
    }

    /// Primary display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rock => "ROCK",
            Self::Paper => "PAPER",
            Self::Scissors => "SCISSORS",
        }
    }

    /// Unicode emoji glyph.
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Rock => "🪨",
            Self::Paper => "📄",
            Self::Scissors => "✂️",
        }
    }

    /// Returns whether `self` beats `other` under standard RPS rules.
    pub fn beats(&self, other: RpsMove) -> bool {
        matches!(
            (self, other),
            (Self::Rock, Self::Scissors)
                | (Self::Scissors, Self::Paper)
                | (Self::Paper, Self::Rock)
        )
    }

    /// Dramatic action verb describing how `self` defeats `other`.
    pub fn verb_against(&self, other: RpsMove) -> &'static str {
        match (self, other) {
            (Self::Rock, Self::Scissors) => "CRUSHES",
            (Self::Scissors, Self::Paper) => "SLICES",
            (Self::Paper, Self::Rock) => "COVERS",
            _ => "CLASHES WITH",
        }
    }

    /// ASCII hand art facing right (Player perspective).
    pub fn ascii_player(&self) -> [&'static str; 6] {
        match self {
            Self::Rock => [
                "    _______      ",
                "---'   ____)     ",
                "      (_____)    ",
                "      (_____)    ",
                "      (____)     ",
                "---.__(___)      ",
            ],
            Self::Paper => [
                "    _______      ",
                "---'   ____)____ ",
                "          ______) ",
                "          _______)",
                "         _______)",
                "---.__________)  ",
            ],
            Self::Scissors => [
                "    _______      ",
                "---'   ____)____ ",
                "          ______) ",
                "       __________)",
                "      (____)     ",
                "---.__(___)      ",
            ],
        }
    }

    /// ASCII hand art facing left (Computer perspective).
    pub fn ascii_cpu(&self) -> [&'static str; 6] {
        match self {
            Self::Rock => [
                "      _______    ",
                "     (____   '---",
                "    (_____)      ",
                "    (_____)      ",
                "     (____)      ",
                "      (___)__.---",
            ],
            Self::Paper => [
                "      _______    ",
                " ____(____   '---",
                "(______          ",
                "(_______         ",
                " (_______        ",
                "   (__________.---",
            ],
            Self::Scissors => [
                "      _______    ",
                " ____(____   '---",
                "(______          ",
                "(__________      ",
                "     (____)      ",
                "      (___)__.---",
            ],
        }
    }
}

/// The result of an RPS showdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpsOutcome {
    PlayerWins,
    ComputerWins,
    Tie,
}

impl RpsOutcome {
    pub fn evaluate(player: RpsMove, cpu: RpsMove) -> Self {
        if player == cpu {
            Self::Tie
        } else if player.beats(cpu) {
            Self::PlayerWins
        } else {
            Self::ComputerWins
        }
    }
}

/// High-entropy, zero-modulo-bias Pseudo-Random Number Generator.
///
/// Combines nanosecond timing entropy, OS entropy seeds, and SplitMix64 avalanche
/// mixing with Lemire's debiased reduction to ensure strictly uniform 1/3 odds.
#[derive(Debug, Clone)]
pub struct RpsRng {
    state: u64,
    stream: u64,
}

impl RpsRng {
    /// Initialize with fresh multi-source system entropy.
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xa5a5a5a5);
        let rand_seed = rand::random::<u64>();
        let pid = std::process::id() as u64;

        let seed = nanos ^ rand_seed.rotate_left(17) ^ (pid.wrapping_mul(0x9e3779b97f4a7c15));
        let stream = rand_seed.wrapping_add(0xda942042e4dd58b5) | 1;

        let mut rng = Self { state: seed, stream };
        // Warm up state
        rng.next_u32();
        rng.next_u32();
        rng
    }

    /// Initialize deterministically with explicit seed (ideal for unit testing).
    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: seed,
            stream: 0xda942042e4dd58b5 | 1,
        }
    }

    /// Step generator and return next 32-bit word using PCG-style permutation.
    pub fn next_u32(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.stream | 1);
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        (xorshifted >> rot) | (xorshifted << ((!rot).wrapping_add(1) & 31))
    }

    /// Generate an unbiased `RpsMove` using Lemire's fast-range debiased reduction.
    ///
    /// This mathematically eliminates any modulo bias:
    /// `P(Rock) = P(Paper) = P(Scissors) = 1/3` exactly.
    pub fn next_move(&mut self) -> RpsMove {
        let s = 3_u64;
        let mut x = self.next_u32() as u64;
        let mut m = x * s;
        let mut l = m as u32;

        if l < 3 {
            let t = (!3_u32).wrapping_add(1) % 3;
            while l < t {
                x = self.next_u32() as u64;
                m = x * s;
                l = m as u32;
            }
        }

        match (m >> 32) as u32 {
            0 => RpsMove::Rock,
            1 => RpsMove::Paper,
            _ => RpsMove::Scissors,
        }
    }
}

impl Default for RpsRng {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the ASCII confrontation arena showing Player and CPU hands side-by-side.
pub fn render_showdown_arena(
    player_name: &str,
    cpu_name: &str,
    player_move: RpsMove,
    cpu_move: RpsMove,
    outcome: RpsOutcome,
) -> String {
    let mut out = String::with_capacity(2048);
    let border = "═".repeat(74);

    out.push_str(&format!("\x1b[1;36m╔{border}╗\x1b[0m\x1b[K\r\n"));
    out.push_str("║  ⚔️  \x1b[1;33mROCK • PAPER • SCISSORS SHOWDOWN: USER VS COMPUTER\x1b[0m            ⚔️   ║\x1b[K\r\n");
    out.push_str(&format!("\x1b[1;36m╚{border}╝\x1b[0m\x1b[K\r\n\x1b[K\r\n"));

    let p_header = format!(
        "\x1b[1;32m[USER] {}\x1b[0m: {} {}",
        player_name,
        player_move.emoji(),
        player_move.name()
    );
    let c_header = format!(
        "\x1b[1;35m[COMPUTER] {}\x1b[0m: {} {}",
        cpu_name,
        cpu_move.emoji(),
        cpu_move.name()
    );

    out.push_str(&format!("  {:<42}  {}\x1b[K\r\n\x1b[K\r\n", p_header, c_header));

    let p_hand = player_move.ascii_player();
    let c_hand = cpu_move.ascii_cpu();

    for i in 0..6 {
        let center = if i == 2 {
            "\x1b[1;33m   ⚡ VS ⚡   \x1b[0m"
        } else {
            "              "
        };
        out.push_str(&format!(
            "  \x1b[1;37m{}\x1b[0m{}\x1b[1;37m{}\x1b[0m\x1b[K\r\n",
            p_hand[i], center, c_hand[i]
        ));
    }

    out.push_str("\x1b[K\r\n");

    match outcome {
        RpsOutcome::PlayerWins => {
            let verb = player_move.verb_against(cpu_move);
            out.push_str(&format!(
                "  \x1b[1;32m🎉 VICTORY! Your [{}] {} Computer's [{}]!\x1b[0m\x1b[K\r\n",
                player_move.name(),
                verb,
                cpu_move.name()
            ));
            out.push_str(&format!(
                "  \x1b[1;33m🏆 {} claims decisive momentum! Charging to battle! 🏆\x1b[0m\x1b[K\r\n\x1b[K\r\n",
                player_name
            ));
        }
        RpsOutcome::ComputerWins => {
            let verb = cpu_move.verb_against(player_move);
            out.push_str(&format!(
                "  \x1b[1;31m💀 DEFEAT! Computer's [{}] {} your [{}]!\x1b[0m\x1b[K\r\n",
                cpu_move.name(),
                verb,
                player_move.name()
            ));
            out.push_str(&format!(
                "  \x1b[1;31m⚔️ {} strikes back with fierce tournament advantage! ⚔️\x1b[0m\x1b[K\r\n\x1b[K\r\n",
                cpu_name
            ));
        }
        RpsOutcome::Tie => {
            out.push_str(&format!(
                "  \x1b[1;33m🤝 STALEMATE! Both champions deployed [{}]!\x1b[0m\x1b[K\r\n",
                player_move.name()
            ));
            out.push_str("  \x1b[1;33m⚔️ Evenly matched! Entering sudden-death joust arena! ⚔️\x1b[0m\x1b[K\r\n\x1b[K\r\n");
        }
    }

    out
}

/// Prompt the user interactively in the terminal for their weapon selection.
pub fn prompt_user_choice() -> Option<RpsMove> {
    print!("\x1b[2J\x1b[H");
    print!("\x1b[1;36m┌────────────────────────────────────────────────────────┐\x1b[0m\x1b[K\r\n");
    print!("│ 🎮 \x1b[1;33mCHOOSE YOUR WEAPON (USER VS COMPUTER):\x1b[0m              │\x1b[K\r\n");
    print!("│    \x1b[1;32m[r] Rock 🪨\x1b[0m       \x1b[1;34m[p] Paper 📄\x1b[0m     \x1b[1;35m[s] Scissors ✂️\x1b[0m  │\x1b[K\r\n");
    print!("│    \x1b[1;31m[q] Flee arena\x1b[0m                                      │\x1b[K\r\n");
    print!("\x1b[1;36m└────────────────────────────────────────────────────────┘\x1b[0m\x1b[K\r\n");
    print!("\x1b[1;33m❯ Select move [r/p/s] (or q to exit): \x1b[0m\x1b[K");
    let _ = io::stdout().flush();

    if let Ok(raw_guard) = forgum_platform::guards::RawModeGuard::acquire() {
        loop {
            if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                if key_event.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                        print!("\x1b[1;32mRock 🪨\x1b[0m\x1b[K\r\n");
                        let _ = io::stdout().flush();
                        std::thread::sleep(Duration::from_millis(250));
                        drop(raw_guard);
                        return Some(RpsMove::Rock);
                    }
                    crossterm::event::KeyCode::Char('p') | crossterm::event::KeyCode::Char('P') => {
                        print!("\x1b[1;34mPaper 📄\x1b[0m\x1b[K\r\n");
                        let _ = io::stdout().flush();
                        std::thread::sleep(Duration::from_millis(250));
                        drop(raw_guard);
                        return Some(RpsMove::Paper);
                    }
                    crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                        print!("\x1b[1;35mScissors ✂️\x1b[0m\x1b[K\r\n");
                        let _ = io::stdout().flush();
                        std::thread::sleep(Duration::from_millis(250));
                        drop(raw_guard);
                        return Some(RpsMove::Scissors);
                    }
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Char('Q')
                    | crossterm::event::KeyCode::Esc => {
                        print!("\x1b[1;31mFleeing arena...\x1b[0m\x1b[K\r\n");
                        let _ = io::stdout().flush();
                        std::thread::sleep(Duration::from_millis(300));
                        drop(raw_guard);
                        return None;
                    }
                    crossterm::event::KeyCode::Char('c')
                        if key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        drop(raw_guard);
                        return None;
                    }
                    _ => {}
                }
            }
        }
    } else {
        let mut input = String::new();
        while io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim().to_ascii_lowercase();
            if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
                println!("\x1b[1;31m  Fled the arena! Cowards don't win trophies!\x1b[0m\n");
                return None;
            }
            if let Some(m) = RpsMove::from_str(&trimmed) {
                return Some(m);
            }
            input.clear();
            print!("\x1b[1;31m  Invalid weapon! Please enter [r]ock, [p]aper, [s]cissors (or q): \x1b[0m");
            let _ = io::stdout().flush();
        }
        None
    }
}

/// Run a live, interactive Rock-Paper-Scissors battle with showdown animation.
pub fn run_rps_battle_live(
    player_name: &str,
    cpu_name: &str,
    preselected_choice: Option<RpsMove>,
    _rounds: u32,
    fps: u16,
) {
    let _cursor_guard = forgum_platform::guards::CursorShowGuard::acquire().ok();
    let _alt_guard = forgum_platform::guards::AltScreenGuard::acquire().ok();

    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();

    let p_move = match preselected_choice {
        Some(m) => m,
        None => match prompt_user_choice() {
            Some(m) => m,
            None => return,
        },
    };

    let mut rng = RpsRng::new();
    let c_move = rng.next_move();
    let outcome = RpsOutcome::evaluate(p_move, c_move);

    // Clear terminal and display dramatic hand showdown
    print!("\x1b[2J\x1b[H");
    let arena_text = render_showdown_arena(player_name, cpu_name, p_move, c_move, outcome);
    print!("{arena_text}");
    let _ = io::stdout().flush();

    // Dramatic pause for player to register showdown result
    std::thread::sleep(Duration::from_millis(1800));

    // Determine joust victor: 1 for Player, 2 for Computer
    let winner = match outcome {
        RpsOutcome::PlayerWins => 1,
        RpsOutcome::ComputerWins => 2,
        RpsOutcome::Tie => {
            if rand::random::<bool>() {
                1
            } else {
                2
            }
        }
    };

    // Transition smoothly into physical jousting battle inside the same alternate screen
    crate::battle::run_battle_live_internal(player_name, cpu_name, winner, fps, false);

    // Drop alternate screen and cursor guards so we leave alternate screen
    drop(_alt_guard);
    drop(_cursor_guard);

    let (winner_name, loser_name) = if winner == 1 {
        (player_name, cpu_name)
    } else {
        (cpu_name, player_name)
    };
    println!(
        "\n  \x1b[1;33m🏆 RPS ARENA CHAMPION: \x1b[1;32m{}\x1b[1;33m defeated \x1b[1;31m{}\x1b[1;33m! All hail the Bovine Victor! 🏆\x1b[0m\n",
        winner_name, loser_name
    );
    let _ = io::stdout().flush();
}

/// Run a fast headless simulation of the Rock-Paper-Scissors battle and return the complete log.
pub fn run_rps_battle(
    player_name: &str,
    cpu_name: &str,
    preselected_choice: Option<RpsMove>,
) -> String {
    let mut rng = RpsRng::new();
    let p_move = preselected_choice.unwrap_or_else(|| rng.next_move());
    let c_move = rng.next_move();
    let outcome = RpsOutcome::evaluate(p_move, c_move);

    let mut log = String::new();
    log.push_str(&render_showdown_arena(player_name, cpu_name, p_move, c_move, outcome));

    let battle_log = run_battle(player_name, cpu_name);
    log.push_str(&battle_log);

    log
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rps_rules_evaluation_accuracy() {
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Rock, RpsMove::Scissors),
            RpsOutcome::PlayerWins
        );
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Scissors, RpsMove::Paper),
            RpsOutcome::PlayerWins
        );
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Paper, RpsMove::Rock),
            RpsOutcome::PlayerWins
        );

        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Scissors, RpsMove::Rock),
            RpsOutcome::ComputerWins
        );
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Paper, RpsMove::Scissors),
            RpsOutcome::ComputerWins
        );
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Rock, RpsMove::Paper),
            RpsOutcome::ComputerWins
        );

        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Rock, RpsMove::Rock),
            RpsOutcome::Tie
        );
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Paper, RpsMove::Paper),
            RpsOutcome::Tie
        );
        assert_eq!(
            RpsOutcome::evaluate(RpsMove::Scissors, RpsMove::Scissors),
            RpsOutcome::Tie
        );
    }

    #[test]
    fn rps_input_parsing_leniency() {
        assert_eq!(RpsMove::from_str("r"), Some(RpsMove::Rock));
        assert_eq!(RpsMove::from_str("ROCK"), Some(RpsMove::Rock));
        assert_eq!(RpsMove::from_str("  p  "), Some(RpsMove::Paper));
        assert_eq!(RpsMove::from_str("Paper"), Some(RpsMove::Paper));
        assert_eq!(RpsMove::from_str("s"), Some(RpsMove::Scissors));
        assert_eq!(RpsMove::from_str("Scissors"), Some(RpsMove::Scissors));
        assert_eq!(RpsMove::from_str("invalid"), None);
    }

    #[test]
    fn rps_rng_unbiased_distribution() {
        let mut rng = RpsRng::with_seed(123456789);
        let mut rock_count = 0;
        let mut paper_count = 0;
        let mut scissors_count = 0;
        let total = 3000;

        for _ in 0..total {
            match rng.next_move() {
                RpsMove::Rock => rock_count += 1,
                RpsMove::Paper => paper_count += 1,
                RpsMove::Scissors => scissors_count += 1,
            }
        }

        // Each choice should be ~1000 with generous statistical bounds [850, 1150]
        assert!(
            rock_count >= 850 && rock_count <= 1150,
            "Rock count out of expected bounds: {}",
            rock_count
        );
        assert!(
            paper_count >= 850 && paper_count <= 1150,
            "Paper count out of expected bounds: {}",
            paper_count
        );
        assert!(
            scissors_count >= 850 && scissors_count <= 1150,
            "Scissors count out of expected bounds: {}",
            scissors_count
        );
    }

    #[test]
    fn rps_headless_simulation_produces_output() {
        let log = run_rps_battle("PlayerOne", "RobotOverlord", Some(RpsMove::Rock));
        assert!(log.contains("ROCK • PAPER • SCISSORS SHOWDOWN"));
        assert!(log.contains("PlayerOne"));
        assert!(log.contains("RobotOverlord"));
        assert!(log.contains("ROCK"));
    }
}
