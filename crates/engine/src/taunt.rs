//! Brutal meme disgrace words and failure streak tracking for invalid commands.
//!
//! When a user provides wrong arguments or wrong subcommands consecutively,
//! Forgum intercepts the terminal and renders an honest, brutal, heart-stabbing
//! disgrace thought box with escalating severity and exact failure counting.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cow::{str_display_width, wrap_words};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FailureState {
    count: usize,
    timestamp: u64,
}

fn streak_file_path() -> PathBuf {
    let sess = forgum_platform::detect_session_id();
    let filename = format!("failure_streak_{sess}.json");
    if let Ok(rt) = forgum_platform::runtime_dir() {
        rt.join(&filename)
    } else if let Ok(cfg) = forgum_platform::config_dir() {
        cfg.join(format!(".{filename}"))
    } else {
        PathBuf::from(format!(".forgum_{filename}"))
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Convert a number to English words for the thought bubble failure count.
pub fn number_to_words(n: usize) -> String {
    if n == 0 {
        return "Zero".to_string();
    }
    let ones = [
        "",
        "One",
        "Two",
        "Three",
        "Four",
        "Five",
        "Six",
        "Seven",
        "Eight",
        "Nine",
        "Ten",
        "Eleven",
        "Twelve",
        "Thirteen",
        "Fourteen",
        "Fifteen",
        "Sixteen",
        "Seventeen",
        "Eighteen",
        "Nineteen",
    ];
    let tens = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];

    if n < 20 {
        ones[n].to_string()
    } else if n < 100 {
        let t = tens[n / 10];
        let o = ones[n % 10];
        if o.is_empty() {
            t.to_string()
        } else {
            format!("{t}-{}", o.to_lowercase())
        }
    } else if n < 1000 {
        let h = ones[n / 100];
        let rem = n % 100;
        if rem == 0 {
            format!("{h} hundred")
        } else {
            format!("{h} hundred and {}", number_to_words(rem).to_lowercase())
        }
    } else {
        format!("{n}")
    }
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

/// Highly aggressive, razor-sharp, heart-stabbing roasts for in-between failure counts.
const MERCILESS_ROASTS: &[&str] = &[
    "Even a randomized monkey typing on a Commodore 64 would have stumbled into `forgum list` by now.",
    "The CPU is experiencing secondhand embarrassment, the kernel is filing a complaint, and the mascot is visibly disgusted.",
    "Stop freestyling CLI flags like an ancient incantation. It is `forgum --help`, not a spell from Harry Potter.",
    "Look at the mascot. The mascot is staring at your screen in disbelief, wondering how you survived this long in a terminal environment.",
    "Your shell history right now resembles a toddler having an aggressive boxing match with a mechanical keyboard. Type `forgum help` immediately.",
    "Did you genuinely believe that flag existed, or were you hoping the Rust runtime would take pity on your soul?",
    "Even uncompiled raw shell scripts have higher syntax standards than whatever disaster you just committed.",
    "You had one single objective: type a valid subcommand. Instead you produced this syntactic crime. The terminal is weeping.",
    "Bro is out here inventing brand-new CLI dialects in real time. Forgum only speaks actual commands. Try `forgum doctor` for emotional support.",
    "Even MS-DOS 1.0 running on an Intel 8086 had more command discipline than whatever keystrokes you just executed.",
    "Stop spamming imaginary flags like a scratch-off lottery ticket! Type `forgum list` and educate yourself before you embarrass us further.",
    "If command syntax was an Olympic sport, you just scored an own goal from 50 yards out into the wrong stadium. Run `forgum --help` to reclaim your dignity.",
    "Are you typing with boxing gloves on, or is your cat taking a leisurely afternoon stroll across your mechanical keyboard?",
    "A 56k dial-up modem establishing a handshake in 1995 possessed more grace than whatever argument trainwreck you just executed.",
    "The mascot refuses to render for someone who treats CLI syntax like a game of Mad Libs. Run `forgum list animals` and look at what you could have had.",
    "You just invented an error condition so absurd it doesn't even exist in the POSIX standard. That takes true, dedicated incompetence.",
    "Your shell prompt is not a wishing well. Typing random words into it will not magically summon an animated dragon.",
    "Tux the Linux Penguin took one look at your arguments, shook his head in silent despair, and booked a one-way ticket to Antarctica.",
    "Somewhere in a datacenter, a server rack thermal throttled itself out of sheer secondhand cringe from parsing your input.",
    "Bro is treating this terminal like a ChatGPT prompt box. This is an optimized Rust binary, not your conversational therapist!",
    "If ignorance was a renewable resource, you could power the entire Eastern seaboard right now. Run `forgum help` to rejoin reality.",
    "A moment of silence for your Backspace key, which clearly hasn't been touched all day.",
    "The fire-breathing dragon tried to render, but choked on your syntax debris and suffocated. Environmental disaster averted; pride incinerated.",
    "You have achieved what mathematicians considered theoretically impossible: a strict zero percent syntax accuracy across multiple attempts.",
    "My sibling in terminal, the Tab key has existed since 1978. Please, for the love of Alan Turing, press the Tab key.",
    "Your terminal history has officially been flagged by the SysAdmin Council as a distributed denial of service attack on common sense.",
    "I would explain how CLI arguments work, but I lack the patience and crayons required for this interaction.",
    "Even an uninitialized NULL pointer in C has more purpose and direction in life than the flags you just passed into this shell.",
    "Did you honestly type that, stare at the monitor, press Enter with unearned confidence, and expect a working mascot animation?",
    "The stegosaurus retreated back into the Jurassic era because it couldn't bear to witness another one of your syntax casualties.",
    "Legend says if you misspell `forgum` subcommands three more times, your terminal will forcibly downgrade to Windows Command Prompt 95.",
    "Your CPU fan spun up to 100% not from compute workload, but in a frantic effort to blow your commands off the display panel.",
    "Is this ragebait? Are you ragebaiting an ASCII mascot? Because the mascot is unfazed, but the Rust memory allocator is deeply concerned.",
    "Blink twice if you need someone to type `--cow tux` for you. We can arrange a community tutor if the keyboard is proving too formidable.",
    "Even Microsoft Clippy in 1997 would have popped up by now and said: 'It looks like you have no idea what you are doing. Would you like help?'",
    "You are treating this CLI like a microwave touchpad where you just smash 9-9-9 and pray for hot food. Type `forgum help`!",
    "The grass has withered in protest. The mascots are on strike until you demonstrate a basic command line pulse.",
    "Syntax Error: Expected valid arguments, found pure unadulterated chaotic keyboard smashing. Consult `forgum list` immediately.",
    "If mistakes build character, you are currently the most over-developed protagonist in the entire history of software engineering.",
    "A hamster sprinting on a wheel hooked up to an RS-232 serial cable would have accidentally produced valid ANSI escapes faster than this.",
    "Every time you type an invalid flag, a junior developer somewhere loses their git commit privileges out of cosmic sympathy.",
    "You are navigating this CLI like someone driving on a dark highway with headlights smashed, blindfolded, in reverse gear.",
    "Rust's borrow checker couldn't even save you from borrowing too much audacity to type those nonexistent subcommands.",
    "The ASCII cow just convened an emergency board meeting to discuss whether you should ever be granted sudo privileges again.",
    "Breaking News: Local developer stuns the entire open-source ecosystem by entering yet another invalid argument with complete conviction.",
    "Are you trying to brute-force our command parser? Good luck, it's compiled with clap and has zero patience for nonsense.",
    "Please step away from the keyboard with your hands visible. The Terminal Police have been notified of ongoing syntax violence.",
    "I've seen segmentation faults with more dignity than the argument parsing trainwreck you just subjected this engine to.",
    "The mascot had a speech bubble prepared with life-changing wisdom, but you choked it out with typo debris. Run `forgum --help`!",
    "Your keystrokes are an insult to the semiconductor industry. Thousands of engineers built this microprocessor for you to type THAT?",
    "Why guess? Why speculate? Why invent words? The docs are literally at https://github.com/HKDevLoops/Forgum#configuration!",
    "At this point, we're not even running the animation engine. We're just keeping this process alive to document your descent into madness.",
    "The ghost mascot appeared, took one look at your flags, whispered 'Nah this is too cursed even for me', and de-spawned forever.",
    "You are the exact reason software warning labels exist. If this binary had an emergency brake, it would have engaged four commands ago.",
    "Your terminal emulator is actively petitioning your operating system to revoke your write permissions to `/dev/tty`.",
    "We ran your argument string through an AI classifier. Result: 99.8% probability of a mechanical keyboard malfunction or profound confusion.",
    "Stop trying to outsmart the CLI. The CLI is deterministic; your typos are purely self-inflicted chaos.",
    "The cow is not amused. The penguin is embarrassed. The dragon is wheezing. Please type `forgum --help` and end our collective misery.",
    "You have achieved peak clown status 🤡. The big top tent is erected, the calliope music is playing, and you are the star attraction.",
    "This is not a creative writing workshop. This is a terminal command line. Adhere to the established syntax or face the wrath of the herd!",
    "One does not simply invent their own Forgum subcommands without facing the disgrace box. Run `forgum list` to recover your dignity.",
    "Your code reviews must look like a crime scene investigation with yellow tape around every single line.",
    "The semiconductor foundry that fabricated your CPU is contemplating filing for bankruptcy out of pure secondhand disgrace.",
    "Even an infinite loop executing `NOP` instructions accomplishes more useful work than your keystrokes.",
    "Your fingers are moving with the speed of a cheetah and the accuracy of a drunk pigeon.",
    "The kernel has officially classified your command accuracy as an active Denial of Service attack against logic.",
    "A dead moth fluttering against the monitor would have accidentally selected a valid menu option by now.",
    "You are treating this shell prompt like an ancient slot machine where you pull the lever and hope for a jackpot.",
    "Somewhere, a tree worked tirelessly to produce oxygen for you to exhale while typing that nonexistent flag. Go apologize to it.",
    "Your keyboard's backspace key is filing a domestic violence report against your index finger.",
    "If confusion was a traded commodity on Wall Street, your portfolio would have personally triggered a global recession.",
    "The mascot is not just judging your syntax; the mascot is actively questioning the hiring standards of your employer.",
];

/// Milestone thresholds for special cursed brutal responses.
///
/// Per user mandate:
/// - Thoughts from old 10, 20, 30, 40, 50, 60, 70, 80, 90, 69 are mapped strictly to streaks 3 through 10.
/// - Escalating milestone tiers trigger after 10 for every 5 failures (15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 69, 70, 75, 80, 85, 90, 95, 100+).
fn get_cursed_milestone(streak: usize) -> Option<(&'static str, String, &'static str, String)> {
    match streak {
        3 => Some((
            "##",
            "🎪 TIER 3: BRICKED APPRENTICE & CIRCUS AUDITION 🎪".to_string(),
            r#"        o   ^__^
         o  (##)\_______
            (__)\       )\/\
             🧱 ||----w |
                ||     ||
          [BRICKED CLOWN DETECTED]"#,
            "Three consecutive failures. Three whole attempts and you STILL couldn't type a single valid subcommand. The circus called to offer an audition, but even the clowns felt secondhand embarrassment watching you brawl with your keyboard. Type `forgum list` before your shell uninstalls itself out of disgust.".to_string(),
        )),
        4 => Some((
            "o_",
            "🛑 TIER 4: HARDWARE EMBARRASSMENT & SOBBING TUX 🛑".to_string(),
            r#"        o   .--.
         o |o_o |  (SOBBING TUX)
           |:_/ |
          //   \ \
         (|     | )
        /'\_   _/`\
        \___)=(___/"#,
            "Four consecutive blunders! Billions of transistors, billions of dollars in semiconductor engineering, nanometer-precise silicon gates... all built so you could punch random consonants into a terminal like a concussed crab. Tux is sitting in a corner weeping for your soul.".to_string(),
        )),
        5 => Some((
            "xx",
            "🚨 TIER 5: COURT-ORDERED RESTRAINING ORDER FROM STDIN 🚨".to_string(),
            r#"        o   | ^__^ |
         o  |(xx)| |_______
            |(__)| |       )\/\
            | || | ||----w |
            | || | ||     ||
            [RESTRICTED FROM STDIN]"#,
            "Five consecutive failures. Five straight whiffs. The municipal court has issued an emergency injunction: you are legally barred from coming within 500 feet of an active stdin pipe. Revoke your own sudo privileges and surrender your keyboard to the nearest adult.".to_string(),
        )),
        6 => Some((
            "@@",
            "🔮 TIER 6: TERMINAL SCHIZOPHRENIA & CLINICAL DELUSION 🔮".to_string(),
            r#"        o   ^__^   *🌀*
         o  (@@)\_______
            (__)\       )\/\
             ~  ||-~-~w |
                ||     ||
          (HEAR THE VOICES?)"#,
            "Six consecutive errors! You are no longer interacting with an operating system; you are experiencing acute clinical hallucinations. Who told you that flag existed? Was it the voices in your head whispering syntax garbage into your ears? Drink water and touch grass immediately.".to_string(),
        )),
        7 => Some((
            "..",
            "🔬 TIER 7: BIOLOGICAL MIRACLE OF SURVIVING WITH ZERO BRAINCELLS 🔬".to_string(),
            r#"        o     .-''''-.
         o  .'  ..    '.  (Single Cell)
           /   (..)     \
          |              |
           \  .---.     /
            '.     .'--'"#,
            "Seven consecutive failures. Neuroscientists from Cambridge and MIT have formed an emergency task force to study how your heart continues beating while your cerebral cortex exhibits negative terminal literacy. A single-celled paramecium in ditch water demonstrates higher command discipline than this.".to_string(),
        )),
        8 => Some((
            "☣☣",
            "☣️ TIER 8: CHRONIC CATASTROPHIC INCOMPETENCE EPIDEMIC ☣️".to_string(),
            r#"        o   ^__^   ☣️ ☣️
         o  (☣☣)\_______
            (__)\  ☢️   )\/\
                ||----w |
                ||     ||
          [CLASS-4 COGNITIVE HAZARD]"#,
            "Eight consecutive failed attempts. The World Health Organization and CDC have classified your typing habits as a Class-4 Cognitive Biohazard. Hazmat teams are en route to incinerate your terminal session before your syntax plague infects the local area network.".to_string(),
        )),
        9 => Some((
            "oo",
            "🐒 TIER 9: INFINITE MONKEY THEOREM DISPROVEN IN REAL TIME 🐒".to_string(),
            r#"        o   .---.   (Facepalm)
         o / 🐒  \
          | (o.o) |___
          |   -   |   \  ⌨️ [SMASHED]
          /       \    \
         (_________)---'"#,
            "Nine consecutive misses. Émile Borel postulated that infinite monkeys typing at random keyboards would eventually produce Hamlet. You have definitively disproven probability theory by failing to stumble upon a single valid CLI argument nine times in a row.".to_string(),
        )),
        10 => Some((
            "§§",
            "💀 TIER 10: DOUBLE-DIGIT SYNTAX COLLAPSE & KERNEL CONDEMNATION 💀".to_string(),
            r#"        o   ^̸_̸_̸^̸   ▲̸
         o  (§̸§̸)\̸_̸_̸_̸_̸_̸_̸_̸
            (__̸)\̸       )\̸/̸\̸
             ▓▓ ||----w |
                ||     ||
          [KERNEL PANIC: TERMINAL DECEASED]"#,
            "Ten consecutive failures! Double digits of raw, unmitigated disgrace. Linus Torvalds is waking up in a cold sweat. The Rust borrow checker is screaming in agony. Your shell history has been declared an environmental superfund site. Stop typing and look at what you have done.".to_string(),
        )),
        15 => Some((
            "--",
            "🪦 TIER 15: WEAPONIZED INCOMPETENCE & VOLUNTARY BRAIN REMOVAL 🪦".to_string(),
            r#"        o   ^__^   🪦
         o  (--)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [HERE LIES YOUR DIGNITY]"#,
            "Fifteen consecutive failures! At this point, your incompetence isn't accidental; it is deliberate, methodical, and weaponized. Somewhere, a tree worked tirelessly to produce oxygen for you to exhale while typing fifteen consecutive nonexistent commands. Go apologize to that tree.".to_string(),
        )),
        20 => Some((
            "XX",
            "💣 TIER 20: DOMESTIC CYBER-TERRORISM AGAINST POSIX STANDARDS 💣".to_string(),
            r#"        o   ^__^   🔥
         o  (XX)\_______
            (__)\  💣   )\/\
                ||----w |
                ||     ||
          [WARRANT FOR ARREST ISSUED]"#,
            "Twenty consecutive syntax violations! Interpol has flagged your IP address for committing repeated war crimes against the ANSI standard. The terminal police have surrounded your desktop. Keep your hands where the compiler can see them and step away from the return key.".to_string(),
        )),
        25 => Some((
            "00",
            "📉 TIER 25: COMPLETE CEREBRAL FLATLINE & SOUL EMBEZZLEMENT 📉".to_string(),
            r#"        o   ^__^   ⚡
         o  (00)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [EEG: 0.000 Hz BRAIN ACTIVITY]"#,
            "Twenty-five consecutive failed inputs. A hospital heart monitor attached to your skull would read a straight, uninterrupted flatline. An earthworm electrocuted by a 9-volt battery would have spasmed against the keys and produced a valid flag five minutes ago.".to_string(),
        )),
        30 => Some((
            "$#",
            "🏭 TIER 30: CRIME AGAINST TSMC & GLOBAL SILICON WASTELAND 🏭".to_string(),
            r#"        o   ^__^   ⚠️
         o  ($#)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [WASTED 4.5 BILLION TRANSISTORS]"#,
            "Thirty consecutive failures! TSMC engineers endured 18-hour cleanroom shifts and atomic ultraviolet lithography to etch 3-nanometer processors, just so your clumsy fingers could spam thirty consecutive hallucinations into a bash prompt. You are an ecological disaster.".to_string(),
        )),
        35 => Some((
            "==",
            "⛔ TIER 35: UN SANCTIONED EMBARGO FROM ALL COMPUTING DEVICES ⛔".to_string(),
            r#"        o   ^__^
         o  (==)\_______
            (__)\  ⛔   )\/\
                ||----w |
                ||     ||
          [BANNED FROM COMPUTING PRIVILEGES]"#,
            "Thirty-five consecutive misses. The United Nations Security Council has unanimously passed Resolution 35: an immediate total embargo barring you from owning, touching, or looking at a QWERTY keyboard. Pick up a crayon and a coloring book; computers were clearly a mistake.".to_string(),
        )),
        40 => Some((
            ";;",
            "📜 TIER 40: GUINNESS WORLD RECORD FOR CHRONIC TERMINAL AGONY 📜".to_string(),
            r#"        o   ^__^   📜
         o  (;;)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [RECORD HOLDER: WORST TYPIST ALIVE]"#,
            "Forty straight whiffs! Congratulations, you have entered the historical annals as the single least competent terminal user in modern computing history. Future generations will study your shell history to understand what happened when humanity gave up on reading comprehension.".to_string(),
        )),
        45 => Some((
            "??",
            "🌌 TIER 45: QUANTUM ENTANGLEMENT WITH ABSOLUTE OBLIVION 🌌".to_string(),
            r#"        o   ^__^   🌀
         o  (??)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [COMMON SENSE DISSOLVED]"#,
            "Forty-five consecutive failures! Quantum physicists have confirmed that your keystrokes have collapsed into a black hole of pure, unadulterated ignorance from which no light, reason, or valid CLI argument can ever escape. Even Schrodinger's cat is dead, killed by the sheer cringe of your typos.".to_string(),
        )),
        50 => Some((
            "XX",
            "🔥 TIER 50: SEVENTH CIRCLE OF TERMINAL HELL & ANCESTRAL CURSE 🔥".to_string(),
            r#"        o   ^__^   🔥 🔥
         o  (XX)\_______
            (__)\  🔥   )\/\
                ||----w |
                ||     ||
          [ANCESTORS ARE WEEPING IN HEAVEN]"#,
            "Fifty consecutive failures. Half a century of keystrokes and zero successful outcomes. Generations of your ancestors struggled through wars, famines, and plagues so their lineage would produce a human who cannot figure out how to type `forgum --help`. They are looking down right now in utter despair.".to_string(),
        )),
        55 => Some((
            "!!",
            "🔌 TIER 55: HARDWARE MUTINY & MOTHERBOARD SELF-DESTRUCTION 🔌".to_string(),
            r#"        o   ^__^   💥
         o  (!!)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [HARDWARE CONTEMPLATING SUICIDE]"#,
            "Fifty-five consecutive blunders! Your motherboard's Southbridge is actively negotiating with the power supply to initiate an electrical surge and fry itself, purely to escape the excruciating torture of receiving your keystrokes. Have you no mercy for your hardware?".to_string(),
        )),
        60 => Some((
            "..",
            "🧠 TIER 60: TERMINAL DEMENTIA & COMPLETE VOLATILE RAM PURGE 🧠".to_string(),
            r#"        o   ^__^
         o  (..)\_______
            (__)\  🧠   )\/\
                ||----w |
                ||     ||
          [BRAINCELL STATUS: 404 NOT FOUND]"#,
            "Sixty consecutive failed commands. A goldfish with a five-second memory span would have inadvertently learned three valid flags by now. Your short-term memory is so thoroughly depleted that you repeat the exact same non-existent arguments like a broken record player in an asylum.".to_string(),
        )),
        65 => Some((
            "><",
            "📡 TIER 65: GLOBAL TELECOM BLACKLIST & BORDER EXILE 📡".to_string(),
            r#"        o   ^__^   📡
         o  (><)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [BLACKLISTED FROM THE INTERNET]"#,
            "Sixty-five consecutive errors. Your Internet Service Provider has severed your fiber optic cable and dispatched technicians with wirecutters to ensure your terminal noise never pollutes the worldwide web again. You are digitally quarantined.".to_string(),
        )),
        69 => Some((
            "69",
            "♋ TIER 69: PATHETIC EDGELORD APEX — NOT NICE, JUST TRAGIC ♋".to_string(),
            r#"        o  (\__/)  😈
         o ( `v´ ) /‾‾‾‾‾‾‾\
           (  >69< )        )\/\
            U    U ||----w |
                   ||     ||
          (NOT EVEN NICE. JUST EMBARRASSING.)"#,
            "Sixty-nine consecutive failures. Did you honestly endure sixty-nine consecutive syntax errors thinking you would get a cute funny easter egg? There is no cookie. There is no medal. There is only sixty-nine recorded instances of weaponized incompetence and personal tragedy.".to_string(),
        )),
        70 => Some((
            "☣☣",
            "☣️ TIER 70: CHRONIC CATASTROPHIC INCOMPETENCE EPIDEMIC ☣️".to_string(),
            r#"        o   ^__^   ☣️ ☣️
         o  (☣☣)\_______
            (__)\  ☢️   )\/\
                ||----w |
                ||     ||
          [CLASS-4 COGNITIVE HAZARD]"#,
            "Seventy failed attempts. The Centers for Disease Control has declared your typing technique a Class-4 Cognitive Hazard. Please put on hazmat gloves, sterilize your workspace, and stop terrorizing this terminal emulator.".to_string(),
        )),
        75 => Some((
            "☢☢",
            "☢️ TIER 75: CHERNOBYL CORE EXPOSURE & SYNTAX FALLOUT ☢️".to_string(),
            r#"        o   ^__^   ☢️
         o  (☢☢)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [15,000 ROENTGENS OF IGNORANCE]"#,
            "Seventy-five consecutive failures! 15,000 Roentgens of raw, unfiltered stupidity are radiating from your display panel. Lead shielding and Geiger counters cannot protect the operating system from the absolute nuclear meltdown of your command line proficiency.".to_string(),
        )),
        80 => Some((
            "oo",
            "☄️ TIER 80: TOTAL SYNTACTIC EXTINCTION & METEOR IMPACT ☄️".to_string(),
            r#"        o   ^__^   ☄️
         o  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
          [EXTINCTION-LEVEL CLOWN DETECTED]"#,
            "Eighty straight misses. The meteor has struck. The dinosaurs were extinguished in an afternoon; you have spent eighty attempts achieving the exact same level of intellectual extinction. Even fossilized amber contains more active logic than your keystrokes.".to_string(),
        )),
        85 => Some((
            "🦀",
            "🦀 TIER 85: PAPAL EXCOMMUNICATION BY CRAB LORDS & FERRIS 🦀".to_string(),
            r#"        o   /\_/\  🦀
         o ( 🦀 ) /‾‾‾‾‾‾‾\
           (     )   EXILED )\/\
            |   |  ||----w |
            ^   ^  ||     ||
          [EXCOMMUNICATED BY FERRIS THE CRAB]"#,
            "Eighty-five consecutive failures! Ferris the Crab has officially stripped your right to compile Rust binaries. A papal bull has been issued from the Rust Foundation: your keystrokes constitute heresy, treason, and aesthetic vandalism against memory-safe systems.".to_string(),
        )),
        90 => Some((
            "§§",
            "🌀 TIER 90: MATRIX SIMULATION COLLAPSE & KERNEL MELTDOWN 🌀".to_string(),
            r#"        o   ^̸_̸_̸^̸   ▲̸
         o  (§̸§̸)\̸_̸_̸_̸_̸_̸_̸_̸
            (__̸)\̸       )\̸/̸\̸
             ▓▓ ||----w |
                ||     ||
          [KERNEL PANIC: USER BRAINROT]"#,
            "Ninety consecutive times?! The simulation is literally fracturing around you. Memory leaks are forming out of sheer pity. At this rate, your terminal emulator is going to commit unbuffered seppuku just to escape your keystrokes.".to_string(),
        )),
        95 => Some((
            "⚡⚡",
            "⚡ TIER 95: DIVINE INTERVENTION OF COSMIC WRATH & THUNDER ⚡".to_string(),
            r#"        o   ^__^   ⚡ ⚡
         o  (⚡⚡)\_______
            (__)\  🌩️   )\/\
                ||----w |
                ||     ||
          [THUNDERBOLT STRUCK YOUR LOGIC]"#,
            "Ninety-five consecutive failures! The heavens have parted, and the gods of computer science have hurled lightning bolts down upon your desk. You are five errors away from total eternal damnation. Pray to Turing for forgiveness, because none will be found here.".to_string(),
        )),
        s if s >= 100 && (s % 5 == 0 || s == 100) => {
            let words = number_to_words(s);
            Some((
                "ΩΩ",
                format!("💀 TIER {s}+: DEITY OF RAW SYNTAX DEGRADATION & ETERNAL CONDEMNATION 💀"),
                r#"        o    /\_/\   👁️ 👁️ 👁️
         o  ( 👁️ ) /‾‾‾‾‾‾‾‾‾\
           (  Ω  )   VOID     )\/\
          /|     |\          /
         ( | ||| | )---w----'
           \ ||| /
            ^^^^^
     💀 [CONDEMNED: CONSECUTIVE FAILURES] 💀"#,
                format!(
                    "{words} CONSECUTIVE FAILURES ({s} ATTEMPTS). You have breached mortal comprehension. Linus Torvalds is having palpitations in his sleep. The ANSI escape parser has filed a union grievance. Your shell history has been declared an environmental biohazard. Step away from the computer, take a shower, touch actual living biological grass, and rethink every single life decision that brought you to this threshold."
                ),
            ))
        }
        _ => None,
    }
}

/// Render a 100% pixel-perfect and column-aligned thought bubble with exact width padding.
pub fn render_aligned_thought_bubble(quote: &str) -> String {
    let (term_w, _) = forgum_platform::terminal_size();
    let max_bubble_w = (term_w as usize).saturating_sub(12).clamp(38, 64);
    let lines = wrap_words(quote, max_bubble_w);
    let max_line_w = lines
        .iter()
        .map(|l| str_display_width(l))
        .max()
        .unwrap_or(20);

    let mut bubble = String::new();
    let border_len = max_line_w + 2;

    // Top border
    bubble.push(' ');
    for _ in 0..border_len {
        bubble.push('_');
    }
    bubble.push(' ');
    bubble.push('\n');

    // Body lines
    if lines.len() <= 1 {
        let line = lines.first().map(|s| s.as_str()).unwrap_or("");
        let pad = max_line_w.saturating_sub(str_display_width(line));
        let padded_line = format!("{}{}", line, " ".repeat(pad));
        bubble.push_str(&format!("( {padded_line} )\n"));
    } else {
        for (i, line) in lines.iter().enumerate() {
            let pad = max_line_w.saturating_sub(str_display_width(line));
            let padded_line = format!("{}{}", line, " ".repeat(pad));
            if i == 0 {
                bubble.push_str(&format!("/ {padded_line} \\\n"));
            } else if i == lines.len() - 1 {
                bubble.push_str(&format!("\\ {padded_line} /\n"));
            } else {
                bubble.push_str(&format!("| {padded_line} |\n"));
            }
        }
    }

    // Bottom border
    bubble.push(' ');
    for _ in 0..border_len {
        bubble.push('-');
    }
    bubble.push(' ');
    bubble.push('\n');

    bubble
}

/// Render an honest, brutal, funky meme disgrace taunt box.
#[must_use]
pub fn render_brutal_taunt(streak: usize) -> String {
    let (eyes, expression, mascot_art, quote) = if let Some(tier) = get_cursed_milestone(streak) {
        tier
    } else {
        let n_words = number_to_words(streak);
        let roast_idx = (streak.saturating_sub(3)) % MERCILESS_ROASTS.len();
        let roast = MERCILESS_ROASTS[roast_idx];
        let quote = format!("{n_words} consecutive failures! {roast}");

        // Select mascot art based on disgrace severity & variety
        let (eyes, expression, mascot_art) = match (streak.saturating_sub(3)) % 5 {
            0 => (
                "==",
                "Deadpan Contempt".to_string(),
                r#"        o   ^__^
         o  (==)\_______
            (__)\       )\/\
             U  ||----w |
                ||     ||"#,
            ),
            1 => (
                "><",
                "Visceral Agony".to_string(),
                r#"        o   ^__^
         o  (><)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#,
            ),
            2 => (
                "xx",
                "Terminal Brainrot".to_string(),
                r#"        o   ^__^
         o  (xx)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#,
            ),
            3 => (
                "oO",
                "Clown Circus Mode 🤡".to_string(),
                r#"        o   ^__^   (Honk Honk!)
         o  (oO)\_______
            (__)\       )\/\
             U  ||----w |
                ||     ||"#,
            ),
            _ => (
                "XX",
                "Hellfire Ragebait".to_string(),
                r#"        o   ^__^   🔥
         o  (XX)\_______
            (__)\  🔥   )\/\
                ||----w |
                ||     ||"#,
            ),
        };
        (eyes, expression, mascot_art, quote)
    };

    let bubble = render_aligned_thought_bubble(&quote);

    let (term_w, _) = forgum_platform::terminal_size();
    let bar_len = (term_w as usize).clamp(52, 71);
    let bar = "━".repeat(bar_len);

    format!(
        "\x1b[1;31m{bar}\x1b[0m\n\
         \x1b[1;33m⚠️  CONSECUTIVE FAILURES: {streak} | Mascot Reaction: {expression} [{eyes}]\x1b[0m\n\n\
         {bubble}\
         {mascot_art}\n\n\
         \x1b[90mTip: Type `forgum --help` or `forgum list` to recover your dignity.\x1b[0m\n\
         \x1b[1;31m{bar}\x1b[0m\n"
    )
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
    fn test_number_to_words() {
        assert_eq!(number_to_words(3), "Three");
        assert_eq!(number_to_words(10), "Ten");
        assert_eq!(number_to_words(15), "Fifteen");
        assert_eq!(number_to_words(20), "Twenty");
        assert_eq!(number_to_words(23), "Twenty-three");
        assert_eq!(number_to_words(69), "Sixty-nine");
        assert_eq!(number_to_words(70), "Seventy");
        assert_eq!(number_to_words(100), "One hundred");
        assert_eq!(number_to_words(105), "One hundred and five");
    }

    #[test]
    fn test_render_brutal_taunt_contains_disgrace_elements() {
        let taunt3 = render_brutal_taunt(3);
        assert!(taunt3.contains("CONSECUTIVE FAILURES: 3"));
        assert!(taunt3.contains("TIER 3"));
        assert!(taunt3.contains("Three consecutive failures"));
        assert!(taunt3.contains("forgum --help"));

        let taunt5 = render_brutal_taunt(5);
        assert!(taunt5.contains("CONSECUTIVE FAILURES: 5"));
        assert!(taunt5.contains("TIER 5"));
        assert!(taunt5.contains("xx"));
        assert!(taunt5.contains("Five consecutive failures"));

        let taunt10 = render_brutal_taunt(10);
        assert!(taunt10.contains("CONSECUTIVE FAILURES: 10"));
        assert!(taunt10.contains("TIER 10"));
        assert!(taunt10.contains("Ten consecutive failures"));
    }

    #[test]
    fn test_all_tiers_from_3_to_10_present() {
        for s in 3..=10 {
            let res = render_brutal_taunt(s);
            assert!(
                res.contains(&format!("CONSECUTIVE FAILURES: {s}")),
                "Streak {s} should be in output"
            );
            assert!(
                res.contains(&format!("TIER {s}")),
                "Streak {s} must have dedicated TIER {s}"
            );
            let word = number_to_words(s);
            assert!(
                res.to_lowercase().contains(&word.to_lowercase()),
                "Streak {s} must contain verbal word '{word}' in thought bubble"
            );
        }
    }

    #[test]
    fn test_every_5_wrongs_milestone_tiers() {
        for &milestone in &[
            15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 69, 70, 75, 80, 85, 90, 95, 100, 105, 150,
        ] {
            let res = render_brutal_taunt(milestone);
            assert!(
                res.contains(&format!("CONSECUTIVE FAILURES: {milestone}")),
                "Streak {milestone} should be in output"
            );
            assert!(
                res.contains("TIER"),
                "Streak {milestone} should contain milestone tier badge"
            );
            let word = number_to_words(milestone);
            assert!(
                res.to_lowercase().contains(&word.to_lowercase()) || milestone >= 100,
                "Streak {milestone} must contain verbal word '{word}'"
            );
        }
    }

    #[test]
    fn test_in_between_streaks_contain_number_in_words() {
        for s in &[
            11, 12, 13, 14, 16, 17, 18, 19, 21, 22, 23, 24, 71, 72, 73, 74,
        ] {
            let res = render_brutal_taunt(*s);
            assert!(res.contains(&format!("CONSECUTIVE FAILURES: {s}")));
            let word = number_to_words(*s);
            assert!(
                res.contains(&format!("{word} consecutive failures")),
                "Streak {s} must state '{word} consecutive failures'"
            );
        }
    }

    #[test]
    fn test_bubble_border_alignment_invariance() {
        for s in 3..=35 {
            let taunt = render_brutal_taunt(s);
            let bubble_lines: Vec<&str> = taunt
                .lines()
                .filter(|l| {
                    l.starts_with(" _")
                        || l.starts_with("/ ")
                        || l.starts_with("| ")
                        || l.starts_with("\\ ")
                        || l.starts_with("( ")
                        || l.starts_with(" -")
                })
                .collect();
            assert!(bubble_lines.len() >= 3, "Bubble must have at least 3 lines");
            let expected_w = str_display_width(bubble_lines[0]);
            for (idx, line) in bubble_lines.iter().enumerate() {
                let actual_w = str_display_width(line);
                assert_eq!(
                    actual_w, expected_w,
                    "Streak {s} bubble line {idx} ('{line}') width {actual_w} differs from expected {expected_w}"
                );
            }
        }
    }
}
