//! Brutal meme disgrace words and failure streak tracking for invalid commands.
//!
//! When a user provides wrong arguments or wrong subcommands more than 2 times
//! consecutively, Forgum intercepts the terminal and renders an honest, brutal,
//! funky meme disgrace box from the ASCII mascots.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FailureState {
    count: usize,
    timestamp: u64,
}

fn streak_file_path() -> PathBuf {
    if let Ok(rt) = forgum_platform::runtime_dir() {
        rt.join("failure_streak.json")
    } else if let Ok(cfg) = forgum_platform::config_dir() {
        cfg.join(".failure_streak.json")
    } else {
        PathBuf::from(".forgum_failure_streak.json")
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Record an invalid command attempt and return the updated consecutive failure streak count.
pub fn record_and_get_failure_streak() -> usize {
    let path = streak_file_path();
    let now = current_timestamp();
    let mut state = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<FailureState>(&s).ok())
            .unwrap_or_default()
    } else {
        FailureState::default()
    };

    // If previous failure was more than 30 minutes ago, restart streak at 1
    if now.saturating_sub(state.timestamp) > 1800 {
        state.count = 1;
    } else {
        state.count += 1;
    }
    state.timestamp = now;

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(&state) {
        let _ = std::fs::write(&path, json);
    }

    state.count
}

/// Reset consecutive failure streak when a valid command executes successfully.
pub fn reset_failure_streak() {
    let path = streak_file_path();
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
}

/// Current failure streak count without incrementing.
#[must_use]
pub fn get_failure_streak() -> usize {
    let path = streak_file_path();
    let now = current_timestamp();
    if path.exists() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<FailureState>(&s) {
                if now.saturating_sub(state.timestamp) <= 1800 {
                    return state.count;
                }
            }
        }
    }
    0
}

/// Curated honest and brutal funky meme disgrace quotes.
const DISGRACE_QUOTES: &[&str] = &[
    "Skill issue detected. Even a randomized monkey typing on a Commodore 64 would have stumbled into `forgum fortune` by now.",
    "Brother, you have failed consecutively. The terminal is weeping, your CPU is experiencing second-hand embarrassment, and the cow is visibly disappointed.",
    "Three strikes and you're STILL freestyling CLI flags? It's `forgum --help`, not an ancient spell invocation. Stop guessing syllables and read the docs!",
    "Certified clown moment 🤡. Look at the cow. The cow is looking at you. The cow is wondering how you survived this long in a terminal environment.",
    "Error 404: Command competence not found. Your shell history right now looks like a toddler had a boxing match with the keyboard. Run `forgum help` immediately.",
    "Astronomical fumble. Did you genuinely believe that subcommand existed, or were you just hoping the Rust compiler would take pity on your soul?",
    "Honest assessment: that was an absolute disgrace of an argument. Even raw shell scripts have higher syntax standards than whatever you just typed.",
    "You had ONE job: type a valid subcommand. Instead you gave us this syntactic catastrophe. The pasture is temporarily closed to amateur typists!",
    "Bro is out here inventing brand-new CLI dialects in real time. Unfortunately for you, Forgum only speaks actual commands. Try `forgum doctor` for emotional support.",
    "Look at this ASCII mascot gazing upon you in sheer, unadulterated disrespect. Even MS-DOS 1.0 had more command discipline than this.",
    "Stop spamming imaginary flags like it's a scratch-off lottery ticket! Type `forgum list` and educate yourself before you embarrass us all again.",
    "Genuinely tragic execution. If command syntax was a sport, you just scored an own goal from 50 yards out. Run `forgum --help` to reclaim your dignity.",
];

/// Render an honest, brutal, funky meme disgrace taunt box.
#[must_use]
pub fn render_brutal_taunt(streak: usize) -> String {
    let quote_idx = (streak.saturating_sub(3)) % DISGRACE_QUOTES.len();
    let quote = DISGRACE_QUOTES[quote_idx];

    // Select mascot eyes based on disgrace severity
    let (eyes, expression, mascot_art) = match streak {
        3 => (
            "==",
            "Deadpan Contempt",
            r#"        \   ^__^
         \  (==)\_______
            (__)\       )\/\
             U  ||----w |
                ||     ||"#,
        ),
        4 => (
            "><",
            "Visceral Agony",
            r#"        \   ^__^
         \  (><)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#,
        ),
        5 => (
            "xx",
            "Terminal Brainrot",
            r#"        \   ^__^
         \  (xx)\_______
            (__)\       )\/\
             || ||----w |
                ||     ||"#,
        ),
        _ => (
            "oO",
            "Clown Circus Mode 🤡",
            r#"        \   ^__^   (Honk Honk!)
         \  (oO)\_______
            (__)\       )\/\
             U  ||--🤡-w |
                ||     ||"#,
        ),
    };

    let wrapped_lines = wrap_speech_bubble_text(quote, 60);
    let max_len = wrapped_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20);
    let border_len = max_len + 2;

    let mut bubble = String::new();
    bubble.push_str(&format!("  {}\n", "_".repeat(border_len)));

    if wrapped_lines.len() == 1 {
        bubble.push_str(&format!(
            " / {:<width$} \\\n",
            wrapped_lines[0],
            width = max_len
        ));
    } else {
        for (i, line) in wrapped_lines.iter().enumerate() {
            if i == 0 {
                bubble.push_str(&format!(" / {:<width$} \\\n", line, width = max_len));
            } else if i == wrapped_lines.len() - 1 {
                bubble.push_str(&format!(" \\ {:<width$} /\n", line, width = max_len));
            } else {
                bubble.push_str(&format!(" | {:<width$} |\n", line, width = max_len));
            }
        }
    }
    bubble.push_str(&format!("  {}\n", "-".repeat(border_len)));

    format!(
        "\x1b[1;31m━━━━━━━━━━━━━━━━━━━━━ 💀 FORGUM DISGRACE RADAR 💀 ━━━━━━━━━━━━━━━━━━━━━\x1b[0m\n\
         \x1b[1;33m⚠️  CONSECUTIVE FAILURES: {streak} | Mascot Reaction: {expression} [{eyes}]\x1b[0m\n\n\
         {bubble}\
         {mascot_art}\n\n\
         \x1b[90mTip: Type `forgum --help` or `forgum list` to recover your dignity.\x1b[0m\n\
         \x1b[1;31m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m\n"
    )
}

fn wrap_speech_bubble_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(text.to_string());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_streak_lifecycle() {
        reset_failure_streak();
        assert_eq!(get_failure_streak(), 0);

        let s1 = record_and_get_failure_streak();
        assert_eq!(s1, 1);
        assert_eq!(get_failure_streak(), 1);

        let s2 = record_and_get_failure_streak();
        assert_eq!(s2, 2);

        let s3 = record_and_get_failure_streak();
        assert_eq!(s3, 3);

        reset_failure_streak();
        assert_eq!(get_failure_streak(), 0);
    }

    #[test]
    fn test_render_brutal_taunt_contains_disgrace_elements() {
        let taunt3 = render_brutal_taunt(3);
        assert!(taunt3.contains("FORGUM DISGRACE RADAR"));
        assert!(taunt3.contains("CONSECUTIVE FAILURES: 3"));
        assert!(taunt3.contains("forgum --help"));

        let taunt5 = render_brutal_taunt(5);
        assert!(taunt5.contains("CONSECUTIVE FAILURES: 5"));
        assert!(taunt5.contains("xx"));

        let taunt10 = render_brutal_taunt(10);
        assert!(taunt10.contains("CONSECUTIVE FAILURES: 10"));
    }

    #[test]
    fn test_wrap_speech_bubble_text() {
        let wrapped = wrap_speech_bubble_text("Short text", 20);
        assert_eq!(wrapped, vec!["Short text"]);

        let wrapped_long = wrap_speech_bubble_text(
            "This is a much longer sentence that should split across multiple lines neatly",
            20,
        );
        assert!(wrapped_long.len() > 1);
        for line in wrapped_long {
            assert!(line.chars().count() <= 20);
        }
    }
}
