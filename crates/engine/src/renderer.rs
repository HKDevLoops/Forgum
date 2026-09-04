//! Renderer trait and backends for terminal output.
//!
//! The `Renderer` trait abstracts how framebuffer damage is written to the
//! terminal. The default `AnsiRenderer` writes ANSI escape sequences directly.
//! `TmuxPassthroughRenderer` wraps output in tmux DCS passthrough sequences.

use std::io::Write;

use crate::framebuffer::{Cell, FrameBuffer};

/// Trait for rendering framebuffer damage to a terminal.
pub trait Renderer: Send {
    /// Write the given damage cells to the output.
    /// `cells` is the full frame buffer, `cols` the width, `damage` the changed positions.
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        cells: &[Cell],
        cols: usize,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()>;

    /// Convenience: render from a FrameBuffer (reads its back buffer for cells).
    fn render_fb(
        &mut self,
        out: &mut dyn Write,
        fb: &FrameBuffer,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        self.render_damage(out, &fb.back, fb.cols(), damage)
    }

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
    is_banner: bool,
}

impl AnsiRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_banner() -> Self {
        Self {
            scratch: Vec::with_capacity(1024),
            is_banner: true,
        }
    }

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
        cells: &[Cell],
        cols: usize,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        if damage.is_empty() {
            return Ok(());
        }
        let buf = &mut self.scratch;
        buf.clear();

        if self.is_banner {
            buf.extend_from_slice(b"\x1b8");
        }

        let mut cur_y = 0;
        let mut i = 0;
        while i < damage.len() {
            let (x0, y0) = damage[i];
            let idx = y0 * cols + x0;
            let cell0 = cells.get(idx).copied().unwrap_or_default();

            if self.is_banner {
                if y0 > cur_y {
                    buf.extend_from_slice(b"\x1b[");
                    Self::write_decimal(buf, (y0 - cur_y) as u32);
                    buf.push(b'B');
                    cur_y = y0;
                } else if y0 < cur_y {
                    buf.extend_from_slice(b"\x1b8");
                    if y0 > 0 {
                        buf.extend_from_slice(b"\x1b[");
                        Self::write_decimal(buf, y0 as u32);
                        buf.push(b'B');
                    }
                    cur_y = y0;
                }
                buf.extend_from_slice(b"\x1b[");
                Self::write_decimal(buf, (x0 + 1) as u32);
                buf.push(b'G');
            } else {
                buf.extend_from_slice(b"\x1b[");
                Self::write_decimal(buf, (y0 + 1) as u32);
                buf.push(b';');
                Self::write_decimal(buf, (x0 + 1) as u32);
                buf.push(b'H');
            }

            if cell0.alpha == 0 {
                let mut run_len = 1;
                while i + run_len < damage.len() {
                    let (nx, ny) = damage[i + run_len];
                    if ny != y0 || nx != x0 + run_len {
                        break;
                    }
                    let nidx = ny * cols + nx;
                    let nc = cells.get(nidx).copied().unwrap_or_default();
                    if nc.alpha != 0 {
                        break;
                    }
                    run_len += 1;
                }
                for _ in 0..run_len {
                    buf.push(b' ');
                }
                i += run_len;
            } else {
                buf.extend_from_slice(b"\x1b[38;2;");
                Self::write_decimal(buf, u32::from(cell0.fg.r));
                buf.push(b';');
                Self::write_decimal(buf, u32::from(cell0.fg.g));
                buf.push(b';');
                Self::write_decimal(buf, u32::from(cell0.fg.b));
                buf.push(b'm');

                let mut ch_buf = [0u8; 4];
                let s = cell0.ch.encode_utf8(&mut ch_buf);
                buf.extend_from_slice(s.as_bytes());

                let mut run_len = 1;
                while i + run_len < damage.len() {
                    let (nx, ny) = damage[i + run_len];
                    if ny != y0 || nx != x0 + run_len {
                        break;
                    }
                    let nidx = ny * cols + nx;
                    let nc = cells.get(nidx).copied().unwrap_or_default();
                    if nc.alpha == 0 || nc.fg != cell0.fg {
                        break;
                    }
                    let s = nc.ch.encode_utf8(&mut ch_buf);
                    buf.extend_from_slice(s.as_bytes());
                    run_len += 1;
                }
                i += run_len;
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
#[derive(Debug, Default)]
pub struct TmuxPassthroughRenderer {
    inner: AnsiRenderer,
}

impl TmuxPassthroughRenderer {
    pub fn new(inner: Box<AnsiRenderer>) -> Self {
        Self { inner: *inner }
    }

    pub fn new_banner() -> Self {
        Self {
            inner: AnsiRenderer::new_banner(),
        }
    }
}

impl Renderer for TmuxPassthroughRenderer {
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        cells: &[Cell],
        cols: usize,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        out.write_all(b"\x1bPtmux;\r")?;
        self.inner.render_damage(out, cells, cols, damage)?;
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

/// Phase 4.3: Synchronized ANSI renderer — wraps each frame in
/// DEC 2026 BeginSynchronizedUpdate / EndSynchronizedUpdate.
/// The terminal holds the previous frame until the new one is complete,
/// eliminating partial repaints (tearing).
#[derive(Debug, Default)]
pub struct SyncAnsiRenderer {
    inner: AnsiRenderer,
}

impl SyncAnsiRenderer {
    pub fn new_banner() -> Self {
        Self {
            inner: AnsiRenderer::new_banner(),
        }
    }
}

impl Renderer for SyncAnsiRenderer {
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        cells: &[Cell],
        cols: usize,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        if damage.is_empty() {
            return Ok(());
        }
        out.write_all(self.begin_sync().as_bytes())?;
        self.inner.render_damage(out, cells, cols, damage)?;
        out.write_all(self.end_sync().as_bytes())?;
        out.flush()
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

#[allow(missing_debug_implementations)]
pub struct PlatformRendererAdapter {
    inner: Box<dyn forgum_platform::GraphicsRenderer>,
    cached_fb: Option<FrameBuffer>,
}

impl PlatformRendererAdapter {
    pub fn new(inner: Box<dyn forgum_platform::GraphicsRenderer>) -> Self {
        Self {
            inner,
            cached_fb: None,
        }
    }
}

impl Renderer for PlatformRendererAdapter {
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        cells: &[Cell],
        cols: usize,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()> {
        let rows = cells.len().checked_div(cols).unwrap_or(0);
        match &mut self.cached_fb {
            Some(fb) if fb.width == cols && fb.height == rows => {
                fb.back.copy_from_slice(cells);
                self.inner.render_damage(out, fb, damage)
            }
            _ => {
                let fb = FrameBuffer::from_raw(cols, rows, cells);
                self.inner.render_damage(out, &fb, damage)
            }
        }
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
        let c = self.get_back(x, y);
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
/// 3. `SyncAnsiRenderer` when terminal supports DEC 2026.
/// 4. The default `AnsiRenderer`.
///
/// The default build keeps path 4 unchanged; capability detection is
/// best-effort and never regresses ANSI.
#[must_use]
pub fn create_renderer() -> Box<dyn Renderer> {
    if let Some(graphics) = forgum_platform::create_graphics_renderer() {
        return Box::new(PlatformRendererAdapter::new(graphics));
    }
    if is_tmux() {
        return Box::new(TmuxPassthroughRenderer::default());
    }
    // Phase 4.7: prefer SyncAnsiRenderer when terminal supports DEC 2026.
    if forgum_platform::terminal_supports_sync() {
        return Box::new(SyncAnsiRenderer::default());
    }
    Box::new(AnsiRenderer::default())
}

/// Create a banner renderer for rendering inline above the prompt without alternate screen.
#[must_use]
pub fn create_banner_renderer() -> Box<dyn Renderer> {
    if is_tmux() {
        return Box::new(TmuxPassthroughRenderer::new_banner());
    }
    if forgum_platform::terminal_supports_sync() {
        return Box::new(SyncAnsiRenderer::new_banner());
    }
    Box::new(AnsiRenderer::new_banner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::{Cell as FbCell, Color, FrameBuffer};

    #[test]
    fn ansi_renderer_writes_move_sequence() {
        let mut fb = FrameBuffer::new(10, 5);
        let _ = fb.set(3, 2, FbCell::new('X', Color::WHITE));

        let mut out = Vec::new();
        let mut renderer = AnsiRenderer::default();
        let damage = vec![(3, 2)];
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(
            s.contains("\x1b[3;4H"),
            "Expected cursor move to row 3 col 4: {s}"
        );
        assert!(s.contains("X"), "Expected character X: {s}");
    }

    #[test]
    fn ansi_renderer_reads_provided_cells() {
        let fb = FrameBuffer::new(10, 5);
        let cells = fb.back.clone();
        let mut out = Vec::new();
        let mut renderer = AnsiRenderer::default();
        renderer.render_damage(&mut out, &cells, 10, &[]).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn ansi_renderer_empty_damage_is_noop() {
        let fb = FrameBuffer::new(10, 5);
        let mut out = Vec::new();
        let mut renderer = AnsiRenderer::default();
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &[])
            .unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn ansi_renderer_banner_mode_uses_relative_cursor_moves() {
        let mut fb = FrameBuffer::new(20, 10);
        let _ = fb.set(5, 2, FbCell::new('B', Color::WHITE));

        let mut out = Vec::new();
        let mut renderer = AnsiRenderer::new_banner();
        let damage = vec![(5, 2)];
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s = String::from_utf8(out).unwrap();
        // Must restore cursor to saved banner origin (\x1b8)
        assert!(s.starts_with("\x1b8"), "Banner render must start with DECRC: {s}");
        // Must move down 2 lines (\x1b[2B)
        assert!(s.contains("\x1b[2B"), "Banner render must move down relative lines: {s}");
        // Must move to column 6 (\x1b[6G)
        assert!(s.contains("\x1b[6G"), "Banner render must move horizontally: {s}");
        // Must NOT contain absolute row cursor move like \x1b[3;6H
        assert!(!s.contains("\x1b[3;6H"), "Banner mode must not use absolute coordinates: {s}");
        assert!(s.contains("B"), "Banner render must output character: {s}");
    }

    #[test]
    fn tmux_renderer_wraps_in_dcs() {
        let fb = FrameBuffer::new(10, 5);
        let mut out = Vec::new();
        let mut renderer = TmuxPassthroughRenderer::default();
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &[])
            .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("\x1bPtmux;"), "Expected tmux DCS start: {s}");
        assert!(s.contains("\x1b\\"), "Expected tmux DCS end: {s}");
    }

    #[test]
    fn is_tmux_returns_false_without_env() {
        let _ = is_tmux();
    }

    #[test]
    fn create_renderer_returns_ansi_by_default() {
        let _r = create_renderer();
    }

    #[test]
    fn sync_ansi_renderer_wraps_in_synchronized_update() {
        let mut fb = FrameBuffer::new(10, 5);
        let _ = fb.set(0, 0, FbCell::new('X', Color::WHITE));

        let mut out = Vec::new();
        let mut renderer = SyncAnsiRenderer::default();
        let damage = vec![(0, 0)];
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(
            s.starts_with("\x1b[?2026h"),
            "Expected BeginSynchronizedUpdate: {s}"
        );
        assert!(
            s.ends_with("\x1b[?2026l"),
            "Expected EndSynchronizedUpdate: {s}"
        );
        assert!(s.contains('X'), "Expected character X: {s}");
    }

    #[test]
    fn sync_ansi_renderer_empty_damage_is_noop() {
        let fb = FrameBuffer::new(10, 5);
        let mut out = Vec::new();
        let mut renderer = SyncAnsiRenderer::default();
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &[])
            .unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn coalesced_run_renders_consecutive_same_color_cells() {
        let mut fb = FrameBuffer::new(10, 3);
        let c = FbCell::new('A', Color::WHITE);
        let _ = fb.set(0, 0, c);
        let _ = fb.set(1, 0, c);
        let _ = fb.set(2, 0, c);

        let mut out = Vec::new();
        let mut renderer = AnsiRenderer::default();
        let damage = vec![(0, 0), (1, 0), (2, 0)];
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s = String::from_utf8(out).unwrap();

        assert_eq!(
            s.matches("\x1b[1;1H").count(),
            1,
            "Expected exactly 1 MoveTo for coalesced run: {s}"
        );
        assert!(s.contains("AAA"), "Expected coalesced 'AAA': {s}");
    }

    #[test]
    fn coalesced_run_breaks_on_color_change() {
        let mut fb = FrameBuffer::new(10, 3);
        let red = FbCell::new('R', Color::rgb(255, 0, 0));
        let blue = FbCell::new('B', Color::rgb(0, 0, 255));
        let _ = fb.set(0, 0, red);
        let _ = fb.set(1, 0, blue);
        let _ = fb.set(2, 0, red);

        let mut out = Vec::new();
        let mut renderer = AnsiRenderer::default();
        let damage = vec![(0, 0), (1, 0), (2, 0)];
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s = String::from_utf8(out).unwrap();

        let color_count = s.matches("\x1b[38;2;").count();
        assert!(
            color_count >= 3,
            "Expected at least 3 color sequences for alternating colors: {s}"
        );
    }

    #[test]
    fn tmux_renderer_reuses_inner_ansi_renderer() {
        let mut renderer = TmuxPassthroughRenderer::default();
        let fb = FrameBuffer::new(10, 3);
        let damage = vec![(0, 0)];
        let mut out1 = Vec::new();
        let mut out2 = Vec::new();
        renderer
            .render_damage(&mut out1, &fb.back, fb.cols(), &damage)
            .unwrap();
        renderer
            .render_damage(&mut out2, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s1 = String::from_utf8(out1).unwrap();
        let s2 = String::from_utf8(out2).unwrap();
        assert!(s1.starts_with("\x1bPtmux;"), "first call wraps DCS: {s1}");
        assert!(s2.starts_with("\x1bPtmux;"), "second call wraps DCS: {s2}");
        assert_eq!(s1, s2, "repeated renders must produce identical output");
    }

    #[test]
    fn tmux_renderer_dcswraps_correctly() {
        let mut renderer = TmuxPassthroughRenderer::default();
        let mut fb = FrameBuffer::new(10, 3);
        let c = FbCell::new('Z', Color::WHITE);
        let _ = fb.set(0, 0, c);
        let damage = vec![(0, 0)];
        let mut out = Vec::new();
        renderer
            .render_damage(&mut out, &fb.back, fb.cols(), &damage)
            .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("\x1bPtmux;"), "must contain DCS start: {s}");
        assert!(s.contains("\x1b\\"), "must contain DCS end (ST): {s}");
        assert!(s.contains('Z'), "must contain rendered character: {s}");
    }
}
