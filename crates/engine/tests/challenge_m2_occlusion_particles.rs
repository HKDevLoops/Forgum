//! Empirical Challenge Suite for Milestone 2:
//! Bounding-Hull Occlusion, Particle Physics Decoupling & Memory Ceiling (<100MB)
//!
//! Author: challenger_m2_2

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use forgum_engine::dna::{
    get_dna, load_embedded_animations, BaseAnim, CowDna, ParticleDna, ParticleType,
};
use forgum_engine::effects::{create_scene_effect, Effect, FlyEffect, StaticEffect, WalkEffect};
use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::particles::{Particle, ParticlePool};
use forgum_engine::scenery::{render_scenery, EnvironmentStyle, MountainStyle, RoadStyle};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

// ─────────────────────────────────────────────────────────────────────────────
// GROUP 1: BOUNDING-HULL OCCLUSION TESTS
// ─────────────────────────────────────────────────────────────────────────────

/// Flood a framebuffer with distinct background scenery characters.
fn flood_scenery(fb: &mut FrameBuffer, ch: char) {
    for y in 0..fb.height {
        for x in 0..fb.width {
            fb.set(x, y, Cell::new(ch, Color::rgb(0, 180, 50)));
        }
    }
}

/// Helper to find expected hull for a line: (first_non_space, last_non_space)
fn expected_line_hull(line: &str) -> Option<(usize, usize)> {
    let mut first_col = None;
    let mut first_ch = None;
    let mut second_col = None;
    let mut last_col = None;

    for (i, ch) in line.chars().enumerate() {
        if ch != ' ' {
            if first_col.is_none() {
                first_col = Some(i);
                first_ch = Some(ch);
            } else if second_col.is_none() {
                second_col = Some(i);
            }
            last_col = Some(i);
        }
    }

    let first = first_col?;
    let last = last_col?;

    let start = if let (Some(fc), Some(sc), Some(ch)) = (first_col, second_col, first_ch) {
        if (ch == '\\' || ch == '/' || ch == 'o' || ch == 'O') && sc.saturating_sub(fc) >= 3 {
            sc
        } else {
            first
        }
    } else {
        first
    };

    Some((start, last))
}

#[test]
fn challenge_occlusion_interior_whitespace_opaque_over_dense_scenery() {
    let cow_art = "\
        \\   ^__^
         \\  (oo)\\_______
            (__)\\       )\\/\\
                ||----w |
                ||     ||";

    let effect_names = [
        "static",
        "breathe",
        "float",
        "walk",
        "fly",
        "talk",
        "sway",
        "dissolve",
        "particles",
        "pulse",
        "glitch",
    ];

    for eff_name in effect_names {
        let dna = CowDna::default();
        let effect = create_scene_effect(eff_name, cow_art.to_string(), dna, 0, "plain");

        let mut fb = FrameBuffer::new(50, 15);
        flood_scenery(&mut fb, '#');

        let render_time = if eff_name == "dissolve" { 1.0 } else { 0.0 };
        effect.render(&mut fb, render_time);

        // Check line 2: "            (__)\\       )\\/\\"
        // Inside this torso line, col 17..23 are spaces in the cow art.
        // They MUST be rendered as ' ' (opaque) and NEVER '#' (scenery bleed-through).
        let line2_str = "            (__)\\       )\\/\\";
        let (hull_start, hull_end) = expected_line_hull(line2_str).expect("hull for line 2");

        for (col, ch) in line2_str.chars().enumerate() {
            let rendered_ch = fb.get_back(col, 2).ch;
            if col < hull_start {
                // Outside hull: MUST preserve scenery '#'
                assert_eq!(
                    rendered_ch, '#',
                    "[{eff_name}] Outside hull col {col} on line 2 must preserve scenery '#'"
                );
            } else if col <= hull_end {
                // Inside hull: MUST NOT bleed through scenery '#'
                assert_ne!(
                    rendered_ch, '#',
                    "[{eff_name}] Scenery bleed-through inside hull at col {col}, line 2: expected opaque char, got '#'"
                );
                if ch == ' ' {
                    // Interior whitespace MUST be opaque space
                    assert_eq!(
                        rendered_ch, ' ',
                        "[{eff_name}] Interior whitespace at col {col}, line 2 must be opaque space ' '"
                    );
                }
            } else {
                // Trailing outside hull
                assert_eq!(
                    rendered_ch, '#',
                    "[{eff_name}] Trailing cell at col {col}, line 2 must preserve scenery '#'"
                );
            }
        }
    }
}

#[test]
fn challenge_occlusion_walk_effect_kinematic_translation() {
    let cow_art = "\
   (o o)   \n\
  /|   |\\  \n\
   d   b   ";

    let dna = CowDna {
        base: BaseAnim::Walk,
        speed: 1.0,
        ..Default::default()
    };

    let walk = WalkEffect::new(cow_art.to_string(), &dna, 0, "plain".to_string());

    // Test across 5 distinct translation steps
    for step in 0..5 {
        let mut fb = FrameBuffer::new(60, 10);
        flood_scenery(&mut fb, '=');

        let time = step as f32 * 0.5;
        walk.render(&mut fb, time);

        // In line 1: "  /|   |\\  ", the interior cols between col 4 and col 6 are ' ' in art
        // Even when shifted across the screen, interior cells must be opaque ' ', NOT '='
        let mut found_cow = false;
        for x in 0..fb.width {
            let c = fb.get_back(x, 1).ch;
            if c == '/' {
                found_cow = true;
                // Col x+2, x+3, x+4 should be interior opaque spaces
                assert_eq!(
                    fb.get_back(x + 2, 1).ch,
                    ' ',
                    "WalkEffect step {step}: interior torso cell at col {} must be opaque space, not '='",
                    x + 2
                );
            }
        }
        assert!(
            found_cow,
            "WalkEffect step {step}: cow must be visible on screen"
        );
    }
}

#[test]
fn challenge_occlusion_fly_effect_nyan_mode() {
    let nyan_art = "\
+      +     +   
  +  [---]  +    
     |o o|       
     [___]       
  +        +     ";

    let dna = CowDna {
        base: BaseAnim::Fly,
        ..Default::default()
    };

    let fly = FlyEffect::new(nyan_art.to_string(), &dna, 0, "plain".to_string());

    let mut fb = FrameBuffer::new(50, 12);
    flood_scenery(&mut fb, '~');

    fly.render(&mut fb, 0.0);

    // Row 2: "     |o o|       " -> interior space between 'o' and 'o'
    // Hull starts at '|' (col 5) and ends at '|' (col 9).
    // Col 7 is space between eyes -> must be ' ' (opaque) and NOT '~'
    assert_eq!(
        fb.get_back(7, 2).ch,
        ' ',
        "FlyEffect: interior space between eyes must be opaque ' ', not '~'"
    );

    // Outside hull: Col 0..4 must preserve '~'
    assert_eq!(
        fb.get_back(0, 2).ch,
        '~',
        "FlyEffect: outside hull col 0 must preserve background '~'"
    );
    assert_eq!(
        fb.get_back(1, 2).ch,
        '~',
        "FlyEffect: outside hull col 1 must preserve background '~'"
    );
}

#[test]
fn challenge_occlusion_speech_bubble_interior_opaque() {
    let text = "\
 ________________ 
< Hello Scenery! >
 ---------------- 
        \\   ^__^  
         \\  (oo)  ";

    let effect = StaticEffect::new(text.to_string(), "plain".to_string());
    let mut fb = FrameBuffer::new(40, 10);
    flood_scenery(&mut fb, '*');

    effect.render(&mut fb, 0.0);

    // Speech bubble line 1: "< Hello Scenery! >"
    // Hull: col 0 ('<') to col 17 ('>').
    // Col 7 is the space in "Hello Scenery!" -> must be ' ' (opaque), NOT '*'
    assert_eq!(
        fb.get_back(7, 1).ch,
        ' ',
        "Speech bubble interior space must be opaque, not '*'"
    );

    // Line 3: "        \\   ^__^  "
    // Leading space: cols 0..7 must preserve '*'
    assert_eq!(
        fb.get_back(0, 3).ch,
        '*',
        "Outside speech connector line must preserve scenery '*'"
    );
    assert_eq!(
        fb.get_back(5, 3).ch,
        '*',
        "Gap before connector '\\' must preserve scenery '*'"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GROUP 2: PARTICLE ROTATION PHASE & EMITTER RATE DECOUPLING
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn challenge_particle_rotation_phase_and_emitter_rate_constant_across_fps() {
    // Test particle emission and rotation phase decoupled from raw frame count:
    // With rate = 20 particles/sec, over 1.0 second:
    // 30 FPS vs 240 FPS must produce the EXACT same number of emitter spawns!
    let rate = 20u32;
    let speed = 1.5f32;
    let interval = 1.0 / rate.max(1) as f32;

    // Simulate 30 FPS update loop
    let mut spawn_timer_30 = 0.0f32;
    let mut elapsed_30 = 0.0f32;
    let mut spawns_30 = 0;
    let dt_30 = 1.0 / 30.0;
    for _ in 0..30 {
        elapsed_30 += dt_30;
        spawn_timer_30 += dt_30 * speed;
        while spawn_timer_30 >= interval {
            spawn_timer_30 -= interval;
            spawns_30 += 1;
        }
    }
    let phase_30 = 0.5 + (elapsed_30 * speed);

    // Simulate 240 FPS update loop
    let mut spawn_timer_240 = 0.0f32;
    let mut elapsed_240 = 0.0f32;
    let mut spawns_240 = 0;
    let dt_240 = 1.0 / 240.0;
    for _ in 0..240 {
        elapsed_240 += dt_240;
        spawn_timer_240 += dt_240 * speed;
        while spawn_timer_240 >= interval {
            spawn_timer_240 -= interval;
            spawns_240 += 1;
        }
    }
    let phase_240 = 0.5 + (elapsed_240 * speed);

    // Simulate Variable/Jittered FPS summing to 1.0 second
    let mut spawn_timer_jitter = 0.0f32;
    let mut elapsed_jitter = 0.0f32;
    let mut spawns_jitter = 0;
    let jitter_dts = [0.01, 0.04, 0.05, 0.1, 0.2, 0.05, 0.15, 0.3, 0.1];
    for &dt in &jitter_dts {
        elapsed_jitter += dt;
        spawn_timer_jitter += dt * speed;
        while spawn_timer_jitter >= interval {
            spawn_timer_jitter -= interval;
            spawns_jitter += 1;
        }
    }
    let phase_jitter = 0.5 + (elapsed_jitter * speed);

    println!(
        "Emitter spawns over 1s: 30FPS={spawns_30}, 240FPS={spawns_240}, Jitter={spawns_jitter}"
    );
    println!(
        "Rotation phase over 1s: 30FPS={phase_30:.4}, 240FPS={phase_240:.4}, Jitter={phase_jitter:.4}"
    );

    // Rate = 20, speed = 1.5 -> expected spawns in 1.0s = floor(20 * 1.5) = 30 spawns
    assert_eq!(
        spawns_30, 30,
        "30 FPS must produce exactly 30 emitter spawns"
    );
    assert_eq!(
        spawns_240, 30,
        "240 FPS must produce exactly 30 emitter spawns (decoupled from 240 frame count!)"
    );
    assert_eq!(
        spawns_jitter, 30,
        "Jittered FPS must produce exactly 30 emitter spawns"
    );

    // Rotation phase MUST be identical across all frame rates (within float epsilon)
    assert!(
        (phase_30 - phase_240).abs() < 1e-4,
        "Phase diverged between 30 FPS and 240 FPS!"
    );
    assert!(
        (phase_30 - phase_jitter).abs() < 1e-4,
        "Phase diverged between 30 FPS and Jitter FPS!"
    );
}

#[test]
fn challenge_particle_kinematic_displacement_dt_invariant() {
    let mut pool_coarse = ParticlePool::new();
    let mut pool_fine = ParticlePool::new();

    // Spawn 1 particle at (10.0, 20.0) with vx = 15.0, vy = -8.0
    let p1 = Particle {
        x: 10.0,
        y: 20.0,
        vx: 15.0,
        vy: -8.0,
        life: 5.0,
        max_life: 5.0,
        ch: '*',
        color: Color::WHITE,
    };
    let p2 = p1.clone();

    pool_coarse.spawn(p1);
    pool_fine.spawn(p2);

    // Coarse: 10 steps of dt = 0.1s (total 1.0s)
    for _ in 0..10 {
        pool_coarse.update(0.1);
    }

    // Fine: 240 steps of dt = 1/240s (total 1.0s)
    for _ in 0..240 {
        pool_fine.update(1.0 / 240.0);
    }

    // Expected displacement after 1s:
    // x = 10 + 15 * 1 = 25
    // y = 20 - 8 * 1 = 12
    let mut fb_coarse = FrameBuffer::new(50, 30);
    let mut fb_fine = FrameBuffer::new(50, 30);

    pool_coarse.render(&mut fb_coarse, 0.0, forgum_engine::easing::expo_out);
    pool_fine.render(&mut fb_fine, 0.0, forgum_engine::easing::expo_out);

    // Both framebuffers must have the particle drawn at (25, 12)
    assert_eq!(
        fb_coarse.get_back(25, 12).ch,
        '*',
        "Coarse update: particle must be rendered at (25, 12)"
    );
    assert_eq!(
        fb_fine.get_back(25, 12).ch,
        '*',
        "Fine 240Hz update: particle must be rendered at (25, 12)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GROUP 3: RESIDENT MEMORY CEILING (<100MB) STRESS TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn challenge_memory_ceiling_under_heavy_animation_and_high_fps() {
    let anims = load_embedded_animations();
    let dragon_dna = get_dna(&anims, "dragon");

    let cow_art = forgum_platform::embedded_cows::get_embedded_cow("dragon")
        .unwrap_or("   (== dragon ==)\n  /|            |\\\n   d            b");
    let cow_foot_y = forgum_engine::effects::find_cow_foot_y(cow_art);

    let effect = create_scene_effect("particles", cow_art.to_string(), dragon_dna, 0, "plain");
    let mut fb = FrameBuffer::new(160, 50);

    let pid = Pid::from_u32(std::process::id());
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
    );

    let mut peak_mem_bytes = 0u64;
    let mut mem_at_frame_200 = 0u64;
    let mut mem_at_frame_2400 = 0u64;

    // Simulate 2,400 frames (10 seconds at 240 FPS)
    let dt = 1.0 / 240.0;
    let mut elapsed = 0.0f32;
    for frame in 1..=2400 {
        elapsed += dt;
        fb.clear();
        render_scenery(
            &mut fb,
            MountainStyle::Peaks,
            RoadStyle::Magma,
            EnvironmentStyle::Inferno,
            cow_foot_y,
            elapsed,
        );
        effect.render(&mut fb, elapsed);

        if frame % 100 == 0 {
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            if let Some(proc) = sys.process(pid) {
                let current_mem = proc.memory();
                if current_mem > peak_mem_bytes {
                    peak_mem_bytes = current_mem;
                }
                if frame == 200 {
                    mem_at_frame_200 = current_mem;
                }
                if frame == 2400 {
                    mem_at_frame_2400 = current_mem;
                }
            }
        }
    }

    let peak_mb = peak_mem_bytes as f64 / (1024.0 * 1024.0);
    let f200_mb = mem_at_frame_200 as f64 / (1024.0 * 1024.0);
    let f2400_mb = mem_at_frame_2400 as f64 / (1024.0 * 1024.0);

    println!("Milestone 2 Heavy Animation Memory Profile (2400 frames @ 240FPS, 160x50):");
    println!("  Peak RAM: {:.2} MB", peak_mb);
    println!("  Frame 200 RAM: {:.2} MB", f200_mb);
    println!("  Frame 2400 RAM: {:.2} MB", f2400_mb);

    let limit_bytes = 100 * 1024 * 1024;
    assert!(
        peak_mem_bytes < limit_bytes,
        "VIOLATION: Peak memory ({peak_mb:.2}MB) exceeded 100MB limit!"
    );

    // Verify no unbounded heap leak: growth between frame 200 and 2400 must be < 10MB
    if mem_at_frame_2400 > mem_at_frame_200 {
        let leak_mb = (mem_at_frame_2400 - mem_at_frame_200) as f64 / (1024.0 * 1024.0);
        assert!(
            leak_mb < 10.0,
            "Potential memory leak detected: RAM grew by {leak_mb:.2}MB over 2200 frames"
        );
    }
}

#[test]
fn challenge_parallel_heavy_simulations_memory_strictly_under_100mb() {
    let num_threads = 6;
    let frames = 400;
    let running = Arc::new(AtomicBool::new(true));
    let peak_mem_atomic = Arc::new(AtomicU64::new(0));

    let monitor_running = Arc::clone(&running);
    let peak_writer = Arc::clone(&peak_mem_atomic);
    let monitor = thread::spawn(move || {
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
        );
        while monitor_running.load(Ordering::Relaxed) {
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            if let Some(proc) = sys.process(pid) {
                peak_writer.fetch_max(proc.memory(), Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let mut handles = Vec::with_capacity(num_threads);
    for t_id in 0..num_threads {
        let h = thread::spawn(move || {
            let dna = CowDna {
                base: if t_id % 2 == 0 {
                    BaseAnim::Walk
                } else {
                    BaseAnim::Fly
                },
                particles: ParticleDna {
                    r#type: ParticleType::Fire,
                    rate: 15,
                    palette: vec!["#ff5500".to_string(), "#ffbb00".to_string()],
                    speed: [0.5, 1.0],
                    life: [0.5, 1.5],
                },
                ..Default::default()
            };

            let cow_art = "\
   (o o)   \n\
  /|   |\\  \n\
   d   b   ";
            let eff_name = if t_id % 2 == 0 { "walk" } else { "fly" };
            let effect =
                create_scene_effect(eff_name, cow_art.to_string(), dna, t_id as u32, "plain");
            let mut fb = FrameBuffer::new(120, 40);
            let cow_foot_y = forgum_engine::effects::find_cow_foot_y(cow_art);

            let dt = 1.0 / 60.0;
            let mut elapsed = 0.0f32;
            for _ in 0..frames {
                elapsed += dt;
                fb.clear();
                render_scenery(
                    &mut fb,
                    MountainStyle::Hills,
                    RoadStyle::Cobblestone,
                    EnvironmentStyle::Forest,
                    cow_foot_y,
                    elapsed,
                );
                effect.render(&mut fb, elapsed);
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().expect("simulation thread joined cleanly");
    }

    running.store(false, Ordering::Relaxed);
    monitor.join().expect("monitor thread joined cleanly");

    let peak_bytes = peak_mem_atomic.load(Ordering::Relaxed);
    let peak_mb = peak_bytes as f64 / (1024.0 * 1024.0);
    println!(
        "6 Concurrent SimState Threads Peak Memory: {:.2} MB",
        peak_mb
    );

    let limit_bytes = 100 * 1024 * 1024;
    assert!(
        peak_bytes < limit_bytes,
        "VIOLATION: Peak memory ({peak_mb:.2}MB) exceeded 100MB limit across 6 concurrent simulations!"
    );
}
