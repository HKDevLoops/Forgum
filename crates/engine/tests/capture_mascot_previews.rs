//! Capture high-fidelity PNG image snapshots for all 132 Forgum mascots
//! using the engine's font rasterizer and authentic "natural" color mode.

use forgum_engine::cow::{expand_cow, load_cow};
use forgum_engine::dna;
use forgum_engine::effects;
use forgum_engine::framebuffer::FrameBuffer;
use std::path::{Path, PathBuf};

use forgum_engine::capture_rasterizer::{
    render_framebuffer_to_rgba, DEFAULT_BG_COLOR as BG_COLOR, DEFAULT_CELL_H as CELL_H,
    DEFAULT_CELL_W as CELL_W,
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
    workspace_root()
        .join("test-renders")
        .join("mascot-previews")
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

fn framebuffer_to_rgba(fb: &FrameBuffer) -> Vec<u8> {
    render_framebuffer_to_rgba(fb, CELL_W, CELL_H, BG_COLOR).0
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
        let max_w = cow_lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(40);
        let max_h = cow_lines.len();

        let canvas_cols = (max_w + 6).max(60);
        let canvas_rows = (max_h + 4).max(18);

        // Color mode "natural" resolves authentic God-given palette
        let mut effect =
            effects::create_effect(cow_dna.base, cow_text, cow_dna.clone(), 0, "natural");

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

    println!(
        "Successfully captured {} mascot previews to {}",
        rendered,
        out.display()
    );
}
