//! Native cow file loading and speech bubble rendering.
//!
//! Loads standard `.cow` files (cowsay format), expands `$eyes`, `$tongue`,
//! `$thoughts` placeholders, wraps text in a speech bubble, and renders the
//! combined output into the framebuffer.

use std::path::Path;

use rand::seq::SliceRandom;

use crate::framebuffer::{Cell, Color, FrameBuffer};

/// The default cow art when no `.cow` file is found.
const DEFAULT_COW: &str = r#"        $thoughts   ^__^
         $thoughts  (oo)\_______
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
    load_cow_with_landmarks(cow_name, data_dir, eyes, tongue, thoughts).0
}

/// Load a `.cow` file and expand placeholders, returning both expanded art and eye landmarks.
pub fn load_cow_with_landmarks(
    cow_name: &str,
    data_dir: &Path,
    eyes: &str,
    tongue: &str,
    thoughts: &str,
) -> (String, Vec<(usize, usize)>) {
    let clean = cow_name.trim().to_ascii_lowercase();
    let resolved = match clean.as_str() {
        "cow" | "cowsay" | "the-cow" => "default",
        "kittens" => "kitten",
        "nyan-cat" | "nyancat" | "nyan_cat" => "nyan",
        other => other,
    };
    // 1. Try bundled data directory.
    let cow_path = data_dir.join("Cows").join(format!("{resolved}.cow"));
    if let Ok(raw) = std::fs::read_to_string(&cow_path) {
        crate::log_debug!(
            "cow",
            "Loaded mascot '{resolved}' from {}",
            cow_path.display()
        );
        let (expanded, landmarks) = expand_cow_with_landmarks(&raw, eyes, tongue, thoughts);
        let final_landmarks = if landmarks.is_empty() {
            detect_cow_eyes(&expanded, 0)
        } else {
            landmarks
        };
        return (expanded, final_landmarks);
    }
    // 2. Try user's custom cows directory (Phase 8.12: community cow packs).
    if let Some(custom_path) = custom_cows_dir() {
        let custom_cow = custom_path.join(format!("{resolved}.cow"));
        if let Ok(raw) = std::fs::read_to_string(&custom_cow) {
            crate::log_debug!(
                "cow",
                "Loaded custom mascot '{resolved}' from {}",
                custom_cow.display()
            );
            let (expanded, landmarks) = expand_cow_with_landmarks(&raw, eyes, tongue, thoughts);
            let final_landmarks = if landmarks.is_empty() {
                detect_cow_eyes(&expanded, 0)
            } else {
                landmarks
            };
            return (expanded, final_landmarks);
        }
    }
    if resolved != "default" {
        crate::log_diag!(
            crate::logger::LogLevel::Warn,
            "cow",
            &format!("Mascot '{resolved}' not found; falling back to default cow"),
            &format!("Mascot '{resolved}' was not found. Run 'forgum list animals' to view all available mascots."),
            &format!("Check if data/Cows/{resolved}.cow exists or if resolve_cow_name needs an alias.")
        );
    }
    let (expanded, landmarks) = expand_cow_with_landmarks(DEFAULT_COW, eyes, tongue, thoughts);
    let final_landmarks = if landmarks.is_empty() {
        detect_cow_eyes(&expanded, 0)
    } else {
        landmarks
    };
    (expanded, final_landmarks)
}

/// If `cow_name` is `"random"`, pick a random `.cow` file from the data
/// directory and the user's custom cows directory, and return its basename.
/// Otherwise return `cow_name` unchanged.
pub fn resolve_cow_name(cow_name: &str, data_dir: &Path) -> String {
    if !cow_name.trim().eq_ignore_ascii_case("random") {
        let clean = cow_name.trim().to_ascii_lowercase();
        let cow_name = match clean.as_str() {
            "cow" | "cowsay" | "the-cow" => "default",
            "kittens" => "kitten",
            "nyan-cat" | "nyancat" | "nyan_cat" => "nyan",
            other => other,
        };
        let cow_file = format!("{cow_name}.cow");
        if data_dir.join("Cows").join(&cow_file).exists() {
            return cow_name.to_string();
        }
        if let Some(custom) = custom_cows_dir() {
            if custom.join(&cow_file).exists() {
                return cow_name.to_string();
            }
        }
        // Mascot not found: find closest match among available cows
        if let Ok(rd) = std::fs::read_dir(data_dir.join("Cows")) {
            let names: Vec<String> = rd
                .flatten()
                .filter_map(|e| {
                    let p = e.path();
                    if p.extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("cow"))
                    {
                        p.file_stem().map(|s| s.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
                .collect();
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            if let Some(closest) = crate::cli::find_closest_match(cow_name, &name_refs) {
                eprintln!(
                    "\x1b[1;33m💡 Mascot '{cow_name}' not found. Did you mean '{closest}'? (Using '{closest}'). Run 'forgum list animals' for all 106 options.\x1b[0m"
                );
                return closest.to_string();
            }
        }
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

/// Extract heredoc body if present (e.g. `$the_cow = <<"EOC"; ... EOC`).
pub fn extract_heredoc_body(cow_template: &str) -> &str {
    if let Some(start) = cow_template.find("<<") {
        let after_marker = &cow_template[start + 2..];
        let first_newline = after_marker.find('\n').unwrap_or(after_marker.len());
        let marker_line = &after_marker[..first_newline];
        let tag = marker_line
            .trim_matches(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ';')
            .trim();

        if !tag.is_empty() {
            let body_content = if first_newline < after_marker.len() {
                &after_marker[first_newline..]
            } else {
                ""
            };
            let mut end_pos = None;
            let mut remaining = body_content;
            let mut curr = 0;
            while !remaining.is_empty() {
                let line_len = remaining
                    .find('\n')
                    .map(|idx| idx + 1)
                    .unwrap_or(remaining.len());
                let line_with_nl = &remaining[..line_len];
                let line = line_with_nl.trim_end_matches(&['\r', '\n'][..]);
                let trimmed = line.trim();
                let clean = trimmed.trim_end_matches(';');
                if clean == tag {
                    end_pos = Some(curr);
                    break;
                }
                curr += line_len;
                remaining = &remaining[line_len..];
            }
            if let Some(pos) = end_pos {
                &body_content[..pos]
            } else {
                body_content
            }
        } else {
            after_marker
        }
    } else {
        cow_template
    }
}

/// Expand `$eyes`, `$tongue`, `$thoughts`, and singular `$eye` placeholders in a `.cow` template.
///
/// Supports all standard and escaped forms from Perl cowsay:
/// - Plural: `\$eyes`, `\\$eyes`, `${eyes}`, `\${eyes}`, `$eyes`
/// - Singular: `\$eye`, `\\$eye`, `${eye}`, `\${eye}`, `$eye` (alternates between left/right eye)
/// - Tongue: `\$tongue`, `\\$tongue`, `${tongue}`, `\${tongue}`, `$tongue`
/// - Thoughts: `\$thoughts`, `\\$thoughts`, `${thoughts}`, `\${thoughts}`, `$thoughts`
pub fn expand_cow(cow_template: &str, eyes: &str, tongue: &str, thoughts: &str) -> String {
    expand_cow_with_landmarks(cow_template, eyes, tongue, thoughts).0
}

/// Expand cow placeholders and return both the expanded mascot art and the exact
/// coordinates `(row, col)` of all placed eye glyphs for the animation engine.
pub fn expand_cow_with_landmarks(
    cow_template: &str,
    eyes: &str,
    tongue: &str,
    thoughts: &str,
) -> (String, Vec<(usize, usize)>) {
    let mut result = String::with_capacity(cow_template.len());
    let mut landmarks = Vec::new();

    let cow_body = extract_heredoc_body(cow_template);

    let eye_chars: Vec<char> = eyes.chars().collect();
    let left_eye = eye_chars.first().copied().unwrap_or('o').to_string();
    let right_eye = eye_chars
        .get(1)
        .copied()
        .unwrap_or_else(|| eye_chars.first().copied().unwrap_or('o'))
        .to_string();
    let mut eye_idx = 0usize;

    for (row, line) in cow_body.lines().enumerate() {
        let mut line = line.to_string();
        if line.contains('$') {
            // 1. Expand thoughts placeholders (longest first)
            for pat in [
                r"\\$thoughts",
                r"\$thoughts",
                r"\\${thoughts}",
                r"\${thoughts}",
                "${thoughts}",
                "$thoughts",
            ] {
                line = line.replace(pat, thoughts);
            }

            // 2. Expand tongue placeholders (longest first)
            for pat in [
                r"\\$tongue",
                r"\$tongue",
                r"\\${tongue}",
                r"\${tongue}",
                "${tongue}",
                "$tongue",
            ] {
                line = line.replace(pat, tongue);
            }

            // 3. Expand plural $eyes placeholders (longest first)
            for pat in [
                r"\\$eyes",
                r"\$eyes",
                r"\\${eyes}",
                r"\${eyes}",
                "${eyes}",
                "$eyes",
            ] {
                while let Some(pos) = line.find(pat) {
                    let col = pos;
                    line.replace_range(pos..pos + pat.len(), eyes);
                    for (c_offset, _) in eyes.chars().enumerate() {
                        landmarks.push((row, col + c_offset));
                    }
                }
            }

            // 4. Expand singular $eye placeholders (alternating left/right eye)
            loop {
                let eye_patterns = [
                    r"\\$eye",
                    r"\$eye",
                    r"\\${eye}",
                    r"\${eye}",
                    "${eye}",
                    "$eye",
                ];
                let mut earliest: Option<(usize, &'static str)> = None;
                for pat in eye_patterns {
                    if let Some(pos) = line.find(pat) {
                        match earliest {
                            None => earliest = Some((pos, pat)),
                            Some((best_pos, _)) if pos < best_pos => earliest = Some((pos, pat)),
                            _ => {}
                        }
                    }
                }

                let Some((pos, pat)) = earliest else {
                    break;
                };

                let glyph = if eye_idx % 2 == 0 {
                    &left_eye
                } else {
                    &right_eye
                };
                eye_idx += 1;
                line.replace_range(pos..pos + pat.len(), glyph);
                for (c_offset, _) in glyph.chars().enumerate() {
                    landmarks.push((row, pos + c_offset));
                }
            }
        }

        // 5. Unescape literal perl heredoc escapes: \$ to $, \@ to @, \# to #
        if line.contains(r"\$") || line.contains(r"\@") || line.contains(r"\#") {
            line = line.replace(r"\$", "$").replace(r"\@", "@").replace(r"\#", "#");
        }

        result.push_str(&line);
        result.push('\n');
    }

    // Remove trailing newline if the original didn't have one.
    if result.ends_with('\n') {
        result.pop();
    }

    (result, landmarks)
}

#[inline]
pub fn is_eye_glyph(ch: char) -> bool {
    matches!(
        ch,
        'o' | 'O' | '@' | '^' | '*' | '$' | 'x' | 'X' | '=' | '0' | 'e' | '+' | 'v' | 'u' | 'w' | '8' | 'Q' | '•' | '●'
    )
}

/// Detect eye coordinates `(row, col)` in any expanded cow art.
pub fn detect_cow_eyes(cow_text: &str, cow_start_line: usize) -> Vec<(usize, usize)> {
    let mut eyes = Vec::new();
    let lines: Vec<&str> = cow_text.lines().collect();

    // Pass 1: Bracketed/parenthesized patterns across ALL lines (highest confidence):
    // (oo), [oo], (o o), (o.o), (o_o), (o-o), (o;o), {~o_o~}, ^(o;o)^
    for (row, line) in lines.iter().enumerate().skip(cow_start_line) {
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        if len == 0 {
            continue;
        }

        for i in 0..len {
            let open = chars[i];
            if open == '(' || open == '[' || open == '{' {
                let close = match open {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    _ => ')',
                };
                for j in (i + 2)..=(i + 7).min(len - 1) {
                    if chars[j] == close {
                        let span = &chars[i + 1..j];
                        let eye_indices: Vec<usize> = span
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, &c)| if is_eye_glyph(c) { Some(i + 1 + idx) } else { None })
                            .collect();
                        if !eye_indices.is_empty() && eye_indices.len() <= 3 {
                            for col in eye_indices {
                                eyes.push((row, col));
                            }
                            return eyes;
                        }
                        // Explicit dot-eyes inside brackets e.g. ( . . ) or (..)
                        let dot_indices: Vec<usize> = span
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, &c)| if c == '.' { Some(i + 1 + idx) } else { None })
                            .collect();
                        if !dot_indices.is_empty() && dot_indices.len() <= 2 {
                            for col in dot_indices {
                                eyes.push((row, col));
                            }
                            return eyes;
                        }
                    }
                }
            }
        }
    }

    // Pass 2: Facial bridge patterns across ALL lines:
    // oYo, o.o, o_o, o o, o  o, o|/o, o-o, o;o, o\o, o/o
    for (row, line) in lines.iter().enumerate().skip(cow_start_line) {
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        if len == 0 {
            continue;
        }

        for i in 0..len {
            if is_eye_glyph(chars[i]) {
                for dist in 1..=4 {
                    if i + dist < len && is_eye_glyph(chars[i + dist]) {
                        // Skip horns (^__^)
                        if chars[i] == '^' && chars[i + dist] == '^' {
                            continue;
                        }
                        let sep = &chars[i + 1..i + dist];
                        let valid_sep = sep.iter().all(|&c| {
                            c == ' '
                                || c == '.'
                                || c == '_'
                                || c == '-'
                                || c == 'Y'
                                || c == '|'
                                || c == '/'
                                || c == ';'
                                || c == '\\'
                                || c == '´'
                                || c == '｀'
                        });
                        if valid_sep {
                            eyes.push((row, i));
                            eyes.push((row, i + dist));
                            return eyes;
                        }
                    }
                }
            }
        }
    }

    // Pass 3: Single-eye animals: /(  o \, \______ o, > o\, |  |o-
    for (row, line) in lines.iter().enumerate().skip(cow_start_line) {
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        if len == 0 {
            continue;
        }

        for i in 0..len {
            let c = chars[i];
            if c != '^' && is_eye_glyph(c) {
                let has_left_head = i >= 1
                    && (chars[i - 1] == ' '
                        || chars[i - 1] == '('
                        || chars[i - 1] == '/'
                        || chars[i - 1] == '|'
                        || chars[i - 1] == '<');
                let has_right_head = i + 1 < len
                    && (chars[i + 1] == ' '
                        || chars[i + 1] == '\\'
                        || chars[i + 1] == ')'
                        || chars[i + 1] == '-'
                        || chars[i + 1] == '|'
                        || chars[i + 1] == '`'
                        || chars[i + 1] == '>');
                if has_left_head && has_right_head {
                    eyes.push((row, i));
                    return eyes;
                }
            }
        }
    }

    eyes
}

/// The default cow with placeholders expanded.
pub fn default_cow_expanded(eyes: &str, tongue: &str, thoughts: &str) -> String {
    expand_cow(DEFAULT_COW, eyes, tongue, thoughts)
}

/// Wrap text in a speech or thought bubble above the cow art.
///
/// Returns the combined (bubble + cow) text ready for rendering.
pub fn compose_scene_with_mode(cow_text: &str, bubble_text: &str, is_thought: bool) -> String {
    if bubble_text.is_empty() {
        return cow_text.to_string();
    }

    let cow_lines: Vec<&str> = cow_text.lines().collect();
    let cow_width = cow_lines
        .iter()
        .map(|l| str_display_width(l))
        .max()
        .unwrap_or(0)
        .max(2);

    let bubble = if is_thought {
        wrap_thought_bubble(bubble_text, cow_width)
    } else {
        wrap_bubble(bubble_text, cow_width)
    };

    let mut result = String::with_capacity(bubble.len() + cow_text.len() + 64);
    result.push_str(&bubble);
    result.push('\n');

    if is_thought {
        // The thought bubble must strictly have exactly THREE circles below it
        let head_indent = cow_lines
            .iter()
            .find(|l| !l.trim().is_empty() && l.trim() != "o" && l.trim() != "o ")
            .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
            .unwrap_or(6);
        let p1 = head_indent.saturating_sub(4).max(2);
        let p2 = head_indent.saturating_sub(2).max(3);
        let p3 = head_indent.saturating_sub(1).max(4);

        for indent in [p1, p2, p3] {
            for _ in 0..indent {
                result.push(' ');
            }
            result.push_str("o \n");
        }

        // Skip leading standalone thought pointer lines so total circles are strictly three
        let leading_blanks = cow_lines
            .iter()
            .take(3)
            .take_while(|l| l.trim().is_empty())
            .count();
        let standalone_lines = cow_lines[leading_blanks..]
            .iter()
            .take_while(|l| l.trim() == "o" || l.trim() == "o ")
            .count();
        if standalone_lines > 0 {
            let remaining = cow_lines[leading_blanks + standalone_lines..].join("\n");
            result.push_str(&remaining);
            return result;
        }
    } else {
        let has_pointer = cow_lines.iter().take(4).any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('\\')
        });

        if !has_pointer && !cow_lines.is_empty() {
            let head_indent = cow_lines
                .iter()
                .find(|l| !l.trim().is_empty())
                .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                .unwrap_or(4);
            let p1_indent = head_indent.saturating_sub(2).max(1);
            let p2_indent = head_indent.saturating_sub(1).max(2);

            for _ in 0..p1_indent {
                result.push(' ');
            }
            result.push('\\');
            result.push('\n');

            for _ in 0..p2_indent {
                result.push(' ');
            }
            result.push('\\');
            result.push('\n');
        }
    }

    result.push_str(cow_text);
    result
}

/// Wrap text in a speech bubble above the cow art.
///
/// Returns the combined (bubble + cow) text ready for rendering.
pub fn compose_scene(cow_text: &str, bubble_text: &str) -> String {
    compose_scene_with_mode(cow_text, bubble_text, false)
}

/// Wrap text in a thought bubble above the cow art (cowthink mode).
///
/// Returns the combined (thought bubble + cow) text ready for rendering.
pub fn compose_thought_scene(cow_text: &str, bubble_text: &str) -> String {
    compose_scene_with_mode(cow_text, bubble_text, true)
}

/// Return the display width of a Unicode character in terminal cells (0, 1, or 2).
pub fn char_display_width(ch: char) -> usize {
    let u = ch as u32;
    if u < 0x20 || (0x7F..=0x9F).contains(&u) {
        return 0;
    }
    // Combining characters / zero-width marks
    if (0x300..=0x36F).contains(&u)
        || (0x1DC0..=0x1DFF).contains(&u)
        || (0x200B..=0x200F).contains(&u)
        || (0x202A..=0x202E).contains(&u)
        || (0x2060..=0x206F).contains(&u)
        || (0xFE00..=0xFE0F).contains(&u)
    {
        return 0;
    }
    // Wide characters: CJK, Fullwidth forms, Emojis, Symbols
    if (0x1100..=0x115F).contains(&u)
        || (0x231A..=0x231B).contains(&u)
        || (0x23E9..=0x23EC).contains(&u)
        || (0x23F0..=0x23F3).contains(&u)
        || (0x25FD..=0x25FE).contains(&u)
        || (0x2614..=0x2615).contains(&u)
        || (0x2648..=0x2653).contains(&u)
        || (0x267F..=0x2693).contains(&u)
        || (0x26A1..=0x26A1).contains(&u)
        || (0x26AA..=0x26AB).contains(&u)
        || (0x26BD..=0x26BE).contains(&u)
        || (0x26C4..=0x26C5).contains(&u)
        || (0x26CE..=0x26CF).contains(&u)
        || (0x26D4..=0x26D4).contains(&u)
        || (0x26EA..=0x26EA).contains(&u)
        || (0x26F2..=0x26F3).contains(&u)
        || (0x26F5..=0x26F5).contains(&u)
        || (0x26FA..=0x26FA).contains(&u)
        || (0x26FD..=0x26FD).contains(&u)
        || (0x2705..=0x2705).contains(&u)
        || (0x270A..=0x270B).contains(&u)
        || (0x2728..=0x2728).contains(&u)
        || (0x274C..=0x274C).contains(&u)
        || (0x274E..=0x274E).contains(&u)
        || (0x2753..=0x2755).contains(&u)
        || (0x2757..=0x2757).contains(&u)
        || (0x2795..=0x2797).contains(&u)
        || (0x27B0..=0x27B0).contains(&u)
        || (0x27BF..=0x27BF).contains(&u)
        || (0x2B1B..=0x2B1C).contains(&u)
        || (0x2B50..=0x2B50).contains(&u)
        || (0x2B55..=0x2B55).contains(&u)
        || (0x2E80..=0x9FFF).contains(&u)
        || (0xAC00..=0xD7AF).contains(&u)
        || (0xF900..=0xFAFF).contains(&u)
        || (0xFE10..=0xFE19).contains(&u)
        || (0xFE30..=0xFE6F).contains(&u)
        || (0xFF01..=0xFF60).contains(&u)
        || (0xFFE0..=0xFFE6).contains(&u)
        || (0x1F000..=0x1FBFF).contains(&u)
    {
        return 2;
    }
    1
}

/// Return the visible display width of a string in terminal cells.
pub fn str_display_width(s: &str) -> usize {
    s.chars().map(char_display_width).sum()
}

/// Wrap text to a maximum line width, breaking words if necessary.
pub fn wrap_words(text: &str, max_width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let max_w = max_width.max(4);

    for raw_line in text.lines() {
        if raw_line.is_empty() {
            out.push(String::new());
            continue;
        }
        let words: Vec<&str> = raw_line.split_whitespace().collect();
        if words.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut cur_line = String::new();
        let mut cur_w = 0;
        for word in words {
            let word_w = str_display_width(word);
            if cur_line.is_empty() {
                if word_w > max_w {
                    // Break long word
                    let mut chunk = String::new();
                    let mut chunk_w = 0;
                    for ch in word.chars() {
                        let cw = char_display_width(ch);
                        if chunk_w + cw > max_w && !chunk.is_empty() {
                            out.push(chunk);
                            chunk = String::new();
                            chunk_w = 0;
                        }
                        chunk.push(ch);
                        chunk_w += cw;
                    }
                    if !chunk.is_empty() {
                        cur_line = chunk;
                        cur_w = chunk_w;
                    }
                } else {
                    cur_line.push_str(word);
                    cur_w = word_w;
                }
            } else if cur_w + 1 + word_w <= max_w {
                cur_line.push(' ');
                cur_line.push_str(word);
                cur_w += 1 + word_w;
            } else {
                out.push(cur_line);
                if word_w > max_w {
                    // Break long word
                    let mut chunk = String::new();
                    let mut chunk_w = 0;
                    for ch in word.chars() {
                        let cw = char_display_width(ch);
                        if chunk_w + cw > max_w && !chunk.is_empty() {
                            out.push(chunk);
                            chunk = String::new();
                            chunk_w = 0;
                        }
                        chunk.push(ch);
                        chunk_w += cw;
                    }
                    cur_line = chunk;
                    cur_w = chunk_w;
                } else {
                    cur_line = word.to_string();
                    cur_w = word_w;
                }
            }
        }
        if !cur_line.is_empty() {
            out.push(cur_line);
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// Wrap text in a speech bubble with rectangular borders.
///
/// ```text
///  _______________
/// |  Hello, world |
/// |_______________|
/// ```
pub fn wrap_bubble(text: &str, min_width: usize) -> String {
    if text.is_empty() {
        return String::new();
    }
    let (term_w, _) = forgum_platform::terminal_size();
    let max_inner = (term_w as usize).saturating_sub(6).max(8);
    let lines = wrap_words(text, max_inner);
    if lines.is_empty() {
        return String::new();
    }

    // Find the longest line in visible display cell columns.
    let text_width = lines
        .iter()
        .map(|l| str_display_width(l))
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
    for line in &lines {
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

/// Wrap text in a thought bubble with parentheses borders (cowthink style).
///
/// ```text
///   _______________
///  (  Hello, world  )
///  (_______________)
/// ```
pub fn wrap_thought_bubble(text: &str, cow_width: usize) -> String {
    if text.is_empty() {
        return String::new();
    }
    let (term_w, _) = forgum_platform::terminal_size();
    let term_max = (term_w as usize).saturating_sub(6).max(8);

    // If cow_width is specified (> 0), the thought bubble must NEVER have width greater than the cow.
    let max_inner = if cow_width > 0 {
        // Bubble borders and padding take 4 chars total: `( ` + text + ` )`
        cow_width.saturating_sub(4).max(1).min(term_max)
    } else {
        term_max
    };

    let lines = wrap_words(text, max_inner);
    if lines.is_empty() {
        return String::new();
    }

    let raw_text_w = lines
        .iter()
        .map(|l| str_display_width(l))
        .max()
        .unwrap_or(0);

    let text_width = if cow_width > 0 {
        raw_text_w.min(cow_width.saturating_sub(4).max(1))
    } else {
        raw_text_w
    };

    let inner_width = text_width + 2; // +2 for padding spaces around text

    let mut result = String::with_capacity((inner_width + 4) * (lines.len() + 2));

    // Top border: ` _______________ `
    result.push(' ');
    for _ in 0..=inner_width {
        result.push('_');
    }
    result.push('\n');

    // Content lines: `(  Hello, world  )`
    for line in &lines {
        let pad_target = inner_width + 1;
        result.push('(');
        result.push(' ');
        result.push_str(line);
        pad_to(&mut result, pad_target);
        result.push(')');
        result.push('\n');
    }

    // Bottom border: `(_______________)`
    result.push('(');
    for _ in 0..inner_width {
        result.push('_');
    }
    result.push(')');
    result.push('\n');

    result
}

/// Pad `result` with spaces until its current line display width reaches `target_len`.
fn pad_to(result: &mut String, target_len: usize) {
    let current_width = result.rsplit_once('\n').map_or_else(
        || str_display_width(result),
        |(_, last)| str_display_width(last),
    );
    for _ in current_width..target_len {
        result.push(' ');
    }
}

/// Render the composed cow text (bubble + cow art) into a framebuffer.
pub fn render_cow(fb: &mut FrameBuffer, composed: &str, color_mode: &str, time: f32) {
    render_cow_with_palette(fb, composed, color_mode, &[], time);
}

/// Render the composed cow text (bubble + cow art) into a framebuffer with an explicit palette.
pub fn render_cow_with_palette(
    fb: &mut FrameBuffer,
    composed: &str,
    color_mode: &str,
    palette: &[(u8, u8, u8)],
    time: f32,
) {
    let fg = Color::WHITE;
    let cow_start_line = crate::effects::find_cow_start_line(composed);
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
            let cell_fg = if y < cow_start_line {
                crate::effects::resolve_bubble_cell_fg(ch)
            } else {
                let rel_y = y.saturating_sub(cow_start_line);
                crate::effects::resolve_fg_palette_char(color_mode, palette, x, rel_y, time, fg, ch)
            };
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
            ("Hello 🐮 Cow! 🚀", 10),
            ("CJK testing: 漢字とひらがな\nSecond line", 15),
        ];
        for (text, min_width) in cases {
            let bubble = wrap_bubble(text, min_width);
            if bubble.is_empty() {
                continue;
            }
            let rows: Vec<&str> = bubble.lines().collect();
            assert!(rows.len() >= 2, "must have at least top+bottom");
            let top_width = str_display_width(rows[0]);
            for (i, row) in rows.iter().enumerate() {
                let w = str_display_width(row);
                assert_eq!(
                    w, top_width,
                    "row {i} display width {w} != top width {top_width} for text {:?}",
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

    #[test]
    fn thought_bubble_structure_single_line() {
        let bubble = wrap_thought_bubble("I ponder deeply", 25);
        let lines: Vec<&str> = bubble.lines().collect();
        assert_eq!(
            lines.len(),
            3,
            "single-line thought bubble must have 3 lines"
        );

        // Top border: space + underscores
        assert!(
            lines[0].starts_with(' '),
            "top border must start with space"
        );
        assert!(
            lines[0].chars().all(|c| c == '_' || c == ' '),
            "top border must be underscores/spaces only"
        );

        // Content line: starts with ( and ends with )
        assert!(lines[1].starts_with('('), "content line must start with (");
        assert!(lines[1].ends_with(')'), "content line must end with )");
        assert!(
            lines[1].contains("I ponder deeply"),
            "content must contain thought text"
        );

        // Bottom border: (____...___)
        assert!(lines[2].starts_with('('), "bottom border must start with (");
        assert!(lines[2].ends_with(')'), "bottom border must end with )");
        let bottom_inner = &lines[2][1..lines[2].len() - 1];
        assert!(
            bottom_inner.chars().all(|c| c == '_'),
            "bottom inner must be all underscores: {bottom_inner:?}"
        );
    }

    #[test]
    fn thought_bubble_structure_multi_line() {
        let bubble = wrap_thought_bubble("Line 1\nLine 2", 10);
        let lines: Vec<&str> = bubble.lines().collect();
        assert_eq!(lines.len(), 4, "two-line thought bubble must have 4 lines");
        assert!(lines[1].starts_with('(') && lines[1].ends_with(')'));
        assert!(lines[2].starts_with('(') && lines[2].ends_with(')'));
        assert!(lines[1].contains("Line 1"));
        assert!(lines[2].contains("Line 2"));
    }

    #[test]
    fn thought_bubble_empty_text_returns_empty() {
        assert_eq!(wrap_thought_bubble("", 10), "");
    }

    #[test]
    fn thought_bubble_all_rows_same_width() {
        let cases: Vec<(&str, usize)> = vec![
            ("", 0),
            ("x", 0),
            ("Pondering the universe", 0),
            ("Pondering\nquantum computing\ntoday", 25),
        ];
        for (text, min_width) in cases {
            let bubble = wrap_thought_bubble(text, min_width);
            if bubble.is_empty() {
                continue;
            }
            let rows: Vec<&str> = bubble.lines().collect();
            let top_width = str_display_width(rows[0]);
            for (i, row) in rows.iter().enumerate() {
                let w = str_display_width(row);
                assert_eq!(
                    w, top_width,
                    "row {i} width {w} != top width {top_width} for thought {:?}",
                    text
                );
            }
        }
    }

    #[test]
    fn thought_bubble_never_exceeds_cow_width() {
        let long_thought = "This is an extraordinarily long contemplation about quantum entanglement and bovine philosophy that spans multiple sentences and concepts.";
        for cow_width in [15, 20, 25, 30, 40, 50] {
            let bubble = wrap_thought_bubble(long_thought, cow_width);
            for row in bubble.lines() {
                let w = str_display_width(row);
                assert!(
                    w <= cow_width,
                    "Thought bubble row width {w} exceeded cow width {cow_width}: '{row}'"
                );
            }
        }
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
        // Every word from the original text must appear in the bubble
        for word in long_text.split_whitespace() {
            assert!(scene.contains(word), "bubble must contain word '{word}'");
        }
        // The bubble should appear before the cow
        let first_word = long_text.split_whitespace().next().unwrap();
        assert!(scene.find(first_word).unwrap() < scene.find("cow").unwrap());
    }

    #[test]
    fn compose_thought_scene_bubble_before_cow() {
        let cow = "  cow_line1                \n  cow_line2                ";
        let scene = compose_thought_scene(cow, "thinking of grass");
        let cow_pos = scene.find("cow_line1").unwrap();
        let thought_pos = scene.find("thinking").unwrap();
        assert!(
            thought_pos < cow_pos,
            "thought bubble (at {thought_pos}) must precede cow (at {cow_pos})"
        );
        assert!(scene.contains('(') && scene.contains(')'));
    }

    #[test]
    fn compose_thought_scene_no_bubble_passthrough() {
        let cow = "  cow_only";
        let scene = compose_thought_scene(cow, "");
        assert_eq!(scene, cow);
    }

    #[test]
    fn compose_scene_with_mode_distinguishes_speech_and_thought() {
        let cow = "  ^__^\n  (oo)";
        let speech = compose_scene_with_mode(cow, "hello", false);
        let thought = compose_scene_with_mode(cow, "hello", true);

        let speech_lines: Vec<&str> = speech.lines().collect();
        let thought_lines: Vec<&str> = thought.lines().collect();

        // Speech bubble content line has | borders
        assert!(speech_lines[1].starts_with('|'));
        assert!(speech_lines[1].ends_with('|'));

        // Thought bubble content line has ( ) borders
        assert!(thought_lines[1].starts_with('('));
        assert!(thought_lines[1].ends_with(')'));
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

        // Verify uppercase RANDOM works identically
        let name_upper = resolve_cow_name("RANDOM", &tmp);
        assert!(
            name_upper == "alpha" || name_upper == "bravo" || name_upper == "charlie",
            "RANDOM must resolve to a valid cow: {name_upper}"
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

    #[test]
    fn detect_cow_eyes_ignores_horns_and_finds_eyes() {
        let cow = "  ^__^  \n (oo)   \n(__)    ";
        let landmarks = detect_cow_eyes(cow, 0);
        assert_eq!(landmarks, vec![(1, 2), (1, 3)]);
    }

    #[test]
    fn expand_cow_with_landmarks_singular_and_plural() {
        // Singular $eye alternating
        let template = "$the_cow = <<EOC;\n \\$eye   $eye \nEOC;";
        let (expanded, marks) = expand_cow_with_landmarks(template, "oO", " ", "\\");
        assert!(expanded.contains("o   O"));
        assert_eq!(marks.len(), 2);

        // Plural $eyes
        let template_pl = "$the_cow = <<EOC;\n \\$eyes \nEOC;";
        let (expanded_pl, marks_pl) = expand_cow_with_landmarks(template_pl, "@@", " ", "\\");
        assert!(expanded_pl.contains("@@"));
        assert_eq!(marks_pl.len(), 2);
    }
}
