//! Adaptive frame scheduler.
//!
//! Three tiers:
//! - **Active** — damage > 0, render at `target_fps`.
//! - **Idle** — N consecutive frames with zero damage, sleep longer (1 Hz).
//! - **Suspended** — explicit pause from the control socket (Phase 1).
//!
//! Tier transitions:
//! - Active → Idle after 15 zero-damage frames.
//! - Idle → Active on first frame with damage.
//! - Suspended → previous tier on `resume()`.
//!
//! The CPU-savings invariant (BUG-E1 regression test): a static cow must
//! fall to Idle within ~0.25 s and stay there.

use std::time::{Duration, Instant};

/// How many consecutive zero-damage frames before we go Idle.
pub const IDLE_THRESHOLD: u32 = 15;

/// Scheduler state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerTier {
    Active,
    Idle,
    Suspended,
}

#[derive(Debug)]
pub struct Scheduler {
    /// Base fps the scheduler was constructed with — the anchor for `set_speed`.
    base_fps: u16,
    target_fps: u16,
    tier: SchedulerTier,
    consecutive_idle_frames: u32,
    last_frame: Instant,
    paused_from: Option<SchedulerTier>,
}

impl Scheduler {
    #[must_use]
    pub fn new(target_fps: u16) -> Self {
        let fps = if target_fps == 0 { 30 } else { target_fps };
        Self {
            base_fps: fps,
            target_fps: fps,
            tier: SchedulerTier::Active,
            consecutive_idle_frames: 0,
            last_frame: Instant::now(),
            paused_from: None,
        }
    }

    #[must_use]
    pub fn tier(&self) -> SchedulerTier {
        self.tier
    }

    #[must_use]
    pub fn target_fps(&self) -> u16 {
        self.target_fps
    }

    /// Multiply the target frame rate by `mult` so subsequent frames animate
    /// faster (`mult > 1.0`) or slower (`mult < 1.0`). The base is the fps the
    /// scheduler was constructed with, so repeated calls scale from the original
    /// rate rather than compounding on the current one.
    ///
    /// A `mult` of `0.0` or less keeps the previous speed (a dead cow can't
    /// speed up). The result is clamped to `[1, 240]` fps to stay sane.
    pub fn set_speed(&mut self, mult: f32) {
        if mult <= 0.0 {
            return;
        }
        let base = if self.base_fps == 0 {
            30
        } else {
            self.base_fps
        };
        let next = (f32::from(base) * mult).round() as i32;
        self.target_fps = next.clamp(1, 240) as u16;
    }

    /// Time since last tick, clamped to 0.1 s (so a stalled frame doesn't
    /// make effects fast-forward).
    #[must_use]
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let dt = now.saturating_duration_since(self.last_frame);
        self.last_frame = now;
        // Clamp: a stalled frame should not advance physics by 30 seconds.
        dt.min(Duration::from_millis(100))
    }

    /// Update the tier based on damage.
    ///
    /// * `damaged_count` — number of cells that differ between back and front.
    pub fn observe(&mut self, damaged_count: usize) {
        if matches!(self.tier, SchedulerTier::Suspended) {
            return;
        }
        if damaged_count == 0 {
            self.consecutive_idle_frames = self.consecutive_idle_frames.saturating_add(1);
            if self.consecutive_idle_frames >= IDLE_THRESHOLD && self.tier == SchedulerTier::Active
            {
                self.tier = SchedulerTier::Idle;
            }
        } else {
            self.consecutive_idle_frames = 0;
            self.tier = SchedulerTier::Active;
        }
    }

    /// How long to sleep before the next frame.
    #[must_use]
    pub fn frame_period(&self) -> Duration {
        match self.tier {
            SchedulerTier::Active => {
                Duration::from_micros(1_000_000 / u64::from(self.target_fps.max(1)))
            }
            SchedulerTier::Idle => Duration::from_secs(1),
            SchedulerTier::Suspended => Duration::from_secs(60),
        }
    }

    pub fn pause(&mut self) {
        if !matches!(self.tier, SchedulerTier::Suspended) {
            self.paused_from = Some(self.tier);
            self.tier = SchedulerTier::Suspended;
        }
    }

    pub fn resume(&mut self) {
        if let Some(prev) = self.paused_from.take() {
            self.tier = prev;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_cow_goes_idle() {
        let mut s = Scheduler::new(30);
        for _ in 0..IDLE_THRESHOLD {
            s.observe(0);
        }
        assert_eq!(s.tier(), SchedulerTier::Idle);
        assert_eq!(s.frame_period(), Duration::from_secs(1));
    }

    #[test]
    fn damage_wakes_from_idle() {
        let mut s = Scheduler::new(30);
        for _ in 0..IDLE_THRESHOLD + 1 {
            s.observe(0);
        }
        assert_eq!(s.tier(), SchedulerTier::Idle);
        s.observe(1);
        assert_eq!(s.tier(), SchedulerTier::Active);
    }

    #[test]
    fn paused_blocks_observation() {
        let mut s = Scheduler::new(30);
        s.pause();
        for _ in 0..100 {
            s.observe(0);
        }
        assert_eq!(s.tier(), SchedulerTier::Suspended);
        s.resume();
        assert_eq!(s.tier(), SchedulerTier::Active);
    }

    #[test]
    fn tick_clamps_long_stalls() {
        let mut s = Scheduler::new(30);
        // Force a long "frame" by sleeping 200 ms before tick.
        std::thread::sleep(Duration::from_millis(200));
        let dt = s.tick();
        assert!(dt <= Duration::from_millis(100));
    }
}
