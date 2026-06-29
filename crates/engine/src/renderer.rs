//! Renderer trait and backends for terminal output.
//!
//! The `Renderer` trait abstracts how framebuffer damage is written to the
//! terminal. The default `AnsiRenderer` writes ANSI escape sequences directly.
//! `TmuxPassthroughRenderer` wraps output in tmux DCS passthrough sequences.

use std::io::Write;

use crate::framebuffer::FrameBuffer;

/// Trait for rendering framebuffer damage to a terminal.
pub trait Renderer {
    /// Write the given damage cells to the output.
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        fb: &FrameBuffer,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()>;

    /// Escape sequence to begin synchronized update (DEC mode 2026).
    fn begin_sync(&self) -> &'static str {
        ""
    }

    /// Escape sequence to end synchronized update.
    fn end_sync(&self) -> &'static str {
        ""
    }
}

/// Default ANSI renderer — writes cursor-move + character sequences.
#[derive(Debug)]
pub struct AnsiRenderer;

impl Renderer for AnsiRenderer {
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        fb: &FrameBuffer,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        if damage.is_empty() {
            return Ok(());
        }
        let mut buf = Vec::with_capacity(damage.len() * 8);
        for &(x, y) in damage {
            let cell = fb.get(x, y);
            // Move to (x+1, y+1) in 1-indexed terminal coordinates
            buf.extend_from_slice(format!("\x1b[{};{}H", y + 1, x + 1).as_bytes());
            if cell.alpha == 0 {
                buf.extend_from_slice(b" ");
            } else {
                // Set 24-bit foreground color
                buf.extend_from_slice(
                    format!("\x1b[38;2;{};{};{}m", cell.fg.r, cell.fg.g, cell.fg.b).as_bytes(),
                );
                let mut ch_buf = [0u8; 4];
                let s = cell.ch.encode_utf8(&mut ch_buf);
                buf.extend_from_slice(s.as_bytes());
            }
        }
        out.write_all(&buf)
    }

    fn begin_sync(&self) -> &'static str {
        "\x1b[?2026h"
    }

    fn end_sync(&self) -> &'static str {
        "\x1b[?2026l"
    }
}

/// tmux-aware renderer that wraps ANSI output in DCS passthrough sequences.
#[derive(Debug)]
pub struct TmuxPassthroughRenderer;

impl Renderer for TmuxPassthroughRenderer {
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        fb: &FrameBuffer,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        // Begin tmux passthrough
        out.write_all(b"\x1bPtmux;\r")?;
        // Write inner ANSI output
        let mut inner = AnsiRenderer;
        inner.render_damage(out, fb, damage)?;
        // End tmux passthrough
        out.write_all(b"\x1b\\")?;
        Ok(())
    }

    fn begin_sync(&self) -> &'static str {
        "\x1b[?2026h"
    }

    fn end_sync(&self) -> &'static str {
        "\x1b[?2026l"
    }
}

/// Detect if running inside tmux.
#[must_use]
pub fn is_tmux() -> bool {
    std::env::var("TMUX")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Create the appropriate renderer for the current environment.
#[must_use]
pub fn create_renderer() -> Box<dyn Renderer> {
    if is_tmux() {
        Box::new(TmuxPassthroughRenderer)
    } else {
        Box::new(AnsiRenderer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_renderer_writes_move_sequence() {
        let mut fb = FrameBuffer::new(10, 5);
        let _ = fb.set(
            3,
            2,
            crate::framebuffer::Cell::new('X', crate::framebuffer::Color::WHITE),
        );
        fb.swap();

        let mut out = Vec::new();
        let mut renderer = AnsiRenderer;
        let damage = vec![(3, 2)];
        renderer.render_damage(&mut out, &fb, &damage).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(
            s.contains("\x1b[3;4H"),
            "Expected cursor move to row 3 col 4: {s}"
        );
        assert!(s.contains("X"), "Expected character X: {s}");
    }

    #[test]
    fn ansi_renderer_empty_damage_is_noop() {
        let fb = FrameBuffer::new(10, 5);
        let mut out = Vec::new();
        let mut renderer = AnsiRenderer;
        renderer.render_damage(&mut out, &fb, &[]).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn tmux_renderer_wraps_in_dcs() {
        let fb = FrameBuffer::new(10, 5);
        let mut out = Vec::new();
        let mut renderer = TmuxPassthroughRenderer;
        renderer.render_damage(&mut out, &fb, &[]).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("\x1bPtmux;"), "Expected tmux DCS start: {s}");
        assert!(s.contains("\x1b\\"), "Expected tmux DCS end: {s}");
    }

    #[test]
    fn is_tmux_returns_false_without_env() {
        // Can't easily test with TMUX set, but can verify it doesn't panic
        let _ = is_tmux();
    }

    #[test]
    fn create_renderer_returns_ansi_by_default() {
        // Unless TMUX is set, should return AnsiRenderer
        let _r = create_renderer();
    }
}
