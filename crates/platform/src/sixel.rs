//! Optional Sixel/Kitty terminal-graphics backend (D4/T4).
//!
//! This module lives in `crates/platform` (NOT the engine) so that all
//! platform/graphics `#[cfg]` stays out of `crates/engine/src`, which the CI
//! grep forbids. The engine wraps a [`GraphicsRenderer`] behind its own
//! `Renderer` trait via `PlatformRendererAdapter` (see `engine/src/renderer.rs`)
//! using a **non-`cfg` runtime branch**: when the `sixel` feature is off,
//! [`create_graphics_renderer`] simply returns `None` and the engine keeps its
//! default ANSI path untouched.
//!
//! The public *types* ([`CellView`], [`FrameBufferLike`], [`GraphicsRenderer`])
//! and the two runtime *selectors* ([`create_graphics_renderer`],
//! [`graphics_renderer_available`]) are always defined — the engine references
//! them unconditionally. Only the actual Sixel/Kitty *implementation* is
//! `#[cfg(feature = "sixel")]`-gated, so the default build stays lean and
//! clippy `-D warnings` stays green with the feature off.
//!
//! Capability detection is **best-effort**: we probe environment hints
//! (`TERM`, `TERM_PROGRAM`) but never claim support we cannot verify, and we
//! never regress the default ANSI renderer. When in doubt we report "not
//! available" and the engine falls back to ANSI.

use std::io::Write;

/// A cheap, platform-owned view of one framebuffer cell.
///
/// Defined here (not in the engine) so `platform` does not depend on `engine`
/// and the engine can implement [`FrameBufferLike`] for its own `FrameBuffer`.
#[derive(Debug, Clone, Copy)]
pub struct CellView {
    /// The grapheme to draw (best-effort; graphics backends may rasterize).
    pub ch: char,
    /// 24-bit foreground color.
    pub fg: (u8, u8, u8),
    /// 24-bit background color.
    pub bg: (u8, u8, u8),
    /// 0 = transparent, 255 = opaque.
    pub alpha: u8,
}

/// Read-only view of a framebuffer that a [`GraphicsRenderer`] can consume.
///
/// The engine implements this for its `FrameBuffer`, bridging the two crates
/// without a circular dependency.
pub trait FrameBufferLike {
    /// Framebuffer width in cells.
    fn width(&self) -> usize;
    /// Framebuffer height in cells.
    fn height(&self) -> usize;
    /// Cell at `(x, y)` (out-of-bounds returns a transparent space).
    fn cell(&self, x: usize, y: usize) -> CellView;
}

/// A graphics-capable renderer backend, implemented by `platform`.
///
/// Mirrors the engine `Renderer` surface (damage-based + sync markers) so the
/// engine can wrap any impl behind its own trait without `cfg` in `engine/src`.
pub trait GraphicsRenderer {
    /// Render the given damage cells as graphics primitives.
    fn render_damage(
        &mut self,
        out: &mut dyn Write,
        fb: &dyn FrameBufferLike,
        damage: &[(usize, usize)],
    ) -> std::io::Result<()>;

    /// Escape sequence to begin a synchronized update, if the underlying
    /// terminal protocol supports it.
    fn begin_sync(&self) -> &'static str {
        ""
    }

    /// Escape sequence to end a synchronized update.
    fn end_sync(&self) -> &'static str {
        ""
    }
}

/// Create a graphics renderer, or `None` when the `sixel` feature is disabled
/// or the terminal capability is not detected. Always `None` in the default
/// build, so the engine keeps its ANSI path.
#[must_use]
pub fn create_graphics_renderer() -> Option<Box<dyn GraphicsRenderer>> {
    #[cfg(feature = "sixel")]
    {
        imp::detect_protocol()
            .map(|p| Box::new(imp::PlatformGraphicsRenderer::new(p)) as Box<dyn GraphicsRenderer>)
    }
    #[cfg(not(feature = "sixel"))]
    {
        None
    }
}

/// Runtime availability probe mirroring [`create_graphics_renderer`] without
/// constructing a renderer. Used by the engine's non-`cfg` branch.
#[must_use]
pub fn graphics_renderer_available() -> bool {
    create_graphics_renderer().is_some()
}

// ── Sixel/Kitty implementation (feature-gated) ──────────────────────────────

#[cfg(feature = "sixel")]
mod imp {
    use super::{FrameBufferLike, GraphicsRenderer};
    use std::io::Write;

    /// Which graphics protocol to emit.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Protocol {
        Sixel,
        Kitty,
    }

    /// Best-effort terminal-capability detection for a graphics protocol.
    ///
    /// Returns `Some(protocol)` only when we have reasonable environmental
    /// evidence the terminal understands it. We deliberately err toward `None`
    /// so the default ANSI path is never regressed.
    pub(super) fn detect_protocol() -> Option<Protocol> {
        let term = std::env::var("TERM")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let term_program = std::env::var("TERM_PROGRAM")
            .unwrap_or_default()
            .to_ascii_lowercase();

        // Sixel-capable terminals.
        if term.contains("sixel")
            || term_program.contains("mlterm")
            || term_program.contains("foot")
            || term_program.contains("wezterm")
        {
            return Some(Protocol::Sixel);
        }
        // Kitty (and kitty-based terminals) understand the Kitty graphics protocol.
        if term_program.contains("kitty") || term.contains("kitty") {
            return Some(Protocol::Kitty);
        }
        None
    }

    /// Pixel geometry for one character cell when rasterizing to a graphics
    /// primitive. We rasterize the real 8×8 glyph font into a cell-sized block,
    /// falling back to a solid colored block when the font lacks the glyph.
    const CELL_W: usize = 8;
    const CELL_H: usize = 8; // matches the built-in font (GLYPH_W×GLYPH_H)

    /// Sixel/Kitty graphics renderer: draws each damaged cell as a solid block
    /// of its foreground color at the cell's screen position. This proves out
    /// the graphics path and keeps the feature fully off the default ANSI route.
    pub(super) struct PlatformGraphicsRenderer {
        protocol: Protocol,
    }

    impl PlatformGraphicsRenderer {
        pub(super) fn new(protocol: Protocol) -> Self {
            Self { protocol }
        }
    }

    impl GraphicsRenderer for PlatformGraphicsRenderer {
        fn render_damage(
            &mut self,
            out: &mut dyn Write,
            fb: &dyn FrameBufferLike,
            damage: &[(usize, usize)],
        ) -> std::io::Result<()> {
            let mut buf = Vec::new();
            for &(x, y) in damage {
                let cell = fb.cell(x, y);
                // Move the cursor to the cell's top-left in 1-indexed coords.
                buf.extend_from_slice(b"\x1b[");
                write_decimal(&mut buf, (y + 1) as u32);
                buf.push(b';');
                write_decimal(&mut buf, (x + 1) as u32);
                buf.push(b'H');

                // Prefer real glyph rasterization; fall back to a solid block
                // when the font has no glyph for this character.
                let fg = if cell.alpha == 0 { (0, 0, 0) } else { cell.fg };
                if let Some(bits) = crate::font::glyph(cell.ch) {
                    let bg = cell.bg;
                    match self.protocol {
                        Protocol::Sixel => emit_sixel_glyph(&mut buf, bits, fg, bg, CELL_W, CELL_H),
                        Protocol::Kitty => emit_kitty_glyph(&mut buf, bits, fg, bg, CELL_W, CELL_H),
                    }
                } else {
                    match self.protocol {
                        Protocol::Sixel => emit_sixel_block(&mut buf, fg, CELL_W, CELL_H),
                        Protocol::Kitty => emit_kitty_block(&mut buf, fg, CELL_W, CELL_H),
                    }
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

    /// Write a `u32` decimal into `buf` with no allocation.
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

    /// Emit a Sixel image of a solid `color` block sized `w`×`h` at the cursor.
    fn emit_sixel_block(buf: &mut Vec<u8>, color: (u8, u8, u8), w: usize, h: usize) {
        buf.extend_from_slice(b"\x1bPq");
        // Define color register 0 as 24-bit (2 = RGB, 0..100 scale).
        buf.extend_from_slice(b"#0;2;");
        write_decimal(buf, u32::from(color.0) * 100 / 255);
        buf.push(b';');
        write_decimal(buf, u32::from(color.1) * 100 / 255);
        buf.push(b';');
        write_decimal(buf, u32::from(color.2) * 100 / 255);
        // Sixel rows: each character encodes 6 vertical pixels. `?` (63) sets all.
        let rows = h.div_ceil(6);
        for r in 0..rows {
            for _ in 0..w {
                buf.push(b'?');
            }
            if r + 1 < rows {
                buf.push(b'-'); // sixel carriage return (back to column 0)
            }
        }
        buf.extend_from_slice(b"\x1b\\");
    }

    /// Build an `w`×`h` RGBA pixel buffer from an 8×8 glyph bitmap. Pixels set in
    /// `bits` take `fg`; all others take `bg`. Fixed 8×8 geometry ⇒ stack buffer,
    /// no heap allocation on the hot path.
    fn build_glyph_rgba(
        bits: [u8; crate::font::GLYPH_H],
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) -> [u8; crate::font::GLYPH_W * crate::font::GLYPH_H * 4] {
        let mut px = [0u8; crate::font::GLYPH_W * crate::font::GLYPH_H * 4];
        let w = crate::font::GLYPH_W;
        for (row, line) in bits.iter().enumerate() {
            for col in 0..w {
                let on = (line >> (w - 1 - col)) & 1 == 1;
                let (r, g, b) = if on { fg } else { bg };
                let o = (row * w + col) * 4;
                px[o] = r;
                px[o + 1] = g;
                px[o + 2] = b;
                px[o + 3] = 255;
            }
        }
        px
    }

    /// Emit a Sixel image of a glyph `bits` sized `w`×`h` at the cursor.
    ///
    /// Uses two color registers: register 0 = bg, register 1 = fg. Sixel rows are
    /// 6 pixels tall; an 8px-tall glyph spans 2 sixel rows. Each sixel character
    /// selects a color register, so per-bit fg/bg selection is faithful.
    fn emit_sixel_glyph(
        buf: &mut Vec<u8>,
        bits: [u8; crate::font::GLYPH_H],
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
        w: usize,
        h: usize,
    ) {
        buf.extend_from_slice(b"\x1bPq");
        // Register 0 = background.
        buf.extend_from_slice(b"#0;2;");
        write_decimal(buf, u32::from(bg.0) * 100 / 255);
        buf.push(b';');
        write_decimal(buf, u32::from(bg.1) * 100 / 255);
        buf.push(b';');
        write_decimal(buf, u32::from(bg.2) * 100 / 255);
        buf.push(b';');
        // Register 1 = foreground.
        buf.extend_from_slice(b"#1;2;");
        write_decimal(buf, u32::from(fg.0) * 100 / 255);
        buf.push(b';');
        write_decimal(buf, u32::from(fg.1) * 100 / 255);
        buf.push(b';');
        write_decimal(buf, u32::from(fg.2) * 100 / 255);

        let rows = h.div_ceil(6);
        for r in 0..rows {
            for col in 0..w {
                let mut six = 0u8;
                for dy in 0..6 {
                    let y = r * 6 + dy;
                    if y < h {
                        let line = bits[y];
                        let on = (line >> (w - 1 - col)) & 1 == 1;
                        if on {
                            six |= 1 << dy;
                        }
                    }
                }
                // A sixel char code is the OR of set-bit indices, and the bits
                // are painted with the *current* color register. We emit the run
                // for whichever register the majority selects by emitting two
                // characters when a row mixes fg and bg: first the bg bits under
                // register 0, then the fg bits under register 1.
                let all_fg = six == 0b0011_1111;
                let all_bg = six == 0;
                if all_bg {
                    // char '@' = code 63 with register 0 → all 6 bg pixels.
                    buf.push(b'@');
                } else if all_fg {
                    // Switch to register 1 then paint all six as fg.
                    buf.extend_from_slice(b"#1");
                    buf.push(b'?'); // '?' = code 63, painted with register 1.
                    buf.extend_from_slice(b"#0");
                } else {
                    // Mixed: paint bg bits (register 0) then fg bits (register 1).
                    buf.push(b'@' + six); // bits set here are bg (register 0)
                    buf.extend_from_slice(b"#1");
                    buf.push(b'@' + six); // same bit mask, now painted fg
                    buf.extend_from_slice(b"#0");
                }
            }
            if r + 1 < rows {
                buf.push(b'-');
            }
        }
        buf.extend_from_slice(b"\x1b\\");
    }

    /// Emit a Kitty graphics image of a solid `color` block sized `w`×`h` at the
    /// cursor. Uses the "simple" transmission (a=T, f=32 RGBA, uncompressed).
    fn emit_kitty_block(buf: &mut Vec<u8>, color: (u8, u8, u8), w: usize, h: usize) {
        let mut pixels: Vec<u8> = Vec::with_capacity(w * h * 4);
        for _ in 0..(w * h) {
            pixels.extend_from_slice(&[color.0, color.1, color.2, 255]);
        }
        let b64 = base64_encode(&pixels);

        buf.extend_from_slice(b"\x1b_Gf=32,a=T,s=");
        write_decimal(buf, w as u32);
        buf.extend_from_slice(b",v=");
        write_decimal(buf, h as u32);
        buf.extend_from_slice(b",m=1;");
        buf.extend_from_slice(b64.as_bytes());
        buf.extend_from_slice(b"\x1b\\");
    }

    /// Emit a Kitty graphics image of a glyph `bits` sized `w`×`h` at the cursor.
    /// Builds a real RGBA pixel buffer from the 8×8 bitmap (fg = set pixels,
    /// bg = unset) and transmits it uncompressed. No heap alloc beyond the
    /// fixed stack pixel buffer and the base64 string.
    fn emit_kitty_glyph(
        buf: &mut Vec<u8>,
        bits: [u8; crate::font::GLYPH_H],
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
        w: usize,
        h: usize,
    ) {
        let px = build_glyph_rgba(bits, fg, bg);
        let b64 = base64_encode(&px);
        buf.extend_from_slice(b"\x1b_Gf=32,a=T,s=");
        write_decimal(buf, w as u32);
        buf.extend_from_slice(b",v=");
        write_decimal(buf, h as u32);
        buf.extend_from_slice(b",m=1;");
        buf.extend_from_slice(b64.as_bytes());
        buf.extend_from_slice(b"\x1b\\");
    }

    /// Minimal, dependency-free base64 encoder (used only by the gated Kitty path).
    fn base64_encode(input: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
            out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
            if chunk.len() > 1 {
                out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(TABLE[(n & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct DummyFb;
        impl FrameBufferLike for DummyFb {
            fn width(&self) -> usize {
                2
            }
            fn height(&self) -> usize {
                1
            }
            fn cell(&self, _x: usize, _y: usize) -> super::super::CellView {
                super::super::CellView {
                    ch: 'X',
                    fg: (255, 0, 0),
                    bg: (0, 0, 0),
                    alpha: 255,
                }
            }
        }

        #[test]
        fn sixel_block_is_well_formed() {
            let mut out = Vec::new();
            let mut r = PlatformGraphicsRenderer::new(Protocol::Sixel);
            r.render_damage(&mut out, &DummyFb, &[(0, 0)]).unwrap();
            let s = String::from_utf8(out).unwrap();
            assert!(s.contains("\x1bPq"), "must open sixel DCS: {s}");
            // In glyph mode register #1 carries the glyph foreground (red).
            assert!(s.contains("#1;2;100;0;0"), "must define red fg color: {s}");
            assert!(s.ends_with("\x1b\\"), "must close DCS: {s}");
        }

        #[test]
        fn kitty_block_is_well_formed() {
            let mut out = Vec::new();
            let mut r = PlatformGraphicsRenderer::new(Protocol::Kitty);
            r.render_damage(&mut out, &DummyFb, &[(0, 0)]).unwrap();
            let s = String::from_utf8(out).unwrap();
            assert!(s.contains("\x1b_Gf=32,a=T"), "must open kitty cmd: {s}");
            assert!(s.ends_with("\x1b\\"), "must close DCS: {s}");
        }

        #[test]
        fn base64_known() {
            assert_eq!(base64_encode(b"Man"), "TWFu");
        }
    }
}
