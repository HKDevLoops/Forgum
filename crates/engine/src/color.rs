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

// ── Lolcat rainbow ─────────────────────────────────────────────────

/// Classic lolcat HSV-based rainbow color.
///
/// `x`, `y` are pixel coordinates; `t` is time; `offset` is hue shift.
pub fn lolcat_color(x: f32, y: f32, t: f32, offset: f32) -> (u8, u8, u8) {
    let angle = 45.0_f32.to_radians();
    let hue = ((x * angle.cos() + y * angle.sin()) / 100.0 + offset / 360.0 + t) % 1.0;
    hsv_to_rgb((hue * 360.0 + 360.0) % 360.0, 0.8, 0.9)
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
        let ok = rgb_to_oklab(128, 64, 32);
        let (r, _g, b) = oklab_to_rgb(ok);
        // Warm color should stay warm after roundtrip
        assert!(
            r >= b,
            "red channel ({r}) should be >= blue ({b}) for warm input"
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
}
