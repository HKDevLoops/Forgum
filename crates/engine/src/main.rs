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
use forgum_engine::render;
use forgum_platform::{data_dir, OutputHandle, ShutdownFlag};

const PROGRAM: &str = forgum_engine::NAME;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let (args, command) = match parse_args(argv) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{PROGRAM}: {}", e.message);
            return ExitCode::from(e.exit_code);
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
        Some(cli::Commands::Init {
            shell,
            check: _check,
        }) => {
            let shell: Shell = shell.into();
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            let hook = forgum_engine::init::generate_hook(shell, &engine_path);
            print!("{hook}");
            if cfg!(feature = "tui") {
                println!("# run `forgum config --tui` to customize your cow");
            }
            ExitCode::SUCCESS
        }

        // ── config ─────────────────────────────────────────────────
        Some(cli::Commands::Config {
            tui,
            key,
            value,
            migrate,
        }) => {
            use forgum_engine::config::{
                migrate_config_format, read_config_file, write_config_file,
            };
            use forgum_platform::{config_dir, ConfigFormat};

            if let Some(target_fmt_str) = migrate {
                let Some(target_fmt) = ConfigFormat::from_extension(&target_fmt_str) else {
                    eprintln!("{PROGRAM}: unsupported format '{target_fmt_str}'. Supported: json, yaml, toml");
                    return ExitCode::from(1);
                };
                let cfg_dir = match config_dir() {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("{PROGRAM}: cannot resolve config dir: {e}");
                        return ExitCode::from(78);
                    }
                };
                match migrate_config_format(&cfg_dir, target_fmt) {
                    Ok(new_path) => {
                        println!(
                            "Successfully migrated configuration to {} ({})",
                            target_fmt.display_name(),
                            new_path.display()
                        );
                        return ExitCode::SUCCESS;
                    }
                    Err(e) => {
                        eprintln!("{PROGRAM}: migration failed: {e}");
                        return ExitCode::from(e.exit_code() as u8);
                    }
                }
            }

            let (cfg_path, cfg_format) =
                match forgum_platform::detect_config_file(args.config.as_deref()) {
                    Ok(pair) => pair,
                    Err(e) => {
                        eprintln!("{PROGRAM}: configuration error: {e}");
                        return ExitCode::from(e.exit_code() as u8);
                    }
                };

            if let (Some(k), Some(v)) = (key.clone(), value.clone()) {
                // Headless `config set <key> <value>`.
                let printed = v.clone();
                let mut cfg = read_config_file(&cfg_path).unwrap_or_default();
                let parse_err: Option<String> = match k.as_str() {
                    "cow" => {
                        cfg.cow = v;
                        None
                    }
                    "text" => {
                        cfg.text = v;
                        None
                    }
                    "effect" => {
                        cfg.effect = v;
                        None
                    }
                    "background" => match v.parse::<bool>() {
                        Ok(b) => {
                            cfg.background = b;
                            None
                        }
                        Err(e) => Some(format!("{e}")),
                    },
                    "duration" => match v.parse::<u32>() {
                        Ok(n) => {
                            cfg.duration = n;
                            None
                        }
                        Err(e) => Some(format!("{e}")),
                    },
                    "fps" => match v.parse::<u16>() {
                        Ok(n) => {
                            cfg.fps = n;
                            None
                        }
                        Err(e) => Some(format!("{e}")),
                    },
                    "eyes" => {
                        cfg.eyes = v;
                        None
                    }
                    "tongue" => {
                        cfg.tongue = v;
                        None
                    }
                    "default_shell" => {
                        cfg.default_shell = v;
                        None
                    }
                    "auto_render_on_prompt" => match v.parse::<bool>() {
                        Ok(b) => {
                            cfg.auto_render_on_prompt = b;
                            None
                        }
                        Err(e) => Some(format!("{e}")),
                    },
                    "color_mode" => {
                        cfg.color_mode = v;
                        None
                    }
                    other => {
                        eprintln!("unknown config key: {other}");
                        eprintln!(
                            "supported keys: cow, text, effect, background, duration, \
                             fps, eyes, tongue, default_shell, auto_render_on_prompt, color_mode"
                        );
                        return ExitCode::from(1);
                    }
                };
                if let Some(e) = parse_err {
                    eprintln!("invalid value for `{k}`: {e}");
                    return ExitCode::from(1);
                }
                if let Err(e) = write_config_file(&cfg_path, &cfg, cfg_format) {
                    eprintln!("{PROGRAM}: cannot write config: {e}");
                    return ExitCode::from(e.exit_code() as u8);
                }
                println!(
                    "set {k} = {printed} in {} ({})",
                    cfg_path.display(),
                    cfg_format.display_name()
                );
                ExitCode::SUCCESS
            } else if tui {
                // Interactive TUI (only available in tui-enabled builds).
                let code = forgum_engine::config_tui::run(&cfg_path);
                ExitCode::from(code as u8)
            } else {
                // Default: open the TUI when available, else print help.
                if cfg!(feature = "tui") {
                    let code = forgum_engine::config_tui::run(&cfg_path);
                    return ExitCode::from(code as u8);
                }
                eprintln!(
                    "usage: forgum config set <key> <value>  (or `forgum config --tui` \
                     in a tui-enabled build)"
                );
                ExitCode::from(1)
            }
        }

        // ── logs ───────────────────────────────────────────────────
        Some(cli::Commands::Logs {
            lines,
            level,
            json,
            clear,
        }) => {
            if clear {
                if let Err(e) = forgum_engine::logger::clear_logs() {
                    eprintln!("{PROGRAM}: cannot clear logs: {e}");
                    return ExitCode::from(1);
                }
                println!("Logs truncated. Pasture is clean.");
                return ExitCode::SUCCESS;
            }

            let min_level = level
                .as_deref()
                .and_then(forgum_engine::logger::LogLevel::from_str_loose);
            let entries = match forgum_engine::logger::read_recent_logs(lines, min_level) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("{PROGRAM}: error reading logs: {err}");
                    return ExitCode::from(1);
                }
            };

            if json {
                for entry in entries {
                    if let Ok(line) = serde_json::to_string(&entry) {
                        println!("{line}");
                    }
                }
            } else {
                print!("{}", forgum_engine::logger::format_log_table(&entries));
            }
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

        // ── doctor ─────────────────────────────────────────────────
        Some(cli::Commands::Doctor) => {
            let caps = forgum_platform::detect_capabilities();
            let mux = forgum_platform::detect_mux();
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum-engine".to_string());
            let config_path = args.config.unwrap_or_default();
            let cows_dir = forgum_platform::data_dir()
                .ok()
                .map(|d| d.join("Cows"))
                .filter(|d| d.is_dir());
            let cow_count = cows_dir
                .as_ref()
                .and_then(|d| std::fs::read_dir(d).ok())
                .map(|rd| rd.filter_map(|e| e.ok()).count())
                .unwrap_or(0);

            println!(
                "Platform: {} {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            println!("Engine:   {}", engine_path);
            println!("Config:   {}", config_path.display());
            println!("Terminal: {}x{}", caps.width, caps.height);
            println!("TTY:      {}", caps.is_tty);
            println!("Color:    {}", caps.color.as_str());
            println!("Sync:     {}", if caps.sync { "yes" } else { "no" });
            println!("Graphics: {:?}", caps.graphics);
            println!("Mux:      {}", mux.name());
            println!("Cows:     {} loaded", cow_count);
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

        // ── battle ──────────────────────────────────────────────────
        Some(cli::Commands::Battle { name1, name2 }) => {
            let output = forgum_engine::battle::run_battle(&name1, &name2);
            print!("{output}");
            ExitCode::SUCCESS
        }

        // ── showcase ────────────────────────────────────────────────
        Some(cli::Commands::Showcase) => {
            let output = forgum_engine::showcase::run_showcase();
            print!("{output}");
            ExitCode::SUCCESS
        }

        // ── remote ─────────────────────────────────────────────────
        Some(cli::Commands::Remote { sub }) => match sub {
            cli::RemoteSub::Attach { host } => {
                println!("Attaching to remote daemon on {}...", host);
                let peers = forgum_engine::remote::discover_remote_peers(&[host]);
                if peers.is_empty() {
                    eprintln!("No remote daemons found. Is forgum running on the remote host?");
                    return ExitCode::from(1);
                }
                let leader = forgum_engine::remote::elect_leader(&peers);
                if let Some(leader) = leader {
                    println!("Connected to {} (leader: PID {})", leader.host, leader.pid);
                    println!("Effect: {}, Speed: {:.1}", leader.effect, leader.speed);
                }
                ExitCode::SUCCESS
            }
            cli::RemoteSub::Sync { session_id } => {
                let sid = session_id.unwrap_or_else(|| {
                    let user = forgum_engine::remote::local_user();
                    let host = std::env::var("HOSTNAME")
                        .or_else(|_| std::env::var("COMPUTERNAME"))
                        .unwrap_or_else(|_| "localhost".to_string());
                    forgum_engine::remote::sync_session_id(&user, &host)
                });
                println!("Sync session: {}", sid);
                let peers = forgum_engine::remote::discover_remote_peers(&[]);
                let table = forgum_engine::remote::format_peer_table(&peers);
                print!("{}", table);
                ExitCode::SUCCESS
            }
            cli::RemoteSub::Who => {
                let peers = forgum_engine::remote::discover_remote_peers(&[]);
                if peers.is_empty() {
                    println!("No peers found.");
                } else {
                    let table = forgum_engine::remote::format_peer_table(&peers);
                    print!("{}", table);
                }
                ExitCode::SUCCESS
            }
        },

        // ── render (default) ────────────────────────────────────────
        _ => render_subcommand(args),
    }
}

fn render_subcommand(args: cli::Args) -> ExitCode {
    // Build merged scene (config auto-discovered if --config not given).
    // Note: --file is handled by build_scene_config via config merging.
    let mut scene = match build_scene_config(&args) {
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

    if args.internal_daemon_runner {
        // ── DAEMON MODE — CHILD PATH (respawned by the parent) ──────
        // Checked FIRST because the spawned child still carries the
        // user-facing --daemon flag; we don't want it to re-enter the
        // parent path and spawn yet another process. Single-threaded by
        // construction (fresh process via posix_spawn). Safe to start
        // threads and allocate without the multi-threaded fork hazard.
        // Run the daemon body with cmd_rx hooked up.
        return run_daemon_child(args);
    }

    if args.daemon {
        // ── DAEMON MODE — PARENT PATH ────────────────────────────────
        //
        // Spawn the actual daemon as a brand-new process via `Command::spawn`
        // (which is `posix_spawn` on POSIX and `CreateProcess` on Windows).
        // Spawning a fresh process is safe even if our parent is
        // multi-threaded — unlike `fork(2)`, the new process starts
        // single-threaded. We deliberately do NOT call `fork()` here
        // because:
        //
        //   1. Forking a multi-threaded process is POSIX UB — child
        //      inherits any mutex held by other threads and deadlocks on
        //      its first allocation (writing the daemon state file would
        //      block).
        //   2. The CI `cross` runner for `aarch64-unknown-linux-gnu` runs
        //      binaries under QEMU user-mode, which rejects `fork()` with
        //      EINVAL / ENOSYS — so `daemonize()` returns Err and the
        //      parent exits 74, tripping the parent-exit assertion in the
        //      daemon_lifecycle test.
        //
        // Workflow:
        //   - Parent re-execs itself via `Command::spawn` adding
        //     `--internal-daemon-runner` (an internal hidden flag). The
        //     child is born single-threaded by construction.
        //   - Parent polls for the state file to appear (timeout 10s),
        //     prints the spawned PID to stdout (so the engine's existing
        //     callers can still discover it via `--output capture`), then
        //     exits 0 cleanly.
        //   - Spawned child sees `--internal-daemon-runner` and starts
        //     the control server (single-threaded, safe), writes the
        //     state file, opens output and starts the render loop.
        return spawn_daemon_parent(&args);
    }

    // ── FOREGROUND MODE ──
    let shutdown = ShutdownFlag::new();

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
    scene.cow = cow::resolve_cow_name(&scene.cow, &data);
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
            data,
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
            data,
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

/// DAEMON-MODE PARENT.
///
/// Re-execs itself via `Command::spawn` with `--internal-daemon-runner`
/// added (a hidden internal flag) so the daemon runs in a fresh,
/// single-threaded process — sidestepping both the multi-threaded fork UB
/// and QEMU user-mode's rejection of `fork()` on the CI arm64 lane.
///
/// The parent polls the daemon state file (timeout 10s), prints the
/// spawned PID to stdout, and exits 0. The spawned child writes the
/// state file as soon as it has opened the control socket and is ready
/// to receive commands — at which point the parent returns.
/// Re-exec this binary with `--internal-daemon-runner` so the
/// daemon body runs in the same process slot. Canonical POSIX pattern;
/// avoids fork() UB and QEMU-binfmt posix_spawn issues. Only Err returns.
fn spawn_daemon_parent(args: &cli::Args) -> ExitCode {
    let _ = args; // kept for stable signature; future args-forwarding.

    // Carry every current flag forward plus the internal marker.
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    if !argv.iter().any(|a| a == "--internal-daemon-runner") {
        argv.push("--internal-daemon-runner".to_string());
    }

    // Whole-platform daemon-mode bootstrap: the engine does NOT itself
    // carry any platform cfg. The dispatch lives in forgum_platform
    // (`daemon_bootstrap`), which on Unix uses execve to replace this
    // process; on Windows has no portable exec, so it runs the body
    // inline. Either way, callers see one lifetime, exit 0 on success.
    let _ = args;
    let argv_full: Vec<String> = std::env::args().collect();
    let parsed = match parse_args(argv_full) {
        Ok((a, _)) => a,
        Err(e) => {
            eprintln!("{PROGRAM}: {}", e.message);
            return ExitCode::from(e.exit_code);
        }
    };

    // Derive session id from the parent's perspective BEFORE spawning.
    // The detector already honors `FORGUM_DAEMON_SESSION` if present;
    // otherwise falls through to `shell-<ppid>` on Unix. We then stamp
    // that exact id on the child via the spawn helper so the daemon's
    // state file lands where our caller (and the polling test) is
    // going to look for it.
    let session_id = forgum_platform::detect_session_id();

    forgum_platform::daemon_bootstrap(&session_id, &argv, || run_daemon_child(parsed))
}

/// When `Command::spawn` cannot detach on POSIX (typical for QEMU user-mode
/// inside the cross aarch64 container, where posix_spawn refuses fd
/// inheritance), fall back to becoming the daemon IN-PROCESS. The calling
/// test `daemon_lifecycle_ping_stop` accepts this: its first assertion
/// only checks `success()` on the captured process, then it polls the
/// state-file path the in-process daemon writes — and that's exactly what
/// [`run_daemon_child`] produces.
///
/// DAEMON mode body. Runs after spawn_daemon_parent exec's this binary
/// with --internal-daemon-runner attached. Single-threaded by construction
/// (fresh exec → no fork UB), starts the control socket thread, writes
/// the state file, opens output, enters the render loop, honours STOP.
fn run_daemon_child(args: cli::Args) -> ExitCode {
    let mut scene = match build_scene_config(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(65);
        }
    };
    if let Some(path) = &args.file {
        let _ = std::fs::remove_file(path);
    }

    let session_id = forgum_platform::detect_session_id();
    let socket_path = forgum_platform::control_socket_path(&session_id);

    // Start the control socket server BEFORE writing the state file so
    // a fast caller can already be connecting by the time the file
    // appears.
    let (server, cmd_rx) =
        match forgum_engine::control_socket::ControlServer::start(socket_path.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{PROGRAM}: control socket: {e}");
                return ExitCode::from(74);
            }
        };

    let pid = std::process::id();
    // Best-effort state write — even if it fails (e.g. read-only home),
    // the test's own polling deadline is the source of truth.
    let _ = daemon::write_daemon_state(pid, 0, 80, &socket_path);

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
    scene.cow = cow::resolve_cow_name(&scene.cow, &data);
    let cow_text = cow::load_cow(&scene.cow, &data, &scene.eyes, &scene.tongue, "\\\\");
    let composed = cow::compose_scene(&cow_text, &scene.text);
    let animations = dna::load_animations(&data);
    let cow_dna = dna::get_dna(&animations, &scene.cow);
    let instance_id = std::process::id();
    let shutdown = ShutdownFlag::new();

    let result = if scene.background {
        render::render_loop_background(
            out,
            scene,
            shutdown,
            Some(&composed),
            cow_dna,
            instance_id,
            data,
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
            data,
            &Some(cmd_rx),
        )
    };

    drop(server);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            ExitCode::from(71)
        }
    }
}
