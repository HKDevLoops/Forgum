use forgum_engine::battle::{Battle, BattlePhase, run_battle};

#[test]
fn test_battle_simulation_full_lifecycle() {
    let mut battle = Battle::with_dimensions("Bessie", "Daisy", 80, 14, 1);
    assert_eq!(battle.phase, BattlePhase::Charging);
    assert!(battle.cow1.alive);
    assert!(battle.cow2.alive);
    assert_eq!(battle.cow1.hp, 100);
    assert_eq!(battle.cow2.hp, 100);

    let mut frames = 0;
    while !battle.is_done() {
        battle.tick();
        frames += 1;
        assert!(frames < 200, "Battle took too many frames to complete");
    }

    assert_eq!(battle.phase, BattlePhase::Done);
    assert!(battle.cow1.alive, "Cow 1 should be alive when winner is 1");
    assert!(!battle.cow2.alive, "Cow 2 should be defeated when winner is 1");
    assert_eq!(battle.cow1.eyes, "^^");
    assert_eq!(battle.cow2.eyes, "xx");
    assert_eq!(battle.cow2.hp, 0);
}

#[test]
fn test_battle_simulation_winner2_lifecycle() {
    let mut battle = Battle::with_dimensions("Bessie", "Daisy", 80, 14, 2);
    while !battle.is_done() {
        battle.tick();
    }

    assert_eq!(battle.phase, BattlePhase::Done);
    assert!(!battle.cow1.alive, "Cow 1 should be defeated when winner is 2");
    assert!(battle.cow2.alive, "Cow 2 should be alive when winner is 2");
    assert_eq!(battle.cow1.eyes, "xx");
    assert_eq!(battle.cow2.eyes, "@@");
    assert_eq!(battle.cow1.hp, 0);
}

#[test]
fn test_battle_run_battle_fast_headless() {
    let start = std::time::Instant::now();
    let log = run_battle("Thunder", "Lightning");
    let elapsed = start.elapsed();

    // Headless simulation must execute in under 100ms
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "run_battle took too long: {:?}",
        elapsed
    );
    assert!(log.contains("Thunder"));
    assert!(log.contains("Lightning"));
    assert!(log.contains("Battle result:"));
    assert!(log.contains("defeats"));
}

#[test]
fn test_battle_compact_and_wide_arenas() {
    for width in [52, 60, 80, 100, 120] {
        let b = Battle::with_dimensions("A", "B", width, 14, 1);
        let frame = b.render_frame();
        let lines: Vec<&str> = frame.lines().collect();
        assert_eq!(lines.len(), 14, "Arena height must match 14 lines");
        for line in lines {
            assert!(
                line.chars().count() <= width + 5,
                "Line width exceeds arena bounds"
            );
        }
    }
}

#[test]
fn test_battle_colored_ansi_output() {
    let b = Battle::new("Champion", "Challenger");
    let ansi_frame = b.render_frame_styled(true);
    let plain_frame = b.render_frame_styled(false);

    assert!(ansi_frame.contains("\x1b[38;2;"));
    assert!(ansi_frame.contains("\x1b[0m"));
    assert!(!plain_frame.contains("\x1b["));
    assert!(plain_frame.contains("Champion"));
    assert!(plain_frame.contains("Challenger"));
}
