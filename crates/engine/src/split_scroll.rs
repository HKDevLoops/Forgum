//! Dynamic geometry, terminal area reservation, and DECSTBM split-scroll margin management.
//!
//! Provides dynamic row and column reservation based on mascot anatomy, viewport dimensions,
//! and prompt headroom invariants.

use std::io::Write;

use crate::protocol::SceneConfig;

/// Format the DECSTBM escape sequence to set scroll margins between `top` and `bottom` (1-indexed).
#[inline]
#[must_use]
pub fn format_decstbm(top: usize, bottom: usize) -> String {
    format!("\x1b[{top};{bottom}r")
}

/// Format the DECSTBM reset sequence to restore full-screen scroll margins.
#[inline]
#[must_use]
pub const fn format_reset_decstbm() -> &'static str {
    "\x1b[r"
}

/// Format the line clear sequence with cursor save/restore.
#[inline]
#[must_use]
pub fn format_clear_line(row: usize) -> String {
    format!("\x1b7\x1b[{row};1H\x1b[2K\x1b8")
}

/// Emit DECSTBM scroll margins to `out` with cursor preservation.
pub fn set_decstbm_margins<W: Write>(
    out: &mut W,
    top: usize,
    bottom: usize,
) -> std::io::Result<()> {
    write!(out, "\x1b7\x1b[{top};{bottom}r\x1b8")?;
    out.flush()
}

/// Reset DECSTBM scroll margins to full screen.
pub fn reset_decstbm_margins<W: Write>(out: &mut W) -> std::io::Result<()> {
    write!(out, "\x1b[r")?;
    out.flush()
}

/// Compute reserved rows and columns based on terminal resolution,
/// mascot height, mascot width, config overrides, and dynamic reservation sizing.
pub fn compute_reserved_dimensions_with_cols(
    total_cols: usize,
    total_rows: usize,
    mascot_lines: usize,
    mascot_cols: usize,
    config: &SceneConfig,
) -> (usize, usize) {
    let bounded_split = config.split_mode.as_deref() == Some("bounded");
    let cols = if let Some(rc) = config.reserve_cols {
        (rc as usize).min(total_cols).max(20)
    } else if bounded_split && mascot_cols > 0 {
        // Clamped to mascot width + padding when bounded split is desired
        (mascot_cols + 4).clamp(20, total_cols.max(20))
    } else {
        total_cols.max(20)
    };

    let raw_rows = if let Some(rr) = config.reserve_rows {
        rr as usize
    } else if let Some(ratio) = config.split_ratio {
        let clamped_ratio = ratio.clamp(0.10, 0.75);
        ((total_rows as f32) * clamped_ratio).round() as usize
    } else {
        // Automatic dynamic row reservation: allocate full height required by mascot lines
        mascot_lines
    };

    // Guarantee user gets at least 4 prompt rows in unreserved area if total_rows allows
    let max_safe_rows = total_rows.saturating_sub(4).max(1);
    let rows = raw_rows.min(max_safe_rows).max(1);

    (cols, rows)
}

/// Compute reserved rows and columns based on terminal resolution,
/// mascot height, config overrides, and dynamic reservation sizing.
pub fn compute_reserved_dimensions(
    total_cols: usize,
    total_rows: usize,
    mascot_lines: usize,
    config: &SceneConfig,
) -> (usize, usize) {
    compute_reserved_dimensions_with_cols(total_cols, total_rows, mascot_lines, 0, config)
}

/// Lifecycle state of the split-overlay guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOverlayState {
    Active,
    Suspended,
}

/// Guard managing DECSTBM split-overlay margin lifecycle and background editor isolation.
///
/// Invariants:
/// 1. When an interactive editor or command (nvim, vim, emacs, nano, etc.) takes the
///    foreground in the terminal, DECSTBM margins are immediately reset to full screen
///    (`\x1b[r`) and split overlay rendering is suspended (0 bytes written to tty).
/// 2. When the parent shell returns to the foreground, DECSTBM margins are re-asserted,
///    the reserved banner area is cleared, and a full frame redraw is signaled.
#[derive(Debug)]
pub struct SplitOverlayGuard {
    parent_pid: Option<u32>,
    top_margin: usize,
    bottom_margin: usize,
    overlay_rows: usize,
    pub state: SplitOverlayState,
    last_check: std::time::Instant,
    just_resumed: bool,
    check_interval: std::time::Duration,
}

impl SplitOverlayGuard {
    pub fn new(
        parent_pid: Option<u32>,
        top_margin: usize,
        bottom_margin: usize,
        overlay_rows: usize,
    ) -> Self {
        Self {
            parent_pid,
            top_margin,
            bottom_margin,
            overlay_rows,
            state: SplitOverlayState::Active,
            last_check: std::time::Instant::now() - std::time::Duration::from_millis(100),
            just_resumed: false,
            check_interval: std::time::Duration::from_millis(20),
        }
    }

    pub fn update_dimensions(
        &mut self,
        top_margin: usize,
        bottom_margin: usize,
        overlay_rows: usize,
    ) {
        self.top_margin = top_margin;
        self.bottom_margin = bottom_margin;
        self.overlay_rows = overlay_rows;
    }

    /// Force the next `should_render` call to immediately re-evaluate the foreground process check.
    pub fn force_check(&mut self) {
        self.last_check = std::time::Instant::now() - self.check_interval;
    }

    /// Check if the split overlay should write output to the terminal.
    ///
    /// Returns `true` if overlay rendering should proceed.
    /// Returns `false` if overlay rendering is suspended due to an active editor/command.
    pub fn should_render<W: Write>(&mut self, out: &mut W) -> bool {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_check) >= self.check_interval {
            self.last_check = now;
            let is_fg = forgum_platform::is_parent_shell_in_foreground(self.parent_pid);

            match self.state {
                SplitOverlayState::Active => {
                    if !is_fg {
                        // Shell lost foreground to an interactive editor/command!
                        // Unconditionally reset DECSTBM margins so the editor has full screen.
                        let _ = out.write_all(b"\x1b[r\x1b[?6l\x1b[?69l\x1b[0m\x1b[?25h");
                        let _ = out.flush();
                        self.state = SplitOverlayState::Suspended;
                        self.just_resumed = false;
                        return false;
                    }
                }
                SplitOverlayState::Suspended => {
                    if is_fg {
                        // Shell returned to foreground!
                        // Re-assert DECSTBM margins with cursor save/restore.
                        let _ = out.write_all(
                            format!(
                                "\x1b7\x1b[?6l\x1b[{};{}r\x1b8",
                                self.top_margin, self.bottom_margin
                            )
                            .as_bytes(),
                        );
                        // Clear the reserved banner area including separator row (VIS-006)
                        let clear_limit = self.top_margin.saturating_sub(1).max(self.overlay_rows);
                        for y in 1..=clear_limit {
                            let _ =
                                out.write_all(format!("\x1b7\x1b[{y};1H\x1b[2K\x1b8").as_bytes());
                        }
                        let _ = out.flush();
                        self.state = SplitOverlayState::Active;
                        self.just_resumed = true;
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        }

        self.state == SplitOverlayState::Active
    }

    /// Returns `true` once if the guard just transitioned from `Suspended` to `Active`,
    /// indicating that the caller should perform a full damage repaint.
    pub fn take_just_resumed(&mut self) -> bool {
        let r = self.just_resumed;
        self.just_resumed = false;
        r
    }

    pub fn is_suspended(&self) -> bool {
        self.state == SplitOverlayState::Suspended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decstbm_formatting_sequences() {
        assert_eq!(format_decstbm(11, 40), "\x1b[11;40r");
        assert_eq!(format_reset_decstbm(), "\x1b[r");
        assert_eq!(format_clear_line(12), "\x1b7\x1b[12;1H\x1b[2K\x1b8");
    }

    #[test]
    fn compute_reserved_dimensions_basic() {
        let config = SceneConfig::default();
        let (cols, rows) = compute_reserved_dimensions(80, 24, 10, &config);
        assert_eq!(cols, 80);
        assert_eq!(rows, 10);
    }

    #[test]
    fn compute_reserved_dimensions_small_panes_30x30_and_15x15() {
        let config = SceneConfig::default();
        // 30x30 pane
        let (cols30, rows30) = compute_reserved_dimensions(30, 30, 8, &config);
        assert_eq!(cols30, 30);
        assert_eq!(rows30, 8);

        // 15x15 micro pane with 6 mascot lines (minimum boundary clamped to 20 columns)
        let (cols15, rows15) = compute_reserved_dimensions(15, 15, 6, &config);
        assert_eq!(cols15, 20);
        assert_eq!(rows15, 6);
    }
}
