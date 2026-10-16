//! CLI argument parsing with clap.
//!
//! Replaces the Phase 0 hand-rolled parser with a derive-based clap CLI.
//! The `Args` struct and `build_scene_config` are kept for backward compat
//! with the render loop, but the parse path is now clap-driven.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::init::Shell;
use crate::protocol::SceneConfig;

/// Forgum animation engine — renders cowsay+fortune+lolcat with effects.
#[derive(Debug, Parser)]
#[command(
    name = "forgum",
    version,
    about = "Forgum — cowsay+fortune+lolcat with a Rust ANSI animation engine",
    long_about = None,
    after_help = "Run `forgum init <shell>` to set up shell integration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to config JSON, TOML, or YAML file.
    #[arg(
        long,
        global = true,
        env = "FORGUM_CONFIG",
        value_name = "PATH",
        help = "Path to config file [env: FORGUM_CONFIG]",
        long_help = "Path to custom JSON, TOML, or YAML configuration file. Overrides default platform config paths."
    )]
    pub config: Option<PathBuf>,

    /// Disable animated effects (gentle fades only).
    #[arg(
        long,
        global = true,
        help = "Disable animations; keep gentle fades only (accessibility)",
        long_help = "Disable animated motions and particle effects for accessibility / reduced motion preferences."
    )]
    pub reduce_motion: bool,

    /// Print just the fortune text (no cow art).
    #[arg(
        long,
        global = true,
        help = "Print just the fortune text without ASCII mascot art",
        long_help = "Bypass ASCII cow art generation and print the speech bubble text directly to stdout."
    )]
    pub text_only: bool,

    /// Path to scene JSON file (alternative to stdin).
    #[arg(
        long,
        short = 'f',
        global = true,
        value_name = "FILE",
        help = "Path to scene JSON file (alternative to stdin)",
        long_help = "Load a full scene specification from a JSON file, specifying animal, text, effects, and scenery together."
    )]
    pub file: Option<PathBuf>,

    /// Cow file basename (without .cow). Alias: --animal.
    #[arg(
        long,
        short = 'c',
        visible_alias = "animal",
        value_name = "NAME",
        help = "Animal mascot name (default, tux, dragon...). Run 'forgum list animals'",
        long_help = "Animal mascot name to render (e.g. default, tux, dragon, stegosaurus, bunny, corgi, ghost, elephant, octopus, random). Run 'forgum list animals' to inspect all 106 available mascots."
    )]
    pub cow: Option<String>,

    /// Load mascot directly from an image file (PNG, JPG, BMP, etc.) instead of a .cow file.
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Load mascot from an image file (PNG, JPG, BMP, etc.)",
        long_help = "Convert an image to ASCII art and use it as the mascot instead of a .cow file."
    )]
    pub image: Option<PathBuf>,

    /// Animation mode: static (stationary mascot) or dynamic (motion).
    #[arg(
        long,
        global = true,
        value_name = "MODE",
        help = "Animation mode: 'static' (stationary) or 'dynamic' (motion)",
        long_help = "Animation mode: 'static' anchors the mascot at a fixed location with subtle eye-blink keep-alive; 'dynamic' enables full multi-frame motion and walking cycles."
    )]
    pub animation: Option<String>,

    /// Specific animation type (e.g. animal_natural, walk, breathe, float, etc.).
    #[arg(
        long,
        visible_alias = "anim-type",
        global = true,
        value_name = "TYPE",
        help = "Specific animation type (animal_natural, walk, breathe...). Run 'forgum list effects'",
        long_help = "Specific animation motion to apply ('animal_natural' for creature's signature DNA animation, walk, breathe, float, fly, talk, sway, pulse, glitch, particles, dissolve). Run 'forgum list effects' for detailed physics descriptions."
    )]
    pub animation_type: Option<String>,

    /// Thematic environment particles (e.g. pasture, inferno, ocean, arctic, space, none).
    #[arg(
        long,
        visible_alias = "env",
        visible_alias = "scenery",
        global = true,
        value_name = "ENVIRONMENT",
        help = "Thematic particle environment (pasture, inferno...). Run 'forgum list scenery'",
        long_help = "Atmospheric particle emitter (pasture, inferno, ocean, arctic, city, forest, savanna, swamp, space, cyber, graveyard, jurassic, hive, throne, none). Run 'forgum list scenery' for all environments."
    )]
    pub environment: Option<String>,

    /// Thematic ground/road style (e.g. dirt, cobblestone, magma, ice, seabed, grid, none).
    #[arg(
        long,
        global = true,
        value_name = "ROAD",
        help = "Ground/road terrain style (dirt, cobblestone...). Run 'forgum list scenery'",
        long_help = "Ground and terrain surface style (dirt, cobblestone, magma, ice, seabed, sidewalk, roof, grid, crypt, savanna, mud, tracks, checkerboard, none). Run 'forgum list scenery' for details."
    )]
    pub road: Option<String>,

    /// Thematic mountain/horizon style (e.g. hills, peaks, volcano, iceberg, skyline, none).
    #[arg(
        long,
        global = true,
        value_name = "MOUNTAIN",
        help = "Procedural mountain horizon (hills, peaks...). Run 'forgum list scenery'",
        long_help = "Procedural mountain horizon style (hills, peaks, volcano, iceberg, skyline, seamount, plateau, crater, gothic, castle, garden, none). Uses multi-harmonic procedural generation unique each session. Run 'forgum list scenery' for details."
    )]
    pub mountain: Option<String>,

    /// Color mode (natural, animal, rainbow, solid, none). Defaults to natural.
    #[arg(
        long,
        global = true,
        value_name = "MODE",
        help = "Color styling mode (natural, animal, rainbow, solid, none). Defaults to natural",
        long_help = "Color styling mode: 'natural' / 'animal_natural' (authentic creature colors), 'animal' (alias for natural), 'rainbow' (lolcat chromatic wave), 'solid' (uniform highlight), 'none' (monochrome ASCII). Run 'forgum list colors' for details."
    )]
    pub color_mode: Option<String>,

    /// Custom hex palette (comma-separated hex codes, e.g. "#ff0000,#00ff00").
    #[arg(
        long,
        global = true,
        value_name = "HEX_LIST",
        help = "Custom hex palette gradient (e.g. '#ff007f,#00f0ff'). Run 'forgum list colors'",
        long_help = "Comma-separated list of 24-bit hex colors to interpolate as a smooth custom gradient (e.g. '#ff5555,#50fa7b,#8be9fd'). Run 'forgum list colors' for built-in palettes."
    )]
    pub palette: Option<String>,

    /// Atmospheric time-of-day solar illumination (auto, dawn, day, dusk, night).
    #[arg(
        long = "time-of-day",
        visible_alias = "tod",
        global = true,
        value_name = "TIME_OF_DAY",
        help = "Solar illumination mode (auto, dawn, day, dusk, night)",
        long_help = "Atmospheric time-of-day solar illumination: 'auto' (detect from system clock), 'dawn' (warm pink/gold sunrise), 'day' (crisp golden daylight), 'dusk' (rich amber/purple twilight), or 'night' (cool celestial moonlight)."
    )]
    pub time_of_day: Option<String>,

    /// Seasonal foliage phenology (auto, spring, summer, autumn, winter).
    #[arg(
        long = "season",
        global = true,
        value_name = "SEASON",
        help = "Seasonal foliage phenology (auto, spring, summer, autumn, winter)",
        long_help = "Seasonal foliage phenology: 'auto' (detect from calendar month), 'spring' (lime buds and pink blossom florets), 'summer' (lush chlorophyll greens), 'autumn' (fiery scarlet maples and golden oaks), or 'winter' (bare skeletal boughs with frost and snowy evergreen needles)."
    )]
    pub season: Option<String>,

    /// Interval in seconds for thought rotation in background mode (0 = infinite).
    #[arg(
        long,
        global = true,
        value_name = "SECONDS",
        help = "Interval in seconds to rotate fortunes (default: 60, 0 = infinite)",
        long_help = "Interval in seconds between rotating fortunes and thoughts in persistent background mode. Set to 0 to keep the initial thought indefinitely."
    )]
    pub thought_interval: Option<u32>,

    /// Lock terminal scroll margins below the background animation (DECSTBM).
    #[arg(
        long,
        alias = "split",
        global = true,
        help = "Lock terminal scroll margins below background animation (DECSTBM)",
        long_help = "Configure DECSTBM terminal scroll margins so terminal output scrolls cleanly beneath the persistent animation banner without pushing it off screen."
    )]
    pub split_scroll: bool,

    /// Explicit rows reserved at the top of the terminal for animation canvas.
    #[arg(
        long,
        global = true,
        value_name = "ROWS",
        help = "Reserve top N rows for persistent animation canvas (e.g. 8, 12, 16)",
        long_help = "Explicitly reserve N rows at the top of the terminal for the animation canvas, restricting terminal scrolling and shell interaction to the bottom unreserved area."
    )]
    pub reserve_rows: Option<u16>,

    /// Explicit columns reserved for animation canvas.
    #[arg(
        long,
        global = true,
        value_name = "COLS",
        help = "Reserve top N columns for animation canvas",
        long_help = "Explicitly constrain animation canvas to N columns, leaving excess horizontal space unoccupied."
    )]
    pub reserve_cols: Option<u16>,

    /// Fractional ratio of terminal rows to reserve for animation canvas (0.1..0.8).
    #[arg(
        long,
        global = true,
        value_name = "RATIO",
        help = "Fractional ratio of terminal height reserved for canvas (e.g. 0.35)",
        long_help = "Dynamically scale the reserved canvas height to a fraction of the terminal height (e.g. 0.35 for 35%), adapting smoothly on window resize."
    )]
    pub split_ratio: Option<f32>,

    /// Split adapter mode: 'decstbm', 'native', 'precmd', or 'auto'.
    #[arg(
        long,
        global = true,
        value_name = "MODE",
        help = "Split adapter mode: 'decstbm', 'native', 'precmd', or 'auto'",
        long_help = "Split shell execution adapter mode: 'decstbm' (DECSTBM hardware margins), 'native' (native pane split via tmux/wezterm/wt.exe), 'precmd' (dynamic inline shell prompt redraw fallback), or 'auto' (detect based on terminal emulator)."
    )]
    pub split_mode: Option<String>,

    /// Text inside the speech bubble.
    #[arg(
        long,
        short = 't',
        global = true,
        value_name = "MESSAGE",
        help = "Text inside the speech bubble",
        long_help = "Text message to display inside the speech or thought bubble. Supports multiline strings and responsive word wrapping."
    )]
    pub text: Option<String>,

    /// Render as a thought bubble instead of a speech bubble (cowthink mode).
    #[arg(
        long,
        short = 'T',
        global = true,
        help = "Render thought bubble instead of speech bubble (cowthink mode)",
        long_help = "Render thought bubbles (circles: 'o') instead of speech bubble lines ('\\\\'). Also accessible via the 'forgum think' subcommand."
    )]
    pub think: bool,

    /// Force speech bubble instead of thought bubble (cowsay mode).
    #[arg(
        long,
        short = 'S',
        global = true,
        help = "Force speech bubble instead of thought bubble (cowsay mode)",
        long_help = "Force speech bubble with delimiter lines ('\\\\') instead of thought bubbles ('o'). Overrides configured think preference."
    )]
    pub speech: bool,

    /// Effect name.
    #[arg(
        long,
        short = 'e',
        visible_alias = "fx",
        visible_alias = "anim",
        global = true,
        value_name = "NAME",
        help = "Animation effect name (animal_natural, walk, breathe...). Run 'forgum list effects'",
        long_help = "Animation effect to apply ('animal_natural' for creature's signature DNA animation, walk, breathe, float, fly, talk, sway, pulse, glitch, particles, dissolve). Run 'forgum list effects' for complete options."
    )]
    pub effect: Option<String>,

    /// Universally randomize mascot, scenery biome, effects, and thoughts.
    #[arg(
        long,
        short = 'r',
        global = true,
        num_args = 0..=1,
        default_missing_value = "all",
        value_name = "TARGET",
        help = "Universally randomize mascot, scenery, effects, and thoughts (or 'all', 'mascot', 'scenery', 'fx', 'thought')",
        long_help = "Universally randomize creature mascot, scenery biome/road/mountain, effects/animation, and fortunes on every invocation. Optional target: 'all', 'mascot', 'scenery', 'fx', 'thought'."
    )]
    pub random: Option<String>,

    /// Eye string (e.g. "oo", "$$").
    #[arg(
        long,
        global = true,
        value_name = "GLYPHS",
        help = "Custom eyes string (e.g. 'oo', '$$', '@@', 'xx')",
        long_help = "Custom eye glyphs for the animal mascot (e.g. 'oo', '$$', '@@', 'xx', '==', '--')."
    )]
    pub eyes: Option<String>,

    /// Tongue string (e.g. "U").
    #[arg(
        long,
        global = true,
        value_name = "GLYPH",
        help = "Custom tongue string (e.g. 'U', 'V', 'p')",
        long_help = "Custom tongue glyph for the animal mascot (e.g. 'U', 'V', 'p', ' ')."
    )]
    pub tongue: Option<String>,

    /// Render above prompt as a non-blocking overlay.
    #[arg(
        long,
        short = 'b',
        global = true,
        help = "Run animation in background mode above the shell prompt",
        long_help = "Run the animated scene continuously above your shell prompt. Updates smoothly in the background without blocking terminal input."
    )]
    pub background: bool,

    /// Render inline as an animated banner above the prompt without taking over the screen.
    #[arg(
        long,
        short = 'B',
        global = true,
        help = "Render inline animated banner without taking over screen",
        long_help = "Render an animated banner inline within terminal text flow without switching to alternate screen buffer."
    )]
    pub banner: bool,

    /// Duration in seconds. 0 = infinite (with --background).
    #[arg(
        long,
        short = 'd',
        global = true,
        value_name = "SECONDS",
        help = "Playback duration in seconds (0 = infinite in background)",
        long_help = "Number of seconds to animate before cleanly exiting. Defaults to 0 (infinite) when --background is enabled."
    )]
    pub duration: Option<u32>,

    /// Target FPS.
    #[arg(
        long,
        global = true,
        value_name = "RATE",
        help = "Target rendering frame rate (1-240, default: 30)",
        long_help = "Target frames per second (1-240). Calibrated with sub-millisecond OS multimedia timer resolution, hybrid spin-sleep pacing, and continuous float wave interpolation for 60Hz, 120Hz, 144Hz, and 240Hz high-refresh displays."
    )]
    pub fps: Option<u16>,

    /// Hardware acceleration mode (auto, dedicated, integrated, npu, cpu).
    #[arg(
        long,
        visible_alias = "gpu",
        visible_alias = "acceleration",
        global = true,
        value_name = "MODE",
        help = "Hardware acceleration mode (auto, dedicated, integrated, npu, cpu)",
        long_help = "Hardware compute acceleration mode: 'auto' (automatically prefers dedicated GPU, then integrated graphics, then NPU, then CPU fallback), 'dedicated' (prefer dedicated discrete GPU cores and VRAM), 'integrated' (prefer integrated/internal graphics), 'npu' (prefer neural processing unit), 'cpu' (force CPU compute)."
    )]
    pub accel: Option<String>,

    /// Viewport aspect ratio preset (auto, 21:9, 16:9, 16:10, 4:3, 1:1, narrow).
    #[arg(
        long = "aspect",
        visible_alias = "aspect-ratio",
        global = true,
        value_name = "RATIO",
        help = "Viewport aspect ratio preset (auto, 21:9, 16:9, 16:10, 4:3, 1:1, narrow)",
        long_help = "Target aspect ratio preset: 'auto' (detect from terminal width and height), '21:9' (ultrawide panoramic), '16:9' (widescreen), '16:10' (golden tall ratio), '4:3' (classic standard), '1:1' (square), 'narrow' (vertical mobile/split pane)."
    )]
    pub aspect: Option<String>,

    /// Viewport aspect ratio adaptation mode (preset, dynamic, auto).
    #[arg(
        long = "aspect-mode",
        global = true,
        value_name = "MODE",
        help = "Aspect ratio adaptation mode (preset, dynamic, auto)",
        long_help = "Aspect ratio adaptation strategy: 'preset' (lock to fixed aspect math), 'dynamic' (recalculate peaks and amplitude live on window resize), 'auto' (dynamic on interactive TUI/split mode, preset on static banners)."
    )]
    pub aspect_mode: Option<String>,

    /// List available options (animals, effects, scenery, colors, shells) and exit.
    #[arg(
        short = 'l',
        long = "list",
        visible_alias = "options",
        value_name = "CATEGORY",
        num_args = 0..=1,
        default_missing_value = "all",
        help = "List available options (animals, effects, scenery, colors, shells)",
        long_help = "Open structured responsive tables showing available options for animals, effects, scenery, colors, and shells. Run 'forgum list [category]' for specific categories."
    )]
    pub list: Option<String>,

    /// Background contrast profile: standard, acrylic_glass, high_contrast, vibrant.
    #[arg(
        long = "contrast-mode",
        visible_alias = "contrast",
        global = true,
        value_name = "MODE",
        help = "Background contrast profile (standard, acrylic_glass, high_contrast, vibrant)",
        long_help = "Background contrast mode for transparent / acrylic / glass terminals: 'standard', 'acrylic_glass' (boost luminance & saturation for transparency), 'high_contrast', 'vibrant'."
    )]
    pub contrast_mode: Option<String>,

    /// Color Vision Deficiency (CVD) accessibility mode: none, protanopia, deuteranopia, tritanopia, achromatopsia.
    #[arg(
        long = "colorblind",
        visible_alias = "colorblind-mode",
        visible_alias = "cvd",
        global = true,
        value_name = "MODE",
        help = "Colorblind accessibility mode (none, protanopia, deuteranopia, tritanopia, achromatopsia)",
        long_help = "Color Vision Deficiency (CVD) accessibility mode applying Daltonization or physiological simulation: 'none', 'protanopia' (red-blind), 'deuteranopia' (green-blind), 'tritanopia' (blue-blind), 'achromatopsia' (monochrome), 'protanopia_sim', 'deuteranopia_sim', 'tritanopia_sim'."
    )]
    pub colorblind: Option<String>,

    /// Enable automatic update check notifications (default: true).
    #[arg(
        long = "check-updates",
        visible_alias = "update-check",
        global = true,
        help = "Enable automatic update checking notifications",
        overrides_with = "no_check_updates"
    )]
    pub check_updates: bool,

    /// Disable automatic update notifications on CLI exit.
    #[arg(
        long = "no-check-updates",
        visible_alias = "no-update-check",
        global = true,
        help = "Disable automatic update notifications"
    )]
    pub no_check_updates: bool,

    /// Positional speech message text (cowsay compatibility).
    #[arg(value_name = "MESSAGE", num_args = 0..)]
    pub message: Vec<String>,

    /// (Phase 1) Spawn as daemon.
    #[arg(long, global = true, hide = true)]
    pub daemon: bool,

    /// (Phase 1) Control socket path.
    #[arg(long, global = true, hide = true)]
    pub control_socket: Option<PathBuf>,

    /// (Internal) Marker set by the parent on respawn so the child knows
    /// it is THE daemon (no second fork). See `Args::internal_daemon_runner`
    /// for the rationale. End users should never pass this.
    #[arg(long, global = true, hide = true)]
    pub internal_daemon_runner: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Render a cow (default command).
    Render {
        /// Animal mascot name.
        #[arg(long, short = 'c', visible_alias = "animal", value_name = "NAME")]
        cow: Option<String>,
        /// Optional speech text to display inside the speech bubble.
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Render a thought bubble (cowthink mode).
    Think {
        /// Animal mascot name.
        #[arg(long, short = 'c', visible_alias = "animal", value_name = "NAME")]
        cow: Option<String>,
        /// Optional thought text (defaults to random fortune if omitted).
        #[arg(num_args = 0..)]
        thought: Vec<String>,
    },
    /// Print a random fortune to stdout.
    Fortune,
    /// Launch the interactive terminal UI & studio.
    #[command(alias = "menu", alias = "ui", alias = "studio")]
    Tui {
        /// Initial tab to focus: mascots, scenery, effects, installer, config, studio
        #[arg(default_value = "")]
        tab: String,
    },
    /// First-time celestial setup wizard & shell installer with host diagnostics.
    #[command(alias = "setup", alias = "wizard", alias = "installer")]
    Install {
        /// Force headless installation without interactive terminal UI.
        #[arg(long)]
        headless: bool,
        /// Allow or decline anonymous motivation telemetry: 'allow' (yes) or 'decline' (no).
        #[arg(long, value_name = "CONSENT")]
        telemetry: Option<String>,
    },
    /// Cleanly uninstall Forgum from your environment with user-directed choice.
    #[command(alias = "remove", alias = "deorbit", alias = "uninstaller")]
    Uninstall {
        /// Uninstallation method: 'soft' (keep configs & custom cows) or 'purge' (clean slate).
        #[arg(short = 'm', long, value_name = "METHOD")]
        method: Option<String>,
        /// Automatically confirm uninstallation without interactive prompts.
        #[arg(short = 'y', long, alias = "yes")]
        non_interactive: bool,
        /// Force interactive celestial uninstaller TUI wizard.
        #[arg(long)]
        tui: bool,
    },
    /// Check for updates or upgrade Forgum using the detected package manager.
    #[command(alias = "upgrade")]
    Update {
        /// Only check for updates without modifying the system.
        #[arg(long)]
        check: bool,
        /// Target release channel to update or switch to (stable, alpha, nightly, dev).
        #[arg(long, value_name = "CHANNEL")]
        channel: Option<String>,
        /// Explicit update method / package manager (prompt, auto, zypper, scoop, winget, brew, apt, dnf, pacman, cargo, standalone).
        #[arg(short = 'm', long, value_name = "METHOD")]
        method: Option<String>,
        /// Automatically confirm prompts without interactive package manager menu.
        #[arg(short = 'y', long, alias = "yes", alias = "non-interactive")]
        non_interactive: bool,
        /// Internal non-blocking background check worker.
        #[arg(long, hide = true)]
        background_worker: bool,
    },
    /// Detect and prune duplicate or shadowed Forgum installations across package managers.
    #[command(
        name = "prune",
        alias = "prune-duplicates",
        alias = "prune-shadows",
        alias = "clean-duplicates"
    )]
    Prune {
        /// Automatically confirm pruning without interactive prompts.
        #[arg(short = 'y', long, alias = "yes", alias = "force")]
        yes: bool,

        /// Display duplicate installations that would be removed without actually deleting them.
        #[arg(long, alias = "dry-run")]
        dry_run: bool,
    },
    /// View, switch, or manage release channels (stable, alpha, nightly, dev).
    #[command(alias = "channels")]
    Channel {
        /// Action: 'get', 'set', 'list', 'switch', or target channel name directly.
        #[arg(value_name = "ACTION")]
        action: Option<String>,
        /// Channel name when action is 'set' or 'switch' (stable, alpha, nightly, dev).
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Generate shell integration hooks.
    Init {
        /// Target shell, or 'list' to view supported shells. If omitted, detects current active shell.
        #[arg(value_enum)]
        shell: Option<ShellArg>,
        /// Only print the generated hook (CI validation); identical output to a
        /// normal `init` but explicit about the use-case.
        #[arg(long)]
        check: bool,
        /// Automatically install / attach the hook into the shell's RC profile.
        #[arg(short = 'i', long)]
        install: bool,
        /// Additional render arguments forwarded to the background engine.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        render_args: Vec<String>,
    },
    /// Generate shell completion scripts and attach to shell profile.
    Completions {
        /// Target shell, or 'list' to view supported shells.
        #[arg(value_enum, default_value = "list")]
        shell: ShellArg,
    },
    /// List available options for animals, effects, scenery, colors, or shells in structured tables.
    #[command(name = "list", alias = "options", alias = "ls", alias = "show")]
    List {
        /// Category to inspect: animals, effects, scenery, colors, shells, or all (default: all).
        #[arg(default_value = "all", value_name = "CATEGORY")]
        category: String,
    },
    /// Print 'ok' and exit (for daemon health checks).
    Status,
    /// Diagnose terminal capabilities and configuration.
    Doctor,
    /// Run comprehensive health check on environment, terminal, shell, config, and daemons.
    #[command(name = "checkhealth", alias = "health")]
    Checkhealth {
        /// Output raw JSON report instead of formatted terminal report.
        #[arg(long)]
        json: bool,
    },
    /// View or edit configuration.
    Config {
        /// List all configuration keys, data types, and current values.
        #[arg(short = 'l', long)]
        list: bool,
        /// Open the interactive config menu.
        #[arg(long)]
        tui: bool,
        /// Set a key to a value (headless), or 'list' to view options.
        #[arg(value_name = "KEY")]
        key: Option<String>,
        /// Value for --key or target key for 'set'/'get'.
        #[arg(value_name = "VALUE")]
        value: Option<String>,
        /// Additional value when 'config set <key> <value>' syntax is passed.
        #[arg(value_name = "EXTRA_VALUE")]
        extra_value: Option<String>,
        /// Migrate configuration to another format (json, yaml, toml).
        #[arg(long, value_name = "FORMAT")]
        migrate: Option<String>,
    },
    /// View, query, or clear structured engine logs.
    #[command(name = "logs", alias = "log", alias = "view-logs", alias = "show-logs")]
    Logs {
        /// Number of recent log lines to display.
        #[arg(short = 'n', long, alias = "limit", default_value = "25")]
        lines: usize,
        /// Minimum log level filter (TRACE, DEBUG, INFO, WARN, ERROR).
        #[arg(short = 'l', long)]
        level: Option<String>,
        /// Output raw JSON Lines instead of the formatted ANSI table.
        #[arg(long)]
        json: bool,
        /// Follow / stream live logs in real time.
        #[arg(short = 'w', long, alias = "tail", alias = "watch")]
        follow: bool,
        /// Print the absolute path to the log files and directory.
        #[arg(short = 'p', long)]
        path: bool,
        /// Open the log directory or file in the default system editor/explorer.
        #[arg(short = 'o', long, alias = "edit")]
        open: bool,
        /// Output the raw unformatted text log directly (cat/dump).
        #[arg(long, alias = "cat", alias = "dump")]
        raw: bool,
        /// Filter/search logs by keyword or substring.
        #[arg(
            short = 's',
            short_alias = 'q',
            long,
            alias = "grep",
            alias = "query",
            alias = "search"
        )]
        filter: Option<String>,
        /// Clear/truncate existing log files.
        #[arg(long, alias = "clean")]
        clear: bool,
        /// Run intelligent diagnostic triage to detect uprising bugs, root causes, and developer hints.
        #[arg(
            short = 'D',
            long,
            alias = "bugs",
            alias = "diagnose",
            alias = "triage"
        )]
        diagnose: bool,
    },
    /// Run diagnostic triage across logs and subsystems to pinpoint bugs and resolutions.
    #[command(name = "diagnose", alias = "triage", alias = "bugradar")]
    Diagnose {
        /// Number of recent log entries to analyze (default: 100).
        #[arg(short = 'n', long, alias = "limit", default_value = "100")]
        lines: usize,
        /// Output raw JSON report instead of formatted ANSI terminal report.
        #[arg(long)]
        json: bool,
    },
    /// tmux integration subcommands.
    Tmux {
        #[command(subcommand)]
        sub: TmuxSub,
    },
    /// One-shot status line for tmux status-right or zellij widget.
    StatusLine {
        /// Maximum visible length in characters.
        #[arg(long, default_value = "70")]
        max_len: usize,
        /// Multiplexer widget format: tmux, zellij, or plain.
        #[arg(long, default_value = "plain")]
        mux: String,
    },
    /// Fleet manager: control all running daemons.
    Herd {
        #[command(subcommand)]
        sub: HerdSub,
    },
    /// Theme management.
    Theme {
        #[command(subcommand)]
        sub: ThemeSub,
    },
    /// Run the showcase demo.
    Demo,
    /// Run the 60-second showcase demo reel.
    Showcase,
    /// Remote sync — follow your cow across SSH sessions.
    Remote {
        #[command(subcommand)]
        sub: RemoteSub,
    },
    /// Run a command and render its output in a cow speech bubble.
    Say {
        /// The command to execute.
        #[arg(required = true, num_args = 1..)]
        cmd: Vec<String>,
    },
    /// Time a command and show duration in a cow popup.
    Timer {
        /// The command to time.
        #[arg(required = true, num_args = 1..)]
        cmd: Vec<String>,
    },
    /// ASCII cow jousting battle between two mascots.
    #[command(alias = "arena")]
    Battle {
        /// Name of the first fighter (positional or --name1).
        #[arg(value_name = "FIGHTER1")]
        fighter1: Option<String>,
        /// Name of the second fighter (positional or --name2).
        #[arg(value_name = "FIGHTER2")]
        fighter2: Option<String>,
        /// Name of the first cow (named flag).
        #[arg(long, default_value = "Alice")]
        name1: String,
        /// Name of the second cow (named flag).
        #[arg(long, default_value = "Bob")]
        name2: String,
        /// Headless / non-interactive mode (dumps battle log without animation).
        #[arg(long)]
        headless: bool,
        /// Force battle victor: 1 for fighter1, 2 for fighter2, 0 for random.
        #[arg(long, default_value = "0")]
        winner: u8,
        /// Frame rate for live animation (default 12 for majestic, realistic pacing).
        #[arg(long, default_value = "12")]
        fps: u16,
    },
    /// Interactive Rock Paper Scissors mascot battle (User vs Computer).
    #[command(name = "rps-battle", alias = "rps")]
    RpsBattle {
        /// Name of the player's mascot / fighter.
        #[arg(long, default_value = "Player")]
        player: String,

        /// Name of the computer's mascot / fighter.
        #[arg(long, default_value = "Computer")]
        cpu: String,

        /// Pre-selected player weapon: rock (r), paper (p), or scissors (s).
        #[arg(short = 'c', long, value_name = "WEAPON")]
        choice: Option<String>,

        /// Headless / non-interactive mode (dumps battle log without live prompt).
        #[arg(long)]
        headless: bool,

        /// Frame rate for live joust animation playback (default 12).
        #[arg(long, default_value = "12")]
        fps: u16,
    },
    /// Emergency recovery command to restore terminal cursor, disable raw mode, clear temporary pipes, and exit cleanly.
    #[command(alias = "clean")]
    Sweep,
    /// Stop any running animation, split mode, or background daemon and restore terminal space.
    #[command(
        name = "stop",
        alias = "kill",
        alias = "halt",
        alias = "reset",
        alias = "unreserve",
        alias = "clear",
        alias = "clear-margins",
        alias = "clear_margins"
    )]
    Stop {
        /// Stop all running Forgum animations across all sessions, not just current.
        #[arg(short = 'a', long)]
        all: bool,
        /// Force kill immediately without waiting for graceful socket shutdown.
        #[arg(long)]
        force: bool,
    },
    /// Convert an image to ASCII art or save as a custom cow mascot.
    Image {
        /// Path to the image file.
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Target width in character columns (auto-scales height by default).
        #[arg(short = 'w', long, value_name = "WIDTH")]
        width: Option<usize>,

        /// Target height in character rows.
        #[arg(short = 'H', long, value_name = "HEIGHT")]
        height: Option<usize>,

        /// Color mode: truecolor, ansi256, grayscale, monochrome (plain text).
        #[arg(short = 'C', long, value_name = "MODE", default_value = "truecolor")]
        color: String,

        /// Luminance ramp: standard, detailed, blocks.
        #[arg(long, value_name = "RAMP", default_value = "standard")]
        ramp: String,

        /// Save generated ASCII art as a custom cow mascot (~/.config/forgum/cows/<NAME>.cow).
        #[arg(long, value_name = "NAME")]
        save_cow: Option<String>,

        /// Output file path to write ASCII art to (instead of stdout).
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Invert luminance mapping.
        #[arg(short = 'i', long)]
        invert: bool,
    },
    /// Moo with the mascot! Quick joyful bovine vocalization.
    #[command(alias = "bovine", alias = "cowsay")]
    Moo {
        /// Optional speech text to moo (defaults to 'Moo! 🐮').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Mascot chilling and vibing peacefully in the terminal.
    #[command(alias = "chill", alias = "zen")]
    Vibe {
        /// Optional speech text while vibing (defaults to 'Vibing in the terminal ✨').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Encourage yourself to touch grass with a walk across lush pastures.
    #[command(
        name = "touch-grass",
        alias = "touch_grass",
        alias = "grass",
        alias = "outside"
    )]
    TouchGrass {
        /// Optional speech text (defaults to 'Go outside and touch some grass! 🌱').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Unleash Gigachad terminal energy.
    #[command(alias = "gigachad", alias = "based")]
    Chad {
        /// Optional speech text (defaults to 'Average Terminal Enjoyer 🗿').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Mascot running wildly with unstoppable zoomies!
    #[command(alias = "speed", alias = "fast", alias = "hyper")]
    Zoomies {
        /// Optional speech text (defaults to 'ZOOMIES! 💨⚡').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Yeet the mascot into orbit with high velocity!
    #[command(alias = "launch", alias = "ascend")]
    Yeet {
        /// Optional speech text (defaults to 'YEET! 🚀🌌').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Perform the legendary reverse-glide moonwalk with sliding hooves.
    #[command(alias = "hee-hee", alias = "smooth-criminal")]
    Moonwalk {
        /// Optional speech text (defaults to 'Hee-Hee! 🕺🎩').
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Display beautiful terminal system telemetry with animated mascot (fastfetch/neofetch replacement).
    #[command(
        alias = "sysinfo",
        alias = "info",
        alias = "neofetch",
        alias = "fastfetch"
    )]
    Fetch {
        /// Force single-frame instantaneous terminal output (default).
        #[arg(long, short = 'i')]
        instant: bool,

        /// Live interactive 240Hz animation mode beside real-time Braille hardware metrics.
        #[arg(long, short = 'l')]
        live: bool,

        /// Specific mascot name to display (defaults to default cow or random).
        #[arg(short = 'c', long, visible_alias = "animal", value_name = "NAME")]
        cow: Option<String>,

        /// Color palette name to apply.
        #[arg(short = 'p', long, value_name = "PALETTE")]
        palette: Option<String>,
    },
    /// Reactive Living Pasture shell telemetry bus integration.
    #[command(name = "telemetry", alias = "pasture")]
    Telemetry {
        #[command(subcommand)]
        sub: TelemetrySub,
    },
    /// Print version and build information.
    Version,
}

#[derive(Debug, Subcommand)]
pub enum TmuxSub {
    /// Print tmux config block to stdout.
    Install,
    /// Print zellij config block to stdout.
    Zellij,
    /// Print wezterm config block to stdout.
    WezTerm,
    /// Print screen config block to stdout.
    Screen,
    /// List supported multiplexer integration options.
    List,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum TelemetrySub {
    /// Report shell command execution telemetry (exit code, elapsed ms, command name).
    Report {
        /// Last command exit code (0 = success, nonzero = failure).
        #[arg(short = 'x', long, default_value = "0")]
        exit_code: i32,

        /// Execution duration in milliseconds.
        #[arg(short = 'm', long, alias = "time", default_value = "0")]
        elapsed_ms: u64,

        /// Command string executed.
        #[arg(short = 'c', long)]
        command: Option<String>,

        /// Terminal session identifier (defaults to current session).
        #[arg(short = 's', long)]
        session: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum HerdSub {
    /// List all running daemons.
    List,
    /// Stop all (or filtered) daemons.
    Stop {
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Set effect on all (or filtered) daemons.
    Effect {
        /// Effect name.
        name: String,
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Set cow/animal model on all (or filtered) daemons.
    Cow {
        /// Cow name or alias.
        name: String,
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Set eyes glyph on all (or filtered) daemons.
    Eyes {
        /// Eyes string (e.g. 'oo', '^^', '$$', '..').
        eyes: String,
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Set tongue glyph on all (or filtered) daemons.
    Tongue {
        /// Tongue string (e.g. 'U ', '||').
        tongue: String,
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Set color mode on all (or filtered) daemons.
    Color {
        /// Color mode (e.g. 'rainbow', 'aurora', 'matrix', 'fire', 'pastel').
        mode: String,
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Set speed on all (or filtered) daemons.
    Speed {
        /// Speed multiplier.
        value: f32,
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Pause all (or filtered) daemons.
    Pause {
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Resume all (or filtered) daemons.
    Resume {
        /// Filter by session ID.
        #[arg(long)]
        session: Option<String>,
        /// Apply to all daemons.
        #[arg(long)]
        all: bool,
    },
    /// Drop all daemons to idle (low FPS).
    Quiet,
    /// Only the focused pane animates; others idle.
    Follow {
        /// Pane to keep active (e.g. "%3"). Defaults to current pane.
        #[arg(long)]
        pane: Option<String>,
    },
    /// Health check: list all daemons and their status.
    Census,
}

#[derive(Debug, Clone, Subcommand)]
pub enum RemoteSub {
    /// Attach this terminal to a remote daemon via SSH.
    Attach {
        /// Remote host to attach to.
        host: String,
    },
    /// Broadcast effect changes to all peers in a sync session.
    Sync {
        /// Sync session ID (auto-detected if not provided).
        #[arg(long)]
        session_id: Option<String>,
    },
    /// List active remote peers.
    Who,
    /// List remote synchronization options and active peers.
    List,
}

#[derive(Debug, Subcommand)]
pub enum ThemeSub {
    /// List available themes.
    List,
    /// Apply a theme to all daemons.
    Apply {
        /// Theme name.
        name: String,
    },
    /// Cycle through themes every N minutes.
    Rotate {
        /// Interval in minutes between theme changes.
        #[arg(long, default_value = "5")]
        interval: u32,
    },
    /// Show and apply the current seasonal theme.
    Seasonal,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellArg {
    Bash,
    Zsh,
    Fish,
    Pwsh,
    Cmd,
    #[value(name = "powershell", alias = "power-shell", alias = "posh")]
    PowerShell,
    Elvish,
    Nushell,
    Carapace,
    Xonsh,
    Tcsh,
    Ksh,
    Ion,
    Oil,
    Yash,
    /// List all supported shells and their configuration paths.
    List,
}

impl From<ShellArg> for Shell {
    fn from(arg: ShellArg) -> Self {
        match arg {
            ShellArg::Bash | ShellArg::List => Shell::Bash,
            ShellArg::Zsh => Shell::Zsh,
            ShellArg::Fish => Shell::Fish,
            ShellArg::Pwsh => Shell::Pwsh,
            ShellArg::Cmd => Shell::Cmd,
            ShellArg::PowerShell => Shell::PowerShell,
            ShellArg::Elvish => Shell::Elvish,
            ShellArg::Nushell => Shell::Nushell,
            ShellArg::Carapace => Shell::Carapace,
            ShellArg::Xonsh => Shell::Xonsh,
            ShellArg::Tcsh => Shell::Tcsh,
            ShellArg::Ksh => Shell::Ksh,
            ShellArg::Ion => Shell::Ion,
            ShellArg::Oil => Shell::Oil,
            ShellArg::Yash => Shell::Yash,
        }
    }
}

/// Backward-compatible command enum (used by main.rs match).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Command {
    #[default]
    Render,
    Think,
    Status,
    Config,
    Tui,
    Fortune,
    Init,
    Completions,
    Options,
    List,
    Tmux,
    StatusLine,
    Herd,
    Theme,
    Demo,
    Showcase,
    Remote,
    Say,
    Timer,
    Battle,
    RpsBattle,
    Doctor,
    Checkhealth,
    Logs,
    Diagnose,
    Install,
    Uninstall,
    Update,
    Prune,
    Channel,
    Sweep,
    Stop,
    Image,
    Moo,
    Vibe,
    TouchGrass,
    Chad,
    Zoomies,
    Yeet,
    Moonwalk,
    Fetch,
    Telemetry,
    Version,
    Unknown(String),
}

/// Parsed arguments (backward-compatible with Phase 0).
#[derive(Debug, Clone, Default)]
pub struct Args {
    pub command: Command,
    pub file: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub background: bool,
    pub banner: bool,
    pub duration: Option<u32>,
    pub fps: Option<u16>,
    pub cow: Option<String>,
    pub image: Option<PathBuf>,
    pub animation: Option<String>,
    pub animation_type: Option<String>,
    pub environment: Option<String>,
    pub road: Option<String>,
    pub mountain: Option<String>,
    pub color_mode: Option<String>,
    pub palette: Option<String>,
    pub time_of_day: Option<String>,
    pub season: Option<String>,
    pub thought_interval: Option<u32>,
    pub split_scroll: bool,
    pub reserve_rows: Option<u16>,
    pub reserve_cols: Option<u16>,
    pub split_ratio: Option<f32>,
    pub split_mode: Option<String>,
    pub text: Option<String>,
    pub effect: Option<String>,
    pub eyes: Option<String>,
    pub tongue: Option<String>,
    pub daemon: bool,
    pub control_socket: Option<PathBuf>,
    /// (Internal) Marker set by the parent on respawn so the child knows
    /// it is THE daemon (no second fork). See `Args::internal_daemon_runner`
    /// for the rationale. End users should never pass this.
    pub internal_daemon_runner: bool,
    pub max_len: Option<usize>,
    pub reduce_motion: bool,
    pub text_only: bool,
    pub think: bool,
    pub speech: bool,
    pub list: Option<String>,
    pub random: Option<String>,
    pub accel: Option<String>,
    pub aspect: Option<String>,
    pub aspect_mode: Option<String>,
    pub contrast_mode: Option<String>,
    pub colorblind: Option<String>,
    pub check_updates: bool,
    pub no_check_updates: bool,
}

/// Error type returned by `parse_args`, carrying the intended process exit code.
#[derive(Debug)]
pub struct CliError {
    pub exit_code: u8,
    pub message: String,
}

/// Normalize CLI tokens so subcommands without '--' (e.g. `forgum cow tux`, `forgum cows`,
/// Normalize command-line arguments so subcommands and options passed without '--' (e.g.
/// `forgum cow tux`, `forgum effect walk`, `forgum fps 60`, `forgum duration 5`)
/// are cleanly mapped to option flags while preserving subcommand values and typo suggestions.
pub fn normalize_argv(argv: &[String]) -> Vec<String> {
    if argv.is_empty() {
        return Vec::new();
    }
    let mut normalized = Vec::with_capacity(argv.len() + 4);
    normalized.push(argv[0].clone());

    let mut i = 1;
    while i < argv.len() {
        let token = &argv[i];
        let token_lower = token.to_ascii_lowercase();

        // If the preceding token was an option or command expecting an argument, keep this token as-is!
        if let Some(prev) = normalized.last() {
            if prev == "list" || prev == "--list" || prev == "-l" || prev.starts_with('-') {
                normalized.push(token.clone());
                i += 1;
                continue;
            }
        }

        // Keyword followed by value without '--'
        if !token.starts_with('-') {
            let next_val = argv.get(i + 1);
            let has_next = next_val.is_some() && !next_val.unwrap().starts_with('-');

            if has_next {
                let val = next_val.unwrap().clone();
                match token_lower.as_str() {
                    "cow" | "animal" | "mascot" => {
                        normalized.push("--cow".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "effect" | "fx" | "anim" | "animation" => {
                        normalized.push("--effect".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "scenery" | "env" | "environment" => {
                        normalized.push("--environment".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "mountain" | "mountains" => {
                        normalized.push("--mountain".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "road" | "roads" => {
                        normalized.push("--road".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "color" | "color-mode" => {
                        normalized.push("--color-mode".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "palette" => {
                        normalized.push("--palette".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "fps" => {
                        normalized.push("--fps".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "duration" => {
                        normalized.push("--duration".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "accel" => {
                        normalized.push("--accel".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "aspect" => {
                        normalized.push("--aspect".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    "contrast" => {
                        normalized.push("--contrast-mode".to_string());
                        normalized.push(val);
                        i += 2;
                        continue;
                    }
                    _ => {}
                }
            }
        }

        normalized.push(token.clone());
        i += 1;
    }

    normalized
}

/// Parse CLI args (backward-compatible wrapper around clap).
pub fn parse_args(argv: Vec<String>) -> Result<(Args, Option<Commands>), CliError> {
    if argv.len() > 1 && argv[1].eq_ignore_ascii_case("help") {
        let cat = argv.get(2).map(|s| s.as_str());
        return Err(CliError {
            exit_code: 0,
            message: crate::options_table::render_help(cat),
        });
    }

    let normalized_argv = normalize_argv(&argv);

    let cli = match Cli::try_parse_from(&normalized_argv) {
        Ok(c) => c,
        Err(e) => {
            // Intercept clap help requests and display the responsive categorized table
            match e.kind() {
                clap::error::ErrorKind::DisplayHelp
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    return Err(CliError {
                        exit_code: 0,
                        message: crate::options_table::render_help(None),
                    });
                }
                clap::error::ErrorKind::DisplayVersion => {
                    return Err(CliError {
                        exit_code: 0,
                        message: e.to_string(),
                    });
                }
                _ => {}
            }

            let exit_code = 64;
            match e.kind() {
                clap::error::ErrorKind::InvalidSubcommand => {
                    let invalid_sub = normalized_argv
                        .iter()
                        .skip(1)
                        .find(|arg| !arg.starts_with('-'))
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    let sub_list = crate::completions::all_subcommands();
                    let sub_names: Vec<&str> = sub_list.iter().map(|(n, _)| *n).collect();
                    let closest = find_closest_match(invalid_sub, &sub_names);
                    let message = crate::completions::format_unknown_subcommand_error(
                        invalid_sub,
                        closest,
                        &sub_list,
                    );
                    return Err(CliError { exit_code, message });
                }
                clap::error::ErrorKind::UnknownArgument => {
                    let invalid_flag = normalized_argv
                        .iter()
                        .skip(1)
                        .find(|arg| arg.starts_with('-'))
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    let subcmd_opt = normalized_argv.iter().skip(1).find(|arg| {
                        !arg.starts_with('-') && SUBCOMMAND_CANDIDATES.contains(&arg.as_str())
                    });
                    if let Some(subcmd) = subcmd_opt {
                        if let Some(valid_flags) = crate::completions::flags_for_subcommand(subcmd)
                        {
                            let mut flag_names: Vec<&str> =
                                valid_flags.iter().map(|(f, _)| *f).collect();
                            if let Some(metas) =
                                crate::completions::flag_metas_for_subcommand(subcmd)
                            {
                                for f in metas {
                                    for alias in f.aliases {
                                        if alias.starts_with("--") {
                                            flag_names.push(alias);
                                        }
                                    }
                                }
                            }
                            let closest = find_closest_match(invalid_flag, &flag_names);
                            let message = crate::completions::format_unknown_subcommand_flag_error(
                                subcmd,
                                invalid_flag,
                                closest,
                                &valid_flags,
                            );
                            return Err(CliError { exit_code, message });
                        }
                    }
                    let root_flags = crate::completions::all_root_flags();
                    let mut flag_names: Vec<&str> = root_flags.iter().map(|(f, _)| *f).collect();
                    for f in crate::completions::ROOT_FLAGS_METADATA {
                        for alias in f.aliases {
                            if alias.starts_with("--") {
                                flag_names.push(alias);
                            }
                        }
                    }
                    let closest = find_closest_match(invalid_flag, &flag_names);
                    let message = crate::completions::format_unknown_root_flag_error(
                        invalid_flag,
                        closest,
                        &root_flags,
                    );
                    return Err(CliError { exit_code, message });
                }
                _ => {
                    let mut message = e.to_string();
                    for arg in argv.iter().skip(1) {
                        if let Some(suggestion) = generate_command_suggestion(arg) {
                            message.push_str("\n\n");
                            message.push_str(&suggestion);
                            break;
                        }
                    }
                    return Err(CliError { exit_code, message });
                }
            }
        }
    };

    // Validate animation static vs dynamic consistency
    if cli.animation.as_deref() == Some("static") {
        if let Some(at) = &cli.animation_type {
            if at != "static" && at != "random" {
                return Err(CliError {
                    exit_code: 64,
                    message: format!(
                        "Cannot specify dynamic animation type '{at}' when animation is static"
                    ),
                });
            }
        }
    }
    if cli.animation.as_deref() == Some("dynamic") {
        if let Some(at) = &cli.animation_type {
            if at == "static" {
                return Err(CliError {
                    exit_code: 64,
                    message: "Cannot specify animation type 'static' when animation is dynamic"
                        .to_string(),
                });
            }
        }
    }

    let mut sub_cow = None;
    let (command, extra_text, is_think) = match &cli.command {
        Some(Commands::Render { message, cow }) => {
            if let Some(c) = cow {
                sub_cow = Some(c.clone());
            }
            let extra = if message.is_empty() {
                None
            } else {
                Some(message.join(" "))
            };
            if cli.list.is_some() {
                (Command::List, extra, false)
            } else {
                (Command::Render, extra, false)
            }
        }
        None => {
            if !cli.message.is_empty() {
                if let Some(suggestion) = is_command_typo(&cli.message[0]) {
                    return Err(CliError {
                        exit_code: 64,
                        message: format!(
                            "Unrecognized command or option '{}'.\n\n{}",
                            cli.message[0], suggestion
                        ),
                    });
                }
            }
            let extra = if cli.message.is_empty() {
                None
            } else {
                Some(cli.message.join(" "))
            };
            if cli.list.is_some() {
                (Command::List, extra, false)
            } else {
                (Command::Render, extra, false)
            }
        }
        Some(Commands::Think { thought, cow }) => {
            if let Some(c) = cow {
                sub_cow = Some(c.clone());
            }
            let t = if thought.is_empty() {
                None
            } else {
                Some(thought.join(" "))
            };
            (Command::Think, t, true)
        }
        Some(Commands::Fortune) => (Command::Fortune, None, false),
        Some(Commands::Tui { .. }) => (Command::Tui, None, false),
        Some(Commands::Init { .. }) => (Command::Init, None, false),
        Some(Commands::Completions { .. }) => (Command::Completions, None, false),
        Some(Commands::Herd { .. }) => (Command::Herd, None, false),
        Some(Commands::Theme { .. }) => (Command::Theme, None, false),
        Some(Commands::Showcase) => (Command::Showcase, None, false),
        Some(Commands::Demo) => (Command::Demo, None, false),
        Some(Commands::Remote { .. }) => (Command::Remote, None, false),
        Some(Commands::StatusLine { .. }) => (Command::StatusLine, None, false),
        Some(Commands::Status) => (Command::Status, None, false),
        Some(Commands::Say { .. }) => (Command::Say, None, false),
        Some(Commands::Timer { .. }) => (Command::Timer, None, false),
        Some(Commands::Battle { .. }) => (Command::Battle, None, false),
        Some(Commands::RpsBattle { .. }) => (Command::RpsBattle, None, false),
        Some(Commands::Doctor) => (Command::Doctor, None, false),
        Some(Commands::Checkhealth { .. }) => (Command::Checkhealth, None, false),
        Some(Commands::Logs { .. }) => (Command::Logs, None, false),
        Some(Commands::Diagnose { .. }) => (Command::Diagnose, None, false),
        Some(Commands::List { .. }) => (Command::List, None, false),
        Some(Commands::Install { .. }) => (Command::Install, None, false),
        Some(Commands::Uninstall { .. }) => (Command::Uninstall, None, false),
        Some(Commands::Update { .. }) => (Command::Update, None, false),
        Some(Commands::Channel { .. }) => (Command::Channel, None, false),
        Some(Commands::Sweep) => (Command::Sweep, None, false),
        Some(Commands::Stop { .. }) => (Command::Stop, None, false),
        Some(Commands::Image { .. }) => (Command::Image, None, false),
        Some(Commands::Config { .. }) => (Command::Config, None, false),
        Some(Commands::Tmux { .. }) => (Command::Tmux, None, false),
        Some(Commands::Moo { message }) => {
            let txt = if message.is_empty() {
                Some("Moo! 🐮".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::Moo, txt, false)
        }
        Some(Commands::Vibe { message }) => {
            let txt = if message.is_empty() {
                Some("Vibing in the terminal ✨".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::Vibe, txt, false)
        }
        Some(Commands::TouchGrass { message }) => {
            let txt = if message.is_empty() {
                Some("Go outside and touch some grass! 🌱".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::TouchGrass, txt, false)
        }
        Some(Commands::Chad { message }) => {
            let txt = if message.is_empty() {
                Some("Average Terminal Enjoyer 🗿".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::Chad, txt, false)
        }
        Some(Commands::Zoomies { message }) => {
            let txt = if message.is_empty() {
                Some("ZOOMIES! 💨⚡".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::Zoomies, txt, false)
        }
        Some(Commands::Yeet { message }) => {
            let txt = if message.is_empty() {
                Some("YEET! 🚀🌌".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::Yeet, txt, false)
        }
        Some(Commands::Moonwalk { message }) => {
            let txt = if message.is_empty() {
                Some("Hee-Hee! 🕺🎩".to_string())
            } else {
                Some(message.join(" "))
            };
            (Command::Moonwalk, txt, false)
        }
        Some(Commands::Fetch { .. }) => (Command::Fetch, None, false),
        Some(Commands::Telemetry { .. }) => (Command::Telemetry, None, false),
        Some(Commands::Prune { .. }) => (Command::Prune, None, false),
        Some(Commands::Version) => (Command::Version, None, false),
    };

    validate_cli_values(&cli, &cli.command)?;

    let max_len = match &cli.command {
        Some(Commands::StatusLine { max_len, .. }) => Some(*max_len),
        _ => None,
    };

    let text = cli.text.or(extra_text);
    let think = cli.think || is_think;
    let cow = cli.cow.or(sub_cow);
    let sub_list = match &cli.command {
        Some(Commands::List { category }) => Some(category.clone()),
        _ => None,
    };
    let list = cli.list.or(sub_list);

    let args = Args {
        command,
        file: cli.file,
        config: cli.config,
        background: cli.background,
        banner: cli.banner,
        duration: cli.duration,
        fps: cli.fps,
        cow,
        image: cli.image,
        animation: cli.animation,
        animation_type: cli.animation_type,
        environment: cli.environment,
        road: cli.road,
        mountain: cli.mountain,
        color_mode: cli.color_mode,
        palette: cli.palette,
        time_of_day: cli.time_of_day,
        season: cli.season,
        thought_interval: cli.thought_interval,
        split_scroll: cli.split_scroll,
        reserve_rows: cli.reserve_rows,
        reserve_cols: cli.reserve_cols,
        split_ratio: cli.split_ratio,
        split_mode: cli.split_mode,
        text,
        effect: cli.effect,
        eyes: cli.eyes,
        tongue: cli.tongue,
        daemon: cli.daemon,
        control_socket: cli.control_socket,
        internal_daemon_runner: cli.internal_daemon_runner,
        max_len,
        reduce_motion: cli.reduce_motion,
        text_only: cli.text_only,
        think,
        speech: cli.speech,
        list,
        random: cli.random,
        accel: cli.accel,
        aspect: cli.aspect,
        aspect_mode: cli.aspect_mode,
        contrast_mode: cli.contrast_mode,
        colorblind: cli.colorblind,
        check_updates: cli.check_updates,
        no_check_updates: cli.no_check_updates,
    };

    Ok((args, cli.command))
}

/// Build the final `SceneConfig` from `Args` and config file.
/// Precedence: CLI > --file > --config > defaults.
pub fn build_scene_config(args: &Args) -> Result<SceneConfig, String> {
    let mut cfg = if let Some(path) = &args.config {
        match crate::config::read_config_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[forgum] Warning: could not parse config file ({}): {e}. Falling back to default configuration.", path.display());
                SceneConfig::default()
            }
        }
    } else {
        // Try auto-discovering config from platform default path.
        match forgum_platform::config_path() {
            Ok(default_path) => crate::config::read_config_file(&default_path).unwrap_or_default(),
            Err(_) => SceneConfig::default(),
        }
    };

    if let Some(path) = &args.file {
        let overlay = match crate::config::read_config_file(path) {
            Ok(o) => o,
            Err(e) => {
                eprintln!(
                    "[forgum] Warning: could not parse scene overlay file ({}): {e}. Ignoring.",
                    path.display()
                );
                SceneConfig::default()
            }
        };
        cfg = crate::config::merge(cfg, overlay);
    }

    // CLI overrides.
    if let Some(c) = &args.cow {
        cfg.cow = c.clone();
    }

    // Auto-apply animal profile defaults if custom cow was provided or configured
    let cli_cow_passed = args.cow.is_some();
    if cli_cow_passed || cfg.cow != "default" {
        let animal_name = args.cow.as_deref().unwrap_or(&cfg.cow);
        let profile = crate::scenery::get_animal_profile(animal_name);

        if args.environment.is_none() && (cli_cow_passed || cfg.environment.is_none()) {
            cfg.environment = Some(profile.environment.as_str().to_string());
        }
        if args.road.is_none() && (cli_cow_passed || cfg.road.is_none()) {
            cfg.road = Some(profile.road.as_str().to_string());
        }
        if args.mountain.is_none() && (cli_cow_passed || cfg.mountain.is_none()) {
            cfg.mountain = Some(profile.mountain.as_str().to_string());
        }
        if args.animation_type.is_none()
            && args.effect.is_none()
            && (cli_cow_passed
                || cfg.effect == "default"
                || cfg.effect == "static"
                || cfg.effect == "animal_natural"
                || cfg.effect == "natural"
                || cfg.effect.is_empty())
        {
            cfg.animation_type = Some(profile.base_anim.as_str().to_string());
            cfg.effect = profile.base_anim.as_str().to_string();
        }
        if args.palette.is_none() && (cli_cow_passed || cfg.palette.is_none()) {
            let natural_hexes = crate::color::get_natural_hex_palette(animal_name);
            if !natural_hexes.is_empty() {
                cfg.palette = Some(natural_hexes.join(","));
            } else if !profile.wildlife_palette.is_empty() {
                cfg.palette = Some(profile.wildlife_palette.join(","));
            }
        }
        if args.eyes.is_none() && (cli_cow_passed || cfg.eyes.is_empty()) {
            cfg.eyes = profile.eyes.to_string();
        }
        if args.tongue.is_none() && (cli_cow_passed || cfg.tongue.is_empty()) {
            cfg.tongue = profile.tongue.to_string();
        }
    }

    // Dynamic scenario adaptation when environment is passed or configured
    let env_to_adapt = args.environment.clone().or_else(|| {
        if !cli_cow_passed {
            cfg.environment.clone()
        } else {
            None
        }
    });
    if let Some(ref e) = env_to_adapt {
        cfg.environment = Some(e.clone());
        let env_style = crate::scenery::EnvironmentStyle::parse(e);
        let (dyn_mtn, dyn_road) = crate::scenery::environment_scenery_defaults(env_style);
        if args.mountain.is_none() && (args.environment.is_some() || cfg.mountain.is_none()) {
            cfg.mountain = Some(dyn_mtn.as_str().to_string());
        }
        if args.road.is_none() && (args.environment.is_some() || cfg.road.is_none()) {
            cfg.road = Some(dyn_road.as_str().to_string());
        }
        if args.animation_type.is_none()
            && (env_style == crate::scenery::EnvironmentStyle::Ocean
                || env_style == crate::scenery::EnvironmentStyle::Space)
        {
            let animal_name = args.cow.as_deref().unwrap_or(&cfg.cow);
            let profile = crate::scenery::get_animal_profile(animal_name);
            if profile.base_anim == crate::dna::BaseAnim::Walk {
                cfg.animation_type = Some("float".to_string());
                cfg.effect = "float".to_string();
            }
        }
    }

    if let Some(a) = &args.animation {
        cfg.animation = Some(a.clone());
    }
    if let Some(at) = &args.animation_type {
        cfg.animation_type = Some(at.clone());
        cfg.effect = at.clone();
    }
    if let Some(r) = &args.road {
        cfg.road = Some(r.clone());
    }
    if let Some(m) = &args.mountain {
        cfg.mountain = Some(m.clone());
    }
    if let Some(e) = &args.effect {
        cfg.effect = e.clone();
    }
    if let Some(cm) = &args.color_mode {
        cfg.color_mode = cm.clone();
    }
    if let Some(p) = &args.palette {
        cfg.palette = Some(p.clone());
    }
    if let Some(tod) = &args.time_of_day {
        cfg.time_of_day = Some(tod.clone());
    }
    if let Some(s) = &args.season {
        cfg.season = Some(s.clone());
    }
    if let Some(r) = &args.random {
        cfg.random = Some(forgum_platform::protocol::RandomSetting::String(r.clone()));
    }

    // Normalize color_mode aliases to "natural"
    if cfg.color_mode == "default"
        || cfg.color_mode == "animal"
        || cfg.color_mode == "animal_natural"
    {
        cfg.color_mode = "natural".to_string();
    }

    // If explicit CLI palette was provided, switch color_mode to "custom" unless an explicit color_mode was given
    if args.palette.is_some() && args.color_mode.is_none() {
        cfg.color_mode = "custom".to_string();
    }

    // In "natural" mode, resolve mascot biological palette if no palette is configured
    if !args.text_only && cfg.color_mode == "natural" && cfg.palette.is_none() {
        let animal_name = args.cow.as_deref().unwrap_or(&cfg.cow);
        let natural_hexes = crate::color::get_natural_hex_palette(animal_name);
        if !natural_hexes.is_empty() {
            cfg.palette = Some(natural_hexes.join(","));
        }
    }

    // If effect or animation_type is animal_natural / natural, ensure animation_type reflects mascot signature DNA
    if cfg.effect == "animal_natural"
        || cfg.effect == "natural"
        || cfg.animation_type.as_deref() == Some("animal_natural")
        || cfg.animation_type.as_deref() == Some("natural")
    {
        let animal_name = args.cow.as_deref().unwrap_or(&cfg.cow);
        let profile = crate::scenery::get_animal_profile(animal_name);
        if cfg.animation_type.is_none()
            || cfg.animation_type.as_deref() == Some("animal_natural")
            || cfg.animation_type.as_deref() == Some("natural")
        {
            cfg.animation_type = Some(profile.base_anim.as_str().to_string());
        }
    }
    if let Some(ti) = args.thought_interval {
        cfg.thought_interval = ti;
    }
    if args.split_scroll {
        cfg.split_scroll = true;
    }
    if let Some(rr) = args.reserve_rows {
        cfg.reserve_rows = Some(rr);
        cfg.split_scroll = true;
    }
    if let Some(rc) = args.reserve_cols {
        cfg.reserve_cols = Some(rc);
    }
    if let Some(sr) = args.split_ratio {
        cfg.split_ratio = Some(sr);
        cfg.split_scroll = true;
    }
    if let Some(sm) = &args.split_mode {
        cfg.split_mode = Some(sm.clone());
        if sm == "decstbm" {
            cfg.split_scroll = true;
        }
    }
    if let Some(t) = &args.text {
        cfg.text = t.clone();
    }
    if let Some(eyes) = &args.eyes {
        cfg.eyes = eyes.clone();
    }
    if let Some(tongue) = &args.tongue {
        cfg.tongue = tongue.clone();
    }
    if args.background {
        cfg.background = true;
    }
    if let Some(d) = args.duration {
        cfg.duration = d;
    }
    if let Some(f) = args.fps {
        cfg.fps = f;
    }
    if args.think {
        cfg.think = true;
    }
    if args.speech {
        cfg.think = false;
    }
    if args.banner {
        cfg.banner = true;
        cfg.shell_attach_mode = "banner".to_string();
    } else if cfg.banner {
        cfg.shell_attach_mode = "banner".to_string();
    }
    if args.reduce_motion {
        cfg.reduce_motion = true;
    }
    if args.text_only {
        cfg.text_only = true;
    }
    let reduce_motion_applies = args.reduce_motion
        || (cfg.reduce_motion && args.effect.is_none() && args.animation_type.is_none());

    if reduce_motion_applies
        && cfg.animation.as_deref() == Some("dynamic")
        && cfg.animation_type.as_deref() != Some("breathe")
    {
        cfg.animation_type = Some("breathe".to_string());
        cfg.effect = "breathe".to_string();
    }
    if let Some(img) = &args.image {
        cfg.image = Some(img.to_string_lossy().to_string());
    }
    if let Some(accel) = &args.accel {
        cfg.accel_mode = Some(accel.clone());
    }
    if let Some(aspect) = &args.aspect {
        cfg.aspect = Some(aspect.clone());
    }
    if let Some(aspect_mode) = &args.aspect_mode {
        cfg.aspect_mode = Some(aspect_mode.clone());
    }
    if let Some(contrast_mode) = &args.contrast_mode {
        cfg.contrast_mode = Some(contrast_mode.clone());
    }
    if let Some(colorblind) = &args.colorblind {
        cfg.colorblind_mode = Some(colorblind.clone());
    }
    if args.no_check_updates {
        cfg.check_updates = false;
    } else if args.check_updates {
        cfg.check_updates = true;
    }

    // Validate mascot taxonomy locomotion consistency
    let animal_name = args.cow.as_deref().unwrap_or(&cfg.cow);
    let biome = forgum_platform::biome::get_mascot_biome(animal_name);
    if let Some((remapped, reason)) = biome.remap_incompatible_effect(&cfg.effect) {
        eprintln!(
            "forgum: Mascot '{animal_name}' is {} (habitat: {}). Remapping incompatible locomotion '{}' -> '{remapped}' ({reason}) for physical consistency.",
            biome.habitat(),
            biome.habitat(),
            cfg.effect
        );
        cfg.effect = remapped.to_string();
    }
    if let Some(anim) = &cfg.animation_type {
        if let Some((remapped, _)) = biome.remap_incompatible_effect(anim) {
            cfg.animation_type = Some(remapped.to_string());
        }
    }

    Ok(cfg)
}

/// Levenshtein distance between two strings.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut prev_row: Vec<usize> = (0..=b_bytes.len()).collect();
    let mut curr_row = vec![0; b_bytes.len() + 1];

    for (i, &ca) in a_bytes.iter().enumerate() {
        curr_row[0] = i + 1;
        for (j, &cb) in b_bytes.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)
                .min(prev_row[j + 1] + 1)
                .min(prev_row[j] + cost);
        }
        prev_row.clone_from_slice(&curr_row);
    }
    prev_row[b_bytes.len()]
}

/// Find closest matching string among candidates using substring or Levenshtein distance <= 3.
pub fn find_closest_match<'a>(query: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let q = query.trim_start_matches('-').to_lowercase();
    if q.is_empty() {
        return None;
    }
    let mut best_match = None;
    let mut best_dist = usize::MAX;

    for &cand in candidates {
        let c = cand.trim_start_matches('-').to_lowercase();
        if c == q {
            return Some(cand);
        }
        if c.contains(&q) || q.contains(&c) {
            return Some(cand);
        }
        let dist = levenshtein_distance(&q, &c);
        if dist <= 3 && dist < best_dist {
            best_dist = dist;
            best_match = Some(cand);
        }
    }
    best_match
}

/// Check if a single argument is an obvious command typo (like 'cows', 'effect') rather than intentional speech text.
pub fn is_direct_misspelling(word: &str) -> bool {
    let q = word.trim().trim_start_matches('-').to_lowercase();
    matches!(
        q.as_str(),
        "cow"
            | "cows"
            | "animal"
            | "animals"
            | "mascot"
            | "mascots"
            | "effect"
            | "effects"
            | "anim"
            | "animation"
            | "animations"
            | "motion"
            | "scenery"
            | "env"
            | "environment"
            | "mountain"
            | "mountains"
            | "road"
            | "roads"
            | "color"
            | "colors"
            | "palette"
            | "palettes"
            | "lolcat"
            | "shell"
            | "shells"
            | "completion"
            | "completions"
    )
}

const SUBCOMMAND_CANDIDATES: &[&str] = &[
    "render",
    "think",
    "fortune",
    "tui",
    "init",
    "completions",
    "list",
    "status",
    "doctor",
    "checkhealth",
    "config",
    "logs",
    "diagnose",
    "tmux",
    "status-line",
    "herd",
    "theme",
    "demo",
    "showcase",
    "remote",
    "say",
    "timer",
    "battle",
    "rps-battle",
    "sweep",
    "stop",
    "install",
    "uninstall",
    "update",
    "channel",
];

/// Check if a single argument is an obvious command typo (like 'renderr', 'cows', 'effect')
/// rather than intentional speech text.
pub fn is_command_typo(word: &str) -> Option<String> {
    let q = word.trim().trim_start_matches('-').to_lowercase();
    if is_direct_misspelling(&q) {
        return generate_command_suggestion(&q);
    }
    for &sub in SUBCOMMAND_CANDIDATES {
        let max_dist = if sub.len() <= 4 { 1 } else { 2 };
        if levenshtein_distance(&q, sub) <= max_dist {
            let sub_list = crate::completions::all_subcommands();
            let mut msg = format!("💡 Did you mean '{sub}'?\n\nAvailable subcommands:");
            for (name, desc) in sub_list {
                msg.push_str(&format!("\n  • {:<16} {}", name, desc));
            }
            return Some(msg);
        }
    }
    None
}

/// Check if an argument string is an explicit query for available options.
pub fn is_list_query(s: &str) -> bool {
    let s = s.trim();
    s.eq_ignore_ascii_case("list")
        || s.eq_ignore_ascii_case("--list")
        || s.eq_ignore_ascii_case("-l")
        || s.eq_ignore_ascii_case("help")
        || s.eq_ignore_ascii_case("--help")
        || s.eq_ignore_ascii_case("-h")
        || s == "?"
        || s.eq_ignore_ascii_case("ls")
        || s.eq_ignore_ascii_case("options")
        || s.eq_ignore_ascii_case("show")
}

/// Validate CLI argument and flag values against acceptable options.
///
/// If a user passes an invalid value to a flag or subcommand, this function reports
/// that it is invalid, suggests the closest match, and lists all acceptable values.
pub fn validate_cli_values(cli: &Cli, command: &Option<Commands>) -> Result<(), CliError> {
    // 1. --effect / --animation-type
    let eff_opt = cli.effect.as_deref().or(cli.animation_type.as_deref());
    if let Some(eff) = eff_opt {
        if !is_list_query(eff) && eff != "random" {
            let valid_choices =
                crate::completions::choices_for_flag("--effect").unwrap_or_default();
            let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
            if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(eff)) {
                let closest = find_closest_match(eff, &valid_names);
                return Err(CliError {
                    exit_code: 64,
                    message: crate::completions::format_invalid_value_error(
                        "--effect",
                        eff,
                        closest,
                        &valid_choices,
                    ),
                });
            }
        }
    }

    // 2. --color-mode
    if let Some(ref cm) = cli.color_mode {
        if !is_list_query(cm) {
            let valid_choices =
                crate::completions::choices_for_flag("--color-mode").unwrap_or_default();
            let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
            if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(cm)) {
                let closest = find_closest_match(cm, &valid_names);
                return Err(CliError {
                    exit_code: 64,
                    message: crate::completions::format_invalid_value_error(
                        "--color-mode",
                        cm,
                        closest,
                        &valid_choices,
                    ),
                });
            }
        }
    }

    // 3. --mountain
    if let Some(ref m) = cli.mountain {
        if !is_list_query(m) && m != "random" {
            let valid_choices =
                crate::completions::choices_for_flag("--mountain").unwrap_or_default();
            let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
            if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(m)) {
                let closest = find_closest_match(m, &valid_names);
                return Err(CliError {
                    exit_code: 64,
                    message: crate::completions::format_invalid_value_error(
                        "--mountain",
                        m,
                        closest,
                        &valid_choices,
                    ),
                });
            }
        }
    }

    // 4. --road
    if let Some(ref r) = cli.road {
        if !is_list_query(r) && r != "random" {
            let valid_choices = crate::completions::choices_for_flag("--road").unwrap_or_default();
            let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
            if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(r)) {
                let closest = find_closest_match(r, &valid_names);
                return Err(CliError {
                    exit_code: 64,
                    message: crate::completions::format_invalid_value_error(
                        "--road",
                        r,
                        closest,
                        &valid_choices,
                    ),
                });
            }
        }
    }

    // 5. --environment
    if let Some(ref e) = cli.environment {
        if !is_list_query(e) && e != "random" {
            let valid_choices =
                crate::completions::choices_for_flag("--environment").unwrap_or_default();
            let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
            if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(e)) {
                let closest = find_closest_match(e, &valid_names);
                return Err(CliError {
                    exit_code: 64,
                    message: crate::completions::format_invalid_value_error(
                        "--environment",
                        e,
                        closest,
                        &valid_choices,
                    ),
                });
            }
        }
    }

    // 6. --accel
    if let Some(ref a) = cli.accel {
        let valid_choices = crate::completions::choices_for_flag("--accel").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(a)) {
            let closest = find_closest_match(a, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--accel",
                    a,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 7. --time-of-day
    if let Some(ref tod) = cli.time_of_day {
        let valid_choices =
            crate::completions::choices_for_flag("--time-of-day").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(tod)) {
            let closest = find_closest_match(tod, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--time-of-day",
                    tod,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 8. --season
    if let Some(ref s) = cli.season {
        let valid_choices = crate::completions::choices_for_flag("--season").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(s)) {
            let closest = find_closest_match(s, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--season",
                    s,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 9. --aspect
    if let Some(ref asp) = cli.aspect {
        let valid_choices = crate::completions::choices_for_flag("--aspect").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(asp)) {
            let closest = find_closest_match(asp, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--aspect",
                    asp,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 10. --aspect-mode
    if let Some(ref am) = cli.aspect_mode {
        let valid_choices =
            crate::completions::choices_for_flag("--aspect-mode").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(am)) {
            let closest = find_closest_match(am, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--aspect-mode",
                    am,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 11. --colorblind
    if let Some(ref cb) = cli.colorblind {
        let valid_choices =
            crate::completions::choices_for_flag("--colorblind").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(cb)) {
            let closest = find_closest_match(cb, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--colorblind",
                    cb,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 12. --split-mode
    if let Some(ref sm) = cli.split_mode {
        let valid_choices =
            crate::completions::choices_for_flag("--split-mode").unwrap_or_default();
        let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
        if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(sm)) {
            let closest = find_closest_match(sm, &valid_names);
            return Err(CliError {
                exit_code: 64,
                message: crate::completions::format_invalid_value_error(
                    "--split-mode",
                    sm,
                    closest,
                    &valid_choices,
                ),
            });
        }
    }

    // 13. Subcommand specific validations
    if let Some(ref cmd) = command {
        match cmd {
            Commands::Update {
                method: Some(ref m),
                ..
            } => {
                if !is_list_query(m) {
                    let valid_choices =
                        crate::completions::choices_for_flag("--method").unwrap_or_default();
                    let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
                    if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(m)) {
                        let closest = find_closest_match(m, &valid_names);
                        return Err(CliError {
                            exit_code: 64,
                            message: crate::completions::format_invalid_value_error(
                                "--method",
                                m,
                                closest,
                                &valid_choices,
                            ),
                        });
                    }
                }
            }
            Commands::Uninstall {
                method: Some(ref m),
                ..
            } => {
                if !is_list_query(m) {
                    let valid_choices =
                        crate::completions::choices_for_flag("--method").unwrap_or_default();
                    let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
                    if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(m)) && m != "purge" {
                        let closest = find_closest_match(m, &valid_names);
                        return Err(CliError {
                            exit_code: 64,
                            message: crate::completions::format_invalid_value_error(
                                "--method",
                                m,
                                closest,
                                &valid_choices,
                            ),
                        });
                    }
                }
            }
            Commands::Config {
                migrate: Some(ref fmt),
                ..
            } => {
                let valid_choices =
                    crate::completions::choices_for_flag("--migrate").unwrap_or_default();
                let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
                if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(fmt)) {
                    let closest = find_closest_match(fmt, &valid_names);
                    return Err(CliError {
                        exit_code: 64,
                        message: crate::completions::format_invalid_value_error(
                            "--migrate",
                            fmt,
                            closest,
                            &valid_choices,
                        ),
                    });
                }
            }
            Commands::Logs {
                level: Some(ref lvl),
                ..
            } => {
                let valid_choices =
                    crate::completions::choices_for_flag("--level").unwrap_or_default();
                let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
                if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(lvl)) {
                    let closest = find_closest_match(lvl, &valid_names);
                    return Err(CliError {
                        exit_code: 64,
                        message: crate::completions::format_invalid_value_error(
                            "--level",
                            lvl,
                            closest,
                            &valid_choices,
                        ),
                    });
                }
            }
            Commands::RpsBattle {
                choice: Some(ref ch),
                ..
            } => {
                let valid_choices =
                    crate::completions::choices_for_flag("--choice").unwrap_or_default();
                let valid_names: Vec<&str> = valid_choices.iter().map(|(n, _)| *n).collect();
                if !valid_names.iter().any(|v| v.eq_ignore_ascii_case(ch)) {
                    let closest = find_closest_match(ch, &valid_names);
                    return Err(CliError {
                        exit_code: 64,
                        message: crate::completions::format_invalid_value_error(
                            "--choice",
                            ch,
                            closest,
                            &valid_choices,
                        ),
                    });
                }
            }
            Commands::List { ref category } => {
                let valid_cats = &[
                    "all",
                    "animals",
                    "effects",
                    "scenery",
                    "colors",
                    "shells",
                    "mountains",
                    "roads",
                    "environments",
                    "config",
                    "multiplexers",
                    "eyes",
                    "tongue",
                ];
                if !valid_cats.iter().any(|c| c.eq_ignore_ascii_case(category)) {
                    let closest = find_closest_match(category, valid_cats);
                    let choices: Vec<(&str, &str)> = crate::completions::LIST_CATEGORY_CHOICES
                        .iter()
                        .map(|c| (c.value, c.description))
                        .collect();
                    return Err(CliError {
                        exit_code: 64,
                        message: crate::completions::format_invalid_value_error(
                            "category", category, closest, &choices,
                        ),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Generate helpful, interactive CLI suggestions based on user input.
pub fn generate_command_suggestion(query: &str) -> Option<String> {
    let q = query.trim().trim_start_matches('-').to_lowercase();
    match q.as_str() {
        "cow" | "cows" | "animal" | "animals" | "mascot" | "mascots" => Some(
            "💡 Did you mean:\n  • forgum --animal <name>      (render a specific animal mascot)\n  • forgum list animals         (view all 106 available animals)\n  • forgum list                 (view all available options)".to_string()
        ),
        "effect" | "effects" | "anim" | "animation" | "animations" | "motion" => Some(
            "💡 Did you mean:\n  • forgum --effect <name>      (set animation effect: walk, breathe, float...)\n  • forgum list effects         (view all available effects)".to_string()
        ),
        "scenery" | "env" | "environment" | "mountain" | "mountains" | "road" | "roads" => Some(
            "💡 Did you mean:\n  • forgum --mountain <style>   (set procedural mountain horizon)\n  • forgum --road <style>       (set ground terrain)\n  • forgum --environment <name> (set atmospheric particles)\n  • forgum list scenery         (view all scenery options)".to_string()
        ),
        "color" | "colors" | "palette" | "palettes" | "lolcat" => Some(
            "💡 Did you mean:\n  • forgum --color-mode <mode>  (set color mode: animal, rainbow, solid, none)\n  • forgum --palette <hex,...>  (set custom hex gradient)\n  • forgum list colors          (view all color modes and palettes)".to_string()
        ),
        "shell" | "shells" | "completion" | "completions" | "tab" => Some(
            "💡 Did you mean:\n  • forgum completions <shell> (install tab completions into your shell profile)\n  • forgum init <shell>        (generate shell prompt integration hook)\n  • forgum list shells         (view supported shells & config paths)".to_string()
        ),
        "help" | "search" | "find" => Some(
            "💡 Did you mean:\n  • forgum --help               (print complete command-line reference)\n  • forgum list                 (browse all options in structured tables)".to_string()
        ),
        other => {
            let candidates = &[
                "render", "think", "fortune", "tui", "init", "completions", "list",
                "status", "doctor", "checkhealth", "config", "logs", "tmux", "status-line",
                "herd", "theme", "demo", "showcase", "remote", "say", "timer", "battle",
                "sweep", "clean", "stop", "kill", "halt", "reset", "unreserve", "clear-margins",
                "--animal", "--animation", "--effect", "--environment", "--road", "--mountain",
                "--color-mode", "--palette", "--thought-interval", "--split-scroll", "--text",
                "--think", "--background", "--banner", "--duration", "--fps", "--list",
            ];
            find_closest_match(other, candidates).map(|closest| {
                format!("💡 Did you mean '{closest}'? Run 'forgum list' to browse available options.")
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argv: &[&str]) -> (Args, Option<Commands>) {
        let argv_str: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
        parse_args(argv_str).unwrap()
    }

    #[test]
    fn remote_attach_subcommand() {
        let (a, cmd) = parse(&["forgum", "remote", "attach", "user@host"]);
        assert_eq!(a.command, Command::Remote);
        assert!(matches!(
            cmd,
            Some(Commands::Remote {
                sub: RemoteSub::Attach { .. }
            })
        ));
    }

    #[test]
    fn no_args_is_render() {
        let (a, _cmd) = parse(&["forgum"]);
        assert_eq!(a.command, Command::Render);
        assert!(!a.background);
    }

    #[test]
    fn render_command_with_flags() {
        let (a, _cmd) = parse(&[
            "forgum",
            "render",
            "--cow",
            "tux",
            "--text",
            "hi",
            "--background",
        ]);
        assert_eq!(a.command, Command::Render);
        assert_eq!(a.cow.as_deref(), Some("tux"));
        assert_eq!(a.text.as_deref(), Some("hi"));
        assert!(a.background);
    }

    #[test]
    fn render_banner_flag() {
        let (a, _cmd) = parse(&["forgum", "--banner"]);
        assert!(a.banner);
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.shell_attach_mode, "banner");

        let (a2, _cmd) = parse(&["forgum", "-B"]);
        assert!(a2.banner);
    }

    #[test]
    fn fortune_subcommand() {
        let (a, cmd) = parse(&["forgum", "fortune"]);
        assert_eq!(a.command, Command::Fortune);
        assert!(matches!(cmd, Some(Commands::Fortune)));
    }

    #[test]
    fn status_subcommand() {
        let (a, cmd) = parse(&["forgum", "status"]);
        assert_eq!(a.command, Command::Status);
        assert!(matches!(cmd, Some(Commands::Status)));
    }

    #[test]
    fn init_bash() {
        let (a, cmd) = parse(&["forgum", "init", "bash"]);
        assert_eq!(a.command, Command::Init);
        assert!(matches!(
            cmd,
            Some(Commands::Init {
                shell: Some(ShellArg::Bash),
                ..
            })
        ));
    }

    #[test]
    fn completions_zsh() {
        let (a, cmd) = parse(&["forgum", "completions", "zsh"]);
        assert_eq!(a.command, Command::Completions);
        assert!(matches!(
            cmd,
            Some(Commands::Completions {
                shell: ShellArg::Zsh
            })
        ));
    }

    #[test]
    fn build_scene_merges_cli_over_config() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg_path = tmp.path().join("config.json");
        std::fs::write(&cfg_path, r#"{"cow":"base","fps":15}"#).unwrap();

        let (a, _) = parse(&[
            "forgum",
            "render",
            "--config",
            cfg_path.to_str().unwrap(),
            "--cow",
            "cli",
        ]);
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.cow, "cli"); // CLI wins
        assert_eq!(cfg.fps, 15); // config file value
    }

    #[test]
    fn tmux_install_subcommand() {
        let (a, cmd) = parse(&["forgum", "tmux", "install"]);
        assert_eq!(a.command, Command::Tmux);
        assert!(matches!(
            cmd,
            Some(Commands::Tmux {
                sub: TmuxSub::Install
            })
        ));
    }

    #[test]
    fn status_line_default_max_len() {
        let (a, cmd) = parse(&["forgum", "status-line"]);
        assert_eq!(a.command, Command::StatusLine);
        assert_eq!(a.max_len, Some(70));
        assert!(matches!(
            cmd,
            Some(Commands::StatusLine { max_len: 70, .. })
        ));
    }

    #[test]
    fn status_line_custom_max_len() {
        let (a, cmd) = parse(&["forgum", "status-line", "--max-len", "40", "--mux", "tmux"]);
        assert_eq!(a.command, Command::StatusLine);
        assert_eq!(a.max_len, Some(40));
        assert!(matches!(
            cmd,
            Some(Commands::StatusLine {
                max_len: 40,
                mux
            }) if mux == "tmux"
        ));
    }

    #[test]
    fn fetch_command_parsing() {
        let (a, cmd) = parse(&["forgum", "fetch", "--cow", "dragon"]);
        assert_eq!(a.command, Command::Fetch);
        assert!(matches!(
            cmd,
            Some(Commands::Fetch {
                cow: Some(c),
                live: false,
                ..
            }) if c == "dragon"
        ));
    }

    #[test]
    fn telemetry_command_parsing() {
        let (a, cmd) = parse(&["forgum", "telemetry", "report", "-x", "0", "-m", "42"]);
        assert_eq!(a.command, Command::Telemetry);
        assert!(matches!(
            cmd,
            Some(Commands::Telemetry {
                sub: TelemetrySub::Report {
                    exit_code: 0,
                    elapsed_ms: 42,
                    ..
                }
            })
        ));
    }

    #[test]
    fn animal_flag_aliases_cow() {
        let (a, _) = parse(&["forgum", "--animal", "dragon"]);
        assert_eq!(a.cow, Some("dragon".to_string()));
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.cow, "dragon");
    }

    #[test]
    fn scenery_and_color_flags_propagate_to_scene_config() {
        let (a, _) = parse(&[
            "forgum",
            "--environment",
            "inferno",
            "--road",
            "magma",
            "--mountain",
            "volcano",
            "--color-mode",
            "animal",
            "--palette",
            "#ff0000,#00ff00",
            "--thought-interval",
            "45",
            "--split-scroll",
        ]);
        assert_eq!(a.environment, Some("inferno".to_string()));
        assert_eq!(a.road, Some("magma".to_string()));
        assert_eq!(a.mountain, Some("volcano".to_string()));
        assert_eq!(a.color_mode, Some("animal".to_string()));
        assert_eq!(a.palette, Some("#ff0000,#00ff00".to_string()));
        assert_eq!(a.thought_interval, Some(45));
        assert!(a.split_scroll);

        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.environment, Some("inferno".to_string()));
        assert_eq!(cfg.road, Some("magma".to_string()));
        assert_eq!(cfg.mountain, Some("volcano".to_string()));
        assert_eq!(cfg.color_mode, "natural");
        assert_eq!(cfg.palette, Some("#ff0000,#00ff00".to_string()));
        assert_eq!(cfg.thought_interval, 45);
        assert!(cfg.split_scroll);
    }

    #[test]
    fn animation_static_rejects_dynamic_type() {
        let argv = vec![
            "forgum".to_string(),
            "--animation".to_string(),
            "static".to_string(),
            "--animation-type".to_string(),
            "walk".to_string(),
        ];
        let err = parse_args(argv).unwrap_err();
        assert_eq!(err.exit_code, 64);
        assert!(err
            .message
            .contains("Cannot specify dynamic animation type 'walk' when animation is static"));
    }

    #[test]
    fn animation_dynamic_rejects_static_type() {
        let argv = vec![
            "forgum".to_string(),
            "--animation".to_string(),
            "dynamic".to_string(),
            "--animation-type".to_string(),
            "static".to_string(),
        ];
        let err = parse_args(argv).unwrap_err();
        assert_eq!(err.exit_code, 64);
        assert!(err
            .message
            .contains("Cannot specify animation type 'static' when animation is dynamic"));
    }

    #[test]
    fn list_subcommand_parses_category() {
        let (a, cmd) = parse(&["forgum", "list", "animals"]);
        assert_eq!(a.command, Command::List);
        match cmd {
            Some(Commands::List { category }) => assert_eq!(category, "animals"),
            _ => panic!("Expected Commands::List"),
        }
    }

    #[test]
    fn list_flag_parses_optional_category() {
        let (a1, _) = parse(&["forgum", "--list"]);
        assert_eq!(a1.command, Command::List);
        assert_eq!(a1.list, Some("all".to_string()));

        let (a2, _) = parse(&["forgum", "-l", "effects"]);
        assert_eq!(a2.command, Command::List);
        assert_eq!(a2.list, Some("effects".to_string()));
    }

    #[test]
    fn suggestions_for_common_misspellings() {
        let err1 = parse_args(vec!["forgum".to_string(), "cows".to_string()]).unwrap_err();
        assert!(err1.message.contains("Did you mean:"));
        assert!(err1.message.contains("forgum list animals"));

        let err2 = parse_args(vec!["forgum".to_string(), "effect".to_string()]).unwrap_err();
        assert!(err2.message.contains("forgum list effects"));
    }

    #[test]
    fn test_color_mode_natural_and_aliases() {
        let (a1, _) = parse(&["forgum", "--color-mode", "natural"]);
        assert_eq!(a1.color_mode, Some("natural".to_string()));
        let cfg1 = build_scene_config(&a1).unwrap();
        assert_eq!(cfg1.color_mode, "natural");
        assert!(cfg1.palette.is_some());

        let (a2, _) = parse(&["forgum", "--color-mode", "animal_natural"]);
        assert_eq!(a2.color_mode, Some("animal_natural".to_string()));
        let cfg2 = build_scene_config(&a2).unwrap();
        assert_eq!(cfg2.color_mode, "natural");
    }

    #[test]
    fn test_effect_animal_natural_and_aliases() {
        let (a1, _) = parse(&["forgum", "--effect", "animal_natural", "--cow", "dragon"]);
        assert_eq!(a1.effect, Some("animal_natural".to_string()));
        let cfg1 = build_scene_config(&a1).unwrap();
        assert_eq!(cfg1.effect, "animal_natural");
        assert_eq!(cfg1.animation_type, Some("breathe".to_string()));

        let (a2, _) = parse(&["forgum", "--fx", "animal_natural"]);
        assert_eq!(a2.effect, Some("animal_natural".to_string()));

        let (a3, _) = parse(&["forgum", "--anim", "animal_natural"]);
        assert_eq!(a3.effect, Some("animal_natural".to_string()));

        let (a4, _) = parse(&[
            "forgum",
            "--animation-type",
            "animal_natural",
            "--cow",
            "ghost",
        ]);
        assert_eq!(a4.animation_type, Some("animal_natural".to_string()));
        let cfg4 = build_scene_config(&a4).unwrap();
        assert_eq!(cfg4.animation_type, Some("float".to_string()));

        let (a5, _) = parse(&["forgum", "--anim-type", "animal_natural"]);
        assert_eq!(a5.animation_type, Some("animal_natural".to_string()));
    }

    #[test]
    fn test_custom_hex_palette_override() {
        let (a, _) = parse(&[
            "forgum",
            "--cow",
            "dragon",
            "--palette",
            "#112233,#445566,#778899",
        ]);
        assert_eq!(a.palette, Some("#112233,#445566,#778899".to_string()));
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.palette, Some("#112233,#445566,#778899".to_string()));
    }

    #[test]
    fn test_scenery_nature_math_environment_adaptation() {
        let (a, _) = parse(&["forgum", "--environment", "ocean", "--cow", "cat"]);
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.environment, Some("ocean".to_string()));
        assert_eq!(cfg.mountain, Some("seamount".to_string()));
        assert_eq!(cfg.road, Some("seabed".to_string()));

        // Explicit road override
        let (a2, _) = parse(&["forgum", "--environment", "ocean", "--road", "magma"]);
        let cfg2 = build_scene_config(&a2).unwrap();
        assert_eq!(cfg2.environment, Some("ocean".to_string()));
        assert_eq!(cfg2.mountain, Some("seamount".to_string()));
        assert_eq!(cfg2.road, Some("magma".to_string()));
    }

    #[test]
    fn test_normalize_argv_subcommand_dispatch() {
        let (a1, _) = parse(&["forgum", "cow", "tux"]);
        assert_eq!(a1.cow, Some("tux".to_string()));

        let (a2, _) = parse(&["forgum", "cow", "dragon", "fps", "60", "duration", "5"]);
        assert_eq!(a2.cow, Some("dragon".to_string()));
        assert_eq!(a2.fps, Some(60));
        assert_eq!(a2.duration, Some(5));

        let (a3, _) = parse(&["forgum", "list", "animals"]);
        assert_eq!(a3.command, Command::List);
        assert_eq!(a3.list, Some("animals".to_string()));

        let (a4, _) = parse(&["forgum", "list", "effects"]);
        assert_eq!(a4.command, Command::List);
        assert_eq!(a4.list, Some("effects".to_string()));

        let (a5, _) = parse(&["forgum", "scenery", "savanna"]);
        assert_eq!(a5.environment, Some("savanna".to_string()));

        let (a6, _) = parse(&["forgum", "effect", "walk"]);
        assert_eq!(a6.effect, Some("walk".to_string()));
    }

    #[test]
    fn test_help_table_interception() {
        let err_help = parse_args(vec!["forgum".to_string(), "help".to_string()]).unwrap_err();
        assert_eq!(err_help.exit_code, 0);
        assert!(err_help.message.contains("Core Commands & Execution Modes"));
        assert!(err_help.message.contains("Dual Representation"));

        let err_help_cat = parse_args(vec![
            "forgum".to_string(),
            "help".to_string(),
            "scenery".to_string(),
        ])
        .unwrap_err();
        assert_eq!(err_help_cat.exit_code, 0);
        assert!(err_help_cat
            .message
            .contains("Procedural Scenery & Nature Mathematics"));
    }
}
