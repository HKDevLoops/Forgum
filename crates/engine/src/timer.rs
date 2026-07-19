use std::process::Command;
use std::time::Instant;

#[derive(Debug)]
pub struct TimerResult {
    pub command: String,
    pub duration_secs: f64,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_timer(cmd: &[String]) -> TimerResult {
    let start = Instant::now();
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map(|o| {
            (
                String::from_utf8_lossy(&o.stdout).to_string(),
                String::from_utf8_lossy(&o.stderr).to_string(),
                o.status.code().unwrap_or(-1),
            )
        })
        .unwrap_or_else(|e| (String::new(), format!("Error: {}", e), -1));

    let duration = start.elapsed().as_secs_f64();

    TimerResult {
        command: cmd.join(" "),
        duration_secs: duration,
        exit_code: output.2,
        stdout: output.0,
        stderr: output.1,
    }
}

pub fn format_duration(secs: f64) -> String {
    if secs < 0.001 {
        format!("{:.0}μs", secs * 1_000_000.0)
    } else if secs < 1.0 {
        format!("{:.1}ms", secs * 1_000.0)
    } else if secs < 60.0 {
        format!("{:.2}s", secs)
    } else {
        let mins = secs as u64 / 60;
        let remaining = secs - (mins as f64 * 60.0);
        format!("{}m {:.1}s", mins, remaining)
    }
}

pub fn render_timer_cow(result: &TimerResult) -> String {
    let duration_str = format_duration(result.duration_secs);
    let status = if result.exit_code == 0 { "✓" } else { "✗" };

    format!(
        "  ┌──────────────────────────────┐\n  │  {} {} {:>8}  │\n  │  cmd: {:<22}  │\n  └──────────────────────────────┘",
        status,
        result.command,
        duration_str,
        if result.command.len() > 22 {
            &result.command[..22]
        } else {
            &result.command
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_microseconds() {
        assert_eq!(format_duration(0.0001), "100μs");
    }

    #[test]
    fn format_duration_milliseconds() {
        assert_eq!(format_duration(0.5), "500.0ms");
    }

    #[test]
    fn format_duration_seconds() {
        assert_eq!(format_duration(2.5), "2.50s");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(90.0), "1m 30.0s");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0.0), "0μs");
    }

    #[test]
    fn format_duration_boundary_1ms() {
        assert_eq!(format_duration(0.001), "1.0ms");
    }

    #[test]
    fn format_duration_boundary_1s() {
        assert_eq!(format_duration(1.0), "1.00s");
    }

    #[test]
    fn format_duration_boundary_60s() {
        assert_eq!(format_duration(60.0), "1m 0.0s");
    }

    #[test]
    fn timer_cow_format() {
        let result = TimerResult {
            command: "cargo build".to_string(),
            duration_secs: 1.5,
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        let cow = render_timer_cow(&result);
        assert!(cow.contains("1.50s"));
        assert!(cow.contains("✓"));
    }

    #[test]
    fn timer_cow_failure_shows叉() {
        let result = TimerResult {
            command: "cargo test".to_string(),
            duration_secs: 0.5,
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
        };
        let cow = render_timer_cow(&result);
        assert!(cow.contains("✗"));
    }
}
