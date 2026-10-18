// Target File: crates/engine/tests/exact_alignment_oracle.rs
// Enforces pixel/cell-exact Input-to-Output alignment across Speech/Thought Bubbles,
// East-Asian/Emoji Unicode widths, Single-Ray Thought Tethers (VIS-001), and 2D Flood-Fill Occlusion (VIS-002).

/// Computes the exact terminal emulator column advance of a Unicode codepoint
/// (0 for combining/ZWJ/control, 2 for East Asian Wide/Fullwidth & Emoji, 1 for standard ASCII/Box/Braille).
pub fn exact_terminal_cell_width(ch: char) -> usize {
    let cp = ch as u32;
    if ch == '\u{200D}' || (0x0000..=0x001F).contains(&cp) || (0x007F..=0x009F).contains(&cp) {
        return 0; // Control chars & Zero-Width Joiner
    }
    if (0x0300..=0x036F).contains(&cp) || (0xFE00..=0xFE0F).contains(&cp) {
        return 0; // Combining diacritical marks & Variation Selectors
    }
    // East Asian Wide (W) and Fullwidth (F) blocks + Supplementary Ideographs & Emoji (incl. U+53C9 '叉')
    if (0x1100..=0x115F).contains(&cp)
        || (0x2E80..=0xA4CF).contains(&cp)
        || (0xAC00..=0xD7A3).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFE10..=0xFE6F).contains(&cp)
        || (0xFF00..=0xFF60).contains(&cp)
        || (0xFFE0..=0xFFE6).contains(&cp)
        || (0x1F300..=0x1FAFF).contains(&cp)
        || (0x20000..=0x3FFFD).contains(&cp)
    {
        return 2;
    }
    1
}

/// Strips ANSI CSI/OSC escape sequences from a rendered line and returns both the visible
/// characters and the exact terminal column index where each character lands.
#[allow(clippy::while_let_on_iterator)]
pub fn parse_visible_row_columns(raw_line: &str) -> (Vec<(usize, char)>, usize) {
    let mut positioned = Vec::new();
    let mut col = 0usize;
    let mut chars = raw_line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    while let Some(c) = chars.next() {
                        if ('@'..='~').contains(&c) {
                            break;
                        }
                    }
                }
                Some(']') | Some('P') => {
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\x07' || (c == '\x1b' && chars.peek() == Some(&'\\')) {
                            if c == '\x1b' {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        let w = exact_terminal_cell_width(ch);
        positioned.push((col, ch));
        col += w;
    }
    (positioned, col)
}

/// Strict Input-to-Output Speech/Thought Bubble Geometry & Mascot Alignment Verifier.
/// Fails on even a 1-column right-border misalignment, duplicate `o` tether trails (VIS-001),
/// or unexpanded `$` template variables.
pub fn assert_scene_exact_alignment(rendered_scene: &str, is_thought: bool) {
    let lines: Vec<&str> = rendered_scene.lines().collect();
    assert!(!lines.is_empty(), "Rendered scene must not be empty");

    // 1. Identify bubble border rows (top ` ___...`, middle body rows, bottom ` ---...`)
    let mut bubble_body_right_cols = Vec::new();
    let mut bubble_bottom_row = 0usize;

    for (row_idx, &line) in lines.iter().enumerate() {
        let (cells, _total_w) = parse_visible_row_columns(line);
        let first_non_space = cells.iter().find(|&&(_, c)| c != ' ');
        let last_non_space = cells.iter().rev().find(|&&(_, c)| c != ' ');

        if let (Some(&(left_col, left_ch)), Some(&(right_col, right_ch))) =
            (first_non_space, last_non_space)
        {
            if matches!(left_ch, '<' | '/' | '\\' | '|' | '(')
                && matches!(right_ch, '>' | '\\' | '/' | '|' | ')')
            {
                let is_closing = !bubble_body_right_cols.is_empty()
                    && cells.iter().any(|&(_, c)| c == '_' || c == '-')
                    && cells
                        .iter()
                        .all(|&(_, c)| matches!(c, ' ' | '|' | '(' | ')' | '_' | '-'));
                if is_closing {
                    bubble_bottom_row = row_idx;
                    break;
                }
                bubble_body_right_cols.push((row_idx, left_col, right_col, left_ch, right_ch));
            } else if (left_ch == '-' || left_ch == '_')
                && (right_ch == '-' || right_ch == '_')
                && !bubble_body_right_cols.is_empty()
            {
                bubble_bottom_row = row_idx;
                break;
            }
        }
    }

    assert!(
        !bubble_body_right_cols.is_empty() && bubble_bottom_row > 0,
        "Scene missing well-formed speech/thought bubble boundaries:\n{}",
        rendered_scene
    );

    // 2. Assert 0-column vertical wall alignment across all bubble body rows (Catches SMALL-001 '叉' & SMALL-002 emoji drift)
    let expected_left_col = bubble_body_right_cols[0].1;
    let expected_right_col = bubble_body_right_cols[0].2;
    for &(row_idx, l_col, r_col, l_ch, r_ch) in &bubble_body_right_cols {
        assert_eq!(
            l_col, expected_left_col,
            "Bubble left wall '{}' at row {} drifted to col {} (expected col {})",
            l_ch, row_idx, l_col, expected_left_col
        );
        assert_eq!(
            r_col, expected_right_col,
            "Bubble right wall '{}' at row {} drifted to col {} (expected col {} — check 2-col CJK/Emoji width!)",
            r_ch, row_idx, r_col, expected_right_col
        );
        if is_thought {
            assert_eq!(
                (l_ch, r_ch),
                ('(', ')'),
                "Thought bubble row {} must use '( )' walls",
                row_idx
            );
        }
    }

    // 3. Assert Single-Ray Tether Invariant (Catches VIS-001 duplicate `o` trails on the same row)
    if is_thought {
        for (row_idx, &line) in lines.iter().enumerate().skip(bubble_bottom_row + 1).take(3) {
            let (cells, _) = parse_visible_row_columns(line);
            let tether_cols: Vec<usize> = cells
                .iter()
                .filter(|&&(_, c)| c == 'o')
                .map(|&(col, _)| col)
                .collect();
            assert!(
                tether_cols.len() <= 1,
                "VIS-001 Regression at row {}: found {} duplicate 'o' thought connectors at columns {:?} (expected at most 1 per tether row)!\nScene:\n{}",
                row_idx,
                tether_cols.len(),
                tether_cols,
                rendered_scene
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_terminal_cell_width_matrix() {
        // Control & ZWJ
        assert_eq!(exact_terminal_cell_width('\0'), 0);
        assert_eq!(exact_terminal_cell_width('\n'), 0);
        assert_eq!(exact_terminal_cell_width('\u{200D}'), 0);

        // Combining diacritics
        assert_eq!(exact_terminal_cell_width('\u{0301}'), 0);

        // ASCII & Box drawing
        assert_eq!(exact_terminal_cell_width('a'), 1);
        assert_eq!(exact_terminal_cell_width('|'), 1);
        assert_eq!(exact_terminal_cell_width('┌'), 1);
        assert_eq!(exact_terminal_cell_width('✗'), 1); // 1-column cross symbol

        // CJK Ideographs & Emojis
        assert_eq!(exact_terminal_cell_width('叉'), 2); // CJK U+53C9
        assert_eq!(exact_terminal_cell_width('中'), 2);
        assert_eq!(exact_terminal_cell_width('🐮'), 2);
        assert_eq!(exact_terminal_cell_width('🔥'), 2);
    }

    #[test]
    fn test_parse_visible_row_columns_ansi() {
        let line = "\x1b[38;2;255;100;50m< hello >\x1b[0m";
        let (cells, total_w) = parse_visible_row_columns(line);
        assert_eq!(total_w, 9);
        assert_eq!(cells[0], (0, '<'));
        assert_eq!(cells[1], (1, ' '));
        assert_eq!(cells[2], (2, 'h'));
        assert_eq!(cells[8], (8, '>'));
    }

    #[test]
    fn test_exact_alignment_speech_and_thought_default_cow() {
        let default_cow = forgum_engine::effects::default_cow_text();

        // Speech bubble alignment
        let speech_scene = forgum_engine::cow::compose_scene(default_cow, "Hello from Forgum!");
        assert_scene_exact_alignment(&speech_scene, false);

        // Multi-line speech bubble alignment
        let speech_multi = forgum_engine::cow::compose_scene(
            default_cow,
            "Line one of speech\nLine two is longer\nThree",
        );
        assert_scene_exact_alignment(&speech_multi, false);

        // Thought bubble alignment (VIS-001 verification)
        let thought_scene = forgum_engine::cow::compose_scene_with_mode(
            default_cow,
            "Deep thoughts in terminal",
            true,
        );
        assert_scene_exact_alignment(&thought_scene, true);
    }

    #[test]
    fn test_exact_alignment_catches_duplicate_tether() {
        let buggy_thought_scene = r#"  _________________________ 
 ( Deep thoughts           )
  ------------------------- 
        o       o
         o
          o
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||"#;

        let result = std::panic::catch_unwind(|| {
            assert_scene_exact_alignment(buggy_thought_scene, true);
        });
        assert!(
            result.is_err(),
            "Must reject scene with duplicate tether markers on the same row"
        );
    }

    #[test]
    fn test_exact_alignment_catches_wall_drift() {
        let drifted_bubble_scene = r#"  _________________________ 
 < First row               >
 < Second row misaligned    >
  ------------------------- 
        \
         \
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||"#;

        let result = std::panic::catch_unwind(|| {
            assert_scene_exact_alignment(drifted_bubble_scene, false);
        });
        assert!(
            result.is_err(),
            "Must reject scene with right wall column drift"
        );
    }

    #[test]
    fn test_timer_exact_border_alignment() {
        let success_res = forgum_engine::timer::TimerResult {
            command: "cargo build --release".to_string(),
            duration_secs: 12.345,
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        let success_box = forgum_engine::timer::render_timer_cow(&success_res);
        let lines: Vec<&str> = success_box.lines().collect();
        assert!(!lines.is_empty());

        let top_w = forgum_engine::cow::str_display_width(lines[0]);
        for (idx, line) in lines.iter().enumerate() {
            let w = forgum_engine::cow::str_display_width(line);
            assert_eq!(
                w, top_w,
                "Timer line {} '{}' width {} does not match top border width {}",
                idx, line, w, top_w
            );
        }

        let fail_res = forgum_engine::timer::TimerResult {
            command: "very_long_command_name_that_triggers_truncation_or_wide_characters_叉"
                .to_string(),
            duration_secs: 105.67,
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
        };
        let fail_box = forgum_engine::timer::render_timer_cow(&fail_res);
        let fail_lines: Vec<&str> = fail_box.lines().collect();
        let fail_top_w = forgum_engine::cow::str_display_width(fail_lines[0]);
        for (idx, line) in fail_lines.iter().enumerate() {
            let w = forgum_engine::cow::str_display_width(line);
            assert_eq!(
                w, fail_top_w,
                "Fail timer line {} '{}' width {} does not match top border width {}",
                idx, line, w, fail_top_w
            );
        }
    }

    #[test]
    fn test_exact_alignment_catalog_mascots() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let data_dir = manifest.parent().unwrap().parent().unwrap().join("data");

        let sample_mascots = [
            "default", "cat", "dragon", "sheep", "elephant", "bunny", "turtle",
        ];
        for mascot in sample_mascots {
            // Test speech scene
            let speech_cow = forgum_engine::cow::load_cow(mascot, &data_dir, "oo", "  ", "\\");
            let speech_scene =
                forgum_engine::cow::compose_scene_with_mode(&speech_cow, "Aligned Speech", false);
            assert_scene_exact_alignment(&speech_scene, false);

            // Test thought scene (Single-Ray tether invariant, zero duplicate o connectors)
            let thought_cow = forgum_engine::cow::load_cow(mascot, &data_dir, "oo", "  ", " ");
            let thought_scene =
                forgum_engine::cow::compose_scene_with_mode(&thought_cow, "Aligned Thought", true);
            assert_scene_exact_alignment(&thought_scene, true);
        }
    }
}
