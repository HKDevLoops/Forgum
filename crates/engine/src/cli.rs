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
    name = "forgum-engine",
    version,
    about = "Forgum animation engine — renders cowsay+fortune+lolcat with effects",
    long_about = None,
    after_help = "Run `forgum-engine init <shell>` to set up shell integration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to config JSON file.
    #[arg(long, global = true, env = "FORGUM_CONFIG")]
    pub config: Option<PathBuf>,

    /// Disable animated effects (gentle fades only).
    #[arg(long, global = true)]
    pub reduce_motion: bool,

    /// Print just the fortune text (no cow art).
    #[arg(long, global = true)]
    pub text_only: bool,

    /// Path to scene JSON file (alternative to stdin).
    #[arg(long, short = 'f', global = true)]
    pub file: Option<PathBuf>,

    /// Cow file basename (without .cow).
    #[arg(long, short = 'c', global = true)]
    pub cow: Option<String>,

    /// Text inside the speech bubble.
    #[arg(long, short = 't', global = true)]
    pub text: Option<String>,

    /// Effect name.
    #[arg(long, short = 'e', global = true)]
    pub effect: Option<String>,

    /// Eye string (e.g. "oo", "$$").
    #[arg(long, global = true)]
    pub eyes: Option<String>,

    /// Tongue string (e.g. "U").
    #[arg(long, global = true)]
    pub tongue: Option<String>,

    /// Render above prompt as a non-blocking overlay.
    #[arg(long, short = 'b', global = true)]
    pub background: bool,

    /// Duration in seconds. 0 = infinite (with --background).
    #[arg(long, short = 'd', global = true)]
    pub duration: Option<u32>,

    /// Target FPS.
    #[arg(long, global = true)]
    pub fps: Option<u16>,

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
    Render,
    /// Print a random fortune to stdout.
    Fortune,
    /// Generate shell integration hooks.
    Init {
        /// Target shell.
        #[arg(value_enum)]
        shell: ShellArg,
        /// Only print the generated hook (CI validation); identical output to a
        /// normal `init` but explicit about the use-case.
        #[arg(long)]
        check: bool,
    },
    /// Generate shell completion scripts.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: ShellArg,
    },
    /// Print 'ok' and exit (for daemon health checks).
    Status,
    /// View or edit configuration.
    Config {
        /// Open the interactive config menu.
        #[arg(long)]
        tui: bool,
        /// Set a key to a value (headless).
        #[arg(value_name = "KEY")]
        key: Option<String>,
        /// Value for --key.
        #[arg(value_name = "VALUE")]
        value: Option<String>,
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
    /// ASCII cow jousting battle.
    Battle {
        /// Name of the first cow.
        #[arg(long, default_value = "Alice")]
        name1: String,
        /// Name of the second cow.
        #[arg(long, default_value = "Bob")]
        name2: String,
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
    PowerShell,
}

impl From<ShellArg> for Shell {
    fn from(arg: ShellArg) -> Self {
        match arg {
            ShellArg::Bash => Shell::Bash,
            ShellArg::Zsh => Shell::Zsh,
            ShellArg::Fish => Shell::Fish,
            ShellArg::Pwsh => Shell::Pwsh,
            ShellArg::Cmd => Shell::Cmd,
            ShellArg::PowerShell => Shell::PowerShell,
        }
    }
}

/// Backward-compatible command enum (used by main.rs match).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Command {
    #[default]
    Render,
    Status,
    Config,
    Fortune,
    Init,
    Completions,
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
    Unknown(String),
}

/// Parsed arguments (backward-compatible with Phase 0).
#[derive(Debug, Clone, Default)]
pub struct Args {
    pub command: Command,
    pub file: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub background: bool,
    pub duration: Option<u32>,
    pub fps: Option<u16>,
    pub cow: Option<String>,
    pub text: Option<String>,
    pub effect: Option<String>,
    pub eyes: Option<String>,
    pub tongue: Option<String>,
    pub daemon: bool,
    pub control_socket: Option<PathBuf>,
    /// (Internal) Marker set by the parent on respawn so the child knows
    /// it is THE daemon (no second fork). End users should never pass
    /// this; it exists because fork()-based daemonization breaks under
    ///
    /// - multi-threaded parents (UB: held mutexes stay locked forever in
    ///   the child, which then deadlocks on its first allocation); and
    /// - QEMU user-mode emulation (CI's `cross` runner for arm64), which
    ///   rejects `fork()` with EINVAL/ENOSYS.
    ///
    /// Instead, the parent uses `Command::spawn` (posix_spawn under the
    /// hood, single-threaded by construction) of itself with this flag,
    /// then waits for the state file to appear, prints the child PID,
    /// and exits 0. The spawned process sees this flag and runs the
    /// daemon body directly.
    pub internal_daemon_runner: bool,
    pub max_len: Option<usize>,
    pub reduce_motion: bool,
    pub text_only: bool,
}

/// Error type returned by `parse_args`, carrying the intended process exit code.
#[derive(Debug)]
pub struct CliError {
    pub exit_code: u8,
    pub message: String,
}

/// Parse CLI args (backward-compatible wrapper around clap).
pub fn parse_args(argv: Vec<String>) -> Result<(Args, Option<Commands>), CliError> {
    let cli = Cli::try_parse_from(&argv).map_err(|e| {
        // clap prints help/version itself and wants to exit 0; real parse
        // errors (UnknownArgument, etc.) should exit 64 (EX_USAGE).
        let exit_code = match e.kind() {
            clap::error::ErrorKind::DisplayHelp
            | clap::error::ErrorKind::DisplayVersion
            | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => 0,
            _ => 64,
        };
        CliError {
            exit_code,
            message: e.to_string(),
        }
    })?;

    let command = match &cli.command {
        Some(Commands::Render) | None => Command::Render,
        Some(Commands::Fortune) => Command::Fortune,
        Some(Commands::Init { .. }) => Command::Init,
        Some(Commands::Completions { .. }) => Command::Completions,
        Some(Commands::Status) => Command::Status,
        Some(Commands::Config { .. }) => Command::Config,
        Some(Commands::Tmux { sub }) => match sub {
            TmuxSub::Install => Command::Tmux,
            TmuxSub::Zellij => Command::Tmux,
            TmuxSub::WezTerm => Command::Tmux,
            TmuxSub::Screen => Command::Tmux,
        },
        Some(Commands::StatusLine { .. }) => Command::StatusLine,
        Some(Commands::Herd { sub }) => match sub {
            HerdSub::Census => Command::Herd,
            _ => Command::Herd,
        },
        Some(Commands::Theme { .. }) => Command::Theme,
        Some(Commands::Demo) => Command::Demo,
        Some(Commands::Showcase) => Command::Showcase,
        Some(Commands::Remote { .. }) => Command::Remote,
        Some(Commands::Say { .. }) => Command::Say,
        Some(Commands::Timer { .. }) => Command::Timer,
        Some(Commands::Battle { .. }) => Command::Battle,
    };

    let max_len = match &cli.command {
        Some(Commands::StatusLine { max_len }) => Some(*max_len),
        _ => None,
    };

    let args = Args {
        command,
        file: cli.file,
        config: cli.config,
        background: cli.background,
        duration: cli.duration,
        fps: cli.fps,
        cow: cli.cow,
        text: cli.text,
        effect: cli.effect,
        eyes: cli.eyes,
        tongue: cli.tongue,
        daemon: cli.daemon,
        control_socket: cli.control_socket,
        internal_daemon_runner: cli.internal_daemon_runner,
        max_len,
        reduce_motion: cli.reduce_motion,
        text_only: cli.text_only,
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
    if let Some(t) = &args.text {
        cfg.text = t.clone();
    }
    if let Some(e) = &args.effect {
        cfg.effect = e.clone();
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

    // If --background and no explicit duration, default to 0 (infinite).
    if cfg.background && args.duration.is_none() && cfg.duration == 0 {
        // already 0, which means infinite — correct
    }

    Ok(cfg)
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
}
