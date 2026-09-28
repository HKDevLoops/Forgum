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
pub fn set_decstbm_margins<W: Write>(out: &mut W, top: usize, bottom: usize) -> std::io::Result<()> {
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
        (mascot_cols + 4).min(total_cols).max(20)
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
}
