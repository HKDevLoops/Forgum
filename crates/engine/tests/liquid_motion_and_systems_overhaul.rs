//! Comprehensive invariant tests for 240Hz liquid motion, decoupled simulation,
//! 2D AsciiRigMap morphing, ambient occlusion moat, and security hardening.

use forgum_engine::engine_core::{LiquidFrameGovernor, FIXED_SIM_DT};
use forgum_engine::image_ascii::{MAX_IMAGE_DIMENSION, MAX_IMAGE_FILE_SIZE};
use forgum_engine::init::{shell_escape_arg, Shell};
use forgum_engine::kinematics::{
    apply_ambient_occlusion_moat, morph_ascii_glyph, AsciiRigMap, CellMask,
};
use std::time::Duration;

#[test]
fn test_liquid_frame_governor_dt_independence() {
    let mut gov_60 = LiquidFrameGovernor::new(60);
    assert_eq!(
        gov_60.target_period(),
        Duration::from_nanos(1_000_000_000 / 60)
    );

    let mut gov_240 = LiquidFrameGovernor::new(240);
    assert_eq!(
        gov_240.target_period(),
        Duration::from_nanos(1_000_000_000 / 240)
    );

    let mut physics_steps_60 = 0usize;
    let _ = gov_60.step_simulation(|dt| {
        physics_steps_60 += 1;
        assert!((dt - (1.0 / 120.0)).abs() < 1e-6);
    });

    let mut physics_steps_240 = 0usize;
    let _ = gov_240.step_simulation(|dt| {
        physics_steps_240 += 1;
        assert!((dt - (1.0 / 120.0)).abs() < 1e-6);
    });

    assert!((FIXED_SIM_DT - (1.0 / 120.0)).abs() < 1e-6);
}

#[test]
fn test_morph_ascii_glyph_articulation() {
    // Forward limb shear: \ -> | -> /
    assert_eq!(morph_ascii_glyph('\\', -0.5, 0.0), '|');
    assert_eq!(morph_ascii_glyph('\\', -1.1, 0.0), '/');

    // Backward limb shear: / -> | -> \
    assert_eq!(morph_ascii_glyph('/', 0.5, 0.0), '|');
    assert_eq!(morph_ascii_glyph('/', 1.1, 0.0), '\\');

    // Upstroke wing shear: v -> ^
    assert_eq!(morph_ascii_glyph('v', 0.0, 0.6), '^');
    assert_eq!(morph_ascii_glyph('v', 0.0, 0.25), '~');

    // Downstroke wing shear: ^ -> v
    assert_eq!(morph_ascii_glyph('^', 0.0, -0.6), 'v');
    assert_eq!(morph_ascii_glyph('^', 0.0, -0.25), '_');

    // Non-articulated characters remain unchanged
    assert_eq!(morph_ascii_glyph('M', 0.5, 0.5), 'M');
    assert_eq!(morph_ascii_glyph('o', 0.5, 0.5), 'o');
}

#[test]
fn test_apply_ambient_occlusion_moat() {
    let bg_color = (200, 200, 200);

    // High-frequency mountain/tree slashes are suppressed to whitespace
    let (ch_slash, col_slash) = apply_ambient_occlusion_moat('/', bg_color, (0, 0, 0));
    assert_eq!(ch_slash, ' ');
    // Luminance dimmed by ~68% (factor 0.32)
    assert!(col_slash.0 < 100);
    assert!(col_slash.1 < 100);
    assert!(col_slash.2 < 100);

    let (ch_pipe, _) = apply_ambient_occlusion_moat('|', bg_color, (0, 0, 0));
    assert_eq!(ch_pipe, ' ');

    // Normal characters retain glyph but are dimmed
    let (ch_char, col_char) = apply_ambient_occlusion_moat('A', bg_color, (0, 0, 0));
    assert_eq!(ch_char, 'A');
    assert!(col_char.0 < 100);
}

#[test]
fn test_ascii_rig_map_concavity_and_hull_preservation() {
    let ascii_art = "  +------+  \n  | (oo) |  \n  +------+  \n    ||||    ";

    let segments = forgum_engine::kinematics::AnatomicalSegments::default();
    let rig_map = AsciiRigMap::build(ascii_art, 0, &segments);
    assert!(rig_map.width > 0);
    assert!(rig_map.height > 0);

    // Top-left whitespace must be exterior transparent or moat
    let mask_top_left = rig_map.cells[0].mask;
    assert!(
        mask_top_left == CellMask::ExteriorTransparent || mask_top_left == CellMask::ExteriorMoat,
        "Top-left must be exterior, got {:?}",
        mask_top_left
    );

    // The glyphs must be glyph opaque
    assert!(rig_map
        .cells
        .iter()
        .any(|c| c.mask == CellMask::GlyphOpaque));
    // The enclosed interior torso cavity must be interior opaque
    assert!(rig_map
        .cells
        .iter()
        .any(|c| c.mask == CellMask::InteriorOpaque));
}

#[test]
fn test_shell_escape_arg_security() {
    // Benign alphanumeric arguments are not modified
    assert_eq!(shell_escape_arg(Shell::Bash, "standard"), "standard");
    assert_eq!(
        shell_escape_arg(Shell::Bash, "--flag=value"),
        "--flag=value"
    );

    // Arguments with spaces or shell metacharacters are safely quoted
    let malicious = "hello; rm -rf /";
    let escaped_bash = shell_escape_arg(Shell::Bash, malicious);
    assert_eq!(escaped_bash, "'hello; rm -rf /'");

    let single_quote_arg = "don't fail";
    let escaped_posix = shell_escape_arg(Shell::Bash, single_quote_arg);
    assert_eq!(escaped_posix, "'don'\\''t fail'");

    let escaped_pwsh = shell_escape_arg(Shell::PowerShell, single_quote_arg);
    assert_eq!(escaped_pwsh, "'don''t fail'");
}

#[test]
fn test_image_ascii_security_bounds() {
    assert_eq!(MAX_IMAGE_FILE_SIZE, 25 * 1024 * 1024);
    assert_eq!(MAX_IMAGE_DIMENSION, 4096);
}
