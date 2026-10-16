//! High-fidelity RGBA Framebuffer Rasterizer for Video & Preview Capture.
//!
//! Provides unified cell-to-RGBA and framebuffer-to-RGBA rasterization
//! using `forgum_platform::font::glyph` bitmap font tables with sub-cell alpha blending
//! and box-drawing character support.
//!
//! Consolidates previously triplicated implementations across tests, TUI capture,
//! and mascot video render pipelines (PERF-006 / PERF-007).

use crate::framebuffer::{Cell, Color, FrameBuffer};

pub const DEFAULT_CELL_W: usize = 8;
pub const DEFAULT_CELL_H: usize = 16;
pub const DEFAULT_BG_COLOR: Color = Color {
    r: 24,
    g: 24,
    b: 32,
    a: 255,
};

/// Render a single terminal cell into an RGBA pixel buffer at the specified grid cell coordinates.
pub fn render_cell_to_rgba(
    pixels: &mut [u8],
    img_width: usize,
    cell_x: usize,
    cell_y: usize,
    cell: &Cell,
    cell_w: usize,
    cell_h: usize,
    default_bg: Color,
) {
    let fg = cell.fg;
    let bg = if cell.bg.a > 0 { cell.bg } else { default_bg };
    let alpha = f32::from(cell.alpha) / 255.0;
    let fr = (f32::from(fg.r) * alpha + f32::from(bg.r) * (1.0 - alpha)) as u8;
    let fg_g = (f32::from(fg.g) * alpha + f32::from(bg.g) * (1.0 - alpha)) as u8;
    let fb = (f32::from(fg.b) * alpha + f32::from(bg.b) * (1.0 - alpha)) as u8;
    let px_x = cell_x * cell_w;
    let px_y = cell_y * cell_h;
    let glyph_bits = forgum_platform::font::glyph(cell.ch)
        .or_else(|| forgum_platform::font::glyph(cell.ch.to_ascii_uppercase()));

    for dy in 0..cell_h {
        let font_y = dy * 8 / cell_h;
        for dx in 0..cell_w {
            let px = (px_y + dy) * img_width + (px_x + dx);
            let base = px * 4;
            if base + 3 < pixels.len() {
                let is_fg = match glyph_bits {
                    Some(bits) => (bits[font_y] >> (7 - (dx.min(7)))) & 1 == 1,
                    None => match cell.ch {
                        '─' | '═' => dy == cell_h / 2 || dy == cell_h / 2 - 1,
                        '│' | '║' => dx == cell_w / 2 || dx == cell_w / 2 - 1,
                        '┌' | '╭' | '╔' => {
                            (dx >= cell_w / 2 && (dy == cell_h / 2 || dy == cell_h / 2 - 1))
                                || (dy >= cell_h / 2 && (dx == cell_w / 2 || dx == cell_w / 2 - 1))
                        }
                        '┐' | '╮' | '╗' => {
                            (dx <= cell_w / 2 && (dy == cell_h / 2 || dy == cell_h / 2 - 1))
                                || (dy >= cell_h / 2 && (dx == cell_w / 2 || dx == cell_w / 2 - 1))
                        }
                        '└' | '╰' | '╚' => {
                            (dx >= cell_w / 2 && (dy == cell_h / 2 || dy == cell_h / 2 - 1))
                                || (dy <= cell_h / 2 && (dx == cell_w / 2 || dx == cell_w / 2 - 1))
                        }
                        '┘' | '╯' | '╝' => {
                            (dx <= cell_w / 2 && (dy == cell_h / 2 || dy == cell_h / 2 - 1))
                                || (dy <= cell_h / 2 && (dx == cell_w / 2 || dx == cell_w / 2 - 1))
                        }
                        '├' | '╠' => {
                            (dx == cell_w / 2 || dx == cell_w / 2 - 1)
                                || (dx >= cell_w / 2 && (dy == cell_h / 2 || dy == cell_h / 2 - 1))
                        }
                        '┤' | '╣' => {
                            (dx == cell_w / 2 || dx == cell_w / 2 - 1)
                                || (dx <= cell_w / 2 && (dy == cell_h / 2 || dy == cell_h / 2 - 1))
                        }
                        '┼' | '╬' => {
                            (dx == cell_w / 2 || dx == cell_w / 2 - 1)
                                || (dy == cell_h / 2 || dy == cell_h / 2 - 1)
                        }
                        '▶' => dx >= (dy.min(cell_h - 1 - dy) / 2) && dx <= cell_w - 2,
                        '█' => true,
                        '▀' => dy < cell_h / 2,
                        '▄' => dy >= cell_h / 2,
                        _ => {
                            if cell.ch != ' '
                                && cell.ch != '\0'
                                && cell.ch != '\r'
                                && cell.ch != '\n'
                            {
                                dx == 0 || dx == cell_w - 1 || dy == 0 || dy == cell_h - 1
                            } else {
                                false
                            }
                        }
                    },
                };

                let (r, g, b) = if is_fg {
                    (fr, fg_g, fb)
                } else {
                    (bg.r, bg.g, bg.b)
                };

                pixels[base] = r;
                pixels[base + 1] = g;
                pixels[base + 2] = b;
                pixels[base + 3] = 255;
            }
        }
    }
}

/// Rasterize an entire `FrameBuffer` into a continuous 32-bit RGBA pixel byte buffer.
/// Returns `(rgba_pixels, image_pixel_width, image_pixel_height)`.
pub fn render_framebuffer_to_rgba(
    fb: &FrameBuffer,
    cell_w: usize,
    cell_h: usize,
    bg_color: Color,
) -> (Vec<u8>, usize, usize) {
    let cols = fb.width;
    let rows = fb.height;
    let img_w = cols * cell_w;
    let img_h = rows * cell_h;
    let mut pixels = vec![0u8; img_w * img_h * 4];

    for y in 0..rows {
        for x in 0..cols {
            let cell = fb.get(x, y);
            render_cell_to_rgba(&mut pixels, img_w, x, y, &cell, cell_w, cell_h, bg_color);
        }
    }

    (pixels, img_w, img_h)
}
