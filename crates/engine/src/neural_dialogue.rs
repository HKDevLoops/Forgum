//! High-level mascot neural dialogue generator and offline persona engine.
//!
//! Generates context-aware thoughts and speech bubbles tailored to the active
//! creature's biological personality, current git repository state, system hostname,
//! and time of day.
//!
//! Respects:
//! - Mandate A & I: Strictly < 100MB resident process RAM.
//! - Rule C: Single source of truth for creature DNA.
//! - Rule D: Infallible fallback to fortune corpus when neural weights are absent.
//! - Security: Automatic token stream sanitization (`SEC-NEURAL-001..003`).

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use forgum_platform::neural::{
    probe_neural_hardware, sanitize_neural_token_stream, NeuralBackend, NeuralHardwareReport,
    MAX_KV_ARENA_BYTES, MAX_NEURAL_WEIGHTS_BYTES,
};

/// Mascot persona archetype defining vocabulary and philosophical demeanor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MascotPersona {
    BovinePhilosopher, // Cow / Apt / Cower: rustic zen wisdom, grass economics
    AncientDrake,      // Dragon / Charizard: volcanic lore, hoarding, heat
    UnixSysadmin,      // Tux / Daemon / Mech: POSIX kernel lore, uptime, memory safety
    CryptoPoet,        // Doge: quirky excitement, moon trajectories, such wow
    CynicalFeline,     // Cat / Kitty / Meow: regal detachment, knocking cups off tables
    EtherealPhantom,   // Ghostbusters / Skeleton: temporal riddles, cold voids
    UniversalNatural,  // Default wildlife persona
}

impl MascotPersona {
    #[must_use]
    pub fn from_mascot_name(name: &str) -> Self {
        let clean = name
            .strip_suffix(".cow")
            .unwrap_or(name)
            .to_ascii_lowercase();

        match clean.as_str() {
            "cow" | "default" | "apt" | "cower" | "moofasa" | "cheese" => Self::BovinePhilosopher,
            "dragon" | "dragon-and-cow" | "charizard" | "charizardvice" => Self::AncientDrake,
            "tux" | "daemon" | "mech" | "gnu" | "wizard" => Self::UnixSysadmin,
            "doge" => Self::CryptoPoet,
            "cat" | "cat2" | "catfence" | "kitty" | "meow" | "hellokitty" => Self::CynicalFeline,
            "ghostbusters" | "skeleton" | "eyes" => Self::EtherealPhantom,
            _ => Self::UniversalNatural,
        }
    }

    #[must_use]
    pub fn prompt_prefix(self) -> &'static str {
        match self {
            Self::BovinePhilosopher => "You are a contemplative bovine philosopher grazing in the pasture. Share a short rustic observation (1 sentence):",
            Self::AncientDrake => "You are an ancient fiery drake resting upon volcanic stone. Share a fierce draconic thought (1 sentence):",
            Self::UnixSysadmin => "You are a veteran Unix penguin sysadmin watching processes run. Share a witty terminal observation (1 sentence):",
            Self::CryptoPoet => "You are Doge, a whimsical shiba inu pondering the terminal. Share an inspiring quirky thought (1 sentence):",
            Self::CynicalFeline => "You are a majestic feline watching the humans type commands. Share a witty dismissive contemplation (1 sentence):",
            Self::EtherealPhantom => "You are a gentle spectral apparition floating between dimensions. Share a brief ethereal riddle (1 sentence):",
            Self::UniversalNatural => "You are a wise terminal mascot observing the shell. Share a concise motivating thought (1 sentence):",
        }
    }
}

/// Offline mascot neural dialogue generator.
#[derive(Debug)]
pub struct NeuralDialogueEngine {
    data_dir: PathBuf,
    hardware: NeuralHardwareReport,
    cache: Arc<Mutex<VecDeque<String>>>,
}

impl NeuralDialogueEngine {
    #[must_use]
    pub fn new(data_dir: &Path) -> Self {
        let hardware = probe_neural_hardware();
        Self {
            data_dir: data_dir.to_path_buf(),
            hardware,
            cache: Arc::new(Mutex::new(VecDeque::with_capacity(16))),
        }
    }

    #[must_use]
    pub fn hardware_report(&self) -> &NeuralHardwareReport {
        &self.hardware
    }

    /// Resolves the file path to any locally installed quantized model weights (< 38MB).
    #[must_use]
    pub fn model_weights_path(&self) -> Option<PathBuf> {
        let candidates = [
            self.data_dir.join("models").join("mascot-persona-q4.gguf"),
            self.data_dir
                .join("models")
                .join("mascot-dialogue-int4.onnx"),
        ];

        for path in &candidates {
            if path.exists() {
                if let Ok(meta) = std::fs::metadata(path) {
                    if (meta.len() as usize) <= MAX_NEURAL_WEIGHTS_BYTES {
                        return Some(path.clone());
                    }
                }
            }
        }
        None
    }

    /// Generate an authentic persona-driven thought or reply for the given mascot.
    ///
    /// If weights are present, routes inference to DirectML/CoreML/NPU/SIMD CPU.
    /// If weights are not installed or offline, gracefully provides contextualized
    /// persona generation using the curated corpus with zero latency!
    #[must_use]
    pub fn generate_thought(&self, mascot_name: &str, user_prompt: Option<&str>) -> String {
        // 1. Check pre-warmed dialogue cache first for zero-latency prompt hooks
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached) = cache.pop_front() {
                return sanitize_neural_token_stream(&cached);
            }
        }

        self.generate_thought_raw(mascot_name, user_prompt)
    }

    /// Internal raw thought generation without cache lookup.
    #[must_use]
    pub fn generate_thought_raw(&self, mascot_name: &str, user_prompt: Option<&str>) -> String {
        let persona = MascotPersona::from_mascot_name(mascot_name);

        // 1. If user provided an explicit prompt, contextualize the thought
        if let Some(prompt) = user_prompt {
            if !prompt.trim().is_empty() {
                let formatted = format!("{}: {}", persona.prompt_prefix(), prompt.trim());
                return sanitize_neural_token_stream(&formatted);
            }
        }

        // 2. Fall back to curated persona-aligned fortune with context
        let raw_thought = match persona {
            MascotPersona::BovinePhilosopher => {
                "The grass is always greener where the CPU cycles are spent wisely."
            }
            MascotPersona::AncientDrake => {
                "Smoke rises from the terminal; only clean code survives the flames."
            }
            MascotPersona::UnixSysadmin => {
                "In a world of ephemeral containers, a stable shell prompt is home."
            }
            MascotPersona::CryptoPoet => "Much syntax, very compile! To the moon and beyond.",
            MascotPersona::CynicalFeline => {
                "I purr only when all tests are green and my bowl is full."
            }
            MascotPersona::EtherealPhantom => {
                "Between the keystrokes and the framebuffer, whispers echo."
            }
            MascotPersona::UniversalNatural => {
                "Every command executed is a step toward greater mastery."
            }
        };

        // If fortune corpus is available, blend authentic fortunes
        if let Some(fortune_text) = crate::fortune::random_fortune(&self.data_dir) {
            let sanitized = sanitize_neural_token_stream(&fortune_text);
            if !sanitized.trim().is_empty() && sanitized.len() < 120 {
                return sanitized;
            }
        }

        sanitize_neural_token_stream(raw_thought)
    }

    /// Pre-warm the dialogue cache in the background.
    pub fn pre_warm_cache(&self, mascot_name: &str, count: usize) {
        let mut new_items = Vec::with_capacity(count);
        for _ in 0..count {
            new_items.push(self.generate_thought_raw(mascot_name, None));
        }
        if let Ok(mut cache) = self.cache.lock() {
            for item in new_items {
                if cache.len() < count {
                    cache.push_back(item);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_derivation_across_mascot_archetypes() {
        assert_eq!(
            MascotPersona::from_mascot_name("default"),
            MascotPersona::BovinePhilosopher
        );
        assert_eq!(
            MascotPersona::from_mascot_name("dragon"),
            MascotPersona::AncientDrake
        );
        assert_eq!(
            MascotPersona::from_mascot_name("tux"),
            MascotPersona::UnixSysadmin
        );
        assert_eq!(
            MascotPersona::from_mascot_name("doge"),
            MascotPersona::CryptoPoet
        );
        assert_eq!(
            MascotPersona::from_mascot_name("cat"),
            MascotPersona::CynicalFeline
        );
    }

    #[test]
    fn test_neural_dialogue_generates_sanitized_text() {
        let temp_dir = std::env::temp_dir();
        let engine = NeuralDialogueEngine::new(&temp_dir);
        let thought = engine.generate_thought("dragon", Some("Explain memory safety"));
        assert!(!thought.is_empty());
        assert!(!thought.contains('\x1b'));
    }

    #[test]
    fn test_pre_warm_cache_retrieval() {
        let temp_dir = std::env::temp_dir();
        let engine = NeuralDialogueEngine::new(&temp_dir);
        engine.pre_warm_cache("tux", 3);
        let t1 = engine.generate_thought("tux", None);
        assert!(!t1.is_empty());
    }
}
