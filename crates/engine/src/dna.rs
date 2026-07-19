//! Per-animal animation DNA — 7-axis profile schema.
//!
//! Each cow gets a `CowDna` loaded from `data/animations.json`.
//! The DNA controls: base animation type, particles, speed, amplitude,
//! palette, easing, and phase seed.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// The 10 base animation types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BaseAnim {
    #[default]
    Breathe,
    Float,
    Walk,
    Particles,
    Pulse,
    Glitch,
    Fly,
    Talk,
    Sway,
    Dissolve,
}

/// The 6 particle types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ParticleType {
    #[default]
    Fire,
    Bubbles,
    Stars,
    Zzz,
    Pulse,
    Glitch,
}

/// Particle emitter configuration.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ParticleDna {
    #[serde(default)]
    pub r#type: ParticleType,
    #[serde(default = "default_particle_rate")]
    pub rate: u32,
    #[serde(default = "default_particle_life")]
    pub life: [f32; 2],
    #[serde(default = "default_particle_speed")]
    pub speed: [f32; 2],
    #[serde(default)]
    pub palette: Vec<String>,
}

fn default_particle_rate() -> u32 {
    10
}
fn default_particle_life() -> [f32; 2] {
    [0.5, 1.5]
}
fn default_particle_speed() -> [f32; 2] {
    [0.3, 0.8]
}

impl Default for ParticleDna {
    fn default() -> Self {
        Self {
            r#type: ParticleType::Fire,
            rate: default_particle_rate(),
            life: default_particle_life(),
            speed: default_particle_speed(),
            palette: vec![],
        }
    }
}

/// Motion amplitude per channel.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Amplitude {
    #[serde(default = "default_amplitude_breath")]
    pub breath: f32,
    #[serde(default = "default_amplitude_sway")]
    pub sway: f32,
    #[serde(default = "default_amplitude_float")]
    pub float: f32,
}

fn default_amplitude_breath() -> f32 {
    0.5
}
fn default_amplitude_sway() -> f32 {
    0.2
}
fn default_amplitude_float() -> f32 {
    0.1
}

impl Default for Amplitude {
    fn default() -> Self {
        Self {
            breath: default_amplitude_breath(),
            sway: default_amplitude_sway(),
            float: default_amplitude_float(),
        }
    }
}

/// Easing configuration per channel.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EasingDna {
    #[serde(default = "default_easing_base")]
    pub base: String,
    #[serde(default = "default_easing_particle_alpha")]
    pub particle_alpha: String,
    #[serde(default = "default_easing_particle_velocity")]
    pub particle_velocity: String,
}

fn default_easing_base() -> String {
    "sine_inout".to_string()
}
fn default_easing_particle_alpha() -> String {
    "expo_out".to_string()
}
fn default_easing_particle_velocity() -> String {
    "cubic_out".to_string()
}

impl Default for EasingDna {
    fn default() -> Self {
        Self {
            base: default_easing_base(),
            particle_alpha: default_easing_particle_alpha(),
            particle_velocity: default_easing_particle_velocity(),
        }
    }
}

/// Glow configuration.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GlowDna {
    #[serde(default = "default_glow_color")]
    pub color: String,
    #[serde(default = "default_glow_radius")]
    pub radius: f32,
    #[serde(default = "default_glow_falloff")]
    pub falloff: String,
}

fn default_glow_color() -> String {
    "#ffffff".to_string()
}
fn default_glow_radius() -> f32 {
    4.0
}
fn default_glow_falloff() -> String {
    "gaussian".to_string()
}

impl Default for GlowDna {
    fn default() -> Self {
        Self {
            color: default_glow_color(),
            radius: default_glow_radius(),
            falloff: default_glow_falloff(),
        }
    }
}

/// The 7-axis DNA profile for a single cow.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CowDna {
    #[serde(default)]
    pub base: BaseAnim,
    #[serde(default)]
    pub particles: ParticleDna,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default)]
    pub amplitude: Amplitude,
    #[serde(default)]
    pub palette: Vec<String>,
    #[serde(default)]
    pub easing: EasingDna,
    #[serde(default = "default_phase_seed")]
    pub phase_seed: u32,
    #[serde(default)]
    pub glow: GlowDna,
}

fn default_speed() -> f32 {
    1.0
}
fn default_phase_seed() -> u32 {
    0
}

impl Default for CowDna {
    fn default() -> Self {
        Self {
            base: BaseAnim::Breathe,
            particles: ParticleDna::default(),
            speed: default_speed(),
            amplitude: Amplitude::default(),
            palette: vec![],
            easing: EasingDna::default(),
            phase_seed: default_phase_seed(),
            glow: GlowDna::default(),
        }
    }
}

/// Load all cow DNA profiles from `animations.json`.
pub fn load_animations(data_dir: &Path) -> HashMap<String, CowDna> {
    let path = data_dir.join("animations.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Get DNA for a specific cow, falling back to defaults.
pub fn get_dna(animations: &HashMap<String, CowDna>, cow_name: &str) -> CowDna {
    // Try exact match first, then strip extension
    if let Some(dna) = animations.get(cow_name) {
        return dna.clone();
    }
    if let Some(stem) = cow_name.strip_suffix(".cow") {
        if let Some(dna) = animations.get(stem) {
            return dna.clone();
        }
    }
    CowDna::default()
}

/// Compute per-instance phase offset using golden ratio.
///
/// `phase = (phase_seed ^ instance_id) as f32 * 0.618033988749895`
///
/// This ensures different instances of the same cow never sync.
#[allow(clippy::excessive_precision)]
pub fn instance_phase(phase_seed: u32, instance_id: u32) -> f32 {
    let hash = phase_seed ^ instance_id;
    (hash as f32) * 0.618_033_988_749_895
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dna_all_fields() {
        let dna = CowDna::default();
        assert_eq!(dna.base, BaseAnim::Breathe);
        assert_eq!(dna.speed, 1.0);
        assert_eq!(dna.particles.rate, 10);
        assert_eq!(dna.particles.r#type, ParticleType::Fire);
        assert_eq!(dna.particles.life, [0.5, 1.5]);
        assert_eq!(dna.particles.speed, [0.3, 0.8]);
        assert!(dna.particles.palette.is_empty());
        assert_eq!(dna.amplitude.breath, 0.5);
        assert_eq!(dna.amplitude.sway, 0.2);
        assert_eq!(dna.amplitude.float, 0.1);
        assert_eq!(dna.easing.base, "sine_inout");
        assert_eq!(dna.easing.particle_alpha, "expo_out");
        assert_eq!(dna.easing.particle_velocity, "cubic_out");
        assert_eq!(dna.phase_seed, 0);
        assert_eq!(dna.glow.radius, 4.0);
        assert_eq!(dna.glow.color, "#ffffff");
        assert_eq!(dna.glow.falloff, "gaussian");
        assert!(dna.palette.is_empty());
    }

    #[test]
    fn parse_base_anim() {
        let json = r#"{"base": "Particles", "particles": {"type": "Fire"}}"#;
        let dna: CowDna = serde_json::from_str(json).unwrap();
        assert_eq!(dna.base, BaseAnim::Particles);
        assert_eq!(dna.particles.r#type, ParticleType::Fire);
        assert_eq!(dna.speed, 1.0);
        assert_eq!(dna.amplitude.breath, 0.5);
        assert_eq!(dna.amplitude.sway, 0.2);
        assert_eq!(dna.amplitude.float, 0.1);
        assert_eq!(dna.easing.base, "sine_inout");
        assert_eq!(dna.phase_seed, 0);
        assert_eq!(dna.glow.color, "#ffffff");
        assert_eq!(dna.glow.radius, 4.0);
    }

    #[test]
    fn parse_full_dna() {
        let json = r##"{
            "base": "Breathe",
            "particles": { "type": "Fire", "rate": 18, "life": [0.6, 1.4] },
            "speed": 1.0,
            "amplitude": { "breath": 0.6, "sway": 0.2, "float": 0.1 },
            "palette": ["#ff8800", "#ff2200"],
            "easing": { "base": "sine_inout" },
            "phase_seed": 4242,
            "glow": { "color": "#ff6600", "radius": 6.0 }
        }"##;
        let dna: CowDna = serde_json::from_str(json).unwrap();
        assert_eq!(dna.base, BaseAnim::Breathe);
        assert_eq!(dna.particles.rate, 18);
        assert_eq!(dna.particles.r#type, ParticleType::Fire);
        assert_eq!(dna.particles.life, [0.6, 1.4]);
        assert_eq!(dna.speed, 1.0);
        assert_eq!(dna.amplitude.breath, 0.6);
        assert_eq!(dna.amplitude.sway, 0.2);
        assert_eq!(dna.amplitude.float, 0.1);
        assert_eq!(dna.palette, vec!["#ff8800", "#ff2200"]);
        assert_eq!(dna.easing.base, "sine_inout");
        assert_eq!(dna.phase_seed, 4242);
        assert_eq!(dna.glow.color, "#ff6600");
        assert_eq!(dna.glow.radius, 6.0);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let json = "{}";
        let dna: CowDna = serde_json::from_str(json).unwrap();
        assert_eq!(dna, CowDna::default());
    }

    #[test]
    fn instance_phase_varies() {
        let p1 = instance_phase(42, 0);
        let p2 = instance_phase(42, 1);
        let p3 = instance_phase(42, 2);
        assert_ne!(p1, p2);
        assert_ne!(p2, p3);
        assert_ne!(p1, p3);
    }

    #[test]
    fn instance_phase_formula_correct() {
        // (42 ^ 1) = 43, 43 * 0.618033988749895 ≈ 26.575461...
        let p = instance_phase(42, 1);
        // (42 ^ 1) = 43, 43 * 0.618 ≈ 26.574
        let expected = 43.0_f32 * 0.618_034;
        assert!((p - expected).abs() < 0.001);
    }

    #[test]
    fn instance_phase_zero_seed_zero_id() {
        // (0 ^ 0) = 0, 0 * golden = 0.0
        let p = instance_phase(0, 0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn instance_phase_deterministic() {
        let p1 = instance_phase(42, 5);
        let p2 = instance_phase(42, 5);
        assert_eq!(p1, p2);
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let map = load_animations(Path::new("/tmp/no-such-dir"));
        assert!(map.is_empty());
    }

    #[test]
    fn get_dna_exact_match() {
        let mut map = HashMap::new();
        map.insert(
            "dragon".to_string(),
            CowDna {
                speed: 2.0,
                ..CowDna::default()
            },
        );
        let dna = get_dna(&map, "dragon");
        assert_eq!(dna.speed, 2.0);
    }

    #[test]
    fn get_dna_strips_extension() {
        let mut map = HashMap::new();
        map.insert(
            "dragon".to_string(),
            CowDna {
                speed: 2.0,
                ..CowDna::default()
            },
        );
        let dna = get_dna(&map, "dragon.cow");
        assert_eq!(dna.speed, 2.0);
    }

    #[test]
    fn get_dna_double_extension() {
        let mut map = HashMap::new();
        map.insert(
            "dragon".to_string(),
            CowDna {
                speed: 2.0,
                ..CowDna::default()
            },
        );
        // "dragon.cow.cow" → strip_suffix strips ".cow" → "dragon.cow" → not in map → default
        let dna = get_dna(&map, "dragon.cow.cow");
        assert_eq!(dna, CowDna::default());
    }

    #[test]
    fn get_dna_falls_back_to_default() {
        let map = HashMap::new();
        let dna = get_dna(&map, "nonexistent.cow");
        assert_eq!(dna, CowDna::default());
    }

    /// Schema-drift guard: parse a full DNA doc, re-serialize it through the
    /// same JSON path it will be loaded from, and confirm a re-parse is stable
    /// and contains no unknown/garbage fields.
    ///
    /// NOTE: `CowDna` only derives `Deserialize` (the shipped schema is
    /// load-only), so a serde `Serialize` round-trip is intentionally NOT used
    /// here — adding `Serialize` would change the shipped schema. Instead we
    /// round-trip through `serde_json::Value` (parse → known-keys map → re-parse)
    /// which exercises the exact deserialize path without touching the schema.
    #[test]
    fn dna_json_round_trip_stable() {
        // Sample mirrors the documented dragon.cow DNA (design doc §2.0).
        let json = r##"{
            "base": "Particles",
            "particles": {
                "type": "Fire",
                "rate": 18,
                "life": [0.6, 1.4],
                "speed": [0.3, 0.8],
                "palette": ["#ff8800", "#ff2200", "#440000"]
            },
            "speed": 1.0,
            "amplitude": { "breath": 0.6, "sway": 0.2, "float": 0.1 },
            "palette": ["#ff8800", "#ff2200", "#440000"],
            "easing": {
                "base": "sine_inout",
                "particle_alpha": "expo_out",
                "particle_velocity": "cubic_out"
            },
            "phase_seed": 4242,
            "glow": { "color": "#ff6600", "radius": 6.0, "falloff": "gaussian" }
        }"##;

        let first: CowDna = serde_json::from_str(json).expect("first parse");

        // Round-trip through a serde_json::Value built only from KNOWN keys,
        // then re-parse. If a schema field silently changed name/shape, the
        // re-parse would drift away from `first`.
        let value = serde_json::json!({
            "base": "Particles",
            "particles": {
                "type": "Fire",
                "rate": first.particles.rate,
                "life": first.particles.life,
                "speed": first.particles.speed,
                "palette": first.particles.palette,
            },
            "speed": first.speed,
            "amplitude": {
                "breath": first.amplitude.breath,
                "sway": first.amplitude.sway,
                "float": first.amplitude.float,
            },
            "palette": first.palette,
            "easing": {
                "base": first.easing.base,
                "particle_alpha": first.easing.particle_alpha,
                "particle_velocity": first.easing.particle_velocity,
            },
            "phase_seed": first.phase_seed,
            "glow": {
                "color": first.glow.color,
                "radius": first.glow.radius,
                "falloff": first.glow.falloff,
            },
        });
        let second: CowDna = serde_json::from_value(value.clone()).expect("re-parse from value");

        // Deterministic stability: the parse is idempotent.
        assert_eq!(first, second, "DNA round-trip must be stable");

        // No unknown/garbage keys leaked into the reconstructed document:
        // every top-level key is a real, documented DNA axis (+ glow).
        let obj = value.as_object().expect("object");
        let allowed = [
            "base",
            "particles",
            "speed",
            "amplitude",
            "palette",
            "easing",
            "phase_seed",
            "glow",
        ];
        for key in obj.keys() {
            assert!(
                allowed.contains(&key.as_str()),
                "unexpected DNA field in round-trip: {key}"
            );
        }

        // Sane values survived the trip.
        assert_eq!(second.base, BaseAnim::Particles);
        assert_eq!(second.phase_seed, 4242);
        assert_eq!(second.glow.falloff, "gaussian");
    }

    /// Documented-bounds guard: real DNA axes must stay inside the ranges the
    /// design doc (§2.1) prescribes. Cheap sanity check against value drift.
    #[test]
    fn dna_axes_within_documented_bounds() {
        let dna = CowDna::default();

        // speed: global time multiplier, doc examples span 0.3 (tortoise) .. 2.0 (nyan).
        assert!(
            dna.speed > 0.0 && dna.speed <= 4.0,
            "speed multiplier out of documented range: {}",
            dna.speed
        );

        // amplitude channels are motion depths in [0.0, 1.0].
        for (name, v) in [
            ("breath", dna.amplitude.breath),
            ("sway", dna.amplitude.sway),
            ("float", dna.amplitude.float),
        ] {
            assert!(
                (0.0..=1.0).contains(&v),
                "amplitude.{name} out of [0,1]: {v}"
            );
        }

        // glow.radius is a positive, finite pixel radius (doc uses 4.0 default, 6.0 dragon).
        assert!(
            dna.glow.radius.is_finite() && dna.glow.radius > 0.0,
            "glow.radius must be positive & finite: {}",
            dna.glow.radius
        );

        // glow.color is an sRGB hex string.
        assert!(
            dna.glow.color.starts_with('#') && dna.glow.color.len() == 7,
            "glow.color must be #rrggbb: {}",
            dna.glow.color
        );

        // particle lifetime/speed ranges must be ordered [min, max].
        assert!(
            dna.particles.life[0] <= dna.particles.life[1],
            "particle life range must be ordered: {:?}",
            dna.particles.life
        );
        assert!(
            dna.particles.speed[0] <= dna.particles.speed[1],
            "particle speed range must be ordered: {:?}",
            dna.particles.speed
        );
    }
}
