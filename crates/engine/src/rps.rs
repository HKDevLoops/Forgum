//! Rock Paper Scissors mascot battle engine (User vs Computer).
//!
//! Features:
//! - High-entropy zero-bias cryptographic PRNG with Lemire's debiased reduction.
//! - Interactive terminal input (`[r]ock`, `[p]aper`, `[s]cissors`).
//! - Full-color ASCII hand showdown arena.
//! - Automatic routing into physical cow jousting battle based on round outcome.

use crate::battle::run_battle;
use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Possible weapon moves in Rock-Paper-Scissors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpsMove {
    Rock,
    Paper,
    Scissors,
}

impl std::str::FromStr for RpsMove {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "r" | "rock" | "1" => Ok(Self::Rock),
            "p" | "paper" | "2" => Ok(Self::Paper),
            "s" | "scissors" | "scissor" | "3" => Ok(Self::Scissors),
            _ => Err(()),
        }
    }
}

impl RpsMove {
    /// Parse user input leniently from strings like "r", "rock", "p", "paper", "s", "scissors".
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        s.parse().ok()
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

        let mut rng = Self {
            state: seed,
            stream,
        };
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
    let border = "=".repeat(74);

    out.push_str(&format!("\x1b[1;36m+{border}+\x1b[0m\x1b[K\r\n"));
    out.push_str("|  ⚔️  \x1b[1;33mROCK • PAPER • SCISSORS SHOWDOWN: USER VS COMPUTER\x1b[0m            ⚔️   |\x1b[K\r\n");
    out.push_str(&format!("\x1b[1;36m+{border}+\x1b[0m\x1b[K\r\n\x1b[K\r\n"));

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

    out.push_str(&format!(
        "  {:<42}  {}\x1b[K\r\n\x1b[K\r\n",
        p_header, c_header
    ));

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
    print!("\x1b[1;36m+--------------------------------------------------------+\x1b[0m\x1b[K\r\n");
    print!("| 🎮 \x1b[1;33mCHOOSE YOUR WEAPON (USER VS COMPUTER):\x1b[0m              |\x1b[K\r\n");
    print!("|    \x1b[1;32m[r] Rock 🪨\x1b[0m       \x1b[1;34m[p] Paper 📄\x1b[0m     \x1b[1;35m[s] Scissors ✂️\x1b[0m  |\x1b[K\r\n");
    print!("|    \x1b[1;31m[q] Flee arena\x1b[0m                                      |\x1b[K\r\n");
    print!("\x1b[1;36m+--------------------------------------------------------+\x1b[0m\x1b[K\r\n");
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

/// Render a single animated frame of the liquid motion RPS showdown.
pub fn render_liquid_showdown_frame(
    player_name: &str,
    cpu_name: &str,
    player_move: RpsMove,
    cpu_move: RpsMove,
    outcome: RpsOutcome,
    elapsed: f32,
) -> String {
    let (term_w, _) = forgum_platform::terminal_size();
    let width = (term_w as usize).clamp(74, 90);
    let mut out = String::with_capacity(3072);
    let border_len = width.saturating_sub(2);
    let border = "=".repeat(border_len);

    out.push_str(&format!("\x1b[1;36m+{border}+\x1b[0m\x1b[K\r\n"));
    out.push_str("|  ⚔️  \x1b[1;33mROCK • PAPER • SCISSORS SHOWDOWN: USER VS COMPUTER\x1b[0m");
    let title_pad = border_len.saturating_sub(62);
    out.push_str(&format!("{}⚔️  |\x1b[K\r\n", " ".repeat(title_pad)));
    out.push_str(&format!("\x1b[1;36m+{border}+\x1b[0m\x1b[K\r\n\x1b[K\r\n"));

    let p_mascot = crate::battle::resolve_fighter_mascot(player_name, "lion");
    let c_mascot = crate::battle::resolve_fighter_mascot(cpu_name, "dragon");
    let (p_species, _, _) = crate::battle::mascot_combat_profile(&p_mascot);
    let (c_species, _, _) = crate::battle::mascot_combat_profile(&c_mascot);

    let (p_header, c_header) = if elapsed < 1.05 {
        (
            format!(
                "\x1b[1;32m[USER] {} ({})\x1b[0m: ✊ Pumping...",
                player_name, p_species
            ),
            format!(
                "\x1b[1;35m[COMPUTER] {} ({})\x1b[0m: ✊ Pumping...",
                cpu_name, c_species
            ),
        )
    } else {
        (
            format!(
                "\x1b[1;32m[USER] {} ({})\x1b[0m: {} {}",
                player_name,
                p_species,
                player_move.emoji(),
                player_move.name()
            ),
            format!(
                "\x1b[1;35m[COMPUTER] {} ({})\x1b[0m: {} {}",
                cpu_name,
                c_species,
                cpu_move.emoji(),
                cpu_move.name()
            ),
        )
    };

    out.push_str(&format!("  {:<44}  {}\x1b[K\r\n", p_header, c_header));

    // Animated countdown status line
    if elapsed < 0.35 {
        out.push_str("    \x1b[1;33m✊ [1/3] ROCK...\x1b[0m\x1b[K\r\n\x1b[K\r\n");
    } else if elapsed < 0.70 {
        out.push_str("    \x1b[1;36m✊ [2/3] PAPER...\x1b[0m\x1b[K\r\n\x1b[K\r\n");
    } else if elapsed < 1.05 {
        out.push_str("    \x1b[1;35m✊ [3/3] SCISSORS...\x1b[0m\x1b[K\r\n\x1b[K\r\n");
    } else if elapsed < 1.45 {
        out.push_str("    \x1b[1;31m💥 [SHOOT!] THE WEAPONS CLASH!\x1b[0m\x1b[K\r\n\x1b[K\r\n");
    } else if elapsed < 1.80 {
        let impact_msg = match outcome {
            RpsOutcome::PlayerWins => "\x1b[1;32m⚡ CRITICAL IMPACT! User breaks through!\x1b[0m",
            RpsOutcome::ComputerWins => {
                "\x1b[1;31m⚡ CRITICAL IMPACT! Computer shatters defense!\x1b[0m"
            }
            RpsOutcome::Tie => "\x1b[1;33m⚡ STALEMATE IMPACT! Equal momentum!\x1b[0m",
        };
        out.push_str(&format!("    {impact_msg}\x1b[K\r\n\x1b[K\r\n"));
    } else {
        out.push_str("    \x1b[1;33m⚡ SHOWDOWN RESOLVED! ⚡\x1b[0m\x1b[K\r\n\x1b[K\r\n");
    }

    let p_hand = if elapsed < 1.05 {
        RpsMove::Rock.ascii_player()
    } else if elapsed < 1.25 {
        if elapsed < 1.15 {
            RpsMove::Rock.ascii_player()
        } else {
            player_move.ascii_player()
        }
    } else {
        player_move.ascii_player()
    };

    let c_hand = if elapsed < 1.05 {
        RpsMove::Rock.ascii_cpu()
    } else if elapsed < 1.25 {
        if elapsed < 1.15 {
            RpsMove::Rock.ascii_cpu()
        } else {
            cpu_move.ascii_cpu()
        }
    } else {
        cpu_move.ascii_cpu()
    };

    let (pad_top, pad_bot) = if elapsed < 1.05 {
        let sub = (elapsed % 0.35) / 0.35;
        let bob = (std::f32::consts::PI * sub).sin();
        if bob > 0.65 {
            (0, 2)
        } else if bob > 0.30 {
            (1, 1)
        } else {
            (2, 0)
        }
    } else {
        (1, 1)
    };

    let (x1, x2) = if elapsed < 1.05 {
        (4.0_f32, 48.0_f32)
    } else if elapsed < 1.45 {
        let tau = ((elapsed - 1.05) / 0.40).clamp(0.0, 1.0);
        let e = 1.0 - (1.0 - tau).powi(3);
        (4.0 + 12.0 * e, 48.0 - 12.0 * e)
    } else {
        let dt = elapsed - 1.45;
        let d = (-3.5 * dt).exp() * (std::f32::consts::TAU * 7.0 * dt).cos();
        let r = 1.0 - (-5.0 * dt).exp();
        match outcome {
            RpsOutcome::PlayerWins => (16.0 + 1.5 * r, 36.0 + 9.0 * r + 2.5 * d),
            RpsOutcome::ComputerWins => (16.0 - 9.0 * r - 2.5 * d, 36.0 - 1.5 * r),
            RpsOutcome::Tie => (16.0 - 3.5 * r - 1.5 * d, 36.0 + 3.5 * r + 1.5 * d),
        }
    };

    let col1 = (x1.round() as usize).clamp(1, 28);
    let col2 = (x2.round() as usize).clamp(col1 + 17, width.saturating_sub(18));

    let (p_color, c_color) = if elapsed < 1.05 {
        ("\x1b[1;32m", "\x1b[1;35m")
    } else if elapsed < 1.45 {
        ("\x1b[1;37m", "\x1b[1;37m")
    } else {
        match outcome {
            RpsOutcome::PlayerWins => ("\x1b[1;32m", "\x1b[1;31m"),
            RpsOutcome::ComputerWins => ("\x1b[1;31m", "\x1b[1;35m"),
            RpsOutcome::Tie => ("\x1b[1;33m", "\x1b[1;33m"),
        }
    };

    for _ in 0..pad_top {
        out.push_str("\x1b[K\r\n");
    }

    for row in 0..6 {
        let p_str = p_hand[row];
        let c_str = c_hand[row];
        let gap_size = col2.saturating_sub(col1 + 17);
        let gap_str = if elapsed >= 1.40 && (row == 2 || row == 3) {
            let spark_idx = (((elapsed * 20.0) as usize) + row) % 4;
            let spark = match spark_idx {
                0 => "\x1b[1;33m✦ \x1b[1;31m💥 \x1b[1;36m✧\x1b[0m",
                1 => "\x1b[1;31m⚡ \x1b[1;33m* \x1b[1;35m✦\x1b[0m",
                2 => "\x1b[1;36m✧ \x1b[1;33m⚡ \x1b[1;31m💥\x1b[0m",
                _ => "\x1b[1;33m💥 \x1b[1;32m+ \x1b[1;33m⚡\x1b[0m",
            };
            let center_pad = gap_size.saturating_sub(7) / 2;
            format!(
                "{}{}{}",
                " ".repeat(center_pad),
                spark,
                " ".repeat(gap_size.saturating_sub(center_pad + 7))
            )
        } else if row == 2 && elapsed >= 1.05 && elapsed < 1.40 {
            let center_pad = gap_size.saturating_sub(10) / 2;
            format!(
                "{}\x1b[1;33m⚡ VS ⚡\x1b[0m{}",
                " ".repeat(center_pad),
                " ".repeat(gap_size.saturating_sub(center_pad + 10))
            )
        } else {
            " ".repeat(gap_size)
        };

        out.push_str(&format!(
            "{}{}{}\x1b[0m{}{}{}\x1b[0m\x1b[K\r\n",
            " ".repeat(col1),
            p_color,
            p_str,
            gap_str,
            c_color,
            c_str
        ));
    }

    for _ in 0..pad_bot {
        out.push_str("\x1b[K\r\n");
    }

    out.push_str("\x1b[K\r\n");

    if elapsed >= 1.70 {
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
                    "  \x1b[1;33m🏆 {} ({}) claims decisive momentum! Charging to battle! 🏆\x1b[0m\x1b[K\r\n\x1b[K\r\n",
                    player_name, p_species
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
                    "  \x1b[1;31m⚔️ {} ({}) strikes back with tournament advantage! ⚔️\x1b[0m\x1b[K\r\n\x1b[K\r\n",
                    cpu_name, c_species
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
    } else {
        out.push_str("\x1b[K\r\n\x1b[K\r\n");
    }

    out
}

/// Run high-refresh liquid motion animated showdown loop with DEC Mode 2026 synchronized updates.
pub fn run_liquid_showdown(
    player_name: &str,
    cpu_name: &str,
    player_move: RpsMove,
    cpu_move: RpsMove,
    outcome: RpsOutcome,
    fps: u16,
) {
    let target_fps = fps.clamp(5, 240) as u32;
    let mut governor = crate::engine_core::LiquidFrameGovernor::new(target_fps);
    let shutdown = forgum_platform::ShutdownFlag::new();
    let total_duration = Duration::from_millis(2500);
    let mut start_time = Instant::now();

    while start_time.elapsed() < total_duration && !shutdown.is_shutdown() {
        if crossterm::event::poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(crossterm::event::Event::Key(k)) = crossterm::event::read() {
                if k.kind == crossterm::event::KeyEventKind::Press {
                    match k.code {
                        crossterm::event::KeyCode::Char('q')
                        | crossterm::event::KeyCode::Char('Q')
                        | crossterm::event::KeyCode::Esc => {
                            return;
                        }
                        crossterm::event::KeyCode::Char(' ')
                        | crossterm::event::KeyCode::Enter
                            if start_time.elapsed() < Duration::from_millis(1600) =>
                        {
                            start_time = Instant::now() - Duration::from_millis(1700);
                        }
                        _ => {}
                    }
                }
            }
        }

        let elapsed = start_time.elapsed().as_secs_f32();
        let frame = render_liquid_showdown_frame(
            player_name,
            cpu_name,
            player_move,
            cpu_move,
            outcome,
            elapsed,
        );

        // Tear-free DEC Mode 2026 synchronized terminal frame
        print!("\x1b[?2026h\x1b[H{frame}\x1b[?2026l");
        let _ = io::stdout().flush();

        governor.pace_frame(&shutdown);
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

    // Live liquid motion animated showdown (up to 240Hz with monotonic pacing)
    run_liquid_showdown(player_name, cpu_name, p_move, c_move, outcome, fps);

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

    // Reset margins and attributes INSIDE the alternate screen before leaving
    let _ = io::stdout().write_all(b"\x1b[r\x1b[?6l\x1b[?69l\x1b[0m\x1b[?25h");
    let _ = io::stdout().flush();

    // Drop alternate screen and cursor guards so we leave alternate screen
    drop(_alt_guard);
    drop(_cursor_guard);

    let p_mascot = crate::battle::resolve_fighter_mascot(player_name, "lion");
    let c_mascot = crate::battle::resolve_fighter_mascot(cpu_name, "dragon");
    let (winner_name, loser_name, winner_species) = if winner == 1 {
        let (species, _, _) = crate::battle::mascot_combat_profile(&p_mascot);
        (player_name, cpu_name, species)
    } else {
        let (species, _, _) = crate::battle::mascot_combat_profile(&c_mascot);
        (cpu_name, player_name, species)
    };
    println!(
        "\n  \x1b[1;33m🏆 RPS ARENA CHAMPION: \x1b[1;32m{}\x1b[1;33m defeated \x1b[1;31m{}\x1b[1;33m! All hail the {} Victor! 🏆\x1b[0m\n",
        winner_name, loser_name, winner_species
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
    log.push_str(&render_showdown_arena(
        player_name,
        cpu_name,
        p_move,
        c_move,
        outcome,
    ));

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

    #[test]
    fn rps_liquid_showdown_frame_renders_all_phases() {
        let p_name = "Player";
        let c_name = "Computer";
        let p_move = RpsMove::Rock;
        let c_move = RpsMove::Scissors;
        let outcome = RpsOutcome::PlayerWins;

        // Phase 1: Countdown pump
        let frame_count1 = render_liquid_showdown_frame(p_name, c_name, p_move, c_move, outcome, 0.2);
        assert!(frame_count1.contains("ROCK • PAPER • SCISSORS SHOWDOWN"));
        assert!(frame_count1.contains("ROCK..."));

        // Phase 2: Shoot & thrust
        let frame_shoot = render_liquid_showdown_frame(p_name, c_name, p_move, c_move, outcome, 1.2);
        assert!(frame_shoot.contains("SHOOT!"));

        // Phase 3: Impact & recoil
        let frame_impact = render_liquid_showdown_frame(p_name, c_name, p_move, c_move, outcome, 1.6);
        assert!(frame_impact.contains("CRITICAL IMPACT"));

        // Phase 4: Victorious fanfare
        let frame_result = render_liquid_showdown_frame(p_name, c_name, p_move, c_move, outcome, 2.0);
        assert!(frame_result.contains("VICTORY"));
        assert!(frame_result.contains("CRUSHES"));
    }
}
