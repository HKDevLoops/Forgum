use std::sync::OnceLock;

/// Detected terminal multiplexer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mux {
    None,
    Tmux { pane: String, session: String },
    Zellij { tab: String },
    Screen { window: String },
    WezTerm { tab: String },
}

impl Mux {
    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Tmux { .. } => "tmux",
            Self::Zellij { .. } => "zellij",
            Self::Screen { .. } => "screen",
            Self::WezTerm { .. } => "wezterm",
        }
    }
}

static MUX: OnceLock<Mux> = OnceLock::new();

#[must_use]
pub fn detect_mux() -> Mux {
    MUX.get_or_init(detect_mux_inner).clone()
}

#[must_use]
fn detect_mux_inner() -> Mux {
    if let Ok(tmux) = std::env::var("TMUX") {
        let session = parse_tmux_session(&tmux);
        let pane = std::env::var("TMUX_PANE").unwrap_or_default();
        return Mux::Tmux { pane, session };
    }
    if std::env::var("ZELLIJ").is_ok() {
        let tab = std::env::var("ZELLIJ_TAB_NAME").unwrap_or_default();
        return Mux::Zellij { tab };
    }
    if std::env::var("STY").is_ok() {
        let window = std::env::var("WINDOW").unwrap_or_default();
        return Mux::Screen { window };
    }
    if std::env::var("WEZTERM_UNIX_SOCKET").is_ok() {
        let tab = std::env::var("WEZTERM_TAB_ID").unwrap_or_default();
        return Mux::WezTerm { tab };
    }
    Mux::None
}

#[must_use]
fn parse_tmux_session(value: &str) -> String {
    if let Some(pos) = value.find(',') {
        let rest = &value[pos + 1..];
        if let Some(end) = rest.find(',') {
            let id = &rest[..end];
            if !id.is_empty() {
                return id.to_string();
            }
        }
    }
    "default".to_string()
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;

    fn clear_all_env() {
        unsafe {
            std::env::remove_var("TMUX");
            std::env::remove_var("TMUX_PANE");
            std::env::remove_var("ZELLIJ");
            std::env::remove_var("ZELLIJ_TAB_NAME");
            std::env::remove_var("STY");
            std::env::remove_var("WINDOW");
            std::env::remove_var("WEZTERM_UNIX_SOCKET");
            std::env::remove_var("WEZTERM_TAB_ID");
        }
    }

    #[test]
    fn detect_env_var_muxes() {
        clear_all_env();
        assert_eq!(detect_mux_inner(), Mux::None);

        unsafe {
            std::env::set_var("TMUX", "/tmp/tmux-1000/default,1234,0");
            std::env::set_var("TMUX_PANE", "3");
        }
        assert_eq!(
            detect_mux_inner(),
            Mux::Tmux {
                pane: "3".to_string(),
                session: "1234".to_string(),
            }
        );

        clear_all_env();
        unsafe {
            std::env::set_var("ZELLIJ", "1");
            std::env::set_var("ZELLIJ_TAB_NAME", "main");
        }
        assert_eq!(
            detect_mux_inner(),
            Mux::Zellij {
                tab: "main".to_string()
            }
        );

        clear_all_env();
        unsafe {
            std::env::set_var("STY", "1234.pts-0.host");
            std::env::set_var("WINDOW", "2");
        }
        assert_eq!(
            detect_mux_inner(),
            Mux::Screen {
                window: "2".to_string()
            }
        );

        clear_all_env();
        unsafe {
            std::env::set_var("WEZTERM_UNIX_SOCKET", "/tmp/wezterm.sock");
            std::env::set_var("WEZTERM_TAB_ID", "5");
        }
        assert_eq!(
            detect_mux_inner(),
            Mux::WezTerm {
                tab: "5".to_string()
            }
        );

        clear_all_env();
        unsafe {
            std::env::set_var("TMUX", "/tmp/tmux-1000/default,1234,0");
        }
        assert_eq!(
            detect_mux_inner(),
            Mux::Tmux {
                pane: String::new(),
                session: "1234".to_string(),
            }
        );

        clear_all_env();
    }

    #[test]
    fn is_active() {
        assert!(!Mux::None.is_active());
        assert!(Mux::Tmux {
            pane: String::new(),
            session: String::new()
        }
        .is_active());
        assert!(Mux::Zellij { tab: String::new() }.is_active());
        assert!(Mux::Screen {
            window: String::new()
        }
        .is_active());
        assert!(Mux::WezTerm { tab: String::new() }.is_active());
    }

    #[test]
    fn name() {
        assert_eq!(Mux::None.name(), "none");
        assert_eq!(
            Mux::Tmux {
                pane: String::new(),
                session: String::new()
            }
            .name(),
            "tmux"
        );
        assert_eq!(Mux::Zellij { tab: String::new() }.name(), "zellij");
        assert_eq!(
            Mux::Screen {
                window: String::new()
            }
            .name(),
            "screen"
        );
        assert_eq!(Mux::WezTerm { tab: String::new() }.name(), "wezterm");
    }

    #[test]
    fn tmux_parse_session_default() {
        assert_eq!(parse_tmux_session("bad"), "default");
        assert_eq!(parse_tmux_session("/tmp/tmux-1000/default,"), "default");
        assert_eq!(parse_tmux_session("/tmp/tmux-1000/default,1234"), "default");
    }

    #[test]
    fn tmux_parse_session_valid() {
        assert_eq!(parse_tmux_session("/tmp/tmux-1000/default,1234,0"), "1234");
    }
}
