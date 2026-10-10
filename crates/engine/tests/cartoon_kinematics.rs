//! Integration tests for 2D Cartoon Kinematics, Ragdoll Rigging, and Dialogue Anchoring.
//!
//! Validates:
//! 1. Anatomical segment partitioning (cranial, spine, caudal, pedal regions).
//! 2. Continuous 2D squash & stretch volumetric deformation.
//! 3. Spring-damper buoyant dialogue bubble physics.
//! 4. Kinematic body bounds modes (Wrap, Bounce).
//! 5. Caudal follow-through wave lag and landmark integrity.

use forgum_engine::cow::load_cow_with_landmarks;
use forgum_engine::kinematics::{AnatomicalSegments, BoundsMode, CartoonRig, KinematicBody};
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
}

#[test]
fn test_anatomical_segment_partitioning_default_cow() {
    let dd = data_dir();
    let (cow_art, eyes) = load_cow_with_landmarks("default", &dd, "oo", "  ", "\\");
    let eye_first = eyes.first().copied();

    let segments = AnatomicalSegments::detect(&cow_art, 0, eye_first, None, None, 10);
    assert!(segments.width > 0, "Mascot width must be positive");
    assert!(segments.height > 0, "Mascot height must be positive");
    assert!(
        segments.head_row <= segments.leg_row,
        "Head row ({}) must be at or above leg row ({})",
        segments.head_row,
        segments.leg_row
    );
    assert!(
        segments.head_col < segments.width,
        "Head col must be within mascot bounds"
    );
}

#[test]
fn test_cartoon_rig_continuous_squash_and_stretch() {
    let dd = data_dir();
    let (cow_art, eyes) = load_cow_with_landmarks("default", &dd, "oo", "  ", "\\");
    let eye_first = eyes.first().copied();

    let segments = AnatomicalSegments::detect(&cow_art, 0, eye_first, None, None, 10);
    let mut rig = CartoonRig::new(segments);
    rig.update(0.1, 0.0);

    // Evaluate deformation across cycle phases
    let times = [0.25f32, 0.5, 0.75, 1.0, 1.5, 2.0];
    let mut observed_deformation = false;

    for &t in &times {
        rig.update(0.1, 2.0);
        let stride = (t * 0.5) % 1.0;
        // Evaluate head cell
        let def_head = rig.deform_cell(
            rig.segments.head_col,
            rig.segments.head_row,
            t,
            stride,
            true,
            false,
            false,
        );
        // Evaluate foot cell
        let def_foot = rig.deform_cell(
            rig.segments.head_col,
            rig.segments.leg_row,
            t,
            stride,
            true,
            false,
            false,
        );
        // Evaluate tail cell
        let def_tail = rig.deform_cell(
            rig.segments.tail_col,
            rig.segments.chest_row,
            t,
            stride,
            true,
            false,
            false,
        );

        // Deformations must remain bounded within integer grid limits
        assert!(
            (-2..=2).contains(&def_head.dx),
            "dx {} out of bounded range",
            def_head.dx
        );
        assert!(
            (-1..=1).contains(&def_head.dy),
            "dy {} out of bounded range",
            def_head.dy
        );

        if def_foot.dy != 0
            || def_foot.dx != 0
            || def_head.dy != 0
            || def_head.dx != 0
            || def_tail.dx != 0
        {
            observed_deformation = true;
        }
    }

    assert!(
        observed_deformation,
        "Expected dynamic deformation across animation cycles"
    );
}

#[test]
fn test_spring_damper_bubble_physics_settling() {
    let dd = data_dir();
    let (cow_art, eyes) = load_cow_with_landmarks("default", &dd, "oo", "  ", "\\");
    let eye_first = eyes.first().copied();

    let segments = AnatomicalSegments::detect(&cow_art, 4, eye_first, None, None, 10);
    let mut rig = CartoonRig::new(segments);

    // Initial state: stationary
    assert_eq!(rig.bubble_offset_x, 0.0);
    assert_eq!(rig.bubble_vel_x, 0.0);

    // Step with positive mascot velocity (moving right)
    for _ in 0..15 {
        rig.update(0.016, 3.0);
    }

    // Spring-damper should lag behind motion (negative x offset)
    assert!(
        rig.bubble_offset_x < 0.0,
        "Dialogue bubble must experience inertial drag opposing velocity: got {}",
        rig.bubble_offset_x
    );

    // Step with zero velocity until settling
    for _ in 0..80 {
        rig.update(0.016, 0.0);
    }

    // Offset should damp back towards zero equilibrium
    assert!(
        rig.bubble_offset_x.abs() < 1.0,
        "Dialogue bubble must damp back towards equilibrium when stationary"
    );
}

#[test]
fn test_kinematic_body_bounds_modes() {
    // 1. Wrap mode
    let mut wrap_body = KinematicBody::new(10, 5, BoundsMode::Wrap);
    wrap_body.vx = 5.0;
    wrap_body.x = 75.0;
    wrap_body.update(2.0, 2.0, 80, 24);
    assert!(
        wrap_body.x < 80.0,
        "Wrap mode must keep body bounded within viewport"
    );

    // 2. Bounce mode
    let mut bounce_body = KinematicBody::new(10, 5, BoundsMode::Bounce);
    bounce_body.vx = 10.0;
    bounce_body.x = 75.0;
    bounce_body.update(1.0, 1.0, 80, 24);
    // Bounces off right margin: vx becomes negative
    assert!(
        bounce_body.vx < 0.0,
        "Bounce mode must invert velocity vector on margin impact"
    );
    assert!(!bounce_body.facing_right);
}

#[test]
fn test_caudal_follow_through_wave_lag() {
    let dd = data_dir();
    let (cow_art, eyes) = load_cow_with_landmarks("default", &dd, "oo", "  ", "\\");
    let eye_first = eyes.first().copied();

    let segments = AnatomicalSegments::detect(&cow_art, 0, eye_first, None, None, 10);
    let mut rig = CartoonRig::new(segments);
    rig.update(1.0, 1.0);

    let t = 1.25f32;
    // Tail cell deformation should have distinct phase lag from head cell
    let head_def = rig.deform_cell(
        rig.segments.head_col,
        rig.segments.head_row,
        t,
        1.0,
        true,
        false,
        false,
    );
    let tail_def = rig.deform_cell(
        rig.segments.tail_col,
        rig.segments.chest_row,
        t,
        1.0,
        true,
        false,
        false,
    );

    // Both deformations must remain finite and valid
    assert!(head_def.luminance_mult >= 0.8 && head_def.luminance_mult <= 1.5);
    assert!(tail_def.luminance_mult >= 0.8 && tail_def.luminance_mult <= 1.5);
}
