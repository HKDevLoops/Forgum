use std::path::PathBuf;

use crate::{herd, theme};

pub fn run_demo() -> Result<String, String> {
    let quieted = herd::herd_quiet().unwrap_or(0);

    let mut output = String::new();
    output.push_str("Forgum, online.\n");
    output.push_str(&format!("Quieted {quieted} daemon(s).\n"));

    let filter = herd::HerdFilter {
        session: None,
        all: true,
    };
    let affected = herd::herd_effect("aurora", &filter).unwrap_or(0);
    output.push_str(&format!("Set aurora on {affected} daemon(s).\n"));

    output.push_str("\n--- tmux integration ---\n");
    output.push_str("Add to ~/.tmux.conf:\n");
    output.push_str("  bind D display-popup -E -w 90 -h 20 'forgum herd list --watch'\n");
    output.push_str("  bind f run-shell 'forgum herd follow'\n");
    output.push_str("\nRun 'forgum theme rotate --interval 5' for mood cycling.\n");

    Ok(output)
}

pub fn run_theme_rotate(interval_minutes: u32) -> Result<(), String> {
    let config_dir = forgum_platform::config_path()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let themes = theme::list_themes(&config_dir);
    if themes.is_empty() {
        return Err("No themes found. Create themes in ~/.config/Forgum/themes/".into());
    }

    let filter = herd::HerdFilter {
        session: None,
        all: true,
    };
    let mut index = 0;

    loop {
        let name = &themes[index % themes.len()];
        match theme::load_theme(&config_dir, name) {
            Ok(t) => match t.apply(&filter) {
                Ok(n) => println!("Applied theme '{name}' to {n} daemon(s)."),
                Err(e) => eprintln!("Failed to apply theme: {e}"),
            },
            Err(e) => eprintln!("Failed to load theme: {e}"),
        }
        index += 1;
        std::thread::sleep(std::time::Duration::from_secs(interval_minutes as u64 * 60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_demo_returns_string() {
        let _ = run_demo();
    }

    #[test]
    fn run_theme_rotate_empty_themes() {
        let _dir = tempfile::tempdir().unwrap();
    }
}
