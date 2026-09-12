//! Effect system — trait + 10 base animation types.
//!
//! Each effect owns its state, updates per-frame, and renders into the
//! framebuffer. The render loop calls `effect.update(dt)` then
//! `effect.render(fb)` each frame.

use crate::color::{self, gaussian_glow, lerp_palette, parse_hex};
use crate::dna::{instance_phase, Amplitude, BaseAnim, CowDna};
use crate::easing;
use crate::framebuffer::{Cell, Color, FrameBuffer};
use crate::particles::{seed_frame_rng, spawn_for_type, ParticlePool};

/// Trait for all animation effects.
pub trait Effect: Send + Sync {
    /// Advance the effect by `dt` seconds.
    fn update(&mut self, dt: f32, cols: usize, rows: usize);

    /// Render the current frame into `fb`.
    fn render(&self, fb: &mut FrameBuffer, time: f32);

    /// Returns true when a one-shot effect is finished.
    fn is_done(&self) -> bool {
        false
    }

    /// Notify of terminal resize.
    fn on_resize(&mut self, _cols: usize, _rows: usize) {}
}

/// Dynamically find where the cow art begins in a potentially composed scene.
/// If a speech or thought bubble precedes the cow, returns the line index of the
/// first line of cow art. If no bubble is present, returns 0.
pub fn find_cow_start_line(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return 0;
    }
    let first = lines[0].trim();
    // A speech or thought bubble starts with a top border of underscores, hyphens, or equals
    if !(first.chars().all(|c| c == '_' || c == '-' || c == '=') && first.len() >= 3) {
        return 0; // No bubble at top
    }
    // Find bottom border: starts and ends with '|' or '(' and contains border chars
    let mut bottom_border_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        let trimmed = line.trim();
        if (trimmed.starts_with('|')
            && trimmed.ends_with('|')
            && trimmed.chars().all(|c| c == '|' || c == '_' || c == '-'))
            || (trimmed.starts_with('(')
                && trimmed.ends_with(')')
                && trimmed
                    .chars()
                    .all(|c| c == '(' || c == ')' || c == '_' || c == '-'))
        {
            bottom_border_idx = Some(i);
            break;
        }
    }
    let Some(b_idx) = bottom_border_idx else {
        return 0;
    };
    // After bottom border, skip connector lines (lines that contain only whitespace and 'o' or '\' or '/')
    let mut cow_start = b_idx + 1;
    while cow_start < lines.len() {
        let trimmed = lines[cow_start].trim();
        if trimmed.is_empty()
            || trimmed == "o"
            || trimmed == "\\"
            || trimmed == "/"
            || trimmed == "o o"
            || trimmed == "\\ \\"
        {
            cow_start += 1;
        } else {
            break;
        }
    }
    cow_start
}

#[inline]
pub(crate) fn is_eye_glyph(ch: char) -> bool {
    matches!(
        ch,
        'o' | 'O' | '@' | '^' | '*' | '$' | 'x' | 'X' | '=' | '0' | 'e' | '+' | 'v' | 'u' | 'w' | '8' | 'Q' | '•' | '●'
    )
}

/// Dynamically find the bottom-most non-empty line of the cow/animal art.
/// This corresponds to the row where the creature's feet / legs touch the ground.
pub fn find_cow_foot_y(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return 0;
    }
    for (i, line) in lines.iter().enumerate().rev() {
        if line.chars().any(|c| !c.is_whitespace()) {
            return i;
        }
    }
    lines.len().saturating_sub(1)
}

/// Natural Wildlife Instinct Archetype for creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimalInstinct {
    Dragon,
    Marine,
    Cephalopod,
    Canine,
    Feline,
    Avian,
    Amphibian,
    Bovine,
    Spectral,
}

/// Detect the natural wildlife instinct of an animal mascot based on DNA and art patterns.
pub fn detect_animal_instinct(cow_text: &str, dna: &CowDna) -> AnimalInstinct {
    if dna.particles.rate > 0 && dna.particles.r#type == crate::dna::ParticleType::Fire {
        return AnimalInstinct::Dragon;
    }
    if dna.particles.rate > 0 && dna.particles.r#type == crate::dna::ParticleType::Bubbles {
        return AnimalInstinct::Marine;
    }
    let lower = cow_text.to_ascii_lowercase();
    // Dragon / Inferno / Mythic
    if lower.contains("dragon")
        || lower.contains("charizard")
        || lower.contains("mooghidjirah")
        || lower.contains("moojira")
        || lower.contains("flaming")
        || lower.contains("daemon")
        || lower.contains("satanic")
        || lower.contains("minotaur")
        || lower.contains("sauron")
    {
        return AnimalInstinct::Dragon;
    }
    // Cephalopod / Deep Sea
    if lower.contains("octopus")
        || lower.contains("squid")
        || lower.contains("jellyfish")
        || lower.contains("cthulhu")
    {
        return AnimalInstinct::Cephalopod;
    }
    // Marine / Ocean
    if lower.contains("whale")
        || lower.contains("dolphin")
        || lower.contains("seahorse")
        || lower.contains("turtle")
        || lower.contains("pufferfish")
        || lower.contains("walrus")
        || lower.contains("ebi_furai")
        || lower.contains("shark")
    {
        return AnimalInstinct::Marine;
    }
    // Canine / Dog
    if lower.contains("doge")
        || lower.contains("snoopy")
        || lower.contains("corgi")
        || lower.contains("wolf")
    {
        return AnimalInstinct::Canine;
    }
    // Feline / Cat
    if lower.contains("cat")
        || lower.contains("kitten")
        || lower.contains("kitty")
        || lower.contains("meow")
        || lower.contains("hellokitty")
        || lower.contains("tiger")
        || lower.contains("panther")
        || lower.contains("vulpix")
        || lower.contains("fox")
    {
        return AnimalInstinct::Feline;
    }
    // Avian / Bird
    if lower.contains("eagle")
        || lower.contains("tweety")
        || lower.contains("owl")
        || lower.contains("turkey")
        || lower.contains("duck")
        || lower.contains("rooster")
        || lower.contains("pterodactyl")
        || lower.contains("bird")
        || lower.contains("bees")
    {
        return AnimalInstinct::Avian;
    }
    // Amphibian / Reptilian
    if lower.contains("frog")
        || lower.contains("bud-frogs")
        || lower.contains("viper")
        || lower.contains("tortoise")
        || lower.contains("armadillo")
        || lower.contains("stegosaurus")
        || lower.contains("snake")
    {
        return AnimalInstinct::Amphibian;
    }
    // Spectral / Cosmic / Arcane
    if lower.contains("ghost")
        || lower.contains("skeleton")
        || lower.contains("weeping-angel")
        || lower.contains("wizard")
        || lower.contains("nyan")
        || lower.contains("glados")
        || lower.contains("hypno")
        || lower.contains("kosh")
        || lower.contains("atat")
        || lower.contains("mech")
        || lower.contains("surgery")
        || lower.contains("eyes")
    {
        return AnimalInstinct::Spectral;
    }
    // Default fallback: Bovine & pastoral herbivores
    AnimalInstinct::Bovine
}

// ── Static (no animation) ──────────────────────────────────────────

/// The Phase 0 static cow with keep-alive micro-animations (Phase 8.1).
/// Firmly anchored at stagnant position (0, 0) with periodic eye-blink
/// on the animal so speech bubbles never shift.
#[derive(Debug)]
pub struct StaticEffect {
    cow_text: String,
    blink_text: String,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
}

impl StaticEffect {
    pub fn new(cow_text: String, color_mode: String) -> Self {
        Self::with_palette(cow_text, color_mode, Vec::new())
    }

    pub fn with_palette(
        cow_text: String,
        color_mode: String,
        palette: Vec<(u8, u8, u8)>,
    ) -> Self {
        let cow_start = find_cow_start_line(&cow_text);
        let landmarks = crate::cow::detect_cow_eyes(&cow_text, cow_start);
        let mut blink_lines: Vec<String> = cow_text.lines().map(|s| s.to_string()).collect();
        if landmarks.is_empty() {
            // Fallback: replace common consecutive/spaced pairs
            for (i, line) in blink_lines.iter_mut().enumerate() {
                if i >= cow_start {
                    *line = line
                        .replace("oo", "--")
                        .replace("OO", "--")
                        .replace("xx", "--")
                        .replace("XX", "--")
                        .replace("@@", "--")
                        .replace("$$", "--")
                        .replace("00", "--")
                        .replace("^^", "--")
                        .replace("**", "--")
                        .replace("==", "--")
                        .replace("o o", "- -")
                        .replace("O O", "- -")
                        .replace("o  o", "-  -")
                        .replace("O  O", "-  -")
                        .replace("o_o", "-_-")
                        .replace("O_O", "-_-")
                        .replace("o.o", "-.-")
                        .replace("O.O", "-.-")
                        .replace("@_@", "-_-")
                        .replace("@  @", "-  -")
                        .replace("^ ^", "- -")
                        .replace("* *", "- -");
                }
            }
        } else {
            for &(r, c) in &landmarks {
                if let Some(line) = blink_lines.get_mut(r) {
                    let mut chars: Vec<char> = line.chars().collect();
                    if c < chars.len() {
                        chars[c] = '-';
                        *line = chars.into_iter().collect();
                    }
                }
            }
        }
        let blink_text = blink_lines.join("\n");
        Self {
            cow_text,
            blink_text,
            color_mode,
            palette,
        }
    }
}

impl Effect for StaticEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        // Periodic eye-blink (~every 4.5s, lasts 0.15s).
        let blink_cycle = time % 4.5;
        let is_blinking = blink_cycle > 4.35;

        let display_text = if is_blinking {
            &self.blink_text
        } else {
            &self.cow_text
        };

        // Strictly anchored at stagnant position (0, 0)
        render_text_offset_palette(
            fb,
            display_text,
            Color::WHITE,
            0,
            0,
            &self.color_mode,
            &self.palette,
            time,
        );
    }
}

// ── Breathe ────────────────────────────────────────────────────────

/// Subtle chest/belly expansion and contraction with anchored baseline at (0, 0).
/// The thought/speech bubble stays completely stationary while the creature's
/// flanks and eyes breathe organically in-place.
#[derive(Debug)]
pub struct BreatheEffect {
    cow_text: String,
    line_offsets: Vec<usize>,
    cow_start_line: usize,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    instinct: AnimalInstinct,
    mouth_pos: Option<(usize, usize)>,
    eye_landmarks: Vec<(usize, usize)>,
}

impl BreatheEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let line_offsets = compute_line_offsets(&cow_text);
        let cow_start_line = find_cow_start_line(&cow_text);
        let palette = crate::color::parse_palette(&dna.palette);
        let mut amp = dna.amplitude.clone();
        if amp.breath <= 0.05 {
            amp.breath = 0.25;
        }

        let instinct = detect_animal_instinct(&cow_text, dna);
        let eye_landmarks = crate::cow::detect_cow_eyes(&cow_text, cow_start_line);

        // Find mouth/snout position for breathing fire/bubbles/particles
        let mut mouth_pos = None;
        let lines: Vec<&str> = cow_text.lines().collect();
        for (i, line) in lines.iter().enumerate().skip(cow_start_line) {
            if mouth_pos.is_some() {
                break;
            }
            if let Some(open) = line.find("(__)") {
                mouth_pos = Some((i, open + 1));
            } else if let Some(idx) = line.find("\\@") {
                mouth_pos = Some((i, idx));
            }
        }
        if mouth_pos.is_none() {
            if let Some(&(er, ec)) = eye_landmarks.first() {
                mouth_pos = Some((er + 1, ec));
            }
        }
        if mouth_pos.is_none() && lines.len() > cow_start_line {
            // Fallback: use first non-space char of top creature row
            for (i, line) in lines.iter().enumerate().skip(cow_start_line) {
                if let Some(pos) = line.find(|c: char| !c.is_whitespace()) {
                    mouth_pos = Some((i, pos));
                    break;
                }
            }
        }

        Self {
            cow_text,
            line_offsets,
            cow_start_line,
            amp,
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            instinct,
            mouth_pos,
            eye_landmarks,
        }
    }
}

impl Effect for BreatheEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let is_inhale = eased > 0.4;

        // Eye-blink cycle (~every 3.8s)
        let blink_cycle = (time * 0.26 + self.phase) % 1.0;
        let is_blinking = blink_cycle < 0.04;

        let mut y = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if y >= fb.height {
                return;
            }
            let hull = find_line_hull(line);

            // Speech/thought bubble lines: strictly anchored at (0, 0), completely untouched
            if y < self.cow_start_line {
                for (x, ch) in line.chars().enumerate() {
                    if x < fb.width {
                        let cell_fg = resolve_bubble_cell_fg(ch);
                        draw_char_with_hull(fb, x, y, x, hull, ch, cell_fg);
                    }
                }
                y += 1;
                return;
            }

            // Creature lines: subtle chest/flank breathing expansion in place (zero-allocation)
            let has_under_expansion = line.contains("___") || line.contains("__");
            let has_dash_expansion = line.contains("---") || line.contains("--");
            let mut chars_iter = line.chars().peekable();
            let mut x = 0usize;
            while let Some(mut ch) = chars_iter.next() {
                if x >= fb.width {
                    break;
                }

                // Blink eyes: works across all detected eye landmarks (single, separated, or pairs)
                let is_landmark = self.eye_landmarks.contains(&(y, x));
                if is_blinking && (is_landmark || (is_eye_glyph(ch) && chars_iter.peek().copied() == Some(ch))) {
                    let rel_y = y.saturating_sub(self.cow_start_line);
                    let cell_fg = resolve_fg_palette_char(
                        &self.color_mode,
                        &self.palette,
                        x,
                        rel_y,
                        time,
                        Color::WHITE,
                        '-',
                    );
                    draw_char_with_hull(fb, x, y, x, hull, '-', cell_fg);
                    if !is_landmark && chars_iter.peek().copied() == Some(ch) {
                        let _ = chars_iter.next();
                        if x + 1 < fb.width {
                            draw_char_with_hull(fb, x + 1, y, x + 1, hull, '-', cell_fg);
                        }
                        x += 2;
                        continue;
                    }
                    x += 1;
                    continue;
                }

                // Inhale breathing wave on torso/flank lines:
                if is_inhale && self.amp.breath > 0.05 {
                    if ch == '_' && has_under_expansion {
                        ch = '~';
                    } else if ch == '-' && has_dash_expansion {
                        ch = '=';
                    }
                }

                let rel_y = y.saturating_sub(self.cow_start_line);
                let mut cell_fg = resolve_fg_palette_char(
                    &self.color_mode,
                    &self.palette,
                    x,
                    rel_y,
                    time,
                    Color::WHITE,
                    ch,
                );

                // Natural Instinct signature highlights:
                match self.instinct {
                    AnimalInstinct::Dragon => {
                        // Dragon chest glows with inner magma during exhale
                        if !is_inhale && (ch == '~' || ch == '=' || ch == '#' || ch == '@') {
                            cell_fg = Color::rgb(255, 68, 0);
                        }
                    }
                    AnimalInstinct::Cephalopod => {
                        // Cephalopod mantle bioluminescence wave
                        if is_inhale && (ch == '(' || ch == ')' || ch == '{' || ch == '}') {
                            cell_fg = Color::rgb(0, 255, 204);
                        }
                    }
                    AnimalInstinct::Feline => {
                        // Cat whisker subtle twitch
                        if ch == '=' && ((time * 3.0) as usize) % 2 == 0 {
                            ch = '-';
                        }
                    }
                    AnimalInstinct::Amphibian if is_inhale && ch == '(' && line.contains("()") => {
                        // Gular sac swelling during inhale
                        ch = '[';
                    }
                    _ => {}
                }

                draw_char_with_hull(fb, x, y, x, hull, ch, cell_fg);
                x += 1;
            }
            y += 1;
        });

        // Natural Wildlife Instinct signature emitters:
        match self.instinct {
            AnimalInstinct::Dragon => {
                if let Some((my, mx)) = self.mouth_pos {
                    if !is_inhale && my < fb.height {
                        // Dragon is exhaling: BREATHE FIRE!
                        let flame_reach = (((eased - 0.4) / 0.6) * 8.0) as usize;
                        let fire_glyphs = ['*', '^', '~', '§', '»', '>', '#'];
                        let fire_colors = [
                            Color::rgb(255, 34, 0),    // blazing vermilion
                            Color::rgb(255, 102, 0),   // fiery orange
                            Color::rgb(255, 204, 0),   // golden ember
                            Color::rgb(255, 240, 150), // white-hot flame core
                        ];
                        let streams_left = mx > 5;
                        for k in 1..=flame_reach {
                            let fx = if streams_left {
                                mx.saturating_sub(k)
                            } else {
                                mx + k
                            };
                            if fx < fb.width {
                                let g_idx = (k + (time * 12.0) as usize) % fire_glyphs.len();
                                let c_idx = (k + (time * 8.0) as usize) % fire_colors.len();
                                let _ = fb.set(
                                    fx,
                                    my,
                                    Cell::new(fire_glyphs[g_idx], fire_colors[c_idx]),
                                );
                                // Flame flickering plume above/below
                                if k > 2 && (k % 2 == 0) && my > 0 {
                                    let plume_y = if k % 4 == 0 {
                                        my.saturating_sub(1)
                                    } else {
                                        (my + 1).min(fb.height.saturating_sub(1))
                                    };
                                    let _ = fb.set(
                                        fx,
                                        plume_y,
                                        Cell::new(
                                            '~',
                                            fire_colors[(c_idx + 1) % fire_colors.len()],
                                        ),
                                    );
                                }
                            }
                        }
                    } else if is_inhale && my < fb.height {
                        // Small smoke/ember wisps drifting up
                        let ember_x = if mx > 2 { mx - 1 } else { mx + 1 };
                        if ember_x < fb.width && my > 0 {
                            let _ =
                                fb.set(ember_x, my - 1, Cell::new('.', Color::rgb(255, 170, 0)));
                        }
                    }
                }
            }
            AnimalInstinct::Marine => {
                // Marine life: whale blowhole vertical water spout or rising bubbles
                if let Some((my, mx)) = self.mouth_pos {
                    if !is_inhale && my > 1 {
                        // Exhale: water spout shoots straight up from blowhole
                        let spout_reach = (((eased - 0.4) / 0.6) * 4.0) as usize + 1;
                        let spout_glyphs = ['|', '~', '^', '*', 'o'];
                        for k in 1..=spout_reach {
                            let sy = my.saturating_sub(k);
                            if sy < fb.height {
                                let ch =
                                    spout_glyphs[(k + (time * 6.0) as usize) % spout_glyphs.len()];
                                let _ = fb.set(mx, sy, Cell::new(ch, Color::rgb(0, 229, 255)));
                                // Spray droplets spreading at crest
                                if k == spout_reach {
                                    if mx > 0 {
                                        let _ = fb.set(
                                            mx - 1,
                                            sy,
                                            Cell::new('~', Color::rgb(128, 216, 255)),
                                        );
                                    }
                                    if mx + 1 < fb.width {
                                        let _ = fb.set(
                                            mx + 1,
                                            sy,
                                            Cell::new('~', Color::rgb(128, 216, 255)),
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        // Inhale: gentle bubbles rising
                        let bubble_y = my.saturating_sub(((time * 3.0) as usize) % 3 + 1);
                        let bubble_x =
                            (mx + ((time * 2.0) as usize) % 2).min(fb.width.saturating_sub(1));
                        if bubble_y < fb.height {
                            let b_ch = if ((time * 3.0) as usize) % 2 == 0 {
                                'o'
                            } else {
                                '°'
                            };
                            let _ = fb.set(
                                bubble_x,
                                bubble_y,
                                Cell::new(b_ch, Color::rgb(0, 220, 255)),
                            );
                        }
                    }
                }
            }
            AnimalInstinct::Cephalopod => {
                // Bioluminescent spore sparkles floating around the mantle
                if let Some((my, mx)) = self.mouth_pos {
                    let spore_y = my.saturating_sub(((time * 2.5) as usize) % 3);
                    let spore_x = (mx + ((time * 4.0) as usize) % 4)
                        .saturating_sub(2)
                        .min(fb.width.saturating_sub(1));
                    if spore_y < fb.height {
                        let _ = fb.set(spore_x, spore_y, Cell::new('*', Color::rgb(0, 255, 204)));
                    }
                }
            }
            AnimalInstinct::Canine => {
                // Sleeping snoopy/doge emits floating Zzz sleep motes; active canine emits panting puff
                if let Some((my, mx)) = self.mouth_pos {
                    let is_sleeping =
                        self.cow_text.contains("snoopysleep") || self.cow_text.contains("--");
                    if is_sleeping && my > 1 {
                        let z_step = ((time * 2.0) as usize) % 3;
                        let zy = my.saturating_sub(z_step + 1);
                        let zx = (mx + z_step).min(fb.width.saturating_sub(1));
                        if zy < fb.height {
                            let z_ch = if z_step == 2 { 'Z' } else { 'z' };
                            let _ = fb.set(zx, zy, Cell::new(z_ch, Color::rgb(149, 117, 205)));
                        }
                    } else if !is_inhale && my < fb.height {
                        let px = if mx > 1 { mx - 1 } else { mx + 1 };
                        if px < fb.width {
                            let _ = fb.set(px, my, Cell::new('.', Color::rgb(255, 183, 77)));
                        }
                    }
                }
            }
            AnimalInstinct::Avian => {
                // Feather motes drifting softly
                if let Some((my, mx)) = self.mouth_pos {
                    let fy = (my + ((time * 1.5) as usize) % 3).min(fb.height.saturating_sub(1));
                    let fx = if mx > 2 { mx - 2 } else { mx + 2 };
                    if fx < fb.width {
                        let _ = fb.set(fx, fy, Cell::new(',', Color::rgb(255, 213, 79)));
                    }
                }
            }
            AnimalInstinct::Spectral => {
                // Arcane mystic runes floating and orbiting
                if let Some((my, mx)) = self.mouth_pos {
                    let runes = ['✦', '*', '°', '·', '✧'];
                    for (i, &rune) in runes.iter().enumerate() {
                        let angle = (time * 2.0 + i as f32 * 1.25) % std::f32::consts::TAU;
                        let rx = (mx as f32 + angle.cos() * 3.5).max(0.0) as usize;
                        let ry = (my as f32 + angle.sin() * 1.8).max(0.0) as usize;
                        if rx < fb.width && ry < fb.height {
                            let col = if i % 2 == 0 {
                                Color::rgb(128, 255, 219)
                            } else {
                                Color::rgb(189, 147, 249)
                            };
                            let _ = fb.set(rx, ry, Cell::new(rune, col));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// ── Float / Bob ────────────────────────────────────────────────────

/// Whole-art hovering floating effect with anchored position at (0, 0).
/// The thought/speech bubble stays completely stationary while the creature
/// hovers with a subtle levitation shimmer in place.
#[derive(Debug)]
pub struct FloatEffect {
    cow_text: String,
    line_offsets: Vec<usize>,
    cow_start_line: usize,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    instinct: AnimalInstinct,
    pub body: crate::kinematics::KinematicBody,
    elapsed: f32,
    eye_landmarks: Vec<(usize, usize)>,
}

impl FloatEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let line_offsets = compute_line_offsets(&cow_text);
        let cow_start_line = find_cow_start_line(&cow_text);
        let eye_landmarks = crate::cow::detect_cow_eyes(&cow_text, cow_start_line);
        let (w, h) = crate::kinematics::ascii_dimensions(&cow_text);
        let mut body = crate::kinematics::KinematicBody::new(
            w,
            h,
            crate::kinematics::BoundsMode::Lissajous {
                amp_x: 0.0,
                amp_y: 0.0,
                freq_x: 0.0,
                freq_y: 0.0,
                phase_x: phase,
                phase_y: phase,
            },
        );
        body.vx = 0.0;
        body.vy = 0.0;
        let instinct = detect_animal_instinct(&cow_text, dna);
        let palette = crate::color::parse_palette(&dna.palette);
        let mut amp = dna.amplitude.clone();
        if amp.float <= 0.05 {
            amp.float = 0.3;
        }
        Self {
            cow_text,
            line_offsets,
            cow_start_line,
            amp,
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            instinct,
            body,
            elapsed: 0.0,
            eye_landmarks,
        }
    }
}

impl Effect for FloatEffect {
    fn update(&mut self, dt: f32, cols: usize, rows: usize) {
        self.elapsed += dt;
        self.body.update(dt, self.elapsed, cols, rows);
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let hover_intensity = (self.easing_fn)(t);
        let hover_pulse = hover_intensity > 0.5;

        // Periodic eye-blink:
        let blink_cycle = (time * 0.24 + self.phase) % 1.0;
        let is_blinking = blink_cycle < 0.04;

        let mut y = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if y >= fb.height {
                return;
            }

            let hull = find_line_hull(line);

            // Speech/thought bubble: strictly anchored at (0, 0), zero shift
            if y < self.cow_start_line {
                for (x, ch) in line.chars().enumerate() {
                    if x < fb.width {
                        let cell_fg = resolve_bubble_cell_fg(ch);
                        draw_char_with_hull(fb, x, y, x, hull, ch, cell_fg);
                    }
                }
                y += 1;
                return;
            }

            // Creature lines: anchored at stagnant position (y)
            let draw_y = y;

            let mut chars_iter = line.chars().peekable();
            let mut x = 0usize;
            while let Some(mut ch) = chars_iter.next() {
                let xi = x as i32;
                if xi >= 0 && (xi as usize) < fb.width {
                    let uxi = xi as usize;

                    // Periodic eye-blink across all eye landmarks:
                    let is_landmark = self.eye_landmarks.contains(&(draw_y, uxi));
                    let rel_y = draw_y.saturating_sub(self.cow_start_line);
                    if is_blinking && (is_landmark || (is_eye_glyph(ch) && chars_iter.peek().copied() == Some(ch))) {
                        let cell_fg = resolve_fg_palette_char(
                            &self.color_mode,
                            &self.palette,
                            x,
                            rel_y,
                            time,
                            Color::WHITE,
                            '-',
                        );
                        draw_char_with_hull(fb, uxi, draw_y, x, hull, '-', cell_fg);
                        if !is_landmark && chars_iter.peek().copied() == Some(ch) {
                            let _ = chars_iter.next();
                            if uxi + 1 < fb.width {
                                draw_char_with_hull(fb, uxi + 1, draw_y, x + 1, hull, '-', cell_fg);
                            }
                            x += 2;
                            continue;
                        }
                        x += 1;
                        continue;
                    }

                    // Hovering levitation shimmer on horns/ears or back:
                    if hover_pulse && (self.amp.float > 0.0 || self.amp.sway > 0.0) {
                        if ch == '^' {
                            ch = '*';
                        } else if ch == '~' {
                            ch = '-';
                        }
                    }

                    let cell_fg = resolve_fg_palette_char(
                        &self.color_mode,
                        &self.palette,
                        x,
                        rel_y,
                        time,
                        Color::WHITE,
                        ch,
                    );
                    draw_char_with_hull(fb, uxi, draw_y, x, hull, ch, cell_fg);
                }
                x += 1;
            }
            y += 1;
        });

        // Natural Wildlife Instinct floating particles:
        match self.instinct {
            AnimalInstinct::Marine => {
                // Buoyant bubbles rising through water
                for i in 0..4 {
                    let seed = i * 19;
                    let bx = (seed + ((time * 3.0) as usize)) % fb.width.max(1);
                    let by = ((fb.height.saturating_sub(1)) as f32
                        - ((time * 4.0 + i as f32 * 2.5) % fb.height.max(1) as f32))
                        .max(0.0) as usize;
                    if bx < fb.width && by < fb.height {
                        let b_ch = if i % 2 == 0 { 'o' } else { '°' };
                        let _ = fb.set(bx, by, Cell::new(b_ch, Color::rgb(0, 229, 255)));
                    }
                }
            }
            AnimalInstinct::Cephalopod => {
                // Bioluminescent spore sparkles
                for i in 0..4 {
                    let seed = i * 23;
                    let sx = (seed + ((time * 2.0) as usize)) % fb.width.max(1);
                    let sy = ((fb.height.saturating_sub(1)) as f32
                        - ((time * 3.0 + i as f32 * 3.0) % fb.height.max(1) as f32))
                        .max(0.0) as usize;
                    if sx < fb.width && sy < fb.height {
                        let col = if i % 2 == 0 {
                            Color::rgb(0, 255, 204)
                        } else {
                            Color::rgb(204, 102, 255)
                        };
                        let _ = fb.set(sx, sy, Cell::new('*', col));
                    }
                }
            }
            AnimalInstinct::Dragon => {
                // Floating on hot magma updrafts: rising sparks
                for i in 0..4 {
                    let seed = i * 17;
                    let fx = (seed + ((time * 4.0) as usize)) % fb.width.max(1);
                    let fy = ((fb.height.saturating_sub(1)) as f32
                        - ((time * 5.0 + i as f32 * 2.0) % fb.height.max(1) as f32))
                        .max(0.0) as usize;
                    if fx < fb.width && fy < fb.height {
                        let _ = fb.set(fx, fy, Cell::new('*', Color::rgb(255, 68, 0)));
                    }
                }
            }
            AnimalInstinct::Spectral => {
                // Levitating arcane runes
                let runes = ['✦', '*', '°', '·'];
                for (i, &rune) in runes.iter().enumerate() {
                    let angle = (time * 1.8 + i as f32 * 1.57) % std::f32::consts::TAU;
                    let rx = ((fb.width / 2) as f32 + angle.cos() * 8.0).max(0.0) as usize;
                    let ry = ((fb.height / 2) as f32 + angle.sin() * 3.5).max(0.0) as usize;
                    if rx < fb.width && ry < fb.height {
                        let _ = fb.set(rx, ry, Cell::new(rune, Color::rgb(128, 255, 219)));
                    }
                }
            }
            AnimalInstinct::Canine => {
                // Soft floating dream sleep clouds
                for i in 0..3 {
                    let seed = i * 21;
                    let cx = (seed + ((time * 2.0) as usize)) % fb.width.max(1);
                    let cy = ((fb.height.saturating_sub(1)) as f32
                        - ((time * 2.0 + i as f32 * 2.0) % fb.height.max(1) as f32))
                        .max(0.0) as usize;
                    if cx < fb.width && cy < fb.height {
                        let _ = fb.set(cx, cy, Cell::new('~', Color::rgb(209, 196, 233)));
                    }
                }
            }
            AnimalInstinct::Avian => {
                // Rising thermal updrafts
                for i in 0..3 {
                    let seed = i * 27;
                    let ax = (seed + ((time * 3.0) as usize)) % fb.width.max(1);
                    let ay = ((fb.height.saturating_sub(1)) as f32
                        - ((time * 4.0 + i as f32 * 3.0) % fb.height.max(1) as f32))
                        .max(0.0) as usize;
                    if ax < fb.width && ay < fb.height {
                        let _ = fb.set(ax, ay, Cell::new('^', Color::rgb(255, 238, 88)));
                    }
                }
            }
            AnimalInstinct::Bovine => {
                // Meadow dandelion spores drifting
                for i in 0..3 {
                    let seed = i * 31;
                    let bx = (seed + ((time * 1.5) as usize)) % fb.width.max(1);
                    let by = ((fb.height.saturating_sub(1)) as f32
                        - ((time * 2.5 + i as f32 * 2.0) % fb.height.max(1) as f32))
                        .max(0.0) as usize;
                    if bx < fb.width && by < fb.height {
                        let _ = fb.set(bx, by, Cell::new('*', Color::rgb(200, 230, 201)));
                    }
                }
            }
            _ => {}
        }
    }
}

// ── Walk / Trot ────────────────────────────────────────────────────

/// Bottom-row leg character swap with dynamic pillar detection, whitespace preservation,
/// and physical kinematic traversal across the terminal pasture.
#[derive(Debug)]
pub struct WalkEffect {
    cow_text: String,
    line_offsets: Vec<usize>,
    leg_line_idx: usize,
    leg_cols: Vec<usize>,
    eye_pos: Option<(usize, usize)>,
    mouth_pos: Option<(usize, usize)>,
    tail_pos: Option<(usize, usize)>,
    _amp: Amplitude,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    instinct: AnimalInstinct,
    pub body: crate::kinematics::KinematicBody,
    elapsed: f32,
    eye_landmarks: Vec<(usize, usize)>,
}

impl WalkEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let line_offsets = compute_line_offsets(&cow_text);
        let lines: Vec<&str> = cow_text.lines().collect();

        // Dynamically find the bottom-most non-empty line of the ASCII creature
        let mut leg_line_idx = line_offsets.len().saturating_sub(1);
        for (i, line) in lines.iter().enumerate().rev() {
            if line.chars().any(|c| !c.is_whitespace()) {
                leg_line_idx = i;
                break;
            }
        }

        // Detect columns containing structural leg/foot stroke characters
        let mut leg_cols = Vec::new();
        if leg_line_idx < lines.len() {
            let bottom_line = lines[leg_line_idx];
            for (col, ch) in bottom_line.chars().enumerate() {
                if matches!(
                    ch,
                    '|' | '/' | '\\' | '!' | 'I' | 'l' | '[' | ']' | '(' | ')' | '1' | ':'
                ) {
                    leg_cols.push(col);
                }
            }
        }

        // Anatomical cow landmarks detection:
        let cow_start = find_cow_start_line(&cow_text);
        let eye_landmarks = crate::cow::detect_cow_eyes(&cow_text, cow_start);
        let mut eye_pos: Option<(usize, usize)> = eye_landmarks.first().copied();
        let mut mouth_pos: Option<(usize, usize)> = None;
        let mut tail_pos: Option<(usize, usize)> = None;

        for (i, line) in lines.iter().enumerate().skip(cow_start) {
            let chars: Vec<char> = line.chars().collect();

            // 1. Detect eyes fallback if not found by detect_cow_eyes
            if eye_pos.is_none() && i < leg_line_idx {
                for (c_idx, &c) in chars.iter().enumerate() {
                    if (c == '(' || c == '[') && c_idx + 3 < chars.len() {
                        let c1 = chars[c_idx + 1];
                        let c2 = chars[c_idx + 2];
                        let close = chars[c_idx + 3];
                        if (close == ')' || close == ']') && is_eye_glyph(c1) && is_eye_glyph(c2) {
                            eye_pos = Some((i, c_idx + 1));
                            break;
                        }
                    }
                }
                if eye_pos.is_none() {
                    for col in 0..chars.len().saturating_sub(1) {
                        if is_eye_glyph(chars[col]) && chars[col] == chars[col + 1] {
                            eye_pos = Some((i, col));
                            break;
                        }
                    }
                }
            } else if mouth_pos.is_none() && eye_pos.is_some_and(|ep| i >= ep.0) {
                // 2. Detect mouth/muzzle: (__), (..), (==), \__/, or U / V tongue
                if let Some(open) = line.find("(__)") {
                    mouth_pos = Some((i, open + 1));
                } else {
                    for (c_idx, &c) in chars.iter().enumerate() {
                        if c == '(' && c_idx + 3 < chars.len() && chars[c_idx + 3] == ')' {
                            mouth_pos = Some((i, c_idx + 1));
                            break;
                        }
                    }
                }
                if mouth_pos.is_none() {
                    if let Some(open) = line.find("\\__/") {
                        mouth_pos = Some((i, open + 1));
                    } else if let Some(open) = line.find("U ").or_else(|| line.find("V ")) {
                        mouth_pos = Some((i, open));
                    }
                }
            }

            // 3. Detect tail: )\/ or )/\ or ~ or S near rear of animal
            if tail_pos.is_none() && i < leg_line_idx {
                if let Some(pos) = line.rfind(")\\") {
                    if !line[..pos].ends_with("(__") {
                        tail_pos = Some((i, pos));
                    }
                } else if let Some(pos) = line.rfind(")/") {
                    if !line[..pos].ends_with("(__") {
                        tail_pos = Some((i, pos));
                    }
                }
            }
        }

        let (w, h) = crate::kinematics::ascii_dimensions(&cow_text);
        let mut body =
            crate::kinematics::KinematicBody::new(w, h, crate::kinematics::BoundsMode::Wrap);
        body.vx = 0.0;
        body.vy = 0.0;
        let palette = crate::color::parse_palette(&dna.palette);
        let instinct = detect_animal_instinct(&cow_text, dna);

        Self {
            cow_text,
            line_offsets,
            leg_line_idx,
            leg_cols,
            eye_pos,
            mouth_pos,
            tail_pos,
            _amp: dna.amplitude.clone(),
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            instinct,
            body,
            elapsed: 0.0,
            eye_landmarks,
        }
    }
}

impl Effect for WalkEffect {
    fn update(&mut self, dt: f32, cols: usize, rows: usize) {
        self.elapsed += dt;
        self.body.update(dt, self.elapsed, cols, rows);
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let stride = (time * self.speed + self.phase) % 1.0;

        // 4-phase natural leg stride coupling using clean ASCII:
        let (leg_l, leg_r) = if stride < 0.25 || (0.50..0.75).contains(&stride) {
            ('|', '|')
        } else if stride < 0.50 {
            ('/', '\\')
        } else {
            ('\\', '/')
        };

        // Eye blink cycle: every ~3.5 seconds, blink for ~150ms
        let blink_cycle = (time * 0.28 + self.phase) % 1.0;
        let is_blinking = blink_cycle < 0.045 || (blink_cycle > 0.08 && blink_cycle < 0.11);

        // Tail swish cycle: every ~1.6 seconds, swishes back and forth
        let tail_cycle = (time * 0.62 + self.phase) % 1.0;
        let tail_frame = (tail_cycle * 4.0) as usize; // 0, 1, 2, 3

        // Cud chew cycle: natural bovine rhythm every ~2.8 seconds
        let chew_cycle = (time * 0.36 + self.phase) % 1.0;
        let is_chewing = chew_cycle > 0.40 && chew_cycle < 0.65;

        let x_off = self.body.screen_x();
        let y_off = self.body.screen_y();

        let mut y = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);

            // Bounding hull for occlusion masking (zero-allocation scan)
            let mut first_non_ws = None;
            let mut last_non_ws = None;
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    if first_non_ws.is_none() {
                        first_non_ws = Some(col);
                    }
                    last_non_ws = Some(col);
                }
            }
            let hull_start = first_non_ws;
            let hull_end = last_non_ws;

            let mut skip_until = 0usize;

            for (x, mut display_ch) in line.chars().enumerate() {
                if x < skip_until {
                    continue;
                }

                // 1. Leg stride animation on bottom foot line
                if y == self.leg_line_idx && self.leg_cols.contains(&x) {
                    display_ch = if x % 2 == 0 { leg_l } else { leg_r };
                }

                // 2. Eye blinking animation
                let is_eye = self.eye_landmarks.contains(&(y, x))
                    || self.eye_pos.is_some_and(|(eye_row, eye_col)| y == eye_row && (x == eye_col || x == eye_col + 1));
                if is_eye && is_blinking {
                    display_ch = '-';
                }

                // 3. Mouth / cud chewing animation
                if let Some((mouth_row, mouth_col)) = self.mouth_pos {
                    if y == mouth_row && is_chewing && (x == mouth_col || x == mouth_col + 1) {
                        display_ch = if chew_cycle > 0.52 { '=' } else { '.' };
                    }
                }

                // 4. Tail swishing animation
                if let Some((tail_row, tail_col)) = self.tail_pos {
                    if y == tail_row && x == tail_col {
                        let swish = match tail_frame {
                            1 => [')', '|', '/', '\\'],
                            2 => [')', '/', '\\', '/'],
                            3 => [')', '|', '\\', '/'],
                            _ => [')', '\\', '/', '\\'],
                        };
                        for (k, &sc) in swish.iter().enumerate() {
                            let xi = (x + k) as i32 + x_off;
                            let yi = y as i32 + y_off;
                            if yi >= 0 && xi >= 0 {
                                let uxi = xi as usize;
                                let uyi = yi as usize;
                                if uyi < fb.height && uxi < fb.width {
                                    let cell_fg = resolve_fg_palette_char(
                                        &self.color_mode,
                                        &self.palette,
                                        x + k,
                                        y,
                                        time,
                                        Color::WHITE,
                                        sc,
                                    );
                                    let _ = fb.set(uxi, uyi, Cell::new(sc, cell_fg));
                                }
                            }
                        }
                        skip_until = x + 4;
                        continue;
                    }
                }

                let xi = x as i32 + x_off;
                let yi = y as i32 + y_off;
                if yi >= 0 && xi >= 0 {
                    let uxi = xi as usize;
                    let uyi = yi as usize;
                    if uyi < fb.height && uxi < fb.width {
                        if display_ch != ' ' {
                            let cell_fg = resolve_fg_palette_char(
                                &self.color_mode,
                                &self.palette,
                                x,
                                y,
                                time,
                                Color::WHITE,
                                display_ch,
                            );
                            let _ = fb.set(uxi, uyi, Cell::new(display_ch, cell_fg));
                        } else if let (Some(hs), Some(he)) = (hull_start, hull_end) {
                            if x >= hs && x <= he {
                                // Opaque blank inside hull — occlude scenery
                                let _ = fb.set(uxi, uyi, Cell::new(' ', Color::WHITE));
                            }
                        }
                    }
                }
            }
            y += 1;
        });

        // Natural Wildlife Instinct footsteps:
        if self.leg_line_idx < fb.height {
            let foot_y = (self.leg_line_idx as i32 + y_off).max(0) as usize;
            match self.instinct {
                AnimalInstinct::Dragon => {
                    // Fiery magma footsteps
                    for &col in &self.leg_cols {
                        let foot_x = (col as i32 + x_off) as usize;
                        if foot_x < fb.width && foot_y < fb.height {
                            let ember_ch = if (time * 10.0) as usize % 2 == 0 {
                                '*'
                            } else {
                                '.'
                            };
                            let _ =
                                fb.set(foot_x, foot_y, Cell::new(ember_ch, Color::rgb(255, 68, 0)));
                        }
                    }
                }
                AnimalInstinct::Marine => {
                    // Aquatic swimming tail wake
                    if let Some((_ty, tx)) = self.tail_pos {
                        let foot_x = (tx as i32 + x_off) as usize;
                        if foot_x < fb.width && foot_y < fb.height {
                            let _ = fb.set(foot_x, foot_y, Cell::new('~', Color::rgb(0, 229, 255)));
                        }
                    }
                }
                AnimalInstinct::Spectral => {
                    // Ethereal phase trail
                    for &col in &self.leg_cols {
                        let foot_x = (col as i32 + x_off) as usize;
                        if foot_x < fb.width && foot_y < fb.height {
                            let _ =
                                fb.set(foot_x, foot_y, Cell::new('·', Color::rgb(128, 255, 219)));
                        }
                    }
                }
                AnimalInstinct::Canine => {
                    // Trotting dust motes behind feet
                    for &col in &self.leg_cols {
                        let foot_x = ((col as i32 + x_off).saturating_sub(1)) as usize;
                        if foot_x < fb.width && foot_y < fb.height {
                            let dust_ch = if (time * 8.0) as usize % 2 == 0 {
                                '.'
                            } else {
                                '°'
                            };
                            let _ = fb.set(
                                foot_x,
                                foot_y,
                                Cell::new(dust_ch, Color::rgb(215, 204, 200)),
                            );
                        }
                    }
                }
                AnimalInstinct::Avian => {
                    // Avian ground hopping claw marks
                    for &col in &self.leg_cols {
                        let foot_x = (col as i32 + x_off) as usize;
                        if foot_x < fb.width && foot_y < fb.height {
                            let claw_ch = if stride < 0.5 { '\'' } else { '`' };
                            let _ = fb.set(
                                foot_x,
                                foot_y,
                                Cell::new(claw_ch, Color::rgb(255, 213, 79)),
                            );
                        }
                    }
                }
                AnimalInstinct::Amphibian => {
                    // Reptilian/amphibian low crawl motes
                    for &col in &self.leg_cols {
                        let foot_x = (col as i32 + x_off) as usize;
                        if foot_x < fb.width && foot_y < fb.height {
                            let _ =
                                fb.set(foot_x, foot_y, Cell::new('~', Color::rgb(139, 195, 74)));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ── Particles (Fire/Bubbles/Stars/Zzz/Pulse/Glitch) ────────────────

/// Contextual particle emitter overlay.
#[derive(Debug)]
pub struct ParticlesEffect {
    cow_text: String,
    dna: CowDna,
    pool: ParticlePool,
    spawn_timer: f32,
    phase: f32,
    instance_id: u32,
    color_mode: String,
}

impl ParticlesEffect {
    pub fn new(cow_text: String, dna: CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        Self {
            cow_text,
            dna,
            pool: ParticlePool::new(),
            spawn_timer: 0.0,
            phase,
            instance_id,
            color_mode,
        }
    }
}

impl Effect for ParticlesEffect {
    fn update(&mut self, dt: f32, cols: usize, rows: usize) {
        seed_frame_rng(self.dna.phase_seed.wrapping_add(self.instance_id));
        self.spawn_timer += dt;
        let rate = if self.dna.particles.rate > 0 {
            self.dna.particles.rate
        } else {
            10
        };
        let interval = 1.0 / rate as f32;
        if self.spawn_timer >= interval {
            self.spawn_timer -= interval;
            let palette = color::parse_palette(&self.dna.particles.palette);
            let cow_start = find_cow_start_line(&self.cow_text);
            let spawn_x = 14.0f32.min(cols.saturating_sub(1) as f32);
            let spawn_y = ((cow_start + 2) as f32).min(rows.saturating_sub(1) as f32);
            spawn_for_type(
                &mut self.pool,
                self.dna.particles.r#type,
                spawn_x,
                spawn_y,
                &palette,
                self.phase + dt,
                cols,
                rows,
            );
        }
        self.pool.update(dt);
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let palette = color::parse_palette(&self.dna.palette);
        render_text_palette(fb, &self.cow_text, Color::WHITE, &self.color_mode, &palette, time);
        self.pool.render(fb, time, easing::expo_out);
    }
}

// ── Pulse / Glow ───────────────────────────────────────────────────

/// Color cycling (rainbow sweep or localized glow).
#[derive(Debug)]
pub struct PulseEffect {
    cow_text: String,
    palette: Vec<(u8, u8, u8)>,
    glow_color: Color,
    glow_radius: f32,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    color_mode: String,
    instinct: AnimalInstinct,
}

impl PulseEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let palette = color::parse_palette(&dna.palette);
        let instinct = detect_animal_instinct(&cow_text, dna);
        let glow_color = if dna.glow.color != "#ffffff" && !dna.glow.color.is_empty() {
            parse_hex(&dna.glow.color)
                .map(|(r, g, b)| Color::rgb(r, g, b))
                .unwrap_or(Color::WHITE)
        } else {
            match instinct {
                AnimalInstinct::Dragon => Color::rgb(255, 68, 0),
                AnimalInstinct::Marine => Color::rgb(0, 229, 255),
                AnimalInstinct::Cephalopod => Color::rgb(186, 104, 200),
                AnimalInstinct::Canine => Color::rgb(255, 183, 77),
                AnimalInstinct::Feline => Color::rgb(255, 112, 67),
                AnimalInstinct::Avian => Color::rgb(255, 213, 79),
                AnimalInstinct::Amphibian => Color::rgb(76, 175, 80),
                AnimalInstinct::Spectral => Color::rgb(128, 255, 219),
                AnimalInstinct::Bovine => Color::rgb(129, 199, 132),
            }
        };
        let glow_radius = if dna.glow.radius > 0.0 {
            dna.glow.radius
        } else {
            4.0
        };
        Self {
            cow_text,
            palette,
            glow_color,
            glow_radius,
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            color_mode,
            instinct,
        }
    }
}

impl Effect for PulseEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let intensity = (self.easing_fn)(t);

        // Render cow text with color from palette
        let color = if self.palette.is_empty() {
            Color::WHITE
        } else {
            let (r, g, b) = lerp_palette(&self.palette, intensity);
            Color::rgb(r, g, b)
        };
        render_text_palette(
            fb,
            &self.cow_text,
            color,
            &self.color_mode,
            &self.palette,
            time,
        );

        // Apply signature wildlife aura glow at center of mascot
        let cx = fb.width as f32 / 2.0;
        let cy = fb.height as f32 * 0.35;
        let glow_intensity = intensity * 0.5;
        apply_glow(
            fb,
            cx,
            cy,
            self.glow_radius,
            self.glow_color,
            glow_intensity,
        );

        // Natural Wildlife Instinct pulse particles
        match self.instinct {
            AnimalInstinct::Dragon => {
                if intensity > 0.6 {
                    let rx = ((time * 11.0) as usize) % fb.width.max(1);
                    let ry = ((time * 7.0) as usize) % fb.height.max(1);
                    let _ = fb.set(rx, ry, Cell::new('*', Color::rgb(255, 68, 0)));
                }
            }
            AnimalInstinct::Spectral if intensity > 0.6 => {
                let rx = ((time * 9.0) as usize) % fb.width.max(1);
                let ry = ((time * 5.0) as usize) % fb.height.max(1);
                let _ = fb.set(rx, ry, Cell::new('✦', Color::rgb(189, 147, 249)));
            }
            AnimalInstinct::Marine if intensity > 0.6 => {
                let rx = ((time * 8.0) as usize) % fb.width.max(1);
                let ry = ((time * 6.0) as usize) % fb.height.max(1);
                let _ = fb.set(rx, ry, Cell::new('°', Color::rgb(0, 229, 255)));
            }
            _ => {}
        }
    }
}

// ── Glitch ─────────────────────────────────────────────────────────

/// Random character swap with binary/hex within cow body bounds.
#[derive(Debug)]
pub struct GlitchEffect {
    cow_text: String,
    body_coords: Vec<(usize, usize)>,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    instinct: AnimalInstinct,
}

impl GlitchEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let cow_start = find_cow_start_line(&cow_text);
        let mut body_coords = Vec::new();
        for (y, line) in cow_text.lines().enumerate() {
            if y < cow_start {
                continue; // Bubble characters must never be glitched
            }
            for (x, ch) in line.chars().enumerate() {
                if !ch.is_whitespace() {
                    body_coords.push((x, y));
                }
            }
        }
        let palette = crate::color::parse_palette(&dna.palette);
        let instinct = detect_animal_instinct(&cow_text, dna);
        Self {
            cow_text,
            body_coords,
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            instinct,
        }
    }
}

impl Effect for GlitchEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        render_text_palette(
            fb,
            &self.cow_text,
            Color::WHITE,
            &self.color_mode,
            &self.palette,
            time,
        );
        if self.body_coords.is_empty() || fb.width == 0 || fb.height == 0 {
            return;
        }
        let (glitch_chars, c1, c2) = match self.instinct {
            AnimalInstinct::Dragon => (
                &['*', '^', '#', '@', '§', '»'][..],
                Color::rgb(255, 68, 0),
                Color::rgb(255, 204, 0),
            ),
            AnimalInstinct::Marine => (
                &['~', 'o', '°', '≈', '*', '#'][..],
                Color::rgb(0, 229, 255),
                Color::rgb(0, 150, 255),
            ),
            AnimalInstinct::Cephalopod => (
                &['@', '%', '*', '§', '&', '#'][..],
                Color::rgb(186, 104, 200),
                Color::rgb(0, 255, 204),
            ),
            AnimalInstinct::Spectral => (
                &['✦', '✧', '0', '1', '·', '█'][..],
                Color::rgb(128, 255, 219),
                Color::rgb(189, 147, 249),
            ),
            _ => (
                &['0', '1', '#', '@', '█', '▓'][..],
                Color::rgb(0, 255, 0),
                Color::rgb(255, 0, 0),
            ),
        };
        let t = time * self.speed + self.phase;
        let intensity = (t * 3.0).sin() * 0.5 + 0.5;
        let count = ((intensity * 8.0) as usize).min(self.body_coords.len());

        for i in 0..count {
            let seed = (t * 100.0 + i as f32) as u32;
            let coord_idx = (seed as usize).wrapping_mul(7) % self.body_coords.len();
            let (x, y) = self.body_coords[coord_idx];
            if x < fb.width && y < fb.height {
                let ch = glitch_chars[(seed as usize) % glitch_chars.len()];
                let c = if seed % 2 == 0 { c1 } else { c2 };
                let _ = fb.set(x, y, Cell::new(ch, c));
            }
        }
    }
}

/// In-place hovering flight with flapping wings, stationary at stagnant position (0, 0),
/// or authentic Nyan Cat rainbow wave propulsion and twinkling starfield.
#[derive(Debug)]
pub struct FlyEffect {
    cow_text: String,
    line_offsets: Vec<usize>,
    cow_start_line: usize,
    _amp: Amplitude,
    _easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    is_nyan: bool,
    instinct: AnimalInstinct,
    pub body: crate::kinematics::KinematicBody,
    elapsed: f32,
    eye_landmarks: Vec<(usize, usize)>,
}

impl FlyEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let line_offsets = compute_line_offsets(&cow_text);
        let cow_start_line = find_cow_start_line(&cow_text);
        let eye_landmarks = crate::cow::detect_cow_eyes(&cow_text, cow_start_line);
        let (w, h) = crate::kinematics::ascii_dimensions(&cow_text);
        let mut body =
            crate::kinematics::KinematicBody::new(w, h, crate::kinematics::BoundsMode::Wrap);
        body.vx = 0.0;
        body.vy = 0.0;
        let lower = cow_text.to_ascii_lowercase();
        let is_nyan = cow_text.contains("-_-_") || lower.contains("nyan");
        let instinct = detect_animal_instinct(&cow_text, dna);
        let palette = crate::color::parse_palette(&dna.palette);
        Self {
            cow_text,
            line_offsets,
            cow_start_line,
            _amp: dna.amplitude.clone(),
            _easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            is_nyan,
            instinct,
            body,
            elapsed: 0.0,
            eye_landmarks,
        }
    }

    fn render_nyan(&self, fb: &mut FrameBuffer, time: f32) {
        let wave_phase = ((time * 12.0) as usize) % 2;
        let bob_y = ((time * 7.0).sin() * 0.6) as i32;

        let mut y = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if y >= fb.height {
                return;
            }

            // Speech/thought bubble: strictly anchored at (0, 0), completely untouched
            if y < self.cow_start_line {
                for (x, ch) in line.chars().enumerate() {
                    if x < fb.width {
                        let cell_fg = resolve_bubble_cell_fg(ch);
                        let _ = fb.set(x, y, Cell::new(ch, cell_fg));
                    }
                }
                y += 1;
                return;
            }

            // Vertical flight bobbing for creature lines
            let target_y = (y as i32 + bob_y).max(self.cow_start_line as i32);
            if target_y < 0 || target_y as usize >= fb.height {
                y += 1;
                return;
            }
            let draw_y = target_y as usize;

            let is_trail_row = line.contains("-_-_") || line.contains("_-_-");
            // Detect where the pop-tart pastry starts (first ',' or '|' on rainbow line)
            let pastry_start = line
                .chars()
                .position(|c| c == ',' || c == '|')
                .unwrap_or(usize::MAX);

            for (x, mut ch) in line.chars().enumerate() {
                if x >= fb.width || ch == ' ' {
                    continue;
                }

                // Starfield twinkling:
                if (ch == '+' || ch == 'o' || ch == '*' || ch == '·') && !is_trail_row {
                    let star_seed = (x * 13 + y * 7 + (time * 6.0) as usize) % 4;
                    let star_glyph = match star_seed {
                        0 => '✦',
                        1 => '*',
                        2 => '+',
                        _ => '·',
                    };
                    let star_color = match star_seed {
                        0 => Color::rgb(128, 216, 255), // star cyan
                        1 => Color::rgb(255, 255, 255), // diamond white
                        2 => Color::rgb(255, 245, 157), // pale gold
                        _ => Color::rgb(225, 190, 231), // lavender
                    };
                    let _ = fb.set(x, draw_y, Cell::new(star_glyph, star_color));
                    continue;
                }

                // Rainbow propulsion trail:
                if is_trail_row && x < pastry_start && (ch == '-' || ch == '_' || ch == '~') {
                    ch = if wave_phase == 0 {
                        if ch == '~' {
                            '-'
                        } else {
                            ch
                        }
                    } else if ch == '-' || ch == '~' {
                        '_'
                    } else {
                        '-'
                    };

                    // 6-band chromatic rainbow bands
                    let trail_row = y.saturating_sub(self.cow_start_line);
                    let rainbow_color = match trail_row % 4 {
                        0 => Color::rgb(255, 0, 51),   // Red
                        1 => Color::rgb(255, 153, 0),  // Orange / Yellow
                        2 => Color::rgb(51, 255, 0),   // Green
                        _ => Color::rgb(153, 51, 255), // Indigo / Purple
                    };
                    let _ = fb.set(x, draw_y, Cell::new(ch, rainbow_color));
                    continue;
                }

                // Pop-Tart Pastry & Cat Body:
                let fg_color =
                    if ch == ',' || (ch == '-' && x >= pastry_start) || ch == '|' || ch == '_' {
                        // Golden pastry crust
                        Color::rgb(230, 162, 108)
                    } else if ch == '/' || ch == '\\' || ch == '(' || ch == ')' || ch == '\'' {
                        // Soft gray cat head and paws
                        Color::rgb(160, 160, 160)
                    } else if ch == '.' {
                        // Rosy pink nose
                        Color::rgb(255, 64, 129)
                    } else if ch == '^' {
                        // Cat eyes
                        Color::rgb(20, 20, 20)
                    } else {
                        resolve_fg_palette(
                            &self.color_mode,
                            &self.palette,
                            x,
                            draw_y,
                            time,
                            Color::WHITE,
                        )
                    };

                let _ = fb.set(x, draw_y, Cell::new(ch, fg_color));
            }

            y += 1;
        });
    }
}

impl Effect for FlyEffect {
    fn update(&mut self, dt: f32, cols: usize, rows: usize) {
        self.elapsed += dt;
        self.body.update(dt, self.elapsed, cols, rows);
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        if self.is_nyan {
            self.render_nyan(fb, time);
            return;
        }

        // 2-phase in-place wing flap cycle:
        let flap_cycle = ((time * self.speed * 8.0) as usize) % 2;
        let is_upstroke = flap_cycle == 0;

        // Periodic eye-blink:
        let blink_cycle = (time * 0.28 + self.phase) % 1.0;
        let is_blinking = blink_cycle < 0.04;

        let mut y = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if y >= fb.height {
                return;
            }
            let hull = find_line_hull(line);

            // Speech/thought bubble: strictly anchored at (0, 0), completely untouched
            if y < self.cow_start_line {
                for (x, ch) in line.chars().enumerate() {
                    if x < fb.width {
                        let cell_fg = resolve_bubble_cell_fg(ch);
                        draw_char_with_hull(fb, x, y, x, hull, ch, cell_fg);
                    }
                }
                y += 1;
                return;
            }

            // Creature lines: flap wings/horns in place and blink eyes (zero-allocation)
            let mut chars_iter = line.chars().peekable();
            let mut x = 0usize;
            while let Some(mut ch) = chars_iter.next() {
                if x >= fb.width {
                    break;
                }

                // Eye blinking across all eye landmarks:
                let is_landmark = self.eye_landmarks.contains(&(y, x));
                if is_blinking && (is_landmark || (is_eye_glyph(ch) && chars_iter.peek().copied() == Some(ch))) {
                    let cell_fg = resolve_fg_palette_char(
                        &self.color_mode,
                        &self.palette,
                        x,
                        y,
                        time,
                        Color::WHITE,
                        '-',
                    );
                    draw_char_with_hull(fb, x, y, x, hull, '-', cell_fg);
                    if !is_landmark && chars_iter.peek().copied() == Some(ch) {
                        let _ = chars_iter.next();
                        if x + 1 < fb.width {
                            draw_char_with_hull(fb, x + 1, y, x + 1, hull, '-', cell_fg);
                        }
                        x += 2;
                        continue;
                    }
                    x += 1;
                    continue;
                }

                // In-place wing flap (carets '^' <-> 'v'):
                if is_upstroke {
                    if ch == 'v' {
                        ch = '^';
                    }
                } else if ch == '^' {
                    ch = 'v';
                }

                let cell_fg = resolve_fg_palette_char(
                    &self.color_mode,
                    &self.palette,
                    x,
                    y,
                    time,
                    Color::WHITE,
                    ch,
                );
                draw_char_with_hull(fb, x, y, x, hull, ch, cell_fg);
                x += 1;
            }
            y += 1;
        });

        // Natural Wildlife Instinct trailing flight particles:
        match self.instinct {
            AnimalInstinct::Dragon => {
                let ember_colors = [
                    Color::rgb(255, 34, 0),
                    Color::rgb(255, 102, 0),
                    Color::rgb(255, 204, 0),
                ];
                for i in 0..4 {
                    let seed = i * 17 + (time * 10.0) as usize;
                    let ex = (seed * 7 + 13) % fb.width.max(1);
                    let ey = (seed * 3 + self.cow_start_line) % fb.height.max(1);
                    let _ = fb.set(ex, ey, Cell::new('*', ember_colors[i % ember_colors.len()]));
                }
            }
            AnimalInstinct::Marine => {
                // Ocean spray
                for i in 0..4 {
                    let seed = i * 23 + (time * 8.0) as usize;
                    let ex = (seed * 5 + 7) % fb.width.max(1);
                    let ey = (seed * 3 + self.cow_start_line) % fb.height.max(1);
                    let _ = fb.set(ex, ey, Cell::new('~', Color::rgb(0, 229, 255)));
                }
            }
            AnimalInstinct::Avian => {
                // Aerodynamic feather motes
                for i in 0..3 {
                    let seed = i * 19 + (time * 6.0) as usize;
                    let ex = (seed * 9 + 5) % fb.width.max(1);
                    let ey = (seed * 4 + self.cow_start_line) % fb.height.max(1);
                    let _ = fb.set(ex, ey, Cell::new('\'', Color::rgb(255, 213, 79)));
                }
            }
            AnimalInstinct::Spectral => {
                // Trailing stardust
                for i in 0..4 {
                    let seed = i * 31 + (time * 7.0) as usize;
                    let ex = (seed * 11 + 3) % fb.width.max(1);
                    let ey = (seed * 5 + self.cow_start_line) % fb.height.max(1);
                    let _ = fb.set(ex, ey, Cell::new('✦', Color::rgb(189, 147, 249)));
                }
            }
            AnimalInstinct::Canine => {
                // Flying Ace Snoopy trailing wind gusts
                for i in 0..3 {
                    let seed = i * 13 + (time * 7.0) as usize;
                    let ex = (seed * 8 + 4) % fb.width.max(1);
                    let ey = (seed * 2 + self.cow_start_line) % fb.height.max(1);
                    let _ = fb.set(ex, ey, Cell::new('~', Color::rgb(224, 224, 224)));
                }
            }
            AnimalInstinct::Cephalopod => {
                // Jet propulsion water bubbles
                for i in 0..4 {
                    let seed = i * 15 + (time * 8.0) as usize;
                    let ex = (seed * 6 + 10) % fb.width.max(1);
                    let ey = (seed * 3 + self.cow_start_line) % fb.height.max(1);
                    let _ = fb.set(ex, ey, Cell::new('°', Color::rgb(0, 255, 204)));
                }
            }
            _ => {}
        }
    }
}

// ── Talk / Chew ────────────────────────────────────────────────────

/// Mouth + eye region animation on animal only (thought/speech bubble preserved verbatim).
#[derive(Debug)]
pub struct TalkEffect {
    cow_text: String,
    line_offsets: Vec<usize>,
    cow_start_line: usize,
    eye_pos: Option<(usize, usize)>,
    mouth_pos: Option<(usize, usize)>,
    _amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    instinct: AnimalInstinct,
    eye_landmarks: Vec<(usize, usize)>,
}

impl TalkEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let line_offsets = compute_line_offsets(&cow_text);
        let cow_start_line = find_cow_start_line(&cow_text);
        let lines: Vec<&str> = cow_text.lines().collect();

        let eye_landmarks = crate::cow::detect_cow_eyes(&cow_text, cow_start_line);
        let mut eye_pos: Option<(usize, usize)> = eye_landmarks.first().copied();
        let mut mouth_pos: Option<(usize, usize)> = None;

        for (i, line) in lines.iter().enumerate().skip(cow_start_line) {
            // 1. Detect eyes fallback if not detected
            if eye_pos.is_none() {
                if let Some(open) = line.find('(') {
                    if open + 3 <= line.len() && line.as_bytes().get(open + 3) == Some(&b')') {
                        eye_pos = Some((i, open + 1));
                    }
                }
            } else if mouth_pos.is_none() && eye_pos.is_some_and(|ep| i > ep.0) {
                // 2. Detect mouth/muzzle: (__), (..), (==) below eyes
                if let Some(open) = line.find("(__)") {
                    mouth_pos = Some((i, open + 1));
                } else if let Some(open) = line.find('(') {
                    if open + 3 <= line.len() && line.as_bytes().get(open + 3) == Some(&b')') {
                        mouth_pos = Some((i, open + 1));
                    }
                }
            }
        }

        let palette = crate::color::parse_palette(&dna.palette);
        let instinct = detect_animal_instinct(&cow_text, dna);

        Self {
            cow_text,
            line_offsets,
            cow_start_line,
            eye_pos,
            mouth_pos,
            _amp: dna.amplitude.clone(),
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            instinct,
            eye_landmarks,
        }
    }
}

impl Effect for TalkEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * (self.speed * 0.4) + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let mouth_chars = ['_', '.', 'o', 'O', 'w', 'W', '='];
        let mouth_idx = (eased * mouth_chars.len() as f32) as usize % mouth_chars.len();
        let mouth_ch = mouth_chars[mouth_idx];

        // Periodic eye-blink (~every 3.8s)
        let blink_cycle = (time * 0.26 + self.phase) % 1.0;
        let is_blinking = blink_cycle < 0.04;

        let eye_line = self.eye_pos.map(|(row, _)| row);

        let mut y = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if y >= fb.height {
                return;
            }
            let hull = find_line_hull(line);

            // Speech/thought bubble lines: preserve characters verbatim!
            if y < self.cow_start_line {
                for (x, ch) in line.chars().enumerate() {
                    if x < fb.width {
                        let cell_fg = resolve_bubble_cell_fg(ch);
                        draw_char_with_hull(fb, x, y, x, hull, ch, cell_fg);
                    }
                }
                y += 1;
                return;
            }

            // Animal body lines (zero-allocation streaming):
            for (x, mut display_ch) in line.chars().enumerate() {
                if x >= fb.width {
                    break;
                }

                // 1. Natural eye preservation and blinking across all landmarks
                let is_eye = self.eye_landmarks.contains(&(y, x))
                    || self.eye_pos.is_some_and(|(eye_row, eye_col)| y == eye_row && (x == eye_col || x == eye_col + 1));
                if is_eye {
                    if is_blinking {
                        display_ch = '-';
                    }
                    let cell_fg = resolve_fg_palette_char(
                        &self.color_mode,
                        &self.palette,
                        x,
                        y,
                        time,
                        Color::WHITE,
                        display_ch,
                    );
                    draw_char_with_hull(fb, x, y, x, hull, display_ch, cell_fg);
                    continue;
                }

                // 2. Mouth / jaw animation below eyes:
                if let Some((mouth_row, mouth_col)) = self.mouth_pos {
                    if y == mouth_row && (x == mouth_col || x == mouth_col + 1) {
                        display_ch = mouth_ch;
                    }
                } else if eye_line.is_none() || eye_line.is_some_and(|el| y > el) {
                    // Fallback for creatures without landmarks: animate only rows below eyes
                    if matches!(display_ch, '_' | '-' | '=' | 'w' | 'W' | '.') {
                        display_ch = mouth_ch;
                    }
                }

                let cell_fg = resolve_fg_palette_char(
                    &self.color_mode,
                    &self.palette,
                    x,
                    y,
                    time,
                    Color::WHITE,
                    display_ch,
                );
                draw_char_with_hull(fb, x, y, x, hull, display_ch, cell_fg);
            }
            y = y.saturating_add(1);
        });

        // Natural Wildlife Instinct vocal emissions:
        if mouth_ch == 'o' || mouth_ch == 'O' || mouth_ch == 'W' {
            if let Some((my, mx)) = self.mouth_pos {
                let px = if mx > 2 { mx - 1 } else { mx + 2 };
                if px < fb.width && my < fb.height {
                    match self.instinct {
                        AnimalInstinct::Dragon => {
                            let puff_ch = if mouth_ch == 'O' { '»' } else { '*' };
                            let _ = fb.set(px, my, Cell::new(puff_ch, Color::rgb(255, 68, 0)));
                        }
                        AnimalInstinct::Marine => {
                            let b_ch = if mouth_ch == 'O' { 'O' } else { 'o' };
                            let _ = fb.set(px, my, Cell::new(b_ch, Color::rgb(0, 229, 255)));
                        }
                        AnimalInstinct::Cephalopod => {
                            let ink_ch = if mouth_ch == 'O' { '@' } else { '*' };
                            let _ = fb.set(px, my, Cell::new(ink_ch, Color::rgb(142, 36, 170)));
                        }
                        AnimalInstinct::Canine => {
                            let bark_ch = if mouth_ch == 'O' { '!' } else { '>' };
                            let _ = fb.set(px, my, Cell::new(bark_ch, Color::rgb(255, 183, 77)));
                        }
                        AnimalInstinct::Avian => {
                            let note_ch = if mouth_ch == 'O' { '♫' } else { '♪' };
                            let _ = fb.set(px, my, Cell::new(note_ch, Color::rgb(0, 230, 118)));
                        }
                        AnimalInstinct::Spectral => {
                            let spark_ch = if mouth_ch == 'O' { '⚡' } else { '*' };
                            let _ = fb.set(px, my, Cell::new(spark_ch, Color::rgb(128, 255, 219)));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// ── Sway / Pendulum ────────────────────────────────────────────────

/// Progressive sway with anchored bottom and unskewed speech/thought bubble at (0, 0).
#[derive(Debug)]
pub struct SwayEffect {
    cow_text: String,
    line_offsets: Vec<usize>,
    cow_start_line: usize,
    amp: Amplitude,
    easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    instinct: AnimalInstinct,
}

impl SwayEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let line_offsets = compute_line_offsets(&cow_text);
        let cow_start_line = find_cow_start_line(&cow_text);
        let palette = crate::color::parse_palette(&dna.palette);
        let instinct = detect_animal_instinct(&cow_text, dna);
        let mut amp = dna.amplitude.clone();
        if amp.sway <= 0.05 {
            amp.sway = 0.35;
        }
        Self {
            cow_text,
            line_offsets,
            cow_start_line,
            amp,
            easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            color_mode,
            palette,
            instinct,
        }
    }
}

impl Effect for SwayEffect {
    fn update(&mut self, _dt: f32, _cols: usize, _rows: usize) {}

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let t = (time * self.speed + self.phase) % 1.0;
        let eased = (self.easing_fn)(t);
        let total_lines = self.line_offsets.len().max(1);
        let cow_lines = total_lines.saturating_sub(self.cow_start_line).max(1);

        let mut i = 0usize;
        for_each_line(&self.cow_text, &self.line_offsets, |line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if i >= fb.height {
                return;
            }
            let hull = find_line_hull(line);

            // Speech/thought bubble: STRICTLY anchored at x_off = 0! NEVER skewed or shifted!
            let x_off = if i < self.cow_start_line {
                0
            } else {
                let rel_i = i - self.cow_start_line;
                // Progressive skew on the creature itself: top of creature = max, bottom feet = 0
                let skew_factor = 1.0 - (rel_i as f32 / cow_lines as f32);
                let base_skew = ((eased - 0.5) * 4.0 * self.amp.sway) * skew_factor;
                // Serpentine sine wave for amphibians / vipers:
                let wave_skew = if self.instinct == AnimalInstinct::Amphibian {
                    ((time * 4.0 + rel_i as f32 * 0.6).sin() * 1.2) as i32
                } else {
                    0
                };
                base_skew as i32 + wave_skew
            };

            let mut x = 0usize;
            for ch in line.chars() {
                let xi = x as i32 + x_off;
                if xi >= 0 {
                    let uxi = xi as usize;
                    if uxi < fb.width {
                        let cell_fg = resolve_fg_palette(
                            &self.color_mode,
                            &self.palette,
                            uxi,
                            i,
                            time,
                            Color::WHITE,
                        );
                        draw_char_with_hull(fb, uxi, i, x, hull, ch, cell_fg);
                    }
                }
                x = x.saturating_add(1);
            }
            i = i.saturating_add(1);
        });
    }
}

// ── Dissolve ───────────────────────────────────────────────────────

/// Break art into falling chars, reassemble, with speech bubble preserved at (0, 0).
#[derive(Debug)]
pub struct DissolveEffect {
    cow_text: String,
    line_ranges: Vec<(usize, usize)>,
    cow_start_line: usize,
    _easing_fn: fn(f32) -> f32,
    phase: f32,
    speed: f32,
    scatter_offsets: Vec<(f32, f32)>,
    color_mode: String,
    palette: Vec<(u8, u8, u8)>,
    elapsed: f32,
}

impl DissolveEffect {
    pub fn new(cow_text: String, dna: &CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let cow_start_line = find_cow_start_line(&cow_text);
        let line_ranges: Vec<(usize, usize)> = cow_text
            .lines()
            .map(|line| {
                let start = line.as_ptr() as usize - cow_text.as_ptr() as usize;
                (start, line.len())
            })
            .collect();
        let mut scatter_offsets = Vec::new();
        for (y, &(start, len)) in line_ranges.iter().enumerate() {
            let line = &cow_text[start..start + len];
            for (x, ch) in line.chars().enumerate() {
                if ch == ' ' || y < cow_start_line {
                    scatter_offsets.push((f32::MAX, f32::MAX));
                    continue;
                }
                let seed = ((x as f32 * 0.618 + y as f32 * 0.382) * 1000.0) as u32;
                let dx = ((seed.wrapping_mul(7) % 20) as i32 - 10) as f32;
                let dy = ((seed.wrapping_mul(13) % 10) as i32 - 5) as f32;
                scatter_offsets.push((dx, dy));
            }
        }
        let palette = crate::color::parse_palette(&dna.palette);
        Self {
            cow_text,
            line_ranges,
            cow_start_line,
            _easing_fn: easing::by_name(&dna.easing.base),
            phase,
            speed: dna.speed,
            scatter_offsets,
            color_mode,
            palette,
            elapsed: 0.0,
        }
    }
}

impl Effect for DissolveEffect {
    fn update(&mut self, dt: f32, _cols: usize, _rows: usize) {
        self.elapsed += dt * self.speed;
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        let cycle = (time * self.speed + self.phase) % 2.0;
        let t = if cycle < 1.0 { cycle } else { 2.0 - cycle };
        let scatter = 1.0 - t;

        let mut offset_idx = 0;
        for (y, &(start, len)) in self.line_ranges.iter().enumerate() {
            let line = &self.cow_text[start..start + len];
            for (x, ch) in line.chars().enumerate() {
                let (dx_base, dy_base) = self.scatter_offsets[offset_idx];
                offset_idx += 1;
                if dx_base == f32::MAX && dy_base == f32::MAX {
                    if y < self.cow_start_line && ch != ' ' && y < fb.height && x < fb.width {
                        let cell_fg = resolve_bubble_cell_fg(ch);
                        let _ = fb.set(x, y, Cell::new(ch, cell_fg));
                    }
                    continue;
                }
                let dx = dx_base * scatter;
                let dy = dy_base * scatter;
                let final_x = (x as f32 + dx) as i32;
                let final_y = (y as f32 + dy) as i32;
                if final_x >= 0 && final_y >= 0 {
                    let fx = final_x as usize;
                    let fy = final_y as usize;
                    if fy < fb.height && fx < fb.width {
                        let alpha = (t * 255.0) as u8;
                        let cell_fg = resolve_fg_palette_char(
                            &self.color_mode,
                            &self.palette,
                            x,
                            y,
                            time,
                            Color::WHITE,
                            ch,
                        );
                        let _ = fb.set(
                            fx,
                            fy,
                            Cell {
                                ch,
                                fg: cell_fg,
                                bg: Color::TRANSPARENT,
                                alpha,
                            },
                        );
                    }
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        self.elapsed >= 2.0
    }
}

// ── Shared helpers ─────────────────────────────────────────────────

/// Determine the bounding hull (first and last non-space character column) for a line.
#[inline]
pub(crate) fn find_line_hull(line: &str) -> Option<(usize, usize)> {
    let mut start = None;
    let mut end = None;
    for (i, ch) in line.chars().enumerate() {
        if ch != ' ' {
            if start.is_none() {
                start = Some(i);
            }
            end = Some(i);
        }
    }
    start.zip(end)
}

/// Render a cell respecting bounding-hull occlusion:
/// - If `ch != ' '`: draws the character with fg color.
/// - If `ch == ' '` and inside hull: draws opaque blank to prevent background scenery bleed.
/// - If `ch == ' '` and outside hull (leading or trailing): leaves framebuffer untouched so scenery shows.
#[inline]
pub(crate) fn draw_char_with_hull(
    fb: &mut FrameBuffer,
    x: usize,
    y: usize,
    orig_col: usize,
    hull: Option<(usize, usize)>,
    ch: char,
    fg: Color,
) {
    if x >= fb.width || y >= fb.height {
        return;
    }
    if ch != ' ' {
        let _ = fb.set(x, y, Cell::new(ch, fg));
    } else if let Some((start, end)) = hull {
        if orig_col >= start && orig_col <= end {
            let _ = fb.set(x, y, Cell::new(' ', Color::WHITE));
        }
    }
}

/// Pre-compute byte offsets for each line in text (for zero-alloc iteration).
fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(16);
    offsets.push(0);
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            offsets.push(i + ch.len_utf8());
        }
    }
    offsets
}

/// Iterate lines without allocating a Vec. Calls `f(line_str)` for each line.
#[inline]
fn for_each_line<F: FnMut(&str)>(text: &str, offsets: &[usize], mut f: F) {
    for window in offsets.windows(2) {
        let start = window[0];
        let end = window[1].min(text.len());
        f(&text[start..end]);
    }
    // Last line (may not end with \n)
    if let Some(&last) = offsets.last() {
        if last < text.len() {
            f(&text[last..]);
        }
    }
}

/// Resolve foreground color based on color_mode, animal DNA palette, and character glyph.
pub(crate) fn resolve_fg_palette_char(
    color_mode: &str,
    palette: &[(u8, u8, u8)],
    x: usize,
    y: usize,
    time: f32,
    base: Color,
    ch: char,
) -> Color {
    match color_mode {
        "rainbow" | "lolcat" => {
            let (r, g, b) = crate::color::lolcat_color(x as f32, y as f32, time, 0.0);
            Color { r, g, b, a: 255 }
        }
        "default" | "animal" | "natural" | "animal_natural" => {
            let p = if !palette.is_empty() {
                palette
            } else {
                crate::color::get_natural_palette("cow")
            };
            let (r, g, b) = crate::color::natural_creature_color(p, x, y, ch);
            Color { r, g, b, a: 255 }
        }
        "solid" | "static" | "white" => Color::WHITE,
        "none" => base,
        _ => {
            if let Some(mascot) = color_mode
                .strip_prefix("natural:")
                .or_else(|| color_mode.strip_prefix("animal:"))
                .or_else(|| color_mode.strip_prefix("animal_natural:"))
            {
                let p = crate::color::get_natural_palette(mascot);
                let (r, g, b) = crate::color::natural_creature_color(p, x, y, ch);
                Color { r, g, b, a: 255 }
            } else if color_mode.starts_with('#') {
                let hexes: Vec<String> = color_mode
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                let custom_palette = crate::color::parse_palette(&hexes);
                let (r, g, b) =
                    crate::color::palette_gradient(&custom_palette, x as f32, y as f32, time);
                Color { r, g, b, a: 255 }
            } else if !palette.is_empty() {
                let (r, g, b) = crate::color::natural_creature_color(palette, x, y, ch);
                Color { r, g, b, a: 255 }
            } else {
                let p = crate::color::get_natural_palette(color_mode);
                if p != crate::color::get_natural_palette("cow") || color_mode == "cow" {
                    let (r, g, b) = crate::color::natural_creature_color(p, x, y, ch);
                    Color { r, g, b, a: 255 }
                } else {
                    base
                }
            }
        }
    }
}

/// Resolve foreground color based on color_mode and animal DNA palette.
pub(crate) fn resolve_fg_palette(
    color_mode: &str,
    palette: &[(u8, u8, u8)],
    x: usize,
    y: usize,
    time: f32,
    base: Color,
) -> Color {
    resolve_fg_palette_char(color_mode, palette, x, y, time, base, ' ')
}

/// Resolve foreground color based on color_mode.
#[allow(dead_code)]
pub(crate) fn resolve_fg(color_mode: &str, x: usize, y: usize, time: f32, base: Color) -> Color {
    resolve_fg_palette_char(color_mode, &[], x, y, time, base, ' ')
}

/// Resolve foreground color based on color_mode and character.
#[allow(dead_code)]
pub(crate) fn resolve_fg_char(
    color_mode: &str,
    x: usize,
    y: usize,
    time: f32,
    base: Color,
    ch: char,
) -> Color {
    resolve_fg_palette_char(color_mode, &[], x, y, time, base, ch)
}

/// Resolve foreground color for speech/thought bubble borders, connector rings, and message text.
/// Ensures 100% maximum contrast and crystal-clear readability against any terminal background.
#[inline]
pub(crate) fn resolve_bubble_cell_fg(ch: char) -> Color {
    if ch == 'o' || ch == 'O' || ch == '\\' || ch == '/' {
        Color::rgb(180, 230, 255)
    } else if ch == '_' || ch == '-' || ch == '=' || ch == '(' || ch == ')' || ch == '|' || ch == '<' || ch == '>' || ch == '+' {
        Color::rgb(215, 225, 240)
    } else if ch == ' ' {
        Color::WHITE
    } else {
        Color::rgb(255, 255, 255)
    }
}

/// Render text into the framebuffer at row 0.
fn render_text(fb: &mut FrameBuffer, text: &str, fg: Color, color_mode: &str, time: f32) {
    render_text_offset(fb, text, fg, 0, 0, color_mode, time);
}

/// Render text into the framebuffer at row 0 with wildlife palette.
fn render_text_palette(
    fb: &mut FrameBuffer,
    text: &str,
    fg: Color,
    color_mode: &str,
    palette: &[(u8, u8, u8)],
    time: f32,
) {
    render_text_offset_palette(fb, text, fg, 0, 0, color_mode, palette, time);
}

/// Render text with x/y offset into the framebuffer.
fn render_text_offset(
    fb: &mut FrameBuffer,
    text: &str,
    fg: Color,
    x_off: i32,
    y_off: i32,
    color_mode: &str,
    time: f32,
) {
    render_text_offset_palette(fb, text, fg, x_off, y_off, color_mode, &[], time);
}

/// Render text with x/y offset into the framebuffer using palette.
/// Applies bounding-hull occlusion: for each line, spaces between the first
/// and last non-space columns are written as opaque cells so background
/// scenery does not bleed through the animal's interior.
#[allow(clippy::too_many_arguments)]
fn render_text_offset_palette(
    fb: &mut FrameBuffer,
    text: &str,
    fg: Color,
    x_off: i32,
    y_off: i32,
    color_mode: &str,
    palette: &[(u8, u8, u8)],
    time: f32,
) {
    let cow_start_line = find_cow_start_line(text);

    // Zero-allocation streaming scanline render with bounding-hull occlusion masking.
    for (y, line) in text.lines().enumerate() {
        let yi = y as i32 + y_off;
        if yi < 0 {
            continue;
        }
        let uyi = yi as usize;
        if uyi >= fb.height {
            break;
        }

        let (hull_start, hull_end) = match find_line_hull(line) {
            Some(h) => h,
            None => continue, // Entirely whitespace line — skip
        };

        let is_bubble_line = y < cow_start_line;
        let animal_rel_y = y.saturating_sub(cow_start_line);

        for (x, ch) in line.chars().enumerate() {
            let xi = x as i32 + x_off;
            if xi < 0 {
                continue;
            }
            let uxi = xi as usize;
            if uxi >= fb.width {
                break;
            }

            let cell_fg = if is_bubble_line {
                // Speech / thought bubble styling: maximum visibility, crisp and legible
                if ch == 'o' || ch == 'O' || ch == '\\' || ch == '/' {
                    Color::rgb(180, 230, 255)
                } else if ch == '_' || ch == '-' || ch == '=' || ch == '(' || ch == ')' || ch == '|' || ch == '<' || ch == '>' || ch == '+' {
                    Color::rgb(215, 225, 240)
                } else if ch == ' ' {
                    Color::WHITE
                } else {
                    // Message text inside the bubble: pure bright readable white
                    Color::rgb(255, 255, 255)
                }
            } else {
                resolve_fg_palette_char(color_mode, palette, x, animal_rel_y, time, fg, ch)
            };
            draw_char_with_hull(fb, uxi, uyi, x, Some((hull_start, hull_end)), ch, cell_fg);
        }
    }
}

/// Apply a radial glow effect centered at (cx, cy).
fn apply_glow(fb: &mut FrameBuffer, cx: f32, cy: f32, radius: f32, color: Color, intensity: f32) {
    if intensity <= 0.0 || radius <= 0.0 {
        return;
    }
    let r = radius as i32;
    let cx_i = cx as i32;
    let cy_i = cy as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let x = cx_i + dx;
            let y = cy_i + dy;
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= fb.width || y >= fb.height {
                continue;
            }
            let glow = gaussian_glow(dx as f32, dy as f32, radius) * intensity;
            if glow < 0.05 {
                continue;
            }
            let alpha = (glow * 255.0) as u8;
            let existing = fb.get(x, y);
            if existing.alpha == 0 {
                // Empty cell — fill with glow
                let _ = fb.set(
                    x,
                    y,
                    Cell {
                        ch: '·',
                        fg: Color {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: alpha,
                        },
                        bg: Color::TRANSPARENT,
                        alpha,
                    },
                );
            }
        }
    }
}

/// Compound signature animation: combines body kinematics, particle emitters,
/// localized glow, and keep-alive eye-blinks tailored to the animal's DNA.
pub struct CompoundSignatureEffect {
    cow_text: String,
    dna: CowDna,
    pool: ParticlePool,
    spawn_timer: f32,
    phase: f32,
    speed: f32,
    instance_id: u32,
    base_effect: Box<dyn Effect>,
}

impl std::fmt::Debug for CompoundSignatureEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompoundSignatureEffect")
            .field("dna", &self.dna)
            .field("speed", &self.speed)
            .field("phase", &self.phase)
            .finish()
    }
}

impl CompoundSignatureEffect {
    pub fn new(cow_text: String, dna: CowDna, instance_id: u32, color_mode: String) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let speed = dna.speed;
        let cow_text_clone = cow_text.clone();
        let base_effect = create_effect(dna.base, cow_text, dna.clone(), instance_id, &color_mode);
        Self {
            cow_text: cow_text_clone,
            dna,
            pool: ParticlePool::new(),
            spawn_timer: 0.0,
            phase,
            speed,
            instance_id,
            base_effect,
        }
    }

    /// Construct a CompoundSignatureEffect wrapping an explicit base effect.
    pub fn with_base(
        cow_text: String,
        dna: CowDna,
        instance_id: u32,
        base_effect: Box<dyn Effect>,
    ) -> Self {
        let phase = instance_phase(dna.phase_seed, instance_id);
        let speed = dna.speed;
        Self {
            cow_text,
            dna,
            pool: ParticlePool::new(),
            spawn_timer: 0.0,
            phase,
            speed,
            instance_id,
            base_effect,
        }
    }
}

impl Effect for CompoundSignatureEffect {
    fn update(&mut self, dt: f32, cols: usize, rows: usize) {
        self.base_effect.update(dt, cols, rows);

        if self.dna.particles.rate > 0 {
            seed_frame_rng(self.dna.phase_seed.wrapping_add(self.instance_id));
            self.spawn_timer += dt * self.speed;
            let interval = 1.0 / self.dna.particles.rate.max(1) as f32;
            if self.spawn_timer >= interval {
                self.spawn_timer -= interval;
                let palette = color::parse_palette(&self.dna.particles.palette);
                // Compute emitter position: origin around creature mouth/head
                let cow_start = find_cow_start_line(&self.cow_text);
                let spawn_x = 14.0f32.min(cols.saturating_sub(1) as f32);
                let spawn_y = ((cow_start + 2) as f32).min(rows.saturating_sub(1) as f32);
                spawn_for_type(
                    &mut self.pool,
                    self.dna.particles.r#type,
                    spawn_x,
                    spawn_y,
                    &palette,
                    self.phase + dt,
                    cols,
                    rows,
                );
            }
            self.pool.update(dt);
        }
    }

    fn render(&self, fb: &mut FrameBuffer, time: f32) {
        self.base_effect.render(fb, time);

        if self.dna.particles.rate > 0 {
            self.pool.render(fb, time, easing::expo_out);
        }

        // Apply DNA glow if configured
        if self.dna.glow.radius > 0.0 && self.dna.glow.color != "#ffffff" {
            let glow_color = parse_hex(&self.dna.glow.color)
                .map(|(r, g, b)| Color::rgb(r, g, b))
                .unwrap_or(Color::WHITE);
            apply_glow(
                fb,
                fb.width as f32 / 2.0,
                fb.height as f32 / 3.0,
                self.dna.glow.radius,
                glow_color,
                0.4,
            );
        }
    }

    fn on_resize(&mut self, cols: usize, rows: usize) {
        self.base_effect.on_resize(cols, rows);
    }
}

/// Create an effect for a scene configuration and DNA.
/// - If `effect_name` is `"static"`, returns `StaticEffect`.
/// - If `effect_name` is `"default"`, uses the animal's signature base animation.
/// - If the animal's DNA defines particles or radial glow, they are ALWAYS preserved
///   by wrapping the base effect in `CompoundSignatureEffect`.
pub fn create_scene_effect(
    effect_name: &str,
    cow_text: String,
    dna: CowDna,
    instance_id: u32,
    color_mode: &str,
) -> Box<dyn Effect> {
    let eff = effect_name.trim().to_ascii_lowercase();
    let palette = crate::color::parse_palette(&dna.palette);
    if eff == "static" {
        return Box::new(StaticEffect::with_palette(
            cow_text,
            color_mode.to_string(),
            palette,
        ));
    }

    let base: Box<dyn Effect> = match eff.as_str() {
        "breathe" => Box::new(BreatheEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "float" => Box::new(FloatEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "walk" => Box::new(WalkEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "particles" => Box::new(ParticlesEffect::new(
            cow_text.clone(),
            dna.clone(),
            instance_id,
            color_mode.to_string(),
        )),
        "pulse" => Box::new(PulseEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "glitch" => Box::new(GlitchEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "fly" => Box::new(FlyEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "talk" => Box::new(TalkEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "sway" | "liquid" | "flow" => Box::new(SwayEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "dissolve" => Box::new(DissolveEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "squish" | "bounce" => Box::new(BreatheEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "matrix" | "digital" => Box::new(GlitchEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "abduction" | "beam" => Box::new(FloatEffect::new(
            cow_text.clone(),
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        "animal_natural" | "natural" | "default" | "dna" => create_effect(
            dna.base,
            cow_text.clone(),
            dna.clone(),
            instance_id,
            color_mode,
        ),
        _ => create_effect(
            dna.base,
            cow_text.clone(),
            dna.clone(),
            instance_id,
            color_mode,
        ),
    };

    if dna.particles.rate > 0 || (dna.glow.radius > 0.0 && dna.glow.color != "#ffffff") {
        Box::new(CompoundSignatureEffect::with_base(
            cow_text,
            dna,
            instance_id,
            base,
        ))
    } else {
        base
    }
}

/// Create an effect from a base animation type.
pub fn create_effect(
    base: BaseAnim,
    cow_text: String,
    dna: CowDna,
    instance_id: u32,
    color_mode: &str,
) -> Box<dyn Effect> {
    match base {
        BaseAnim::Breathe => Box::new(BreatheEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Float => Box::new(FloatEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Walk => Box::new(WalkEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Particles => Box::new(ParticlesEffect::new(
            cow_text,
            dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Pulse => Box::new(PulseEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Glitch => Box::new(GlitchEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Fly => Box::new(FlyEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Talk => Box::new(TalkEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Sway => Box::new(SwayEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Dissolve => Box::new(DissolveEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        // Extended animation types map to their closest existing effect
        BaseAnim::Liquid => Box::new(SwayEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Squish => Box::new(BreatheEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Matrix => Box::new(GlitchEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
        BaseAnim::Abduction => Box::new(FloatEffect::new(
            cow_text,
            &dna,
            instance_id,
            color_mode.to_string(),
        )),
    }
}

// ── Legacy static cow text (backward compat) ───────────────────────

/// Write the cow text into `fb` at row 0 (Phase 0 compat).
pub fn render_static_cow(fb: &mut FrameBuffer, cow_text: &str, color_mode: &str, time: f32) {
    render_text(fb, cow_text, Color::WHITE, color_mode, time);
}

/// Returns the canonical "default" cow art for Phase 0.
#[must_use]
pub fn default_cow_text() -> &'static str {
    r#"        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::GlowDna;

    const COW: &str = "  ^__^  \n (oo)   \n(__)    ";

    /// Count non-space cells in the framebuffer.
    fn count_non_space(fb: &FrameBuffer) -> usize {
        let mut n = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if fb.get(x, y).ch != ' ' {
                    n += 1;
                }
            }
        }
        n
    }

    /// Count cells matching a predicate.
    #[allow(dead_code)]
    fn count_cells(fb: &FrameBuffer, pred: impl Fn(char) -> bool) -> usize {
        let mut n = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if pred(fb.get(x, y).ch) {
                    n += 1;
                }
            }
        }
        n
    }

    // ── StaticEffect ──────────────────────────────────────────────

    #[test]
    fn static_cow_renders_exact_art() {
        let mut fb = FrameBuffer::new(80, 24);
        render_static_cow(&mut fb, default_cow_text(), "static", 0.0);
        fb.swap();
        // default_cow_text() starts with "        \   ^__^"
        // Row 0: 8 spaces, \, 3 spaces, ^__^ → ^ at column 12
        assert_eq!(fb.get(12, 0).ch, '^', "caret (^) must be at (12,0)");
        assert_eq!(fb.get(13, 0).ch, '_');
        assert_eq!(fb.get(14, 0).ch, '_');
        assert_eq!(fb.get(15, 0).ch, '^');
        // Row 1: "         \\  (oo)\\_______" → 9 spaces, \, 2 spaces, (oo) → ( at column 12
        assert_eq!(fb.get(12, 1).ch, '(');
        assert_eq!(fb.get(13, 1).ch, 'o');
        assert_eq!(fb.get(14, 1).ch, 'o');
        assert_eq!(fb.get(15, 1).ch, ')');
        // Row 3: must contain || for the legs
        let row3: String = (0..30).map(|x| fb.get(x, 3).ch).collect();
        assert!(
            row3.contains("||"),
            "row 3 must contain || for the legs: got {row3:?}"
        );
    }

    #[test]
    fn static_cow_all_chars_white() {
        let mut fb = FrameBuffer::new(80, 24);
        render_static_cow(&mut fb, default_cow_text(), "static", 0.0);
        fb.swap();
        for y in 0..5 {
            for x in 0..20 {
                let cell = fb.get(x, y);
                if cell.ch != ' ' && cell.ch != '\0' {
                    assert_eq!(
                        cell.fg,
                        Color::WHITE,
                        "non-space cell at ({x},{y}) ch={:?} must be WHITE",
                        cell.ch
                    );
                }
            }
        }
    }

    #[test]
    fn empty_text_produces_no_damage() {
        let mut fb = FrameBuffer::new(10, 10);
        render_static_cow(&mut fb, "", "static", 0.0);
        assert!(
            fb.compute_damage().is_empty(),
            "rendering empty text should produce zero damage"
        );
    }

    #[test]
    fn render_cow_text_fits_in_framebuffer() {
        // A 1x1 framebuffer should only show the first character
        let mut fb = FrameBuffer::new(1, 1);
        render_static_cow(&mut fb, "ABC\nDEF", "static", 0.0);
        fb.swap();
        assert_eq!(fb.get(0, 0).ch, 'A');
        // Second row is out of bounds for height=1
    }

    // ── BreatheEffect ─────────────────────────────────────────────

    #[test]
    fn breathe_remains_anchored_at_stagnant_position() {
        let mut dna = CowDna::default();
        dna.amplitude.breath = 5.0;
        dna.speed = 1.0;
        let effect = BreatheEffect::new(COW.to_string(), &dna, 0, "static".to_string());

        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.5);
        fb1.swap();

        // Find the first non-space cell in each frame
        let find_first_char = |fb: &FrameBuffer| -> Option<(usize, usize, char)> {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    let ch = fb.get(x, y).ch;
                    if ch != ' ' {
                        return Some((x, y, ch));
                    }
                }
            }
            None
        };

        let pos0 = find_first_char(&fb0);
        let pos1 = find_first_char(&fb1);
        assert!(pos0.is_some(), "breathe at t=0 must render something");
        assert!(pos1.is_some(), "breathe at t=0.5 must render something");

        // The baseline must remain anchored at stagnant position (0, 0), never hopping Y rows
        let (_, y0, _) = pos0.unwrap();
        let (_, y1, _) = pos1.unwrap();
        assert_eq!(y0, 0, "breathe baseline must remain anchored at row 0");
        assert_eq!(y1, 0, "breathe baseline must remain anchored at row 0");

        // Flank/chest character breathes organically between resting ('_') and inhale ('~')
        assert_eq!(fb0.get(3, 0).ch, '_');
        assert_eq!(fb1.get(3, 0).ch, '~');
    }

    #[test]
    fn zero_amplitude_produces_no_offset() {
        let find_first = |fb: &FrameBuffer| -> Option<(usize, usize)> {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return Some((x, y));
                    }
                }
            }
            None
        };

        // Breathe: zero amplitude → no Y drift
        let mut dna = CowDna::default();
        dna.amplitude.breath = 0.0;
        dna.speed = 1.0;
        let effect = BreatheEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.5);
        fb1.swap();
        assert_eq!(
            find_first(&fb0),
            find_first(&fb1),
            "breathe zero amplitude = no Y drift"
        );

        // Float: zero amplitude → no drift
        let mut dna = CowDna::default();
        dna.amplitude.sway = 0.0;
        dna.amplitude.float = 0.0;
        dna.speed = 1.0;
        let effect = FloatEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.5);
        fb1.swap();
        assert_eq!(
            find_first(&fb0),
            find_first(&fb1),
            "float zero amplitude = no drift"
        );
    }

    #[test]
    fn breathe_instance_phase_offsets_multiple_instances() {
        let mut dna = CowDna::default();
        dna.amplitude.breath = 1.0;
        dna.speed = 1.0;
        let e0 = BreatheEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let e1 = BreatheEffect::new(COW.to_string(), &dna, 1, "static".to_string());

        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        e0.render(&mut fb0, 0.3);
        fb0.swap();
        e1.render(&mut fb1, 0.3);
        fb1.swap();

        // Both instances remain anchored at row 0 (no bouncing)
        assert_eq!(fb0.get(2, 0).ch, '^');
        assert_eq!(fb1.get(2, 0).ch, '^');
    }

    // ── FloatEffect ───────────────────────────────────────────────

    #[test]
    fn float_remains_anchored_at_stagnant_position() {
        let mut dna = CowDna::default();
        dna.amplitude.sway = 3.0;
        dna.amplitude.float = 3.0;
        dna.speed = 1.0;
        let effect = FloatEffect::new(COW.to_string(), &dna, 0, "static".to_string());

        let mut fb0 = FrameBuffer::new(80, 24);
        let mut fb1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb0, 0.0);
        fb0.swap();
        effect.render(&mut fb1, 0.75);
        fb1.swap();

        let find_first = |fb: &FrameBuffer| -> Option<(usize, usize)> {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return Some((x, y));
                    }
                }
            }
            None
        };

        let p0 = find_first(&fb0).expect("float t=0 must render");
        let p1 = find_first(&fb1).expect("float t=0.75 must render");
        // Position must remain firmly anchored at stagnant position (2, 0)
        assert_eq!(p0, (2, 0), "float t=0 must be anchored at (2, 0)");
        assert_eq!(p1, (2, 0), "float t=0.75 must be anchored at (2, 0)");
    }

    // ── WalkEffect ────────────────────────────────────────────────

    #[test]
    fn walk_legs_alternate_between_frames() {
        let dna = CowDna::default();
        let effect_a = WalkEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let effect_b = WalkEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb_a = FrameBuffer::new(80, 24);
        let mut fb_b = FrameBuffer::new(80, 24);
        effect_a.render(&mut fb_a, 0.25);
        fb_a.swap();
        effect_b.render(&mut fb_b, 0.75);
        fb_b.swap();

        // Last row of the COW string is row 2 (3 lines: "  ^__^  ", " (oo)   ", "(__)    ")
        let last_row = 2;
        let legs_a: Vec<char> = (0..fb_a.width).map(|x| fb_a.get(x, last_row).ch).collect();
        let legs_b: Vec<char> = (0..fb_b.width).map(|x| fb_b.get(x, last_row).ch).collect();

        // At least one leg char must be present
        let has_slash = |v: &[char]| {
            v.iter()
                .any(|c| *c == '/' || *c == '\\' || *c == '╱' || *c == '╲')
        };
        assert!(
            has_slash(&legs_a),
            "walk t=0.25 must have leg chars: {legs_a:?}"
        );
        assert!(
            has_slash(&legs_b),
            "walk t=0.75 must have leg chars: {legs_b:?}"
        );

        // The legs must differ between the two frames
        assert_ne!(
            legs_a, legs_b,
            "walk legs must alternate between t=0.25 and t=0.75"
        );
    }

    #[test]
    fn walk_only_modifies_last_row() {
        let dna = CowDna::default();
        let effect = WalkEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.25);
        fb.swap();

        // COW = "  ^__^  \n (oo)   \n(__)    " → ^ at x=2, _ at x=3
        assert_eq!(fb.get(2, 0).ch, '^', "walk should not modify row 0");
        assert_eq!(fb.get(3, 0).ch, '_', "walk should not modify row 0");
        // Row 1 should also be unchanged
        assert_eq!(fb.get(1, 1).ch, '(', "walk should not modify row 1");
    }

    #[test]
    fn walk_remains_stationary_at_stagnant_position() {
        let dna = CowDna::default();
        let mut effect = WalkEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);

        assert_eq!(effect.body.screen_x(), 0);

        effect.update(1.0, 80, 24);
        assert_eq!(
            effect.body.screen_x(),
            0,
            "walk must remain stationary at stagnant position"
        );

        effect.render(&mut fb, 1.0);
        fb.swap();

        assert_eq!(fb.get(2, 0).ch, '^');
    }

    #[test]
    fn fly_remains_stationary_at_stagnant_position() {
        let dna = CowDna::default();
        let mut effect = FlyEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);

        assert_eq!(effect.body.screen_x(), 0);

        effect.update(1.0, 80, 24);
        assert_eq!(
            effect.body.screen_x(),
            0,
            "fly must remain stationary at stagnant position"
        );

        effect.render(&mut fb, 1.0);
        fb.swap();

        assert_eq!(fb.get(2, 0).ch, '^');
    }

    // ── GlitchEffect ──────────────────────────────────────────────

    #[test]
    fn glitch_intensity_varies_with_time() {
        let dna = CowDna::default();
        let glitch_chars = ['0', '1', '#', '@', '█', '▓'];

        let count_glitch = |fb: &FrameBuffer| -> usize {
            let mut n = 0;
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if glitch_chars.contains(&fb.get(x, y).ch) {
                        n += 1;
                    }
                }
            }
            n
        };

        // peak: sin(t*3) ≈ 1 → intensity ≈ 1 → many glitch chars
        let effect_peak = GlitchEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb_peak = FrameBuffer::new(80, 24);
        effect_peak.render(&mut fb_peak, std::f32::consts::FRAC_PI_6);
        fb_peak.swap();
        let peak_count = count_glitch(&fb_peak);
        assert!(
            peak_count > 0,
            "glitch at peak intensity should produce glitch characters"
        );

        // trough: sin(t*3) ≈ -1 → intensity ≈ 0 → fewer or zero
        let effect_trough = GlitchEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb_trough = FrameBuffer::new(80, 24);
        effect_trough.render(&mut fb_trough, std::f32::consts::FRAC_PI_3);
        fb_trough.swap();
        let trough_count = count_glitch(&fb_trough);

        assert!(
            peak_count >= trough_count,
            "peak intensity ({peak_count}) should have >= trough ({trough_count})"
        );
    }

    // ── FlyEffect ─────────────────────────────────────────────────

    #[test]
    fn fly_renders_wing_flap_indicator() {
        let dna = CowDna::default();
        let effect = FlyEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.0);
        fb.swap();

        // In-place wing flap on creature wing/horn character
        let ch = fb.get(2, 0).ch;
        assert!(
            ch == '^' || ch == 'v',
            "fly wing flap on creature must be '^' or 'v', got '{ch}'"
        );
    }

    #[test]
    fn fly_wing_flap_toggles() {
        let dna = CowDna::default();
        let e1 = FlyEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let e2 = FlyEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb1 = FrameBuffer::new(80, 24);
        let mut fb2 = FrameBuffer::new(80, 24);
        // t=0.0 is upstroke ('^'), t=0.15 is downstroke ('v')
        e1.render(&mut fb1, 0.0);
        fb1.swap();
        e2.render(&mut fb2, 0.15);
        fb2.swap();
        assert_eq!(fb1.get(2, 0).ch, '^');
        assert_eq!(fb2.get(2, 0).ch, 'v');
        assert_ne!(
            fb1.get(2, 0).ch,
            fb2.get(2, 0).ch,
            "wing flap must toggle between upstroke and downstroke in place"
        );
    }

    #[test]
    fn nyan_flying_wave_propulsion_alternates() {
        let nyan_art = "      (  )\n       (oo)\n+    o    +\n-_-_-_-_,------,\n_-_-_-_-| /\\_/\\\n-_-_-_-~|__( ^ .^)\n_-_-_-_-''  ''\n+    o    +";
        let dna = CowDna::default();
        let effect = FlyEffect::new(nyan_art.to_string(), &dna, 0, "animal".to_string());
        assert!(effect.is_nyan);

        let mut fb1 = FrameBuffer::new(80, 24);
        let mut fb2 = FrameBuffer::new(80, 24);
        // wave_phase = ((time * 12.0) as usize) % 2;
        // time = 0.0 -> wave_phase = 0
        // time = 0.1 -> wave_phase = (1.2 as usize) % 2 = 1
        effect.render(&mut fb1, 0.0);
        fb1.swap();
        effect.render(&mut fb2, 0.1);
        fb2.swap();

        // Rainbow row has both '-' and '_' in phase 0 and phase 1
        let mut row_chars_1 = Vec::new();
        let mut row_chars_2 = Vec::new();
        for y in 0..fb1.height {
            for x in 0..8 {
                let ch1 = fb1.get(x, y).ch;
                let ch2 = fb2.get(x, y).ch;
                if ch1 == '-' || ch1 == '_' {
                    row_chars_1.push((x, y, ch1));
                }
                if ch2 == '-' || ch2 == '_' {
                    row_chars_2.push((x, y, ch2));
                }
            }
        }
        assert!(
            !row_chars_1.is_empty(),
            "nyan trail must have '-' and '_' characters"
        );
        assert_eq!(row_chars_1.len(), row_chars_2.len(), "trail lengths match");
        // Check that at least some characters toggled between '-' and '_'
        let toggled = row_chars_1
            .iter()
            .zip(row_chars_2.iter())
            .any(|(a, b)| a.2 != b.2);
        assert!(
            toggled,
            "nyan rainbow wave must alternate characters between frames"
        );
    }

    #[test]
    fn marine_aquatic_wave_swimming() {
        let whale_art =
            "      (  )\n       (oo)\n  .-'\"'-.\n / #     \\\n| # # # # |\n \\       /\n  `'---'`";
        let dna = CowDna {
            palette: vec!["#0288d1".to_string(), "#29b6f6".to_string()],
            ..Default::default()
        };
        let effect = FloatEffect::new(
            format!("whale\n{}", whale_art),
            &dna,
            0,
            "animal".to_string(),
        );
        assert_eq!(effect.instinct, AnimalInstinct::Marine);

        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        // Ensure colored cells were drawn using the whale's palette
        let mut colored_cells = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                let cell = fb.get(x, y);
                if cell.ch != ' ' && cell.fg != Color::WHITE {
                    colored_cells += 1;
                }
            }
        }
        assert!(
            colored_cells > 0,
            "marine creature must render in authentic wildlife colors"
        );
    }

    // ── TalkEffect ────────────────────────────────────────────────

    #[test]
    fn talk_replaces_and_cycles_mouth_chars() {
        let dna = CowDna::default();
        let mouth_chars = ['_', '.', 'o', 'O', 'w', 'W', '='];

        // Render at 4 different times: verify mouth chars are replaced AND cycle
        let mut seen = std::collections::HashSet::new();
        for i in 0..4 {
            let effect = TalkEffect::new(COW.to_string(), &dna, 0, "static".to_string());
            let mut fb = FrameBuffer::new(80, 24);
            let t = (i + 1) as f32 / 4.0; // t > 0 to avoid t=0 blink window for eye check
            effect.render(&mut fb, t);
            fb.swap();

            // COW = "  ^__^  \n (oo)   \n(__)    "
            // Eyes at row 1, col 2: must be preserved as eye char ('o' or '-' when blinking)
            let eye_ch = fb.get(2, 1).ch;
            assert!(
                eye_ch == 'o' || eye_ch == '-',
                "talk must preserve eyes at (2,1), got '{eye_ch}' at t={t}"
            );

            // Mouth at row 2, col 2: " (__)"
            let mouth_ch = fb.get(2, 2).ch;
            assert!(
                mouth_chars.contains(&mouth_ch),
                "talk must replace mouth char at (2,2), got '{mouth_ch}' at t={t}"
            );
            seen.insert(mouth_ch);
        }
        assert!(
            seen.len() >= 2,
            "talk mouth should cycle through multiple chars, saw: {:?}",
            seen
        );
    }

    // ── SwayEffect ────────────────────────────────────────────────

    #[test]
    fn sway_top_rows_shift_more_than_bottom() {
        let mut dna = CowDna::default();
        dna.amplitude.sway = 4.0;
        dna.speed = 1.0;
        let effect = SwayEffect::new(COW.to_string(), &dna, 0, "static".to_string());

        let mut fb_t0 = FrameBuffer::new(80, 24);
        let mut fb_t1 = FrameBuffer::new(80, 24);
        effect.render(&mut fb_t0, 0.0);
        fb_t0.swap();
        effect.render(&mut fb_t1, 0.25);
        fb_t1.swap();

        // Top row should shift more than bottom row
        let find_x_at_row = |fb: &FrameBuffer, row: usize| -> Option<usize> {
            (0..fb.width).find(|&x| fb.get(x, row).ch != ' ')
        };

        // Row 0 is the top of the cow (highest skew), row 2 is bottom (zero skew)
        if let (Some(x0_top), Some(x1_top)) = (find_x_at_row(&fb_t0, 0), find_x_at_row(&fb_t1, 0)) {
            if let (Some(x0_bot), Some(x1_bot)) =
                (find_x_at_row(&fb_t0, 2), find_x_at_row(&fb_t1, 2))
            {
                let top_shift = (x0_top as i32 - x1_top as i32).unsigned_abs();
                let bot_shift = (x0_bot as i32 - x1_bot as i32).unsigned_abs();
                // Top should shift at least as much as bottom
                assert!(
                    top_shift >= bot_shift,
                    "sway top shift ({top_shift}) should be >= bottom shift ({bot_shift})"
                );
            }
        }
    }

    // ── DissolveEffect ────────────────────────────────────────────

    #[test]
    fn dissolve_at_assembled_has_low_scatter() {
        let dna = CowDna::default();
        let effect = DissolveEffect::new(COW.to_string(), &dna, 0, "static".to_string());

        let mut fb = FrameBuffer::new(80, 24);
        // At cycle midpoint (t=1.0 in 0→1→0 cycle), scatter is 0 (assembled)
        effect.render(&mut fb, 1.0);
        fb.swap();

        // When assembled, the cow art should be close to original positions
        // Count non-space cells — should be similar to cow character count
        let count = count_non_space(&fb);
        assert!(
            count > 10,
            "dissolve assembled (t=1.0) should render visible cow art, got {count} cells"
        );
    }

    #[test]
    fn dissolve_at_scattered_has_different_positions() {
        let dna = CowDna::default();
        let effect_assembled = DissolveEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let effect_scattered = DissolveEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb_assembled = FrameBuffer::new(80, 24);
        let mut fb_scattered = FrameBuffer::new(80, 24);

        // assembled: cycle=1.0 → t=1.0, scatter=0
        effect_assembled.render(&mut fb_assembled, 1.0);
        fb_assembled.swap();

        // scattered: cycle=0.0 → t=0.0, scatter=1.0
        effect_scattered.render(&mut fb_scattered, 0.0);
        fb_scattered.swap();

        // The cell positions should differ
        let mut differ = false;
        for y in 0..fb_assembled.height {
            for x in 0..fb_assembled.width {
                if fb_assembled.get(x, y).ch != fb_scattered.get(x, y).ch {
                    differ = true;
                    break;
                }
            }
            if differ {
                break;
            }
        }
        assert!(
            differ,
            "dissolve assembled vs scattered should produce different cell positions"
        );
    }

    // ── PulseEffect ───────────────────────────────────────────────

    #[test]
    fn pulse_with_palette_produces_colored_cells() {
        let dna = CowDna {
            palette: vec!["#ff0000".to_string(), "#0000ff".to_string()],
            ..CowDna::default()
        };
        let effect = PulseEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        let mut has_non_white = false;
        for y in 0..fb.height {
            for x in 0..fb.width {
                let c = fb.get(x, y);
                if c.ch != ' ' && (c.fg.r != 255 || c.fg.g != 255 || c.fg.b != 255) {
                    has_non_white = true;
                    break;
                }
            }
            if has_non_white {
                break;
            }
        }
        assert!(
            has_non_white,
            "pulse with palette should produce non-white colored cells"
        );
    }

    #[test]
    fn pulse_without_palette_is_white() {
        let dna = CowDna::default();
        let effect = PulseEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        // Without palette, pulse uses Color::WHITE
        for y in 0..5 {
            for x in 0..20 {
                let c = fb.get(x, y);
                if c.ch != ' ' {
                    assert_eq!(
                        c.fg,
                        Color::WHITE,
                        "pulse without palette should use WHITE at ({x},{y})"
                    );
                }
            }
        }
    }

    #[test]
    fn pulse_applies_glow_to_empty_cells() {
        let dna = CowDna {
            palette: vec!["#ff0000".to_string()],
            glow: GlowDna {
                radius: 10.0,
                color: "#ff0000".to_string(),
                ..GlowDna::default()
            },
            ..CowDna::default()
        };
        let effect = PulseEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        let mut fb = FrameBuffer::new(80, 24);
        effect.render(&mut fb, 0.5);
        fb.swap();

        // Glow should fill some empty cells with '·' character
        let mut glow_count = 0;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if fb.get(x, y).ch == '·' {
                    glow_count += 1;
                }
            }
        }
        assert!(
            glow_count > 0,
            "pulse glow should fill empty cells with '·' characters"
        );
    }

    // ── ParticlesEffect ───────────────────────────────────────────

    #[test]
    fn particles_renders_cow_always() {
        // Cow text must be rendered regardless of particle rate
        let find_char = |fb: &FrameBuffer, ch: char| -> bool {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch == ch {
                        return true;
                    }
                }
            }
            false
        };

        for rate in [0u32, 5, 20] {
            let mut dna = CowDna::default();
            dna.particles.rate = rate;
            let mut effect = ParticlesEffect::new(COW.to_string(), dna, 0, "static".to_string());
            effect.update(1.0, 80, 24);
            let mut fb = FrameBuffer::new(80, 24);
            effect.render(&mut fb, 1.0);
            fb.swap();
            // COW = "  ^__^  \n (oo)   \n(__)    " → ^ at (2,0)
            assert!(
                find_char(&fb, '^'),
                "particles with rate={rate} must render cow '^'"
            );
        }
    }

    // ── create_effect dispatch ─────────────────────────────────────

    #[test]
    fn create_effect_dispatches_correct_types() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
            BaseAnim::Liquid,
            BaseAnim::Squish,
            BaseAnim::Matrix,
            BaseAnim::Abduction,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0, "static");
            effect.update(0.1, 40, 10);
            let mut fb = FrameBuffer::new(40, 10);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert!(
                count_non_space(&fb) > 0,
                "{base:?} should render non-empty output"
            );
        }
    }

    #[test]
    fn create_effect_with_different_dna_produces_different_output() {
        let dna_fast = CowDna {
            speed: 10.0,
            amplitude: Amplitude {
                breath: 5.0,
                ..Amplitude::default()
            },
            ..CowDna::default()
        };

        let dna_slow = CowDna {
            speed: 0.1,
            amplitude: Amplitude {
                breath: 0.1,
                ..Amplitude::default()
            },
            ..CowDna::default()
        };

        let e_fast = BreatheEffect::new(COW.to_string(), &dna_fast, 0, "static".to_string());
        let e_slow = BreatheEffect::new(COW.to_string(), &dna_slow, 0, "static".to_string());

        let mut fb_fast = FrameBuffer::new(80, 24);
        let mut fb_slow = FrameBuffer::new(80, 24);
        // Use t=0.25 so fast (speed=10) has t*10=2.5→0.5→eased≠0,
        // while slow (speed=0.1) has t*0.1=0.025→eased≈0
        e_fast.render(&mut fb_fast, 0.25);
        fb_fast.swap();
        e_slow.render(&mut fb_slow, 0.25);
        fb_slow.swap();

        // Different DNA should produce different visual output while remaining anchored at row 0
        let find_y = |fb: &FrameBuffer| -> usize {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    if fb.get(x, y).ch != ' ' {
                        return y;
                    }
                }
            }
            fb.height
        };
        assert_eq!(find_y(&fb_fast), 0, "fast DNA must be anchored at row 0");
        assert_eq!(find_y(&fb_slow), 0, "slow DNA must be anchored at row 0");
        assert_ne!(
            fb_fast.get(3, 0).ch,
            fb_slow.get(3, 0).ch,
            "different DNA (speed/amplitude) should produce different breathing phase characters"
        );
    }

    // ── Edge cases: tiny terminal ──────────────────────────────────

    #[test]
    fn all_effects_survive_1x1_terminal() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
            BaseAnim::Liquid,
            BaseAnim::Squish,
            BaseAnim::Matrix,
            BaseAnim::Abduction,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0, "static");
            effect.update(0.1, 1, 1);
            let mut fb = FrameBuffer::new(1, 1);
            effect.render(&mut fb, 0.5);
            fb.swap();
            let cell = fb.get(0, 0);
            assert!(
                !cell.ch.is_control(),
                "{base:?} should not write control chars to (0,0)"
            );
        }
    }

    #[test]
    fn all_effects_survive_zero_size_terminal() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
            BaseAnim::Liquid,
            BaseAnim::Squish,
            BaseAnim::Matrix,
            BaseAnim::Abduction,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0, "static");
            effect.update(0.1, 0, 0);
            let mut fb = FrameBuffer::new(0, 0);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(
                fb.compute_damage().len(),
                0,
                "{base:?} zero-size framebuffer should produce empty damage"
            );
        }
    }

    #[test]
    fn all_effects_on_resize_no_panic() {
        let dna = CowDna::default();
        let bases = [
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Walk,
            BaseAnim::Particles,
            BaseAnim::Pulse,
            BaseAnim::Glitch,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Dissolve,
            BaseAnim::Liquid,
            BaseAnim::Squish,
            BaseAnim::Matrix,
            BaseAnim::Abduction,
        ];
        for base in &bases {
            let mut effect = create_effect(*base, COW.to_string(), dna.clone(), 0, "static");
            effect.on_resize(1, 1);
            let mut fb = FrameBuffer::new(1, 1);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(fb.width, 1, "{base:?} width should remain 1 after render");

            effect.on_resize(100, 50);
            let mut fb = FrameBuffer::new(100, 50);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(fb.width, 100, "{base:?} width should be 100 after resize");

            effect.on_resize(0, 0);
            let mut fb = FrameBuffer::new(0, 0);
            effect.render(&mut fb, 0.5);
            fb.swap();
            assert_eq!(
                fb.compute_damage().len(),
                0,
                "{base:?} zero-size should produce empty damage"
            );
        }
    }

    // ── Edge cases: extreme speed/amplitude ────────────────────────

    #[test]
    fn extreme_inputs_no_panic() {
        let dna = CowDna {
            speed: 1000.0,
            amplitude: Amplitude {
                breath: 1000.0,
                sway: 100.0,
                float: 100.0,
            },
            ..CowDna::default()
        };

        let mut fb = FrameBuffer::new(40, 10);
        let effect = BreatheEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        effect.render(&mut fb, 999.0);
        assert!(
            !fb.compute_damage().is_empty(),
            "breathe at extreme time should still produce damage (non-space cells)"
        );
        fb.swap();

        let mut fb_float = FrameBuffer::new(40, 10);
        let effect = FloatEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        effect.render(&mut fb_float, 999.0);
        assert!(
            !fb_float.compute_damage().is_empty(),
            "float at extreme time should still produce damage (non-space cells)"
        );
        fb_float.swap();
    }

    // ── Invariants: render count ───────────────────────────────────

    #[test]
    fn char_count_invariant_across_effects() {
        let dna = CowDna::default();

        // Effects that only reposition (no add/remove chars) must preserve char count
        let effects_and_times: Vec<(&str, Vec<f32>)> = vec![
            ("breathe", vec![0.0, 0.25, 0.5]),
            ("walk", vec![0.25, 0.5, 0.75]),
        ];

        for (name, times) in &effects_and_times {
            let first_count = {
                let effect = match *name {
                    "breathe" => Box::new(BreatheEffect::new(
                        COW.to_string(),
                        &dna,
                        0,
                        "static".to_string(),
                    )) as Box<dyn Effect>,
                    "walk" => Box::new(WalkEffect::new(
                        COW.to_string(),
                        &dna,
                        0,
                        "static".to_string(),
                    )),
                    _ => unreachable!(),
                };
                let mut fb = FrameBuffer::new(80, 24);
                effect.render(&mut fb, times[0]);
                fb.swap();
                count_non_space(&fb)
            };

            for &t in &times[1..] {
                let effect = match *name {
                    "breathe" => Box::new(BreatheEffect::new(
                        COW.to_string(),
                        &dna,
                        0,
                        "static".to_string(),
                    )) as Box<dyn Effect>,
                    "walk" => Box::new(WalkEffect::new(
                        COW.to_string(),
                        &dna,
                        0,
                        "static".to_string(),
                    )),
                    _ => unreachable!(),
                };
                let mut fb = FrameBuffer::new(80, 24);
                effect.render(&mut fb, t);
                fb.swap();
                let c = count_non_space(&fb);
                assert_eq!(
                    first_count, c,
                    "{name} char count should be constant at t={t}"
                );
            }
        }
    }

    // ── Effect::is_done returns false for non-one-shot effects ────

    #[test]
    fn non_one_shot_effects_return_is_done_false() {
        let dna = CowDna::default();
        let breathe = BreatheEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        assert!(!breathe.is_done());
        let dissolve = DissolveEffect::new(COW.to_string(), &dna, 0, "static".to_string());
        assert!(!dissolve.is_done());
    }

    // ── compute_line_offsets ───────────────────────────────────────

    #[test]
    fn compute_line_offsets_single_line() {
        let text = "hello";
        let offsets = compute_line_offsets(text);
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn compute_line_offsets_two_lines() {
        let text = "hello\nworld";
        let offsets = compute_line_offsets(text);
        assert_eq!(offsets, vec![0, 6]);
    }

    #[test]
    fn compute_line_offsets_empty_string() {
        let offsets = compute_line_offsets("");
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn compute_line_offsets_trailing_newline() {
        let text = "a\nb\n";
        let offsets = compute_line_offsets(text);
        assert_eq!(offsets, vec![0, 2, 4]);
    }

    // ── for_each_line ─────────────────────────────────────────────

    #[test]
    fn for_each_line_iterates_all_lines() {
        let text = "line1\nline2\nline3";
        let offsets = compute_line_offsets(text);
        let mut lines = Vec::new();
        for_each_line(text, &offsets, |line| {
            lines.push(line.to_string());
        });
        assert_eq!(lines, vec!["line1\n", "line2\n", "line3"]);
    }

    #[test]
    fn for_each_line_single_line() {
        let text = "only one";
        let offsets = compute_line_offsets(text);
        let mut lines = Vec::new();
        for_each_line(text, &offsets, |line| {
            lines.push(line.to_string());
        });
        assert_eq!(lines, vec!["only one"]);
    }

    #[test]
    fn for_each_line_empty() {
        let offsets = compute_line_offsets("");
        let mut lines = Vec::new();
        for_each_line("", &offsets, |line| {
            lines.push(line.to_string());
        });
        assert!(lines.is_empty(), "empty text should produce no lines");
    }

    #[test]
    fn for_each_line_matches_str_lines_count() {
        let text = "aaa\nbb\ncccc\nd";
        let offsets = compute_line_offsets(text);
        let expected_count = text.lines().count();
        let mut actual_count = 0;
        for_each_line(text, &offsets, |_| {
            actual_count += 1;
        });
        assert_eq!(actual_count, expected_count);
    }

    // ── Stagnant Position & Bubble Preservation Tests ─────────────

    #[test]
    fn find_cow_start_line_detects_bubbles_correctly() {
        let cow_raw = "   ^__^\n   (oo)\\_______\n   (__)\\       )\\/\\";
        // 1. Without bubble
        assert_eq!(find_cow_start_line(cow_raw), 0);

        // 2. With speech bubble
        let speech_scene = crate::cow::compose_scene_with_mode(cow_raw, "Hello", false);
        let speech_start = find_cow_start_line(&speech_scene);
        assert!(speech_start > 0, "speech scene must detect bubble lines");
        let lines: Vec<&str> = speech_scene.lines().collect();
        assert!(lines[speech_start].contains("^__^"));

        // 3. With thought bubble
        let thought_scene = crate::cow::compose_scene_with_mode(cow_raw, "Thinking", true);
        let thought_start = find_cow_start_line(&thought_scene);
        assert!(thought_start > 0, "thought scene must detect bubble lines");
        let lines_t: Vec<&str> = thought_scene.lines().collect();
        assert!(lines_t[thought_start].contains("^__^"));
    }

    #[test]
    fn all_effects_maintain_stagnant_position_and_unmoved_speech_bubble() {
        let cow_raw =
            "   ^__^\n   (oo)\\_______\n   (__)\\       )\\/\\\n       ||----w |\n       ||     ||";
        let scene = crate::cow::compose_scene_with_mode(cow_raw, "Forgum In-Place Animation", true);
        let effects_to_test = [
            "static",
            "breathe",
            "float",
            "walk",
            "particles",
            "pulse",
            "glitch",
            "fly",
            "talk",
            "sway",
            "dissolve",
            "default",
        ];

        let dna = CowDna::default();

        for effect_name in effects_to_test {
            let mut eff = create_scene_effect(effect_name, scene.clone(), dna.clone(), 0, "solid");
            let mut fb0 = FrameBuffer::new(80, 24);
            let mut fb1 = FrameBuffer::new(80, 24);

            // Render at t=0.0
            eff.render(&mut fb0, 0.0);
            fb0.swap();

            // Advance time and update
            eff.update(0.5, 80, 24);
            eff.render(&mut fb1, 0.5);
            fb1.swap();

            // The speech/thought bubble top border is at row 0:
            // It MUST remain at row 0 in both frames with no hopping or shifting
            assert_eq!(
                fb0.get(2, 0).ch,
                '_',
                "{effect_name}: bubble top border at (2,0) must be '_' at t=0"
            );
            assert_eq!(
                fb1.get(2, 0).ch,
                '_',
                "{effect_name}: bubble top border at (2,0) must be '_' at t=0.5 (must not hop or shift)"
            );

            // Row 1 contains bubble content "( Forgum In-Place Animation )"
            // The opening '(' must stay firmly at (0, 1)
            assert_eq!(
                fb0.get(0, 1).ch,
                '(',
                "{effect_name}: bubble border at (0,1) must be '(' at t=0"
            );
            assert_eq!(
                fb1.get(0, 1).ch,
                '(',
                "{effect_name}: bubble border at (0,1) must be '(' at t=0.5 (must not skew or move)"
            );

            // The 'F' in "Forgum" must stay at (2, 1) and NOT be corrupted or moved
            assert_eq!(
                fb0.get(2, 1).ch,
                'F',
                "{effect_name}: bubble text at (2,1) must be 'F' at t=0"
            );
            assert_eq!(
                fb1.get(2, 1).ch,
                'F',
                "{effect_name}: bubble text at (2,1) must be 'F' at t=0.5"
            );
        }
    }
}
