//! `forgum-engine` — the Forgum animation engine binary.
//!
//! Phase 2: clap CLI, native cow renderer, fortune, shell hooks, completions.
//! Phase 0: RAII guards, signal handlers, no keystroke reads, `duration=0` semantics.

use std::process::ExitCode;

use clap::CommandFactory;
use forgum_engine::cli;
use forgum_engine::cli::{build_scene_config, parse_args};
use forgum_engine::cow;
use forgum_engine::dna;
use forgum_engine::fortune;
use forgum_engine::init::Shell;
use forgum_engine::protocol_io::read_scene;
use forgum_engine::render;
use forgum_platform::{data_dir, OutputHandle, ShutdownFlag};

const PROGRAM: &str = forgum_engine::NAME;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let (args, command) = match parse_args(argv) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(64);
        }
    };

    match command {
        // ── fortune ──────────────────────────────────────────────────
        Some(cli::Commands::Fortune) => {
            let data = match data_dir() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("{PROGRAM}: cannot find data directory: {e}");
                    return ExitCode::from(78);
                }
            };
            match fortune::random_fortune(&data) {
                Some(f) => {
                    println!("{f}");
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!("{PROGRAM}: no fortunes found in {}", data.display());
                    ExitCode::from(78)
                }
            }
        }

        // ── init <shell> ────────────────────────────────────────────
        Some(cli::Commands::Init { shell }) => {
            let shell: Shell = shell.into();
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            let hook = forgum_engine::init::generate_hook(shell, &engine_path);
            print!("{hook}");
            ExitCode::SUCCESS
        }

        // ── completions <shell> ──────────────────────────────────────
        Some(cli::Commands::Completions { shell }) => {
            let mut cmd = forgum_engine::cli::Cli::command();
            let shell: Shell = shell.into();
            if let Err(e) = forgum_engine::completions::generate_completions(shell, &mut cmd) {
                eprintln!("{PROGRAM}: completions: {e}");
                return ExitCode::from(71);
            }
            ExitCode::SUCCESS
        }

        // ── status ──────────────────────────────────────────────────
        Some(cli::Commands::Status) | None if args.command == cli::Command::Status => {
            println!("ok");
            ExitCode::SUCCESS
        }

        // ── render (default) ────────────────────────────────────────
        _ => render_subcommand(args),
    }
}

fn render_subcommand(args: cli::Args) -> ExitCode {
    // Read scene: --file overrides stdin; if neither, use defaults.
    let scene_from_file = match read_scene(args.file.as_deref(), false) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(e.exit_code() as u8);
        }
    };
    let _ = scene_from_file;

    // Build merged scene (config auto-discovered if --config not given).
    let scene = match build_scene_config(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(65);
        }
    };

    // Clean up temp file after building config (BUG-D2 fix).
    if let Some(path) = &args.file {
        let _ = std::fs::remove_file(path);
    }

    // Phase 1 stubs.
    if args.daemon {
        eprintln!("{PROGRAM}: --daemon is a Phase 1 feature; running foreground instead.");
    }
    if args.control_socket.is_some() {
        eprintln!("{PROGRAM}: --control-socket is a Phase 1 feature; ignoring.");
    }

    let shutdown = ShutdownFlag::new();

    // Open the output handle (stdout, /dev/tty, or CONOUT$).
    let out = match OutputHandle::open() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{PROGRAM}: cannot open output: {e}");
            return ExitCode::from(e.exit_code() as u8);
        }
    };

    // Load and compose cow art with speech bubble.
    let data = match data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{PROGRAM}: cannot find data directory: {e}");
            return ExitCode::from(78);
        }
    };
    let cow_text = cow::load_cow(&scene.cow, &data, &scene.eyes, &scene.tongue, "\\\\");
    let composed = cow::compose_scene(&cow_text, &scene.text);

    // Load animation DNA for this cow
    let animations = dna::load_animations(&data);
    let cow_dna = dna::get_dna(&animations, &scene.cow);
    let instance_id = std::process::id();

    let result = if scene.background {
        render::render_loop_background(out, scene, shutdown, Some(&composed), cow_dna, instance_id, &None)
    } else {
        render::render_loop_foreground(out, scene, shutdown, Some(&composed), cow_dna, instance_id, &None)
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            ExitCode::from(71)
        }
    }
}
