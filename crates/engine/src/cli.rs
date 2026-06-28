//! CLI argument parsing.
//!
//! Phase 0 uses a hand-rolled parser instead of `clap` (clap comes in
//! Phase 2 when the surface stabilizes). The shape of the parser mirrors
//! what `clap` will produce so the Phase 2 swap is mechanical.

use std::path::PathBuf;

use crate::protocol::SceneConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Command {
    #[default]
    Render,
    Status,
    /// Unknown / not-yet-implemented subcommand. We accept it so the binary
    /// doesn't bail before printing the help text.
    Unknown(String),
}

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
    pub daemon: bool,
    pub control_socket: Option<PathBuf>,
    pub show_help: bool,
    pub show_version: bool,
}

pub fn parse_args<I: IntoIterator<Item = String>>(argv: I) -> Result<Args, String> {
    let mut iter = argv.into_iter();
    let _prog = iter.next(); // skip program name

    let mut args = Args::default();

    // First positional arg is the command.
    if let Some(cmd) = iter.next() {
        args.command = match cmd.as_str() {
            "render" => Command::Render,
            "status" => Command::Status,
            "help" | "--help" | "-h" => {
                args.show_help = true;
                Command::Render
            }
            "version" | "--version" | "-V" => {
                args.show_version = true;
                Command::Render
            }
            other => Command::Unknown(other.to_string()),
        };
    }

    while let Some(arg) = iter.next() {
        let (key, value) = match arg.split_once('=') {
            Some((k, v)) => (k.to_string(), Some(v.to_string())),
            None => (arg, None),
        };

        match key.as_str() {
            "--background" | "-b" => args.background = true,
            "--daemon" => args.daemon = true,
            "--file" => {
                args.file = Some(PathBuf::from(require_value("--file", value, &mut iter)?));
            }
            "--config" => {
                args.config = Some(PathBuf::from(require_value("--config", value, &mut iter)?));
            }
            "--control-socket" => {
                args.control_socket = Some(PathBuf::from(require_value(
                    "--control-socket",
                    value,
                    &mut iter,
                )?));
            }
            "--cow" => {
                args.cow = Some(require_value("--cow", value, &mut iter)?);
            }
            "--text" => {
                args.text = Some(require_value("--text", value, &mut iter)?);
            }
            "--effect" => {
                args.effect = Some(require_value("--effect", value, &mut iter)?);
            }
            "--duration" => {
                let s = require_value("--duration", value, &mut iter)?;
                args.duration = Some(
                    s.parse()
                        .map_err(|_| format!("--duration: not a number: {s}"))?,
                );
            }
            "--fps" => {
                let s = require_value("--fps", value, &mut iter)?;
                args.fps = Some(s.parse().map_err(|_| format!("--fps: not a number: {s}"))?);
            }
            "--help" | "-h" => args.show_help = true,
            "--version" | "-V" => args.show_version = true,
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    // If --background is set and no explicit --duration, default to 0 (infinite).
    if args.background && args.duration.is_none() {
        args.duration = Some(0);
    }

    Ok(args)
}

fn require_value<I: Iterator<Item = String>>(
    flag: &str,
    inline: Option<String>,
    iter: &mut I,
) -> Result<String, String> {
    if let Some(v) = inline {
        return Ok(v);
    }
    iter.next()
        .ok_or_else(|| format!("{flag} requires a value"))
}

/// Build the final `SceneConfig` from `Args`, the file JSON (if any), and the
/// config JSON (if any). Precedence: CLI > --file > --config > defaults.
pub fn build_scene_config(args: &Args) -> Result<SceneConfig, String> {
    let mut cfg = if let Some(path) = &args.config {
        crate::config::read_config_file(path).map_err(|e| format!("--config: {e}"))?
    } else {
        SceneConfig::default()
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
    if args.background {
        cfg.background = true;
    }
    if let Some(d) = args.duration {
        cfg.duration = d;
    }
    if let Some(f) = args.fps {
        cfg.fps = f;
    }

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argv: &[&str]) -> Args {
        parse_args(argv.iter().map(|s| s.to_string())).unwrap()
    }

    #[test]
    fn no_args_is_render() {
        let a = parse(&["forgum-engine"]);
        assert_eq!(a.command, Command::Render);
        assert!(!a.background);
    }

    #[test]
    fn render_command_with_flags() {
        let a = parse(&[
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
        assert_eq!(a.duration, Some(0)); // implicit
    }

    #[test]
    fn equals_syntax() {
        let a = parse(&["forgum-engine", "render", "--cow=tux", "--text=hi"]);
        assert_eq!(a.cow.as_deref(), Some("tux"));
        assert_eq!(a.text.as_deref(), Some("hi"));
    }

    #[test]
    fn missing_value_errors() {
        let r = parse_args([
            "forgum-engine".to_string(),
            "render".to_string(),
            "--cow".to_string(),
        ]);
        assert!(r.is_err());
    }

    #[test]
    fn unknown_arg_errors() {
        let r = parse_args([
            "forgum-engine".to_string(),
            "render".to_string(),
            "--bogus".to_string(),
        ]);
        assert!(r.is_err());
    }

    #[test]
    fn status_command() {
        let a = parse(&["forgum-engine", "status"]);
        assert_eq!(a.command, Command::Status);
    }

    #[test]
    fn build_scene_merges_cli_over_file_over_config() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg_path = tmp.path().join("config.json");
        std::fs::write(&cfg_path, r#"{"cow":"base","fps":15}"#).unwrap();
        let file_path = tmp.path().join("scene.json");
        // Explicit fps in scene.json so merge semantics are predictable.
        std::fs::write(&file_path, r#"{"cow":"file","text":"fromfile","fps":45}"#).unwrap();

        let args = parse(&[
            "forgum-engine",
            "render",
            "--config",
            cfg_path.to_str().unwrap(),
            "--file",
            file_path.to_str().unwrap(),
            "--cow",
            "cli",
        ]);
        let cfg = build_scene_config(&args).unwrap();
        assert_eq!(cfg.cow, "cli"); // CLI wins
        assert_eq!(cfg.text, "fromfile"); // file wins over config
        assert_eq!(cfg.fps, 45); // file wins over config
    }
}
