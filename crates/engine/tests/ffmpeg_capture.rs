use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use forgum_engine::cow::load_cow;
use forgum_engine::dna;
use forgum_engine::effects;
use forgum_engine::framebuffer::{Color, FrameBuffer};

const CELL_W: usize = 8;
const CELL_H: usize = 16;
const CANVAS_COLS: usize = 80;
const CANVAS_ROWS: usize = 24;
const FPS: u32 = 30;
const FRAMES_PER_ANIM: usize = 60;
const BG_COLOR: Color = Color {
    r: 30,
    g: 30,
    b: 40,
    a: 255,
};

fn workspace_root() -> PathBuf {
    let m = Path::new(env!("CARGO_MANIFEST_DIR"));
    m.parent().unwrap().parent().unwrap().to_path_buf()
}
fn data_dir() -> PathBuf {
    workspace_root().join("data")
}
fn video_dir() -> PathBuf {
    workspace_root().join("test-renders").join("video")
}

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
fn ffprobe_available() -> bool {
    Command::new("ffprobe")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
fn all_cow_names() -> Vec<String> {
    let cows_dir = data_dir().join("Cows");
    let mut names: Vec<String> = std::fs::read_dir(&cows_dir)
        .unwrap()
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
                            // Border box fallback for non-ASCII/unmapped glyphs
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
            "28",
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
fn probe_video(path: &Path) -> Result<serde_json::Value, String> {
    let o = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,codec_name,avg_frame_rate,duration,nb_frames,r_frame_rate",
            "-of",
            "json",
            &path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(String::from_utf8_lossy(&o.stderr).to_string());
    }
    serde_json::from_slice(&o.stdout).map_err(|e| e.to_string())
}
fn render_anim_frames(
    cow_name: &str,
    base_anim: Option<dna::BaseAnim>,
    frames: usize,
) -> Vec<Vec<u8>> {
    let dd = data_dir();
    let anims = dna::load_animations(&dd);
    let cow_raw = load_cow(cow_name, &dd, "oo", " ", "o");
    let cow_text = forgum_engine::cow::compose_scene_with_mode(
        &cow_raw,
        "Moo! Forgum rules the pasture!",
        true,
    );
    let cow_dna = dna::get_dna(&anims, cow_name);
    let base = base_anim.unwrap_or(cow_dna.base);
    let mut effect = effects::create_effect(base, cow_text, cow_dna.clone(), 0, "static");
    let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
    let mut out = Vec::with_capacity(frames);
    for frame in 0..frames {
        let time = frame as f32 / FPS as f32;
        let dt = 1.0 / FPS as f32;
        fb.clear();
        effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
        effect.render(&mut fb, time);
        fb.swap();
        out.push(framebuffer_to_rgba(&fb));
    }
    out
}

#[test]
fn ffmpeg_availability() {
    if !ffmpeg_available() {
        eprintln!("SKIP: ffmpeg not found in PATH");
        return;
    }
    if !ffprobe_available() {
        eprintln!("SKIP: ffprobe not found");
        return;
    }
    let o = Command::new("ffmpeg").arg("-version").output().unwrap();
    assert!(o.status.success());
    println!(
        "ffmpeg: {}",
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .next()
            .unwrap_or("")
    );
}

#[test]
fn ffmpeg_single_cow_smoke_capture() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let cow = "default";
    let frames = render_anim_frames(cow, None, FRAMES_PER_ANIM);
    assert_eq!(frames.len(), FRAMES_PER_ANIM);
    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;
    let out = video_dir().join(format!("{cow}.mp4"));
    encode_rgba_frames_to_mp4(&frames, w, h, FPS, &out).expect("encode failed");
    assert!(out.exists(), "mp4 not created");
    let md = std::fs::metadata(&out).unwrap();
    assert!(md.len() > 1024, "mp4 too small: {} bytes", md.len());
    let probe = probe_video(&out).expect("ffprobe failed");
    let stream = &probe["streams"][0];
    assert_eq!(stream["width"].as_u64().unwrap() as usize, w);
    assert_eq!(stream["height"].as_u64().unwrap() as usize, h);
    assert_eq!(stream["codec_name"].as_str().unwrap(), "h264");
    println!(
        "OK {cow}.mp4 {} bytes {}x{} codec={}",
        md.len(),
        w,
        h,
        stream["codec_name"].as_str().unwrap()
    );
    let png_out = video_dir().join(format!("{cow}_frame30.png"));
    let st = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &out.to_string_lossy(),
            "-vf",
            "select=eq(n\\,30)",
            "-vframes",
            "1",
            &png_out.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert!(
        st.status.success(),
        "extract frame: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    assert!(png_out.exists());
    assert!(std::fs::metadata(&png_out).unwrap().len() > 500);
    println!("Extracted {}", png_out.display());
}

#[test]
fn ffmpeg_scenery_capture() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let dd = data_dir();
    let anims = dna::load_animations(&dd);
    let cow_name = "default";
    let cow_raw = load_cow(cow_name, &dd, "oo", " ", "\\\\");
    let cow_text = forgum_engine::cow::compose_scene_with_mode(
        &cow_raw,
        "Testing scenery layout: mountain, animal, and road.",
        false,
    );
    let road_y = effects::find_cow_foot_y(&cow_text) + 1;
    let cow_dna = dna::get_dna(&anims, cow_name);
    let mut effect = effects::create_effect(cow_dna.base, cow_text, cow_dna.clone(), 0, "animal");
    let (mtn_style, road_style, env_style) = forgum_engine::scenery::resolve_archetype(cow_name);

    let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
    let mut frames = Vec::with_capacity(FRAMES_PER_ANIM);
    for frame in 0..FRAMES_PER_ANIM {
        let time = frame as f32 / FPS as f32;
        let dt = 1.0 / FPS as f32;
        fb.clear();
        forgum_engine::scenery::render_scenery(
            &mut fb, mtn_style, road_style, env_style, road_y, time,
        );

        effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
        effect.render(&mut fb, time);
        fb.swap();
        frames.push(framebuffer_to_rgba(&fb));
    }

    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;
    let mp4_out = video_dir().join("scenery_issue.mp4");
    let png_out = video_dir().join("scenery_issue.png");
    encode_rgba_frames_to_mp4(&frames, w, h, FPS, &mp4_out).expect("encode failed");
    let st = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &mp4_out.to_string_lossy(),
            "-vf",
            "select=eq(n\\,10)",
            "-vframes",
            "1",
            &png_out.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert!(st.status.success());
    println!("Scenery capture saved to {}", png_out.display());
}

#[test]
fn ffmpeg_nyan_and_wildlife_capture() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let dd = data_dir();
    let anims = dna::load_animations(&dd);
    let showcases = [
        ("nyan-cat", "Nyanyanyanyanyanyanya!"),
        ("pterodactyl", "Soaring above prehistoric cliffs"),
        ("octopus", "Gliding through the deep sapphire trench"),
        ("charlie", "Golden retriever on a sunny mountain path"),
    ];

    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;

    for (cow_name, thought) in showcases {
        let profile = forgum_engine::scenery::get_animal_profile(cow_name);
        let cow_raw = load_cow(cow_name, &dd, profile.eyes, profile.tongue, "o");
        let cow_text = forgum_engine::cow::compose_scene_with_mode(&cow_raw, thought, true);
        let road_y = effects::find_cow_foot_y(&cow_text) + 1;
        let cow_dna = dna::get_dna(&anims, cow_name);
        let mut effect = effects::create_effect(profile.base_anim, cow_text, cow_dna, 0, "animal");

        let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
        let mut frames = Vec::with_capacity(FRAMES_PER_ANIM);
        for frame in 0..FRAMES_PER_ANIM {
            let time = frame as f32 / FPS as f32;
            let dt = 1.0 / FPS as f32;
            fb.clear();
            forgum_engine::scenery::render_scenery(
                &mut fb,
                profile.mountain,
                profile.road,
                profile.environment,
                road_y,
                time,
            );
            effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
            effect.render(&mut fb, time);
            fb.swap();
            frames.push(framebuffer_to_rgba(&fb));
        }

        let tag = format!("{cow_name}_showcase");
        let mp4_out = video_dir().join(format!("{tag}.mp4"));
        let png_out = video_dir().join(format!("{tag}.png"));
        encode_rgba_frames_to_mp4(&frames, w, h, FPS, &mp4_out).expect("encode failed");
        let st = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &mp4_out.to_string_lossy(),
                "-vf",
                "select=eq(n\\,30)",
                "-vframes",
                "1",
                &png_out.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert!(st.status.success());
        println!("Saved showcase video {} and image {}", mp4_out.display(), png_out.display());
    }
}

#[test]
fn ffmpeg_new_animals_showcase_capture() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let dd = data_dir();
    let anims = dna::load_animations(&dd);
    let showcases = [
        ("corgi", "A cheerful corgi trotting along cobblestone hills!"),
        ("duck", "Quack! Floating peacefully across misty wetlands."),
        ("wolf", "Howling through the moonlit pines and rocky crags."),
        ("tiger", "Prowling through the amber savanna grass."),
        ("vader", "The dark side of the terminal is a pathway to many abilities..."),
        ("moose", "Majestic northern moose commanding the boreal forest."),
        ("moofasa", "The sun will never set on our kingdom."),
        ("minotaur", "Guardian of the labyrinth."),
        ("tux", "Powered by Linux and written in pure Rust."),
    ];

    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;

    for (cow_name, thought) in showcases {
        let profile = forgum_engine::scenery::get_animal_profile(cow_name);
        let cow_raw = load_cow(cow_name, &dd, profile.eyes, profile.tongue, "o");
        let cow_text = forgum_engine::cow::compose_scene_with_mode(&cow_raw, thought, true);
        let road_y = effects::find_cow_foot_y(&cow_text) + 1;
        let cow_dna = dna::get_dna(&anims, cow_name);
        let mut effect = effects::create_effect(profile.base_anim, cow_text, cow_dna, 0, "animal");

        let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
        let mut frames = Vec::with_capacity(FRAMES_PER_ANIM);
        for frame in 0..FRAMES_PER_ANIM {
            let time = frame as f32 / FPS as f32;
            let dt = 1.0 / FPS as f32;
            fb.clear();
            forgum_engine::scenery::render_scenery(
                &mut fb,
                profile.mountain,
                profile.road,
                profile.environment,
                road_y,
                time,
            );
            effect.update(dt, CANVAS_COLS, CANVAS_ROWS);
            effect.render(&mut fb, time);
            fb.swap();
            frames.push(framebuffer_to_rgba(&fb));
        }

        let tag = format!("{cow_name}_showcase");
        let mp4_out = video_dir().join(format!("{tag}.mp4"));
        let png_out = video_dir().join(format!("{tag}.png"));
        encode_rgba_frames_to_mp4(&frames, w, h, FPS, &mp4_out).expect("encode failed");
        let st = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &mp4_out.to_string_lossy(),
                "-vf",
                "select=eq(n\\,30)",
                "-vframes",
                "1",
                &png_out.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert!(st.status.success());
        println!("Saved showcase video {} and image {}", mp4_out.display(), png_out.display());
    }
}

#[test]
fn ffmpeg_all_effects_capture() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let cow = "default";
    let effects_list = [
        dna::BaseAnim::Breathe,
        dna::BaseAnim::Float,
        dna::BaseAnim::Walk,
        dna::BaseAnim::Particles,
        dna::BaseAnim::Pulse,
        dna::BaseAnim::Glitch,
        dna::BaseAnim::Fly,
        dna::BaseAnim::Talk,
        dna::BaseAnim::Sway,
        dna::BaseAnim::Dissolve,
    ];
    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;
    let mut failures = Vec::new();
    for base in effects_list {
        let frames = render_anim_frames(cow, Some(base), FRAMES_PER_ANIM);
        let tag = format!("{cow}_{base:?}").to_lowercase();
        let out = video_dir().join(format!("{tag}.mp4"));
        if let Err(e) = encode_rgba_frames_to_mp4(&frames, w, h, FPS, &out) {
            failures.push(format!("{tag}: {e}"));
            continue;
        }
        if !out.exists() || std::fs::metadata(&out).unwrap().len() < 512 {
            failures.push(format!("{tag}: file missing/small"));
            continue;
        }
        match probe_video(&out) {
            Ok(v) => {
                let s = &v["streams"][0];
                if s["codec_name"].as_str() != Some("h264") {
                    failures.push(format!("{tag}: bad codec {:?}", s["codec_name"]));
                }
                let dur: f64 = s["duration"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                let exp = FRAMES_PER_ANIM as f64 / FPS as f64;
                if (dur - exp).abs() > 0.2 {
                    failures.push(format!("{tag} duration {dur} != {exp}"));
                }
            }
            Err(e) => failures.push(format!("{tag} probe: {e}")),
        }
        let decoded = Command::new("ffmpeg")
            .args([
                "-i",
                &out.to_string_lossy(),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "pipe:1",
            ])
            .output()
            .unwrap();
        if !decoded.status.success() || decoded.stdout.len() < w * h * 4 * FRAMES_PER_ANIM * 9 / 10
        {
            failures.push(format!("{tag} decode bytes {}", decoded.stdout.len()));
        }
        println!("OK {tag}.mp4 + decode {} bytes", decoded.stdout.len());
    }
    assert!(
        failures.is_empty(),
        "effect capture failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn ffmpeg_all_cows_capture() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let names = all_cow_names();
    assert!(names.len() >= 100, "need >=100 cows");
    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;
    let whole = std::env::var("FORGUM_FFMPEG_WHOLE")
        .ok()
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    let frames_needed = if whole { FRAMES_PER_ANIM } else { 30 };
    let limit = if whole { names.len() } else { 20 };
    let mut ok = 0usize;
    let mut fails: Vec<String> = Vec::new();
    for name in names.iter().take(limit) {
        let frames = render_anim_frames(name, None, frames_needed);
        let out = video_dir().join(format!("{name}.mp4"));
        match encode_rgba_frames_to_mp4(&frames, w, h, FPS, &out) {
            Ok(_) => {
                let sz = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
                if sz < 512 {
                    fails.push(format!("{name}: small {sz}"));
                } else {
                    ok += 1;
                }
            }
            Err(e) => fails.push(format!("{name}: {e}")),
        }
    }
    println!(
        "ffmpeg_all_cows: {ok}/{limit} ok ({frames_needed} frames each), {} fails",
        fails.len()
    );
    if !fails.is_empty() {
        for f in &fails {
            eprintln!("FAIL {f}");
        }
    }
    assert!(
        fails.is_empty(),
        "{} cows failed ffmpeg capture",
        fails.len()
    );
    if ok > 0 {
        let sample = video_dir().join(format!("{}.mp4", names[0]));
        let probe = probe_video(&sample).expect("probe sample");
        let nb = probe["streams"][0]["nb_frames"].as_str().unwrap_or("30");
        println!("Sample probe nb_frames={nb} (expected {frames_needed})");
        let out = Command::new("ffmpeg")
            .args([
                "-i",
                &sample.to_string_lossy(),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "pipe:1",
            ])
            .output()
            .expect("ffmpeg decode");
        assert!(
            out.status.success(),
            "decode failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let expected_bytes = w * h * 4 * frames_needed;
        assert!(
            out.stdout.len() >= expected_bytes * 9 / 10,
            "decoded {} < expected {}",
            out.stdout.len(),
            expected_bytes
        );
        println!("ffmpeg decode OK: {} bytes", out.stdout.len());
    }
}

#[test]
#[ignore]
fn ffmpeg_whole_output_capture_and_manifest() {
    if !ffmpeg_available() || !ffprobe_available() {
        eprintln!("SKIP no ffmpeg");
        return;
    }
    let names = all_cow_names();
    assert!(names.len() >= 100);
    let w = CANVAS_COLS * CELL_W;
    let h = CANVAS_ROWS * CELL_H;
    let out_dir = video_dir();
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut manifest: Vec<serde_json::Value> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    let mut total_frames = 0usize;
    for (idx, name) in names.iter().enumerate() {
        if idx % 20 == 0 {
            println!("whole-output {idx}/{}: {name}", names.len());
        }
        let out = out_dir.join(format!("{name}.mp4"));
        let needs_encode = !out.exists()
            || std::fs::metadata(&out)
                .map(|m| m.len() < 512)
                .unwrap_or(true)
            || std::env::var("FORGUM_REENCODE").ok().as_deref() == Some("1");
        if needs_encode {
            let frames = render_anim_frames(name, None, FRAMES_PER_ANIM);
            if let Err(e) = encode_rgba_frames_to_mp4(&frames, w, h, FPS, &out) {
                failures.push(format!("{name} encode: {e}"));
                continue;
            }
        }
        let meta = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
        if meta < 512 {
            failures.push(format!("{name} small file {meta}"));
            continue;
        }
        match probe_video(&out) {
            Ok(p) => {
                let s = &p["streams"][0];
                let duration: f64 = s["duration"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                let expected_dur = FRAMES_PER_ANIM as f64 / FPS as f64;
                if (duration - expected_dur).abs() > 0.2 {
                    failures.push(format!("{name} duration {duration} != {expected_dur}"));
                }
                let rgba = framebuffer_to_rgba(&{
                    let dd = data_dir();
                    let anims = dna::load_animations(&dd);
                    let cow_raw = load_cow(name, &dd, "oo", " ", "o");
                    let cow_text = forgum_engine::cow::compose_scene_with_mode(
                        &cow_raw,
                        "Moo! Forgum rules the pasture!",
                        true,
                    );
                    let cow_dna = dna::get_dna(&anims, name);
                    let mut eff = effects::create_effect(
                        cow_dna.base,
                        cow_text,
                        cow_dna.clone(),
                        0,
                        "static",
                    );
                    let mut fb = FrameBuffer::new(CANVAS_COLS, CANVAS_ROWS);
                    for f in 0..FRAMES_PER_ANIM {
                        let t = f as f32 / FPS as f32;
                        fb.clear();
                        eff.update(1.0 / FPS as f32, CANVAS_COLS, CANVAS_ROWS);
                        eff.render(&mut fb, t);
                        fb.swap();
                    }
                    fb
                });
                let hash = blake3::hash(&rgba).to_hex().to_string();
                manifest.push(serde_json::json!({
                    "cow": name,
                    "file": format!("{name}.mp4"),
                    "frames": FRAMES_PER_ANIM,
                    "fps": FPS,
                    "width": w,
                    "height": h,
                    "duration": duration,
                    "codec": s["codec_name"].as_str().unwrap_or("h264"),
                    "size_bytes": meta,
                    "blake3_rgba_last_frame": hash,
                }));
                total_frames += FRAMES_PER_ANIM;
            }
            Err(e) => failures.push(format!("{name} probe: {e}")),
        }
    }
    let manifest_path = out_dir.join("manifest.json");
    let whole = serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "cows": names.len(),
        "frames_per_cow": FRAMES_PER_ANIM,
        "total_frames": total_frames,
        "fps": FPS,
        "resolution": format!("{w}x{h}"),
        "entries": manifest,
    });
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&whole).unwrap(),
    )
    .unwrap();
    println!(
        "Whole-output manifest: {} cows, {} frames, written to {}",
        names.len(),
        total_frames,
        manifest_path.display()
    );
    assert!(manifest_path.exists());
    let concat_list = out_dir.join("concat.txt");
    let mut concat_content = String::new();
    for e in whole["entries"].as_array().unwrap() {
        concat_content.push_str(&format!("file '{}'\n", e["file"].as_str().unwrap()));
    }
    std::fs::write(&concat_list, &concat_content).unwrap();
    let whole_mp4 = out_dir.join("whole_output.mp4");
    let st = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            &concat_list.to_string_lossy(),
            "-c",
            "copy",
            &whole_mp4.to_string_lossy(),
        ])
        .output()
        .unwrap();
    if !st.status.success() {
        eprintln!(
            "concat copy failed, re-encoding: {}",
            String::from_utf8_lossy(&st.stderr)
        );
        let _ = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                &concat_list.to_string_lossy(),
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                &whole_mp4.to_string_lossy(),
            ])
            .output();
    }
    if whole_mp4.exists() {
        let probe = probe_video(&whole_mp4).unwrap_or(serde_json::json!({}));
        println!(
            "whole_output.mp4: {} bytes probe={}",
            std::fs::metadata(&whole_mp4).unwrap().len(),
            probe
        );
        let decoded = Command::new("ffmpeg")
            .args([
                "-i",
                &whole_mp4.to_string_lossy(),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(decoded.status.success(), "whole decode failed");
        println!("whole_output decode OK {} bytes", decoded.stdout.len());
    }
    assert!(
        failures.is_empty(),
        "whole-output failures:\n{}",
        failures.join("\n")
    );
    println!(
        "WHOLE-OUTPUT CAPTURE OK: {}/{} cows",
        whole["entries"].as_array().unwrap().len(),
        names.len()
    );
}
