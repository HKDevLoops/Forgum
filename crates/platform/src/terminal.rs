//! Terminal capability detection.
//!
//! Determines what the current terminal can render: truecolor (24-bit),
//! 256-color, or basic 8-color. Used by the color module to pick the right
//! escape-sequence flavor.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Color depth tiers we can target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorLevel {
    /// Basic 8 colors (CGA-era palette).
    Ansi8,
    /// 256-color palette (xterm-256color).
    Ansi256,
    /// 24-bit truecolor (modern terminals: kitty, Alacritty, iTerm2, Windows Terminal).
    TrueColor,
}

impl ColorLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ansi8 => "ansi8",
            Self::Ansi256 => "ansi256",
            Self::TrueColor => "truecolor",
        }
    }
}

/// Graphics protocol a terminal may support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsCaps {
    /// No graphics protocol.
    None,
    /// DEC Sixel (`\x1bP...`).
    Sixel,
    /// Kitty graphics protocol (`\x1b_G...`).
    Kitty,
}

/// Known terminal emulator types and environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalEmulator {
    /// Microsoft Windows Terminal (wt.exe) with ConPTY.
    WindowsTerminal,
    /// WezTerm terminal emulator (wezterm-gui / wezterm cli).
    WezTerm,
    /// Kitty GPU-accelerated terminal emulator.
    Kitty,
    /// Alacritty GPU-accelerated VT-standard terminal emulator.
    Alacritty,
    /// Ghostty GPU-accelerated VT-standard terminal emulator.
    Ghostty,
    /// iTerm2 macOS terminal emulator.
    ITerm2,
    /// Foot Wayland-native terminal emulator.
    Foot,
    /// GNOME Terminal and VTE-based terminals (Tilix, Terminator, XFCE Terminal, Guake).
    Vte,
    /// KDE Konsole and Qt-based terminals (Yakuake).
    Konsole,
    /// Traditional X11 / VT terminals (xterm).
    XTerm,
    /// Unicode rxvt (urxvt).
    Urxvt,
    /// MinTTY (Git Bash / Cygwin / MSYS2).
    Mintty,
    /// Apple macOS default Terminal.app.
    AppleTerminal,
    /// Legacy Windows Console Host (conhost.exe without modern ConPTY / VT margins).
    ConHost,
    /// Linux Virtual Console (Kernel VT driver / FBDev `/dev/tty1-6`).
    LinuxConsole,
    /// Headless serial console or hypervisor terminal.
    SerialConsole,
    /// Non-interactive or dumb terminal (`TERM=dumb`, raw pipe).
    Dumb,
    /// Generic ANSI/VT-compatible terminal emulator.
    GenericVt,
}

impl TerminalEmulator {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WindowsTerminal => "Windows Terminal",
            Self::WezTerm => "WezTerm",
            Self::Kitty => "Kitty",
            Self::Alacritty => "Alacritty",
            Self::Ghostty => "Ghostty",
            Self::ITerm2 => "iTerm2",
            Self::Foot => "Foot",
            Self::Vte => "VTE (GNOME Terminal/Tilix)",
            Self::Konsole => "Konsole",
            Self::XTerm => "XTerm",
            Self::Urxvt => "urxvt",
            Self::Mintty => "Mintty",
            Self::AppleTerminal => "Apple Terminal",
            Self::ConHost => "Legacy Windows ConHost",
            Self::LinuxConsole => "Linux Virtual Console (TTY)",
            Self::SerialConsole => "Serial/Hypervisor Console",
            Self::Dumb => "Dumb / Non-TTY",
            Self::GenericVt => "Generic VT",
        }
    }

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::WindowsTerminal => "windows_terminal",
            Self::WezTerm => "wezterm",
            Self::Kitty => "kitty",
            Self::Alacritty => "alacritty",
            Self::Ghostty => "ghostty",
            Self::ITerm2 => "iterm2",
            Self::Foot => "foot",
            Self::Vte => "vte",
            Self::Konsole => "konsole",
            Self::XTerm => "xterm",
            Self::Urxvt => "urxvt",
            Self::Mintty => "mintty",
            Self::AppleTerminal => "apple_terminal",
            Self::ConHost => "conhost",
            Self::LinuxConsole => "linux_console",
            Self::SerialConsole => "serial_console",
            Self::Dumb => "dumb",
            Self::GenericVt => "generic_vt",
        }
    }

    /// Whether this terminal honors DEC Set Top and Bottom Margins (`\x1b[top;bottom r`).
    #[must_use]
    pub const fn supports_decstbm(self) -> bool {
        match self {
            Self::WindowsTerminal
            | Self::WezTerm
            | Self::Kitty
            | Self::Alacritty
            | Self::Ghostty
            | Self::ITerm2
            | Self::Foot
            | Self::Vte
            | Self::Konsole
            | Self::XTerm
            | Self::Urxvt
            | Self::Mintty
            | Self::AppleTerminal
            | Self::GenericVt => true,
            Self::ConHost | Self::LinuxConsole | Self::SerialConsole | Self::Dumb => false,
        }
    }

    /// Whether this terminal supports DEC Set Left and Right Margins (`\x1b[?69h` + `\x1b[left;right s`).
    #[must_use]
    pub const fn supports_decslrm(self) -> bool {
        matches!(
            self,
            Self::WezTerm
                | Self::Alacritty
                | Self::Ghostty
                | Self::ITerm2
                | Self::Foot
                | Self::XTerm
        )
    }

    /// Whether this terminal honors DEC 2026 synchronized updates (`\x1b[?2026h`).
    #[must_use]
    pub const fn supports_sync_updates(self) -> bool {
        matches!(
            self,
            Self::WindowsTerminal
                | Self::WezTerm
                | Self::Kitty
                | Self::Ghostty
                | Self::ITerm2
                | Self::Foot
                | Self::Alacritty
        )
    }

    /// Whether an external native split-pane API or CLI command exists for this emulator.
    #[must_use]
    pub const fn native_split_available(self) -> bool {
        matches!(
            self,
            Self::WindowsTerminal | Self::WezTerm | Self::Kitty | Self::ITerm2
        )
    }

    /// Whether this terminal natively supports 24-bit TrueColor RGB.
    #[must_use]
    pub const fn supports_truecolor(self) -> bool {
        match self {
            Self::WindowsTerminal
            | Self::WezTerm
            | Self::Kitty
            | Self::Ghostty
            | Self::Alacritty
            | Self::Foot
            | Self::ITerm2
            | Self::Mintty
            | Self::Vte
            | Self::Konsole => true,
            Self::ConHost | Self::LinuxConsole | Self::SerialConsole | Self::Dumb => false,
            Self::AppleTerminal | Self::Urxvt | Self::XTerm | Self::GenericVt => false,
        }
    }

    /// Known hardware, virtualization, or firmware limitation notes.
    #[must_use]
    pub const fn limitation_notes(self) -> Option<&'static str> {
        match self {
            Self::ConHost => Some(
                "Legacy Windows ConHost lacks DECSTBM scroll isolation; use Windows Terminal or precmd fallback.",
            ),
            Self::LinuxConsole => Some(
                "Linux Virtual Console (Kernel VT) does not support DECSTBM scroll isolation; banner or precmd redraw recommended.",
            ),
            Self::SerialConsole => Some(
                "Serial / Hypervisor console detected; high-latency or minimal VT support.",
            ),
            Self::Dumb => Some(
                "Dumb terminal or raw pipe; cursor positioning and margins disabled.",
            ),
            _ => None,
        }
    }

    /// Recommended split shell adapter mode for this terminal.
    #[must_use]
    pub const fn recommended_split_mode(self) -> SplitMode {
        if !self.supports_decstbm() {
            if matches!(self, Self::Dumb) {
                SplitMode::Disabled
            } else {
                SplitMode::PrecmdFallback
            }
        } else {
            SplitMode::Decstbm
        }
    }
}

/// Split shell execution adapter modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitMode {
    /// Seamless single-pane simultaneous execution (DECSTBM hardware margin scroll region).
    Seamless,
    /// Mode 1: Pure DECSTBM margin split (Standard for Alacritty, Ghostty, Foot, VTE, Konsole, XTerm, Mintty, macOS Terminal, modern Windows Terminal).
    Decstbm,
    /// Mode 2: Native Multiplexer / Terminal API split (tmux, zellij, wezterm cli, kitty remote, wt.exe).
    NativeApi,
    /// Mode 3: Dynamic Shell Precmd Redraw Fallback (for terminals lacking DECSTBM like legacy ConHost, Linux TTY, VMs).
    PrecmdFallback,
    /// Mode 4: Plain text / banner fallback (unsupported / dumb / pipes).
    Disabled,
}

impl SplitMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Seamless => "seamless",
            Self::Decstbm => "decstbm",
            Self::NativeApi => "native_api",
            Self::PrecmdFallback => "precmd_fallback",
            Self::Disabled => "disabled",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "seamless" => Some(Self::Seamless),
            "decstbm" => Some(Self::Decstbm),
            "native" | "native_api" => Some(Self::NativeApi),
            "precmd" | "precmd_fallback" => Some(Self::PrecmdFallback),
            "disabled" | "none" => Some(Self::Disabled),
            _ => None,
        }
    }
}

/// Native split pane execution plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSplitPlan {
    pub target: &'static str,
    pub program: String,
    pub args: Vec<String>,
    pub explanation: &'static str,
}

/// Plan a native multiplexer or terminal split pane command.
pub fn plan_native_split(
    emulator: TerminalEmulator,
    mux: &crate::mux::Mux,
    rows: usize,
    ratio: f32,
    forgum_args: &[String],
) -> Option<NativeSplitPlan> {
    // 1. Multiplexers take precedence when active
    match mux {
        crate::mux::Mux::Tmux { .. } => {
            let mut args = vec![
                "split-window".to_string(),
                "-b".to_string(),
                "-v".to_string(),
                "-l".to_string(),
                rows.max(1).to_string(),
            ];
            args.extend_from_slice(forgum_args);
            return Some(NativeSplitPlan {
                target: "tmux",
                program: "tmux".to_string(),
                args,
                explanation: "Creates a native tmux split pane docked above the current pane",
            });
        }
        crate::mux::Mux::Zellij { .. } => {
            let mut args = vec![
                "action".to_string(),
                "new-pane".to_string(),
                "-d".to_string(),
                "up".to_string(),
                "--".to_string(),
            ];
            args.extend_from_slice(forgum_args);
            return Some(NativeSplitPlan {
                target: "zellij",
                program: "zellij".to_string(),
                args,
                explanation: "Creates a native Zellij pane docked upward",
            });
        }
        crate::mux::Mux::WezTerm { .. } => {
            let percent = ((ratio.clamp(0.1, 0.8)) * 100.0).round() as u32;
            let mut args = vec![
                "cli".to_string(),
                "split-pane".to_string(),
                "--top".to_string(),
                "--percent".to_string(),
                percent.to_string(),
                "--".to_string(),
            ];
            args.extend_from_slice(forgum_args);
            return Some(NativeSplitPlan {
                target: "wezterm",
                program: "wezterm".to_string(),
                args,
                explanation: "Creates a native WezTerm top pane via wezterm cli",
            });
        }
        _ => {}
    }

    // 2. Terminal emulator native split APIs
    match emulator {
        TerminalEmulator::WezTerm => {
            let percent = ((ratio.clamp(0.1, 0.8)) * 100.0).round() as u32;
            let mut args = vec![
                "cli".to_string(),
                "split-pane".to_string(),
                "--top".to_string(),
                "--percent".to_string(),
                percent.to_string(),
                "--".to_string(),
            ];
            args.extend_from_slice(forgum_args);
            Some(NativeSplitPlan {
                target: "wezterm",
                program: "wezterm".to_string(),
                args,
                explanation: "Creates a native WezTerm top pane via wezterm cli",
            })
        }
        TerminalEmulator::Kitty => {
            let percent = ((ratio.clamp(0.1, 0.8)) * 100.0).round() as u32;
            let mut args = vec![
                "@".to_string(),
                "launch".to_string(),
                "--location=hsplit".to_string(),
                format!("--bias={percent}"),
            ];
            args.extend_from_slice(forgum_args);
            Some(NativeSplitPlan {
                target: "kitty",
                program: "kitty".to_string(),
                args,
                explanation: "Creates a native Kitty horizontal split via kitty remote control",
            })
        }
        TerminalEmulator::WindowsTerminal => {
            let mut args = vec![
                "-w".to_string(),
                "0".to_string(),
                "sp".to_string(),
                "-H".to_string(),
                "-s".to_string(),
                format!("{:.2}", ratio.clamp(0.1, 0.8)),
            ];
            args.extend_from_slice(forgum_args);
            Some(NativeSplitPlan {
                target: "windows_terminal",
                program: "wt.exe".to_string(),
                args,
                explanation:
                    "Creates a native Windows Terminal horizontal split pane via wt.exe CLI",
            })
        }
        TerminalEmulator::ITerm2 => {
            let script = format!(
                "tell application \"iTerm2\" to tell current session of current window to split horizontally with default profile command \"{}\"",
                forgum_args.join(" ")
            );
            Some(NativeSplitPlan {
                target: "iterm2",
                program: "osascript".to_string(),
                args: vec!["-e".to_string(), script],
                explanation: "Creates a native iTerm2 horizontal split pane via AppleScript",
            })
        }
        _ => None,
    }
}

/// Detect the terminal emulator identity from the process environment.
#[must_use]
pub fn detect_terminal_emulator() -> TerminalEmulator {
    // Windows Terminal
    if std::env::var_os("WT_SESSION").is_some() || std::env::var_os("WT_PROFILE_ID").is_some() {
        return TerminalEmulator::WindowsTerminal;
    }

    // WezTerm
    if std::env::var_os("WEZTERM_PANE").is_some()
        || std::env::var_os("WEZTERM_EXECUTABLE").is_some()
        || std::env::var_os("WEZTERM_UNIX_SOCKET").is_some()
    {
        return TerminalEmulator::WezTerm;
    }

    // Kitty
    if std::env::var_os("KITTY_WINDOW_ID").is_some() || std::env::var_os("KITTY_PID").is_some() {
        return TerminalEmulator::Kitty;
    }

    // Ghostty
    if std::env::var_os("GHOSTTY_RESOURCES_DIR").is_some() {
        return TerminalEmulator::Ghostty;
    }

    // Alacritty
    if std::env::var_os("ALACRITTY_LOG").is_some()
        || std::env::var_os("ALACRITTY_WINDOW_ID").is_some()
        || std::env::var_os("ALACRITTY_SOCKET").is_some()
    {
        return TerminalEmulator::Alacritty;
    }

    // Foot
    if std::env::var_os("FOOT_SERVER_SOCKET").is_some() {
        return TerminalEmulator::Foot;
    }

    // iTerm2
    if std::env::var_os("ITERM_SESSION_ID").is_some() {
        return TerminalEmulator::ITerm2;
    }

    // Mintty
    if std::env::var_os("MINTTY_SHORTCUT").is_some() {
        return TerminalEmulator::Mintty;
    }

    // VTE-based (GNOME Terminal, Tilix, Terminator, XFCE Terminal)
    if std::env::var_os("VTE_VERSION").is_some()
        || std::env::var_os("TILIX_ID").is_some()
        || std::env::var_os("TERMINATOR_UUID").is_some()
    {
        return TerminalEmulator::Vte;
    }

    // Konsole / Yakuake
    if std::env::var_os("KONSOLE_VERSION").is_some()
        || std::env::var_os("KONSOLE_DBUS_SERVICE").is_some()
        || std::env::var_os("KONSOLE_DBUS_SESSION").is_some()
    {
        return TerminalEmulator::Konsole;
    }

    // Urxvt
    if std::env::var_os("RXVT_SOCKET").is_some() {
        return TerminalEmulator::Urxvt;
    }

    // XTerm
    if std::env::var_os("XTERM_VERSION").is_some() {
        return TerminalEmulator::XTerm;
    }

    // Check TERM_PROGRAM
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        let tp_lower = tp.to_ascii_lowercase();
        match tp_lower.as_str() {
            "wezterm" => return TerminalEmulator::WezTerm,
            "kitty" => return TerminalEmulator::Kitty,
            "ghostty" => return TerminalEmulator::Ghostty,
            "alacritty" => return TerminalEmulator::Alacritty,
            "foot" => return TerminalEmulator::Foot,
            "iterm.app" | "iterm2" => return TerminalEmulator::ITerm2,
            "mintty" => return TerminalEmulator::Mintty,
            "apple_terminal" => return TerminalEmulator::AppleTerminal,
            "vscode" => return TerminalEmulator::GenericVt,
            _ => {}
        }
    }

    // Check TERM
    if let Ok(term) = std::env::var("TERM") {
        let term_lower = term.to_ascii_lowercase();
        if term_lower == "dumb" {
            return TerminalEmulator::Dumb;
        }
        if term_lower == "linux" {
            return TerminalEmulator::LinuxConsole;
        }
        if term_lower.contains("ghostty") {
            return TerminalEmulator::Ghostty;
        }
        if term_lower.contains("kitty") {
            return TerminalEmulator::Kitty;
        }
        if term_lower.contains("alacritty") {
            return TerminalEmulator::Alacritty;
        }
        if term_lower.contains("foot") {
            return TerminalEmulator::Foot;
        }
        if term_lower.contains("rxvt") {
            return TerminalEmulator::Urxvt;
        }
        if term_lower == "xterm" || term_lower.contains("xterm") {
            return TerminalEmulator::XTerm;
        }
        if matches!(
            term_lower.as_str(),
            "vt100" | "vt102" | "vt220" | "serial" | "cons25"
        ) {
            return TerminalEmulator::SerialConsole;
        }
    }

    // Fallback for Windows: if on Windows OS and no modern terminal matched, it's legacy ConHost
    if cfg!(windows) || std::env::consts::OS == "windows" {
        return TerminalEmulator::ConHost;
    }

    TerminalEmulator::GenericVt
}

/// Cached terminal capability snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCapabilities {
    pub color: ColorLevel,
    pub width: u16,
    pub height: u16,
    pub is_tty: bool,
    /// Whether the terminal honors DEC 2026 synchronized updates. Conservative (false) by default.
    pub sync: bool,
    /// Graphics protocol the terminal is believed to support. Default None.
    pub graphics: GraphicsCaps,
    /// Detected terminal emulator identity.
    pub emulator: TerminalEmulator,
    /// Recommended split shell adapter mode.
    pub split_mode: SplitMode,
}

impl TerminalCapabilities {
    /// Force a fresh probe (overrides the cached result for this call only).
    pub fn probe() -> Self {
        let (width, height) = terminal_size();
        let color = detect_color_level();
        let is_tty = is_stdout_tty();
        let sync = detect_sync_support();
        let graphics = detect_graphics_cap();
        let emulator = detect_terminal_emulator();
        let has_term = is_tty || has_controlling_terminal();
        let split_mode = if !has_term {
            SplitMode::Disabled
        } else {
            emulator.recommended_split_mode()
        };
        Self {
            color,
            width,
            height,
            is_tty,
            sync,
            graphics,
            emulator,
            split_mode,
        }
    }
}

/// Process-wide cached capabilities. Recomputed once per process; users who
/// need to re-probe (e.g., after a resize) call [`TerminalCapabilities::probe`].
static CAPABILITIES: OnceLock<TerminalCapabilities> = OnceLock::new();

/// Return the cached capabilities (initializing on first call).
#[must_use]
pub fn detect_capabilities() -> TerminalCapabilities {
    CAPABILITIES
        .get_or_init(TerminalCapabilities::probe)
        .to_owned()
}

/// Query the controlling terminal dimensions directly from `CONOUT$` (Windows)
/// or `/dev/tty` (Unix). This is essential for background daemons or processes
/// spawned with stdout redirected, ensuring they detect the true terminal size
/// instead of falling back to 80x24.
#[cfg(windows)]
#[allow(unsafe_code)]
pub fn query_controlling_terminal_size() -> Option<(u16, u16)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Console::{
        GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO,
    };
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("CONOUT$")
        .ok()?;
    let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
    unsafe {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if GetConsoleScreenBufferInfo(handle, &mut csbi) != 0 {
            let w = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
            let h = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        }
    }
    None
}

#[cfg(unix)]
#[allow(unsafe_code)]
pub fn query_controlling_terminal_size() -> Option<(u16, u16)> {
    use std::os::unix::io::AsRawFd;
    let file = std::fs::File::open("/dev/tty").ok()?;
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(file.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) == 0
            && ws.ws_col > 0
            && ws.ws_row > 0
        {
            return Some((ws.ws_col, ws.ws_row));
        }
    }
    None
}

#[cfg(not(any(windows, unix)))]
pub fn query_controlling_terminal_size() -> Option<(u16, u16)> {
    None
}

/// Check if a controlling terminal is available (via CONOUT$ on Windows or /dev/tty on Unix).
#[must_use]
pub fn has_controlling_terminal() -> bool {
    #[cfg(windows)]
    {
        std::fs::OpenOptions::new()
            .write(true)
            .open("CONOUT$")
            .is_ok()
    }
    #[cfg(unix)]
    {
        std::path::Path::new("/dev/tty").exists()
    }
    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}

/// Probes whether the current terminal and environment support UTF-8 encoding.
#[must_use]
pub fn detect_utf8_support() -> bool {
    #[cfg(windows)]
    {
        if std::env::var("WT_SESSION").is_ok()
            || std::env::var("WEZTERM_PANE").is_ok()
            || std::env::var("ALACRITTY_LOG").is_ok()
            || std::env::var("GHOSTTY_RESOURCES_DIR").is_ok()
            || std::env::var("ConEmuPID").is_ok()
        {
            return true;
        }
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let lower = val.to_ascii_lowercase();
                if lower.contains("utf-8") || lower.contains("utf8") {
                    return true;
                }
            }
        }
        #[allow(unsafe_code)]
        let cp = unsafe { windows_sys::Win32::System::Console::GetConsoleOutputCP() };
        if cp == 65001 {
            return true;
        }
        false
    }
    #[cfg(unix)]
    {
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let lower = val.to_ascii_lowercase();
                if lower.contains("utf-8") || lower.contains("utf8") {
                    return true;
                }
            }
        }
        if std::env::var("TERM_PROGRAM").is_ok()
            || std::env::var("COLORTERM").is_ok()
            || std::env::var("KITTY_WINDOW_ID").is_ok()
        {
            return true;
        }
        // Modern Unix environments default to UTF-8
        true
    }
    #[cfg(not(any(windows, unix)))]
    {
        true
    }
}

/// Read terminal size from stdout or controlling terminal. Falls back to (80, 24) on error.
#[must_use]
pub fn terminal_size() -> (u16, u16) {
    if let Ok((w, h)) = crossterm::terminal::size() {
        if w > 0 && h > 0 {
            return (w, h);
        }
    }
    if let Some((w, h)) = query_controlling_terminal_size() {
        return (w, h);
    }
    (80, 24)
}

#[must_use]
pub fn is_stdout_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&std::io::stdout())
}

/// Detect the color level supported by the current terminal.
///
/// Detection order:
/// 1. `COLORTERM=truecolor` or `=24bit` → [`ColorLevel::TrueColor`].
/// 2. `TERM` contains `256color` → [`ColorLevel::Ansi256`].
/// 3. `TERM` is unset or `dumb` → [`ColorLevel::Ansi8`].
/// 4. Fallback: [`ColorLevel::Ansi8`].
#[must_use]
pub fn detect_color_level() -> ColorLevel {
    if let Ok(ct) = std::env::var("COLORTERM") {
        let ct = ct.to_ascii_lowercase();
        if ct == "truecolor" || ct == "24bit" {
            return ColorLevel::TrueColor;
        }
    }
    // Check known emulator capabilities (Windows Terminal, WezTerm, Ghostty, Alacritty, Kitty, Foot, iTerm2, etc.)
    if detect_terminal_emulator().supports_truecolor() {
        return ColorLevel::TrueColor;
    }
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        match tp.to_ascii_lowercase().as_str() {
            "vscode" | "tabby" | "warp" | "hyper" => return ColorLevel::TrueColor,
            _ => {}
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        let term = term.to_ascii_lowercase();
        if term.contains("truecolor") || term.contains("24bit") {
            return ColorLevel::TrueColor;
        }
        if term.contains("256color") || term.contains("256-color") {
            return ColorLevel::Ansi256;
        }
        if term == "dumb" || term.is_empty() {
            return ColorLevel::Ansi8;
        }
    }
    ColorLevel::Ansi256 // reasonable default for modern Linux/macOS
}

/// Whether the terminal is believed to support DEC 2026 synchronized updates.
///
/// Returns `true` only when stdout is a tty AND either the detected emulator
/// supports it, or an allowlisted `TERM_PROGRAM` is set.
#[must_use]
pub fn detect_sync_support() -> bool {
    if !is_stdout_tty() && !has_controlling_terminal() {
        return false;
    }
    let emu = detect_terminal_emulator();
    if emu.supports_sync_updates() {
        return true;
    }
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        match tp.to_ascii_lowercase().as_str() {
            "vscode" | "tabby" | "hyper" => return true,
            _ => {}
        }
    }
    false
}

/// Best-effort detection of a graphics protocol the terminal understands.
///
/// Mirrors the env-hint logic used by the Sixel/Kitty backend, but returns a
/// cfg-free enum instead of the (feature-gated) `Protocol`. Errs toward
/// [`GraphicsCaps::None`] so the default ANSI path is never regressed.
#[must_use]
pub fn detect_graphics_cap() -> GraphicsCaps {
    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let term_program = std::env::var("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();

    if term.contains("sixel")
        || term_program.contains("mlterm")
        || term_program.contains("foot")
        || term_program.contains("wezterm")
        || term_program.contains("tabby")
    {
        return GraphicsCaps::Sixel;
    }
    if term_program.contains("kitty")
        || term.contains("kitty")
        || std::env::var_os("KITTY_WINDOW_ID").is_some()
    {
        return GraphicsCaps::Kitty;
    }
    GraphicsCaps::None
}

/// Runtime gate for the synchronized-update escape sequences.
///
/// Combines the conservative capability probe into a single bool the engine can
/// branch on at runtime (no `#[cfg]` required in `engine/src`).
#[must_use]
pub fn terminal_supports_sync() -> bool {
    detect_sync_support()
}

/// Return the row reserved for the prompt. The engine never writes to rows
/// at or below this index in background mode.
#[must_use]
pub fn overlay_height(total_rows: u16) -> u16 {
    // Reserve the bottom 3 rows (typical prompt height). Floor of 1 row so
    // even a 1-row terminal gets a visible overlay.
    total_rows.saturating_sub(3).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // `cargo test` runs the unit tests of one binary in parallel by
    // default, on threads that share the *process-wide* env. Tests
    // below mutate `TERM` and `TERM_PROGRAM`. Without this mutex a
    // sibling test running on another worker thread racing the same
    // mutation exposes non-deterministic state at the assert site
    // (CI log: "left: None, right: Kitty" under `cargo test --workspace`
    // because a `set_var("TERM_PROGRAM", "WezTerm")` somewhere else
    // interleaved with our `set_var("TERM_PROGRAM", "kitty")`).
    //
    // The lock is process-global; only the env-mutating tests take it.
    // Tests that don't touch env run as before and stay parallel.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn size_fallback_sane() {
        let (w, h) = terminal_size();
        assert!(w > 0 && w <= 1000);
        assert!(h > 0 && h <= 1000);
    }

    #[test]
    fn overlay_height_clamps() {
        assert_eq!(overlay_height(40), 37);
        assert_eq!(overlay_height(3), 1);
        assert_eq!(overlay_height(1), 1);
        assert_eq!(overlay_height(0), 1);
    }

    #[test]
    fn color_level_strings() {
        assert_eq!(ColorLevel::Ansi8.as_str(), "ansi8");
        assert_eq!(ColorLevel::Ansi256.as_str(), "ansi256");
        assert_eq!(ColorLevel::TrueColor.as_str(), "truecolor");
    }

    #[test]
    fn detect_is_stable() {
        // Just make sure it doesn't panic.
        let _ = detect_color_level();
    }

    #[test]
    fn capabilities_struct_has_sync_and_graphics_fields() {
        let caps = TerminalCapabilities::probe();
        // Type checks: sync is a bool, graphics is GraphicsCaps.
        let _: bool = caps.sync;
        let _: GraphicsCaps = caps.graphics;
    }

    #[test]
    fn detect_sync_support_conservative_fallback() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let saved_wt_session = std::env::var_os("WT_SESSION");
        let saved_wt_profile = std::env::var_os("WT_PROFILE_ID");
        let saved_tp = std::env::var_os("TERM_PROGRAM");
        let saved_term = std::env::var_os("TERM");

        // No allowlisted program and no tty-ish env → conservative false.
        std::env::remove_var("TERM_PROGRAM");
        std::env::remove_var("WT_SESSION");
        std::env::remove_var("WT_PROFILE_ID");
        std::env::set_var("TERM", "dumb");

        // With dumb terminal and no known program, detect_sync_support must be false.
        assert!(!detect_sync_support());

        // An allowlisted program returns true (when stdout is a tty).
        std::env::set_var("TERM_PROGRAM", "WezTerm");
        if is_stdout_tty() {
            assert!(detect_sync_support());
        }

        // Restore environment
        if let Some(v) = saved_wt_session {
            std::env::set_var("WT_SESSION", v);
        } else {
            std::env::remove_var("WT_SESSION");
        }
        if let Some(v) = saved_wt_profile {
            std::env::set_var("WT_PROFILE_ID", v);
        } else {
            std::env::remove_var("WT_PROFILE_ID");
        }
        if let Some(v) = saved_tp {
            std::env::set_var("TERM_PROGRAM", v);
        } else {
            std::env::remove_var("TERM_PROGRAM");
        }
        if let Some(v) = saved_term {
            std::env::set_var("TERM", v);
        } else {
            std::env::remove_var("TERM");
        }
    }

    #[test]
    fn detect_graphics_cap_kitty_and_sixel() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        std::env::remove_var("TERM");
        std::env::remove_var("TERM_PROGRAM");
        assert_eq!(detect_graphics_cap(), GraphicsCaps::None);

        std::env::set_var("TERM_PROGRAM", "kitty");
        assert_eq!(detect_graphics_cap(), GraphicsCaps::Kitty);
        std::env::remove_var("TERM_PROGRAM");

        std::env::set_var("TERM", "sixel");
        assert_eq!(detect_graphics_cap(), GraphicsCaps::Sixel);
        std::env::remove_var("TERM");
    }

    #[test]
    fn probe_populates_sync_and_graphics() {
        let caps = TerminalCapabilities::probe();
        let _: bool = caps.sync;
        let _: GraphicsCaps = caps.graphics;
        let _: TerminalEmulator = caps.emulator;
        let _: SplitMode = caps.split_mode;
    }

    #[test]
    fn terminal_emulator_names_and_ids() {
        let emulators = [
            (
                TerminalEmulator::WindowsTerminal,
                "Windows Terminal",
                "windows_terminal",
            ),
            (TerminalEmulator::WezTerm, "WezTerm", "wezterm"),
            (TerminalEmulator::Kitty, "Kitty", "kitty"),
            (TerminalEmulator::Alacritty, "Alacritty", "alacritty"),
            (TerminalEmulator::Ghostty, "Ghostty", "ghostty"),
            (TerminalEmulator::ITerm2, "iTerm2", "iterm2"),
            (TerminalEmulator::Foot, "Foot", "foot"),
            (TerminalEmulator::Vte, "VTE (GNOME Terminal/Tilix)", "vte"),
            (TerminalEmulator::Konsole, "Konsole", "konsole"),
            (TerminalEmulator::XTerm, "XTerm", "xterm"),
            (TerminalEmulator::Urxvt, "urxvt", "urxvt"),
            (TerminalEmulator::Mintty, "Mintty", "mintty"),
            (
                TerminalEmulator::AppleTerminal,
                "Apple Terminal",
                "apple_terminal",
            ),
            (
                TerminalEmulator::ConHost,
                "Legacy Windows ConHost",
                "conhost",
            ),
            (
                TerminalEmulator::LinuxConsole,
                "Linux Virtual Console (TTY)",
                "linux_console",
            ),
            (
                TerminalEmulator::SerialConsole,
                "Serial/Hypervisor Console",
                "serial_console",
            ),
            (TerminalEmulator::Dumb, "Dumb / Non-TTY", "dumb"),
            (TerminalEmulator::GenericVt, "Generic VT", "generic_vt"),
        ];

        for (emu, name, id) in emulators {
            assert_eq!(emu.name(), name);
            assert_eq!(emu.id(), id);
        }
    }

    #[test]
    fn terminal_emulator_decstbm_and_split_modes() {
        // Modern terminals support DECSTBM and default to Decstbm mode
        assert!(TerminalEmulator::WindowsTerminal.supports_decstbm());
        assert_eq!(
            TerminalEmulator::WindowsTerminal.recommended_split_mode(),
            SplitMode::Decstbm
        );

        assert!(TerminalEmulator::WezTerm.supports_decstbm());
        assert!(TerminalEmulator::WezTerm.supports_decslrm());
        assert!(TerminalEmulator::WezTerm.native_split_available());

        assert!(TerminalEmulator::Kitty.supports_decstbm());
        assert!(TerminalEmulator::Kitty.native_split_available());

        assert!(TerminalEmulator::Alacritty.supports_decstbm());
        assert!(TerminalEmulator::Ghostty.supports_decstbm());
        assert!(TerminalEmulator::Foot.supports_decstbm());
        assert!(TerminalEmulator::Vte.supports_decstbm());
        assert!(TerminalEmulator::Konsole.supports_decstbm());
        assert!(TerminalEmulator::ITerm2.supports_decstbm());

        // Hardware / legacy limitations
        assert!(!TerminalEmulator::ConHost.supports_decstbm());
        assert_eq!(
            TerminalEmulator::ConHost.recommended_split_mode(),
            SplitMode::PrecmdFallback
        );
        assert!(TerminalEmulator::ConHost.limitation_notes().is_some());

        assert!(!TerminalEmulator::LinuxConsole.supports_decstbm());
        assert_eq!(
            TerminalEmulator::LinuxConsole.recommended_split_mode(),
            SplitMode::PrecmdFallback
        );
        assert!(TerminalEmulator::LinuxConsole.limitation_notes().is_some());

        assert!(!TerminalEmulator::Dumb.supports_decstbm());
        assert_eq!(
            TerminalEmulator::Dumb.recommended_split_mode(),
            SplitMode::Disabled
        );
    }

    #[test]
    fn plan_native_split_multiplexers_and_apis() {
        let no_mux = crate::mux::Mux::None;
        let tmux_mux = crate::mux::Mux::Tmux {
            pane: "%1".into(),
            session: "dev".into(),
        };
        let zellij_mux = crate::mux::Mux::Zellij { tab: "main".into() };

        let args = vec![
            "render".to_string(),
            "--animal".to_string(),
            "tux".to_string(),
        ];

        // Tmux plan
        let plan_tmux = plan_native_split(TerminalEmulator::GenericVt, &tmux_mux, 12, 0.35, &args);
        assert!(plan_tmux.is_some());
        let p_tmux = plan_tmux.unwrap();
        assert_eq!(p_tmux.target, "tmux");
        assert_eq!(p_tmux.program, "tmux");
        assert!(p_tmux.args.contains(&"split-window".to_string()));
        assert!(p_tmux.args.contains(&"12".to_string()));

        // Zellij plan
        let plan_zellij =
            plan_native_split(TerminalEmulator::GenericVt, &zellij_mux, 12, 0.35, &args);
        assert!(plan_zellij.is_some());
        let p_zellij = plan_zellij.unwrap();
        assert_eq!(p_zellij.target, "zellij");
        assert_eq!(p_zellij.program, "zellij");

        // WezTerm plan
        let plan_wez = plan_native_split(TerminalEmulator::WezTerm, &no_mux, 10, 0.30, &args);
        assert!(plan_wez.is_some());
        let p_wez = plan_wez.unwrap();
        assert_eq!(p_wez.target, "wezterm");
        assert_eq!(p_wez.program, "wezterm");
        assert!(p_wez.args.contains(&"split-pane".to_string()));
        assert!(p_wez.args.contains(&"30".to_string()));

        // Kitty plan
        let plan_kitty = plan_native_split(TerminalEmulator::Kitty, &no_mux, 10, 0.25, &args);
        assert!(plan_kitty.is_some());
        let p_kitty = plan_kitty.unwrap();
        assert_eq!(p_kitty.target, "kitty");
        assert_eq!(p_kitty.program, "kitty");
        assert!(p_kitty.args.contains(&"--bias=25".to_string()));

        // Windows Terminal plan
        let plan_wt =
            plan_native_split(TerminalEmulator::WindowsTerminal, &no_mux, 10, 0.35, &args);
        assert!(plan_wt.is_some());
        let p_wt = plan_wt.unwrap();
        assert_eq!(p_wt.target, "windows_terminal");
        assert_eq!(p_wt.program, "wt.exe");
        assert!(p_wt.args.contains(&"sp".to_string()));

        // Alacritty has no native split API (pure VT)
        let plan_alacritty =
            plan_native_split(TerminalEmulator::Alacritty, &no_mux, 10, 0.35, &args);
        assert!(plan_alacritty.is_none());
    }

    #[test]
    fn detect_terminal_emulator_environment_variables() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());

        // Helper to clear common emulator indicators
        let clear_vars = || {
            std::env::remove_var("WT_SESSION");
            std::env::remove_var("WT_PROFILE_ID");
            std::env::remove_var("WEZTERM_PANE");
            std::env::remove_var("WEZTERM_EXECUTABLE");
            std::env::remove_var("WEZTERM_UNIX_SOCKET");
            std::env::remove_var("KITTY_WINDOW_ID");
            std::env::remove_var("KITTY_PID");
            std::env::remove_var("GHOSTTY_RESOURCES_DIR");
            std::env::remove_var("ALACRITTY_LOG");
            std::env::remove_var("ALACRITTY_WINDOW_ID");
            std::env::remove_var("ALACRITTY_SOCKET");
            std::env::remove_var("FOOT_SERVER_SOCKET");
            std::env::remove_var("ITERM_SESSION_ID");
            std::env::remove_var("MINTTY_SHORTCUT");
            std::env::remove_var("VTE_VERSION");
            std::env::remove_var("TILIX_ID");
            std::env::remove_var("TERMINATOR_UUID");
            std::env::remove_var("KONSOLE_VERSION");
            std::env::remove_var("KONSOLE_DBUS_SERVICE");
            std::env::remove_var("RXVT_SOCKET");
            std::env::remove_var("XTERM_VERSION");
            std::env::remove_var("TERM_PROGRAM");
            std::env::remove_var("TERM");
        };

        clear_vars();
        std::env::set_var("WT_SESSION", "9876-uuid");
        assert_eq!(
            detect_terminal_emulator(),
            TerminalEmulator::WindowsTerminal
        );

        clear_vars();
        std::env::set_var("WEZTERM_PANE", "3");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::WezTerm);

        clear_vars();
        std::env::set_var("KITTY_WINDOW_ID", "1");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Kitty);

        clear_vars();
        std::env::set_var("GHOSTTY_RESOURCES_DIR", "/usr/share/ghostty");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Ghostty);

        clear_vars();
        std::env::set_var("ALACRITTY_LOG", "/tmp/alacritty.log");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Alacritty);

        clear_vars();
        std::env::set_var("FOOT_SERVER_SOCKET", "/run/user/1000/foot.sock");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Foot);

        clear_vars();
        std::env::set_var("ITERM_SESSION_ID", "w0t0p0:...");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::ITerm2);

        clear_vars();
        std::env::set_var("VTE_VERSION", "7600");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Vte);

        clear_vars();
        std::env::set_var("KONSOLE_VERSION", "230800");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Konsole);

        clear_vars();
        std::env::set_var("TERM", "linux");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::LinuxConsole);

        clear_vars();
        std::env::set_var("TERM", "dumb");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::Dumb);

        clear_vars();
        std::env::set_var("TERM", "vt100");
        assert_eq!(detect_terminal_emulator(), TerminalEmulator::SerialConsole);

        clear_vars();
    }
}
