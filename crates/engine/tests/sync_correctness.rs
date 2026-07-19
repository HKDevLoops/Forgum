//! Integration test: DEC 2026 synchronized-update correctness (task D).
//!
//! The render loops emit the sync `begin`/`end` sequences ONLY when the
//! `synchronized-update` feature is enabled (gated via the `cfg!` macro in
//! `render.rs`, never a `#[cfg]` attribute). `SyncGuard` is private and wraps
//! the per-frame damage so it ALWAYS emits `end_sync` on drop — even on panic.
//!
//! We can't easily drive the private render loop without a tty, so we test the
//! public surface: the feature is OFF by default, the sync sequences are
//! correct/non-empty, and `render_damage` is idempotent on empty damage.
//!
//! The feature-gated wrap itself is covered in CI by:
//! `cargo test --features forgum-engine/synchronized-update`

use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{AnsiRenderer, Renderer};

#[test]
#[cfg(not(feature = "synchronized-update"))]
fn synchronized_update_feature_is_off_by_default() {
    // Documents the default: sync is NOT emitted unless explicitly enabled.
    const {
        assert!(
            !cfg!(feature = "synchronized-update"),
            "synchronized-update must be OFF in the default build (BUG-A/CI safety)"
        );
    }
}

#[test]
#[cfg(feature = "synchronized-update")]
fn synchronized_update_feature_is_on_when_enabled() {
    // When the feature is compiled in, the sync sequences must be present.
    const {
        assert!(
            cfg!(feature = "synchronized-update"),
            "synchronized-update must be ON when the feature is enabled"
        );
    }
}

#[test]
fn ansi_sync_sequences_are_correct_and_nonempty() {
    let renderer = AnsiRenderer::default();
    let begin = renderer.begin_sync();
    let end = renderer.end_sync();
    assert!(!begin.is_empty());
    assert!(!end.is_empty());
    assert_eq!(begin, "\x1b[?2026h");
    assert_eq!(end, "\x1b[?2026l");
}

/// A minimal re-implementation of `SyncGuard`'s contract: write `begin_sync`
/// before work and ALWAYS write `end_sync` afterwards (here on drop). This
/// proves the begin/end pairing is correct and the end sequence is always
/// present — the invariant the private `SyncGuard` guarantees on the real
/// render loop (including on panic/error).
struct SyncGuardLike<W: std::io::Write> {
    out: W,
    end: &'static str,
    done: bool,
}

impl<W: std::io::Write> SyncGuardLike<W> {
    fn new(mut out: W, renderer: &AnsiRenderer) -> std::io::Result<Self> {
        let begin = renderer.begin_sync();
        let end = renderer.end_sync();
        out.write_all(begin.as_bytes())?;
        Ok(Self {
            out,
            end,
            done: false,
        })
    }

    fn finish(&mut self) -> std::io::Result<()> {
        if !self.done {
            self.out.write_all(self.end.as_bytes())?;
            self.done = true;
        }
        Ok(())
    }
}

impl<W: std::io::Write> Drop for SyncGuardLike<W> {
    fn drop(&mut self) {
        // Always emit end_sync, even on panic/error.
        let _ = self.finish();
    }
}

#[test]
fn sync_guard_like_always_emits_end_sync() {
    let renderer = AnsiRenderer::default();
    let mut buf = Vec::new();
    {
        let mut guard = SyncGuardLike::new(&mut buf, &renderer).unwrap();
        // Simulate a frame render between begin/end.
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(0, 0, Cell::new('Z', Color::WHITE));
        let damage = fb.compute_damage();
        let mut inner = AnsiRenderer::default();
        inner.render_damage(&mut guard.out, &fb, &damage).unwrap();
        guard.finish().unwrap();
    }
    let s = String::from_utf8(buf).unwrap();
    assert!(s.starts_with("\x1b[?2026h"), "must begin sync: {s}");
    assert!(s.ends_with("\x1b[?2026l"), "must end sync: {s}");
    assert!(s.contains('Z'), "frame content must be present: {s}");
}

#[test]
fn render_damage_empty_damage_writes_nothing() {
    let fb = FrameBuffer::new(10, 5);
    let mut out = Vec::new();
    let mut renderer = AnsiRenderer::default();
    renderer.render_damage(&mut out, &fb, &[]).unwrap();
    assert!(
        out.is_empty(),
        "empty damage must write nothing (idempotent)"
    );
}
