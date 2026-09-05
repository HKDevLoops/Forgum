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

use serde::{Deserialize, Serialize};

/// State of the persistent thought deck to ensure 100% fair uniform distribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThoughtDeckState {
    pub corpus_hash: u64,
    pub deck: Vec<usize>,
    pub cursor: usize,
    pub last_drawn: Option<String>,
}

fn compute_corpus_hash(fortunes: &[String]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    fortunes.len().hash(&mut hasher);
    for f in fortunes {
        f.hash(&mut hasher);
    }
    hasher.finish()
}

/// Built-in fallback fortunes when no external files are installed.
const FALLBACK_FORTUNES: &[&str] = &[
    "The cow that never moos has the most to say.",
    "A bug in production is just an unexpected feature in disguise.",
    "Your pasture is currently green; beware of memory leaks.",
    "TrueColor terminal vibes make even compilation errors look artistic.",
    "Simplicity is prerequisite for reliability. — Edsger W. Dijkstra",
    "To iterate is human, to recurse divine. — L. Peter Deutsch",
    "Don't panic! The six-legged cow is always with you in the terminal.",
    "There are only 10 types of people: those who understand binary and those who don't.",
    "Computers are fast, but memory leaks are eternal.",
];

/// Pick a random fortune from the list (legacy independent sampling).
pub fn pick_fortune(fortunes: &[String]) -> Option<&str> {
    let mut rng = rand::thread_rng();
    fortunes.choose(&mut rng).map(|s| s.as_str())
}

/// Pick a fortune using the persistent Fisher-Yates shuffled deck cycle.
///
/// Guarantees:
/// 1. 100% of all thoughts in the corpus are presented before any thought repeats.
/// 2. Uniform coverage without nearby duplicate clustering.
/// 3. Boundary duplicate protection across reshuffles.
pub fn pick_fortune_distributed(fortunes: &[String], data_dir: &Path) -> Option<String> {
    if fortunes.is_empty() {
        return None;
    }

    let corpus_hash = compute_corpus_hash(fortunes);
    let state_file = if data_dir.join("Cows").exists() {
        forgum_platform::runtime_dir()
            .unwrap_or_else(|_| data_dir.to_path_buf())
            .join("thought_state.json")
    } else {
        data_dir.join("thought_state.json")
    };

    let mut state: ThoughtDeckState = std::fs::read_to_string(&state_file)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .filter(|s: &ThoughtDeckState| {
            s.corpus_hash == corpus_hash && s.deck.len() == fortunes.len()
        })
        .unwrap_or_else(|| {
            let mut deck: Vec<usize> = (0..fortunes.len()).collect();
            let mut rng = rand::thread_rng();
            deck.shuffle(&mut rng);
            ThoughtDeckState {
                corpus_hash,
                deck,
                cursor: 0,
                last_drawn: None,
            }
        });

    if state.cursor >= state.deck.len() {
        let mut rng = rand::thread_rng();
        state.deck.shuffle(&mut rng);
        if let Some(ref last) = state.last_drawn {
            if state.deck.len() > 1 && &fortunes[state.deck[0]] == last {
                state.deck.swap(0, 1);
            }
        }
        state.cursor = 0;
    }

    let chosen_idx = state.deck[state.cursor];
    let chosen = fortunes[chosen_idx].clone();
    state.cursor += 1;
    state.last_drawn = Some(chosen.clone());

    if let Ok(json) = serde_json::to_string(&state) {
        let _ = std::fs::write(&state_file, json);
    }

    Some(chosen)
}

/// Load and pick a fortune from the data directory using fair deck distribution.
/// Falls back to built-in fortunes if no external files are found.
pub fn random_fortune(data_dir: &Path) -> Option<String> {
    let fortunes = load_fortunes(data_dir);
    if fortunes.is_empty() {
        let fallback_vec: Vec<String> = FALLBACK_FORTUNES.iter().map(|s| s.to_string()).collect();
        pick_fortune_distributed(&fallback_vec, data_dir)
    } else {
        pick_fortune_distributed(&fortunes, data_dir)
    }
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

    #[test]
    fn distributed_deck_covers_all_items_before_repeat() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fortunes = vec![
            "Alpha".to_string(),
            "Beta".to_string(),
            "Gamma".to_string(),
            "Delta".to_string(),
            "Epsilon".to_string(),
        ];

        let mut first_cycle = Vec::new();
        for _ in 0..fortunes.len() {
            let item = pick_fortune_distributed(&fortunes, temp_dir.path()).unwrap();
            assert!(
                !first_cycle.contains(&item),
                "duplicate before deck exhaustion: {item}"
            );
            first_cycle.push(item);
        }
        assert_eq!(first_cycle.len(), 5);

        // Next draw should start a new cycle and not be empty
        let next_item = pick_fortune_distributed(&fortunes, temp_dir.path()).unwrap();
        assert!(fortunes.contains(&next_item));
    }

    #[test]
    fn distributed_deck_boundary_protection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fortunes = vec!["A".to_string(), "B".to_string()];

        let mut draws = Vec::new();
        for _ in 0..10 {
            draws.push(pick_fortune_distributed(&fortunes, temp_dir.path()).unwrap());
        }

        // Check that no consecutive draws are identical across deck reshuffles
        for window in draws.windows(2) {
            assert_ne!(
                window[0], window[1],
                "consecutive duplicate at boundary: {:?}",
                window
            );
        }
    }
}
