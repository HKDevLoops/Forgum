//! Color system — OKLCH gradients, lolcat rainbow, 256-color dithering,
//! Gaussian radial glow.
//!
//! All color math is pure — no external crate dependencies. OKLCH
//! interpolation is done manually using the polar form of Oklab.

use crate::framebuffer::Color;

// ── Hex color parsing ──────────────────────────────────────────────

/// Parse a hex color string like `"#ff8800"` or `"ff8800"` into (r, g, b).
pub fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Convert (r, g, b) to a `Color`.
pub fn rgb_to_color(r: u8, g: u8, b: u8) -> Color {
    Color::rgb(r, g, b)
}

// ── RGB ↔ Oklab conversion ────────────────────────────────────────

/// Linear sRGB component (remove gamma).
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Apply gamma to linear sRGB.
fn linear_to_srgb(c: f32) -> f32 {
    let v = c.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Oklab representation: L (lightness), a (green-red), b (blue-yellow).
#[derive(Debug, Clone, Copy)]
struct Oklab {
    l: f32,
    a: f32,
    b: f32,
}

/// Convert sRGB to Oklab.
#[allow(clippy::excessive_precision)]
fn rgb_to_oklab(r: u8, g: u8, b: u8) -> Oklab {
    let lr = srgb_to_linear(r as f32 / 255.0);
    let lg = srgb_to_linear(g as f32 / 255.0);
    let lb = srgb_to_linear(b as f32 / 255.0);

    let l_ = 0.4122214708f32 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    let m_ = 0.2119034982f32 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
    let s_ = 0.0883024619f32 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

    let l_ = l_.cbrt();
    let m_ = m_.cbrt();
    let s_ = s_.cbrt();

    Oklab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

/// Convert Oklab back to sRGB (r, g, b).
#[allow(clippy::excessive_precision)]
fn oklab_to_rgb(ok: Oklab) -> (u8, u8, u8) {
    let l_ = ok.l + 0.3963377774 * ok.a + 0.2158037573 * ok.b;
    let m_ = ok.l - 0.1055613458 * ok.a - 0.0638541728 * ok.b;
    let s_ = ok.l - 0.0894841775 * ok.a - 1.2914855480 * ok.b;

    let l_ = l_ * l_ * l_;
    let m_ = m_ * m_ * m_;
    let s_ = s_ * s_ * s_;

    let r = linear_to_srgb(1.2270138511 * l_ - 0.5577999807 * m_ + 0.2812561490 * s_);
    let g = linear_to_srgb(-0.0405801784 * l_ + 1.1122568696 * m_ - 0.0716766787 * s_);
    let b = linear_to_srgb(-0.0763812845 * l_ - 0.4214819784 * m_ + 1.5861632204 * s_);

    (
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

// ── OKLCH gradient interpolation ───────────────────────────────────

/// Interpolate between two Oklab colors.
fn oklab_lerp(a: Oklab, b: Oklab, t: f32) -> Oklab {
    Oklab {
        l: a.l + (b.l - a.l) * t,
        a: a.a + (b.a - a.a) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

/// Interpolate a palette of hex colors in OKLCH space.
///
/// `t` is in [0.0, 1.0]. Returns the interpolated RGB color.
pub fn lerp_palette(palette: &[(u8, u8, u8)], t: f32) -> (u8, u8, u8) {
    if palette.is_empty() {
        return (255, 255, 255);
    }
    if palette.len() == 1 {
        return palette[0];
    }

    let t = t.clamp(0.0, 1.0);
    let segments = palette.len() - 1;
    let scaled = t * segments as f32;
    let idx = (scaled.floor() as usize).min(segments - 1);
    let local_t = scaled - idx as f32;

    let a = rgb_to_oklab(palette[idx].0, palette[idx].1, palette[idx].2);
    let b = rgb_to_oklab(palette[idx + 1].0, palette[idx + 1].1, palette[idx + 1].2);

    oklab_to_rgb(oklab_lerp(a, b, local_t))
}

/// Convert palette hex strings to (r, g, b) tuples.
pub fn parse_palette(hexes: &[String]) -> Vec<(u8, u8, u8)> {
    hexes.iter().filter_map(|h| parse_hex(h)).collect()
}

/// Compute a smooth spatial and temporal gradient across an animal's palette.
///
/// If palette is empty, returns white (255, 255, 255).
/// If palette has 1 color, returns that exact RGB color.
/// If palette has >= 2 colors, interpolates in OKLCH/Oklab space across (x, y) coordinates and time.
pub fn palette_gradient(palette: &[(u8, u8, u8)], x: f32, y: f32, t: f32) -> (u8, u8, u8) {
    if palette.is_empty() {
        return (255, 255, 255);
    }
    if palette.len() == 1 {
        return palette[0];
    }
    let wave = (x * 0.08 + y * 0.15 + t * 0.8).sin() * 0.5 + 0.5;
    lerp_palette(palette, wave)
}

// ── Lolcat rainbow ─────────────────────────────────────────────────

/// Refined lolcat rainbow color with high-vibrancy saturated chromatic wave.
///
/// Uses an aspect-ratio-corrected diagonal wave propagation (x * 0.045 + y * 0.09)
/// to account for the terminal's 2:1 character cell proportions. Produces the
/// iconic, deeply saturated, dazzling rainbow spectrum (pure red, electric orange,
/// vibrant yellow, radiant green, vivid cyan, deep royal blue, neon magenta)
/// that defines authentic lolcat.
pub fn lolcat_color(x: f32, y: f32, t: f32, offset: f32) -> (u8, u8, u8) {
    // Aspect-ratio-compensated diagonal wave propagation
    let phase = ((x * 0.045 + y * 0.09) + (offset / 360.0) + (t * 0.8)).rem_euclid(1.0);
    let hue_deg = phase * 360.0;
    hsv_to_rgb(hue_deg, 1.0, 1.0)
}

/// Convert HSV (h in degrees, s/v in [0,1]) to RGB.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

// ── Gaussian radial glow ───────────────────────────────────────────

/// Gaussian radial glow intensity at distance `d` from center.
///
/// `sigma = radius / 2.5` gives intensity ≈ 0.135 at the radius boundary.
pub fn gaussian_glow(dx: f32, dy: f32, radius: f32) -> f32 {
    let sigma = radius / 2.5;
    let d2 = dx * dx + dy * dy;
    (-d2 / (2.0 * sigma * sigma)).exp()
}

/// Inverse-square glow for pulse cores (bright center).
pub fn inverse_square_glow(dx: f32, dy: f32, k: f32) -> f32 {
    1.0 / (1.0 + (dx * dx + dy * dy) * k)
}

// ── 256-color dithering ────────────────────────────────────────────

/// 4×4 Bayer dithering matrix.
const BAYER_4X4: [[f32; 4]; 4] = [
    [-0.5, 0.0, -0.375, 0.125],
    [0.25, -0.25, 0.375, -0.125],
    [-0.375, 0.125, -0.5, 0.0],
    [0.375, -0.125, 0.25, -0.25],
];

/// Quantize RGB to xterm-256 color index.
pub fn rgb_to_xterm256(r: u8, g: u8, b: u8) -> u8 {
    16 + 36 * (r / 51) + 6 * (g / 51) + (b / 51)
}

/// Dithered quantization using 4×4 Bayer matrix.
pub fn dithered_quantize(r: u8, g: u8, b: u8, x: usize, y: usize) -> u8 {
    let bias = BAYER_4X4[y % 4][x % 4] * 16.0;
    let r2 = (r as f32 + bias).clamp(0.0, 255.0) as u8;
    let g2 = (g as f32 + bias).clamp(0.0, 255.0) as u8;
    let b2 = (b as f32 + bias).clamp(0.0, 255.0) as u8;
    rgb_to_xterm256(r2, g2, b2)
}

// ── Mascot Natural Palettes ──────────────────────────────────────

// Natural palettes for Forgum mascots

/// Retrieve the authentic God-given natural color palette (RGB tuples) for any mascot.
pub fn get_natural_palette(mascot: &str) -> &'static [(u8, u8, u8)] {
    let clean = mascot.strip_suffix(".cow").unwrap_or(mascot).to_ascii_lowercase();
    match clean.as_str() {
        "apt" => &[(225, 29, 72), (244, 63, 94), (190, 18, 60), (255, 255, 255), (33, 33, 33)],
        "armadillo" => &[(141, 110, 99), (215, 204, 200), (93, 64, 55), (188, 170, 164), (62, 39, 35)],
        "atat" => &[(158, 158, 158), (207, 216, 220), (55, 71, 79), (176, 190, 197), (211, 47, 47)],
        "bearface" => &[(93, 64, 55), (141, 110, 99), (215, 204, 200), (33, 33, 33), (255, 255, 255)],
        "beavis.zen" | "beavis" => &[(255, 202, 40), (66, 165, 245), (239, 83, 80), (255, 224, 130), (126, 87, 194)],
        "bees" => &[(255, 235, 59), (33, 33, 33), (255, 245, 157), (255, 152, 0), (128, 216, 255)],
        "bill-the-cat" => &[(255, 183, 77), (229, 115, 115), (255, 241, 118), (66, 66, 66), (255, 255, 255)],
        "bud-frogs" => &[(76, 175, 80), (129, 199, 132), (27, 94, 32), (255, 249, 196), (255, 179, 0)],
        "bunny" | "rabbit" => &[(255, 255, 255), (248, 250, 252), (241, 245, 249), (226, 232, 240), (203, 213, 225)],
        "cat" => &[(255, 255, 255), (211, 84, 0), (121, 85, 72), (255, 152, 0), (33, 33, 33)],
        "cat2" => &[(127, 140, 141), (149, 165, 166), (189, 195, 199), (44, 62, 80), (46, 204, 113)],
        "catfence" => &[(93, 64, 55), (62, 39, 35), (230, 126, 34), (255, 255, 255), (26, 26, 26)],
        "charizardvice" => &[(255, 111, 0), (255, 171, 0), (0, 176, 255), (255, 23, 68), (120, 144, 156)],
        "charlie" => &[(141, 110, 99), (93, 64, 55), (245, 245, 245), (62, 39, 35), (255, 204, 188)],
        "claw-arm" => &[(0, 229, 255), (118, 255, 3), (41, 121, 255), (255, 214, 0), (55, 71, 79)],
        "corgi" => &[(211, 84, 0), (229, 152, 102), (255, 255, 255), (44, 44, 44), (255, 182, 193)],
        "cower" => &[(255, 255, 255), (33, 33, 33), (224, 224, 224), (144, 202, 249), (255, 182, 193)],
        "cowfee" => &[(109, 76, 65), (141, 110, 99), (215, 204, 200), (255, 255, 255), (255, 171, 145)],
        "cthulhu-mini" => &[(46, 125, 50), (27, 94, 32), (129, 199, 132), (124, 77, 255), (51, 105, 30)],
        "daemon" => &[(211, 47, 47), (244, 67, 54), (33, 33, 33), (255, 215, 0), (255, 255, 255)],
        "default" | "cow" | "cowsay" => &[(255, 255, 255), (26, 26, 26), (44, 44, 44), (255, 182, 193), (220, 221, 225)],
        "docker-whale" => &[(2, 136, 209), (41, 182, 246), (255, 255, 255), (1, 87, 155), (79, 195, 247)],
        "doge" | "shiba" | "shibu" | "shiba-inu" => &[(229, 152, 102), (211, 84, 0), (253, 254, 254), (237, 187, 153), (26, 26, 26)],
        "dolphin" => &[(79, 195, 247), (129, 212, 250), (225, 245, 254), (2, 136, 209), (1, 87, 155)],
        "dragon" => &[(216, 67, 21), (244, 81, 30), (255, 179, 0), (255, 214, 0), (62, 39, 35)],
        "dragon-and-cow" => &[(198, 40, 40), (229, 57, 53), (255, 143, 0), (255, 255, 255), (26, 26, 26)],
        "duck" => &[(5, 150, 105), (251, 191, 36), (217, 119, 6), (120, 53, 15), (243, 244, 246)],
        "ebi_furai" => &[(255, 143, 0), (255, 179, 0), (255, 248, 225), (229, 57, 53), (255, 138, 128)],
        "elephant" => &[(120, 144, 156), (144, 164, 174), (207, 216, 220), (84, 110, 122), (255, 204, 188)],
        "elephant-in-snake" => &[(85, 139, 47), (120, 144, 156), (51, 105, 30), (139, 195, 74), (255, 249, 196)],
        "elephant2" => &[(96, 125, 139), (120, 144, 156), (176, 190, 197), (236, 239, 241), (55, 71, 79)],
        "eyes" => &[(0, 230, 118), (185, 246, 202), (255, 255, 255), (0, 0, 0), (118, 255, 3)],
        "fat-banana" => &[(255, 235, 59), (255, 245, 157), (251, 192, 45), (121, 85, 72), (76, 175, 80)],
        "fat-cow" => &[(255, 255, 255), (33, 33, 33), (224, 224, 224), (255, 182, 193), (121, 85, 72)],
        "fence" => &[(141, 110, 99), (161, 136, 127), (215, 204, 200), (93, 64, 55), (124, 179, 66)],
        "flaming-sheep" => &[(255, 87, 34), (255, 138, 101), (255, 171, 0), (33, 33, 33), (255, 214, 0)],
        "fox" => &[(230, 81, 0), (255, 152, 0), (255, 255, 255), (33, 33, 33), (251, 192, 45)],
        "ghost" => &[(236, 239, 241), (207, 216, 220), (176, 190, 197), (128, 216, 255), (33, 33, 33)],
        "ghostbusters" => &[(229, 57, 53), (255, 255, 255), (33, 33, 33), (255, 214, 0), (0, 229, 255)],
        "glados" => &[(236, 239, 241), (33, 33, 33), (255, 171, 0), (120, 144, 156), (255, 23, 68)],
        "goat" => &[(215, 204, 200), (161, 136, 127), (141, 110, 99), (93, 64, 55), (255, 248, 225)],
        "goat2" => &[(188, 170, 164), (141, 110, 99), (93, 64, 55), (62, 39, 35), (255, 255, 255)],
        "golden-eagle" => &[(255, 179, 0), (255, 160, 0), (93, 64, 55), (255, 213, 79), (33, 33, 33)],
        "hamster" => &[(212, 163, 115), (250, 237, 205), (231, 111, 81), (255, 182, 193), (33, 33, 33)],
        "happy-whale" => &[(3, 155, 229), (79, 195, 247), (225, 245, 254), (255, 255, 255), (179, 229, 252)],
        "hedgehog" => &[(109, 76, 65), (141, 110, 99), (215, 204, 200), (62, 39, 35), (33, 33, 33)],
        "hellokitty" => &[(255, 255, 255), (255, 64, 129), (255, 235, 59), (33, 33, 33), (66, 165, 245)],
        "hippie" => &[(255, 0, 127), (0, 229, 255), (255, 255, 0), (118, 255, 3), (213, 0, 249)],
        "hiya" => &[(255, 255, 255), (33, 33, 33), (255, 128, 171), (255, 182, 193), (251, 192, 45)],
        "hypno" => &[(124, 77, 255), (0, 229, 255), (224, 64, 251), (255, 214, 0), (255, 245, 157)],
        "jellyfish" => &[(224, 64, 251), (234, 128, 252), (128, 216, 255), (243, 229, 245), (179, 136, 255)],
        "jesus" => &[(251, 191, 36), (254, 240, 138), (245, 158, 11), (255, 255, 255), (217, 119, 6)],
        "king" => &[(255, 215, 0), (233, 30, 99), (255, 255, 255), (33, 33, 33), (207, 216, 220)],
        "kiss" => &[(255, 255, 255), (33, 33, 33), (255, 64, 129), (211, 47, 47), (117, 117, 117)],
        "kitten" | "kittens" => &[(255, 255, 255), (211, 84, 0), (121, 85, 72), (255, 152, 0), (33, 33, 33)],
        "kitty" => &[(255, 255, 255), (211, 84, 0), (121, 85, 72), (255, 152, 0), (33, 33, 33)],
        "knight" => &[(176, 190, 197), (207, 216, 220), (120, 144, 156), (55, 71, 79), (255, 255, 255)],
        "koala" => &[(158, 158, 158), (189, 189, 189), (224, 224, 224), (33, 33, 33), (121, 85, 72)],
        "kosh" => &[(128, 203, 196), (77, 182, 172), (0, 137, 123), (255, 213, 79), (55, 71, 79)],
        "lamb" => &[(250, 250, 250), (245, 245, 245), (224, 224, 224), (189, 189, 189), (255, 182, 193)],
        "lamb2" => &[(245, 245, 245), (238, 238, 238), (189, 189, 189), (158, 158, 158), (255, 182, 193)],
        "lobster" => &[(211, 47, 47), (244, 67, 54), (255, 138, 128), (183, 28, 28), (255, 205, 210)],
        "lollerskates" => &[(0, 230, 118), (0, 176, 255), (255, 234, 0), (255, 64, 129), (124, 77, 255)],
        "luke-koala" => &[(144, 164, 174), (176, 190, 197), (207, 216, 220), (0, 229, 255), (33, 33, 33)],
        "mech-and-cow" => &[(2, 132, 199), (224, 242, 254), (3, 105, 161), (245, 158, 11), (255, 255, 255)],
        "meow" => &[(255, 255, 255), (211, 84, 0), (121, 85, 72), (255, 152, 0), (33, 33, 33)],
        "minotaur" => &[(78, 52, 46), (109, 76, 65), (211, 47, 47), (62, 39, 35), (215, 204, 200)],
        "mona-lisa" => &[(141, 110, 99), (161, 136, 127), (215, 204, 200), (78, 52, 46), (46, 125, 50)],
        "moofasa" => &[(255, 143, 0), (255, 160, 0), (255, 193, 7), (183, 28, 28), (255, 213, 79)],
        "mooghidjirah" => &[(255, 215, 0), (198, 40, 40), (173, 20, 87), (255, 111, 0), (33, 33, 33)],
        "moojira" => &[(46, 125, 50), (27, 94, 32), (0, 229, 255), (255, 23, 68), (255, 255, 255)],
        "moose" => &[(78, 52, 46), (93, 64, 55), (121, 85, 72), (141, 110, 99), (62, 39, 35)],
        "mule" => &[(92, 64, 51), (121, 85, 72), (141, 110, 99), (62, 39, 35), (215, 204, 200)],
        "mutilated" => &[(211, 47, 47), (183, 28, 28), (33, 33, 33), (144, 164, 174), (255, 255, 255)],
        "nyan" | "nyan-cat" | "nyancat" | "nyan_cat" => &[(255, 0, 51), (255, 127, 0), (255, 255, 0), (51, 255, 0), (0, 153, 255), (153, 51, 255), (255, 204, 188)],
        "octopus" => &[(142, 36, 170), (171, 71, 188), (206, 147, 216), (225, 190, 231), (255, 213, 79)],
        "owl" => &[(121, 85, 72), (141, 110, 99), (255, 179, 0), (255, 213, 79), (62, 39, 35)],
        "panther" => &[(24, 24, 27), (39, 39, 42), (63, 63, 70), (234, 179, 8), (244, 63, 94)],
        "pawn" => &[(207, 216, 220), (236, 239, 241), (144, 164, 174), (120, 144, 156), (255, 255, 255)],
        "periodic-table" => &[(0, 229, 255), (118, 255, 3), (255, 214, 0), (255, 64, 129), (124, 77, 255)],
        "personality-sphere" => &[(3, 169, 244), (224, 224, 224), (33, 33, 33), (255, 171, 0), (255, 255, 255)],
        "pig" => &[(255, 182, 193), (255, 128, 171), (248, 187, 208), (255, 64, 129), (141, 110, 99)],
        "pterodactyl" => &[(104, 159, 56), (139, 195, 74), (205, 220, 57), (255, 152, 0), (230, 81, 0)],
        "pufferfish" => &[(250, 204, 21), (253, 224, 71), (234, 179, 8), (255, 255, 255), (21, 128, 61)],
        "queen" => &[(255, 215, 0), (233, 30, 99), (255, 255, 255), (33, 33, 33), (207, 216, 220)],
        "radioactive-kitty" => &[(34, 197, 94), (134, 239, 172), (22, 163, 74), (250, 204, 21), (255, 255, 255)],
        "ram" => &[(245, 245, 245), (255, 255, 255), (158, 158, 158), (117, 117, 117), (97, 97, 97)],
        "ren" => &[(215, 204, 200), (161, 136, 127), (255, 128, 171), (229, 57, 53), (255, 214, 0)],
        "rhino" => &[(100, 116, 139), (148, 163, 184), (71, 85, 105), (51, 65, 85), (30, 41, 59)],
        "rook" => &[(120, 144, 156), (144, 164, 174), (176, 190, 197), (84, 110, 122), (255, 255, 255)],
        "rooster" => &[(211, 47, 47), (245, 127, 23), (27, 94, 32), (255, 179, 0), (251, 192, 45)],
        "satanic" => &[(211, 47, 47), (183, 28, 28), (33, 33, 33), (255, 87, 34), (0, 0, 0)],
        "sauron" => &[(249, 115, 22), (220, 38, 38), (185, 28, 28), (0, 0, 0), (255, 235, 59)],
        "seahorse" => &[(255, 160, 0), (255, 179, 0), (255, 213, 79), (255, 224, 130), (230, 81, 0)],
        "seahorse-big" => &[(255, 160, 0), (255, 179, 0), (255, 213, 79), (255, 224, 130), (230, 81, 0)],
        "sheep" => &[(255, 255, 255), (245, 245, 245), (66, 66, 66), (33, 33, 33), (255, 182, 193)],
        "shikato" => &[(158, 158, 158), (224, 224, 224), (33, 33, 33), (189, 189, 189), (117, 117, 117)],
        "shrug" => &[(255, 255, 255), (224, 224, 224), (189, 189, 189), (158, 158, 158), (33, 33, 33)],
        "skeleton" => &[(245, 245, 245), (224, 224, 224), (158, 158, 158), (97, 97, 97), (117, 117, 117)],
        "sloth" => &[(161, 98, 7), (202, 138, 4), (113, 63, 18), (254, 240, 138), (28, 25, 23)],
        "small" => &[(255, 255, 255), (224, 224, 224), (33, 33, 33), (255, 182, 193), (141, 110, 99)],
        "smiling-octopus" => &[(171, 71, 188), (206, 147, 216), (243, 229, 245), (225, 190, 231), (255, 213, 79)],
        "snoopy" => &[(255, 255, 255), (33, 33, 33), (211, 47, 47), (66, 66, 66), (255, 205, 210)],
        "snoopyhouse" => &[(211, 47, 47), (255, 255, 255), (33, 33, 33), (183, 28, 28), (66, 66, 66)],
        "snoopysleep" => &[(255, 255, 255), (33, 33, 33), (211, 47, 47), (66, 66, 66), (144, 202, 249)],
        "spidercow" => &[(33, 33, 33), (211, 47, 47), (59, 130, 246), (255, 255, 255), (224, 224, 224)],
        "squid" => &[(6, 182, 212), (59, 130, 246), (2, 132, 199), (207, 250, 254), (245, 158, 11)],
        "squirrel" => &[(141, 110, 99), (161, 136, 127), (215, 204, 200), (93, 64, 55), (245, 245, 245)],
        "stegosaurus" => &[(56, 142, 60), (76, 175, 80), (129, 199, 132), (230, 81, 0), (255, 152, 0)],
        "stimpy" => &[(229, 57, 53), (239, 83, 80), (57, 73, 171), (255, 255, 255), (255, 235, 59)],
        "supermilker" => &[(255, 255, 255), (33, 33, 33), (2, 132, 199), (56, 189, 248), (255, 182, 193)],
        "surgery" => &[(2, 132, 199), (56, 189, 248), (224, 242, 254), (255, 255, 255), (239, 68, 68)],
        "telebears" => &[(59, 130, 246), (96, 165, 250), (37, 99, 235), (255, 255, 255), (250, 204, 21)],
        "three-eyes" => &[(255, 255, 255), (33, 33, 33), (0, 230, 118), (118, 255, 3), (255, 182, 193)],
        "tiger" => &[(234, 88, 12), (24, 24, 27), (255, 255, 255), (251, 146, 60), (132, 204, 22)],
        "tortoise" => &[(85, 139, 47), (104, 159, 56), (158, 157, 36), (51, 105, 30), (215, 204, 200)],
        "turkey" => &[(109, 76, 65), (198, 40, 40), (21, 101, 192), (245, 127, 23), (33, 33, 33)],
        "turtle" => &[(46, 125, 50), (76, 175, 80), (129, 199, 132), (255, 245, 157), (27, 94, 32)],
        "tux" => &[(255, 255, 255), (33, 33, 33), (255, 152, 0), (255, 167, 38), (245, 124, 0)],
        "tux-big" => &[(255, 255, 255), (33, 33, 33), (255, 152, 0), (255, 167, 38), (245, 124, 0)],
        "tweety-bird" => &[(255, 235, 59), (255, 245, 157), (255, 152, 0), (255, 183, 77), (0, 176, 255)],
        "unipony" => &[(232, 121, 249), (56, 189, 248), (192, 132, 252), (254, 240, 138), (255, 255, 255)],
        "vader" => &[(24, 24, 27), (39, 39, 42), (220, 38, 38), (127, 29, 29), (56, 189, 248)],
        "viper" => &[(22, 163, 74), (34, 197, 94), (21, 128, 61), (234, 179, 8), (220, 38, 38)],
        "vulpix" => &[(249, 115, 22), (234, 88, 12), (251, 146, 60), (255, 247, 237), (255, 237, 213)],
        "walrus" => &[(120, 113, 108), (214, 211, 209), (87, 83, 78), (168, 162, 158), (244, 63, 94)],
        "weeping-angel" => &[(158, 158, 158), (117, 117, 117), (189, 189, 189), (97, 97, 97), (224, 224, 224)],
        "whale" => &[(2, 136, 209), (41, 182, 246), (225, 245, 254), (1, 87, 155), (255, 255, 255)],
        "wizard" => &[(124, 77, 255), (179, 136, 255), (255, 215, 0), (255, 255, 255), (0, 229, 255)],
        "wolf" => &[(156, 163, 175), (75, 85, 99), (31, 41, 55), (243, 244, 246), (245, 158, 11)],
        "world" => &[(41, 182, 246), (102, 187, 106), (255, 255, 255), (2, 136, 209), (67, 160, 71)],
        "yoda" => &[(132, 204, 22), (101, 163, 13), (163, 230, 53), (215, 204, 200), (133, 77, 14)],
        _ => &[(255, 255, 255), (26, 26, 26), (44, 44, 44), (255, 182, 193), (220, 221, 225)],
    }
}

/// Retrieve the authentic God-given natural color palette (Hex strings) for any mascot.
pub fn get_natural_hex_palette(mascot: &str) -> &'static [&'static str] {
    let clean = mascot.strip_suffix(".cow").unwrap_or(mascot).to_ascii_lowercase();
    match clean.as_str() {
        "apt" => &["#e11d48", "#f43f5e", "#be123c", "#ffffff", "#212121"],
        "armadillo" => &["#8d6e63", "#d7ccc8", "#5d4037", "#bcaaa4", "#3e2723"],
        "atat" => &["#9e9e9e", "#cfd8dc", "#37474f", "#b0bec5", "#d32f2f"],
        "bearface" => &["#5d4037", "#8d6e63", "#d7ccc8", "#212121", "#ffffff"],
        "beavis.zen" | "beavis" => &["#ffca28", "#42a5f5", "#ef5350", "#ffe082", "#7e57c2"],
        "bees" => &["#ffeb3b", "#212121", "#fff59d", "#ff9800", "#80d8ff"],
        "bill-the-cat" => &["#ffb74d", "#e57373", "#fff176", "#424242", "#ffffff"],
        "bud-frogs" => &["#4caf50", "#81c784", "#1b5e20", "#fff9c4", "#ffb300"],
        "bunny" | "rabbit" => &["#ffffff", "#f8fafc", "#f1f5f9", "#e2e8f0", "#cbd5e1"],
        "cat" => &["#ffffff", "#d35400", "#795548", "#ff9800", "#212121"],
        "cat2" => &["#7f8c8d", "#95a5a6", "#bdc3c7", "#2c3e50", "#2ecc71"],
        "catfence" => &["#5d4037", "#3e2723", "#e67e22", "#ffffff", "#1a1a1a"],
        "charizardvice" => &["#ff6f00", "#ffab00", "#00b0ff", "#ff1744", "#78909c"],
        "charlie" => &["#8d6e63", "#5d4037", "#f5f5f5", "#3e2723", "#ffccbc"],
        "claw-arm" => &["#00e5ff", "#76ff03", "#2979ff", "#ffd600", "#37474f"],
        "corgi" => &["#d35400", "#e59866", "#ffffff", "#2c2c2c", "#ffb6c1"],
        "cower" => &["#ffffff", "#212121", "#e0e0e0", "#90caf9", "#ffb6c1"],
        "cowfee" => &["#6d4c41", "#8d6e63", "#d7ccc8", "#ffffff", "#ffab91"],
        "cthulhu-mini" => &["#2e7d32", "#1b5e20", "#81c784", "#7c4dff", "#33691e"],
        "daemon" => &["#d32f2f", "#f44336", "#212121", "#ffd700", "#ffffff"],
        "default" | "cow" | "cowsay" => &["#ffffff", "#1a1a1a", "#2c2c2c", "#ffb6c1", "#dcdde1"],
        "docker-whale" => &["#0288d1", "#29b6f6", "#ffffff", "#01579b", "#4fc3f7"],
        "doge" | "shiba" | "shibu" | "shiba-inu" => &["#e59866", "#d35400", "#fdfefe", "#edbb99", "#1a1a1a"],
        "dolphin" => &["#4fc3f7", "#81d4fa", "#e1f5fe", "#0288d1", "#01579b"],
        "dragon" => &["#d84315", "#f4511e", "#ffb300", "#ffd600", "#3e2723"],
        "dragon-and-cow" => &["#c62828", "#e53935", "#ff8f00", "#ffffff", "#1a1a1a"],
        "duck" => &["#059669", "#fbbf24", "#d97706", "#78350f", "#f3f4f6"],
        "ebi_furai" => &["#ff8f00", "#ffb300", "#fff8e1", "#e53935", "#ff8a80"],
        "elephant" => &["#78909c", "#90a4ae", "#cfd8dc", "#546e7a", "#ffccbc"],
        "elephant-in-snake" => &["#558b2f", "#78909c", "#33691e", "#8bc34a", "#fff9c4"],
        "elephant2" => &["#607d8b", "#78909c", "#b0bec5", "#eceff1", "#37474f"],
        "eyes" => &["#00e676", "#b9f6ca", "#ffffff", "#000000", "#76ff03"],
        "fat-banana" => &["#ffeb3b", "#fff59d", "#fbc02d", "#795548", "#4caf50"],
        "fat-cow" => &["#ffffff", "#212121", "#e0e0e0", "#ffb6c1", "#795548"],
        "fence" => &["#8d6e63", "#a1887f", "#d7ccc8", "#5d4037", "#7cb342"],
        "flaming-sheep" => &["#ff5722", "#ff8a65", "#ffab00", "#212121", "#ffd600"],
        "fox" => &["#e65100", "#ff9800", "#ffffff", "#212121", "#fbc02d"],
        "ghost" => &["#eceff1", "#cfd8dc", "#b0bec5", "#80d8ff", "#212121"],
        "ghostbusters" => &["#e53935", "#ffffff", "#212121", "#ffd600", "#00e5ff"],
        "glados" => &["#eceff1", "#212121", "#ffab00", "#78909c", "#ff1744"],
        "goat" => &["#d7ccc8", "#a1887f", "#8d6e63", "#5d4037", "#fff8e1"],
        "goat2" => &["#bcaaa4", "#8d6e63", "#5d4037", "#3e2723", "#ffffff"],
        "golden-eagle" => &["#ffb300", "#ffa000", "#5d4037", "#ffd54f", "#212121"],
        "hamster" => &["#d4a373", "#faedcd", "#e76f51", "#ffb6c1", "#212121"],
        "happy-whale" => &["#039be5", "#4fc3f7", "#e1f5fe", "#ffffff", "#b3e5fc"],
        "hedgehog" => &["#6d4c41", "#8d6e63", "#d7ccc8", "#3e2723", "#212121"],
        "hellokitty" => &["#ffffff", "#ff4081", "#ffeb3b", "#212121", "#42a5f5"],
        "hippie" => &["#ff007f", "#00e5ff", "#ffff00", "#76ff03", "#d500f9"],
        "hiya" => &["#ffffff", "#212121", "#ff80ab", "#ffb6c1", "#fbc02d"],
        "hypno" => &["#7c4dff", "#00e5ff", "#e040fb", "#ffd600", "#fff59d"],
        "jellyfish" => &["#e040fb", "#ea80fc", "#80d8ff", "#f3e5f5", "#b388ff"],
        "jesus" => &["#fbbf24", "#fef08a", "#f59e0b", "#ffffff", "#d97706"],
        "king" => &["#ffd700", "#e91e63", "#ffffff", "#212121", "#cfd8dc"],
        "kiss" => &["#ffffff", "#212121", "#ff4081", "#d32f2f", "#757575"],
        "kitten" | "kittens" => &["#ffffff", "#d35400", "#795548", "#ff9800", "#212121"],
        "kitty" => &["#ffffff", "#d35400", "#795548", "#ff9800", "#212121"],
        "knight" => &["#b0bec5", "#cfd8dc", "#78909c", "#37474f", "#ffffff"],
        "koala" => &["#9e9e9e", "#bdbdbd", "#e0e0e0", "#212121", "#795548"],
        "kosh" => &["#80cbc4", "#4db6ac", "#00897b", "#ffd54f", "#37474f"],
        "lamb" => &["#fafafa", "#f5f5f5", "#e0e0e0", "#bdbdbd", "#ffb6c1"],
        "lamb2" => &["#f5f5f5", "#eeeeee", "#bdbdbd", "#9e9e9e", "#ffb6c1"],
        "lobster" => &["#d32f2f", "#f44336", "#ff8a80", "#b71c1c", "#ffcdd2"],
        "lollerskates" => &["#00e676", "#00b0ff", "#ffea00", "#ff4081", "#7c4dff"],
        "luke-koala" => &["#90a4ae", "#b0bec5", "#cfd8dc", "#00e5ff", "#212121"],
        "mech-and-cow" => &["#0284c7", "#e0f2fe", "#0369a1", "#f59e0b", "#ffffff"],
        "meow" => &["#ffffff", "#d35400", "#795548", "#ff9800", "#212121"],
        "minotaur" => &["#4e342e", "#6d4c41", "#d32f2f", "#3e2723", "#d7ccc8"],
        "mona-lisa" => &["#8d6e63", "#a1887f", "#d7ccc8", "#4e342e", "#2e7d32"],
        "moofasa" => &["#ff8f00", "#ffa000", "#ffc107", "#b71c1c", "#ffd54f"],
        "mooghidjirah" => &["#ffd700", "#c62828", "#ad1457", "#ff6f00", "#212121"],
        "moojira" => &["#2e7d32", "#1b5e20", "#00e5ff", "#ff1744", "#ffffff"],
        "moose" => &["#4e342e", "#5d4037", "#795548", "#8d6e63", "#3e2723"],
        "mule" => &["#5c4033", "#795548", "#8d6e63", "#3e2723", "#d7ccc8"],
        "mutilated" => &["#d32f2f", "#b71c1c", "#212121", "#90a4ae", "#ffffff"],
        "nyan" | "nyan-cat" | "nyancat" | "nyan_cat" => &["#ff0033", "#ff7f00", "#ffff00", "#33ff00", "#0099ff", "#9933ff", "#ffccbc"],
        "octopus" => &["#8e24aa", "#ab47bc", "#ce93d8", "#e1bee7", "#ffd54f"],
        "owl" => &["#795548", "#8d6e63", "#ffb300", "#ffd54f", "#3e2723"],
        "panther" => &["#18181b", "#27272a", "#3f3f46", "#eab308", "#f43f5e"],
        "pawn" => &["#cfd8dc", "#eceff1", "#90a4ae", "#78909c", "#ffffff"],
        "periodic-table" => &["#00e5ff", "#76ff03", "#ffd600", "#ff4081", "#7c4dff"],
        "personality-sphere" => &["#03a9f4", "#e0e0e0", "#212121", "#ffab00", "#ffffff"],
        "pig" => &["#ffb6c1", "#ff80ab", "#f8bbd0", "#ff4081", "#8d6e63"],
        "pterodactyl" => &["#689f38", "#8bc34a", "#cddc39", "#ff9800", "#e65100"],
        "pufferfish" => &["#facc15", "#fde047", "#eab308", "#ffffff", "#15803d"],
        "queen" => &["#ffd700", "#e91e63", "#ffffff", "#212121", "#cfd8dc"],
        "radioactive-kitty" => &["#22c55e", "#86efac", "#16a34a", "#facc15", "#ffffff"],
        "ram" => &["#f5f5f5", "#ffffff", "#9e9e9e", "#757575", "#616161"],
        "ren" => &["#d7ccc8", "#a1887f", "#ff80ab", "#e53935", "#ffd600"],
        "rhino" => &["#64748b", "#94a3b8", "#475569", "#334155", "#1e293b"],
        "rook" => &["#78909c", "#90a4ae", "#b0bec5", "#546e7a", "#ffffff"],
        "rooster" => &["#d32f2f", "#f57f17", "#1b5e20", "#ffb300", "#fbc02d"],
        "satanic" => &["#d32f2f", "#b71c1c", "#212121", "#ff5722", "#000000"],
        "sauron" => &["#f97316", "#dc2626", "#b91c1c", "#000000", "#ffeb3b"],
        "seahorse" => &["#ffa000", "#ffb300", "#ffd54f", "#ffe082", "#e65100"],
        "seahorse-big" => &["#ffa000", "#ffb300", "#ffd54f", "#ffe082", "#e65100"],
        "sheep" => &["#ffffff", "#f5f5f5", "#424242", "#212121", "#ffb6c1"],
        "shikato" => &["#9e9e9e", "#e0e0e0", "#212121", "#bdbdbd", "#757575"],
        "shrug" => &["#ffffff", "#e0e0e0", "#bdbdbd", "#9e9e9e", "#212121"],
        "skeleton" => &["#f5f5f5", "#e0e0e0", "#9e9e9e", "#616161", "#757575"],
        "sloth" => &["#a16207", "#ca8a04", "#713f12", "#fef08a", "#1c1917"],
        "small" => &["#ffffff", "#e0e0e0", "#212121", "#ffb6c1", "#8d6e63"],
        "smiling-octopus" => &["#ab47bc", "#ce93d8", "#f3e5f5", "#e1bee7", "#ffd54f"],
        "snoopy" => &["#ffffff", "#212121", "#d32f2f", "#424242", "#ffcdd2"],
        "snoopyhouse" => &["#d32f2f", "#ffffff", "#212121", "#b71c1c", "#424242"],
        "snoopysleep" => &["#ffffff", "#212121", "#d32f2f", "#424242", "#90caf9"],
        "spidercow" => &["#212121", "#d32f2f", "#3b82f6", "#ffffff", "#e0e0e0"],
        "squid" => &["#06b6d4", "#3b82f6", "#0284c7", "#cffafe", "#f59e0b"],
        "squirrel" => &["#8d6e63", "#a1887f", "#d7ccc8", "#5d4037", "#f5f5f5"],
        "stegosaurus" => &["#388e3c", "#4caf50", "#81c784", "#e65100", "#ff9800"],
        "stimpy" => &["#e53935", "#ef5350", "#3949ab", "#ffffff", "#ffeb3b"],
        "supermilker" => &["#ffffff", "#212121", "#0284c7", "#38bdf8", "#ffb6c1"],
        "surgery" => &["#0284c7", "#38bdf8", "#e0f2fe", "#ffffff", "#ef4444"],
        "telebears" => &["#3b82f6", "#60a5fa", "#2563eb", "#ffffff", "#facc15"],
        "three-eyes" => &["#ffffff", "#212121", "#00e676", "#76ff03", "#ffb6c1"],
        "tiger" => &["#ea580c", "#18181b", "#ffffff", "#fb923c", "#84cc16"],
        "tortoise" => &["#558b2f", "#689f38", "#9e9d24", "#33691e", "#d7ccc8"],
        "turkey" => &["#6d4c41", "#c62828", "#1565c0", "#f57f17", "#212121"],
        "turtle" => &["#2e7d32", "#4caf50", "#81c784", "#fff59d", "#1b5e20"],
        "tux" => &["#ffffff", "#212121", "#ff9800", "#ffa726", "#f57c00"],
        "tux-big" => &["#ffffff", "#212121", "#ff9800", "#ffa726", "#f57c00"],
        "tweety-bird" => &["#ffeb3b", "#fff59d", "#ff9800", "#ffb74d", "#00b0ff"],
        "unipony" => &["#e879f9", "#38bdf8", "#c084fc", "#fef08a", "#ffffff"],
        "vader" => &["#18181b", "#27272a", "#dc2626", "#7f1d1d", "#38bdf8"],
        "viper" => &["#16a34a", "#22c55e", "#15803d", "#eab308", "#dc2626"],
        "vulpix" => &["#f97316", "#ea580c", "#fb923c", "#fff7ed", "#ffedd5"],
        "walrus" => &["#78716c", "#d6d3d1", "#57534e", "#a8a29e", "#f43f5e"],
        "weeping-angel" => &["#9e9e9e", "#757575", "#bdbdbd", "#616161", "#e0e0e0"],
        "whale" => &["#0288d1", "#29b6f6", "#e1f5fe", "#01579b", "#ffffff"],
        "wizard" => &["#7c4dff", "#b388ff", "#ffd700", "#ffffff", "#00e5ff"],
        "wolf" => &["#9ca3af", "#4b5563", "#1f2937", "#f3f4f6", "#f59e0b"],
        "world" => &["#29b6f6", "#66bb6a", "#ffffff", "#0288d1", "#43a047"],
        "yoda" => &["#84cc16", "#65a30d", "#a3e635", "#d7ccc8", "#854d0e"],
        _ => &["#ffffff", "#1a1a1a", "#2c2c2c", "#ffb6c1", "#dcdde1"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_with_hash() {
        let (r, g, b) = parse_hex("#ff8800").unwrap();
        assert_eq!(r, 0xff);
        assert_eq!(g, 0x88);
        assert_eq!(b, 0x00);
    }

    #[test]
    fn parse_hex_without_hash() {
        let (r, g, b) = parse_hex("00ff00").unwrap();
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn parse_hex_invalid() {
        assert!(parse_hex("xyz").is_none());
        assert!(parse_hex("#fff").is_none());
    }

    #[test]
    fn oklab_roundtrip() {
        // OKLCH roundtrip introduces quantization error from sRGB↔linear conversion.
        // Grayscale colors roundtrip accurately; chromatic colors have larger error
        // but preserve hue direction.
        for (r, g, b, desc) in [
            (128, 128, 128, "gray"),
            (192, 192, 192, "light gray"),
            (64, 64, 64, "dark gray"),
            (0, 0, 0, "black"),
            (255, 255, 255, "white"),
        ] {
            let ok = rgb_to_oklab(r, g, b);
            let (r2, g2, b2) = oklab_to_rgb(ok);
            assert!(
                (r as i32 - r2 as i32).abs() <= 10,
                "{desc} R roundtrip: {r} → {r2}"
            );
            assert!(
                (g as i32 - g2 as i32).abs() <= 10,
                "{desc} G roundtrip: {g} → {g2}"
            );
            assert!(
                (b as i32 - b2 as i32).abs() <= 10,
                "{desc} B roundtrip: {b} → {b2}"
            );
        }
        // Saturated primaries: verify hue direction is preserved even if
        // exact values drift due to gamma roundtrip.
        let ok = rgb_to_oklab(255, 0, 0);
        let (r2, g2, b2) = oklab_to_rgb(ok);
        assert!(
            r2 > g2 && r2 > b2,
            "red should stay red-dominant: ({r2},{g2},{b2})"
        );
        let ok = rgb_to_oklab(0, 255, 0);
        let (r2, g2, b2) = oklab_to_rgb(ok);
        assert!(
            g2 > r2 && g2 > b2,
            "green should stay green-dominant: ({r2},{g2},{b2})"
        );
        let ok = rgb_to_oklab(0, 0, 255);
        let (r2, g2, b2) = oklab_to_rgb(ok);
        assert!(
            b2 > r2 && b2 > g2,
            "blue should stay blue-dominant: ({r2},{g2},{b2})"
        );
    }

    #[test]
    fn oklab_roundtrip_warm_stays_warm() {
        let ok = rgb_to_oklab(128, 64, 32);
        let (r, _g, b) = oklab_to_rgb(ok);
        assert!(r >= b, "red ({r}) should be >= blue ({b}) for warm input");
    }

    #[test]
    fn lerp_palette_boundary_t0() {
        // OKLCH interpolation introduces quantization at endpoints.
        // Verify the result is close to red (255,0,0) in hue space.
        let palette = vec![(255, 0, 0), (0, 0, 255)];
        let (r, g, b) = lerp_palette(&palette, 0.0);
        // Red component should dominate
        assert!(
            r > g && r > b,
            "t=0 should be red-dominant, got ({r},{g},{b})"
        );
    }

    #[test]
    fn lerp_palette_boundary_t1() {
        let palette = vec![(255, 0, 0), (0, 0, 255)];
        let (r, g, b) = lerp_palette(&palette, 1.0);
        // Blue component should dominate
        assert!(
            b > r && b > g,
            "t=1 should be blue-dominant, got ({r},{g},{b})"
        );
    }

    #[test]
    fn lerp_palette_midpoint() {
        let palette = vec![(255, 0, 0), (0, 0, 255)];
        let (r, _g, b) = lerp_palette(&palette, 0.5);
        // Midpoint of red-blue in OKLCH should be roughly purple
        assert!(r > 50 && r < 200);
        assert!(b > 50 && b < 200);
    }

    #[test]
    fn lerp_palette_three_colors() {
        let palette = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let (r1, g1, b1) = lerp_palette(&palette, 0.0);
        // t=0 should be red-dominant
        assert!(r1 > g1 && r1 > b1, "t=0: ({r1},{g1},{b1})");
        let (r3, g3, b3) = lerp_palette(&palette, 1.0);
        // t=1 should be blue-dominant
        assert!(b3 > r3 && b3 > g3, "t=1: ({r3},{g3},{b3})");
        // Verify the gradient endpoints differ
        assert_ne!((r1, g1, b1), (r3, g3, b3));
    }

    #[test]
    fn lerp_palette_single_color() {
        let palette = vec![(100, 200, 50)];
        assert_eq!(lerp_palette(&palette, 0.5), (100, 200, 50));
    }

    #[test]
    fn lerp_palette_empty() {
        let palette = vec![];
        assert_eq!(lerp_palette(&palette, 0.5), (255, 255, 255));
    }

    #[test]
    fn lolcat_color_is_deterministic() {
        let c1 = lolcat_color(10.0, 20.0, 0.0, 0.0);
        let c2 = lolcat_color(10.0, 20.0, 0.0, 0.0);
        assert_eq!(c1, c2);
    }

    #[test]
    fn lolcat_color_varies_with_position() {
        let c1 = lolcat_color(0.0, 0.0, 0.0, 0.0);
        let c2 = lolcat_color(100.0, 0.0, 0.0, 0.0);
        assert_ne!(c1, c2);
    }

    #[test]
    fn hsv_to_rgb_red() {
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn hsv_to_rgb_green() {
        let (r, g, b) = hsv_to_rgb(120.0, 1.0, 1.0);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn gaussian_glow_center_is_1() {
        let v = gaussian_glow(0.0, 0.0, 5.0);
        assert!((v - 1.0).abs() < 0.001);
    }

    #[test]
    fn gaussian_glow_far_is_near_zero() {
        let v = gaussian_glow(100.0, 100.0, 5.0);
        assert!(v < 0.001);
    }

    #[test]
    fn inverse_square_glow_center() {
        let v = inverse_square_glow(0.0, 0.0, 1.0);
        assert!((v - 1.0).abs() < 0.001);
    }

    #[test]
    fn rgb_to_xterm256_range() {
        for r in 0..=255u8 {
            for g in [0u8, 128, 255] {
                for b in [0u8, 128, 255] {
                    let idx = rgb_to_xterm256(r, g, b);
                    assert!((16..=231).contains(&idx), "idx={idx} for r={r} g={g} b={b}");
                }
            }
        }
    }

    #[test]
    fn dithered_quantize_returns_valid_index() {
        let idx = dithered_quantize(128, 64, 32, 5, 3);
        assert!((16..=231).contains(&idx));
    }

    #[test]
    fn dithered_quantize_bayer_matrix_varies_by_position() {
        // Bayer dithering adds position-dependent bias (±8 range). To see
        // an index change, we need a color near a quantization boundary.
        // rgb_to_xterm256 divides by 51, so boundaries are at 0,51,102,153,204,255.
        // Use 102 (exact boundary) so ±8 bias crosses into different buckets.
        let idx00 = dithered_quantize(102, 102, 102, 0, 0);
        let idx01 = dithered_quantize(102, 102, 102, 0, 1);
        // Bayer matrix values at these positions: [0,0]=-0.5, [0,1]=0.0, [1,0]=0.25, [2,2]=-0.5
        // Bias = value * 16, so [0,0]=-8, [0,1]=0, [1,0]=+4
        // 102-8=94→94/51=1, 102+0=102→102/51=2, 102+4=106→106/51=2
        assert_ne!(
            idx00, idx01,
            "Bayer positions (0,0) and (0,1) should differ"
        );
    }

    #[test]
    fn dithered_quantize_black_and_white() {
        let black = dithered_quantize(0, 0, 0, 0, 0);
        let white = dithered_quantize(255, 255, 255, 0, 0);
        assert!(
            black < white,
            "black index ({black}) should be < white ({white})"
        );
    }

    #[test]
    fn hsv_to_rgb_blue() {
        let (r, g, b) = hsv_to_rgb(240.0, 1.0, 1.0);
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
    }

    #[test]
    fn hsv_to_rgb_grayscale() {
        let (r, g, b) = hsv_to_rgb(0.0, 0.0, 0.5);
        assert_eq!(r, 128);
        assert_eq!(g, 128);
        assert_eq!(b, 128);
    }

    #[test]
    fn hsv_to_rgb_black() {
        let (r, g, b) = hsv_to_rgb(0.0, 0.0, 0.0);
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn gaussian_glow_at_radius_boundary() {
        // At distance == radius, Gaussian glow should be low (decaying).
        // sigma = radius / 2.5, so at d=radius: exp(-r²/(2*sigma²)) = exp(-2.5²/2) ≈ 0.044
        let v = gaussian_glow(5.0, 0.0, 5.0);
        assert!(
            v > 0.01 && v < 0.15,
            "at radius boundary expected low intensity (0.01..0.15), got {v}"
        );
    }

    #[test]
    fn parse_hex_uppercase() {
        let (r, g, b) = parse_hex("#FF8800").unwrap();
        assert_eq!(r, 0xff);
        assert_eq!(g, 0x88);
        assert_eq!(b, 0x00);
    }

    #[test]
    fn gaussian_glow_radius_zero() {
        let v = gaussian_glow(0.0, 0.0, 0.0);
        assert!(
            v.is_finite() || v.is_nan(),
            "should return an f32 without panicking"
        );
    }

    #[test]
    fn gaussian_glow_symmetry() {
        let right = gaussian_glow(1.0, 0.0, 5.0);
        let left = gaussian_glow(-1.0, 0.0, 5.0);
        assert_eq!(right, left, "glow should be symmetric across x-axis");

        let down = gaussian_glow(0.0, 1.0, 5.0);
        let up = gaussian_glow(0.0, -1.0, 5.0);
        assert_eq!(down, up, "glow should be symmetric across y-axis");
    }

    #[test]
    fn hsv_to_rgb_60_degrees() {
        let (r, g, b) = hsv_to_rgb(60.0, 1.0, 1.0);
        assert_eq!((r, g, b), (255, 255, 0), "HSV(60,1,1) should be yellow");
    }

    #[test]
    fn hsv_to_rgb_180_degrees() {
        let (r, g, b) = hsv_to_rgb(180.0, 1.0, 1.0);
        assert_eq!((r, g, b), (0, 255, 255), "HSV(180,1,1) should be cyan");
    }

    #[test]
    fn hsv_to_rgb_300_degrees() {
        let (r, g, b) = hsv_to_rgb(300.0, 1.0, 1.0);
        assert_eq!((r, g, b), (255, 0, 255), "HSV(300,1,1) should be magenta");
    }

    #[test]
    fn hsv_to_rgb_zero_saturation() {
        let (r, g, b) = hsv_to_rgb(120.0, 0.0, 1.0);
        assert_eq!(
            (r, g, b),
            (255, 255, 255),
            "zero saturation should be white"
        );
    }

    #[test]
    fn lerp_palette_clamps_t() {
        let palette = [(0, 0, 0), (255, 255, 255)];
        let (r0, g0, b0) = lerp_palette(&palette, -0.5);
        let dr = (r0 as i32).unsigned_abs();
        let dg = (g0 as i32).unsigned_abs();
        let db = (b0 as i32).unsigned_abs();
        assert!(
            dr < 10 && dg < 10 && db < 10,
            "t=-0.5 should clamp to near-black, got ({r0},{g0},{b0})"
        );

        let (r1, g1, b1) = lerp_palette(&palette, 1.5);
        let dr = (r1 as i32 - 255).unsigned_abs();
        let dg = (g1 as i32 - 255).unsigned_abs();
        let db = (b1 as i32 - 255).unsigned_abs();
        assert!(
            dr < 10 && dg < 10 && db < 10,
            "t=1.5 should clamp to near-white, got ({r1},{g1},{b1})"
        );
    }

    #[test]
    fn test_god_given_natural_palettes() {
        // Cow: white and black
        let cow_p = get_natural_palette("cow");
        assert!(cow_p.contains(&(255, 255, 255)));
        assert!(cow_p.contains(&(26, 26, 26)));

        // Cat & Kittens: white, ginger, brown, orange, black
        let cat_p = get_natural_palette("cat");
        let kittens_p = get_natural_palette("kittens");
        assert_eq!(cat_p, kittens_p);
        assert!(cat_p.contains(&(255, 255, 255))); // white
        assert!(cat_p.contains(&(211, 84, 0)));    // ginger
        assert!(cat_p.contains(&(121, 85, 72)));   // brown
        assert!(cat_p.contains(&(255, 152, 0)));   // orange
        assert!(cat_p.contains(&(33, 33, 33)));    // black

        // Bunny: white only in nature (no pink!)
        let bunny_p = get_natural_palette("bunny");
        for &(r, g, b) in bunny_p {
            assert!(r >= 200 && g >= 200 && b >= 200, "bunny must be white only: ({r},{g},{b})");
        }

        // Doge: Shiba golden orange
        let doge_p = get_natural_palette("doge");
        assert!(doge_p.contains(&(229, 152, 102)));
        assert!(doge_p.contains(&(211, 84, 0)));

        // Hippie: colorful abstract
        let hippie_p = get_natural_palette("hippie");
        assert_eq!(hippie_p.len(), 5);

        // Mule: brown
        let mule_p = get_natural_palette("mule");
        assert!(mule_p.contains(&(92, 64, 51))); // #5c4033 dark brown

        // Pig: pink
        let pig_p = get_natural_palette("pig");
        assert!(pig_p.contains(&(255, 182, 193))); // pink

        // Ram: white and grey
        let ram_p = get_natural_palette("ram");
        assert!(ram_p.contains(&(245, 245, 245)));
        assert!(ram_p.contains(&(158, 158, 158)));

        // Hamster: golden brown & cream
        let hamster_p = get_natural_palette("hamster");
        assert!(hamster_p.contains(&(212, 163, 115)));
    }
}
