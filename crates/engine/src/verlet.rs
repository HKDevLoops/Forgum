//! Verlet integration chains for secondary motion (tails, capes, ears).
//!
//! `VerletChain<N>` simulates N linked points with distance constraints.
//! Position-based dynamics (not velocity-based) for unconditional stability.

/// A Verlet-integrated chain of `N` linked points.
#[derive(Debug, Clone)]
pub struct VerletChain<const N: usize> {
    pos: [[f32; 2]; N],
    prev: [[f32; 2]; N],
    rest_len: [f32; N],
    damping: f32,
}

impl<const N: usize> VerletChain<N> {
    /// Create a new chain hanging downward from `anchor`.
    pub fn new(anchor: [f32; 2], link_len: f32) -> Self {
        let mut pos = [[0.0f32; 2]; N];
        let mut prev = [[0.0f32; 2]; N];
        for i in 0..N {
            pos[i] = [anchor[0], anchor[1] + i as f32 * link_len];
            prev[i] = pos[i];
        }
        let mut rest_len = [0.0f32; N];
        rest_len.fill(link_len);
        Self {
            pos,
            prev,
            rest_len,
            damping: 0.98,
        }
    }

    /// Set damping factor (0.0–1.0, default 0.98).
    pub fn with_damping(mut self, d: f32) -> Self {
        self.damping = d;
        self
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// - `anchor` — position to pin link 0 to (the attachment point)
    /// - `gravity` — downward acceleration (positive = down)
    /// - `wind` — horizontal force [x, y]
    pub fn step(&mut self, dt: f32, gravity: f32, anchor: [f32; 2], wind: [f32; 2]) {
        // 1. Verlet integrate
        for i in 1..N {
            let vel_x = (self.pos[i][0] - self.prev[i][0]) * self.damping;
            let vel_y = (self.pos[i][1] - self.prev[i][1]) * self.damping;
            self.prev[i] = self.pos[i];
            self.pos[i][0] += vel_x + wind[0] * dt * dt;
            self.pos[i][1] += vel_y + gravity * dt * dt;
        }
        // Pin link 0
        self.pos[0] = anchor;
        self.prev[0] = anchor;

        // 2. Distance-constraint relaxation (4 iterations)
        for _ in 0..4 {
            for i in 0..N.saturating_sub(1) {
                let dx = self.pos[i + 1][0] - self.pos[i][0];
                let dy = self.pos[i + 1][1] - self.pos[i][1];
                let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                let diff = (dist - self.rest_len[i]) / dist * 0.5;
                if i != 0 {
                    self.pos[i][0] += dx * diff;
                    self.pos[i][1] += dy * diff;
                }
                self.pos[i + 1][0] -= dx * diff;
                self.pos[i + 1][1] -= dy * diff;
            }
        }
    }

    /// Apply an impulse to a specific link.
    pub fn impulse(&mut self, link_idx: usize, force: [f32; 2]) {
        if link_idx < N {
            self.pos[link_idx][0] += force[0];
            self.pos[link_idx][1] += force[1];
        }
    }

    /// Get the position of link `i`.
    #[must_use]
    pub fn position(&self, i: usize) -> [f32; 2] {
        self.pos[i.min(N - 1)]
    }

    /// Get the angle of segment `i` (in radians, from horizontal).
    #[must_use]
    pub fn segment_angle(&self, i: usize) -> f32 {
        if i >= N - 1 {
            return 0.0;
        }
        let dx = self.pos[i + 1][0] - self.pos[i][0];
        let dy = self.pos[i + 1][1] - self.pos[i][1];
        dy.atan2(dx)
    }

    /// Get all positions as a slice.
    #[must_use]
    pub fn positions(&self) -> &[[f32; 2]; N] {
        &self.pos
    }
}

/// Type alias for a 4-link tail (cats, bunnies).
pub type Tail4 = VerletChain<4>;

/// Type alias for a 6-link tail (dragons).
pub type Tail6 = VerletChain<6>;

/// Type alias for a 3-link cape (batman).
pub type Cape3 = VerletChain<3>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_hangs_downward() {
        let chain = Tail4::new([10.0, 0.0], 2.0);
        assert!((chain.position(0)[1] - 0.0).abs() < 0.001);
        assert!((chain.position(1)[1] - 2.0).abs() < 0.001);
        assert!((chain.position(2)[1] - 4.0).abs() < 0.001);
        assert!((chain.position(3)[1] - 6.0).abs() < 0.001);
    }

    #[test]
    fn chain_stays_connected() {
        let mut chain = Tail4::new([10.0, 0.0], 2.0);
        for _ in 0..100 {
            chain.step(0.016, 9.8, [10.0, 0.0], [0.0, 0.0]);
        }
        for i in 0..3 {
            let dx = chain.position(i + 1)[0] - chain.position(i)[0];
            let dy = chain.position(i + 1)[1] - chain.position(i)[1];
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(
                (dist - 2.0).abs() < 0.3,
                "link {i} distance {dist} too far from rest 2.0"
            );
        }
    }

    #[test]
    fn chain_responds_to_wind() {
        let mut chain = Tail4::new([10.0, 0.0], 2.0);
        for _ in 0..50 {
            chain.step(0.016, 0.0, [10.0, 0.0], [50.0, 0.0]);
        }
        assert!(chain.position(3)[0] > 10.0);
    }

    #[test]
    fn impulse_moves_link() {
        let mut chain = Tail4::new([10.0, 0.0], 2.0);
        let before = chain.position(2);
        chain.impulse(2, [5.0, 0.0]);
        let after = chain.position(2);
        assert!((after[0] - before[0] - 5.0).abs() < 0.001);
    }

    #[test]
    fn anchor_pinned_during_step() {
        let mut chain = Tail4::new([10.0, 0.0], 2.0);
        chain.step(0.016, 9.8, [20.0, 5.0], [0.0, 0.0]);
        assert!((chain.position(0)[0] - 20.0).abs() < 0.001);
        assert!((chain.position(0)[1] - 5.0).abs() < 0.001);
    }

    #[test]
    fn segment_angle_is_vertical_when_hanging() {
        let chain = Tail4::new([10.0, 0.0], 2.0);
        let angle = chain.segment_angle(0);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 0.01);
    }
}
