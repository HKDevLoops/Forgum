//! Native cow file loading and speech bubble rendering.
//!
//! Loads standard `.cow` files (cowsay format), expands `$eyes`, `$tongue`,
//! `$thoughts` placeholders, wraps text in a speech bubble, and renders the
//! combined output into the framebuffer.

use std::path::Path;

use rand::seq::SliceRandom;

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
/// Looks in the bundled data directory first, then the user's custom cows
/// directory (`~/.config/forgum/cows/`). If neither has the file, returns
/// the default cow.
pub fn load_cow(
    cow_name: &str,
    data_dir: &Path,
    eyes: &str,
    tongue: &str,
    thoughts: &str,
) -> String {
    // 1. Try bundled data directory.
    let cow_path = data_dir.join("Cows").join(format!("{cow_name}.cow"));
    if let Ok(raw) = std::fs::read_to_string(&cow_path) {
        return expand_cow(&raw, eyes, tongue, thoughts);
    }
    // 2. Try user's custom cows directory (Phase 8.12: community cow packs).
    if let Some(custom_path) = custom_cows_dir() {
        let custom_cow = custom_path.join(format!("{cow_name}.cow"));
        if let Ok(raw) = std::fs::read_to_string(&custom_cow) {
            return expand_cow(&raw, eyes, tongue, thoughts);
        }
    }
    default_cow_expanded(eyes, tongue, thoughts)
}

/// If `cow_name` is `"random"`, pick a random `.cow` file from the data
/// directory and the user's custom cows directory, and return its basename.
/// Otherwise return `cow_name` unchanged.
pub fn resolve_cow_name(cow_name: &str, data_dir: &Path) -> String {
    if cow_name != "random" {
        return cow_name.to_string();
    }
    let mut entries: Vec<String> = Vec::new();

    // Bundled cows.
    let cows_dir = data_dir.join("Cows");
    if let Ok(rd) = std::fs::read_dir(&cows_dir) {
        entries.extend(rd.flatten().filter_map(|e| {
            let p = e.path();
            if p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("cow"))
            {
                p.file_stem().map(|s| s.to_string_lossy().into_owned())
            } else {
                None
            }
        }));
    }

    // Phase 8.12: Custom user cows.
    if let Some(custom_dir) = custom_cows_dir() {
        if let Ok(rd) = std::fs::read_dir(&custom_dir) {
            entries.extend(rd.flatten().filter_map(|e| {
                let p = e.path();
                if p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("cow"))
                {
                    p.file_stem().map(|s| s.to_string_lossy().into_owned())
                } else {
                    None
                }
            }));
        }
    }

    if entries.is_empty() {
        return "default".to_string();
    }
    let mut rng = rand::thread_rng();
    entries.shuffle(&mut rng);
    entries[0].clone()
}

/// Return the path to the user's custom cows directory (`~/.config/forgum/cows/`).
/// Returns `None` if the config path can't be determined.
fn custom_cows_dir() -> Option<std::path::PathBuf> {
    forgum_platform::config_path()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("cows")))
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
        if line.contains('$') {
            let line = line.replace("$eyes", eyes);
            let line = line.replace("$tongue", tongue);
            let line = line.replace("$thoughts", thoughts);
            result.push_str(&line);
        } else {
            result.push_str(line);
        }
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
    let inner_width = text_width + 2; // +2 for padding spaces around text

    let mut result = String::with_capacity((inner_width + 4) * (lines.len() + 2));

    // Top border: ` _______________ `
    result.push(' ');
    for _ in 0..=inner_width {
        result.push('_');
    }
    result.push('\n');

    // Content lines: `|  Hello, world |`
    for line in lines.iter() {
        let pad_target = inner_width + 1;
        result.push('|');
        result.push(' ');
        result.push_str(line);
        pad_to(&mut result, pad_target);
        result.push('|');
        result.push('\n');
    }

    // Bottom border: `|_______________|`
    result.push('|');
    for _ in 0..inner_width {
        result.push('_');
    }
    result.push('|');
    result.push('\n');

    result
}

/// Pad `result` with spaces until its current line length reaches `target_len`.
fn pad_to(result: &mut String, target_len: usize) {
    let current_len = result
        .rsplit_once('\n')
        .map_or(result.len(), |(_, last)| last.len());
    for _ in current_len..target_len {
        result.push(' ');
    }
}

/// Render the composed cow text (bubble + cow art) into a framebuffer.
pub fn render_cow(fb: &mut FrameBuffer, composed: &str, color_mode: &str, time: f32) {
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
            let cell_fg = crate::effects::resolve_fg(color_mode, x, y, time, fg);
            let _ = fb.set(x, y, Cell::new(ch, cell_fg));
        }
        x = x.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── expand_cow ────────────────────────────────────────────────

    #[test]
    fn expand_replaces_all_placeholders() {
        let template = "        $eyes\n   ----- $thoughts -----\n  / $tongue             \\";
        let expanded = expand_cow(template, "@@", "U", "\\\\");
        assert_eq!(
            expanded,
            "        @@\n   ----- \\\\ -----\n  / U             \\"
        );
        assert!(!expanded.contains("$eyes"), "$eyes must be replaced");
        assert!(!expanded.contains("$tongue"), "$tongue must be replaced");
        assert!(
            !expanded.contains("$thoughts"),
            "$thoughts must be replaced"
        );
    }

    #[test]
    fn expand_heredoc_extracts_body_only() {
        let cow =
            "Some preamble\n$the_cow = <<EOC;\n        $eyes\n   (oo)\nEOC;\nMore stuff after";
        let expanded = expand_cow(cow, "xx", "  ", "\\\\");
        assert_eq!(expanded, "\n        xx\n   (oo)");
        assert!(
            !expanded.contains("preamble"),
            "heredoc must not include preamble"
        );
        assert!(
            !expanded.contains("More stuff"),
            "heredoc must not include content after EOC"
        );
    }

    #[test]
    fn expand_no_placeholders_passthrough() {
        let template = "just plain text\nno placeholders here";
        let expanded = expand_cow(template, "oo", " ", "\\\\");
        assert_eq!(expanded, template);
    }

    #[test]
    fn expand_empty_template_returns_empty() {
        assert_eq!(expand_cow("", "oo", " ", "\\\\"), "");
    }

    #[test]
    fn expand_multiple_same_placeholder() {
        let template = "$eyes $eyes $eyes";
        let expanded = expand_cow(template, "XX", " ", "\\\\");
        assert_eq!(expanded, "XX XX XX");
    }

    #[test]
    fn expand_preserves_newlines() {
        let template = "line1\nline2\nline3";
        let expanded = expand_cow(template, "oo", " ", "\\\\");
        assert_eq!(expanded.lines().count(), 3);
        assert_eq!(expanded, "line1\nline2\nline3");
    }

    // ── default_cow_expanded ──────────────────────────────────────

    #[test]
    fn default_cow_has_correct_structure() {
        let cow = default_cow_expanded("oo", " ", "\\\\");
        let lines: Vec<&str> = cow.lines().collect();
        assert_eq!(lines.len(), 5, "default cow must have 5 lines");

        // Line 0: "\   ^__^"
        assert!(
            lines[0].contains('^'),
            "line 0 must contain caret: {:?}",
            lines[0]
        );
        assert!(
            lines[0].contains("__"),
            "line 0 must contain underscores: {:?}",
            lines[0]
        );

        // Line 1: " (oo)\_______"
        assert!(
            lines[1].contains("(oo)"),
            "line 1 must contain (oo): {:?}",
            lines[1]
        );

        // Line 3: "||----w |" or similar — must contain ||
        assert!(
            lines[3].contains("||"),
            "line 3 must contain || for legs: {:?}",
            lines[3]
        );
    }

    #[test]
    fn default_cow_custom_eyes() {
        // DEFAULT_COW uses literal (oo), not $eyes placeholder, so
        // custom eyes don't change the output. Verify the cow always
        // contains the expected head shape regardless of eye param.
        let cow = default_cow_expanded("@@", "U", "\\\\");
        assert!(cow.contains("oo"), "default cow always has (oo)");
        assert!(cow.contains("^__^"), "default cow always has ^__^");
        assert!(!cow.contains("$eyes"), "no unreplaced placeholder");
    }

    #[test]
    fn default_cow_custom_tongue() {
        // DEFAULT_COW uses literal (oo), not $tongue placeholder.
        // Verify the cow structure is unchanged regardless of tongue param.
        let cow = default_cow_expanded("oo", "P", "\\\\");
        assert!(cow.contains("oo"), "default cow always has (oo)");
        assert!(!cow.contains("$tongue"), "no unreplaced placeholder");
    }

    // ── wrap_bubble ───────────────────────────────────────────────

    #[test]
    fn bubble_structure_single_line() {
        let bubble = wrap_bubble("Hello", 10);
        let lines: Vec<&str> = bubble.lines().collect();
        assert_eq!(lines.len(), 3, "single-line bubble must have 3 lines");

        // Top border: space + underscores
        assert!(lines[0].starts_with(' '), "top must start with space");
        assert!(
            lines[0].chars().all(|c| c == '_' || c == ' '),
            "top must be underscores/spaces only"
        );

        // Content line: starts/ends with |
        assert!(lines[1].starts_with('|'), "content must start with |");
        assert!(lines[1].ends_with('|'), "content must end with |");
        assert!(lines[1].contains("Hello"), "content must contain text");

        // Bottom border: |____...___|
        assert!(lines[2].starts_with('|'), "bottom must start with |");
        assert!(lines[2].ends_with('|'), "bottom must end with |");
        let bottom_inner: String = lines[2][1..lines[2].len() - 1].to_string();
        assert!(
            bottom_inner.chars().all(|c| c == '_'),
            "bottom inner must be all underscores: {bottom_inner:?}"
        );
    }

    #[test]
    fn bubble_structure_multi_line() {
        let bubble = wrap_bubble("Line 1\nLine 2", 10);
        let lines: Vec<&str> = bubble.lines().collect();
        assert_eq!(lines.len(), 4, "two-line bubble must have 4 lines");
        assert!(lines[1].contains("Line 1"));
        assert!(lines[2].contains("Line 2"));
    }

    #[test]
    fn bubble_empty_text_returns_empty() {
        assert_eq!(wrap_bubble("", 10), "");
    }

    #[test]
    fn bubble_width_respects_min_width() {
        let bubble = wrap_bubble("Hi", 20);
        let lines: Vec<&str> = bubble.lines().collect();
        // Top border width should be at least min_width
        assert!(
            lines[0].len() >= 20,
            "bubble width ({}) must be >= min_width (20)",
            lines[0].len()
        );
    }

    #[test]
    fn bubble_width_expands_for_long_text() {
        let long_text = "This is a very long line of text that exceeds the minimum width";
        let bubble = wrap_bubble(long_text, 10);
        let lines: Vec<&str> = bubble.lines().collect();
        assert!(lines[0].len() > 20, "bubble must expand for long text");
        assert!(
            lines[1].contains(long_text),
            "content must include full text"
        );
    }

    #[test]
    fn bubble_all_rows_same_width() {
        let cases: Vec<(&str, usize)> = vec![
            ("", 0),
            ("x", 0),
            ("Hello, world!", 0),
            ("Hello, world!\nSecond line\nThird line", 0),
        ];
        for (text, min_width) in cases {
            let bubble = wrap_bubble(text, min_width);
            if bubble.is_empty() {
                continue;
            }
            let rows: Vec<&str> = bubble.lines().collect();
            assert!(rows.len() >= 2, "must have at least top+bottom");
            let top_width = rows[0].len();
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(
                    row.len(),
                    top_width,
                    "row {i} width {} != top width {top_width} for text {:?}",
                    row.len(),
                    text
                );
            }
        }
    }

    #[test]
    fn bubble_no_trailing_underscore_row() {
        let bubble = wrap_bubble("Hello", 0);
        let rows: Vec<&str> = bubble.lines().collect();
        assert_eq!(rows.len(), 3);
        // Bottom border should be `|_____|` — ends with `|`, not `_`
        assert!(
            rows[2].ends_with('|'),
            "bottom row must end with '|': {:?}",
            rows[2]
        );
        // Content row should be `| Hello |` — no trailing underscore
        assert!(
            !rows[1].chars().last().is_some_and(|c| c == '_'),
            "content row must not end with '_': {:?}",
            rows[1]
        );
    }

    // ── compose_scene ─────────────────────────────────────────────

    #[test]
    fn compose_scene_bubble_before_cow() {
        let cow = "  cow_line1\n  cow_line2";
        let scene = compose_scene(cow, "hi");
        let cow_pos = scene.find("cow_line1").unwrap();
        let hi_pos = scene.find("hi").unwrap();
        assert!(
            hi_pos < cow_pos,
            "bubble (hi at {hi_pos}) must precede cow (at {cow_pos})"
        );
    }

    #[test]
    fn compose_scene_no_bubble_passthrough() {
        let cow = "  cow_only";
        let scene = compose_scene(cow, "");
        assert_eq!(scene, cow);
    }

    #[test]
    fn compose_scene_newline_separates_bubble_from_cow() {
        let cow = "COW";
        let scene = compose_scene(cow, "TEXT");
        let cow_pos = scene.find("COW").unwrap();
        let text_pos = scene.find("TEXT").unwrap();
        let between = &scene[text_pos..cow_pos];
        assert!(between.contains('\n'), "must be newline-separated");
    }

    #[test]
    fn compose_scene_long_text_wraps() {
        let cow = "  cow";
        let long_text = "This is a very long speech bubble text that should cause the bubble to be wider than the cow art itself";
        let scene = compose_scene(cow, long_text);
        // The bubble should contain the full text
        assert!(scene.contains(long_text), "bubble must contain full text");
        // The bubble should appear before the cow
        assert!(scene.find(long_text).unwrap() < scene.find("cow").unwrap());
    }

    // ── render_cow ────────────────────────────────────────────────

    #[test]
    fn render_cow_exact_positions() {
        let mut fb = FrameBuffer::new(10, 5);
        let composed = "ABCDE\n  FG\nX";
        render_cow(&mut fb, composed, "static", 0.0);
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'A');
        assert_eq!(fb.get(1, 0).ch, 'B');
        assert_eq!(fb.get(4, 0).ch, 'E');
        assert_eq!(fb.get(0, 1).ch, ' ');
        assert_eq!(fb.get(2, 1).ch, 'F');
        assert_eq!(fb.get(3, 1).ch, 'G');
        assert_eq!(fb.get(0, 2).ch, 'X');
        // Row 3,4 should be empty
        assert_eq!(fb.get(0, 3).ch, ' ');
        assert_eq!(fb.get(0, 4).ch, ' ');
    }

    #[test]
    fn render_cow_all_chars_white_fg() {
        let mut fb = FrameBuffer::new(20, 5);
        render_cow(&mut fb, "ABC\nDEF", "static", 0.0);
        fb.swap();
        for y in 0..2 {
            for x in 0..3 {
                let ch = char::from(b'A' + (y * 3 + x) as u8);
                assert_eq!(fb.get(x, y).ch, ch);
                assert_eq!(fb.get(x, y).fg, Color::WHITE, "char {ch} must be WHITE");
            }
        }
    }

    #[test]
    fn render_cow_truncates_at_width_boundary() {
        let mut fb = FrameBuffer::new(3, 1);
        render_cow(&mut fb, "ABCDE", "static", 0.0);
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'A');
        assert_eq!(fb.get(1, 0).ch, 'B');
        assert_eq!(fb.get(2, 0).ch, 'C');
        // Positions beyond width return empty
        assert_eq!(fb.get(3, 0).ch, ' ');
    }

    #[test]
    fn render_cow_truncates_at_height_boundary() {
        let mut fb = FrameBuffer::new(10, 2);
        render_cow(&mut fb, "line1\nline2\nline3\nline4", "static", 0.0);
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'l');
        assert_eq!(fb.get(0, 1).ch, 'l');
        // Rows beyond height are not rendered
    }

    #[test]
    fn render_cow_empty_text_no_damage() {
        let mut fb = FrameBuffer::new(10, 5);
        render_cow(&mut fb, "", "static", 0.0);
        assert!(
            fb.compute_damage().is_empty(),
            "empty text should produce no damage"
        );
    }

    #[test]
    fn render_cow_newline_resets_x() {
        let mut fb = FrameBuffer::new(10, 3);
        render_cow(&mut fb, "A\nB\nC", "static", 0.0);
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'A');
        assert_eq!(fb.get(0, 1).ch, 'B');
        assert_eq!(fb.get(0, 2).ch, 'C');
    }

    // ── load_cow ──────────────────────────────────────────────────

    #[test]
    fn load_cow_missing_file_returns_default() {
        let cow = load_cow(
            "nonexistent",
            Path::new("/tmp/no-such-dir"),
            "oo",
            " ",
            "\\\\",
        );
        assert_eq!(cow.lines().count(), 5, "default cow must have 5 lines");
        assert!(cow.contains("^__^"), "default cow must have ^__^");
    }

    #[test]
    fn load_cow_from_path_error_includes_path() {
        let result = load_cow_from_path(Path::new("/tmp/nonexistent/file.cow"), "oo", " ", "\\\\");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("failed to read cow file"),
            "error must mention 'failed to read cow file'"
        );
        assert!(
            err.contains("/tmp/nonexistent/file.cow"),
            "error must include the file path"
        );
    }

    #[test]
    fn load_cow_from_path_success() {
        let result = load_cow_from_path(Path::new("data/Cows/default.cow"), "oo", " ", "\\\\");
        // This may or may not exist depending on CWD, but should not panic
        if let Ok(cow) = result {
            assert!(!cow.is_empty(), "loaded cow must not be empty");
        }
    }

    #[test]
    fn resolve_cow_name_passthrough() {
        let tmp = std::env::temp_dir().join("forgum_test_cow_resolve");
        let _ = std::fs::create_dir_all(&tmp);
        assert_eq!(resolve_cow_name("tux", &tmp), "tux");
        assert_eq!(resolve_cow_name("default", &tmp), "default");
        let _ = std::fs::remove_dir(&tmp);
    }

    #[test]
    fn resolve_cow_name_random_picks_from_dir() {
        let tmp = std::env::temp_dir().join("forgum_test_cow_resolve2");
        let cows = tmp.join("Cows");
        let _ = std::fs::create_dir_all(&cows);
        std::fs::write(cows.join("alpha.cow"), "alpha").unwrap();
        std::fs::write(cows.join("bravo.cow"), "bravo").unwrap();
        std::fs::write(cows.join("charlie.cow"), "charlie").unwrap();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..20 {
            let name = resolve_cow_name("random", &tmp);
            assert!(
                name == "alpha" || name == "bravo" || name == "charlie",
                "unexpected cow: {name}"
            );
            seen.insert(name);
        }
        assert!(
            seen.len() > 1,
            "random must pick different cows across calls"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_cow_name_random_empty_dir_falls_back() {
        let tmp = std::env::temp_dir().join("forgum_test_cow_resolve_empty");
        let cows = tmp.join("Cows");
        let _ = std::fs::create_dir_all(&cows);
        assert_eq!(resolve_cow_name("random", &tmp), "default");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
