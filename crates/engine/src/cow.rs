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
    let sanitized = Path::new(&clean)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("default");
    let safe_clean = if sanitized.is_empty() || sanitized.contains("..") {
        "default"
    } else {
        sanitized
    };
    let resolved = match safe_clean {
        "cow" | "cowsay" | "the-cow" => "default",
        "kittens" => "kitten",
        "nyan-cat" | "nyancat" | "nyan_cat" => "nyan",
        other => other,
    };
    // 1. Try bundled data directory.
    let cow_path = data_dir.join("Cows").join(format!("{resolved}.cow"));
    if let Ok(raw) = forgum_platform::read_text_file_compatible(&cow_path) {
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
        if let Ok(raw) = forgum_platform::read_text_file_compatible(&custom_cow) {
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
    // 2b. Try installed marketplace packs.
    if let Ok(packs_dir) = forgum_platform::marketplace::packs_dir() {
        let pack_cow = packs_dir.join(resolved).join(format!("{resolved}.cow"));
        if let Ok(raw) = forgum_platform::read_text_file_compatible(&pack_cow) {
            crate::log_debug!(
                "cow",
                "Loaded marketplace pack mascot '{resolved}' from {}",
                pack_cow.display()
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
    // 3. Try compile-time embedded built-in mascot catalog (100% zero-disk resilience).
    if let Some(embedded) = forgum_platform::embedded_cows::get_embedded_cow(resolved) {
        crate::log_debug!("cow", "Loaded embedded built-in mascot '{resolved}'");
        let (expanded, landmarks) = expand_cow_with_landmarks(embedded, eyes, tongue, thoughts);
        let final_landmarks = if landmarks.is_empty() {
            detect_cow_eyes(&expanded, 0)
        } else {
            landmarks
        };
        return (expanded, final_landmarks);
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
        let sanitized = Path::new(&clean)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("default");
        let safe_clean = if sanitized.is_empty() || sanitized.contains("..") {
            "default"
        } else {
            sanitized
        };
        let cow_name = match safe_clean {
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
        if forgum_platform::embedded_cows::get_embedded_cow(cow_name).is_some() {
            return cow_name.to_string();
        }
        if let Ok(packs_dir) = forgum_platform::marketplace::packs_dir() {
            if packs_dir.join(cow_name).join(&cow_file).exists() {
                return cow_name.to_string();
            }
        }
        // Mascot not found: find closest match among available cows
        let mut available_names: Vec<String> =
            forgum_platform::embedded_cows::all_embedded_cow_names()
                .iter()
                .map(|s| s.to_string())
                .collect();
        if let Ok(rd) = std::fs::read_dir(data_dir.join("Cows")) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("cow"))
                {
                    if let Some(stem) = p.file_stem().map(|s| s.to_string_lossy().into_owned()) {
                        if !available_names.contains(&stem) {
                            available_names.push(stem);
                        }
                    }
                }
            }
        }
        let name_refs: Vec<&str> = available_names.iter().map(|s| s.as_str()).collect();
        if let Some(closest) = crate::cli::find_closest_match(cow_name, &name_refs) {
            eprintln!(
                "\x1b[1;33m💡 Mascot '{cow_name}' not found. Did you mean '{closest}'? (Using '{closest}'). Run 'forgum list animals' for all available options.\x1b[0m"
            );
            return closest.to_string();
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
        for name in forgum_platform::embedded_cows::all_embedded_cow_names() {
            entries.push(name.to_string());
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
    let raw = forgum_platform::read_text_file_compatible(path)
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

                let glyph = if eye_idx.is_multiple_of(2) {
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
            line = line
                .replace(r"\$", "$")
                .replace(r"\@", "@")
                .replace(r"\#", "#");
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
        'o' | 'O'
            | '@'
            | '^'
            | '*'
            | '$'
            | 'x'
            | 'X'
            | '='
            | '0'
            | 'e'
            | '+'
            | 'v'
            | 'u'
            | 'w'
            | '8'
            | 'Q'
            | '•'
            | '●'
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
                            .filter_map(|(idx, &c)| {
                                if is_eye_glyph(c) {
                                    Some(i + 1 + idx)
                                } else {
                                    None
                                }
                            })
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

/// Strip common leading whitespace across all non-blank lines of mascot art,
/// shifting the animal flush to column 0 to maximize horizontal room in small panes.
#[must_use]
pub fn strip_common_leading_indent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text.to_string();
    }

    lines
        .iter()
        .map(|l| {
            if l.len() >= min_indent && l.chars().take(min_indent).all(|c| c == ' ') {
                &l[min_indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Intelligently compress or sample repeated internal torso, neck, or filler lines proportionally
/// to fit within `max_rows` while strictly preserving all anatomical landmarks (horns, eyes, muzzle, legs, tail).
#[must_use]
pub fn compress_mascot_height_preserving_landmarks(cow_text: &str, max_rows: usize) -> String {
    let raw_lines: Vec<&str> = cow_text.lines().collect();
    if raw_lines.is_empty() || raw_lines.len() <= max_rows {
        return cow_text.to_string();
    }

    // Locate first and last non-empty art lines
    let first_art = match raw_lines.iter().position(|l| !l.trim().is_empty()) {
        Some(pos) => pos,
        None => return cow_text.to_string(),
    };
    let last_art = match raw_lines.iter().rposition(|l| !l.trim().is_empty()) {
        Some(pos) => pos,
        None => return cow_text.to_string(),
    };

    let art_height = last_art.saturating_sub(first_art) + 1;
    if art_height <= max_rows {
        return raw_lines[first_art..=last_art].join("\n");
    }

    // 1. Identify anatomical landmark lines
    let mut landmarks: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();

    // Top landmark: cranial crest / horns / ears
    landmarks.insert(first_art);
    if first_art + 1 < last_art {
        let second_line = raw_lines[first_art + 1];
        if second_line.contains('^') || second_line.contains('/') || second_line.contains('\\') {
            landmarks.insert(first_art + 1);
        }
    }

    // Eyes landmark
    let eyes = detect_cow_eyes(cow_text, 0);
    for (eye_row, _) in eyes {
        if eye_row >= first_art && eye_row <= last_art {
            landmarks.insert(eye_row);
        }
    }

    // Mouth / snout landmark
    for (r, line) in raw_lines
        .iter()
        .enumerate()
        .take(last_art + 1)
        .skip(first_art)
    {
        if line.contains("(__)")
            || line.contains("\\__/")
            || line.contains("(..)")
            || line.contains("(==)")
            || line.contains("\\_/")
            || line.contains("v ")
            || line.contains("w ")
            || line.contains("U ")
        {
            landmarks.insert(r);
            break;
        }
    }

    // Tail landmark
    for (r, line) in raw_lines
        .iter()
        .enumerate()
        .take(last_art + 1)
        .skip(first_art)
    {
        if line.contains(")\\/") || line.contains(")/") || line.contains("~\\") {
            landmarks.insert(r);
            break;
        }
    }

    // Bottom landmarks: feet / hooves
    landmarks.insert(last_art);
    if last_art > first_art + 1 {
        let prev_line = raw_lines[last_art - 1];
        if prev_line.contains("||")
            || prev_line.contains('|')
            || prev_line.contains('/')
            || prev_line.contains('\\')
        {
            landmarks.insert(last_art - 1);
        }
    }

    // If landmark count exceeds max_rows (e.g. extremely tight budget), prioritize landmarks
    if landmarks.len() >= max_rows {
        let mut priority_lines = Vec::new();
        // Priority 1: Eyes
        for (eye_row, _) in detect_cow_eyes(cow_text, 0) {
            if landmarks.contains(&eye_row) && !priority_lines.contains(&eye_row) {
                priority_lines.push(eye_row);
            }
        }
        // Priority 2: Mouth
        for &r in &landmarks {
            let line = raw_lines[r];
            if (line.contains("(__)") || line.contains("\\__/")) && !priority_lines.contains(&r) {
                priority_lines.push(r);
            }
        }
        // Priority 3: Feet
        if !priority_lines.contains(&last_art) {
            priority_lines.push(last_art);
        }
        // Priority 4: Horns / head top
        if !priority_lines.contains(&first_art) {
            priority_lines.push(first_art);
        }
        // Add other landmarks
        for &r in &landmarks {
            if !priority_lines.contains(&r) {
                priority_lines.push(r);
            }
        }

        priority_lines.truncate(max_rows);
        priority_lines.sort_unstable();
        return priority_lines
            .iter()
            .map(|&idx| raw_lines[idx])
            .collect::<Vec<&str>>()
            .join("\n");
    }

    // 2. We have spare budget: sample non-landmark filler rows proportionally
    let remaining_budget = max_rows.saturating_sub(landmarks.len());
    let landmark_vec: Vec<usize> = landmarks.iter().copied().collect();

    // Group non-landmark filler intervals between consecutive landmarks
    let mut intervals: Vec<(usize, usize)> = Vec::new();
    for w in landmark_vec.windows(2) {
        if w[1] > w[0] + 1 {
            intervals.push((w[0] + 1, w[1] - 1));
        }
    }

    let total_filler_count: usize = intervals.iter().map(|(s, e)| e - s + 1).sum();
    let mut sampled_indices: std::collections::BTreeSet<usize> = landmarks.clone();

    if total_filler_count > 0 && remaining_budget > 0 {
        let mut budget_assigned = 0;
        let mut interval_budgets = Vec::new();

        for &(s, e) in &intervals {
            let count = e - s + 1;
            let share = ((count as f32 / total_filler_count as f32) * remaining_budget as f32)
                .round() as usize;
            interval_budgets.push(share);
            budget_assigned += share;
        }

        // Adjust rounding discrepancies
        while budget_assigned > remaining_budget {
            if let Some(max_b) = interval_budgets.iter_mut().max_by_key(|b| **b) {
                if *max_b > 0 {
                    *max_b -= 1;
                    budget_assigned -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        while budget_assigned < remaining_budget {
            if let Some((idx, _)) = intervals.iter().enumerate().max_by_key(|&(i, (s, e))| {
                let count = e - s + 1;
                count.saturating_sub(interval_budgets[i])
            }) {
                interval_budgets[idx] += 1;
                budget_assigned += 1;
            } else {
                break;
            }
        }

        // Sample lines uniformly within each interval
        for (i, &(s, e)) in intervals.iter().enumerate() {
            let k = interval_budgets[i];
            let count = e - s + 1;
            if k == 0 {
                continue;
            }
            if k >= count {
                for line_idx in s..=e {
                    sampled_indices.insert(line_idx);
                }
            } else {
                let step = count as f32 / k as f32;
                for j in 0..k {
                    let offset = ((j as f32 + 0.5) * step).floor() as usize;
                    let pick = (s + offset).min(e);
                    sampled_indices.insert(pick);
                }
            }
        }
    }

    let mut result_indices: Vec<usize> = sampled_indices.into_iter().collect();
    result_indices.sort_unstable();
    result_indices.truncate(max_rows);

    result_indices
        .iter()
        .map(|&idx| raw_lines[idx])
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Compact a mascot's ASCII art to fit comfortably and look aesthetically pleasing
/// in small terminal panes (e.g. 30x30, 20x20, down to 15x15) and enforce Mascot Bounded Height Governor.
#[must_use]
pub fn compact_mascot_for_viewport(
    cow_text: &str,
    target_width: usize,
    target_height: usize,
) -> String {
    let unindented = strip_common_leading_indent(cow_text);
    let line_count = unindented.lines().count();

    // If within comfortable dimensions and not oversized, return unindented
    if target_width > 35 && target_height >= 20 && line_count <= target_height.saturating_sub(4) {
        return unindented;
    }

    // Determine vertical budget (comfortable 8-12 line ceiling in compact viewports)
    let budget_h = if target_height <= 15 {
        7
    } else if target_height <= 20 {
        9
    } else if target_height <= 25 {
        12
    } else {
        target_height.saturating_sub(4).max(12)
    };

    // Apply Mascot Bounded Height Governor if line count exceeds vertical budget
    let height_compacted = if line_count > budget_h {
        compress_mascot_height_preserving_landmarks(&unindented, budget_h)
    } else {
        unindented
    };

    let lines: Vec<&str> = height_compacted.lines().collect();
    let cur_width = lines
        .iter()
        .map(|l| str_display_width(l))
        .max()
        .unwrap_or(0);

    // If already fitting within target_width, return height_compacted art
    if cur_width <= target_width {
        return height_compacted;
    }

    // For nano-panes (width <= 18, e.g. 15x15): compact long ASCII spans like "____" or "----"
    let mut compacted_lines = Vec::with_capacity(lines.len());
    for line in lines {
        if str_display_width(line) <= target_width {
            compacted_lines.push(line.to_string());
        } else {
            let mut s = line.to_string();
            while s.contains("____") && str_display_width(&s) > target_width {
                s = s.replacen("____", "__", 1);
            }
            while s.contains("----") && str_display_width(&s) > target_width {
                s = s.replacen("----", "--", 1);
            }
            while s.contains("    ") && str_display_width(&s) > target_width {
                s = s.replacen("    ", "  ", 1);
            }
            compacted_lines.push(s);
        }
    }

    compacted_lines.join("\n")
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

    let (term_w, term_h) = forgum_platform::terminal_size();
    let is_small_pane = term_w <= 35 || term_h <= 16;
    let line_count = cow_text.lines().count();
    let is_oversized = line_count > (term_h as usize).saturating_sub(4);

    let adapted_cow = if is_small_pane || is_oversized {
        compact_mascot_for_viewport(cow_text, term_w as usize, term_h as usize)
    } else {
        cow_text.to_string()
    };

    let cow_lines: Vec<&str> = adapted_cow.lines().collect();
    let cow_width = cow_lines
        .iter()
        .map(|l| str_display_width(l))
        .max()
        .unwrap_or(0)
        .max(2);

    let term_max = if is_small_pane {
        (term_w as usize).saturating_sub(2).max(6)
    } else {
        (term_w as usize).saturating_sub(6).max(8)
    };

    // In nano-panes (15x15 or small height), compact bubble text if long to preserve mascot headroom
    let effective_bubble_text = if term_w <= 18 && bubble_text.len() > 12 {
        let truncated: String = bubble_text.chars().take(9).collect();
        format!("{truncated}...")
    } else if term_h <= 18 && bubble_text.len() > 24 {
        let truncated: String = bubble_text.chars().take(20).collect();
        format!("{truncated}...")
    } else {
        bubble_text.to_string()
    };

    let bubble = if is_thought {
        wrap_thought_bubble(&effective_bubble_text, cow_width.min(term_max))
    } else {
        wrap_bubble(&effective_bubble_text, cow_width.min(term_max))
    };

    let bubble_w = bubble.lines().next().map(str_display_width).unwrap_or(0);

    // Detect creature head / eyes landmark column for intelligent anchor alignment
    let eyes = detect_cow_eyes(&adapted_cow, 0);
    let head_x = if let Some(&(_, eye_col)) = eyes.first() {
        eye_col
    } else {
        cow_lines
            .iter()
            .find(|l| {
                !l.trim().is_empty() && l.trim() != "o" && l.trim() != "o " && l.trim() != "\\"
            })
            .and_then(|l| l.find(|c: char| !c.is_whitespace()))
            .unwrap_or(2)
    };

    let mut result = String::with_capacity(bubble.len() + adapted_cow.len() + 64);
    result.push_str(&bubble);
    result.push('\n');

    if is_thought {
        let bubble_anchor = head_x.clamp(1, bubble_w.saturating_sub(2).max(1));
        if is_small_pane {
            // Micro-pane: single thought bubble pointer circle to preserve vertical space
            for _ in 0..bubble_anchor {
                result.push(' ');
            }
            result.push_str("o \n");
        } else {
            // Standard pane: three circles
            let p1 = bubble_anchor;
            let p2 = ((bubble_anchor as f32 * 0.60 + head_x as f32 * 0.40).round() as usize).max(1);
            let p3 = ((bubble_anchor as f32 * 0.20 + head_x as f32 * 0.80).round() as usize).max(1);

            for indent in [p1, p2, p3] {
                for _ in 0..indent {
                    result.push(' ');
                }
                result.push_str("o \n");
            }
        }

        // Clean up duplicate thought stem 'o' circles from cow art lines (VIS-001)
        let mut cleaned_lines = Vec::new();
        let mut skipped_standalone = 0;
        let mut processed_art_lines = 0;

        for line in cow_lines.iter() {
            let trimmed = line.trim();
            // Drop standalone 'o' lines at the top of the cow (up to 3)
            if processed_art_lines == 0
                && (trimmed == "o" || trimmed == "o ")
                && skipped_standalone < 3
            {
                skipped_standalone += 1;
                continue;
            }
            // Blank out embedded leading thought 'o' from template $thoughts on the first 3 lines
            if processed_art_lines < 3 {
                if let Some(pos) = line.find('o') {
                    let prefix = &line[..pos];
                    let suffix = &line[pos + 1..];
                    if prefix.chars().all(|c| c.is_whitespace())
                        && (suffix.is_empty()
                            || suffix.starts_with(' ')
                            || suffix.starts_with('\t'))
                    {
                        let mut new_line = line.to_string();
                        new_line.replace_range(pos..=pos, " ");
                        cleaned_lines.push(new_line);
                        if !trimmed.is_empty() {
                            processed_art_lines += 1;
                        }
                        continue;
                    }
                }
            }
            cleaned_lines.push(line.to_string());
            if !trimmed.is_empty() {
                processed_art_lines += 1;
            }
        }
        result.push_str(&cleaned_lines.join("\n"));
        return result;
    } else {
        let has_pointer = cow_lines.iter().take(4).any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('\\') || trimmed.starts_with('/')
        });

        if !has_pointer && !cow_lines.is_empty() {
            let bubble_anchor = head_x.clamp(1, bubble_w.saturating_sub(2).max(1));
            if is_small_pane {
                // Micro-pane: single pointer stroke
                for _ in 0..bubble_anchor {
                    result.push(' ');
                }
                result.push('\\');
                result.push('\n');
            } else {
                let p1_indent = bubble_anchor;
                let p2_indent =
                    ((bubble_anchor as f32 * 0.40 + head_x as f32 * 0.60).round() as usize).max(1);

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
    }

    result.push_str(&adapted_cow);
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

/// Return the visible display width of a string in terminal cells, ignoring non-printing ANSI escapes.
pub fn str_display_width(s: &str) -> usize {
    let stripped = crate::status_line::strip_ansi(s);
    stripped.chars().map(char_display_width).sum()
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
    let term_max = (term_w as usize).saturating_sub(6).max(8);
    let eff_min_w = min_width.min(term_max);
    let max_inner = term_max;
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
        .max(eff_min_w.saturating_sub(2))
        .min(max_inner);
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

    let text_width = raw_text_w;
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
        let name = resolve_cow_name("random", &tmp);
        // With zero-disk embedded cows, an empty directory falls back to embedded mascots
        assert!(
            forgum_platform::embedded_cows::get_embedded_cow(&name).is_some() || name == "default",
            "Random with empty disk dir must resolve to an embedded mascot or default, got: {name}"
        );
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

    #[test]
    fn strip_common_leading_indent_shifts_mascot_left() {
        let indented = "        ^__^\n        (oo)\\_______\n            (__)\\";
        let unindented = strip_common_leading_indent(indented);
        assert!(unindented.starts_with("^__^"));
        assert!(unindented.contains("\n(oo)\\_______"));
        assert!(unindented.contains("\n    (__)\\"));
    }

    #[test]
    fn compact_mascot_for_viewport_fits_in_15x15_nano_pane() {
        let raw_cow = r#"        $thoughts   ^__^
         $thoughts  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#;
        let expanded = expand_cow(raw_cow, "oo", "U", "\\");
        let compacted = compact_mascot_for_viewport(&expanded, 15, 15);
        for line in compacted.lines() {
            assert!(
                str_display_width(line) <= 15,
                "Line '{line}' exceeds 15 columns for nano-pane (len={})",
                str_display_width(line)
            );
        }
    }

    #[test]
    fn test_compress_mascot_height_preserving_landmarks_oversized() {
        // Construct an oversized mascot with 24 lines:
        // Horns, eyes, muzzle, 18 filler torso lines, legs, and feet
        let mut lines = Vec::new();
        lines.push("   ^__^   ");
        lines.push("  (oo)\\_______");
        lines.push("  (__)\\       )\\");
        lines.extend(std::iter::repeat_n("       |     |  ", 18));
        lines.push("      ||----w | ");
        lines.push("      ||     || ");
        lines.push("      db     db ");

        let oversized = lines.join("\n");
        assert_eq!(oversized.lines().count(), 24);

        // Compress to 10 rows
        let compacted = compress_mascot_height_preserving_landmarks(&oversized, 10);
        let compacted_lines: Vec<&str> = compacted.lines().collect();

        assert!(
            compacted_lines.len() <= 10,
            "Compacted lines ({}) must be <= 10",
            compacted_lines.len()
        );

        // Verify all anatomical landmarks are strictly preserved
        assert!(
            compacted.contains("^__^"),
            "Horns/ears landmark must be preserved"
        );
        assert!(
            compacted.contains("(oo)"),
            "Eyes landmark must be preserved"
        );
        assert!(
            compacted.contains("(__)"),
            "Muzzle landmark must be preserved"
        );
        assert!(compacted.contains("db"), "Feet landmark must be preserved");
    }

    #[test]
    fn test_compact_mascot_for_viewport_governs_height_under_25() {
        let mut lines = Vec::new();
        lines.push("   ^__^   ");
        lines.push("  (oo)\\_______");
        lines.push("  (__)\\       )\\");
        lines.extend(std::iter::repeat_n("       |     |  ", 22));
        lines.push("      ||     || ");
        let oversized = lines.join("\n");

        // target_height = 20 (budget_h = 9)
        let compacted = compact_mascot_for_viewport(&oversized, 80, 20);
        assert!(
            compacted.lines().count() <= 10,
            "Mascot height must be bounded to <= 10 lines for target_height=20, got {}",
            compacted.lines().count()
        );
        assert!(compacted.contains("(oo)"), "Eyes must be preserved");
        assert!(compacted.contains("||"), "Feet must be preserved");
    }
}
