//! Integration test: renderer + effects public API surface.

use forgum_engine::dna::{BaseAnim, CowDna};
use forgum_engine::effects::{create_effect, default_cow_text, render_static_cow, Effect};
use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{
    create_renderer, is_tmux, AnsiRenderer, Renderer, TmuxPassthroughRenderer,
};

#[test]
fn static_effect_new_and_render() {
    let effect = forgum_engine::effects::StaticEffect::new(default_cow_text().to_string());
    let mut fb = FrameBuffer::new(40, 12);
    effect.render(&mut fb, 0.0);
    fb.swap();
    // The cow art should have produced non-space cells.
    let mut found = false;
    for y in 0..fb.height {
        for x in 0..fb.width {
            if fb.get(x, y).ch != ' ' {
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    assert!(found, "StaticEffect must render visible cow art");
}

#[test]
fn render_static_cow_writes_cells() {
    let mut fb = FrameBuffer::new(40, 12);
    render_static_cow(&mut fb, "  ^__^  \n (oo)   \n(__)    ");
    fb.swap();
    assert_eq!(fb.get(2, 0).ch, '^');
}

#[test]
fn create_effect_renders_for_each_base_anim() {
    let dna = CowDna::default();
    let bases = [
        BaseAnim::Breathe,
        BaseAnim::Float,
        BaseAnim::Walk,
        BaseAnim::Particles,
        BaseAnim::Pulse,
        BaseAnim::Glitch,
        BaseAnim::Fly,
        BaseAnim::Talk,
        BaseAnim::Sway,
        BaseAnim::Dissolve,
    ];
    for base in &bases {
        let mut effect = create_effect(*base, "  ^__^  ".to_string(), dna.clone(), 0);
        // Each base animation must construct, update, and render without
        // panicking. Some effects draw only on specific phases, so we assert the
        // render step runs cleanly rather than requiring visible pixels.
        for t in [0.0_f32, 0.5, 1.0, 2.0] {
            effect.update(0.1, 40, 12);
            let mut fb = FrameBuffer::new(40, 12);
            effect.render(&mut fb, t);
            fb.swap();
            let _ = fb.compute_damage();
        }
        assert!(!effect.is_done(), "{base:?} should not be done yet");
    }
}

#[test]
fn effect_is_done_default_false() {
    let effect = forgum_engine::effects::StaticEffect::new("x".to_string());
    assert!(!effect.is_done());
}

#[test]
fn ansi_renderer_writes_move_and_char() {
    let mut fb = FrameBuffer::new(10, 5);
    let _ = fb.set(3, 2, Cell::new('X', Color::WHITE));
    fb.swap();

    let mut out = Vec::new();
    let mut renderer = AnsiRenderer;
    let damage = vec![(3, 2)];
    renderer.render_damage(&mut out, &fb, &damage).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("\x1b[3;4H"), "expected cursor move: {s}");
    assert!(s.contains('X'), "expected char X: {s}");
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
    assert!(s.contains("\x1bPtmux;"), "expected tmux DCS start: {s}");
    assert!(s.contains("\x1b\\"), "expected tmux DCS end: {s}");
}

#[test]
fn is_tmux_returns_bool() {
    // Force a deterministic value, then restore.
    let prior = std::env::var("TMUX").ok();
    std::env::remove_var("TMUX");
    assert!(!is_tmux(), "is_tmux must be false without TMUX set");
    std::env::set_var("TMUX", "/tmp/tmux-1000/default,1234,0");
    assert!(is_tmux(), "is_tmux must be true with TMUX set");
    match prior {
        Some(v) => std::env::set_var("TMUX", v),
        None => std::env::remove_var("TMUX"),
    }
}

#[test]
fn create_renderer_returns_renderer() {
    let _renderer = create_renderer();
}

#[test]
fn scene_config_default_is_static() {
    let cfg = forgum_engine::protocol::SceneConfig::default();
    assert_eq!(cfg.cow, "default");
    assert_eq!(cfg.effect, "static");
    assert_eq!(cfg.fps, 30);
    assert!(!cfg.background);
}
