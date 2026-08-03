//! Integration test: render correctness, specifically the stale-frame bug (BUG-A).
//!
//! The render pipeline builds the next frame into `back` (via effects calling
//! `fb.set`) then `compute_damage()` and `AnsiRenderer::render_damage`. The
//! renderer MUST read the caller-provided cells, not a stale buffer.

use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{AnsiRenderer, Renderer};

#[test]
fn renderer_emits_cells_from_slice() {
    let mut fb = FrameBuffer::new(10, 5);
    fb.set(3, 2, Cell::new('X', Color::WHITE));

    let damage = fb.compute_damage().to_vec();
    assert!(damage.contains(&(3, 2)), "damage must include (3,2)");

    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer
        .render_damage(&mut out, &fb.back, fb.cols(), &damage)
        .unwrap();
    let s = String::from_utf8(out).unwrap();

    assert!(
        s.contains('X'),
        "back-buffer cell 'X' missing from output: {s}"
    );
}

#[test]
fn get_back_vs_get_semantics() {
    let mut fb = FrameBuffer::new(4, 4);
    fb.set(1, 1, Cell::new('A', Color::WHITE));
    assert_eq!(fb.get_back(1, 1).ch, 'A');
    assert_eq!(fb.get(1, 1).ch, ' ');

    fb.swap();
    // After swap: front has 'A' (the last back), back is the old empty front.
    assert_eq!(fb.get(1, 1).ch, 'A');

    fb.set(2, 2, Cell::new('B', Color::WHITE));
    assert_eq!(fb.get_back(2, 2).ch, 'B');
    assert_eq!(fb.get(2, 2).ch, ' ');
}

#[test]
fn render_damage_noop_on_empty_damage() {
    let fb = FrameBuffer::new(10, 5);
    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer
        .render_damage(&mut out, &fb.back, fb.cols(), &[])
        .unwrap();
    assert!(out.is_empty());
}
