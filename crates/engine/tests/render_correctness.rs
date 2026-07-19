//! Integration test: render correctness, specifically the stale-frame bug (BUG-A).
//!
//! The render pipeline builds the next frame into `back` (via effects calling
//! `fb.set`) then `compute_damage()` and `AnsiRenderer::render_damage`. The
//! renderer MUST read `back` (the just-built frame), not `front` (the last
//! swapped frame), otherwise every frame shows the previous frame's content.

use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{AnsiRenderer, Renderer};

#[test]
fn renderer_emits_back_buffer_cell_not_stale_front() {
    let mut fb = FrameBuffer::new(10, 5);
    // Build an initial frame and swap it in.
    fb.set(3, 2, Cell::new('X', Color::WHITE));
    fb.swap();

    // Now build a NEW frame in back without swapping: (0,0) = 'Y'.
    // Front still holds 'X' at (3,2). If the renderer read `front` (the buggy
    // behaviour), output would contain 'X' but NOT the freshly built 'Y'.
    fb.set(0, 0, Cell::new('Y', Color::WHITE));

    let damage = fb.compute_damage();
    assert!(damage.contains(&(0, 0)), "damage must include (0,0)");

    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer.render_damage(&mut out, &fb, &damage).unwrap();
    let s = String::from_utf8(out).unwrap();

    assert!(
        s.contains('Y'),
        "stale-frame bug NOT fixed: back-buffer cell 'Y' missing from output: {s}"
    );
}

#[test]
fn get_back_vs_get_semantics() {
    let mut fb = FrameBuffer::new(4, 4);
    // Set in back before any swap.
    fb.set(1, 1, Cell::new('A', Color::WHITE));
    // Before swap: get_back sees 'A', get (front) sees empty.
    assert_eq!(fb.get_back(1, 1).ch, 'A');
    assert_eq!(fb.get(1, 1).ch, ' ');

    fb.swap();
    // After swap: both agree.
    assert_eq!(fb.get_back(1, 1).ch, 'A');
    assert_eq!(fb.get(1, 1).ch, 'A');

    // New unswapped write to back.
    fb.set(2, 2, Cell::new('B', Color::WHITE));
    assert_eq!(fb.get_back(2, 2).ch, 'B');
    // Front must not reflect the unswapped write.
    assert_eq!(fb.get(2, 2).ch, ' ');
}

#[test]
fn render_damage_noop_on_empty_damage() {
    let fb = FrameBuffer::new(10, 5);
    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer.render_damage(&mut out, &fb, &[]).unwrap();
    assert!(out.is_empty());
}
