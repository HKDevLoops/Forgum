//! Integration test verifying terminal theme detection and background-adaptive contrast.

use forgum_platform::biome::natural_creature_color_adaptive;
use forgum_platform::terminal_theme::{calculate_luminance, detect_terminal_theme, parse_hex_color};

#[test]
fn test_terminal_theme_detection_and_invariants() {
    let theme = detect_terminal_theme();
    assert!(!theme.name.is_empty(), "Theme name must be populated");
    assert_eq!(
        theme.luminance,
        calculate_luminance(theme.bg.0, theme.bg.1, theme.bg.2),
        "Theme luminance must match Rec. 601 calculation"
    );
    assert_eq!(
        theme.is_light,
        theme.luminance >= 128,
        "is_light must accurately reflect luminance threshold"
    );
}

#[test]
fn test_adaptive_color_contrasts_against_light_terminals() {
    // Simulate a light terminal background (e.g. Windows Terminal One Half Light or Solarized Light)
    let light_bg = (250, 250, 250); // #fafafa, lum = 250
    let bg_lum = calculate_luminance(light_bg.0, light_bg.1, light_bg.2) as f32;

    // Default Holstein cow coat is normally pure white (#ffffff)
    let white_palette = [(255, 255, 255), (26, 26, 26), (44, 44, 44), (255, 182, 193), (255, 255, 255)];

    let adapted_body = natural_creature_color_adaptive(&white_palette, 10, 5, '(', light_bg);
    let adapted_lum = calculate_luminance(adapted_body.0, adapted_body.1, adapted_body.2) as f32;

    // Contrast delta must be at least 70 against light background
    let delta = bg_lum - adapted_lum;
    assert!(
        delta >= 70.0,
        "White cow coat on light terminal must darken for contrast! Got adapted lum {adapted_lum}, delta {delta}"
    );

    // Eyes on light background must also have strong contrast
    let adapted_eye = natural_creature_color_adaptive(&white_palette, 10, 2, 'o', light_bg);
    let eye_lum = calculate_luminance(adapted_eye.0, adapted_eye.1, adapted_eye.2) as f32;
    assert!(
        (bg_lum - eye_lum) >= 60.0,
        "Cow eye on light terminal must contrast against background! Got eye lum {eye_lum}"
    );
}

#[test]
fn test_adaptive_color_contrasts_against_dark_terminals() {
    // Simulate a dark terminal background (e.g. Catppuccin Mocha #1e1e2e or black #0c0c0c)
    let dark_bg = (30, 30, 46); // lum ~ 32

    // Dark panther palette
    let panther_palette = [(18, 18, 24), (33, 33, 40), (45, 45, 55), (255, 182, 193), (255, 235, 59)];

    let adapted_body = natural_creature_color_adaptive(&panther_palette, 10, 5, '(', dark_bg);
    let body_lum = calculate_luminance(adapted_body.0, adapted_body.1, adapted_body.2);

    // Must meet the luminance floor of at least 75
    assert!(
        body_lum >= 75,
        "Dark panther coat on dark terminal must have luminance floor >= 75, got {body_lum}"
    );
}

#[test]
fn test_hex_color_parser_variations() {
    assert_eq!(parse_hex_color("#1E1E2E"), Some((30, 30, 46)));
    assert_eq!(parse_hex_color("FAFAFA"), Some((250, 250, 250)));
    assert_eq!(parse_hex_color("#fff"), Some((255, 255, 255)));
    assert_eq!(parse_hex_color("#FF1E1E2E"), Some((30, 30, 46)));
    assert_eq!(parse_hex_color("not_a_color"), None);
}
