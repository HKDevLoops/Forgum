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
        help = "Target rendering frame rate (1-120, default: 30)",
        long_help = "Target frames per second. Forgum uses delta-time interpolation and sub-millisecond timer pacing."
    )]
    pub fps: Option<u16>,

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
        /// Optional speech text to display inside the speech bubble.
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
    /// Render a thought bubble (cowthink mode).
    Think {
        /// Optional thought text (defaults to random fortune if omitted).
        #[arg(num_args = 0..)]
        thought: Vec<String>,
    },
    /// Print a random fortune to stdout.
    Fortune,
    /// Launch the interactive terminal UI & studio.
    #[command(alias = "menu", alias = "ui", alias = "studio")]
    Tui {
        /// Initial tab to focus: mascots, scenery, effects, installer, config
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
        /// Target release channel to update or switch to (stable, nightly, dev).
        #[arg(long, value_name = "CHANNEL")]
        channel: Option<String>,
    },
    /// View, switch, or manage release channels (stable, nightly, dev).
    #[command(alias = "channels")]
    Channel {
        /// Action: 'get', 'set', 'list', 'switch', or target channel name directly.
        #[arg(value_name = "ACTION")]
        action: Option<String>,
        /// Channel name when action is 'set' or 'switch' (stable, nightly, dev).
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Generate shell integration hooks.
    Init {
        /// Target shell, or 'list' to view supported shells.
        #[arg(value_enum, default_value = "list")]
        shell: ShellArg,
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
    /// One-shot status line for tmux status-right.
    StatusLine {
        /// Maximum visible length in characters.
        #[arg(long, default_value = "70")]
        max_len: usize,
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
    Channel,
    Sweep,
    Stop,
    Image,
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
    pub list: Option<String>,
    pub random: Option<String>,
}

/// Error type returned by `parse_args`, carrying the intended process exit code.
#[derive(Debug)]
pub struct CliError {
    pub exit_code: u8,
    pub message: String,
}

/// Parse CLI args (backward-compatible wrapper around clap).
pub fn parse_args(argv: Vec<String>) -> Result<(Args, Option<Commands>), CliError> {
    let cli = match Cli::try_parse_from(&argv) {
        Ok(c) => c,
        Err(e) => {
            // clap prints help/version itself and wants to exit 0; real parse
            // errors (UnknownArgument, etc.) should exit 64 (EX_USAGE).
            let exit_code = match e.kind() {
                clap::error::ErrorKind::DisplayHelp
                | clap::error::ErrorKind::DisplayVersion
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => 0,
                _ => 64,
            };
            let mut message = e.to_string();
            if exit_code != 0 {
                for arg in argv.iter().skip(1) {
                    if let Some(suggestion) = generate_command_suggestion(arg) {
                        message.push_str("\n\n");
                        message.push_str(&suggestion);
                        break;
                    }
                }
            }
            return Err(CliError { exit_code, message });
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

    let (command, extra_text, is_think) = match &cli.command {
        Some(Commands::Render { message }) => {
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
            if cli.list.is_some() {
                (Command::List, None, false)
            } else {
                (Command::Render, None, false)
            }
        }
        Some(Commands::Think { thought }) => {
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
        Some(Commands::Status) => (Command::Status, None, false),
        Some(Commands::Config { .. }) => (Command::Config, None, false),
        Some(Commands::Tmux { sub }) => (
            match sub {
                TmuxSub::Install => Command::Tmux,
                TmuxSub::Zellij => Command::Tmux,
                TmuxSub::WezTerm => Command::Tmux,
                TmuxSub::Screen => Command::Tmux,
                TmuxSub::List => Command::Tmux,
            },
            None,
            false,
        ),
        Some(Commands::StatusLine { .. }) => (Command::StatusLine, None, false),
        Some(Commands::Herd { sub }) => (
            match sub {
                HerdSub::Census => Command::Herd,
                _ => Command::Herd,
            },
            None,
            false,
        ),
        Some(Commands::Theme { .. }) => (Command::Theme, None, false),
        Some(Commands::Demo) => (Command::Demo, None, false),
        Some(Commands::Showcase) => (Command::Showcase, None, false),
        Some(Commands::Remote { .. }) => (Command::Remote, None, false),
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
    };

    let max_len = match &cli.command {
        Some(Commands::StatusLine { max_len }) => Some(*max_len),
        _ => None,
    };

    let text = cli.text.or(extra_text);
    let think = cli.think || is_think;

    let args = Args {
        command,
        file: cli.file,
        config: cli.config,
        background: cli.background,
        banner: cli.banner,
        duration: cli.duration,
        fps: cli.fps,
        cow: cli.cow,
        image: cli.image,
        animation: cli.animation,
        animation_type: cli.animation_type,
        environment: cli.environment,
        road: cli.road,
        mountain: cli.mountain,
        color_mode: cli.color_mode,
        palette: cli.palette,
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
        list: cli.list,
        random: cli.random,
    };

    Ok((args, cli.command))
}

/// Build the final `SceneConfig` from `Args` and config file.
/// Precedence: CLI > --file > --config > defaults.
pub fn build_scene_config(args: &Args) -> Result<SceneConfig, String> {
    let mut cfg = if let Some(path) = &args.config {
        crate::config::read_config_file(path).map_err(|e| format!("--config: {e}"))?
    } else {
        // Try auto-discovering config from platform default path.
        match forgum_platform::config_path() {
            Ok(default_path) => crate::config::read_config_file(&default_path).unwrap_or_default(),
            Err(_) => SceneConfig::default(),
        }
    };

    if let Some(path) = &args.file {
        let overlay = crate::config::read_config_file(path).map_err(|e| format!("--file: {e}"))?;
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
    if let Some(r) = &args.random {
        cfg.random = Some(forgum_platform::protocol::RandomSetting::String(r.clone()));
    }

    // Normalize color_mode aliases to "natural"
    if cfg.color_mode == "default" || cfg.color_mode == "animal" || cfg.color_mode == "animal_natural" {
        cfg.color_mode = "natural".to_string();
    }

    // If color_mode is natural and no explicit CLI palette was provided, delegate to mascot's natural palette (unless text_only is active)
    if !args.text_only && cfg.color_mode == "natural" && args.palette.is_none() {
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
    if args.banner {
        cfg.shell_attach_mode = "banner".to_string();
    }
    if let Some(img) = &args.image {
        cfg.image = Some(img.to_string_lossy().to_string());
    }

    // If --background and no explicit duration, default to 0 (infinite).
    if cfg.background && args.duration.is_none() && cfg.duration == 0 {
        // already 0, which means infinite — correct
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
        let (a, cmd) = parse(&["forgum-engine", "remote", "attach", "user@host"]);
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
        let (a, _cmd) = parse(&["forgum-engine"]);
        assert_eq!(a.command, Command::Render);
        assert!(!a.background);
    }

    #[test]
    fn render_command_with_flags() {
        let (a, _cmd) = parse(&[
            "forgum-engine",
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
        let (a, _cmd) = parse(&["forgum-engine", "--banner"]);
        assert!(a.banner);
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.shell_attach_mode, "banner");

        let (a2, _cmd) = parse(&["forgum-engine", "-B"]);
        assert!(a2.banner);
    }

    #[test]
    fn fortune_subcommand() {
        let (a, cmd) = parse(&["forgum-engine", "fortune"]);
        assert_eq!(a.command, Command::Fortune);
        assert!(matches!(cmd, Some(Commands::Fortune)));
    }

    #[test]
    fn status_subcommand() {
        let (a, cmd) = parse(&["forgum-engine", "status"]);
        assert_eq!(a.command, Command::Status);
        assert!(matches!(cmd, Some(Commands::Status)));
    }

    #[test]
    fn init_bash() {
        let (a, cmd) = parse(&["forgum-engine", "init", "bash"]);
        assert_eq!(a.command, Command::Init);
        assert!(matches!(
            cmd,
            Some(Commands::Init {
                shell: ShellArg::Bash,
                ..
            })
        ));
    }

    #[test]
    fn completions_zsh() {
        let (a, cmd) = parse(&["forgum-engine", "completions", "zsh"]);
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
            "forgum-engine",
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
        let (a, cmd) = parse(&["forgum-engine", "tmux", "install"]);
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
        let (a, cmd) = parse(&["forgum-engine", "status-line"]);
        assert_eq!(a.command, Command::StatusLine);
        assert_eq!(a.max_len, Some(70));
        assert!(matches!(cmd, Some(Commands::StatusLine { max_len: 70 })));
    }

    #[test]
    fn status_line_custom_max_len() {
        let (a, cmd) = parse(&["forgum-engine", "status-line", "--max-len", "40"]);
        assert_eq!(a.command, Command::StatusLine);
        assert_eq!(a.max_len, Some(40));
        assert!(matches!(cmd, Some(Commands::StatusLine { max_len: 40 })));
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
        assert_eq!(cfg.color_mode, "animal");
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
        assert_eq!(cfg2.color_mode, "animal_natural");
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

        let (a4, _) = parse(&["forgum", "--animation-type", "animal_natural", "--cow", "ghost"]);
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
        let (a2, _) = parse(&[
            "forgum",
            "--environment",
            "ocean",
            "--road",
            "magma",
        ]);
        let cfg2 = build_scene_config(&a2).unwrap();
        assert_eq!(cfg2.environment, Some("ocean".to_string()));
        assert_eq!(cfg2.mountain, Some("seamount".to_string()));
        assert_eq!(cfg2.road, Some("magma".to_string()));
    }
}

