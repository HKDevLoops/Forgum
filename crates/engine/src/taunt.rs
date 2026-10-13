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

/// Curated honest, brutal, and ragebaiting funky meme disgrace quotes (65+ quotes).
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
    "Are you typing with your elbows, or is your cat taking a leisurely stroll across your mechanical keyboard? Either way, stop it.",
    "I've seen dial-up modems establish handshakes with more elegance than whatever argument trainwreck you just executed.",
    "The cow refuses to render for someone who treats CLI syntax like a game of Mad Libs. Run `forgum cows` and look at what you could have had.",
    "Congratulations! You just invented a brand new error code that doesn't even exist in the POSIX standard. That takes true dedication to failure.",
    "Do you need a hug, a glass of warm milk, or just a copy of the README? Because this level of syntax griefing is concerning.",
    "Your shell prompt is not a wishing well. Typing random words into it will not magically grant you an animated dragon.",
    "Tux the Linux Penguin just waddled over, took one look at your command arguments, shook his head in silence, and went back to Antarctica.",
    "Somewhere in a datacenter, a server just thermal throttled itself out of sheer secondhand cringe from parsing your CLI arguments.",
    "Bro is treating `forgum` like a ChatGPT prompt box. This is a binary compiled in Rust, not your conversational therapist!",
    "If ignorance is bliss, you must be in an absolute state of nirvana right now. Run `forgum help` to exit the dreamworld.",
    "A moment of silence for your Backspace key, which clearly isn't being utilized anywhere near enough today.",
    "The dragon tried to breathe fire, but suffocated on your invalid arguments instead. Environmental disaster averted; pride demolished.",
    "You have achieved what computer scientists previously deemed mathematically impossible: zero percent syntax accuracy across multiple attempts.",
    "My brother in terminal, tab completion exists. It has existed since 1978. Please, for the love of Dijkstra, press the Tab key.",
    "Your terminal history has officially been flagged by the SysAdmin Council as suspicious activity. Reason: Unprecedented levels of typos.",
    "I would explain how CLI arguments work, but I lack the crayons and patience required for this specific interaction.",
    "Even an uninitialized C pointer has more direction and purpose in life than whatever flags you just passed into this terminal.",
    "Did you honestly type that, look at the screen, press Enter with full confidence, and expect a working mascot animation?",
    "The stegosaurus walked back into the Jurassic era because it couldn't bear to witness another one of your syntax errors.",
    "Legend says if you misspell `forgum` subcommands three more times, your terminal will forcibly revert back to Windows Command Prompt 95.",
    "Your CPU fan just spun up to 100% not from compute workload, but in a frantic attempt to blow your commands off the screen.",
    "Is this ragebait? Are you ragebaiting an ASCII mascot? Because the cow is unfazed, but the Rust runtime is deeply concerned.",
    "Blink twice if you need someone to type `--cow tux` for you. We can arrange a tutor if the mechanical keyboard is proving too formidable.",
    "Even Microsoft Clippy in 1997 would have popped up by now and said: 'It looks like you have no idea what you are doing. Would you like help?'",
    "You are treating this CLI like a microwave touchpad where you just mash 9-9-9 and hope for hot food. Type `forgum help`!",
    "The pasture grass has officially withered in protest. Mascots are on strike until you demonstrate a basic command line pulse.",
    "Syntax Error: Expected valid arguments, found pure unadulterated chaotic keyboard smashing. Consult `forgum list` immediately.",
    "I'm not mad, just disappointed. Actually, never mind, the cow is furious. Look at those deadpan eyes judging your ancestry.",
    "If mistakes build character, you are currently the most well-developed protagonist in the entire history of software engineering.",
    "A hamster running on a wheel hooked up to a UART terminal would have accidentally produced valid ANSI escapes faster than this.",
    "Every time you type an invalid flag, a junior developer somewhere loses their git commit privileges out of cosmic sympathy.",
    "You are navigating this CLI like someone driving on the highway with their headlights off, blindfolded, in reverse gear.",
    "Rust's borrow checker couldn't even save you from borrowing too much audacity to type those nonexistent subcommands.",
    "The ASCII cow just called a team meeting to discuss whether you should be allowed sudo privileges ever again.",
    "Breaking News: Local developer stuns the entire open-source ecosystem by entering 12th consecutive invalid argument in a row.",
    "Are you trying to brute-force our command parser? Good luck, it's written with clap and zero patience for nonsense.",
    "Please step away from the keyboard with your hands visible. The Terminal Police have been notified of ongoing syntax violence.",
    "I've seen segfaults with more dignity than the argument parsing failure you just subjected this engine to.",
    "The cow had a speech bubble prepared with life-changing wisdom, but you choked it out with typo debris. Run `forgum --help`!",
    "Your keystrokes are an insult to the semiconductor industry. Thousands of engineers built this microprocessor for you to type THAT?",
    "Why guess? Why speculate? Why invent words? The docs are literally at https://github.com/HKDevLoops/Forgum#configuration!",
    "At this point, we're not even running the animation engine. We're just keeping this process alive to document your descent into madness.",
    "The ghost mascot appeared, took one look at your flags, said 'Nah this is too spooky even for me', and de-spawned.",
    "You are the reason software warning labels exist. If this binary had an emergency brake, it would have engaged 4 commands ago.",
    "Even an echo chamber has more original and accurate responses than the typos you keep recycling into your command line.",
    "Take a deep breath. Count to ten. Drink some water. And for heaven's sake, type `forgum cows` instead of whatever that was.",
    "Your terminal emulator is actively petitioning your operating system to revoke your write permissions to `/dev/tty`.",
    "We ran your argument string through an AI classifier. Result: 99.8% probability of a keyboard malfunction or profound confusion.",
    "Stop trying to outsmart the CLI. The CLI is deterministic; your typos are purely self-inflicted chaos.",
    "The cow is not amused. The penguin is embarrassed. The dragon is wheezing. Please type `forgum --help` and end our collective misery.",
    "You have achieved peak clown status 🤡. The big top tent is erected, the calliope music is playing, and you are the star attraction.",
    "This is not a creative writing workshop. This is a terminal command line. Adhere to the established syntax or face the wrath of the herd!",
    "One does not simply walk into Mordor, nor does one simply invent their own Forgum subcommands without facing the disgrace box.",
];

/// Milestone thresholds for special cursed brutal responses.
fn get_cursed_milestone(
    streak: usize,
) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    if streak >= 100 {
        Some((
            "ΩΩ",
            "💀 TIER 100+: DEITY OF RAW SYNTAX DEGRADATION & ETERNAL CONDEMNATION 💀",
            r#"        \    /\_/\   👁️ 👁️ 👁️
         \  ( 👁️ ) /‾‾‾‾‾‾‾‾‾\
           (  Ω  )   VOID     )\/\
          /|     |\          /
         ( | ||| | )---w----'
           \ ||| /
            ^^^^^
     💀 [100 CONSECUTIVE FAILURES: CONDEMNED] 💀"#,
            "ONE HUNDRED CONSECUTIVE FAILURES. You have breached mortal comprehension. Linus Torvalds is having palpitations in his sleep. The ANSI escape parser has filed a union grievance. Your shell history has been declared an environmental biohazard. Step away from the computer, take a shower, touch actual living biological grass, and rethink every decision that brought you to this cursed threshold.",
        ))
    } else if streak >= 90 {
        Some((
            "§§",
            "🌀 TIER 90: MATRIX SIMULATION COLLAPSE & KERNEL MELTDOWN 🌀",
            r#"        \   ^̸_̸_̸^̸   ▲̸
         \  (§̸§̸)\̸_̸_̸_̸_̸_̸_̸_̸
            (__̸)\̸       )\̸/̸\̸
             ▓▓ ||----w |
                ||     ||
          [KERNEL PANIC: USER BRAINROT]"#,
            "Ninety consecutive times?! The simulation is literally fracturing around you. Memory leaks are forming out of sheer pity. At this rate, your terminal emulator is going to commit unbuffered seppuku just to escape your keystrokes.",
        ))
    } else if streak >= 80 {
        Some((
            "oo",
            "🐒 TIER 80: INFINITE MONKEY THEOREM DISPROVEN IN REAL TIME 🐒",
            r#"        \   .---.   (Facepalm)
         \ / 🐒  \
          | (o.o) |___
          |   -   |   \  ⌨️ [SMASHED]
          /       \    \
         (_________)---'"#,
            "Eighty straight misses. Mathematicians postulated that infinite monkeys at infinite typewriters would eventually reproduce Shakespeare. You have conclusively proven that even one human can fail to produce a single valid CLI flag eighty times in a row.",
        ))
    } else if streak >= 70 {
        Some((
            "☣☣",
            "☣️ TIER 70: CHRONIC CATASTROPHIC INCOMPETENCE EPIDEMIC ☣️",
            r#"        \   ^__^   ☣️ ☣️
         \  (☣☣)\_______
            (__)\  ☢️   )\/\
             || ||----w |
                ||     ||
          [CLASS-4 COGNITIVE HAZARD]"#,
            "Seventy failed attempts. The Centers for Disease Control has declared your typing technique a Class-4 Cognitive Hazard. Please put on hazmat gloves and stop terrorizing this terminal.",
        ))
    } else if streak == 69 {
        Some((
            "69",
            "♋ TIER 69: CURSED EDGELORD APEX — NOT EVEN NICE, JUST EMBARRASSING ♋",
            r#"        \  (\__/)  😈
         \ ( `v´ ) /‾‾‾‾‾‾‾\
           (  >69< )        )\/\
            U    U ||----w |
                   ||     ||
          (NOT EVEN NICE. JUST SAD.)"#,
            "Sixty-nine consecutive failures. You thought this would trigger a funny easter egg? No. You are just astronomically bad at command lines. There is no prize, no applause, only sixty-nine recorded instances of absolute syntax bankruptcy.",
        ))
    } else if streak >= 60 {
        Some((
            "..",
            "🔬 TIER 60: BIOLOGICAL MIRACLE OF SURVIVING WITH ZERO BRAINCELLS 🔬",
            r#"        \     .-''''-.
         \  .'  ..    '.  (Single Cell)
           /   (..)     \
          |              |
           \  .---.     /
            '.     .'--'"#,
            "Sixty failures. Neuroscientists are gathering from across the globe to study how you maintain involuntary respiratory functions while simultaneously exhibiting negative terminal literacy.",
        ))
    } else if streak >= 50 {
        Some((
            "@@",
            "🔮 TIER 50: TERMINAL SCHIZOPHRENIA & CLINICAL DELUSION 🔮",
            r#"        \   ^__^   *🌀*
         \  (@@)\_______
            (__)\       )\/\
             ~  ||-~-~w |
                ||     ||
          (HEAR THE VOICES?)"#,
            "Fifty consecutive errors. You are no longer interacting with a CLI; you are having a vivid hallucination. Who told you that flag existed? Was it the voices in your head? Drink water immediately.",
        ))
    } else if streak >= 40 {
        Some((
            "xx",
            "🚨 TIER 40: COURT-ORDERED RESTRAINING ORDER FROM STDIN 🚨",
            r#"        \   | ^__^ |
         \  |(xx)| |_______
            |(__)| |       )\/\
            | || | ||----w |
            | || | ||     ||
            [RESTRICTED FROM STDIN]"#,
            "Forty straight whiffs. The judge has signed the emergency injunction: you are legally barred from being within 500 feet of an active stdin pipe. Turn in your terminal privileges.",
        ))
    } else if streak >= 30 {
        Some((
            "o_",
            "🛑 TIER 30: CRITICAL SKILL DEFICIENCY & HARDWARE EMBARRASSMENT 🛑",
            r#"        \   .--.
         \ |o_o |  (SOBBING TUX)
           |:_/ |
          //   \ \
         (|     | )
        /'\_   _/`\
        \___)=(___/"#,
            "Thirty times in a row! Your CPU spent billions of transistors rendering nanometer-accurate Silicon gates, just for you to punch random consonants into a terminal like a drunken crab.",
        ))
    } else if streak >= 20 {
        Some((
            "oO",
            "🎪 TIER 20: ASTRONOMICAL DUMBASS ASCENSION & CIRCUS HEADLINER 🎪",
            r#"        \   .---.
         \ / 🤡  \  (HONK HONK)
          /|  O  |\/\/\/\
         / |     |      \
        *  ||---||--w   /
           ^^   ^^"#,
            "Twenty consecutive blunders! The circus called; they don't want you back because even the clowns were getting embarrassed by your act. Type `forgum help` before your shell uninstalls itself.",
        ))
    } else if streak >= 10 {
        Some((
            "##",
            "🧱 TIER 10: CERTIFIED BRICKED KEYBOARD & SENSORY DEPRIVATION 🧱",
            r#"        \   ^__^
         \  (##)\_______
            (__)\       )\/\
             🧱 ||----w |
                ||     ||
          [BRICKED BRAIN DETECTED]"#,
            "TEN CONSECUTIVE FAILURES. Double digits of pure unadulterated shame. Did a cat walk across your keyboard, or are your fingers genuinely made of concrete? Run `forgum list` right now.",
        ))
    } else {
        None
    }
}

/// Render an honest, brutal, funky meme disgrace taunt box.
#[must_use]
pub fn render_brutal_taunt(streak: usize) -> String {
    let (eyes, expression, mascot_art, quote) = if let Some(tier) = get_cursed_milestone(streak) {
        tier
    } else {
        let quote_idx = (streak.saturating_sub(3)) % DISGRACE_QUOTES.len();
        let quote = DISGRACE_QUOTES[quote_idx];

        // Select mascot art based on disgrace severity & variety
        let (eyes, expression, mascot_art) = match (streak.saturating_sub(3)) % 6 {
            0 => (
                "==",
                "Deadpan Contempt",
                r#"        \   ^__^
         \  (==)\_______
            (__)\       )\/\
             U  ||----w |
                ||     ||"#,
            ),
            1 => (
                "><",
                "Visceral Agony",
                r#"        \   ^__^
         \  (><)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#,
            ),
            2 => (
                "xx",
                "Terminal Brainrot",
                r#"        \   ^__^
         \  (xx)\_______
            (__)\       )\/\
             || ||----w |
                ||     ||"#,
            ),
            3 => (
                "oO",
                "Clown Circus Mode 🤡",
                r#"        \   ^__^   (Honk Honk!)
         \  (oO)\_______
            (__)\       )\/\
             U  ||--🤡-w |
                ||     ||"#,
            ),
            4 => (
                "@@",
                "Shell-Shocked Tux",
                r#"        \   .--.
         \ |@_@ |  (Facepalm)
           |:_/ |
          //   \ \
         (|     | )
        /'\_   _/`\"#,
            ),
            _ => (
                "XX",
                "Hellfire Ragebait",
                r#"        \   ^__^   🔥
         \  (XX)\_______
            (__)\  🔥   )\/\
             || ||----w |
                ||     ||"#,
            ),
        };
        (eyes, expression, mascot_art, quote)
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
        assert!(taunt10.contains("TIER 10"));
    }

    #[test]
    fn test_cursed_milestone_tiers() {
        for &milestone in &[10, 20, 30, 40, 50, 60, 69, 70, 80, 90, 100, 150] {
            let res = render_brutal_taunt(milestone);
            assert!(
                res.contains(&format!("CONSECUTIVE FAILURES: {milestone}")),
                "Streak {milestone} should be in output"
            );
            assert!(
                res.contains("TIER"),
                "Streak {milestone} should contain milestone tier badge"
            );
        }

        let res69 = render_brutal_taunt(69);
        assert!(res69.contains("TIER 69"));
        assert!(res69.contains("CURSED EDGELORD APEX"));

        let res100 = render_brutal_taunt(100);
        assert!(res100.contains("TIER 100+"));
        assert!(res100.contains("DEITY OF RAW SYNTAX DEGRADATION"));
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
