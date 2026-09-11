//! Structured, responsive terminal tables for inspecting CLI options.
//!
//! Provides `forgum options [category]` (animals, effects, scenery, colors, all)
//! with adaptive column sizing, word wrapping (CSS flexbox/Tailwind model), and Unicode box borders.

use crate::cow::{str_display_width, wrap_words};

/// Format an adaptive Unicode or ASCII table.
pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    let (term_w, _) = forgum_platform::terminal_size();
    let total_width = (term_w as usize).max(60);

    // Border and padding overhead: 1 border per col + 1 on end, plus 2 spaces padding per col
    let num_cols = headers.len();
    let overhead = (num_cols + 1) + (num_cols * 2);
    let available_content = total_width.saturating_sub(overhead).max(num_cols * 8);

    // Compute column widths
    // First N-1 columns get compact width based on their header/content, last column takes the rest
    let mut col_widths = vec![0usize; num_cols];
    let mut used_for_initial = 0;

    for (col_idx, w) in col_widths
        .iter_mut()
        .enumerate()
        .take(num_cols.saturating_sub(1))
    {
        let max_content = rows
            .iter()
            .filter_map(|r| r.get(col_idx))
            .map(|cell| str_display_width(cell))
            .max()
            .unwrap_or(0);
        let header_w = str_display_width(headers[col_idx]);
        let ideal = max_content.max(header_w).clamp(8, 26);
        *w = ideal;
        used_for_initial += ideal;
    }

    // Last column takes remaining width
    let last_col_idx = num_cols.saturating_sub(1);
    let remaining = available_content.saturating_sub(used_for_initial).max(18);
    col_widths[last_col_idx] = remaining;

    let use_unicode = true;

    let (tl, tr, bl, br, tj, bj, lj, rj, cj, hz, vt) = if use_unicode {
        ('┌', '┐', '└', '┘', '┬', '┴', '├', '┤', '┼', '─', '│')
    } else {
        ('+', '+', '+', '+', '+', '+', '+', '+', '+', '-', '|')
    };

    let mut out = String::new();

    // Top border
    out.push(tl);
    for (i, w) in col_widths.iter().enumerate() {
        for _ in 0..*w + 2 {
            out.push(hz);
        }
        if i + 1 < num_cols {
            out.push(tj);
        }
    }
    out.push(tr);
    out.push('\n');

    // Header row
    out.push(vt);
    for (i, h) in headers.iter().enumerate() {
        out.push(' ');
        let display_w = str_display_width(h);
        out.push_str("\x1b[1;36m");
        out.push_str(h);
        out.push_str("\x1b[0m");
        for _ in display_w..col_widths[i] {
            out.push(' ');
        }
        out.push(' ');
        out.push(vt);
    }
    out.push('\n');

    // Header separator
    out.push(lj);
    for (i, w) in col_widths.iter().enumerate() {
        for _ in 0..*w + 2 {
            out.push(hz);
        }
        if i + 1 < num_cols {
            out.push(cj);
        }
    }
    out.push(rj);
    out.push('\n');

    // Rows
    for row in rows {
        // Wrap each cell's text to fit column width
        let mut wrapped_cells: Vec<Vec<String>> = Vec::with_capacity(num_cols);
        let mut max_cell_lines = 1;
        for (i, w) in col_widths.iter().enumerate() {
            let content = row.get(i).copied().unwrap_or("");
            let lines = wrap_words(content, *w);
            max_cell_lines = max_cell_lines.max(lines.len());
            wrapped_cells.push(lines);
        }

        // Render each line of the row
        for line_idx in 0..max_cell_lines {
            out.push(vt);
            for (col_idx, w) in col_widths.iter().enumerate() {
                out.push(' ');
                let line_str = wrapped_cells[col_idx]
                    .get(line_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let line_w = str_display_width(line_str);
                out.push_str(line_str);
                for _ in line_w..*w {
                    out.push(' ');
                }
                out.push(' ');
                out.push(vt);
            }
            out.push('\n');
        }
    }

    // Bottom border
    out.push(bl);
    for (i, w) in col_widths.iter().enumerate() {
        for _ in 0..*w + 2 {
            out.push(hz);
        }
        if i + 1 < num_cols {
            out.push(bj);
        }
    }
    out.push(br);
    out.push('\n');

    out
}

/// Render structured options table for the specified category.
pub fn render_options(category: &str) -> String {
    let cat = category.to_ascii_lowercase();
    let mut out = String::new();

    let show_all = cat.is_empty() || cat == "all";
    let show_animals = show_all
        || cat == "animals"
        || cat == "animal"
        || cat == "cows"
        || cat == "cow"
        || cat == "mascots";
    let show_effects = show_all
        || cat == "effects"
        || cat == "effect"
        || cat == "animations"
        || cat == "animation"
        || cat == "fx";
    let show_scenery = show_all || cat == "scenery";
    let show_mountains = show_all || show_scenery || cat == "mountains" || cat == "mountain";
    let show_roads = show_all || show_scenery || cat == "roads" || cat == "road";
    let show_environments =
        show_all || show_scenery || cat == "environments" || cat == "environment" || cat == "env";
    let show_colors = show_all
        || cat == "colors"
        || cat == "color"
        || cat == "color-mode"
        || cat == "palette"
        || cat == "palettes";
    let show_shells = show_all
        || cat == "shells"
        || cat == "shell"
        || cat == "completions"
        || cat == "completion"
        || cat == "hooks"
        || cat == "hook";
    let show_config =
        show_all || cat == "config" || cat == "configs" || cat == "settings" || cat == "keys";
    let show_mux = show_all
        || cat == "tmux"
        || cat == "zellij"
        || cat == "wezterm"
        || cat == "multiplexers"
        || cat == "mux";
    let show_eyes = show_all || cat == "eyes" || cat == "eye";
    let show_tongue = show_all || cat == "tongue" || cat == "tongues";
    let show_logs = show_all
        || cat == "logs"
        || cat == "log"
        || cat == "diagnostics"
        || cat == "diagnose"
        || cat == "triage"
        || cat == "bugradar";

    if !show_animals
        && !show_effects
        && !show_mountains
        && !show_roads
        && !show_environments
        && !show_colors
        && !show_shells
        && !show_config
        && !show_mux
        && !show_eyes
        && !show_tongue
        && !show_logs
    {
        out.push_str(&format!(
            "\n\x1b[1;33mUnknown options category '{category}'.\x1b[0m\n\n\
            \x1b[1;36mAvailable categories:\x1b[0m\n\
              • \x1b[1mall\x1b[0m            - Complete options catalog across all domains\n\
              • \x1b[1manimals\x1b[0m        - 132 animal mascots & visual archetypes (--animal / --cow)\n\
              • \x1b[1meffects\x1b[0m        - Dynamic animation motion effects (--effect / --animation)\n\
              • \x1b[1mscenery\x1b[0m        - Procedural mountains, roads, and particle environments\n\
              • \x1b[1mmountains\x1b[0m      - Procedural mountain horizons (--mountain)\n\
              • \x1b[1mroads\x1b[0m          - Ground and terrain surface styles (--road)\n\
              • \x1b[1menvironments\x1b[0m   - Atmospheric particle emitter themes (--env / --environment)\n\
              • \x1b[1mcolors\x1b[0m         - Chromatic color modes and hex palettes (--color-mode)\n\
              • \x1b[1mshells\x1b[0m         - Supported shells, config paths, and tab completion\n\
              • \x1b[1mconfig\x1b[0m         - Supported configuration keys, types, and schema\n\
              • \x1b[1mmultiplexers\x1b[0m   - Terminal multiplexers (tmux, zellij, wezterm, screen)\n\
              • \x1b[1meyes\x1b[0m           - Cow eye expressions and glyphs (--eyes)\n\
              • \x1b[1mtongue\x1b[0m         - Cow tongue glyphs and expressions (--tongue)\n\
              • \x1b[1mlogs\x1b[0m           - Structured logging, streaming, live triage, and bug radar\n\n\
            \x1b[1;32mUsage:\x1b[0m\n\
              forgum list [category]\n\
              forgum --list [category]\n\
              forgum <arg> list   (e.g. forgum --animal list, forgum --effect list)\n"
        ));
        return out;
    }

    if show_animals {
        out.push_str(
            "\n\x1b[1;35m━━━ Curated Archetype Animal Mascots (--cow / --animal) ━━━\x1b[0m\n",
        );
        let headers = [
            "Mascot Name",
            "Base Effect",
            "Scenery Archetype",
            "Description",
        ];
        let rows = vec![
            vec![
                "default",
                "Walk",
                "Pasture",
                "Classic bovine mascot with natural bovine chew cadence and walking cycle",
            ],
            vec![
                "dragon",
                "Walk",
                "Inferno",
                "Ferocious fire-breathing dragon walking across volcanic magma beds",
            ],
            vec![
                "tux",
                "Walk",
                "Arctic",
                "Beloved Linux penguin waddling over polar icebergs and snow",
            ],
            vec![
                "cat",
                "Breathe",
                "City",
                "Playful feline silhouette perched high above urban rooftops and skyscrapers",
            ],
            vec![
                "ghost",
                "Float",
                "Graveyard",
                "Spooky spectral phantom hovering gracefully through foggy tombstones",
            ],
            vec![
                "elephant",
                "Walk",
                "Savanna",
                "Majestic giant marching across sun-drenched golden African plains",
            ],
            vec![
                "bunny",
                "Walk",
                "Pasture",
                "Cute rabbit hopping gently along grassy trails and floral hedges",
            ],
            vec![
                "corgi",
                "Walk",
                "Pasture",
                "Energetic corgi with animated short legs and cheerful cadence",
            ],
            vec![
                "fox",
                "Walk",
                "Forest",
                "Clever woodland fox stalking quietly through ancient whispering pines",
            ],
            vec![
                "octopus",
                "Float",
                "Ocean",
                "Deep-sea cephalopod undulating gently above submarine tectonic ridges",
            ],
            vec![
                "vader",
                "Walk",
                "Space",
                "Imperial dark lord striding through cosmic void and starfields",
            ],
            vec![
                "moofasa",
                "Walk",
                "Savanna",
                "Regal lion monarch watching over the golden grasslands",
            ],
            vec![
                "stegosaurus",
                "Walk",
                "Jurassic",
                "Armored dinosaur grazing beside primeval cycads and palm ferns",
            ],
            vec![
                "random",
                "Dynamic",
                "Dynamic",
                "Picks a random mascot and matching procedural landscape each launch",
            ],
        ];
        out.push_str(&format_table(&headers, &rows));

        out.push_str("\n\x1b[1;35m━━━ Complete Cow Mascot Catalog (132 Available Built-In Animals) ━━━\x1b[0m\n");
        let cat_headers = ["Category", "Mascot Names"];
        let cat_rows = vec![
            vec!["Farm & Domestic", "default, bunny, cat, cat2, catfence, charlie, corgi, doge, duck, fat-cow, goat, goat2, hippie, kitty, kitten, lamb, lamb2, meow, milk, moose, mule, owl, pig, ram, rooster, sheep, shrug, squirrel, turkey"],
            vec!["Wild & Safari", "armadillo, bearface, elephant, elephant2, elephant-in-snake, fox, hedgehog, koala, lobster, luke-koala, moofasa, panther, rhino, sloth, telebears, tiger, tortoise, tweety-bird, wolf"],
            vec!["Oceanic & Amphibian", "bud-frogs, docker-whale, dolphin, ebi_furai, happy-whale, jellyfish, octopus, pufferfish, seahorse, seahorse-big, smiling-octopus, squid, turtle, walrus, whale"],
            vec!["Fantasy & Sci-Fi", "atat, cthulhu-mini, daemon, dragon, dragon-and-cow, ghost, ghostbusters, glados, mech-and-cow, minotaur, mooghidjirah, moojira, pterodactyl, sauron, stegosaurus, tux, tux-big, unipony, vader, wizard, yoda"],
            vec!["Pop Culture & Fun", "beavis.zen, bill-the-cat, charizardvice, fat-banana, flaming-sheep, golden-eagle, hellokitty, hypno, kiss, lollerskates, mona-lisa, nyan, radioactive-kitty, ren, snoopy, snoopyhouse, snoopysleep, stimpy, vulpix"],
            vec!["Abstract & Quirky", "apt, bees, claw-arm, cower, cowfee, eyes, fence, hiya, jesus, king, knight, kosh, mutilated, pawn, periodic-table, personality-sphere, queen, rook, satanic, shikato, skeleton, small, spidercow, supermilker, surgery, three-eyes, viper, weeping-angel, world"],
        ];
        out.push_str(&format_table(&cat_headers, &cat_rows));
    }

    if show_effects {
        out.push_str("\n\x1b[1;35m━━━ Forgum Animation Effects (--effect) ━━━\x1b[0m\n");
        let headers = ["Effect", "Type", "Motion & Anatomical Cadence Description"];
        let rows = vec![
            vec!["walk", "Dynamic Movement", "Natural forward leg oscillation with realistic ~2.8s bovine cud chew pacing and peaceful rests"],
            vec!["breathe", "Idle Respiration", "Gentle rhythmic chest and torso expansion and contraction idle breathing cycle"],
            vec!["float", "Levitation", "Zero-gravity sinusoidal hovering and gentle vertical drift across the viewport"],
            vec!["fly", "Aerial Trajectory", "Wing-flapping aerial trajectory sweeping across the horizon and mountain summits"],
            vec!["talk", "Speech Articulation", "Conversational jaw and mouth movement synchronized to speech bubble text"],
            vec!["sway", "Harmonic Pendulum", "Graceful rhythmic pendulum swaying back and forth with inertial dampening"],
            vec!["pulse", "Energy Resonance", "Periodic scale and brightness expansion radiating outwards in harmonic waves"],
            vec!["glitch", "Cyberpunk Distortion", "Digital VHS glitch tearing, horizontal displacement, and electric particle bursts"],
            vec!["particles", "Ambient Emitter", "Atmospheric elemental particles (snowflakes, embers, stars, spores) swirling around mascot"],
            vec!["dissolve", "Phase Transition", "Particle deconstruction and dynamic molecular reassembly sequence"],
        ];
        out.push_str(&format_table(&headers, &rows));
    }

    if show_mountains {
        out.push_str("\n\x1b[1;35m━━━ Procedural Mountains & Horizon (--mountain) ━━━\x1b[0m\n");
        let m_headers = [
            "Mountain Style",
            "Harmonic Ridge Contour",
            "Thematic Environment",
        ];
        let m_rows = vec![
            vec![
                "hills",
                "Gentle sinusoidal harmonic curves",
                "Pastoral verdant rolling hills and meadows",
            ],
            vec![
                "peaks",
                "Sharp multi-octave jagged summits",
                "Alpine mountain range with snow-capped ridges",
            ],
            vec![
                "volcano",
                "Steep caldera with smoking summit",
                "Volcanic wasteland with active smoldering vents",
            ],
            vec![
                "iceberg",
                "Angular crystalline ice cliffs",
                "Polar ocean and frozen glacier ridges",
            ],
            vec![
                "skyline",
                "Rectangular tiered skyscraper silhouette",
                "Metropolitan urban architectural skyline",
            ],
            vec![
                "seamount",
                "Submerged undulating oceanic trenches",
                "Deep marine abyss and hydrothermal mounds",
            ],
            vec![
                "plateau",
                "Flat-topped mesas with sheer vertical drop",
                "Southwestern desert canyons and arid bluffs",
            ],
            vec![
                "crater",
                "Bowl-shaped impact rims and ejecta ridges",
                "Lunar surface and celestial impact terrain",
            ],
            vec![
                "gothic",
                "Spire silhouettes and cathedral buttresses",
                "Gothic necropolis and shadowed spires",
            ],
            vec![
                "castle",
                "Fortress battlements and crenellations",
                "Medieval stone ramparts and watchtowers",
            ],
            vec![
                "garden",
                "Terraced botanical hedges and trellises",
                "Sculpted royal gardens and floral terraces",
            ],
            vec![
                "none",
                "Flat horizon line without elevation",
                "Disables mountain layer",
            ],
        ];
        out.push_str(&format_table(&m_headers, &m_rows));
    }

    if show_roads {
        out.push_str("\n\x1b[1;35m━━━ Ground & Road Surfaces (--road) ━━━\x1b[0m\n");
        let r_headers = ["Road Style", "Surface Texture", "Matching Theme"];
        let r_rows = vec![
            vec![
                "dirt",
                "Pebbles, dust particles, and earthen ruts",
                "Countryside trail and farm meadow",
            ],
            vec![
                "cobblestone",
                "Interlocking medieval stone pavement blocks",
                "Old European town and gothic plaza",
            ],
            vec![
                "magma",
                "Cracked obsidian crust over glowing lava",
                "Volcanic wasteland and infernal abyss",
            ],
            vec![
                "ice",
                "Compact crystalline ice and packed snow",
                "Polar tundra and glacial path",
            ],
            vec![
                "seabed",
                "Sand ripples and deep-sea coral rubble",
                "Ocean floor and aquatic seabed",
            ],
            vec![
                "sidewalk",
                "Paved urban concrete with asphalt curb",
                "Modern city street and sidewalk",
            ],
            vec![
                "roof",
                "Layered ceramic shingles and roof tiles",
                "Rooftop vantage points",
            ],
            vec![
                "grid",
                "Glowing retro-futuristic vector grid",
                "Cyberpunk matrix and virtual reality",
            ],
            vec![
                "crypt",
                "Ancient weathered stone flagstones",
                "Haunted cemetery and catacomb pathway",
            ],
            vec![
                "savanna",
                "Sun-baked arid cracked clay",
                "African savanna and desert plains",
            ],
            vec![
                "mud",
                "Soft squishy mud and wet marsh ruts",
                "Bayou and rain-soaked wetlands",
            ],
            vec![
                "tracks",
                "Parallel steel train tracks with ties",
                "Industrial railroad line",
            ],
            vec![
                "checkerboard",
                "Alternating high-contrast dual tiles",
                "Retro arcade and game board",
            ],
            vec![
                "none",
                "Empty clear baseline without road glyphs",
                "Disables road layer",
            ],
        ];
        out.push_str(&format_table(&r_headers, &r_rows));
    }

    if show_environments {
        out.push_str("\n\x1b[1;35m━━━ Environmental Particle Systems (--environment) ━━━\x1b[0m\n");
        let e_headers = [
            "Environment",
            "Atmospheric Particle Dynamics",
            "Signature Theme",
        ];
        let e_rows = vec![
            vec![
                "pasture",
                "Drifting dandelion seeds and gentle pollen",
                "Peaceful countryside meadow",
            ],
            vec![
                "inferno",
                "Rising volcanic embers, sparks, and smoke",
                "Magma caverns and fire pits",
            ],
            vec![
                "ocean",
                "Rising oxygen bubbles and bioluminescent motes",
                "Deep aquatic ocean waters",
            ],
            vec![
                "arctic",
                "Swirling crystalline snowflakes and frost gusts",
                "Frozen polar blizzard",
            ],
            vec![
                "city",
                "Floating neon dust, light flecks, and smog",
                "Bustling metropolitan night",
            ],
            vec![
                "forest",
                "Falling autumn leaves and whispering spores",
                "Ancient enchanted woodlands",
            ],
            vec![
                "savanna",
                "Heat shimmer mirages and arid dust motes",
                "Sun-drenched savanna plains",
            ],
            vec![
                "swamp",
                "Will-o'-the-wisps and floating marsh vapors",
                "Murky bayou and damp wetlands",
            ],
            vec![
                "space",
                "Slowly drifting distant stars and cosmic nebulae",
                "Infinite stellar cosmos",
            ],
            vec![
                "cyber",
                "Descending matrix code glyphs and data packets",
                "Virtual digital network",
            ],
            vec![
                "graveyard",
                "Swirling spectral wisps and low-lying ground fog",
                "Gothic haunted cemetery",
            ],
            vec![
                "jurassic",
                "Prehistoric fern spores and ambient primeval mist",
                "Primeval dinosaur jungle",
            ],
            vec![
                "hive",
                "Bioluminescent organic pheromone motes",
                "Alien bio-mechanical nest",
            ],
            vec![
                "throne",
                "Golden ceremonial incensed dust particles",
                "Imperial gothic cathedral",
            ],
            vec![
                "none",
                "Clear transparent atmosphere without particles",
                "Disables particle emitter",
            ],
        ];
        out.push_str(&format_table(&e_headers, &e_rows));
    }

    if show_colors {
        out.push_str("\n\x1b[1;35m━━━ Color Modes (--color-mode) ━━━\x1b[0m\n");
        let headers = ["Color Mode", "Color Mapping Algorithm", "Visual Output"];
        let rows = vec![
            vec!["default", "Animal-specific natural wildlife color palette (alias: animal)", "Each creature receives its authentic natural wildlife hue (dragon=fire/gold, whale=ocean azure, cat=amber)"],
            vec!["rainbow", "Refined OKLCH perceptual lightness-uniform chromatic spectrum (alias: lolcat)", "Smooth non-banding perceptual rainbow flowing diagonally across characters in real time"],
            vec!["solid", "Pure single-color monochromatic highlight", "High-contrast focused primary color accentuating ASCII contours cleanly"],
            vec!["none", "Terminal emulator raw default color palette", "Monochrome compatibility output adhering to user's terminal background & foreground"],
        ];
        out.push_str(&format_table(&headers, &rows));
    }

    if show_shells {
        out.push_str("\n\x1b[1;35m━━━ Shell Integration & Completions (forgum completions <shell>) ━━━\x1b[0m\n");
        let s_headers = [
            "Shell",
            "Integration Command",
            "Tab Completion Target File",
            "Configuration Profile",
        ];
        let s_rows = vec![
            vec![
                "bash",
                "forgum init bash",
                "~/.local/share/bash-completion/completions/forgum",
                "~/.bashrc",
            ],
            vec![
                "zsh",
                "forgum init zsh",
                "~/.zsh/completions/_forgum",
                "~/.zshrc",
            ],
            vec![
                "fish",
                "forgum init fish",
                "~/.config/fish/completions/forgum.fish",
                "~/.config/fish/config.fish",
            ],
            vec![
                "pwsh",
                "forgum init pwsh",
                "~/.local/share/powershell/Completions/_forgum.ps1",
                "$PROFILE (PowerShell 7+)",
            ],
            vec![
                "powershell",
                "forgum init powershell",
                "WindowsPowerShell\\Completions\\_forgum.ps1",
                "$PROFILE (Windows PowerShell 5.1)",
            ],
            vec![
                "cmd",
                "forgum init cmd",
                "clink completion script",
                "Command Prompt / Clink / AutoRun",
            ],
            vec![
                "elvish",
                "forgum init elvish",
                "~/.config/elvish/lib/forgum.elv",
                "~/.config/elvish/rc.elv",
            ],
            vec![
                "nushell",
                "forgum init nushell",
                "~/.config/nushell/completions/forgum.nu",
                "config.nu",
            ],
            vec![
                "carapace",
                "forgum init carapace",
                "~/.config/carapace/specs/forgum.yaml",
                "Carapace multi-shell spec",
            ],
            vec![
                "xonsh",
                "forgum init xonsh",
                "~/.config/forgum/completions/forgum.xsh",
                "~/.xonshrc",
            ],
            vec![
                "tcsh",
                "forgum init tcsh",
                "~/.config/forgum/completions/forgum.csh",
                "~/.tcshrc",
            ],
            vec![
                "ksh",
                "forgum init ksh",
                "~/.config/forgum/completions/forgum.ksh",
                "~/.kshrc",
            ],
            vec![
                "ion",
                "forgum init ion",
                "~/.config/forgum/completions/forgum.ion",
                "~/.config/ion/initrc",
            ],
            vec![
                "oil",
                "forgum init oil",
                "~/.config/forgum/completions/forgum.oil",
                "~/.config/oil/oshrc",
            ],
            vec![
                "yash",
                "forgum init yash",
                "~/.config/forgum/completions/forgum.yash",
                "~/.yashrc",
            ],
        ];
        out.push_str(&format_table(&s_headers, &s_rows));
    }

    if show_config {
        out.push_str(
            "\n\x1b[1;35m━━━ Configuration Keys (forgum config set <key> <value>) ━━━\x1b[0m\n",
        );
        let c_headers = [
            "Key",
            "Data Type",
            "Accepted Values / Examples",
            "Description",
        ];
        let c_rows = vec![
            vec![
                "cow",
                "string",
                "default, tux, dragon... (106 mascots)",
                "Mascot animal basename to render",
            ],
            vec![
                "text",
                "string",
                "Custom speech bubble text",
                "Message displayed in animal speech/thought bubble",
            ],
            vec![
                "effect",
                "string",
                "walk, breathe, float, fly, talk, sway...",
                "Animation motion effect",
            ],
            vec![
                "background",
                "boolean",
                "true, false",
                "Enable persistent non-blocking overlay mode",
            ],
            vec![
                "duration",
                "integer",
                "0 (infinite), 5, 10, 60 (seconds)",
                "Playback duration in seconds before exit",
            ],
            vec![
                "fps",
                "integer",
                "10, 30, 60 (range: 1-120)",
                "Target rendering frame rate",
            ],
            vec![
                "eyes",
                "string",
                "oo, $$, @@, xx, ==",
                "Custom eye glyphs on animal mascot",
            ],
            vec![
                "tongue",
                "string",
                "U, V, p, (space)",
                "Custom tongue glyph on animal mascot",
            ],
            vec![
                "default_shell",
                "string",
                "bash, zsh, fish, pwsh, cmd, elvish, nushell",
                "Default shell target for hook generation",
            ],
            vec![
                "auto_render_on_prompt",
                "boolean",
                "true, false",
                "Automatically render cow banner before each prompt",
            ],
            vec![
                "color_mode",
                "string",
                "animal, rainbow, solid, none",
                "Chromatic styling algorithm",
            ],
            vec![
                "shell_attach_mode",
                "string",
                "overlay, banner, split",
                "Shell integration rendering layout mode",
            ],
            vec![
                "environment",
                "string",
                "pasture, inferno, ocean, arctic, space...",
                "Atmospheric particle emitter theme",
            ],
            vec![
                "road",
                "string",
                "dirt, cobblestone, magma, ice, seabed, grid...",
                "Ground and terrain surface style",
            ],
            vec![
                "mountain",
                "string",
                "hills, peaks, volcano, iceberg, skyline...",
                "Procedural mountain horizon style",
            ],
            vec![
                "thought_interval",
                "integer",
                "0 (infinite), 30, 60 (seconds)",
                "Interval to rotate fortunes in background mode",
            ],
            vec![
                "split_scroll",
                "boolean",
                "true, false",
                "Lock terminal scroll margins below banner (DECSTBM)",
            ],
        ];
        out.push_str(&format_table(&c_headers, &c_rows));
    }

    if show_mux {
        out.push_str("\n\x1b[1;35m━━━ Terminal Multiplexer Integration (forgum tmux <subcommand>) ━━━\x1b[0m\n");
        let m_headers = [
            "Multiplexer",
            "Subcommand",
            "Status Line Integration Example",
            "Features",
        ];
        let m_rows = vec![
            vec![
                "tmux",
                "forgum tmux install",
                "set -g status-right '#(forgum status-line)'",
                "DCS escape encapsulation, dynamic status line",
            ],
            vec![
                "zellij",
                "forgum tmux zellij",
                "zellij compact-bar widget",
                "Full truecolor blit, synchronized redraws",
            ],
            vec![
                "wezterm",
                "forgum tmux wezterm",
                "wezterm.on('update-right-status')",
                "GPU accelerated rendering, ANSI pass-through",
            ],
            vec![
                "screen",
                "forgum tmux screen",
                "hardstatus string",
                "Legacy terminal multiplexer fallback",
            ],
        ];
        out.push_str(&format_table(&m_headers, &m_rows));
    }

    if show_eyes {
        out.push_str("\n\x1b[1;35m━━━ Cow Eye Expressions (--eyes) ━━━\x1b[0m\n");
        let eye_headers = ["Eye Glyphs", "Expression Name", "Description"];
        let eye_rows = vec![
            vec!["oo", "Default", "Classic open bovine eyes"],
            vec!["$$", "Greedy", "Capitalist dollar sign eyes"],
            vec!["==", "Paranoid", "Squinting suspicious slits"],
            vec!["**", "Stoned", "Dazed or starry-eyed gaze"],
            vec!["xx", "Dead", "Knocked out cartoon X-eyes"],
            vec!["--", "Tired", "Exhausted half-asleep flat eyelids"],
            vec!["OO", "Wired", "Wide-awake high-caffeine astonishment"],
            vec!["..", "Youthful", "Gentle innocent small dot eyes"],
            vec!["@@", "Hypnotized", "Spiral trance-state spirals"],
            vec![
                "custom",
                "User-Defined",
                "Any 2-character ASCII string (e.g. ^^, ><, --)",
            ],
        ];
        out.push_str(&format_table(&eye_headers, &eye_rows));
    }

    if show_tongue {
        out.push_str("\n\x1b[1;35m━━━ Cow Tongue Expressions (--tongue) ━━━\x1b[0m\n");
        let tongue_headers = ["Tongue Glyph", "Expression Name", "Description"];
        let tongue_rows = vec![
            vec!["U", "Default", "Standard curved cow tongue protruding"],
            vec!["V", "Serpentine", "Forked snake / dragon tongue"],
            vec!["p", "Playful", "Playful side-sticking tongue"],
            vec!["U ", "Drool", "Long extended bovine tongue with drop"],
            vec![" ", "None", "Closed mouth with no tongue visible"],
            vec![
                "custom",
                "User-Defined",
                "Any custom ASCII character string",
            ],
        ];
        out.push_str(&format_table(&tongue_headers, &tongue_rows));
    }

    if show_logs {
        out.push_str(
            "\n\x1b[1;35m━━━ Structured Logging & Bug Diagnostics (forgum logs) ━━━\x1b[0m\n",
        );
        let l_headers = ["Command / Flag", "Action", "Description"];
        let l_rows = vec![
            vec![
                "forgum logs",
                "View Table",
                "Display formatted ANSI table of recent structured log events",
            ],
            vec![
                "forgum logs -w",
                "Stream Live",
                "Stream log events in real time as they are produced (Ctrl+C to exit)",
            ],
            vec![
                "forgum logs -o",
                "Open in Editor",
                "Launch default system text editor (or explorer) on forgum.log",
            ],
            vec![
                "forgum logs -p",
                "Print Paths",
                "Print absolute filesystem paths to log directory, text log, and JSONL store",
            ],
            vec![
                "forgum logs --cat",
                "Dump Raw",
                "Print raw unformatted text log directly to stdout for piping",
            ],
            vec![
                "forgum logs -s <q>",
                "Filter / Grep",
                "Search log entries by case-insensitive keyword or subsystem",
            ],
            vec![
                "forgum logs -D",
                "Bug Radar Triage",
                "Run automated anomaly detection: pinpoints root causes, uprising bugs, and fixes",
            ],
            vec![
                "forgum logs -l <lvl>",
                "Level Filter",
                "Filter by minimum severity: TRACE, DEBUG, INFO, WARN, ERROR",
            ],
            vec![
                "forgum logs -n <N>",
                "Line Limit",
                "Specify number of recent entries to retrieve (default: 25)",
            ],
            vec![
                "forgum logs --json",
                "JSON Output",
                "Export log events as structured JSON Lines",
            ],
            vec![
                "forgum logs --clear",
                "Clean Logs",
                "Truncate existing text and JSONL log files to start fresh",
            ],
            vec![
                "forgum diagnose",
                "Direct Triage",
                "Top-level shortcut to run full diagnostic bug radar and remediation report",
            ],
        ];
        out.push_str(&format_table(&l_headers, &l_rows));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_table_produces_non_empty_table() {
        let headers = ["Name", "Effect", "Description"];
        let rows = vec![
            vec!["dragon", "walk", "Fire-breathing dragon walking over magma"],
            vec!["tux", "walk", "Linux penguin on arctic icebergs"],
        ];
        let table = format_table(&headers, &rows);
        assert!(table.contains("dragon"));
        assert!(table.contains("tux"));
        assert!(table.contains("┌"));
        assert!(table.contains("└"));
    }

    #[test]
    fn render_options_all_contains_categories() {
        let text = render_options("all");
        assert!(text.contains("Animal Mascots"));
        assert!(text.contains("Animation Effects"));
        assert!(text.contains("Mountains"));
        assert!(text.contains("Color Modes"));
        assert!(text.contains("Shell Integration"));
        assert!(text.contains("Configuration Keys"));
        assert!(text.contains("Terminal Multiplexer"));
        assert!(text.contains("Structured Logging & Bug Diagnostics"));
    }

    #[test]
    fn render_options_specific_categories() {
        let text_mtn = render_options("mountains");
        assert!(text_mtn.contains("Procedural Mountains"));
        assert!(!text_mtn.contains("Animal Mascots"));

        let text_cfg = render_options("config");
        assert!(text_cfg.contains("Configuration Keys"));
        assert!(text_cfg.contains("thought_interval"));

        let text_eyes = render_options("eyes");
        assert!(text_eyes.contains("Cow Eye Expressions"));
        assert!(text_eyes.contains("Greedy"));

        let text_tongue = render_options("tongue");
        assert!(text_tongue.contains("Cow Tongue Expressions"));
        assert!(text_tongue.contains("Serpentine"));

        let text_logs = render_options("logs");
        assert!(text_logs.contains("Structured Logging & Bug Diagnostics"));
        assert!(text_logs.contains("forgum logs -w"));
        assert!(text_logs.contains("forgum logs -o"));
        assert!(text_logs.contains("forgum logs -D"));
        assert!(!text_logs.contains("Animal Mascots"));
    }

    #[test]
    fn render_options_unknown_suggests_categories() {
        let text = render_options("foobar");
        assert!(text.contains("Unknown options category 'foobar'"));
        assert!(text.contains("Available categories:"));
        assert!(text.contains("animals"));
        assert!(text.contains("effects"));
        assert!(text.contains("eyes"));
        assert!(text.contains("tongue"));
    }
}
