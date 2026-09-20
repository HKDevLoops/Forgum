//! `forgum` — the Forgum animation engine binary.
//!
//! Phase 2: clap CLI, native cow renderer, fortune, shell hooks, completions.
//! Phase 0: RAII guards, signal handlers, no keystroke reads, `duration=0` semantics.

use crate as forgum_engine;
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

const PROGRAM: &str = "forgum";

fn is_list_query(s: &str) -> bool {
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

pub fn run() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        crate::log_error!("panic", "PANIC occurred at {}: {}", location, msg);
        eprintln!("forgum panic at {}: {}", location, msg);
    }));

    let mut argv: Vec<String> = std::env::args().collect();
    if let Some(arg0) = argv.first() {
        let file_stem = std::path::Path::new(arg0)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_stem == "forgum-init" {
            argv.insert(1, "init".to_string());
        }
    }
    let (args, command) = match parse_args(argv) {
        Ok(v) => v,
        Err(e) => {
            if e.exit_code == 0 {
                print!("{}", e.message);
            } else {
                eprintln!("{PROGRAM}: {}", e.message);
            }
            return ExitCode::from(e.exit_code);
        }
    };

    if let Some(cat) = &args.list {
        let table = forgum_engine::options_table::render_options(cat);
        print!("{table}");
        return ExitCode::SUCCESS;
    }

    // Intercept any argument passed with 'list', '--list', '?', 'help', 'options'
    if let Some(cow) = &args.cow {
        if is_list_query(cow) {
            print!(
                "{}",
                forgum_engine::options_table::render_options("animals")
            );
            return ExitCode::SUCCESS;
        }
    }
    if let Some(eff) = &args.effect {
        if is_list_query(eff) {
            print!(
                "{}",
                forgum_engine::options_table::render_options("effects")
            );
            return ExitCode::SUCCESS;
        }
    }
    if let Some(anim) = &args.animation {
        if is_list_query(anim) {
            print!(
                "{}",
                forgum_engine::options_table::render_options("effects")
            );
            return ExitCode::SUCCESS;
        }
    }
    if let Some(at) = &args.animation_type {
        if is_list_query(at) {
            print!(
                "{}",
                forgum_engine::options_table::render_options("effects")
            );
            return ExitCode::SUCCESS;
        }
    }
    if let Some(env) = &args.environment {
        if is_list_query(env) {
            print!(
                "{}",
                forgum_engine::options_table::render_options("environments")
            );
            return ExitCode::SUCCESS;
        }
    }
    if let Some(road) = &args.road {
        if is_list_query(road) {
            print!("{}", forgum_engine::options_table::render_options("roads"));
            return ExitCode::SUCCESS;
        }
    }
    if let Some(mtn) = &args.mountain {
        if is_list_query(mtn) {
            print!(
                "{}",
                forgum_engine::options_table::render_options("mountains")
            );
            return ExitCode::SUCCESS;
        }
    }
    if let Some(cm) = &args.color_mode {
        if is_list_query(cm) {
            print!("{}", forgum_engine::options_table::render_options("colors"));
            return ExitCode::SUCCESS;
        }
    }
    if let Some(pal) = &args.palette {
        if is_list_query(pal) {
            print!("{}", forgum_engine::options_table::render_options("colors"));
            return ExitCode::SUCCESS;
        }
    }
    if let Some(eyes) = &args.eyes {
        if is_list_query(eyes) {
            print!("{}", forgum_engine::options_table::render_options("eyes"));
            return ExitCode::SUCCESS;
        }
    }
    if let Some(tongue) = &args.tongue {
        if is_list_query(tongue) {
            print!("{}", forgum_engine::options_table::render_options("tongue"));
            return ExitCode::SUCCESS;
        }
    }

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

        // ── think ───────────────────────────────────────────────────
        Some(cli::Commands::Think { thought, .. }) => handle_think_command(args, thought),

        // ── init <shell> ────────────────────────────────────────────
        Some(cli::Commands::Init {
            shell,
            check: _check,
            install,
            render_args,
        }) => {
            if matches!(shell, cli::ShellArg::List) {
                let table = forgum_engine::options_table::render_options("shells");
                print!("{table}");
                return ExitCode::SUCCESS;
            }
            let shell: Shell = shell.into();
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum".to_string());

            let mut combined_render_args = Vec::new();
            if let Some(cow) = &args.cow {
                combined_render_args.push(format!("--animal {cow}"));
            }
            if let Some(anim) = &args.animation {
                combined_render_args.push(format!("--animation {anim}"));
            }
            if let Some(at) = &args.animation_type {
                combined_render_args.push(format!("--animation-type {at}"));
            }
            if let Some(env) = &args.environment {
                combined_render_args.push(format!("--environment {env}"));
            }
            if let Some(road) = &args.road {
                combined_render_args.push(format!("--road {road}"));
            }
            if let Some(mountain) = &args.mountain {
                combined_render_args.push(format!("--mountain {mountain}"));
            }
            if let Some(cm) = &args.color_mode {
                combined_render_args.push(format!("--color-mode {cm}"));
            }
            if let Some(pal) = &args.palette {
                combined_render_args.push(format!("--palette {pal}"));
            }
            if let Some(ti) = args.thought_interval {
                combined_render_args.push(format!("--thought-interval {ti}"));
            }
            if args.split_scroll {
                combined_render_args.push("--split-scroll".to_string());
            }
            if let Some(rr) = args.reserve_rows {
                combined_render_args.push(format!("--reserve-rows {rr}"));
            }
            if let Some(rc) = args.reserve_cols {
                combined_render_args.push(format!("--reserve-cols {rc}"));
            }
            if let Some(sr) = args.split_ratio {
                combined_render_args.push(format!("--split-ratio {sr}"));
            }
            if let Some(eff) = &args.effect {
                combined_render_args.push(format!("--effect {eff}"));
            }
            if let Some(eyes) = &args.eyes {
                combined_render_args.push(format!("--eyes {eyes}"));
            }
            if let Some(tongue) = &args.tongue {
                combined_render_args.push(format!("--tongue {tongue}"));
            }
            combined_render_args.extend(render_args);

            let hook = forgum_engine::init::generate_hook_with_args(
                shell,
                &engine_path,
                &combined_render_args,
            );

            if install {
                match shell.shell_rc_path() {
                    Some(rc_path) => {
                        let existing = std::fs::read_to_string(&rc_path).unwrap_or_default();
                        let begin_marker = if matches!(shell, Shell::Cmd) {
                            "rem >>> forgum (cmd) >>>"
                        } else {
                            &format!("# >>> forgum ({shell}) >>>")
                        };
                        let end_marker = if matches!(shell, Shell::Cmd) {
                            "rem <<< forgum <<<"
                        } else {
                            "# <<< forgum <<<"
                        };
                        let updated = forgum_platform::shell::update_delimited_block(
                            &existing,
                            begin_marker,
                            end_marker,
                            &hook,
                        );
                        if let Some(parent) = rc_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match std::fs::write(&rc_path, updated) {
                            Ok(_) => {
                                println!("\x1b[1;32m✓\x1b[0m Forgum shell integration successfully installed into {}", rc_path.display());
                                match shell {
                                    Shell::Pwsh | Shell::PowerShell => {
                                        println!("\x1b[36m💡 Run `. $PROFILE` or restart your shell to activate.\x1b[0m");
                                    }
                                    _ => {
                                        println!("\x1b[36m💡 Run `source {}` or restart your shell to activate.\x1b[0m", rc_path.display());
                                    }
                                }
                                ExitCode::SUCCESS
                            }
                            Err(e) => {
                                eprintln!("{PROGRAM}: failed to write shell RC file {}: {e}", rc_path.display());
                                ExitCode::from(1)
                            }
                        }
                    }
                    None => {
                        eprintln!("{PROGRAM}: automatic installation is not supported for shell '{shell}'. Please copy the hook manually:");
                        print!("{hook}");
                        ExitCode::from(1)
                    }
                }
            } else {
                print!("{hook}");
                if cfg!(feature = "tui") {
                    if matches!(shell, Shell::Cmd) {
                        println!("rem run `forgum config --tui` to customize your cow");
                    } else {
                        println!("# run `forgum config --tui` to customize your cow");
                    }
                }
                ExitCode::SUCCESS
            }
        }

        // ── tui ─────────────────────────────────────────────────────
        Some(cli::Commands::Tui { tab }) => {
            let code = if let Some(path_str) = args.config.as_deref() {
                let p = std::path::PathBuf::from(path_str);
                forgum_engine::config_tui::run(&p)
            } else {
                forgum_engine::config_tui::run_standalone(if tab.is_empty() {
                    None
                } else {
                    Some(&tab)
                })
            };
            ExitCode::from(code as u8)
        }

        // ── install / setup wizard ─────────────────────────────────
        Some(cli::Commands::Install {
            headless,
            telemetry,
        }) => {
            if let Some(t) = telemetry {
                let allowed = matches!(t.to_lowercase().as_str(), "allow" | "yes" | "1" | "true");
                let _ = forgum_platform::set_telemetry_consent(allowed);
            }
            if headless {
                forgum_platform::record_tried();
                if let Ok(d) = forgum_platform::paths::config_dir() {
                    let _ = std::fs::create_dir_all(&d);
                    let cfg_path = d.join("config.json");
                    if !cfg_path.exists() {
                        let default_cfg = forgum_platform::protocol::SceneConfig::default();
                        if let Ok(s) = serde_json::to_string_pretty(&default_cfg) {
                            let _ = std::fs::write(&cfg_path, s);
                        }
                    }
                }
                forgum_platform::record_installed();
                println!("\x1b[1;32m✓\x1b[0m Headless Forgum installation completed.");
                ExitCode::SUCCESS
            } else {
                let code = forgum_engine::config_tui::run_installer_wizard();
                ExitCode::from(code as u8)
            }
        }

        // ── uninstall ──────────────────────────────────────────────
        Some(cli::Commands::Uninstall {
            method,
            non_interactive,
            tui,
        }) => {
            let is_tty = forgum_platform::terminal::detect_capabilities().is_tty;
            if (tui || (method.is_none() && !non_interactive)) && is_tty {
                let code = forgum_engine::config_tui::run_uninstaller_wizard();
                ExitCode::from(code as u8)
            } else {
                let mode = match method.as_deref().unwrap_or("soft").to_lowercase().as_str() {
                    "purge" | "clean" | "all" => forgum_platform::UninstallMode::Purge,
                    _ => forgum_platform::UninstallMode::Soft,
                };
                println!("\x1b[1;36m━━━ Forgum De-Orbit Uninstaller ━━━\x1b[0m");
                println!("Executing: \x1b[1;33m{}\x1b[0m\n", mode.title());
                let report = forgum_platform::perform_uninstallation(mode);
                for log in report.logs {
                    println!("  {log}");
                }
                for err in report.errors {
                    eprintln!("  \x1b[1;31m✗\x1b[0m {err}");
                }
                println!("\n\x1b[1;32m✓ Uninstallation completed.\x1b[0m");
                ExitCode::SUCCESS
            }
        }

        // ── update ─────────────────────────────────────────────────
        Some(cli::Commands::Update { check, channel }) => {
            handle_update_command(check, channel.as_deref())
        }

        // ── channel ────────────────────────────────────────────────
        Some(cli::Commands::Channel { action, name }) => {
            handle_channel_command(action.as_deref(), name.as_deref())
        }

        // ── config ─────────────────────────────────────────────────
        Some(cli::Commands::Config {
            tui,
            list,
            key,
            value,
            extra_value,
            migrate,
        }) => {
            if list || key.as_deref().is_some_and(is_list_query) || value.as_deref().is_some_and(is_list_query) {
                let table = forgum_engine::options_table::render_options("config");
                print!("{table}");
                return ExitCode::SUCCESS;
            }
            use forgum_engine::config::{
                migrate_config_format, read_config_file, write_config_file,
            };
            use forgum_platform::{config_dir, ConfigFormat};

            if let Some(target_fmt_str) = migrate.as_deref() {
                let Some(target_fmt) = ConfigFormat::from_extension(target_fmt_str) else {
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

            // Support:
            // 1. `config set <key> <val>`
            // 2. `config get <key>`
            // 3. `config <key> <val>` (implicit set)
            // 4. `config <key>` (implicit get)
            // 5. `config` / `config --tui` (open TUI)
            enum ConfigOp {
                Set(String, String),
                Get(String),
                OpenTui,
            }

            let op = match (key.as_deref(), value.as_deref(), extra_value.as_deref()) {
                (Some("set"), Some(k), Some(v)) => ConfigOp::Set(k.to_string(), v.to_string()),
                (Some("set"), Some(_k), None) => {
                    eprintln!("usage: forgum config set <key> <value>");
                    return ExitCode::from(1);
                }
                (Some("get"), Some(k), _) => ConfigOp::Get(k.to_string()),
                (Some(k), Some(v), _) => ConfigOp::Set(k.to_string(), v.to_string()),
                (Some(k), None, _) => ConfigOp::Get(k.to_string()),
                (None, None, _) => ConfigOp::OpenTui,
                _ => ConfigOp::OpenTui,
            };

            match op {
                ConfigOp::Get(k) => {
                    let cfg = read_config_file(&cfg_path).unwrap_or_default();
                    match k.as_str() {
                        "cow" => println!("{}", cfg.cow),
                        "text" => println!("{}", cfg.text),
                        "effect" => println!("{}", cfg.effect),
                        "background" => println!("{}", cfg.background),
                        "duration" => println!("{}", cfg.duration),
                        "fps" => println!("{}", cfg.fps),
                        "eyes" => println!("{}", cfg.eyes),
                        "tongue" => println!("{}", cfg.tongue),
                        "default_shell" => println!("{}", cfg.default_shell),
                        "auto_render_on_prompt" => println!("{}", cfg.auto_render_on_prompt),
                        "think" => println!("{}", cfg.think),
                        "color_mode" => println!("{}", cfg.color_mode),
                        "shell_attach_mode" | "attach_mode" => println!("{}", cfg.shell_attach_mode),
                        "environment" | "env" => println!("{}", cfg.environment.as_deref().unwrap_or("none")),
                        "road" => println!("{}", cfg.road.as_deref().unwrap_or("none")),
                        "mountain" | "mtn" => println!("{}", cfg.mountain.as_deref().unwrap_or("none")),
                        "palette" => println!("{}", cfg.palette.as_deref().unwrap_or("none")),
                        "thought_interval" => println!("{}", cfg.thought_interval),
                        "split_scroll" => println!("{}", cfg.split_scroll),
                        "reserve_rows" => println!("{}", cfg.reserve_rows.map(|n| n.to_string()).unwrap_or_else(|| "none".to_string())),
                        "reserve_cols" => println!("{}", cfg.reserve_cols.map(|n| n.to_string()).unwrap_or_else(|| "none".to_string())),
                        "split_ratio" => println!("{}", cfg.split_ratio.map(|r| r.to_string()).unwrap_or_else(|| "none".to_string())),
                        "animation" => println!("{}", cfg.animation.as_deref().unwrap_or("none")),
                        "animation_type" | "anim_type" => println!("{}", cfg.animation_type.as_deref().unwrap_or("none")),
                        "image" => println!("{}", cfg.image.as_deref().unwrap_or("none")),
                        "split_mode" => println!("{}", cfg.split_mode.as_deref().unwrap_or("none")),
                        "editor" => println!("{}", cfg.editor.as_deref().unwrap_or("auto")),
                        other => {
                            eprintln!("unknown config key: {other}");
                            eprintln!(
                                "supported keys: cow, text, effect, background, duration, \
                                 fps, eyes, tongue, default_shell, auto_render_on_prompt, think, \
                                 color_mode, shell_attach_mode, environment, road, mountain, \
                                 palette, thought_interval, split_scroll, split_mode, reserve_rows, reserve_cols, \
                                 split_ratio, animation, animation_type, image, editor"
                            );
                            return ExitCode::from(1);
                        }
                    }
                    ExitCode::SUCCESS
                }
                ConfigOp::Set(k, v) => {
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
                        "think" => match v.parse::<bool>() {
                            Ok(b) => {
                                cfg.think = b;
                                None
                            }
                            Err(e) => Some(format!("{e}")),
                        },
                        "color_mode" => {
                            cfg.color_mode = v;
                            None
                        }
                        "shell_attach_mode" | "attach_mode" => {
                            cfg.shell_attach_mode = v;
                            None
                        }
                        "environment" | "env" => {
                            cfg.environment = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "road" => {
                            cfg.road = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "mountain" | "mtn" => {
                            cfg.mountain = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "palette" => {
                            cfg.palette = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "thought_interval" => match v.parse::<u32>() {
                            Ok(n) => {
                                cfg.thought_interval = n;
                                None
                            }
                            Err(e) => Some(format!("{e}")),
                        },
                        "split_scroll" => match v.parse::<bool>() {
                            Ok(b) => {
                                cfg.split_scroll = b;
                                None
                            }
                            Err(e) => Some(format!("{e}")),
                        },
                        "reserve_rows" => match v.parse::<u16>() {
                            Ok(n) => {
                                cfg.reserve_rows = Some(n);
                                None
                            }
                            Err(e) => Some(format!("{e}")),
                        },
                        "reserve_cols" => match v.parse::<u16>() {
                            Ok(n) => {
                                cfg.reserve_cols = Some(n);
                                None
                            }
                            Err(e) => Some(format!("{e}")),
                        },
                        "split_ratio" => match v.parse::<f32>() {
                            Ok(r) => {
                                cfg.split_ratio = Some(r);
                                None
                            }
                            Err(e) => Some(format!("{e}")),
                        },
                        "animation" => {
                            cfg.animation = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "animation_type" | "anim_type" => {
                            cfg.animation_type = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "image" => {
                            cfg.image = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "split_mode" => {
                            cfg.split_mode = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        "editor" => {
                            cfg.editor = if v.is_empty() || v == "none" {
                                None
                            } else {
                                Some(v)
                            };
                            None
                        }
                        other => {
                            eprintln!("unknown config key: {other}");
                            eprintln!(
                                "supported keys: cow, text, effect, background, duration, \
                                 fps, eyes, tongue, default_shell, auto_render_on_prompt, think, \
                                 color_mode, shell_attach_mode, environment, road, mountain, \
                                 palette, thought_interval, split_scroll, split_mode, reserve_rows, reserve_cols, \
                                 split_ratio, animation, animation_type, image, editor"
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
                }
                ConfigOp::OpenTui => {
                    if tui || cfg!(feature = "tui") {
                        let code = forgum_engine::config_tui::run(&cfg_path);
                        ExitCode::from(code as u8)
                    } else {
                        eprintln!(
                            "usage: forgum config set <key> <value>  (or `forgum config --tui` in a tui-enabled build)"
                        );
                        ExitCode::from(1)
                    }
                }
            }
        }

        Some(cli::Commands::Logs {
            lines,
            level,
            json,
            follow,
            path,
            open,
            raw,
            filter,
            clear,
            diagnose,
        }) => {
            if path {
                if let Some((dir, text, jsonl)) = forgum_engine::logger::get_log_paths() {
                    println!("Log Directory: {}", dir.display());
                    println!("Text Log:      {}", text.display());
                    println!("JSONL Log:     {}", jsonl.display());
                } else {
                    eprintln!("{PROGRAM}: cannot determine log directory");
                }
                return ExitCode::SUCCESS;
            }

            if open {
                match forgum_engine::logger::open_log_in_system(false) {
                    Ok(opened_path) => {
                        println!(
                            "Opened log in default system viewer: {}",
                            opened_path.display()
                        );
                        return ExitCode::SUCCESS;
                    }
                    Err(e) => {
                        eprintln!("{PROGRAM}: unable to open log viewer: {e}");
                        return ExitCode::from(1);
                    }
                }
            }

            if raw {
                match forgum_engine::logger::read_raw_log() {
                    Ok(content) => {
                        if content.is_empty() {
                            println!("Log file is empty or no events recorded yet.");
                        } else {
                            print!("{content}");
                        }
                        return ExitCode::SUCCESS;
                    }
                    Err(e) => {
                        eprintln!("{PROGRAM}: error reading raw log: {e}");
                        return ExitCode::from(1);
                    }
                }
            }

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
            let mut entries = match forgum_engine::logger::read_recent_logs(lines, min_level) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("{PROGRAM}: error reading logs: {err}");
                    return ExitCode::from(1);
                }
            };

            if let Some(ref q) = filter {
                entries = forgum_engine::logger::filter_entries(&entries, q);
            }

            if diagnose {
                let diag = forgum_engine::logger::diagnose_logs(&entries);
                if json {
                    if let Ok(s) = serde_json::to_string_pretty(&diag) {
                        println!("{s}");
                    }
                } else {
                    print!("{}", forgum_engine::logger::format_diagnostic_report(&diag));
                }
                return ExitCode::SUCCESS;
            }

            if json {
                for entry in &entries {
                    if let Ok(line) = serde_json::to_string(&entry) {
                        println!("{line}");
                    }
                }
            } else {
                print!("{}", forgum_engine::logger::format_log_table(&entries));
            }

            if follow {
                println!("\x1b[90mStreaming live logs (Ctrl+C to stop)...\x1b[0m");
                let mut last_seen = entries.len();
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Ok(mut all_entries) =
                        forgum_engine::logger::read_recent_logs(0, min_level)
                    {
                        if let Some(ref q) = filter {
                            all_entries = forgum_engine::logger::filter_entries(&all_entries, q);
                        }
                        if all_entries.len() > last_seen {
                            for entry in &all_entries[last_seen..] {
                                if json {
                                    if let Ok(line) = serde_json::to_string(&entry) {
                                        println!("{line}");
                                    }
                                } else {
                                    let level_badge = match entry.level.as_str() {
                                        "ERROR" => "\x1b[1;31m[ERROR]\x1b[0m",
                                        "WARN" => "\x1b[1;33m[WARN ]\x1b[0m",
                                        "INFO" => "\x1b[1;32m[INFO ]\x1b[0m",
                                        _ => "\x1b[1;36m[DEBUG]\x1b[0m",
                                    };
                                    println!(
                                        "{} [{}] {}",
                                        level_badge, entry.target, entry.message
                                    );
                                }
                            }
                            last_seen = all_entries.len();
                        }
                    }
                }
            }

            ExitCode::SUCCESS
        }

        // ── diagnose ────────────────────────────────────────────────
        Some(cli::Commands::Diagnose { lines, json }) => {
            let entries = match forgum_engine::logger::read_recent_logs(lines, None) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("{PROGRAM}: error reading logs: {err}");
                    return ExitCode::from(1);
                }
            };
            let diag = forgum_engine::logger::diagnose_logs(&entries);
            if json {
                if let Ok(s) = serde_json::to_string_pretty(&diag) {
                    println!("{s}");
                }
            } else {
                print!("{}", forgum_engine::logger::format_diagnostic_report(&diag));
            }
            ExitCode::SUCCESS
        }

        // ── completions <shell> ──────────────────────────────────────
        Some(cli::Commands::Completions { shell }) => {
            if matches!(shell, cli::ShellArg::List) {
                let table = forgum_engine::options_table::render_options("shells");
                print!("{table}");
                return ExitCode::SUCCESS;
            }
            let mut cmd = forgum_engine::cli::Cli::command();
            let shell: Shell = shell.into();
            match forgum_engine::completions::install_completions(shell, &mut cmd) {
                Ok(res) => {
                    print!("{}", res.message);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: completions installation error: {e}");
                    ExitCode::from(71)
                }
            }
        }

        // ── list / options [category] ────────────────────────────────
        Some(cli::Commands::List { category }) => {
            let table = forgum_engine::options_table::render_options(&category);
            print!("{table}");
            ExitCode::SUCCESS
        }

        // ── status ──────────────────────────────────────────────────
        Some(cli::Commands::Status) | None if args.command == cli::Command::Status => {
            handle_status_command(&args)
        }

        // ── doctor ─────────────────────────────────────────────────
        Some(cli::Commands::Doctor) => {
            let caps = forgum_platform::detect_capabilities();
            let mux = forgum_platform::detect_mux();
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum".to_string());
            let config_info = forgum_platform::detect_config_file(args.config.as_deref())
                .map(|(p, fmt)| format!("{} ({})", p.display(), fmt.display_name()))
                .unwrap_or_else(|e| format!("(error: {e})"));
            let log_dir_info = forgum_platform::log_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            let cows_dir = forgum_platform::data_dir()
                .ok()
                .map(|d| d.join("Cows"))
                .filter(|d| d.is_dir());
            let cow_count = cows_dir
                .as_ref()
                .and_then(|d| std::fs::read_dir(d).ok())
                .map(|rd| rd.filter_map(|e| e.ok()).count())
                .unwrap_or(0);

            let native_plan = forgum_platform::terminal::plan_native_split(
                caps.emulator,
                &mux,
                10,
                0.35,
                &["render".to_string()],
            );
            let native_status = if let Some(plan) = native_plan {
                format!("available ({})", plan.program)
            } else {
                "none".to_string()
            };

            println!(
                "Platform:   {} {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            println!("Engine:     {}", engine_path);
            println!("Config:     {}", config_info);
            println!("Logs:       {}", log_dir_info);
            println!("Terminal:   {}x{}", caps.width, caps.height);
            println!("Emulator:   {} ({})", caps.emulator.name(), caps.emulator.id());
            println!("TTY:        {}", caps.is_tty);
            println!("Color:      {}", caps.color.as_str());
            println!("Sync:       {}", if caps.sync { "yes" } else { "no" });
            println!(
                "DECSTBM:    {}",
                if caps.emulator.supports_decstbm() {
                    "supported"
                } else {
                    "unsupported (fallback to precmd)"
                }
            );
            println!(
                "DECSLRM:    {}",
                if caps.emulator.supports_decslrm() {
                    "supported"
                } else {
                    "unsupported"
                }
            );
            println!("Split Mode: {}", caps.split_mode.as_str());
            println!("Native Split: {}", native_status);
            println!("Graphics:   {:?}", caps.graphics);
            println!("Mux:        {}", mux.name());
            let source = forgum_platform::detect_installation_source();
            let receipt = forgum_platform::read_receipt().unwrap_or(None);
            let channel = receipt
                .as_ref()
                .map(|r| r.channel)
                .unwrap_or(forgum_platform::ReleaseChannel::Stable);
            let shadows = forgum_platform::detect_shadow_installations();

            println!("Source:     {}", source.name());
            println!("Channel:    {}", channel.display_name());
            let inactive_shadows: Vec<_> = shadows.iter().filter(|s| !s.is_active).collect();
            if inactive_shadows.is_empty() {
                println!("Shadows:    none (clean single-source installation)");
            } else {
                println!("Shadows:    ⚠️ {} detected", inactive_shadows.len());
                for s in &shadows {
                    println!("            {}", s.display_line());
                }
            }
            println!("Cows:       {} loaded", cow_count);
            if let Some(limitation) = caps.emulator.limitation_notes() {
                println!("Limitation: \x1b[1;33m{}\x1b[0m", limitation);
            }
            ExitCode::SUCCESS
        }

        // ── checkhealth ────────────────────────────────────────────
        Some(cli::Commands::Checkhealth { json }) => {
            let report = forgum_engine::checkhealth::run_health_check(args.config.as_deref());
            if json {
                match serde_json::to_string_pretty(&report) {
                    Ok(out) => println!("{out}"),
                    Err(e) => {
                        eprintln!("{PROGRAM}: failed to serialize health report: {e}");
                        return ExitCode::from(65);
                    }
                }
            } else {
                print!("{}", report.format_ansi());
            }

            if report.error_count > 0 {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }

        // ── tmux install ───────────────────────────────────────────
        Some(cli::Commands::Tmux {
            sub: cli::TmuxSub::Install,
        }) => {
            let engine_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "forgum".to_string());
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
                .unwrap_or_else(|| "forgum".to_string());
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
                .unwrap_or_else(|| "forgum".to_string());
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
                .unwrap_or_else(|| "forgum".to_string());
            print!(
                "{}",
                forgum_engine::init::generate_screen_config(&engine_path)
            );
            ExitCode::SUCCESS
        }

        // ── tmux list ───────────────────────────────────────────
        Some(cli::Commands::Tmux {
            sub: cli::TmuxSub::List,
        }) => {
            let table = forgum_engine::options_table::render_options("tmux");
            print!("{table}");
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
            if is_list_query(&name) {
                let table = forgum_engine::options_table::render_options("effects");
                print!("{table}");
                return ExitCode::SUCCESS;
            }
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

        // ── herd cow ───────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Cow { name, session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_cow(&name, &filter) {
                Ok(n) => {
                    println!("Set cow on {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd cow: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd eyes ──────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Eyes { eyes, session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_eyes(&eyes, &filter) {
                Ok(n) => {
                    println!("Set eyes on {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd eyes: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd tongue ────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub:
                cli::HerdSub::Tongue {
                    tongue,
                    session,
                    all,
                },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_tongue(&tongue, &filter) {
                Ok(n) => {
                    println!("Set tongue on {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd tongue: {e}");
                    ExitCode::from(1)
                }
            }
        }

        // ── herd color ─────────────────────────────────────────────
        Some(cli::Commands::Herd {
            sub: cli::HerdSub::Color { mode, session, all },
        }) => {
            let filter = forgum_engine::herd::HerdFilter { session, all };
            match forgum_engine::herd::herd_color(&mode, &filter) {
                Ok(n) => {
                    println!("Set color mode on {n} daemon(s).");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{PROGRAM}: herd color: {e}");
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
            let themes = forgum_engine::theme::list_all_themes(&config_dir);
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
            if is_list_query(&name) {
                let config_dir = forgum_platform::config_path()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or_else(|| PathBuf::from("."));
                let themes = forgum_engine::theme::list_all_themes(&config_dir);
                println!("\x1b[1;36m━━━ Available Themes ━━━\x1b[0m");
                if themes.is_empty() {
                    println!("No custom themes found in configuration directory.");
                } else {
                    for t in &themes {
                        println!("  • {t}");
                    }
                }
                println!(
                    "\nTip: run `forgum theme seasonal` to preview and apply the seasonal theme."
                );
                return ExitCode::SUCCESS;
            }
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
            let cmd_output = forgum_engine::say::execute_say_cmd(&cmd);
            if args.image.is_some() {
                let mut scene = match build_scene_config(&args) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{PROGRAM}: {e}");
                        return ExitCode::from(65);
                    }
                };
                scene.text = cmd_output;
                render_subcommand_with_scene(args, scene)
            } else {
                let data_dir = forgum_platform::data_dir().unwrap_or_else(|_| PathBuf::from("."));
                let cow_text = cow::load_cow("default", &data_dir, "oo", "U", "\\\\");
                let output = cow::compose_scene(&cow_text, &cmd_output);
                print!("{output}");
                ExitCode::SUCCESS
            }
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
        Some(cli::Commands::Battle {
            fighter1,
            fighter2,
            name1,
            name2,
            headless,
            winner,
            fps,
        }) => {
            let n1 = fighter1.unwrap_or(name1);
            let n2 = fighter2.unwrap_or(name2);
            if !headless && forgum_platform::is_stdout_tty() {
                forgum_engine::daemon::stop_session_daemon_and_reset_margins();
                forgum_engine::battle::run_battle_live(&n1, &n2, winner, fps);
            } else {
                let output = forgum_engine::battle::run_battle(&n1, &n2);
                print!("{output}");
            }
            ExitCode::SUCCESS
        }

        // ── rps-battle ───────────────────────────────────────────────
        Some(cli::Commands::RpsBattle {
            player,
            cpu,
            choice,
            headless,
            fps,
        }) => {
            let parsed_choice = choice
                .as_deref()
                .and_then(forgum_engine::rps::RpsMove::from_str);
            if !headless && forgum_platform::is_stdout_tty() {
                forgum_engine::daemon::stop_session_daemon_and_reset_margins();
                forgum_engine::rps::run_rps_battle_live(&player, &cpu, parsed_choice, 1, fps);
            } else {
                let output = forgum_engine::rps::run_rps_battle(&player, &cpu, parsed_choice);
                print!("{output}");
            }
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
            cli::RemoteSub::Who | cli::RemoteSub::List => {
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

        // ── stop / kill / halt / reset / unreserve / clear-margins ───
        Some(cli::Commands::Stop { all, force }) => handle_stop_command(all, force),
        None if args.command == cli::Command::Stop => handle_stop_command(false, false),

        // ── sweep ───────────────────────────────────────────────────
        Some(cli::Commands::Sweep) => handle_stop_command(false, false),

        // ── image ───────────────────────────────────────────────────
        Some(cli::Commands::Image {
            path,
            width,
            height,
            color,
            ramp,
            save_cow,
            output,
            invert,
        }) => {
            let color_mode = match color.parse::<forgum_engine::image_ascii::ColorMode>() {
                Ok(cm) => cm,
                Err(e) => {
                    eprintln!("{PROGRAM}: {e}");
                    return ExitCode::from(64);
                }
            };
            let ramp_style = match ramp.parse::<forgum_engine::image_ascii::LuminanceRamp>() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{PROGRAM}: {e}");
                    return ExitCode::from(64);
                }
            };

            if let Some(cow_name) = save_cow {
                match forgum_engine::image_ascii::image_to_cow(&path, &cow_name, width) {
                    Ok(cow_content) => {
                        match forgum_engine::image_ascii::save_custom_cow(&cow_name, &cow_content) {
                            Ok(saved_path) => {
                                println!(
                                    "Saved custom mascot cow '{cow_name}' to {}",
                                    saved_path.display()
                                );
                                if let Some(out_path) = output {
                                    if let Err(e) = std::fs::write(&out_path, &cow_content) {
                                        eprintln!(
                                            "{PROGRAM}: failed to write output file {}: {e}",
                                            out_path.display()
                                        );
                                        return ExitCode::from(1);
                                    }
                                }
                                ExitCode::SUCCESS
                            }
                            Err(e) => {
                                eprintln!("{PROGRAM}: failed to save custom cow: {e}");
                                ExitCode::from(1)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{PROGRAM}: failed to convert image to cow: {e}");
                        ExitCode::from(1)
                    }
                }
            } else {
                let config = forgum_engine::image_ascii::AsciiConverterConfig {
                    target_width: width,
                    target_height: height,
                    color_mode,
                    ramp: ramp_style,
                    invert,
                };
                match forgum_engine::image_ascii::convert_image_path_to_ascii(&path, &config) {
                    Ok(ascii_art) => {
                        if let Some(out_path) = output {
                            if let Err(e) = std::fs::write(&out_path, &ascii_art) {
                                eprintln!(
                                    "{PROGRAM}: failed to write output file {}: {e}",
                                    out_path.display()
                                );
                                return ExitCode::from(1);
                            }
                        } else {
                            println!("{ascii_art}");
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("{PROGRAM}: image conversion error: {e}");
                        ExitCode::from(1)
                    }
                }
            }
        }

        // ── render (default) ────────────────────────────────────────
        _ => render_subcommand(args),
    }
}

struct PaneGuard {
    session_id: String,
    _server: Option<forgum_engine::control_socket::ControlServer>,
}

impl PaneGuard {
    /// Register an instance (foreground/banner/background) so that any second invocation
    /// on the same pane sees this PID and refuses to render. Starts a control socket server
    /// so live commands (like `forgum think` or `forgum stop`) can reach it.
    fn acquire(
        session_id: &str,
        pid: u32,
    ) -> (Self, Option<crossbeam_channel::Receiver<crate::control_socket::ControlCmd>>) {
        let socket_path = forgum_platform::control_socket_path(session_id);
        let (server, cmd_rx) =
            match forgum_engine::control_socket::ControlServer::start(socket_path.clone()) {
                Ok((srv, rx)) => (Some(srv), Some(rx)),
                Err(_) => (None, None),
            };
        let sock_str = if server.is_some() {
            socket_path.to_string_lossy().to_string()
        } else {
            String::new()
        };
        let state_path = forgum_platform::daemon_state_path(session_id);
        let state = daemon::DaemonState {
            pid,
            ob_y1: 0,
            cols: forgum_platform::terminal_size().0,
            socket_path: sock_str,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("{}Z", d.as_secs()))
                .unwrap_or_else(|_| "unknown".into()),
        };
        let _ = state.write(&state_path);
        if let Ok(rt) = forgum_platform::runtime_dir() {
            let _ = state.write(&rt.join("daemon.json"));
        }
        (
            Self {
                session_id: session_id.to_string(),
                _server: server,
            },
            cmd_rx,
        )
    }
}

impl Drop for PaneGuard {
    fn drop(&mut self) {
        daemon::cleanup_daemon_state(&self.session_id);
    }
}

fn handle_status_command(args: &cli::Args) -> ExitCode {
    let version = env!("CARGO_PKG_VERSION");
    let source = forgum_platform::detect_installation_source();
    let detected_cfg = forgum_platform::detect_config_file(args.config.as_deref());
    let (config_path_display, config_fmt_display) = match &detected_cfg {
        Ok((p, fmt)) => (p.display().to_string(), fmt.display_name().to_string()),
        Err(e) => (format!("None ({e})"), "None".to_string()),
    };

    let cfg = match build_scene_config(args) {
        Ok(s) => s,
        Err(_) => forgum_engine::protocol::SceneConfig::default(),
    };

    let session_id = forgum_platform::detect_session_id();
    let state_path = forgum_platform::daemon_state_path(&session_id);
    let daemon_status = if state_path.exists() {
        if let Ok(st) = daemon::DaemonState::read(&state_path) {
            if st.is_alive() {
                format!(
                    "\x1b[1;32mActive\x1b[0m (PID {}, socket: '{}')",
                    st.pid,
                    if st.socket_path.is_empty() { "foreground" } else { &st.socket_path }
                )
            } else {
                "\x1b[1;33mIdle\x1b[0m (previous session terminated)".to_string()
            }
        } else {
            "\x1b[1;33mIdle\x1b[0m".to_string()
        }
    } else {
        "\x1b[1;33mIdle\x1b[0m (0 running on current pane)".to_string()
    };

    let is_tty = crossterm::tty::IsTty::is_tty(&std::io::stdout());
    let (c_bold, c_cyan, c_green, c_yellow, c_reset) = if is_tty {
        ("\x1b[1m", "\x1b[1;36m", "\x1b[1;32m", "\x1b[1;33m", "\x1b[0m")
    } else {
        ("", "", "", "", "")
    };

    println!("{c_cyan}=============================================================================={c_reset}");
    println!("{c_bold}forgum status — Engine v{version}{c_reset}");
    println!("{c_cyan}=============================================================================={c_reset}\n");

    let receipt = forgum_platform::read_receipt().unwrap_or(None);
    let channel = receipt
        .as_ref()
        .map(|r| r.channel)
        .unwrap_or(forgum_platform::ReleaseChannel::Stable);
    let shadows = forgum_platform::detect_shadow_installations();

    // 1. Release & Update Subsystem (Channel & Status)
    println!("{c_cyan}## Updates & Release Subsystem (Channel & Status){c_reset}");
    println!("  • Current Version:  {c_green}v{version}{c_reset} (installed)");
    println!("  • Install Source:   {c_bold}{}{c_reset}", source.name());
    println!("  • Release Channel:  {c_green}{c_bold}{}{c_reset}", channel.display_name());
    let channel_url = match channel {
        forgum_platform::ReleaseChannel::Stable => {
            "GitHub Releases (https://github.com/HKDevLoops/Forgum/releases/latest)"
        }
        forgum_platform::ReleaseChannel::Nightly => {
            "GitHub Nightly (https://github.com/HKDevLoops/Forgum/releases/tag/nightly)"
        }
        forgum_platform::ReleaseChannel::Dev => "Local Development Build",
    };
    println!("  • Update Stream:    {channel_url}");
    println!("  • Update Check:     `forgum update --check` (or `{}`)", source.check_command());
    println!("  • Upgrade Command:  `forgum update` (or `{}`)", source.update_command());

    let inactive_shadows: Vec<_> = shadows.iter().filter(|s| !s.is_active).collect();
    if inactive_shadows.is_empty() {
        println!("  • Shadow Binaries:  {c_green}✓ Clean (0 shadow installations detected){c_reset}\n");
    } else {
        println!("  • Shadow Binaries:  {c_yellow}⚠️ Detected {} shadow installation(s):{c_reset}", inactive_shadows.len());
        for s in &shadows {
            println!("      {}", s.display_line());
        }
        println!();
    }

    // 2. Active User Configurations
    println!("{c_cyan}## Active User Configurations{c_reset}");
    println!("  • Config File:      {c_bold}{config_path_display}{c_reset} ({config_fmt_display})");
    println!("  • Mascot (cow):     {c_yellow}'{}'{c_reset}", cfg.cow);
    println!("  • Animation Effect: {c_yellow}'{}'{c_reset}", cfg.effect);
    println!("  • Target FPS:       {c_bold}{}{c_reset}", cfg.fps);
    println!("  • Color Mode:       {c_yellow}'{}'{c_reset}{}", cfg.color_mode, if let Some(ref pal) = cfg.palette { format!(" (palette: {pal})") } else { String::new() });
    println!("  • Shell Attach:     {c_yellow}'{}'{c_reset}", cfg.shell_attach_mode);
    println!("  • Auto on Prompt:   {c_bold}{}{c_reset}", cfg.auto_render_on_prompt);
    println!("  • Duration:         {c_bold}{}s{c_reset} (0 = permanent / persistent)", cfg.duration);
    println!("  • Space Split:      split_scroll={}, ratio={}, reserve_rows={}",
        cfg.split_scroll,
        cfg.split_ratio.map(|r| format!("{:.2}", r)).unwrap_or_else(|| "auto".into()),
        cfg.reserve_rows.map(|r| r.to_string()).unwrap_or_else(|| "auto".into())
    );
    if let Some(ref env) = cfg.environment {
        println!("  • Environment:      {c_yellow}'{env}'{c_reset}");
    }
    if let Some(ref rd) = cfg.road {
        println!("  • Road Texture:     {c_yellow}'{rd}'{c_reset}");
    }
    if let Some(ref mtn) = cfg.mountain {
        println!("  • Mountain Range:   {c_yellow}'{mtn}'{c_reset}");
    }
    let random_display = match &cfg.random {
        Some(forgum_platform::protocol::RandomSetting::Bool(true)) => {
            format!("{c_green}all (universally active on startup){c_reset}")
        }
        Some(forgum_platform::protocol::RandomSetting::Bool(false)) => {
            format!("{c_yellow}disabled{c_reset}")
        }
        Some(forgum_platform::protocol::RandomSetting::String(s)) => {
            format!("{c_green}'{s}'{c_reset}")
        }
        Some(forgum_platform::protocol::RandomSetting::List(l)) => {
            format!("{c_green}'{}'{c_reset}", l.join(", "))
        }
        None => format!("{c_yellow}disabled (default){c_reset}"),
    };
    println!("  • Random Mode:      {random_display}");
    println!("  • Daemon Session:   {daemon_status}\n");

    // 3. What's New in this Release
    println!("{c_cyan}## What's New in this Release (v{version}){c_reset}");
    println!("  {c_green}• 🎲 First-Class RANDOM Property:{c_reset} Universally randomizes mascots, scenery biomes, effects, static/dynamic animations, and thoughts on startup.");
    println!("  {c_green}• 🛡️  Single-Instance Pane Guard:{c_reset} Eliminates overlapping animation renders on the same pane; forwards thoughts and commands live via IPC.");
    println!("  {c_green}• ⚡ Universal IPC Control Socket:{c_reset} Foreground, banner, and background modes listen on control sockets for live thought hotswapping.");
    println!("  {c_green}• 🌿 True Kinematic Bobbing:{c_reset} Physical sinusoidal floating/flying kinematics for aerial creatures with anchored speech bubbles.");
    println!("  {c_green}• 🎨 Native TrueColor Detection:{c_reset} Automatic 16.7M 24-bit RGB TrueColor palette activation on Windows Terminal, WezTerm, and modern emulators.");
    println!("  {c_green}• 📐 Clean Space & Margin Reset:{c_reset} DECSTBM scroll margins and reserved rows are restored cleanly without erasing terminal history.");
    println!("  {c_green}• 🚀 <100MB RAM Nature Math:{c_reset} Zero-allocation procedural mountain, tree, and road rendering running strictly on pre-allocated framebuffers.");
    println!("{c_cyan}=============================================================================={c_reset}");

    ExitCode::SUCCESS
}

fn handle_channel_command(action: Option<&str>, name: Option<&str>) -> ExitCode {
    let receipt = forgum_platform::read_receipt().unwrap_or(None);
    let current_channel = receipt
        .as_ref()
        .map(|r| r.channel)
        .unwrap_or(forgum_platform::ReleaseChannel::Stable);
    let source = forgum_platform::detect_installation_source();

    let target_str = match action {
        None | Some("list") | Some("status") => {
            println!("\x1b[1;36m━━━ Forgum Release Channels ━━━\x1b[0m");
            println!("Active Channel:   \x1b[1;32m{}\x1b[0m", current_channel.display_name());
            println!("Install Source:   \x1b[1m{}\x1b[0m", source.name());
            println!("Active Version:   v{}", env!("CARGO_PKG_VERSION"));
            if let Some(r) = receipt {
                println!(
                    "Receipt Location: {}",
                    forgum_platform::Receipt::file_path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| "unknown".to_string())
                );
                println!("Receipt Binary:   {}", r.bin_path.display());
            }
            println!("\nAvailable Channels:");
            for ch in forgum_platform::ReleaseChannel::ALL {
                let marker = if *ch == current_channel {
                    " \x1b[1;32m●\x1b[0m"
                } else {
                    "  "
                };
                println!("{marker} \x1b[1m{:<8}\x1b[0m — {}", ch.as_str(), ch.description());
            }
            println!("\nUsage:");
            println!("  forgum channel switch <stable|nightly|dev>");
            println!("  forgum update --channel <stable|nightly|dev>");

            let shadows = forgum_platform::detect_shadow_installations();
            let inactive: Vec<_> = shadows.iter().filter(|s| !s.is_active).collect();
            if !inactive.is_empty() {
                println!("\n\x1b[1;33m⚠️  Shadow Installations Detected:\x1b[0m");
                for s in &shadows {
                    println!("  {}", s.display_line());
                }
            }

            return ExitCode::SUCCESS;
        }
        Some("get") => {
            println!("{}", current_channel.as_str());
            return ExitCode::SUCCESS;
        }
        Some("set") | Some("switch") => name,
        other => other,
    };

    let Some(target) = target_str else {
        eprintln!("usage: forgum channel switch <stable|nightly|dev>");
        return ExitCode::from(1);
    };

    let target_channel = match target.parse::<forgum_platform::ReleaseChannel>() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(1);
        }
    };

    match forgum_platform::record_receipt(target_channel, Some(source)) {
        Ok(_) => {
            println!(
                "\x1b[1;32m✓\x1b[0m Successfully switched release channel to: \x1b[1m{}\x1b[0m",
                target_channel.display_name()
            );
            println!(
                "  Updated receipt at: {}",
                forgum_platform::Receipt::file_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            );
            println!("  Future `forgum update` calls will synchronize against this stream.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{PROGRAM}: failed to save receipt: {e}");
            ExitCode::from(1)
        }
    }
}

fn handle_update_command(check: bool, channel_flag: Option<&str>) -> ExitCode {
    if let Some(ch_str) = channel_flag {
        match ch_str.parse::<forgum_platform::ReleaseChannel>() {
            Ok(c) => {
                let _ = forgum_platform::record_receipt(c, None);
                println!("Active channel switched to: \x1b[1;32m{}\x1b[0m", c.display_name());
            }
            Err(e) => {
                eprintln!("{PROGRAM}: {e}");
                return ExitCode::from(1);
            }
        }
    }

    let receipt = forgum_platform::read_receipt().unwrap_or(None);
    let channel = receipt
        .as_ref()
        .map(|r| r.channel)
        .unwrap_or(forgum_platform::ReleaseChannel::Stable);
    let source = forgum_platform::detect_installation_source();

    let shadows = forgum_platform::detect_shadow_installations();
    let conflicts: Vec<_> = shadows.iter().filter(|s| !s.is_active).collect();
    if !conflicts.is_empty() {
        println!(
            "\x1b[1;33m⚠️  Warning: Multiple Forgum installations detected across package managers:\x1b[0m"
        );
        for s in &shadows {
            println!("    {}", s.display_line());
        }
        println!("  \x1b[90m(Suggestion: Remove duplicate versions to avoid PATH ambiguity)\x1b[0m\n");
    }

    let action_str = if check {
        "Checking for available updates"
    } else {
        "Updating Forgum"
    };

    println!("\x1b[1;36m━━━ Forgum Auto-Update ━━━\x1b[0m");
    println!("Installed via:   \x1b[1;32m{}\x1b[0m", source.name());
    println!("Release Channel: \x1b[1;33m{}\x1b[0m", channel.display_name());
    println!(
        "Action:          \x1b[1;33m{}\x1b[0m (Command: `{}`)\n",
        action_str,
        if check {
            source.check_command()
        } else {
            source.update_command()
        }
    );

    match source {
        forgum_platform::PackageManager::DirectBinary => {
            let version = env!("CARGO_PKG_VERSION");
            let channel_stream = match channel {
                forgum_platform::ReleaseChannel::Stable => {
                    "https://github.com/HKDevLoops/Forgum/releases/latest"
                }
                forgum_platform::ReleaseChannel::Nightly => {
                    "https://github.com/HKDevLoops/Forgum/releases/tag/nightly"
                }
                forgum_platform::ReleaseChannel::Dev => {
                    "Local git repository (compile with `cargo build --release`)"
                }
            };
            println!(
                "Forgum v{version} is running as a Standalone Binary.\n\
                 Target stream for channel {channel}:\n\
                 {channel_stream}\n\n\
                 To install or update via an official package manager:\n\
                   Scoop:  scoop update forgum\n\
                   WinGet: winget upgrade HKDevLoops.Forgum\n\
                   Brew:   brew upgrade forgum"
            );
            ExitCode::SUCCESS
        }
        _ => {
            match forgum_platform::execute_package_manager_action(source, check) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{output}");
                    }
                    println!("\n\x1b[1;32m✓ Action completed.\x1b[0m");
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("\x1b[1;31m✗ Update action failed:\x1b[0m {err}");
                    ExitCode::from(1)
                }
            }
        }
    }
}

fn handle_stop_command(all: bool, force: bool) -> ExitCode {
    let mut stopped_count = 0;
    let mut known_pids = std::collections::HashSet::new();

    // 1. Session and runtime daemon state discovery
    let session_id = forgum_platform::detect_session_id();
    let session_path = forgum_platform::daemon_state_path(&session_id);
    let mut max_ob_y1: u16 = 0;

    let mut state_paths = Vec::new();
    if session_path.exists() {
        state_paths.push(session_path.clone());
    }

    if let Ok(rt) = forgum_platform::runtime_dir() {
        let generic = rt.join("daemon.json");
        if generic.exists() && !state_paths.contains(&generic) {
            state_paths.push(generic);
        }

        if all || state_paths.is_empty() {
            if let Ok(entries) = std::fs::read_dir(&rt) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();
                    if name_str.starts_with("daemon-") && name_str.ends_with(".json") && !state_paths.contains(&path) {
                        state_paths.push(path);
                    }
                }
            }
        }
    }

    for path in &state_paths {
        if let Ok(state) = daemon::DaemonState::read(path) {
            if state.ob_y1 > max_ob_y1 {
                max_ob_y1 = state.ob_y1;
            }
            known_pids.insert(state.pid);
            if state.is_alive() {
                if !force {
                    let _ = crate::herd::send_command(&state.socket_path, r#"{"cmd":"STOP"}"#);
                    std::thread::sleep(std::time::Duration::from_millis(60));
                }
                if force || state.is_alive() {
                    if forgum_platform::kill_process(state.pid) {
                        stopped_count += 1;
                    }
                } else {
                    stopped_count += 1;
                }
            }
        }
        let _ = std::fs::remove_file(path);
    }

    // Clean daemon state for current session
    daemon::cleanup_daemon_state(&session_id);

    // 2. Scan OS process table for any unlisted / orphaned forgum processes
    let unlisted = forgum_platform::find_forgum_pids();
    for pid in unlisted {
        if all || !known_pids.contains(&pid) {
            if forgum_platform::kill_process(pid) {
                stopped_count += 1;
            }
        }
    }

    // Clean up any remaining socket or state files in runtime directory
    if let Ok(rt) = forgum_platform::runtime_dir() {
        if let Ok(entries) = std::fs::read_dir(&rt) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("daemon") || name_str.ends_with(".sock") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    // 3. Complete space reservation and terminal margins reset
    let (_, height) = forgum_platform::terminal_size();
    let mut clean_buf = Vec::new();
    if max_ob_y1 > 0 {
        let clear_rows = (max_ob_y1 as usize).min(height as usize);
        clean_buf.extend_from_slice(b"\x1b7");
        for y in 1..=clear_rows {
            clean_buf.extend_from_slice(format!("\x1b[{y};1H\x1b[2K").as_bytes());
        }
        clean_buf.extend_from_slice(b"\x1b[r\x1b[?6l\x1b[?69l\x1b8\x1b[0m\x1b[?25h");
    } else if stopped_count > 0 {
        let clear_rows = (height as usize).min(16);
        clean_buf.extend_from_slice(b"\x1b7");
        for y in 1..=clear_rows {
            clean_buf.extend_from_slice(format!("\x1b[{y};1H\x1b[2K").as_bytes());
        }
        clean_buf.extend_from_slice(b"\x1b[r\x1b[?6l\x1b[?69l\x1b8\x1b[0m\x1b[?25h");
    } else {
        // Reset margins cleanly without destroying existing terminal scrollback lines
        clean_buf.extend_from_slice(b"\x1b[r\x1b[?6l\x1b[?69l\x1b8\x1b[0m\x1b[?25h");
    }
    let _ = std::io::Write::write_all(&mut std::io::stdout(), &clean_buf);
    let _ = std::io::Write::flush(&mut std::io::stdout());

    // Disable raw mode if left active
    let _ = crossterm::terminal::disable_raw_mode();

    if stopped_count > 0 {
        println!("forgum: stopped {stopped_count} running animation(s), terminal margins and reserved space reset.");
    } else {
        println!("forgum: pasture cleared; terminal margins and reserved space reset.");
    }

    ExitCode::SUCCESS
}

fn handle_think_command(mut args: cli::Args, thought: Vec<String>) -> ExitCode {
    let data = match data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{PROGRAM}: cannot find data directory: {e}");
            return ExitCode::from(78);
        }
    };

    let thought_text = if !thought.is_empty() {
        let joined = thought.join(" ");
        if joined.trim().eq_ignore_ascii_case("random") {
            fortune::random_fortune(&data)
                .unwrap_or_else(|| "The cow that never moos has the most to say.".to_string())
        } else {
            joined
        }
    } else if let Some(ref t) = args.text {
        if t.trim().eq_ignore_ascii_case("random") {
            fortune::random_fortune(&data)
                .unwrap_or_else(|| "The cow that never moos has the most to say.".to_string())
        } else {
            t.clone()
        }
    } else if !crossterm::tty::IsTty::is_tty(&std::io::stdin()) {
        use std::io::Read;
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        let trimmed = buf.trim().to_string();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("random") {
            trimmed
        } else {
            fortune::random_fortune(&data)
                .unwrap_or_else(|| "The cow that never moos has the most to say.".to_string())
        }
    } else {
        fortune::random_fortune(&data)
            .unwrap_or_else(|| "The cow that never moos has the most to say.".to_string())
    };

    // 1. Check if an active session daemon is running. If so, forward via IPC to update in-place without duplicate process.
    let session_id = forgum_platform::detect_session_id();
    let state_path = forgum_platform::daemon_state_path(&session_id);
    let mut active_socket: Option<String> = None;

    // Track whether we found a live foreground instance (no IPC socket) vs a live daemon (with socket).
    let mut foreground_alive = false;

    if state_path.exists() {
        if let Ok(state) = daemon::DaemonState::read(&state_path) {
            if state.is_alive() {
                if state.socket_path.is_empty() {
                    // Foreground instance: has no control socket — cannot IPC-forward.
                    foreground_alive = true;
                } else {
                    active_socket = Some(state.socket_path);
                }
            }
        }
    }
    if active_socket.is_none() && !foreground_alive {
        if let Ok(rt) = forgum_platform::runtime_dir() {
            let generic = rt.join("daemon.json");
            if generic.exists() {
                if let Ok(state) = daemon::DaemonState::read(&generic) {
                    if state.is_alive() {
                        if state.socket_path.is_empty() {
                            foreground_alive = true;
                        } else {
                            active_socket = Some(state.socket_path);
                        }
                    }
                }
            }
        }
    }

    if !args.daemon {
        if let Some(socket_path) = active_socket {
            // Live daemon with IPC socket: forward all flags then the thought text.
            if let Some(ref cow) = args.cow {
                let payload = serde_json::json!({"cmd": "COW", "arg": cow}).to_string();
                let _ = crate::herd::send_command(&socket_path, &payload);
            }
            if let Some(ref color) = args.color_mode {
                let payload = serde_json::json!({"cmd": "COLOR", "arg": color}).to_string();
                let _ = crate::herd::send_command(&socket_path, &payload);
            }
            let payload = serde_json::json!({"cmd": "THINK", "arg": thought_text}).to_string();
            if crate::herd::send_command(&socket_path, &payload).is_ok() {
                return ExitCode::SUCCESS;
            }
        }
        if foreground_alive {
            // An instance is already running on this pane: another instance on the same pane must not render or overlap.
            return ExitCode::SUCCESS;
        }
    }

    // 2. Clean up any dead/stale session daemon to prevent visual collisions.
    // (stop_session_daemon_and_reset_margins is safe and no-ops if no daemon was running)
    daemon::stop_session_daemon_and_reset_margins();

    // 3. If --daemon was explicitly requested, stop existing daemon and spawn a new one cleanly.
    if args.daemon {
        args.think = true;
        args.text = Some(thought_text);
        return spawn_daemon_parent(&args);
    }

    // 4. If explicit animation flags were passed, run bounded foreground animation.
    let has_explicit_animation = args.animation.is_some()
        || args.effect.is_some()
        || args.duration.is_some()
        || args.banner;

    if has_explicit_animation {
        let mut scene = match build_scene_config(&args) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{PROGRAM}: {e}");
                return ExitCode::from(65);
            }
        };
        scene.think = true;
        scene.text = thought_text;
        scene.background = false;
        scene.split_scroll = false;
        if scene.duration == 0 {
            scene.duration = 2; // Bounded burst animation, never hang forever
        }
        return render_subcommand_with_scene(args, scene);
    }

    // 5. Standalone cowthink mode: print static composed thought cow cleanly to stdout.
    let mut scene = match build_scene_config(&args) {
        Ok(s) => s,
        Err(_) => forgum_engine::protocol::SceneConfig::default(),
    };
    scene.think = true;
    scene.text = thought_text;
    resolve_scene_randomness(&mut scene, &data, &args);

    let thoughts_glyph = "o";
    let cow_text = cow::load_cow(
        &scene.cow,
        &data,
        &scene.eyes,
        &scene.tongue,
        thoughts_glyph,
    );
    let composed = cow::compose_scene_with_mode(&cow_text, &scene.text, true);

    print_colored_scene(&composed, &scene.cow, &scene.color_mode, scene.palette.as_deref());
    ExitCode::SUCCESS
}

fn print_colored_scene(
    composed: &str,
    animal_name: &str,
    color_mode: &str,
    custom_palette: Option<&str>,
) {
    if !crossterm::tty::IsTty::is_tty(&std::io::stdout()) || color_mode == "none" {
        print!("{composed}");
        return;
    }

    let palette_tuples: Vec<(u8, u8, u8)> = if let Some(pal) = custom_palette {
        let hexes: Vec<String> = pal
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        crate::color::parse_palette(&hexes)
    } else {
        crate::color::get_natural_palette(animal_name).to_vec()
    };

    let cow_start_line = forgum_engine::effects::find_cow_start_line(composed);
    let mut out = std::io::stdout().lock();
    use std::io::Write;

    for (y, line) in composed.lines().enumerate() {
        let is_bubble_line = y < cow_start_line;
        let animal_rel_y = y.saturating_sub(cow_start_line);

        let mut current_fg: Option<crate::framebuffer::Color> = None;

        for (x, ch) in line.chars().enumerate() {
            let cell_fg = if is_bubble_line {
                crate::effects::resolve_bubble_line_char_fg(line, x, ch)
            } else {
                crate::effects::resolve_fg_palette_char(
                    color_mode,
                    &palette_tuples,
                    x,
                    animal_rel_y,
                    0.0,
                    crate::framebuffer::Color::WHITE,
                    ch,
                )
            };

            if current_fg != Some(cell_fg) {
                current_fg = Some(cell_fg);
                let _ = write!(out, "\x1b[38;2;{};{};{}m", cell_fg.r, cell_fg.g, cell_fg.b);
            }
            let _ = write!(out, "{ch}");
        }
        let _ = writeln!(out, "\x1b[0m");
    }
    let _ = out.flush();
}

fn render_subcommand(args: cli::Args) -> ExitCode {
    // Read scene from --file or stdin (capped to 4 MB per BUG-D4 / BUG-D5).
    let scene_input = match forgum_engine::protocol_io::read_scene(args.file.as_deref(), true) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(e.exit_code() as u8);
        }
    };

    // Build merged scene (config auto-discovered if --config not given).
    // Note: --file is handled by build_scene_config via config merging.
    let mut scene = match build_scene_config(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(65);
        }
    };
    scene = forgum_engine::config::merge(scene_input, scene);
    render_subcommand_with_scene(args, scene)
}

/// Universally resolve all `RANDOM` parameters: mascot, scenery (environment/road/mountain),
/// fx and animation (including static vs dynamic animations), and fortune thoughts.
///
/// Uses Fibonacci phyllotaxis low-discrepancy mathematics (matching procedural tree spawning in
/// `scenery.rs`) to mathematically guarantee zero repetition and maximum dispersion across consecutive runs.
pub fn resolve_scene_randomness(
    scene: &mut forgum_engine::protocol::SceneConfig,
    data_dir: &std::path::Path,
    explicit_args: &cli::Args,
) {
    let random_setting = scene.random.clone();
    let is_random_enabled = scene.cow.trim().eq_ignore_ascii_case("random")
        || scene.environment.as_deref().map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || scene.road.as_deref().map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || scene.mountain.as_deref().map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || scene.animation.as_deref().map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || scene.effect.trim().eq_ignore_ascii_case("random")
        || scene.animation_type.as_deref().map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || scene.text.trim().is_empty()
        || scene.text.trim().eq_ignore_ascii_case("random")
        || random_setting.is_some();

    if !is_random_enabled {
        scene.cow = cow::resolve_cow_name(&scene.cow, data_dir);
        return;
    }

    // Sample Fibonacci phyllotaxis low-discrepancy coordinates
    let coords = crate::random_engine::advance_and_sample();

    // ── 1. MASCOT (COW) ──────────────────────────────────────────────────
    let should_randomize_cow = scene.cow.trim().eq_ignore_ascii_case("random")
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_mascot())
            && explicit_args.cow.is_none());

    if should_randomize_cow {
        let mut cow_names: Vec<String> = Vec::new();
        let cows_dir = data_dir.join("Cows");
        if let Ok(rd) = std::fs::read_dir(&cows_dir) {
            cow_names.extend(rd.flatten().filter_map(|e| {
                let p = e.path();
                if p.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("cow")) {
                    p.file_stem().map(|s| s.to_string_lossy().into_owned())
                } else {
                    None
                }
            }));
        }
        if let Ok(cfg_path) = forgum_platform::config_path() {
            if let Some(custom_dir) = cfg_path.parent().map(|p| p.join("cows")) {
                if let Ok(rd) = std::fs::read_dir(&custom_dir) {
                    cow_names.extend(rd.flatten().filter_map(|e| {
                        let p = e.path();
                        if p.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("cow")) {
                            p.file_stem().map(|s| s.to_string_lossy().into_owned())
                        } else {
                            None
                        }
                    }));
                }
            }
        }
        cow_names.sort();
        cow_names.dedup();
        if let Some(picked) = coords.pick_from_slice(&cow_names, coords.u_mascot) {
            scene.cow = picked.clone();
        } else {
            scene.cow = cow::resolve_cow_name("random", data_dir);
        }

        // Auto-adapt animal profile defaults if user didn't explicitly override them
        let profile = crate::scenery::get_animal_profile(&scene.cow);
        if explicit_args.eyes.is_none() && (scene.eyes.is_empty() || scene.eyes == "oo") {
            scene.eyes = profile.eyes.to_string();
        }
        if explicit_args.tongue.is_none() && (scene.tongue.is_empty() || scene.tongue == "  ") {
            scene.tongue = profile.tongue.to_string();
        }
        if explicit_args.palette.is_none() && scene.palette.is_none() {
            let natural_hexes = crate::color::get_natural_hex_palette(&scene.cow);
            if !natural_hexes.is_empty() {
                scene.palette = Some(natural_hexes.join(","));
            }
        }
        // If scenery was NOT explicitly pinned and NOT separately randomized, default to animal's native biome
        if explicit_args.environment.is_none()
            && scene.environment.as_deref().map_or(true, |e| e == "default" || e.is_empty())
            && !random_setting.as_ref().map_or(false, |r| r.randomizes_scenery())
        {
            scene.environment = Some(profile.environment.as_str().to_string());
            scene.road = Some(profile.road.as_str().to_string());
            scene.mountain = Some(profile.mountain.as_str().to_string());
        }
    } else {
        scene.cow = cow::resolve_cow_name(&scene.cow, data_dir);
    }

    // ── 2. SCENERY (ENVIRONMENT / ROAD / MOUNTAIN) ──────────────────────
    let should_randomize_env = scene
        .environment
        .as_deref()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_scenery())
            && explicit_args.environment.is_none());

    let should_randomize_road = scene
        .road
        .as_deref()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_scenery())
            && explicit_args.road.is_none());

    let should_randomize_mountain = scene
        .mountain
        .as_deref()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_scenery())
            && explicit_args.mountain.is_none());

    if should_randomize_env {
        let env_slice = crate::scenery::EnvironmentStyle::ALL_NON_NONE;
        let picked_env = *coords
            .pick_from_slice(env_slice, coords.u_env)
            .unwrap_or(&crate::scenery::EnvironmentStyle::Pasture);
        scene.environment = Some(picked_env.as_str().to_string());

        // Harmonize road and mountain to the random biome if not explicitly locked or separately randomized
        if !should_randomize_road && explicit_args.road.is_none() && scene.road.is_none() {
            let (dyn_mtn, dyn_road) = crate::scenery::environment_scenery_defaults(picked_env);
            scene.road = Some(dyn_road.as_str().to_string());
            if !should_randomize_mountain && explicit_args.mountain.is_none() && scene.mountain.is_none() {
                scene.mountain = Some(dyn_mtn.as_str().to_string());
            }
        }
    }

    if should_randomize_road {
        let road_slice = crate::scenery::RoadStyle::ALL_NON_NONE;
        let picked_road = *coords
            .pick_from_slice(road_slice, coords.u_road)
            .unwrap_or(&crate::scenery::RoadStyle::Dirt);
        scene.road = Some(picked_road.as_str().to_string());
    }

    if should_randomize_mountain {
        let mtn_slice = crate::scenery::MountainStyle::ALL_NON_NONE;
        let picked_mtn = *coords
            .pick_from_slice(mtn_slice, coords.u_mountain)
            .unwrap_or(&crate::scenery::MountainStyle::Hills);
        scene.mountain = Some(picked_mtn.as_str().to_string());
    }

    // ── 3. FX AND ANIMATIONS (STATIC & DYNAMIC) ─────────────────────────
    let should_randomize_anim = scene
        .animation
        .as_deref()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_fx())
            && explicit_args.animation.is_none());

    let should_randomize_fx = scene.effect.trim().eq_ignore_ascii_case("random")
        || scene
            .animation_type
            .as_deref()
            .map_or(false, |s| s.trim().eq_ignore_ascii_case("random"))
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_fx())
            && explicit_args.effect.is_none()
            && explicit_args.animation_type.is_none());

    if should_randomize_anim {
        // Deterministic Fibonacci phyllotaxis static vs dynamic selection
        let is_static = coords.is_static_animation();
        if is_static {
            scene.animation = Some("static".to_string());
            if should_randomize_fx || scene.effect == "default" {
                scene.effect = "static".to_string();
                scene.animation_type = Some("static".to_string());
            }
        } else {
            scene.animation = Some("dynamic".to_string());
            if should_randomize_fx || scene.effect == "default" || scene.effect == "static" {
                let dyn_list = crate::effects::DYNAMIC_ANIMATION_TYPES;
                let picked_dyn = *coords
                    .pick_from_slice(dyn_list, coords.u_effect)
                    .unwrap_or(&"walk");
                scene.effect = picked_dyn.to_string();
                scene.animation_type = Some(picked_dyn.to_string());
            }
        }
    } else if should_randomize_fx {
        let effect_names = crate::effects::ALL_EFFECTS;
        let picked_effect = *coords
            .pick_from_slice(effect_names, coords.u_effect)
            .unwrap_or(&"breathe");
        scene.effect = picked_effect.to_string();
        scene.animation_type = Some(picked_effect.to_string());
        if picked_effect == "static" {
            scene.animation = Some("static".to_string());
        } else {
            scene.animation = Some("dynamic".to_string());
        }
    }

    // ── 4. THOUGHT / TEXT ────────────────────────────────────────────────
    let should_randomize_thought = scene.text.trim().is_empty()
        || scene.text.trim().eq_ignore_ascii_case("random")
        || (random_setting.as_ref().map_or(false, |r| r.randomizes_thought())
            && explicit_args.text.is_none());

    if should_randomize_thought {
        let fortunes = fortune::load_fortunes(data_dir);
        if !fortunes.is_empty() {
            if let Some(picked) = coords.pick_from_slice(&fortunes, coords.u_thought) {
                scene.text = picked.clone();
            }
        } else if let Some(f) = fortune::random_fortune(data_dir) {
            scene.text = f;
        }
    }
}

fn render_subcommand_with_scene(
    args: cli::Args,
    mut scene: forgum_engine::protocol::SceneConfig,
) -> ExitCode {
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

    if args.split_mode.as_deref() == Some("native")
        || scene.split_mode.as_deref() == Some("native")
    {
        let caps = forgum_platform::detect_capabilities();
        let mux = forgum_platform::detect_mux();
        let reserved_rows = args
            .reserve_rows
            .or(scene.reserve_rows)
            .unwrap_or(10) as usize;
        let ratio = args
            .split_ratio
            .or(scene.split_ratio)
            .unwrap_or(0.35);
        let mut forward_args: Vec<String> = std::env::args()
            .skip(1)
            .filter(|a| a != "--split-mode" && a != "native")
            .collect();
        forward_args.insert(0, "render".to_string());
        if let Some(plan) = forgum_platform::plan_native_split(
            caps.emulator,
            &mux,
            reserved_rows,
            ratio,
            &forward_args,
        ) {
            match std::process::Command::new(&plan.program)
                .args(&plan.args)
                .spawn()
            {
                Ok(_) => {
                    println!(
                        "Spawned native {} split pane ({})",
                        plan.target, plan.explanation
                    );
                    return ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!(
                        "{PROGRAM}: failed to spawn native split command '{}': {e}. Falling back to standard DECSTBM split.",
                        plan.program
                    );
                }
            }
        } else {
            eprintln!(
                "{PROGRAM}: native split API not supported on {} (mux: {}). Falling back to standard DECSTBM split.",
                caps.emulator.name(),
                mux.name()
            );
        }
    }

    let is_split_mode = args.split_scroll
        || scene.split_scroll
        || args.reserve_rows.is_some()
        || scene.reserve_rows.is_some()
        || args.split_ratio.is_some()
        || scene.split_ratio.is_some()
        || scene.shell_attach_mode == "split";

    // ── SINGLE-INSTANCE PANE GUARD ───────────────────────────────────────
    // If an instance of forgum is already running on this pane/session,
    // another instance on the same pane must NOT render. Instead:
    // 1. If args.daemon or is_split_mode: report existing PID and return 0.
    // 2. If new text / mascot / effect / color args were supplied: forward to
    //    the active instance via IPC control socket and exit 0 without competing output.
    // 3. If no specific text/args: update active mascot with a fresh fortune via IPC.
    // 4. If the active instance is an uncommunicative foreground instance: exit 0.
    let session_id = forgum_platform::detect_session_id();
    let state_path = forgum_platform::daemon_state_path(&session_id);
    let mut active_state: Option<daemon::DaemonState> = None;

    if state_path.exists() {
        if let Ok(st) = daemon::DaemonState::read(&state_path) {
            if st.is_alive() && st.pid != std::process::id() {
                active_state = Some(st);
            }
        }
    }
    if active_state.is_none() {
        if let Ok(rt) = forgum_platform::runtime_dir() {
            let generic = rt.join("daemon.json");
            if generic.exists() {
                if let Ok(st) = daemon::DaemonState::read(&generic) {
                    if st.is_alive() && st.pid != std::process::id() {
                        active_state = Some(st);
                    }
                }
            }
        }
    }

    if let Some(st) = active_state {
        // If split/daemon was requested and one already exists: report PID and return.
        if args.daemon || is_split_mode {
            println!("{}", st.pid);
            return ExitCode::SUCCESS;
        }

        // Forward updates to the running instance via control socket if responsive.
        let mut forwarded = false;
        if !st.socket_path.is_empty() {
            if let Some(ref cow) = args.cow {
                let payload = serde_json::json!({"cmd": "COW", "arg": cow}).to_string();
                let _ = crate::herd::send_command(&st.socket_path, &payload);
                forwarded = true;
            }
            if let Some(ref effect) = args.effect {
                let payload = serde_json::json!({"cmd": "EFFECT", "arg": effect}).to_string();
                let _ = crate::herd::send_command(&st.socket_path, &payload);
                forwarded = true;
            }
            if let Some(ref color) = args.color_mode {
                let payload = serde_json::json!({"cmd": "COLOR", "arg": color}).to_string();
                let _ = crate::herd::send_command(&st.socket_path, &payload);
                forwarded = true;
            }

            let text_to_send = if let Some(ref t) = args.text {
                Some(t.clone())
            } else if !scene.text.trim().is_empty() {
                Some(scene.text.clone())
            } else {
                None
            };

            if let Some(ref msg) = text_to_send {
                let cmd_name = if scene.think || args.think || args.command == cli::Command::Think {
                    "THINK"
                } else {
                    "TEXT"
                };
                let payload = serde_json::json!({"cmd": cmd_name, "arg": msg}).to_string();
                if crate::herd::send_command(&st.socket_path, &payload).is_ok() {
                    forwarded = true;
                }
            } else if !forwarded {
                if let Ok(data) = data_dir() {
                    if let Some(f) = fortune::random_fortune(&data) {
                        let payload = serde_json::json!({"cmd": "TEXT", "arg": f}).to_string();
                        let _ = crate::herd::send_command(&st.socket_path, &payload);
                        forwarded = true;
                    }
                }
            }
        }

        if forwarded || st.is_alive() {
            return ExitCode::SUCCESS;
        }

        // Stale state: clean up and proceed
        daemon::cleanup_daemon_state(&session_id);
    }

    if args.daemon || is_split_mode {
        // ── DAEMON / SPLIT MODE — PARENT PATH ────────────────────────
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

    let data = match data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{PROGRAM}: cannot find data directory: {e}");
            return ExitCode::from(78);
        }
    };

    resolve_scene_randomness(&mut scene, &data, &args);

    let is_thought = scene.think
        || args.command == cli::Command::Think
        || (args.text.is_none() && scene.text.trim().is_empty());

    if scene.text.trim().is_empty() {
        scene.text = fortune::random_fortune(&data)
            .unwrap_or_else(|| "The cow that never moos has the most to say.".to_string());
    }

    if args.text_only {
        println!("{}", scene.text);
        return ExitCode::SUCCESS;
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

    let thoughts_glyph = if is_thought { "o" } else { "\\" };
    let cow_text = if let Some(img_path) = &scene.image {
        let term_w = crossterm::terminal::size()
            .map(|s| s.0 as usize)
            .unwrap_or(80);
        let max_w = (term_w * 45 / 100)
            .clamp(24, 55)
            .min(term_w.saturating_sub(6));
        match forgum_engine::image_ascii::image_to_cow(img_path, "image_mascot", Some(max_w)) {
            Ok(raw_cow) => cow::expand_cow(&raw_cow, &scene.eyes, &scene.tongue, thoughts_glyph),
            Err(e) => {
                eprintln!("{PROGRAM}: warning: failed to load image '{img_path}': {e}. Falling back to default mascot.");
                scene.cow = cow::resolve_cow_name(&scene.cow, &data);
                cow::load_cow(
                    &scene.cow,
                    &data,
                    &scene.eyes,
                    &scene.tongue,
                    thoughts_glyph,
                )
            }
        }
    } else {
        scene.cow = cow::resolve_cow_name(&scene.cow, &data);
        cow::load_cow(
            &scene.cow,
            &data,
            &scene.eyes,
            &scene.tongue,
            thoughts_glyph,
        )
    };
    let composed = cow::compose_scene_with_mode(&cow_text, &scene.text, is_thought);

    let animations = dna::load_animations(&data);
    let mut cow_dna = dna::get_dna(&animations, &scene.cow);
    if let Some(ref pal_str) = scene.palette {
        let hexes: Vec<String> = pal_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !hexes.is_empty() {
            cow_dna.palette = hexes;
        }
    } else if scene.color_mode == "natural"
        || scene.color_mode == "default"
        || scene.color_mode == "animal"
        || scene.color_mode == "animal_natural"
        || cow_dna.palette.is_empty()
    {
        let natural_hexes = crate::color::get_natural_hex_palette(&scene.cow);
        if !natural_hexes.is_empty() {
            cow_dna.palette = natural_hexes.iter().map(|&s| s.to_string()).collect();
        }
    }
    let instance_id = std::process::id();
    // Acquire pane lock: writes our PID to the daemon state file and opens control socket
    // so any second `forgum` invocation on the same pane forwards commands or exits without
    // competing rendering. Cleaned up via Drop when we exit.
    let (_pane_guard, cmd_rx) = PaneGuard::acquire(&session_id, instance_id);

    let result = if scene.background || scene.split_scroll || args.split_scroll {
        render::render_loop_background(
            out,
            scene,
            shutdown,
            Some(&composed),
            cow_dna,
            instance_id,
            data,
            &cmd_rx,
        )
    } else if args.banner {
        render::render_loop_banner(
            out,
            scene,
            shutdown,
            Some(&composed),
            cow_dna,
            instance_id,
            data,
            &cmd_rx,
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
            &cmd_rx,
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

    resolve_scene_randomness(&mut scene, &data, &args);

    let is_thought = scene.think
        || args.command == cli::Command::Think
        || (args.text.is_none() && scene.text.trim().is_empty());

    if scene.text.trim().is_empty() {
        scene.text = fortune::random_fortune(&data)
            .unwrap_or_else(|| "The cow that never moos has the most to say.".to_string());
    }

    scene.cow = cow::resolve_cow_name(&scene.cow, &data);
    let thoughts_glyph = if is_thought { "o" } else { "\\" };
    let cow_text = cow::load_cow(
        &scene.cow,
        &data,
        &scene.eyes,
        &scene.tongue,
        thoughts_glyph,
    );
    let composed = cow::compose_scene_with_mode(&cow_text, &scene.text, is_thought);
    let animations = dna::load_animations(&data);
    let mut cow_dna = dna::get_dna(&animations, &scene.cow);
    if let Some(ref pal_str) = scene.palette {
        let hexes: Vec<String> = pal_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !hexes.is_empty() {
            cow_dna.palette = hexes;
        }
    } else if scene.color_mode == "natural"
        || scene.color_mode == "default"
        || scene.color_mode == "animal"
        || scene.color_mode == "animal_natural"
        || cow_dna.palette.is_empty()
    {
        let natural_hexes = crate::color::get_natural_hex_palette(&scene.cow);
        if !natural_hexes.is_empty() {
            cow_dna.palette = natural_hexes.iter().map(|&s| s.to_string()).collect();
        }
    }
    let instance_id = std::process::id();
    let shutdown = ShutdownFlag::new();

    let is_overlay_mode = scene.background
        || scene.split_scroll
        || args.split_scroll
        || args.background
        || args.reserve_rows.is_some()
        || scene.reserve_rows.is_some()
        || args.split_ratio.is_some()
        || scene.split_ratio.is_some()
        || scene.shell_attach_mode == "split"
        || scene.shell_attach_mode == "reactive";

    let result = if is_overlay_mode {
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
    daemon::cleanup_daemon_state(&session_id);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            ExitCode::from(71)
        }
    }
}
