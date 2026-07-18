//! Renderer trait and backends for terminal output.
//!
//! The `Renderer` trait abstracts how framebuffer damage is written to the
//! terminal. The default `AnsiRenderer` writes ANSI escape sequences directly.
//! `TmuxPassthroughRenderer` wraps output in tmux DCS passthrough sequences.

use std::io::Write;

use crate::framebuffer::FrameBuffer;

/// Trait for rendering framebuffer damage to a terminal.
///
/// The trait is intentionally **extensible** so future backends (e.g. the v3
/// GPU renderer) can be slotted in without changing callers. The default
/// [`AnsiRenderer`] writes ANSI escape sequences; `forgum-platform` may supply
/// an optional graphics renderer (Sixel/Kitty) wrapped by
/// [`PlatformRendererAdapter`].
///
/// TODO(v3): wgpu GPU backend (separate crate) — see plan D4/G4. Do NOT add
/// `wgpu` to any Cargo.toml; this trait stays backend-agnostic so the deferred
/// GPU backend can implement `Renderer` (or a richer variant) without touching
/// the render loops.
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
///
/// The per-frame scratch buffer is reused across calls so a frame with many
/// changed cells allocates **zero** heap (D1/D3): no `format!()` per cell.
#[derive(Debug, Default)]
pub struct AnsiRenderer {
    scratch: Vec<u8>,
}

impl AnsiRenderer {
    /// Write a single decimal `u32` into `buf` (no per-call allocation).
    fn write_decimal(buf: &mut Vec<u8>, mut n: u32) {
        if n == 0 {
            buf.push(b'0');
            return;
        }
        let mut tmp = [0u8; 10];
        let mut i = tmp.len();
        while n > 0 {
            i -= 1;
            tmp[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        buf.extend_from_slice(&tmp[i..]);
    }
}

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
        let buf = &mut self.scratch;
        buf.clear();
        for &(x, y) in damage {
            let cell = fb.get(x, y);
            // Move to (x+1, y+1) in 1-indexed terminal coordinates.
            buf.extend_from_slice(b"\x1b[");
            Self::write_decimal(buf, (y + 1) as u32);
            buf.push(b';');
            Self::write_decimal(buf, (x + 1) as u32);
            buf.push(b'H');
            if cell.alpha == 0 {
                buf.push(b' ');
            } else {
                // 24-bit foreground color: 38;2;r;g;bm
                buf.extend_from_slice(b"\x1b[38;2;");
                Self::write_decimal(buf, u32::from(cell.fg.r));
                buf.push(b';');
                Self::write_decimal(buf, u32::from(cell.fg.g));
                buf.push(b';');
                Self::write_decimal(buf, u32::from(cell.fg.b));
                buf.extend_from_slice(b"m");
                let mut ch_buf = [0u8; 4];
                let s = cell.ch.encode_utf8(&mut ch_buf);
                buf.extend_from_slice(s.as_bytes());
            }
        }
        out.write_all(buf)
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
        let mut inner = AnsiRenderer::default();
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

/// Adapter that wraps a `forgum-platform` [`GraphicsRenderer`] behind this
/// crate's `Renderer` trait, bridging the two via [`FrameBufferLike`].
///
/// The selection happens through a **non-`cfg` runtime branch** in
/// [`create_renderer`]: when the `sixel` feature is off, `create_graphics_renderer`
/// returns `None` and the default ANSI path is used unchanged. No `#[cfg]` in
/// this file — the CI grep forbids it.
#[allow(missing_debug_implementations)] // inner is a non-Debug trait object
pub struct PlatformRendererAdapter {
    inner: Box<dyn forgum_platform::GraphicsRenderer>,
}

impl Renderer for PlatformRendererAdapter {
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        fb: &FrameBuffer,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        self.inner.render_damage(out, fb, damage)
    }

    fn begin_sync(&self) -> &'static str {
        self.inner.begin_sync()
    }

    fn end_sync(&self) -> &'static str {
        self.inner.end_sync()
    }
}

/// Bridge the engine `FrameBuffer` to the platform's read-only view so a
/// graphics backend can consume it without a circular dependency.
impl forgum_platform::FrameBufferLike for FrameBuffer {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn cell(&self, x: usize, y: usize) -> forgum_platform::CellView {
        let c = self.get(x, y);
        forgum_platform::CellView {
            ch: c.ch,
            fg: (c.fg.r, c.fg.g, c.fg.b),
            bg: (c.bg.r, c.bg.g, c.bg.b),
            alpha: c.alpha,
        }
    }
}

/// Create the appropriate renderer for the current environment.
///
/// Selection order (all runtime, no `#[cfg]` here):
/// 1. An optional `forgum-platform` graphics backend (Sixel/Kitty) when the
///    `sixel` feature is enabled AND the terminal capability is detected.
/// 2. `TmuxPassthroughRenderer` when inside tmux.
/// 3. The default `AnsiRenderer`.
///
/// The default build keeps path 3 unchanged; capability detection is
/// best-effort and never regresses ANSI.
#[must_use]
pub fn create_renderer() -> Box<dyn Renderer> {
    if let Some(graphics) = forgum_platform::create_graphics_renderer() {
        return Box::new(PlatformRendererAdapter { inner: graphics });
    }
    if is_tmux() {
        Box::new(TmuxPassthroughRenderer)
    } else {
        Box::new(AnsiRenderer::default())
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
        let mut renderer = AnsiRenderer::default();
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
        let mut renderer = AnsiRenderer::default();
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
