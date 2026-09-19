//! Image to ASCII Art converter and dynamic scene mascot integration.
//!
//! Provides high-fidelity conversion of images (PNG, JPEG, WebP, BMP) to ASCII art,
//! aspect ratio correction for monospace fonts (1:2 font cell ratio),
//! color modes (TrueColor, Ansi256, Grayscale, Monochrome), luminance ramps,
//! and standard `.cow` mascot generation with `$thoughts` and `$eyes` anchors.

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use image::{DynamicImage, GenericImageView};

/// Error type for image to ASCII and cow conversions.
#[derive(Debug, thiserror::Error)]
pub enum ImageAsciiError {
    #[error("failed to open or decode image at '{path}': {source}")]
    ImageDecode {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },
    #[error("failed to decode image: {0}")]
    Image(#[from] image::ImageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("platform error: {0}")]
    Platform(#[from] forgum_platform::PlatformError),
    #[error("invalid dimension: {0}")]
    InvalidDimension(String),
    #[error("invalid mascot name: {0}")]
    InvalidMascotName(String),
    #[error("{0}")]
    Custom(String),
}

/// Color rendering mode for ASCII conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// 24-bit RGB ANSI escape sequences (\x1b[38;2;r;g;bm).
    #[default]
    TrueColor,
    /// 256-color ANSI escape sequences (\x1b[38;5;Nm).
    Ansi256,
    /// Grayscale 24-level ANSI escape sequences.
    Grayscale,
    /// Plain ASCII text with zero ANSI escape codes.
    Monochrome,
}

impl ColorMode {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrueColor => "truecolor",
            Self::Ansi256 => "ansi256",
            Self::Grayscale => "grayscale",
            Self::Monochrome => "monochrome",
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ColorMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "truecolor" | "true" | "24bit" | "rgb" => Ok(Self::TrueColor),
            "ansi256" | "256" | "ansi" => Ok(Self::Ansi256),
            "grayscale" | "greyscale" | "gray" | "grey" => Ok(Self::Grayscale),
            "monochrome" | "mono" | "plain" | "none" | "ascii" => Ok(Self::Monochrome),
            other => Err(format!(
                "unknown color mode '{other}'. Expected truecolor, ansi256, grayscale, or monochrome"
            )),
        }
    }
}

/// Luminance character ramps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LuminanceRamp {
    /// Standard 10-level ramp (" .:-=+*#%@").
    #[default]
    Standard,
    /// Detailed 69-level ramp with rich gradient expression.
    Detailed,
    /// Unicode block element ramp (" ░▒▓█").
    Blocks,
}

impl LuminanceRamp {
    #[must_use]
    pub const fn characters(&self) -> &'static str {
        match self {
            Self::Standard => " .:-=+*#%@",
            Self::Detailed => {
                " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"
            }
            Self::Blocks => " ░▒▓█",
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Detailed => "detailed",
            Self::Blocks => "blocks",
        }
    }
}

impl fmt::Display for LuminanceRamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LuminanceRamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "standard" | "std" | "default" => Ok(Self::Standard),
            "detailed" | "detail" | "fine" => Ok(Self::Detailed),
            "blocks" | "block" => Ok(Self::Blocks),
            other => Err(format!(
                "unknown luminance ramp '{other}'. Expected standard, detailed, or blocks"
            )),
        }
    }
}

/// Configuration options for ASCII art generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiConverterConfig {
    /// Target width in character columns.
    pub target_width: Option<usize>,
    /// Target height in character rows.
    pub target_height: Option<usize>,
    /// Color output mode.
    pub color_mode: ColorMode,
    /// Character ramp for luminance mapping.
    pub ramp: LuminanceRamp,
    /// Invert luminance mapping (bright pixels become dense or sparse).
    pub invert: bool,
}

impl Default for AsciiConverterConfig {
    fn default() -> Self {
        Self {
            target_width: Some(40),
            target_height: None,
            color_mode: ColorMode::TrueColor,
            ramp: LuminanceRamp::Standard,
            invert: false,
        }
    }
}

/// Calculate target width and height taking terminal font cell aspect ratio into account.
///
/// Monospace font cells in terminals typically have an aspect ratio of approximately 1:2
/// (characters are twice as tall as they are wide). Applying a vertical scaling factor of 0.5
/// preserves geometric circle and square proportions in the rendered ASCII art.
#[must_use]
pub fn calculate_dimensions(
    orig_w: u32,
    orig_h: u32,
    target_width: Option<usize>,
    target_height: Option<usize>,
) -> (u32, u32) {
    let orig_w = orig_w.max(1);
    let orig_h = orig_h.max(1);

    match (target_width, target_height) {
        (Some(w), Some(h)) => (w.max(1) as u32, h.max(1) as u32),
        (Some(w), None) => {
            let w = w.max(1) as u32;
            let h = ((orig_h as f32 * w as f32 / orig_w as f32) * 0.5)
                .round()
                .max(1.0) as u32;
            (w, h)
        }
        (None, Some(h)) => {
            let h = h.max(1) as u32;
            let w = ((orig_w as f32 * h as f32 / orig_h as f32) * 2.0)
                .round()
                .max(1.0) as u32;
            (w, h)
        }
        (None, None) => {
            let w = 40u32.min(orig_w).max(10);
            let h = ((orig_h as f32 * w as f32 / orig_w as f32) * 0.5)
                .round()
                .max(1.0) as u32;
            (w, h)
        }
    }
}

/// Convert an RGB color to ANSI 256-color index.
#[must_use]
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        if r < 8 {
            16
        } else if r > 248 {
            231
        } else {
            232 + (((r as u16 - 8) * 24) / 240) as u8
        }
    } else {
        let r_idx = (r as u16 * 5 / 255) as u8;
        let g_idx = (g as u16 * 5 / 255) as u8;
        let b_idx = (b as u16 * 5 / 255) as u8;
        16 + 36 * r_idx + 6 * g_idx + b_idx
    }
}

/// Convert an in-memory `DynamicImage` to ASCII art string.
#[must_use]
pub fn convert_image_to_ascii(img: &DynamicImage, config: &AsciiConverterConfig) -> String {
    let (orig_w, orig_h) = img.dimensions();
    let (target_w, target_h) =
        calculate_dimensions(orig_w, orig_h, config.target_width, config.target_height);

    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3);
    let rgba_img = resized.to_rgba8();
    let ramp_chars: Vec<char> = config.ramp.characters().chars().collect();
    let ramp_len = ramp_chars.len();

    let mut result = String::with_capacity((target_w * target_h * 16) as usize);

    for y in 0..target_h {
        let mut line_str = String::with_capacity((target_w * 16) as usize);

        for x in 0..target_w {
            let pixel = rgba_img.get_pixel(x, y);
            let [r, g, b, a] = pixel.0;

            // Treat transparent pixels as spaces with zero escape codes
            if a < 64 {
                line_str.push(' ');
                continue;
            }

            // ITU-R BT.601 standard luminance
            let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            let mut norm = (luma / 255.0).clamp(0.0, 1.0);
            if config.invert {
                norm = 1.0 - norm;
            }

            let idx = ((ramp_len - 1) as f32 * norm).round() as usize;
            let ch = ramp_chars[idx.min(ramp_len - 1)];

            if ch == ' ' {
                line_str.push(' ');
                continue;
            }

            match config.color_mode {
                ColorMode::Monochrome => {
                    line_str.push(ch);
                }
                ColorMode::TrueColor => {
                    line_str.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}\x1b[0m"));
                }
                ColorMode::Ansi256 => {
                    let code = rgb_to_ansi256(r, g, b);
                    line_str.push_str(&format!("\x1b[38;5;{code}m{ch}\x1b[0m"));
                }
                ColorMode::Grayscale => {
                    let gray_code = 232 + (norm * 23.0).round() as u8;
                    line_str.push_str(&format!("\x1b[38;5;{gray_code}m{ch}\x1b[0m"));
                }
            }
        }

        // Trim trailing space characters if monochrome, but preserve formatting
        let trimmed = if config.color_mode == ColorMode::Monochrome {
            line_str.trim_end()
        } else {
            &line_str
        };

        result.push_str(trimmed);
        if y + 1 < target_h {
            result.push('\n');
        }
    }

    result
}

/// Convert an image file on disk to an ASCII art string.
pub fn convert_image_path_to_ascii(
    path: impl AsRef<Path>,
    config: &AsciiConverterConfig,
) -> Result<String, ImageAsciiError> {
    let p = path.as_ref();
    let img = image::open(p).map_err(|e| ImageAsciiError::ImageDecode {
        path: p.to_path_buf(),
        source: e,
    })?;
    Ok(convert_image_to_ascii(&img, config))
}

/// Convert an in-memory `DynamicImage` into a standard `.cow` format mascot string.
///
/// Automatically inserts `$thoughts` speech-pointer anchors and `$eyes` facial anchors
/// near the top-left head region so speech bubbles and eye animations integrate naturally.
pub fn image_to_cow_from_image(
    img: &DynamicImage,
    _cow_name: &str,
    width: Option<usize>,
) -> Result<String, ImageAsciiError> {
    let config = AsciiConverterConfig {
        target_width: width.or(Some(40)),
        target_height: None,
        color_mode: ColorMode::Monochrome,
        ramp: LuminanceRamp::Standard,
        invert: false,
    };

    let ascii_art = convert_image_to_ascii(img, &config);
    let mut lines: Vec<String> = ascii_art.lines().map(|l| l.to_string()).collect();

    // Determine the indent of the mascot head (first line with characters)
    let head_indent = lines
        .iter()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
        .unwrap_or(6);

    // Place $eyes in the upper head area (within the first 4 non-empty lines)
    let mut placed_eyes = false;
    let search_limit = lines.len().min(4);
    for line in lines.iter_mut().take(search_limit) {
        let chars: Vec<char> = line.chars().collect();
        if let Some(first_non_ws) = chars.iter().position(|c| !c.is_whitespace()) {
            if chars.len() >= first_non_ws + 3 {
                let eye_pos = first_non_ws + 1;
                let mut new_line = String::new();
                for (i, &ch) in chars.iter().enumerate() {
                    if i == eye_pos {
                        new_line.push_str("$eyes");
                    } else if i == eye_pos + 1 {
                        // Omit one character because $eyes expands to 2 characters ("oo")
                    } else {
                        new_line.push(ch);
                    }
                }
                *line = new_line;
                placed_eyes = true;
                break;
            }
        }
    }

    // Fallback if the image lines were unusually sparse
    if !placed_eyes && !lines.is_empty() {
        if let Some(first_line) = lines.first_mut() {
            first_line.push_str(" $eyes");
        }
    }

    // Generate natural speech bubble pointer lines with $thoughts
    let p1_indent = head_indent.saturating_sub(2).max(2);
    let p2_indent = head_indent.saturating_sub(1).max(3);
    let pointer1 = format!("{}$thoughts", " ".repeat(p1_indent));
    let pointer2 = format!("{}$thoughts", " ".repeat(p2_indent));

    let body = lines.join("\n");
    let cow_content = format!(
        "$the_cow = <<\"EOC\";\n{}\n{}\n{}\nEOC\n",
        pointer1, pointer2, body
    );

    Ok(cow_content)
}

/// Convert an image file on disk into a standard `.cow` format mascot string.
pub fn image_to_cow(
    img_path: impl AsRef<Path>,
    cow_name: &str,
    width: Option<usize>,
) -> Result<String, ImageAsciiError> {
    let p = img_path.as_ref();
    let img = image::open(p).map_err(|e| ImageAsciiError::ImageDecode {
        path: p.to_path_buf(),
        source: e,
    })?;
    image_to_cow_from_image(&img, cow_name, width)
}

/// Get the custom cows directory path (`~/.config/forgum/cows/`).
pub fn custom_cows_dir() -> Result<PathBuf, ImageAsciiError> {
    let base = forgum_platform::config_dir()?;
    Ok(base.join("cows"))
}

/// Save a `.cow` mascot string into the user's custom cows directory.
pub fn save_custom_cow(cow_name: &str, cow_content: &str) -> Result<PathBuf, ImageAsciiError> {
    let dir = custom_cows_dir()?;
    std::fs::create_dir_all(&dir)?;
    let clean_name = cow_name.trim().trim_end_matches(".cow");
    let safe_name = Path::new(clean_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("custom");
    if safe_name.is_empty() || safe_name.contains("..") {
        return Err(ImageAsciiError::Custom("Invalid mascot name".to_string()));
    }
    let target = dir.join(format!("{safe_name}.cow"));
    std::fs::write(&target, cow_content)?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x < width / 2 && y < height / 2 {
                // Top-left white square
                *pixel = Rgba([255, 255, 255, 255]);
            } else if x >= width / 2 && y >= height / 2 {
                // Bottom-right red square
                *pixel = Rgba([200, 30, 30, 255]);
            } else {
                // Transparent
                *pixel = Rgba([0, 0, 0, 0]);
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_aspect_ratio_correction() {
        let (w, h) = calculate_dimensions(100, 100, Some(40), None);
        assert_eq!(w, 40);
        // Vertical scaling 0.5 factor preserves geometry on ~1:2 font cells
        assert_eq!(h, 20);

        let (w2, h2) = calculate_dimensions(100, 100, None, Some(20));
        assert_eq!(w2, 40);
        assert_eq!(h2, 20);
    }

    #[test]
    fn test_monochrome_ascii_conversion() {
        let img = create_test_image(40, 40);
        let config = AsciiConverterConfig {
            target_width: Some(20),
            target_height: Some(10),
            color_mode: ColorMode::Monochrome,
            ramp: LuminanceRamp::Standard,
            invert: false,
        };
        let ascii = convert_image_to_ascii(&img, &config);
        assert!(!ascii.is_empty());
        // Monochrome must contain no ANSI escape sequences
        assert!(!ascii.contains("\x1b["));
        // Should contain brightest char '@' or dense ramp chars for the white region
        assert!(ascii.contains('@') || ascii.contains('#') || ascii.contains('%'));
    }

    #[test]
    fn test_truecolor_ansi_emission() {
        let img = create_test_image(40, 40);
        let config = AsciiConverterConfig {
            target_width: Some(20),
            target_height: Some(10),
            color_mode: ColorMode::TrueColor,
            ramp: LuminanceRamp::Standard,
            invert: false,
        };
        let ascii = convert_image_to_ascii(&img, &config);
        // TrueColor must contain 24-bit ANSI sequences
        assert!(ascii.contains("\x1b[38;2;"));
        assert!(ascii.contains("\x1b[0m"));
    }

    #[test]
    fn test_inversion_toggle() {
        let img = create_test_image(40, 40);
        let config_normal = AsciiConverterConfig {
            target_width: Some(20),
            target_height: Some(10),
            color_mode: ColorMode::Monochrome,
            ramp: LuminanceRamp::Standard,
            invert: false,
        };
        let config_invert = AsciiConverterConfig {
            invert: true,
            ..config_normal.clone()
        };
        let ascii_norm = convert_image_to_ascii(&img, &config_normal);
        let ascii_inv = convert_image_to_ascii(&img, &config_invert);
        assert_ne!(ascii_norm, ascii_inv);
    }

    #[test]
    fn test_image_to_cow_formatting_with_placeholders() {
        let img = create_test_image(60, 60);
        let cow_str = image_to_cow_from_image(&img, "mascot", Some(30)).unwrap();
        assert!(cow_str.contains("$the_cow = <<\"EOC\";"));
        assert!(cow_str.contains("$thoughts"));
        assert!(cow_str.contains("$eyes"));
        assert!(cow_str.contains("EOC"));

        // Verify expand_cow cleanly expands placeholders
        let expanded = crate::cow::expand_cow(&cow_str, "oo", "U", "\\");
        assert!(!expanded.contains("$eyes"));
        assert!(!expanded.contains("$thoughts"));
        assert!(expanded.contains("oo"));
        assert!(expanded.contains('\\'));
    }
}
