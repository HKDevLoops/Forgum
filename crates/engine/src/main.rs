//! `forgum-engine` — the Forgum animation engine binary.
//!
//! Phase 0 implements only the safety/perf/contract fixes from `01-BUGS`:
//! RAII guards, signal handlers, no keystroke reads, `duration=0` semantics,
//! `Cell::dirty` PartialEq, bounded stdin, non-zero exit on parse error.
//!
//! Real rendering effects (aurora, plasma, particles) land in Phase 3. Real
//! cow file parsing lands in Phase 2. The CLI is hand-rolled in Phase 0;
//! `clap` lands in Phase 2.

use std::process::ExitCode;

use forgum_engine::cli;
use forgum_engine::cli::{build_scene_config, parse_args};
use forgum_engine::protocol_io::read_scene;
use forgum_engine::render;
use forgum_platform::{OutputHandle, ShutdownFlag};

const VERSION: &str = forgum_engine::VERSION;
const PROGRAM: &str = forgum_engine::NAME;

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            eprintln!("Try '{PROGRAM} --help'.");
            return ExitCode::from(64);
        }
    };

    if args.show_version {
        println!("{PROGRAM} {VERSION}");
        return ExitCode::SUCCESS;
    }
    if args.show_help {
        print_help();
        return ExitCode::SUCCESS;
    }

    // Read scene: --file overrides stdin; if neither, use defaults.
    let scene_from_file = match read_scene(args.file.as_deref()) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(e.exit_code() as u8);
        }
    };
    let _ = scene_from_file; // currently unused — CLI overrides land in build_scene_config via --config

    // Build merged scene.
    let scene = match build_scene_config(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(65);
        }
    };

    // Daemon mode stub — Phase 1 will replace this with setsid + PID file.
    if args.daemon {
        eprintln!("{PROGRAM}: --daemon is a Phase 1 feature; running foreground instead.");
    }

    // Control-socket stub — Phase 1.
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

    let result = match args.command {
        cli::Command::Status => {
            println!("ok");
            Ok(())
        }
        cli::Command::Render => {
            if scene.background {
                render::render_loop_background(out, scene, shutdown)
            } else {
                render::render_loop_foreground(out, scene, shutdown)
            }
        }
        cli::Command::Unknown(cmd) => {
            eprintln!("{PROGRAM}: unknown command: {cmd}");
            eprintln!("Try '{PROGRAM} --help'.");
            return ExitCode::from(64);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            ExitCode::from(71)
        }
    }
}

fn print_help() {
    println!(
        "{PROGRAM} {VERSION} — Forgum animation engine

USAGE:
    {PROGRAM} <command> [OPTIONS]

COMMANDS:
    render    Render a cow (default).
    status    Print 'ok' and exit (for daemon health checks).
    help      Print this message.

RENDER OPTIONS:
    --cow <name>            Cow file basename (default: default)
    --text <s>              Text inside the speech bubble
    --effect <name>         Effect name (default: static)
    --background            Render above prompt as a non-blocking overlay
    --duration <seconds>    Seconds; 0 = infinite (default: 0 with --background)
    --fps <n>               Target FPS (default: 30)
    --config <path>         Path to config JSON
    --file <path>           Path to scene JSON (alternative to stdin)
    --daemon                (Phase 1) Spawn detached; prints warning in Phase 0
    --control-socket <path> (Phase 1) Control socket path; ignored in Phase 0

EXIT CODES:
    0   success
    64  usage error (unknown flag, missing value)
    65  data error (invalid JSON, oversized input)
    71  OS error (signal/detach failure)
    78  configuration error (path escape, no terminal)

EXAMPLES:
    {PROGRAM} render --text 'hello world'
    {PROGRAM} render --cow tux --background --duration 0
    echo '{{\"text\":\"hi\"}}' | {PROGRAM} render --background
"
    );
}
