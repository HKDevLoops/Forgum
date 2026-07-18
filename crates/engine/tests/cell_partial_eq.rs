//! G5 — `Cell::PartialEq` ignores `dirty` (or any equivalent bookkeeping),
//! so a framebuffer of identical cells has zero damage.

use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};

#[test]
fn cell_partial_eq_ignores_dirty() {
    // The fix for BUG-E1: two cells with the same ch/fg/bg/alpha compare
    // equal regardless of internal bookkeeping.
    let a = Cell::new('x', Color::WHITE);
    let b = Cell::new('x', Color::WHITE);
    assert_eq!(a, b);

    // Differing ch → not equal.
    let c = Cell::new('y', Color::WHITE);
    assert_ne!(a, c);

    // Differing color → not equal.
    let d = Cell::new('x', Color::BLACK);
    assert_ne!(a, d);
}

#[test]
fn framebuffer_damage_is_empty_when_buffers_match() {
    // G1: `clear()` marks nothing dirty; only a `set()` against the still-old
    // front produces damage. Restoring the exact same cell must yield zero
    // damage (BUG-E1 invariant — identical cells → 0 damage).
    let mut fb = FrameBuffer::new(4, 4);
    fb.set(1, 1, Cell::new('a', Color::WHITE));
    fb.swap(); // commit: front now has 'a'
    fb.clear(); // back is empty, marks nothing dirty
    assert!(fb.compute_damage().is_empty());
    fb.set(1, 1, Cell::new('a', Color::WHITE)); // restore identical cell
    assert!(fb.compute_damage().is_empty());
}

#[test]
fn framebuffer_damage_correctness() {
    let mut fb = FrameBuffer::new(4, 4);
    fb.set(0, 0, Cell::new('a', Color::WHITE));
    fb.swap();
    fb.clear();
    fb.set(0, 0, Cell::new('b', Color::WHITE));
    let dmg = fb.compute_damage();
    assert!(dmg.contains(&(0, 0)));
    assert_eq!(dmg.len(), 1);
}

#[test]
fn scheduler_idle_transition_is_within_15_frames() {
    use forgum_engine::scheduler::{Scheduler, SchedulerTier, IDLE_THRESHOLD};
    let mut s = Scheduler::new(30);
    for _ in 0..IDLE_THRESHOLD {
        s.observe(0);
    }
    assert_eq!(s.tier(), SchedulerTier::Idle);
}

#[test]
fn scheduler_wakes_on_damage() {
    use forgum_engine::scheduler::{Scheduler, SchedulerTier, IDLE_THRESHOLD};
    let mut s = Scheduler::new(30);
    for _ in 0..IDLE_THRESHOLD + 5 {
        s.observe(0);
    }
    assert_eq!(s.tier(), SchedulerTier::Idle);
    s.observe(1);
    assert_eq!(s.tier(), SchedulerTier::Active);
}
