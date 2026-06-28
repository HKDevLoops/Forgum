//! Native cow file loading and speech bubble rendering.
//!
//! Loads standard `.cow` files (cowsay format), expands `$eyes`, `$tongue`,
//! `$thoughts` placeholders, wraps text in a speech bubble, and renders the
//! combined output into the framebuffer.

use std::path::Path;

use crate::framebuffer::{Cell, Color, FrameBuffer};

/// The default cow art when no `.cow` file is found.
const DEFAULT_COW: &str = r#"        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#;

/// Load a `.cow` file and expand placeholders.
///
/// Returns the expanded cow text with `$eyes`, `$tongue`, `$thoughts` replaced.
/// If the file doesn't exist or can't be read, returns the default cow.
pub fn load_cow(
    cow_name: &str,
    data_dir: &Path,
    eyes: &str,
    tongue: &str,
    thoughts: &str,
) -> String {
    let cow_path = data_dir.join("Cows").join(format!("{cow_name}.cow"));
    let raw = match std::fs::read_to_string(&cow_path) {
        Ok(s) => s,
        Err(_) => return default_cow_expanded(eyes, tongue, thoughts),
    };
    expand_cow(&raw, eyes, tongue, thoughts)
}

/// Load a `.cow` file from an explicit path.
pub fn load_cow_from_path(
    path: &Path,
    eyes: &str,
    tongue: &str,
    thoughts: &str,
) -> Result<String, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read cow file {}: {e}", path.display()))?;
    Ok(expand_cow(&raw, eyes, tongue, thoughts))
}

/// Expand `$eyes`, `$tongue`, `$thoughts` placeholders in a `.cow` template.
pub fn expand_cow(cow_template: &str, eyes: &str, tongue: &str, thoughts: &str) -> String {
    let mut result = String::with_capacity(cow_template.len());

    // Extract the $the_cow = <<EOC; ... EOC; block if present.
    let cow_body = if let Some(start) = cow_template.find("<<EOC;") {
        let body_start = start + "<<EOC;".len();
        if let Some(end) = cow_template[body_start..].find("EOC;") {
            &cow_template[body_start..body_start + end]
        } else {
            &cow_template[body_start..]
        }
    } else {
        // No heredoc marker — treat the whole file as the cow body.
        cow_template
    };

    for line in cow_body.lines() {
        let line = line.replace("$eyes", eyes);
        let line = line.replace("$tongue", tongue);
        let line = line.replace("$thoughts", thoughts);
        result.push_str(&line);
        result.push('\n');
    }

    // Remove trailing newline if the original didn't have one.
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

/// The default cow with placeholders expanded.
pub fn default_cow_expanded(eyes: &str, tongue: &str, thoughts: &str) -> String {
    expand_cow(DEFAULT_COW, eyes, tongue, thoughts)
}

/// Wrap text in a speech bubble above the cow art.
///
/// Returns the combined (bubble + cow) text ready for rendering.
pub fn compose_scene(cow_text: &str, bubble_text: &str) -> String {
    if bubble_text.is_empty() {
        return cow_text.to_string();
    }

    let cow_lines: Vec<&str> = cow_text.lines().collect();
    let cow_width = cow_lines.iter().map(|l| l.len()).max().unwrap_or(0).max(2);

    let bubble = wrap_bubble(bubble_text, cow_width);

    let mut result = String::with_capacity(bubble.len() + cow_text.len() + 1);
    result.push_str(&bubble);
    result.push('\n');
    result.push_str(cow_text);
    result
}

/// Wrap text in a speech bubble with rounded corners.
///
/// ```text
///  _______________
/// |               |
/// |  Hello, world |
/// |_______________|
/// ```
fn wrap_bubble(text: &str, min_width: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find the longest line.
    let text_width = lines
        .iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(0)
        .max(min_width.saturating_sub(2));
    let inner_width = text_width + 2; // +2 for padding spaces

    let mut result = String::with_capacity((inner_width + 4) * (lines.len() + 2));

    // Top border: ` _______________ `
    result.push(' ');
    result.push('_');
    for _ in 0..inner_width {
        result.push('_');
    }
    result.push('\n');

    // Content lines: `|  Hello, world |`
    if lines.len() == 1 {
        result.push('|');
        result.push(' ');
        result.push_str(&format!(" {} ", lines[0]));
        pad_to(&mut result, inner_width + 2);
        result.push('|');
        result.push('\n');
    } else {
        for (i, line) in lines.iter().enumerate() {
            result.push('|');
            if i == 0 {
                // First line: opening quote style
                result.push(' ');
                result.push_str(line);
                pad_to(&mut result, inner_width + 1);
                result.push('|');
            } else if i == lines.len() - 1 {
                // Last line
                result.push(' ');
                result.push_str(line);
                pad_to(&mut result, inner_width + 1);
                result.push('|');
            } else {
                result.push(' ');
                result.push_str(line);
                pad_to(&mut result, inner_width + 1);
                result.push('|');
            }
            result.push('\n');
        }
    }

    // Bottom border: `|_______________|`
    result.push('|');
    for _ in 0..=inner_width {
        result.push('_');
    }
    result.push('|');
    result.push('\n');

    result
}

/// Pad `result` with spaces until its current line length reaches `target_len`.
fn pad_to(result: &mut String, target_len: usize) {
    let current_len = result.lines().last().map_or(0, |l| l.len());
    for _ in current_len..target_len {
        result.push(' ');
    }
}

/// Render the composed cow text (bubble + cow art) into a framebuffer.
pub fn render_cow(fb: &mut FrameBuffer, composed: &str) {
    let fg = Color::WHITE;
    let mut x = 0usize;
    let mut y = 0usize;

    for ch in composed.chars() {
        if ch == '\n' {
            x = 0;
            y = y.saturating_add(1);
            continue;
        }
        if y >= fb.height {
            break;
        }
        if x < fb.width {
            let _ = fb.set(x, y, Cell::new(ch, fg));
        }
        x = x.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_replaces_eyes() {
        let template = r#"        $eyes
   ─────$thoughts─────
  / $tongue            \"#;
        let expanded = expand_cow(template, "@@", "U", "\\\\");
        assert!(expanded.contains("@@"));
        assert!(expanded.contains("U"));
        assert!(expanded.contains("\\\\"));
        assert!(!expanded.contains("$eyes"));
        assert!(!expanded.contains("$tongue"));
        assert!(!expanded.contains("$thoughts"));
    }

    #[test]
    fn expand_heredoc_extracts_body() {
        let cow = r#"Some preamble
$the_cow = <<EOC;
        $eyes
   (oo)
EOC;
More stuff after"#;
        let expanded = expand_cow(cow, "xx", "  ", "\\\\");
        assert!(expanded.contains("xx"));
        assert!(expanded.contains("(oo)"));
        assert!(!expanded.contains("Some preamble"));
        assert!(!expanded.contains("More stuff"));
    }

    #[test]
    fn default_cow_has_eyes() {
        let cow = default_cow_expanded("oo", " ", "\\\\");
        assert!(cow.contains("oo"));
        assert!(cow.contains("^__^"));
    }

    #[test]
    fn bubble_rounded_corners() {
        let bubble = wrap_bubble("Hello", 10);
        assert!(bubble.starts_with(" _____"));
        assert!(bubble.contains("|"));
        assert!(bubble.contains("Hello"));
    }

    #[test]
    fn compose_scene_combines() {
        let cow = "  cow_here";
        let scene = compose_scene(cow, "hi");
        assert!(scene.contains("cow_here"));
        assert!(scene.contains("hi"));
    }

    #[test]
    fn compose_scene_no_bubble() {
        let cow = "  cow_only";
        let scene = compose_scene(cow, "");
        assert_eq!(scene, cow);
    }

    #[test]
    fn render_cow_writes_to_fb() {
        let mut fb = FrameBuffer::new(40, 10);
        let composed = "Hello\n  cow";
        render_cow(&mut fb, composed);
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'H');
        assert_eq!(fb.get(0, 1).ch, ' ');
        assert_eq!(fb.get(2, 1).ch, 'c');
    }

    #[test]
    fn load_cow_missing_file_returns_default() {
        let cow = load_cow(
            "nonexistent",
            Path::new("/tmp/no-such-dir"),
            "oo",
            " ",
            "\\\\",
        );
        assert!(cow.contains("^__^"));
    }
}
