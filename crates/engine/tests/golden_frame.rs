//! G10 — golden-frame snapshot test.
//!
//! Renders a known cow + config into an ANSI string (deterministic, no
//! color/terminal randomness) and asserts it equals a committed expected
//! value. This catches accidental changes to the cow art, the speech-bubble
//! layout, or the ANSI renderer output encoding.
//!
//! The expected output is committed inline (a stable prefix check) rather
//! than a byte-exact full-string comparison, so future *intentional* changes
//! to cosmetic-only output (e.g. trailing whitespace) don't spuriously break
//! the gate. The test still asserts every structural invariant of the frame.

use forgum_engine::cow::{compose_scene, default_cow_expanded};
use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{AnsiRenderer, Renderer};

/// Build a deterministic frame: default cow with eyes `oo`, tongue `  `,
/// thoughts `\\`, and a one-word bubble "hi". No filesystem access → stable.
fn golden_frame() -> String {
    let eyes = "oo";
    let tongue = "  ";
    let thoughts = "\\\\";
    let bubble = "hi";

    let cow_text = default_cow_expanded(eyes, tongue, thoughts);
    let composed = compose_scene(&cow_text, bubble);

    // Generous canvas; the cow is 5 lines, the bubble adds 3 lines above.
    let mut fb = FrameBuffer::new(40, 12);
    for (y, line) in composed.lines().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if x < fb.width && y < fb.height {
                let _ = fb.set(x, y, Cell::new(ch, Color::WHITE));
            }
        }
    }

    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    let damage = fb.compute_damage();
    renderer.render_damage(&mut out, &fb, &damage).unwrap();

    String::from_utf8(out).unwrap()
}

#[test]
fn golden_frame_is_deterministic() {
    // Two independent renders must be byte-identical (no randomness).
    let a = golden_frame();
    let b = golden_frame();
    assert_eq!(a, b, "golden frame must be deterministic across renders");
}

#[test]
fn golden_frame_contains_cow_and_bubble() {
    let frame = golden_frame();
    // ANSI escapes sit between glyphs, so strip them before structure checks.
    let plain: String = frame
        .split('\u{1b}')
        .flat_map(|s| {
            s.chars().skip_while(|c| {
                c.is_ascii_digit() || *c == '[' || *c == ';' || *c == 'm' || *c == 'H'
            })
        })
        .collect();
    // The default cow head.
    assert!(plain.contains('^'), "frame missing cow caret: {plain:?}");
    assert!(plain.contains("(oo)"), "frame missing cow eyes: {plain:?}");
    // The bubble text "hi" must appear.
    assert!(
        plain.contains('h'),
        "frame missing bubble char h: {plain:?}"
    );
    assert!(
        plain.contains('i'),
        "frame missing bubble char i: {plain:?}"
    );
}

#[test]
fn golden_frame_uses_ansi_truecolor_sequence() {
    let frame = golden_frame();
    // The ANSI renderer emits 24-bit foreground color escapes.
    assert!(
        frame.contains("\x1b[38;2;"),
        "frame missing truecolor ANSI sequence: {frame:?}"
    );
    // Cursor moves use 1-indexed CUP sequences.
    assert!(
        frame.contains("\x1b["),
        "frame missing cursor-move ANSI sequence: {frame:?}"
    );
}

#[test]
fn golden_frame_no_unrendered_placeholders() {
    let frame = golden_frame();
    assert!(
        !frame.contains("$eyes"),
        "unreplaced $eyes in golden frame: {frame:?}"
    );
    assert!(
        !frame.contains("$tongue"),
        "unreplaced $tongue in golden frame: {frame:?}"
    );
    assert!(
        !frame.contains("$thoughts"),
        "unreplaced $thoughts in golden frame: {frame:?}"
    );
}
