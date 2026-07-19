use std::path::PathBuf;
use std::process::Command;

use crate::cow;

pub fn run_say(cmd: &[String]) -> String {
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|e| format!("Error running {}: {}", cmd[0], e));

    let text = output.trim().to_string();
    if text.is_empty() {
        return "No output.".to_string();
    }

    let data_dir = data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cow_text = cow::load_cow("default", &data_dir, "oo", "U", "\\\\");
    cow::compose_scene(&cow_text, &text)
}

fn data_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "cannot determine home directory".to_string())?;
    let data = std::path::PathBuf::from(home).join(".forgum").join("data");
    if data.exists() {
        Ok(data)
    } else {
        Err(format!("data directory not found: {}", data.display()))
    }
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut result = Vec::new();
    for line in text.lines() {
        if line.len() <= max_width {
            result.push(line.to_string());
        } else {
            let mut remaining = line.to_string();
            while remaining.len() > max_width {
                let break_at = remaining[..max_width].rfind(' ').unwrap_or(max_width);
                result.push(remaining[..break_at].to_string());
                remaining = remaining[break_at..].trim_start().to_string();
            }
            if !remaining.is_empty() {
                result.push(remaining);
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
}
