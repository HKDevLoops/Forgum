//! Fortune cookie loading and selection.
//!
//! Reads fortune files from `data/Fortunes/`, picks a random line, and returns
//! it for the speech bubble. Supports multiple fortune files and user additions.

use std::path::Path;

use rand::seq::SliceRandom;

/// Load all fortunes from the data directory.
///
/// Reads `fortunes.txt` and any `.txt` files in the `Fortunes/` subdirectory.
/// Returns an empty vector if the directory doesn't exist or contains no files.
pub fn load_fortunes(data_dir: &Path) -> Vec<String> {
    let fortunes_dir = data_dir.join("Fortunes");
    let mut all = Vec::new();

    // Read the main fortunes file.
    let main_file = fortunes_dir.join("fortunes.txt");
    if let Ok(content) = std::fs::read_to_string(&main_file) {
        parse_fortunes(&content, &mut all);
    }

    // Read any additional .txt files.
    if let Ok(entries) = std::fs::read_dir(&fortunes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "txt")
                && path.file_name().is_some_and(|n| n != "fortunes.txt")
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    parse_fortunes(&content, &mut all);
                }
            }
        }
    }

    all
}

/// Parse fortune text into individual fortunes.
///
/// Fortunes are separated by `%` on its own line (standard fortune format).
/// If no `%` separators are found, each non-empty line is a separate fortune.
fn parse_fortunes(content: &str, out: &mut Vec<String>) {
    if content.contains('%') {
        // Standard fortune format: % separated
        for fortune in content.split('%') {
            let trimmed = fortune.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
    } else {
        // Line-based format: each non-empty line is a fortune
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
    }
}

/// Pick a random fortune from the list.
pub fn pick_fortune(fortunes: &[String]) -> Option<&str> {
    let mut rng = rand::thread_rng();
    fortunes.choose(&mut rng).map(|s| s.as_str())
}

/// Load and pick a single random fortune from the data directory.
pub fn random_fortune(data_dir: &Path) -> Option<String> {
    let fortunes = load_fortunes(data_dir);
    pick_fortune(&fortunes).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_percent_separated() {
        let mut out = Vec::new();
        parse_fortunes("Fortune 1%\nFortune 2\n%\nFortune 3", &mut out);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], "Fortune 1");
        assert_eq!(out[1], "Fortune 2");
        assert_eq!(out[2], "Fortune 3");
    }

    #[test]
    fn parse_line_based() {
        let mut out = Vec::new();
        parse_fortunes("Line one\nLine two\n\nLine three", &mut out);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], "Line one");
    }

    #[test]
    fn parse_empty_is_empty() {
        let mut out = Vec::new();
        parse_fortunes("", &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn pick_returns_something() {
        let fortunes = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let picked = pick_fortune(&fortunes);
        assert!(picked.is_some());
        assert!(["A", "B", "C"].contains(&picked.unwrap()));
    }

    #[test]
    fn pick_empty_returns_none() {
        let fortunes: Vec<String> = Vec::new();
        assert!(pick_fortune(&fortunes).is_none());
    }

    #[test]
    fn load_fortunes_missing_dir() {
        let fortunes = load_fortunes(Path::new("/tmp/no-such-forgum-dir"));
        assert!(fortunes.is_empty());
    }
}
