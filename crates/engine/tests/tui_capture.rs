//! Capture photo (PNG) and video (MP4) of the LazyGit & Zellij inspired TUI dashboard.
//!
//! Verifies:
//! - Full in-memory execution with no config file dependency.
//! - Responsive layout rendering on ratatui TestBackend.
//! - Rasterization to PNG screenshots (Mascots, Installer, Scenery).
//! - 30 FPS video recording via ffmpeg encoding showing dynamic live preview animation.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use forgum_engine::framebuffer::{Color, FrameBuffer};
use forgum_tui::app::{ConfigApp, Tab};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const CELL_W: usize = 8;
const CELL_H: usize = 16;
const COLS: usize = 90;
const ROWS: usize = 28;
const FPS: u32 = 30;

const BG_COLOR: Color = Color {
    r: 24,
    g: 24,
    b: 37,
    a: 255,
};

fn workspace_root() -> PathBuf {
    let m = Path::new(env!("CARGO_MANIFEST_DIR"));
    m.parent().unwrap().parent().unwrap().to_path_buf()
}

fn test_renders_dir() -> PathBuf {
    workspace_root().join("test-renders")
}

fn video_dir() -> PathBuf {
    test_renders_dir().join("video")
}

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn ratatui_color_to_rgb(c: ratatui::style::Color) -> (u8, u8, u8) {
    match c {
        ratatui::style::Color::Reset => (248, 248, 242),
        ratatui::style::Color::Black => (24, 24, 37),
        ratatui::style::Color::Red => (255, 85, 85),
        ratatui::style::Color::Green => (80, 250, 123),
        ratatui::style::Color::Yellow => (241, 250, 140),
        ratatui::style::Color::Blue => (139, 233, 253),
        ratatui::style::Color::Magenta => (255, 121, 198),
        ratatui::style::Color::Cyan => (139, 233, 253),
        ratatui::style::Color::Gray => (140, 140, 160),
        ratatui::style::Color::DarkGray => (98, 114, 164),
        ratatui::style::Color::LightRed => (255, 110, 110),
        ratatui::style::Color::LightGreen => (105, 255, 148),
        ratatui::style::Color::LightYellow => (255, 255, 160),
        ratatui::style::Color::LightBlue => (160, 200, 255),
        ratatui::style::Color::LightMagenta => (255, 160, 220),
        ratatui::style::Color::LightCyan => (160, 255, 255),
        ratatui::style::Color::White => (248, 248, 242),
        ratatui::style::Color::Indexed(i) => {
            let hue = (i as f32 / 256.0) * std::f32::consts::TAU;
            let r = ((hue.sin() * 0.5 + 0.5) * 255.0) as u8;
            let g = (((hue + 2.094).sin() * 0.5 + 0.5) * 255.0) as u8;
            let b = (((hue + 4.188).sin() * 0.5 + 0.5) * 255.0) as u8;
            (r, g, b)
        }
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
    }
}

fn buffer_to_framebuffer(buffer: &ratatui::buffer::Buffer) -> FrameBuffer {
    let w = buffer.area.width as usize;
    let h = buffer.area.height as usize;
    let mut fb = FrameBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let cell = &buffer[(x as u16, y as u16)];
            let symbol = cell.symbol();
            let ch = symbol.chars().next().unwrap_or(' ');
            let (fr, fg_g, fb_b) = ratatui_color_to_rgb(cell.fg);
            let (br, bg_g, bb_b) = ratatui_color_to_rgb(cell.bg);

            fb.set(
                x,
                y,
                forgum_engine::framebuffer::Cell {
                    ch,
                    fg: Color {
                        r: fr,
                        g: fg_g,
                        b: fb_b,
                        a: 255,
                    },
                    bg: Color {
                        r: br,
                        g: bg_g,
                        b: bb_b,
                        a: if cell.bg == ratatui::style::Color::Reset {
                            0
                        } else {
                            255
                        },
                    },
                    alpha: 255,
                },
            );
        }
    }
    fb.swap();
    fb
}

fn render_cell_to_pixels(
    pixels: &mut [u8],
    img_width: usize,
    cell_x: usize,
    cell_y: usize,
    cell: &forgum_engine::framebuffer::Cell,
) {
    let fg = cell.fg;
    let bg = if cell.bg.a > 0 { cell.bg } else { BG_COLOR };
    let px_x = cell_x * CELL_W;
    let px_y = cell_y * CELL_H;
    let glyph_bits = forgum_platform::font::glyph(cell.ch)
        .or_else(|| forgum_platform::font::glyph(cell.ch.to_ascii_uppercase()));

    for dy in 0..CELL_H {
        let font_y = dy * 8 / CELL_H;
        for dx in 0..CELL_W {
            let px = (px_y + dy) * img_width + (px_x + dx);
            let base = px * 4;
            if base + 3 < pixels.len() {
                let is_fg = match glyph_bits {
                    Some(bits) => (bits[font_y] >> (7 - (dx.min(7)))) & 1 == 1,
                    None => match cell.ch {
                        '─' | '═' => dy == CELL_H / 2 || dy == CELL_H / 2 - 1,
                        '│' | '║' => dx == CELL_W / 2 || dx == CELL_W / 2 - 1,
                        '┌' | '╭' | '╔' => {
                            (dx >= CELL_W / 2 && (dy == CELL_H / 2 || dy == CELL_H / 2 - 1))
                                || (dy >= CELL_H / 2 && (dx == CELL_W / 2 || dx == CELL_W / 2 - 1))
                        }
                        '┐' | '╮' | '╗' => {
                            (dx <= CELL_W / 2 && (dy == CELL_H / 2 || dy == CELL_H / 2 - 1))
                                || (dy >= CELL_H / 2 && (dx == CELL_W / 2 || dx == CELL_W / 2 - 1))
                        }
                        '└' | '╰' | '╚' => {
                            (dx >= CELL_W / 2 && (dy == CELL_H / 2 || dy == CELL_H / 2 - 1))
                                || (dy <= CELL_H / 2 && (dx == CELL_W / 2 || dx == CELL_W / 2 - 1))
                        }
                        '┘' | '╯' | '╝' => {
                            (dx <= CELL_W / 2 && (dy == CELL_H / 2 || dy == CELL_H / 2 - 1))
                                || (dy <= CELL_H / 2 && (dx == CELL_W / 2 || dx == CELL_W / 2 - 1))
                        }
                        '├' | '╠' => {
                            (dx == CELL_W / 2 || dx == CELL_W / 2 - 1)
                                || (dx >= CELL_W / 2 && (dy == CELL_H / 2 || dy == CELL_H / 2 - 1))
                        }
                        '┤' | '╣' => {
                            (dx == CELL_W / 2 || dx == CELL_W / 2 - 1)
                                || (dx <= CELL_W / 2 && (dy == CELL_H / 2 || dy == CELL_H / 2 - 1))
                        }
                        '┼' | '╬' => {
                            (dx == CELL_W / 2 || dx == CELL_W / 2 - 1)
                                || (dy == CELL_H / 2 || dy == CELL_H / 2 - 1)
                        }
                        '▶' => dx >= (dy.min(CELL_H - 1 - dy) / 2) && dx <= CELL_W - 2,
                        '█' => true,
                        '▀' => dy < CELL_H / 2,
                        '▄' => dy >= CELL_H / 2,
                        _ => {
                            if cell.ch != ' '
                                && cell.ch != '\0'
                                && cell.ch != '\r'
                                && cell.ch != '\n'
                            {
                                dx == 0 || dx == CELL_W - 1 || dy == 0 || dy == CELL_H - 1
                            } else {
                                false
                            }
                        }
                    },
                };

                let (r, g, b) = if is_fg {
                    (fg.r, fg.g, fg.b)
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

fn framebuffer_to_rgba(fb: &FrameBuffer) -> Vec<u8> {
    let img_w = fb.width * CELL_W;
    let img_h = fb.height * CELL_H;
    let mut pixels = vec![0u8; img_w * img_h * 4];
    for y in 0..fb.height {
        for x in 0..fb.width {
            let cell = fb.get(x, y);
            render_cell_to_pixels(&mut pixels, img_w, x, y, &cell);
        }
    }
    pixels
}

fn save_framebuffer_png(fb: &FrameBuffer, out_path: &Path) {
    let img_w = (fb.width * CELL_W) as u32;
    let img_h = (fb.height * CELL_H) as u32;
    let rgba = framebuffer_to_rgba(fb);
    let img = image::RgbaImage::from_raw(img_w, img_h, rgba).expect("valid image buffer");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    img.save(out_path).expect("save PNG");
}

fn encode_rgba_frames_to_mp4(
    frames: &[Vec<u8>],
    width: usize,
    height: usize,
    fps: u32,
    out: &Path,
) -> Result<(), String> {
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &format!("{width}x{height}"),
            "-r",
            &fps.to_string(),
            "-i",
            "pipe:0",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "26",
            "-pix_fmt",
            "yuv420p",
            "-r",
            &fps.to_string(),
            "-an",
            &out.to_string_lossy(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn ffmpeg: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("no stdin")?;
        for f in frames {
            stdin
                .write_all(f)
                .map_err(|e| format!("write frame: {e}"))?;
        }
    }
    let out_res = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out_res.status.success() {
        return Err(format!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&out_res.stderr)
        ));
    }
    Ok(())
}

#[test]
fn capture_tui_screenshots_and_video() {
    // 1. Initialize app completely in-memory with NO config file
    let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));

    let backend = TestBackend::new(COLS as u16, ROWS as u16);
    let mut terminal = Terminal::new(backend).unwrap();

    // ── Capture Screen 1: Mascots Dashboard (Tab 0) ──────────────────────────
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    let buf = terminal.backend().buffer();
    println!("DEBUG: Buffer area: {:?}", buf.area);
    let mut non_space = 0;
    for y in 0..ROWS {
        for x in 0..COLS {
            let cell = &buf[(x as u16, y as u16)];
            let sym = cell.symbol();
            if sym != " " && !sym.is_empty() {
                non_space += 1;
                if non_space <= 5 {
                    println!(
                        "DEBUG: Cell ({},{}) = '{}' fg={:?} bg={:?}",
                        x, y, sym, cell.fg, cell.bg
                    );
                }
            }
        }
    }
    println!("DEBUG: Non space count = {}", non_space);

    let fb_mascots = buffer_to_framebuffer(terminal.backend().buffer());
    let png_mascots = test_renders_dir().join("tui_dashboard_mascots.png");
    save_framebuffer_png(&fb_mascots, &png_mascots);
    assert!(png_mascots.is_file(), "Mascots PNG must exist");

    // ── Capture Screen 2: Shell Installer Wizard (Tab 3) ─────────────────────
    app.current_tab = Tab::Installer;
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    let fb_installer = buffer_to_framebuffer(terminal.backend().buffer());
    let png_installer = test_renders_dir().join("tui_installer_wizard.png");
    save_framebuffer_png(&fb_installer, &png_installer);
    assert!(png_installer.is_file(), "Installer PNG must exist");

    // ── Capture Screen 3: Scenery & Biomes (Tab 1) ───────────────────────────
    app.current_tab = Tab::Scenery;
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    let fb_scenery = buffer_to_framebuffer(terminal.backend().buffer());
    let png_scenery = test_renders_dir().join("tui_scenery_biomes.png");
    save_framebuffer_png(&fb_scenery, &png_scenery);
    assert!(png_scenery.is_file(), "Scenery PNG must exist");

    // ── Capture Video: 30 FPS Live Holographic Motion (60 frames = 2 seconds) ─
    if ffmpeg_available() {
        app.current_tab = Tab::Mascots;
        let mut frames: Vec<Vec<u8>> = Vec::new();
        let dt = 1.0 / FPS as f32;

        for _ in 0..60 {
            app.tick(dt);
            terminal
                .draw(|f| {
                    app.render(f);
                })
                .unwrap();
            let fb = buffer_to_framebuffer(terminal.backend().buffer());
            frames.push(framebuffer_to_rgba(&fb));
        }

        let img_w = COLS * CELL_W;
        let img_h = ROWS * CELL_H;
        let mp4_path = video_dir().join("tui_live_preview.mp4");

        let res = encode_rgba_frames_to_mp4(&frames, img_w, img_h, FPS, &mp4_path);
        assert!(res.is_ok(), "Video encoding must succeed: {:?}", res.err());
        assert!(mp4_path.is_file(), "TUI MP4 video file must exist");
        let meta = std::fs::metadata(&mp4_path).unwrap();
        assert!(
            meta.len() > 1000,
            "MP4 video must contain valid encoded frames"
        );
    }
}
