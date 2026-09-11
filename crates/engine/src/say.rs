use std::path::PathBuf;

use crate::cow;

pub fn execute_say_cmd(cmd: &[String]) -> String {
    let output = forgum_platform::execute_command_with_shell_fallback(cmd)
        .map(|o| {
            let mut out = String::from_utf8_lossy(&o.stdout).to_string();
            if out.trim().is_empty() && !o.stderr.is_empty() {
                out = String::from_utf8_lossy(&o.stderr).to_string();
            }
            out
        })
        .unwrap_or_else(|e| {
            format!(
                "Error running {}: {}",
                cmd.first().map(|s| s.as_str()).unwrap_or(""),
                e
            )
        });

    let text = output.trim().to_string();
    if text.is_empty() {
        "No output.".to_string()
    } else {
        text
    }
}

pub fn run_say(cmd: &[String]) -> String {
    let display_text = execute_say_cmd(cmd);
    let data_dir = forgum_platform::data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cow_text = cow::load_cow("default", &data_dir, "oo", "U", "\\\\");
    cow::compose_scene(&cow_text, &display_text)
}

pub fn run_say_with_image(cmd: &[String], image_path: &std::path::Path) -> Result<String, String> {
    let display_text = execute_say_cmd(cmd);
    let raw_cow = crate::image_ascii::image_to_cow(image_path, "say_mascot", Some(40))
        .map_err(|e| format!("Failed to convert image: {e}"))?;
    let cow_text = cow::expand_cow(&raw_cow, "oo", "U", "\\\\");
    Ok(cow::compose_scene(&cow_text, &display_text))
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return text.lines().map(|l| l.to_string()).collect();
    }
    let mut result = Vec::new();
    for line in text.lines() {
        let char_count = line.chars().count();
        if char_count <= max_width {
            result.push(line.to_string());
        } else {
            let mut chars: Vec<char> = line.chars().collect();
            while chars.len() > max_width {
                let slice = &chars[..max_width];
                let break_at = slice.iter().rposition(|&c| c == ' ').unwrap_or(max_width);
                let line_chunk: String = chars[..break_at].iter().collect();
                result.push(line_chunk);
                let mut remainder = &chars[break_at..];
                while let Some(&' ') = remainder.first() {
                    remainder = &remainder[1..];
                }
                chars = remainder.to_vec();
            }
            if !chars.is_empty() {
                let remaining_chunk: String = chars.into_iter().collect();
                result.push(remaining_chunk);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_text_short_lines() {
        let result = wrap_text("hello\nworld", 20);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn wrap_text_long_line() {
        let result = wrap_text("this is a very long line that needs wrapping", 10);
        assert!(result.len() > 1);
        for line in &result {
            assert!(line.len() <= 10);
        }
    }

    #[test]
    fn wrap_text_empty_string() {
        let result = wrap_text("", 10);
        assert!(result.is_empty());
    }

    #[test]
    fn wrap_text_exact_width() {
        let result = wrap_text("12345", 5);
        assert_eq!(result, vec!["12345"]);
    }

    #[test]
    fn wrap_text_no_spaces() {
        let result = wrap_text("abcdefghij", 5);
        assert_eq!(result, vec!["abcde", "fghij"]);
    }

    #[test]
    fn wrap_text_single_word_no_wrap() {
        let result = wrap_text("hello", 80);
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn wrap_text_one_over_width() {
        let result = wrap_text("hello", 4);
        assert_eq!(result.len(), 2, "should wrap into two lines");
        assert_eq!(result[0], "hell");
        assert_eq!(result[1], "o");
    }

    #[test]
    fn wrap_text_long_word() {
        let long = "a".repeat(100);
        let result = wrap_text(&long, 10);
        assert!(!result.is_empty());
        for line in &result {
            assert!(line.len() <= 10);
        }
    }

    #[test]
    fn wrap_text_unicode_emojis() {
        let crabs = "🦀🦀🦀🦀🦀 🐄🐄🐄🐄🐄 ✨✨✨✨✨";
        let result = wrap_text(crabs, 7);
        assert!(!result.is_empty());
        for line in &result {
            assert!(line.chars().count() <= 7);
        }
    }
}
