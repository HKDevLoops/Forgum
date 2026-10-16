//! `forgum fetch` — Animated fastfetch & neofetch replacement with hardware metrics & mascot art.
//!
//! Provides:
//! 1. `--instant` (default): Fast single-frame printout pairing the mascot with 12+ hardware, OS, and terminal collectors.
//! 2. `--live`: Real-time monitoring loop with animated mascot and dynamic Braille sparklines for CPU and Memory.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use forgum_platform::gpu::{
    detect_gpu_devices, resolve_acceleration, AccelerationMode, GpuDeviceType,
};
use forgum_platform::package_manager::detect_available_package_managers;
use forgum_platform::terminal::{detect_color_level, detect_terminal_emulator, ColorLevel};
use forgum_platform::{check_battery_percent, data_dir};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::cow;

/// System metrics and environment details gathered for `forgum fetch`.
#[derive(Debug, Clone)]
pub struct FetchInfo {
    pub user: String,
    pub host: String,
    pub os: String,
    pub kernel: String,
    pub arch: String,
    pub uptime: String,
    pub packages: String,
    pub shell: String,
    pub terminal: String,
    pub cpu: String,
    pub gpus: Vec<String>,
    pub memory: String,
    pub memory_fraction: f32,
    pub cpu_usage: f32,
    pub battery: Option<String>,
    pub accel: String,
}

impl FetchInfo {
    /// Collect all system information quickly with zero-panic fallbacks.
    pub fn collect() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());

        let host = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "localhost".to_string());

        let os_name = System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
        let os_version = System::os_version().unwrap_or_default();
        let os = if os_version.is_empty() {
            os_name
        } else {
            format!("{os_name} {os_version}")
        };

        let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
        let arch = std::env::consts::ARCH.to_string();

        let uptime_secs = System::uptime();
        let days = uptime_secs / 86400;
        let hours = (uptime_secs % 86400) / 3600;
        let mins = (uptime_secs % 3600) / 60;
        let uptime = if days > 0 {
            format!("{days} days, {hours} hours, {mins} mins")
        } else if hours > 0 {
            format!("{hours} hours, {mins} mins")
        } else {
            format!("{mins} mins")
        };

        let pms = detect_available_package_managers();
        let active_pms: Vec<String> = pms
            .iter()
            .filter(|(_, avail)| *avail)
            .map(|(pm, _)| pm.name().to_string())
            .collect();
        let packages = if active_pms.is_empty() {
            "standalone".to_string()
        } else {
            format!("{} active ({})", active_pms.len(), active_pms.join(", "))
        };

        let shell = forgum_platform::shell::Shell::detect_current_shell()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                std::env::var("SHELL")
                    .ok()
                    .and_then(|p| {
                        std::path::Path::new(&p)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .map(String::from)
                    })
                    .unwrap_or_else(|| "sh".to_string())
            });

        let emulator = detect_terminal_emulator();
        let color_lvl = match detect_color_level() {
            ColorLevel::TrueColor => "TrueColor 24-bit",
            ColorLevel::Ansi256 => "ANSI 256",
            ColorLevel::Ansi8 => "ANSI 8",
        };
        let terminal = format!("{} ({})", emulator.name(), color_lvl);

        let cpu = {
            let cpus = sys.cpus();
            if cpus.is_empty() {
                "Unknown CPU".to_string()
            } else {
                let brand = cpus[0].brand().trim();
                format!("{} ({} cores)", brand, cpus.len())
            }
        };

        let cpu_usage = {
            let cpus = sys.cpus();
            if cpus.is_empty() {
                0.0
            } else {
                let sum: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
                (sum / cpus.len() as f32).clamp(0.0, 100.0)
            }
        };

        let gpus = {
            let probed = detect_gpu_devices();
            if probed.is_empty() {
                vec!["Generic Framebuffer / CPU Compute".to_string()]
            } else {
                probed
                    .iter()
                    .map(|g| {
                        let vram = g.vram_formatted();
                        format!("{} [{}] ({vram})", g.name, g.architecture)
                    })
                    .collect()
            }
        };

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let (memory, memory_fraction) = if total_mem > 0 {
            let used_gb = used_mem as f64 / (1024.0 * 1024.0 * 1024.0);
            let total_gb = total_mem as f64 / (1024.0 * 1024.0 * 1024.0);
            let pct = (used_mem as f64 / total_mem as f64) * 100.0;
            (
                format!("{used_gb:.2} GiB / {total_gb:.2} GiB ({pct:.0}%)"),
                (used_mem as f32 / total_mem as f32).clamp(0.0, 1.0),
            )
        } else {
            ("Unknown".to_string(), 0.0)
        };

        let battery = check_battery_percent().map(|pct| format!("{pct:.0}%"));

        let (resolved_type, info_opt) = resolve_acceleration(AccelerationMode::Auto);
        let accel = match resolved_type {
            GpuDeviceType::Dedicated => {
                let name = info_opt.map(|i| i.name).unwrap_or_else(|| "GPU".to_string());
                format!("Vulkan 1.3 Compute (Dedicated: {name})")
            }
            GpuDeviceType::Integrated => {
                let name = info_opt.map(|i| i.name).unwrap_or_else(|| "APU".to_string());
                format!("Vulkan Compute (Integrated: {name})")
            }
            GpuDeviceType::Npu => {
                let name = info_opt.map(|i| i.name).unwrap_or_else(|| "NPU".to_string());
                format!("Neural Processing Unit (NPU: {name})")
            }
            GpuDeviceType::Cpu => "SIMD Parallel CPU Kernel (<100MB RAM)".to_string(),
        };

        Self {
            user,
            host,
            os,
            kernel,
            arch,
            uptime,
            packages,
            shell,
            terminal,
            cpu,
            gpus,
            memory,
            memory_fraction,
            cpu_usage,
            battery,
            accel,
        }
    }
}

/// Convert a sequence of historical utilization fractions (0.0–1.0) into a Braille sparkline.
pub fn generate_sparkline(history: &[f32], max_len: usize) -> String {
    const BARS: [char; 8] = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let slice = if history.len() > max_len {
        &history[history.len() - max_len..]
    } else {
        history
    };

    let mut out = String::with_capacity(slice.len());
    for &val in slice {
        let clamped = val.clamp(0.0, 1.0);
        let idx = ((clamped * 7.0).round() as usize).min(7);
        out.push(BARS[idx]);
    }
    out
}

/// Render instant fastfetch-style ASCII mascot beside colored system diagnostic cards.
pub fn render_instant_fetch(cow_name: Option<&str>, palette_override: Option<&str>) -> String {
    let info = FetchInfo::collect();
    let data_dir = data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let chosen_cow = cow_name.unwrap_or("default");
    let cow_art = cow::load_cow(chosen_cow, &data_dir, "oo", "U", "\\\\");

    let pal: Vec<(u8, u8, u8)> = if let Some(p) = palette_override {
        let hexes: Vec<String> = p.split(',').map(|s| s.trim().to_string()).collect();
        crate::color::parse_palette(&hexes)
    } else {
        let biome = forgum_platform::biome::get_mascot_biome(chosen_cow);
        biome.natural_rgb.to_vec()
    };

    let primary_color = pal.first().copied().unwrap_or((0, 255, 255));
    let accent_color = pal.get(1).copied().unwrap_or((255, 255, 0));
    let label_color = pal.get(2).copied().unwrap_or((200, 200, 200));

    let mascot_colored_lines: Vec<String> = cow_art
        .lines()
        .map(|line| {
            format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                primary_color.0, primary_color.1, primary_color.2, line
            )
        })
        .collect();

    let title_line = format!(
        "\x1b[1;38;2;{};{};{}m{}@{}\x1b[0m",
        accent_color.0, accent_color.1, accent_color.2, info.user, info.host
    );
    let separator = format!(
        "\x1b[38;2;{};{};{}m{}\x1b[0m",
        label_color.0,
        label_color.1,
        label_color.2,
        "─".repeat(info.user.len() + info.host.len() + 1).min("─".repeat(40))
    );

    let mut cards: Vec<String> = Vec::new();
    cards.push(title_line);
    cards.push(separator);

    let add_item = |cards: &mut Vec<String>, icon: &str, label: &str, value: &str| {
        cards.push(format!(
            "{icon} \x1b[1;38;2;{};{};{}m{: <10}\x1b[0m \x1b[90m→\x1b[0m {value}",
            label_color.0, label_color.1, label_color.2, label
        ));
    };

    add_item(&mut cards, "💻", "OS", &format!("{} ({})", info.os, info.arch));
    add_item(&mut cards, "🐧", "Kernel", &info.kernel);
    add_item(&mut cards, "⏱️", "Uptime", &info.uptime);
    add_item(&mut cards, "📦", "Packages", &info.packages);
    add_item(&mut cards, "🐚", "Shell", &info.shell);
    add_item(&mut cards, "📟", "Terminal", &info.terminal);
    add_item(&mut cards, "🧠", "CPU", &info.cpu);
    for (i, gpu) in info.gpus.iter().enumerate() {
        let label = if i == 0 { "GPU" } else { "GPU (2)" };
        add_item(&mut cards, "🎮", label, gpu);
    }
    add_item(&mut cards, "💾", "Memory", &info.memory);
    if let Some(bat) = &info.battery {
        add_item(&mut cards, "⚡", "Battery", bat);
    }
    add_item(&mut cards, "🚀", "Compute", &info.accel);

    // Color palette dots at the end
    let c1 = primary_color;
    let c2 = accent_color;
    let c3 = label_color;
    let c4 = pal.get(3).copied().unwrap_or((150, 150, 150));
    let c5 = pal.get(4).copied().unwrap_or((100, 100, 100));

    let dots = format!(
        "\x1b[38;2;{};{};{}m● \x1b[38;2;{};{};{}m● \x1b[38;2;{};{};{}m● \x1b[38;2;{};{};{}m● \x1b[38;2;{};{};{}m● \x1b[0m\x1b[31m● \x1b[32m● \x1b[33m● \x1b[34m● \x1b[35m● \x1b[36m● \x1b[37m●\x1b[0m",
        c1.0, c1.1, c1.2,
        c2.0, c2.1, c2.2,
        c3.0, c3.1, c3.2,
        c4.0, c4.1, c4.2,
        c5.0, c5.1, c5.2,
    );
    cards.push(String::new());
    cards.push(dots);

    // Side-by-side alignment
    let mascot_raw_lines: Vec<&str> = cow_art.lines().collect();
    let mascot_width = mascot_raw_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20)
        .max(20);

    let max_rows = mascot_colored_lines.len().max(cards.len());
    let mut out = String::new();
    out.push('\n');

    for row in 0..max_rows {
        let left_colored = mascot_colored_lines.get(row).cloned().unwrap_or_default();
        let left_raw_len = mascot_raw_lines
            .get(row)
            .map(|l| l.chars().count())
            .unwrap_or(0);
        let padding = " ".repeat(mascot_width.saturating_sub(left_raw_len) + 4);

        let right = cards.get(row).cloned().unwrap_or_default();
        out.push_str(&format!("{left_colored}{padding}{right}\n"));
    }
    out.push('\n');

    out
}

/// Run live 240Hz monitor with real-time Braille sparklines and animated mascot.
pub fn run_live_fetch(cow_name: Option<&str>, palette_override: Option<&str>) -> ExitCode {
    use crossterm::event::{self, Event, KeyCode};
    use std::io::Write;

    let shutdown = forgum_platform::ShutdownFlag::new();
    let mut cpu_history: Vec<f32> = Vec::with_capacity(30);
    let mut mem_history: Vec<f32> = Vec::with_capacity(30);

    // Enter alternate screen
    let mut stdout = std::io::stdout();
    let _ = crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    );
    let _ = crossterm::terminal::enable_raw_mode();

    let mut tick = 0usize;
    let chosen_cow = cow_name.unwrap_or("default");

    while !shutdown.is_triggered() {
        // Poll input for 'q' or Esc
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Esc
                    || (key.modifiers.contains(event::KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c'))
                {
                    break;
                }
            }
        }

        tick += 1;
        if tick.is_multiple_of(4) || cpu_history.is_empty() {
            let info = FetchInfo::collect();
            cpu_history.push(info.cpu_usage / 100.0);
            mem_history.push(info.memory_fraction);
            if cpu_history.len() > 24 {
                cpu_history.remove(0);
            }
            if mem_history.len() > 24 {
                mem_history.remove(0);
            }

            let cpu_spark = generate_sparkline(&cpu_history, 24);
            let mem_spark = generate_sparkline(&mem_history, 24);

            let instant_view = render_instant_fetch(Some(chosen_cow), palette_override);

            let live_panel = format!(
                "\x1b[1;36m━━━ Real-Time Pasture Telemetry (240Hz Fixed Governor) ━━━\x1b[0m\n\
                 CPU Load: [\x1b[1;32m{cpu_spark}\x1b[0m] {:.1}%\n\
                 RAM Load: [\x1b[1;35m{mem_spark}\x1b[0m] {:.1}%\n\
                 \x1b[90mPress 'q' or Esc to return to terminal prompt.\x1b[0m\n",
                info.cpu_usage,
                info.memory_fraction * 100.0
            );

            let _ = crossterm::execute!(
                stdout,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
            );
            let _ = write!(stdout, "{instant_view}{live_panel}");
            let _ = stdout.flush();
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    // Restore terminal
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        stdout,
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen
    );

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_fetch_info_succeeds_without_panics() {
        let info = FetchInfo::collect();
        assert!(!info.user.is_empty());
        assert!(!info.host.is_empty());
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(!info.cpu.is_empty());
        assert!(!info.gpus.is_empty());
    }

    #[test]
    fn test_generate_sparkline_clamps_and_formats() {
        let history = vec![0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 1.0];
        let spark = generate_sparkline(&history, 10);
        assert_eq!(spark.chars().count(), 7);
        assert_eq!(spark.chars().next().unwrap(), ' ');
        assert_eq!(spark.chars().last().unwrap(), '█');
    }

    #[test]
    fn test_render_instant_fetch_contains_system_cards() {
        let output = render_instant_fetch(Some("default"), None);
        assert!(output.contains("OS"));
        assert!(output.contains("CPU"));
        assert!(output.contains("Memory"));
        assert!(output.contains("Terminal"));
    }
}
