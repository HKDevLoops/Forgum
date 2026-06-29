//! `forgum-engine` — the Forgum animation engine binary.
//!
//! Phase 2: clap CLI, native cow renderer, fortune, shell hooks, completions.
//! Phase 0: RAII guards, signal handlers, no keystroke reads, `duration=0` semantics.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::CommandFactory;
use forgum_engine::cli;
use forgum_engine::cli::{build_scene_config, parse_args};
use forgum_engine::cow;
use forgum_engine::daemon;
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

        // ── tmux install ───────────────────────────────────────────
        Some(cli::Commands::Tmux {
            sub: cli::TmuxSub::Install,
        }) => {
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            print!(
                "{}",
                forgum_engine::init::generate_tmux_config(&engine_path)
            );
            ExitCode::SUCCESS
        }

        // ── tmux zellij ─────────────────────────────────────────
        Some(cli::Commands::Tmux {
            sub: cli::TmuxSub::Zellij,
        }) => {
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            print!(
                "{}",
                forgum_engine::init::generate_zellij_config(&engine_path)
            );
            ExitCode::SUCCESS
        }

        // ── tmux wezterm ────────────────────────────────────────
        Some(cli::Commands::Tmux {
            sub: cli::TmuxSub::WezTerm,
        }) => {
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            print!(
                "{}",
                forgum_engine::init::generate_wezterm_config(&engine_path)
            );
            ExitCode::SUCCESS
        }

        // ── tmux screen ─────────────────────────────────────────
        Some(cli::Commands::Tmux {
            sub: cli::TmuxSub::Screen,
        }) => {
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            print!(
                "{}",
                forgum_engine::init::generate_screen_config(&engine_path)
            );
            ExitCode::SUCCESS
        }

        // ── status-line ────────────────────────────────────────────
        Some(cli::Commands::StatusLine { max_len }) => {
            let line = forgum_engine::status_line::render_status_line(max_len);
            print!("{line}");
            ExitCode::SUCCESS
        }

        // ── herd list ──────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::List,
        }) => {
            let entries = forgum_engine::herd::discover_daemons();
            print!("{}", forgum_engine::herd::format_table(&entries));
            ExitCode::SUCCESS
        }

        // ── herd stop ──────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Stop { session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_stop(&filter) {
                Ok(n) => {
                    println!("Stopped {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd stop: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd effect ────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Effect { name, session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_effect(&name, &filter) {
                Ok(n) => {
                    println!("Set effect on {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd effect: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd speed ─────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub:
                cli::HerdSub::Speed {
                    value,
                    session,
                    all,
                },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_speed(value, &filter) {
                Ok(n) => {
                    println!("Set speed on {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd speed: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd pause ─────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Pause { session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_pause(&filter) {
                Ok(n) => {
                    println!("Paused {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd pause: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd resume ────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Resume { session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_resume(&filter) {
                Ok(n) => {
                    println!("Resumed {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd resume: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd quiet ─────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Quiet,
        }) => match forgum_engine::herd::herd_quiet() {
            Ok(n) => {
                println!("Quieted {n} daemon(s).");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{PROGRAM}: herd quiet: {e}");
                ExitCode::from(1)
            }
        },

        // ── herd census ───────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Census,
        }) => {
            let entries = forgum_engine::herd::herd_census();
            print!("{}", forgum_engine::herd::format_table(&entries));
            ExitCode::SUCCESS
        }

        // ── theme list ─────────────────────────────────────────────
        Some(cli::Commands::Theme {
            sub: cli::ThemeSub::List,
        }) => {
            let config_dir = forgum_platform::config_path()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            let themes = forgum_engine::theme::list_themes(&config_dir);
            if themes.is_empty() {
                println!("No themes found.");
            } else {
                for name in &themes {
                    println!("{name}");
                }
            }
            ExitCode::SUCCESS
        }

        // ── theme apply ────────────────────────────────────────────
        Some(cli::Commands::Theme {
            sub: cli::ThemeSub::Apply { name },
        }) => {
            let config_dir = forgum_platform::config_path()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            match forgum_engine::theme::load_theme(&config_dir, &name) {
                Ok(theme) => {
                    let filter = forgum_engine::herd::HerdFilter {
                        session: None,
                        all: true,
                    };
                    match theme.apply(&filter) {
                        Ok(n) => {
                            println!("Applied theme '{name}' to {n} daemon(s).");
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("{PROGRAM}: theme apply: {e}");
                            ExitCode::from(1)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: theme apply: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── theme rotate ────────────────────────────────────────────
        Some(cli::Commands::Theme {
            sub: cli::ThemeSub::Rotate { interval },
        }) => {
            if let Err(e) = forgum_engine::demo::run_theme_rotate(interval) {
                eprintln!("{PROGRAM}: theme rotate: {e}");
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }

        // ── theme seasonal ──────────────────────────────────────────
        Some(cli::Commands::Theme {
            sub: cli::ThemeSub::Seasonal,
        }) => {
            let theme = forgum_engine::theme::seasonal_theme();
            println!(
                "Seasonal theme: {} effect, {} cow, eyes={}, tongue={}",
                theme.effect.as_deref().unwrap_or("default"),
                theme.cow.as_deref().unwrap_or("default"),
                theme.eyes.as_deref().unwrap_or("oo"),
                theme.tongue.as_deref().unwrap_or("U")
            );
            ExitCode::SUCCESS
        }

        // ── herd follow ────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Follow { pane },
        }) => match forgum_engine::herd::herd_follow(pane.as_deref()) {
            Ok(n) => {
                println!("Follow mode set on {n} daemon(s).");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{PROGRAM}: herd follow: {e}");
                ExitCode::from(1)
            }
        },

        // ── demo ───────────────────────────────────────────────────
        Some(cli::Commands::Demo) => match forgum_engine::demo::run_demo() {
            Ok(output) => {
                print!("{output}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{PROGRAM}: demo: {e}");
                ExitCode::from(1)
            }
        },

        // ── say ─────────────────────────────────────────────────────
        Some(cli::Commands::Say { cmd }) => {
            let output = forgum_engine::say::run_say(&cmd);
            print!("{output}");
            ExitCode::SUCCESS
        }

        // ── timer ───────────────────────────────────────────────────
        Some(cli::Commands::Timer { cmd }) => {
            let result = forgum_engine::timer::run_timer(&cmd);
            let cow = forgum_engine::timer::render_timer_cow(&result);
            println!("{cow}");
            if !result.stdout.is_empty() {
                println!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprintln!("{}", result.stderr);
            }
            ExitCode::from(result.exit_code as u8)
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

    let shutdown = ShutdownFlag::new();

    if args.daemon {
        // ── DAEMON MODE ──
        let session_id = forgum_platform::detect_session_id();
        let socket_path = forgum_platform::control_socket_path(&session_id);

        // Start control socket server before forking.
        let (server, cmd_rx) =
            match forgum_engine::control_socket::ControlServer::start(socket_path.clone()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{PROGRAM}: control socket: {e}");
                    return ExitCode::from(74);
                }
            };

        // Daemonize: parent exits, child continues.
        match forgum_platform::daemonize() {
            Ok(true) => {
                unreachable!();
            }
            Ok(false) => {
                // Child: write daemon state.
                let pid = std::process::id();
                let _ = daemon::write_daemon_state(pid, 0, 80, &socket_path);
            }
            Err(e) => {
                eprintln!("{PROGRAM}: daemonize: {e}");
                return ExitCode::from(74);
            }
        }

        // Open output and render (same as foreground, but with cmd_rx).
        let out = match OutputHandle::open() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot open output: {e}");
                return ExitCode::from(e.exit_code() as u8);
            }
        };

        let data = match data_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot find data directory: {e}");
                return ExitCode::from(78);
            }
        };
        let cow_text = cow::load_cow(&scene.cow, &data, &scene.eyes, &scene.tongue, "\\\\");
        let composed = cow::compose_scene(&cow_text, &scene.text);
        let animations = dna::load_animations(&data);
        let cow_dna = dna::get_dna(&animations, &scene.cow);
        let instance_id = std::process::id();

        let result = if scene.background {
            render::render_loop_background(
                out,
                scene,
                shutdown,
                Some(&composed),
                cow_dna,
                instance_id,
                &Some(cmd_rx),
            )
        } else {
            render::render_loop_foreground(
                out,
                scene,
                shutdown,
                Some(&composed),
                cow_dna,
                instance_id,
                &Some(cmd_rx),
            )
        };

        // Cleanup on exit.
        drop(server);

        match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{PROGRAM}: {e}");
                ExitCode::from(71)
            }
        }
    } else {
        // ── FOREGROUND MODE ──
        let out = match OutputHandle::open() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot open output: {e}");
                return ExitCode::from(e.exit_code() as u8);
            }
        };

        let data = match data_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot find data directory: {e}");
                return ExitCode::from(78);
            }
        };
        let cow_text = cow::load_cow(&scene.cow, &data, &scene.eyes, &scene.tongue, "\\\\");
        let composed = cow::compose_scene(&cow_text, &scene.text);

        let animations = dna::load_animations(&data);
        let cow_dna = dna::get_dna(&animations, &scene.cow);
        let instance_id = std::process::id();

        let result = if scene.background {
            render::render_loop_background(
                out,
                scene,
                shutdown,
                Some(&composed),
                cow_dna,
                instance_id,
                &None,
            )
        } else {
            render::render_loop_foreground(
                out,
                scene,
                shutdown,
                Some(&composed),
                cow_dna,
                instance_id,
                &None,
            )
        };

        match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{PROGRAM}: {e}");
                ExitCode::from(71)
            }
        }
    }
}
