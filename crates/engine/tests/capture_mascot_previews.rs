//! Capture high-fidelity PNG image snapshots for all 132 Forgum mascots
//! using the engine's font rasterizer and authentic "natural" color mode.

use std::path::{Path, PathBuf};
use forgum_engine::cow::{expand_cow, load_cow};
use forgum_engine::dna;
use forgum_engine::effects;
use forgum_engine::framebuffer::{Color, FrameBuffer};

const CELL_W: usize = 8;
const CELL_H: usize = 16;
const BG_COLOR: Color = Color {
    r: 24,
    g: 24,
    b: 32,
    a: 255,
};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("workspace root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn data_dir() -> PathBuf {
    workspace_root().join("data")
}

fn output_dir() -> PathBuf {
    workspace_root().join("test-renders").join("mascot-previews")
}

fn all_cow_names() -> Vec<String> {
    let cows_dir = data_dir().join("Cows");
    let mut names: Vec<String> = std::fs::read_dir(&cows_dir)
        .expect("data/Cows/ must exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("cow"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    names.sort();
    names
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
    let alpha = cell.alpha as f32 / 255.0;
    let fr = (fg.r as f32 * alpha + bg.r as f32 * (1.0 - alpha)) as u8;
    let fg_g = (fg.g as f32 * alpha + bg.g as f32 * (1.0 - alpha)) as u8;
    let fb = (fg.b as f32 * alpha + bg.b as f32 * (1.0 - alpha)) as u8;
    let px_x = cell_x * CELL_W;
    let px_y = cell_y * CELL_H;
    let glyph_bits = forgum_platform::font::glyph(cell.ch);

    for dy in 0..CELL_H {
        let font_y = dy * 8 / CELL_H;
        for dx in 0..CELL_W {
            let px = (px_y + dy) * img_width + (px_x + dx);
            let base = px * 4;
            if base + 3 < pixels.len() {
                let is_fg = match glyph_bits {
                    Some(bits) => (bits[font_y] >> (7 - (dx.min(7)))) & 1 == 1,
                    None => {
                        if cell.ch != ' ' && cell.ch != '\0' && cell.ch != '\r' && cell.ch != '\n' {
                            dx == 0 || dx == CELL_W - 1 || dy == 0 || dy == CELL_H - 1
                        } else {
                            false
                        }
                    }
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

#[test]
fn capture_all_132_mascot_previews() {
    let dd = data_dir();
    let out = output_dir();
    std::fs::create_dir_all(&out).expect("failed to create output dir");

    let names = all_cow_names();
    assert_eq!(names.len(), 132, "Expected exactly 132 cows");

    let animations = dna::load_animations(&dd);
    println!("Capturing PNG previews for {} mascots...", names.len());

    let mut rendered = 0;
    for name in &names {
        let cow_raw = load_cow(name, &dd, "oo", " ", "\\\\");
        let cow_text = expand_cow(&cow_raw, "oo", " ", "\\\\");
        let cow_dna = dna::get_dna(&animations, name);

        // Determine canvas size from cow dimensions
        let cow_lines: Vec<&str> = cow_text.lines().collect();
        let max_w = cow_lines.iter().map(|l| l.chars().count()).max().unwrap_or(40);
        let max_h = cow_lines.len();

        let canvas_cols = (max_w + 6).max(60);
        let canvas_rows = (max_h + 4).max(18);

        // Color mode "natural" resolves authentic God-given palette
        let mut effect = effects::create_effect(
            cow_dna.base,
            cow_text,
            cow_dna.clone(),
            0,
            "natural",
        );

        let mut fb = FrameBuffer::new(canvas_cols, canvas_rows);
        // Settle animation to mid-stride/breathe
        for step in 0..5 {
            let dt = 1.0 / 30.0;
            let time = step as f32 * dt;
            fb.clear();
            effect.update(dt, canvas_cols, canvas_rows);
            effect.render(&mut fb, time);
            fb.swap();
        }

        let rgba = framebuffer_to_rgba(&fb);
        let img_w = (canvas_cols * CELL_W) as u32;
        let img_h = (canvas_rows * CELL_H) as u32;
        let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(img_w, img_h, rgba)
            .expect("create image buffer");

        let png_path = out.join(format!("{name}.png"));
        img.save(&png_path).expect("save PNG preview");
        rendered += 1;
    }

    println!("Successfully captured {} mascot previews to {}", rendered, out.display());
}
