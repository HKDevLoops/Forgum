//! Integration test: render correctness, specifically the stale-frame bug (BUG-A).
//!
//! The render pipeline builds the next frame into `back` (via effects calling
//! `fb.set`) then `compute_damage()` and `AnsiRenderer::render_damage`. The
//! renderer MUST read the caller-provided cells, not a stale buffer.

use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{AnsiRenderer, Renderer};

#[test]
fn renderer_emits_cells_from_slice() {
    let mut fb = FrameBuffer::new(10, 5);
    fb.set(3, 2, Cell::new('X', Color::WHITE));

    let damage = fb.compute_damage().to_vec();
    assert!(damage.contains(&(3, 2)), "damage must include (3,2)");

    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer
        .render_damage(&mut out, &fb.back, fb.cols(), &damage)
        .unwrap();
    let s = String::from_utf8(out).unwrap();

    assert!(
        s.contains('X'),
        "back-buffer cell 'X' missing from output: {s}"
    );
}

#[test]
fn get_back_vs_get_semantics() {
    let mut fb = FrameBuffer::new(4, 4);
    fb.set(1, 1, Cell::new('A', Color::WHITE));
    assert_eq!(fb.get_back(1, 1).ch, 'A');
    assert_eq!(fb.get(1, 1).ch, ' ');

    fb.swap();
    // After swap: front has 'A' (the last back), back is the old empty front.
    assert_eq!(fb.get(1, 1).ch, 'A');

    fb.set(2, 2, Cell::new('B', Color::WHITE));
    assert_eq!(fb.get_back(2, 2).ch, 'B');
    assert_eq!(fb.get(2, 2).ch, ' ');
}

#[test]
fn render_damage_noop_on_empty_damage() {
    let fb = FrameBuffer::new(10, 5);
    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer
        .render_damage(&mut out, &fb.back, fb.cols(), &[])
        .unwrap();
    assert!(out.is_empty());
}

#[test]
fn rule_c_convex_hull_occlusion_integrity() {
    use forgum_engine::effects::{Effect, StaticEffect};

    // 1. Create a 30x10 framebuffer completely flooded with background scenery characters ('#')
    let mut fb = FrameBuffer::new(30, 10);
    for y in 0..10 {
        for x in 0..30 {
            fb.set(x, y, Cell::new('#', Color::rgb(0, 255, 0)));
        }
    }

    // 2. An ASCII animal with internal whitespace between bounding edges:
    // Row 0: "   (o o)   " (bounding hull: col 3 to col 7; interior col 4 is ' ')
    // Row 1: "  /|   |\\  " (bounding hull: col 2 to col 8; interior cols 4..6 are ' ')
    // Row 2: "   d   b   " (bounding hull: col 3 to col 7; interior cols 4..6 are ' ')
    let cow_art = "   (o o)   \n  /|   |\\  \n   d   b   ";
    let effect = StaticEffect::new(cow_art.to_string(), "plain".to_string());

    // 3. Render effect on top of the dense '#' scenery at time = 0.0
    effect.render(&mut fb, 0.0);

    // 4. Assert outside hull cells PRESERVE the background scenery '#'
    assert_eq!(
        fb.get_back(0, 0).ch,
        '#',
        "Outside hull cell (0,0) must preserve scenery"
    );
    assert_eq!(
        fb.get_back(1, 0).ch,
        '#',
        "Outside hull cell (1,0) must preserve scenery"
    );
    assert_eq!(
        fb.get_back(2, 0).ch,
        '#',
        "Outside hull cell (2,0) must preserve scenery"
    );
    assert_eq!(
        fb.get_back(8, 0).ch,
        '#',
        "Outside hull cell (8,0) must preserve scenery"
    );
    assert_eq!(
        fb.get_back(0, 1).ch,
        '#',
        "Outside hull cell (0,1) must preserve scenery"
    );
    assert_eq!(
        fb.get_back(1, 1).ch,
        '#',
        "Outside hull cell (1,1) must preserve scenery"
    );
    assert_eq!(
        fb.get_back(9, 1).ch,
        '#',
        "Outside hull cell (9,1) must preserve scenery"
    );

    // Hull border cells must contain animal chars
    assert_eq!(fb.get_back(3, 0).ch, '(');
    assert_eq!(fb.get_back(7, 0).ch, ')');
    assert_eq!(fb.get_back(2, 1).ch, '/');
    assert_eq!(fb.get_back(8, 1).ch, '\\');

    // Rule C Occlusion: INTERIOR whitespace cells between hull bounds must be masked with ' ', NOT '#'
    assert_eq!(
        fb.get_back(5, 0).ch,
        ' ',
        "Rule C violated: scenery '#' bled into space between animal eyes (5,0)"
    );
    for x in 4..=6 {
        assert_eq!(
            fb.get_back(x, 1).ch,
            ' ',
            "Rule C violated: scenery '#' bled into animal torso at col {x}, row 1"
        );
    }
}

#[test]
fn nature_mathematics_invariants() {
    use forgum_engine::dna::BaseAnim;
    use forgum_engine::scenery::{
        calculate_inter_tree_distance, calculate_mountain_height, calculate_mountain_peaks,
        calculate_multi_tree_span, calculate_tree_count, calculate_tree_height,
        calculate_tree_position, EnvironmentStyle, MountainStyle,
    };

    // 1. Peak count scales deterministically with viewport width
    let peaks_narrow = calculate_mountain_peaks(40, MountainStyle::Peaks);
    let peaks_medium = calculate_mountain_peaks(80, MountainStyle::Peaks);
    let peaks_wide = calculate_mountain_peaks(160, MountainStyle::Peaks);
    assert!(peaks_wide >= peaks_medium && peaks_medium >= peaks_narrow);

    // 2. Elevation profiles stay strictly clamped within viewport boundaries
    for x in 0..120 {
        let elev = calculate_mountain_height(x as f32, 120, 20, MountainStyle::Peaks, 0.0);
        assert!(
            elev >= 0.0 && elev <= 20.0,
            "Elevation out of bounds at x={x}: {elev}"
        );
    }

    // 3. Fibonacci phyllotaxis tree spacing is strictly positive and monotonic
    let d0 = calculate_inter_tree_distance(0, 16.0, 12.0);
    let d1 = calculate_inter_tree_distance(1, 16.0, 12.0);
    assert!((10.0..=32.0).contains(&d0));
    assert!((10.0..=32.0).contains(&d1));

    let p0 = calculate_tree_position(0, 16.0, 12.0);
    let p1 = calculate_tree_position(1, 16.0, 12.0);
    let p2 = calculate_tree_position(2, 16.0, 12.0);
    assert!(p1 > p0 && p2 > p1);

    let span = calculate_multi_tree_span(0, 2, 16.0, 12.0);
    assert_eq!(span, p2 - p0);

    // 4. Animal-relative canopy scaling: airborne vs ground creatures
    let walk_h = calculate_tree_height(0, 10, BaseAnim::Walk, 25);
    let fly_h = calculate_tree_height(0, 10, BaseAnim::Fly, 25);
    assert!(
        walk_h > fly_h,
        "Ground animal trees should be taller than airborne canopy"
    );

    // 5. Environmental tree density contrasts
    let forest_count = calculate_tree_count(100, EnvironmentStyle::Forest, 8);
    let savanna_count = calculate_tree_count(100, EnvironmentStyle::Savanna, 8);
    let city_count = calculate_tree_count(100, EnvironmentStyle::City, 8);
    assert!(forest_count > savanna_count);
    assert_eq!(city_count, 0);
}
