//! Visual rendering test — renders ALL 106 cows with their DNA profiles
//! into PNG images for quality inspection.
//!
//! Run with: cargo test --test visual_render -- --nocapture
//! Output:   test-renders/<cow_name>.png (one per cow, frame 30 snapshot)
//!           test-renders/contact-sheet.png (all cows in a grid)
//!           test-renders/gallery.html (browseable HTML page)
//!           test-renders/golden/ (blake3 hashes for regression)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use forgum_engine::cow::{expand_cow, load_cow};
use forgum_engine::dna;
use forgum_engine::effects;
use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};

// ── Constants ──────────────────────────────────────────────────────

/// Terminal cell dimensions in pixels.
const CELL_W: usize = 8;
const CELL_H: usize = 16;

/// Number of animation frames to render per cow.
const FRAMES_PER_COW: usize = 60;

/// Which frame to save as the "hero" snapshot.
const HERO_FRAME: usize = 30;

/// Canvas size in terminal cells.
const CANVAS_COLS: usize = 80;
const CANVAS_ROWS: usize = 24;

/// Background color (dark terminal).
const BG_COLOR: Color = Color {
    r: 30,
    g: 30,
    b: 40,
    a: 255,
};

// ── Helpers ────────────────────────────────────────────────────────

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
    workspace_root().join("test-renders")
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

// ── Framebuffer → PNG conversion ───────────────────────────────────

/// Render a single cell into a pixel buffer at the given cell position.
fn render_cell_to_pixels(
    pixels: &mut [u8],
    img_width: usize,
    cell_x: usize,
    cell_y: usize,
    cell: &Cell,
) {
    let fg = cell.fg;
    let bg = if cell.bg.a > 0 { cell.bg } else { BG_COLOR };

    let alpha = cell.alpha as f32 / 255.0;
    let fr = (fg.r as f32 * alpha + bg.r as f32 * (1.0 - alpha)) as u8;
    let fg_g = (fg.g as f32 * alpha + bg.g as f32 * (1.0 - alpha)) as u8;
    let fb = (fg.b as f32 * alpha + bg.b as f32 * (1.0 - alpha)) as u8;

    let px_x = cell_x * CELL_W;
    let px_y = cell_y * CELL_H;

    let (r, g, b) = if cell.ch == ' ' || cell.ch == '\0' {
        (bg.r, bg.g, bg.b)
    } else {
        (fr, fg_g, fb)
    };

    for dy in 0..CELL_H {
        for dx in 0..CELL_W {
            let px = (px_y + dy) * img_width + (px_x + dx);
            let base = px * 4;
            if base + 3 < pixels.len() {
                pixels[base] = r;
                pixels[base + 1] = g;
                pixels[base + 2] = b;
                pixels[base + 3] = 255;
            }
        }
    }
}

/// Convert a FrameBuffer to an RGBA pixel buffer.
fn framebuffer_to_pixels(fb: &FrameBuffer) -> Vec<u8> {
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

/// Save a framebuffer as a PNG file.
fn save_fb_png(fb: &FrameBuffer, path: &Path) {
    let img_w = (fb.width * CELL_W) as u32;
    let img_h = (fb.height * CELL_H) as u32;
    let pixels = framebuffer_to_pixels(fb);
    let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(img_w, img_h, pixels)
        .expect("failed to create image buffer");
    img.save(path).expect("failed to save PNG");
}

/// Create a contact sheet: all cows in a grid layout.
fn create_contact_sheet(renders: &[(String, FrameBuffer)], cols: usize, path: &Path) {
    if renders.is_empty() {
        return;
    }

    let thumb_w = CANVAS_COLS * CELL_W;
    let thumb_h = CANVAS_ROWS * CELL_H;
    let label_h = 20usize;
    let rows = renders.len().div_ceil(cols);

    let sheet_w = cols * thumb_w;
    let sheet_h = rows * (thumb_h + label_h);

    let mut sheet = vec![0u8; sheet_w * sheet_h * 4];

    for (i, (_name, fb)) in renders.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let ox = col * thumb_w;
        let oy = row * (thumb_h + label_h) + label_h;

        let src_pixels = framebuffer_to_pixels(fb);
        for dy in 0..thumb_h {
            for dx in 0..thumb_w {
                let src_px = (dy * thumb_w + dx) * 4;
                let dst_px = ((oy + dy) * sheet_w + (ox + dx)) * 4;
                if src_px + 3 < src_pixels.len() && dst_px + 3 < sheet.len() {
                    sheet[dst_px] = src_pixels[src_px];
                    sheet[dst_px + 1] = src_pixels[src_px + 1];
                    sheet[dst_px + 2] = src_pixels[src_px + 2];
                    sheet[dst_px + 3] = 255;
                }
            }
        }

        // Label background strip.
        let label_y = oy - label_h;
        for dy in 0..label_h {
            for dx in 0..thumb_w {
                let base = ((label_y + dy) * sheet_w + (ox + dx)) * 4;
                if base + 3 < sheet.len() {
                    sheet[base] = 40;
                    sheet[base + 1] = 40;
                    sheet[base + 2] = 50;
                    sheet[base + 3] = 255;
                }
            }
        }
    }

    let img =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(sheet_w as u32, sheet_h as u32, sheet)
            .expect("failed to create contact sheet");
    img.save(path).expect("failed to save contact sheet");
}

/// Generate an HTML gallery page.
fn create_gallery(renders: &[(String, FrameBuffer)], path: &Path) {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html>
<head>
<title>Forgum Visual Render Gallery</title>
<style>
body { background: #1a1a2e; color: #eee; font-family: monospace; margin: 20px; }
h1 { color: #ff8800; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(640px, 1fr)); gap: 20px; }
.card { background: #16213e; border: 1px solid #0f3460; border-radius: 8px; padding: 12px; }
.card h3 { margin: 0 0 8px 0; color: #e94560; font-size: 14px; }
.card img { width: 100%; image-rendering: pixelated; background: #1e1e2e; border-radius: 4px; }
</style>
</head>
<body>
<h1>Forgum Visual Render Gallery</h1>
<p>106 cows x 60 frames. Each image shows frame 30 (mid-animation).</p>
<div class="grid">
"#,
    );

    for (name, _fb) in renders {
        html.push_str(&format!(
            r#"  <div class="card">
    <h3>{name}</h3>
    <img src="{name}.png" alt="{name}">
  </div>
"#,
        ));
    }

    html.push_str("</div>\n</body>\n</html>");
    std::fs::write(path, html).expect("failed to write gallery HTML");
}

// ── The visual render test ─────────────────────────────────────────

#[test]
fn render_all_cows_to_png() {
    let dd = data_dir();
    let out = output_dir();

    std::fs::create_dir_all(&out).expect("failed to create output dir");
    std::fs::create_dir_all(out.join("golden")).expect("failed to create golden dir");

    let names = all_cow_names();
    assert!(
        names.len() >= 100,
        "Expected >=100 cows, found {}",
        names.len()
    );

    let animations = dna::load_animations(&dd);
    let mut renders: Vec<(String, FrameBuffer)> = Vec::new();
    let mut _golden_hashes: HashMap<String, String> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();

    for name in &names {
        let cow_raw = load_cow(name, &dd, "oo", " ", "\\\\");
        let cow_text = expand_cow(&cow_raw, "oo", " ", "\\\\");
        let cow_dna = dna::get_dna(&animations, name);

        let mut effect = effects::create_effect(cow_dna.base, cow_text.clone(), cow_dna.clone(), 0);

        let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
        let mut hero_fb = None;

        for frame in 0..FRAMES_PER_COW {
            let time = frame as f32 / 30.0;
            let dt = 1.0 / 30.0;

            fb.clear();
            effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
            effect.render(&mut fb, time);

            if frame == HERO_FRAME {
                hero_fb = Some(fb.clone());
            }

            fb.swap();
        }

        if let Some(ref hero) = hero_fb {
            let png_path = out.join(format!("{name}.png"));
            if let Err(e) = std::panic::catch_unwind(|| {
                save_fb_png(hero, &png_path);
            }) {
                failures.push(format!("{name}: PNG save failed: {e:?}"));
                continue;
            }

            let hash = blake3::hash(&framebuffer_to_pixels(hero));
            _golden_hashes.insert(name.clone(), hash.to_hex().to_string());

            let golden_path = out.join("golden").join(format!("{name}.blake3"));
            std::fs::write(&golden_path, hash.to_hex().as_bytes())
                .expect("failed to write golden hash");

            renders.push((name.clone(), hero.clone()));
        }
    }

    if !renders.is_empty() {
        let sheet_path = out.join("contact-sheet.png");
        create_contact_sheet(&renders, 3, &sheet_path);

        let gallery_path = out.join("gallery.html");
        create_gallery(&renders, &gallery_path);
    }

    let total = names.len();
    let ok = renders.len();
    let fail = failures.len();

    println!("\n=== Visual Render Results ===");
    println!("Total cows:  {total}");
    println!("Rendered:    {ok}");
    println!("Failed:      {fail}");

    if !failures.is_empty() {
        println!("\nFailures:");
        for f in &failures {
            println!("  FAIL: {f}");
        }
    }

    println!("\nOutput directory: {}", out.display());
    println!("  - Individual PNGs: test-renders/<cow>.png");
    println!("  - Contact sheet:   test-renders/contact-sheet.png");
    println!("  - HTML gallery:    test-renders/gallery.html");
    println!("  - Golden hashes:   test-renders/golden/<cow>.blake3");

    assert!(
        failures.is_empty(),
        "{} cows failed to render:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ── Golden-visual regression test ──────────────────────────────────
//
// Only tests cows with particle_rate=0 (fully deterministic effects).
// Cows with particles use RNG and are non-deterministic.

#[test]
fn golden_visual_regression_deterministic_cows() {
    let dd = data_dir();
    let out = output_dir();
    let golden_dir = out.join("golden");

    if !golden_dir.exists() {
        eprintln!(
            "Skipping golden regression -- no golden hashes found. \
             Run render_all_cows_to_png first."
        );
        return;
    }

    let names = all_cow_names();
    let animations = dna::load_animations(&dd);
    let mut regressions: Vec<String> = Vec::new();
    let mut skipped = 0usize;

    for name in &names {
        let cow_dna = dna::get_dna(&animations, name);

        // Skip cows with particles — they use RNG and are non-deterministic.
        if cow_dna.particles.rate > 0 {
            skipped += 1;
            continue;
        }

        let golden_path = golden_dir.join(format!("{name}.blake3"));
        if !golden_path.exists() {
            continue;
        }

        let expected = std::fs::read_to_string(&golden_path)
            .expect("failed to read golden hash")
            .trim()
            .to_string();

        let cow_raw = load_cow(name, &dd, "oo", " ", "\\\\");
        let cow_text = expand_cow(&cow_raw, "oo", " ", "\\\\");
        let mut effect = effects::create_effect(cow_dna.base, cow_text.clone(), cow_dna.clone(), 0);

        let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
        for frame in 0..=HERO_FRAME {
            let time = frame as f32 / 30.0;
            let dt = 1.0 / 30.0;
            fb.clear();
            effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
            effect.render(&mut fb, time);
            fb.swap();
        }

        let hash = blake3::hash(&framebuffer_to_pixels(&fb));
        let actual = hash.to_hex().to_string();

        if actual != expected {
            regressions.push(format!("{name}: expected {expected}, got {actual}"));
        }
    }

    eprintln!(
        "Golden regression: {} cows tested, {} skipped (particles/rng)",
        names.len() - skipped,
        skipped,
    );

    if !regressions.is_empty() {
        eprintln!("\n=== Visual Regressions ===");
        for r in &regressions {
            eprintln!("  FAIL: {r}");
        }
        eprintln!("\nTo re-golden: delete test-renders/golden/ and re-run render_all_cows_to_png");
    }

    assert!(
        regressions.is_empty(),
        "{} visual regressions detected:\n{}",
        regressions.len(),
        regressions.join("\n")
    );
}

// ── Rendering quality checks ───────────────────────────────────────

#[test]
fn all_effects_produce_nonempty_frames() {
    let dd = data_dir();
    let names = all_cow_names();
    let animations = dna::load_animations(&dd);
    let mut issues: Vec<String> = Vec::new();

    for name in &names {
        let cow_raw = load_cow(name, &dd, "oo", " ", "\\\\");
        let cow_text = expand_cow(&cow_raw, "oo", " ", "\\\\");
        let cow_dna = dna::get_dna(&animations, name);
        let mut effect = effects::create_effect(cow_dna.base, cow_text.clone(), cow_dna.clone(), 0);

        let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
        let mut any_content = false;

        for frame in 0..FRAMES_PER_COW {
            let time = frame as f32 / 30.0;
            let dt = 1.0 / 30.0;
            fb.clear();
            effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
            effect.render(&mut fb, time);

            for y in 0..CANVAS_ROWS {
                for x in 0..CANVAS_COLS {
                    let cell = fb.get(x, y);
                    if cell.ch != ' ' && cell.ch != '\0' {
                        any_content = true;
                        break;
                    }
                }
                if any_content {
                    break;
                }
            }

            fb.swap();
        }

        if !any_content {
            issues.push(format!(
                "{name}: produced NO visible content across {FRAMES_PER_COW} frames"
            ));
        }
    }

    if !issues.is_empty() {
        eprintln!("\n=== Rendering Quality Issues ===");
        for i in &issues {
            eprintln!("  FAIL: {i}");
        }
    }

    assert!(
        issues.is_empty(),
        "{} cows have rendering issues:\n{}",
        issues.len(),
        issues.join("\n")
    );
}

#[test]
fn all_cows_have_correct_color_range() {
    let dd = data_dir();
    let names = all_cow_names();
    let animations = dna::load_animations(&dd);
    let mut issues: Vec<String> = Vec::new();

    for name in &names {
        let cow_raw = load_cow(name, &dd, "oo", " ", "\\\\");
        let cow_text = expand_cow(&cow_raw, "oo", " ", "\\\\");
        let cow_dna = dna::get_dna(&animations, name);

        // Only check color range for cows whose base animation IS Particles.
        // Other base types (Breathe, Float, etc.) render white cow text only;
        // particles.rate is just a config that would be used if base were Particles.
        if cow_dna.base != dna::BaseAnim::Particles {
            continue;
        }

        let mut effect = effects::create_effect(cow_dna.base, cow_text.clone(), cow_dna.clone(), 0);

        let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
        let mut has_colored_cells = false;

        // Run multiple frames so particles have time to spawn and render.
        for frame in 0..30 {
            let time = frame as f32 / 30.0;
            let dt = 1.0 / 30.0;
            fb.clear();
            effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
            effect.render(&mut fb, time);
            fb.swap();

            for y in 0..CANVAS_ROWS {
                for x in 0..CANVAS_COLS {
                    let cell = fb.get(x, y);
                    if cell.ch != ' ' && (cell.fg.r != 255 || cell.fg.g != 255 || cell.fg.b != 255)
                    {
                        has_colored_cells = true;
                        break;
                    }
                }
                if has_colored_cells {
                    break;
                }
            }
            if has_colored_cells {
                break;
            }
        }

        if !has_colored_cells && cow_dna.particles.rate > 0 {
            issues.push(format!(
                "{name}: base=Particles with particle rate {} but no colored cells rendered",
                cow_dna.particles.rate
            ));
        }
    }

    if !issues.is_empty() {
        eprintln!("\n=== Color Issues ===");
        for i in &issues {
            eprintln!("  FAIL: {i}");
        }
    }

    assert!(
        issues.is_empty(),
        "{} cows have color issues:\n{}",
        issues.len(),
        issues.join("\n")
    );
}
