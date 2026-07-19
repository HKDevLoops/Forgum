//! Double-buffered framebuffer.
//!
//! **The fix for BUG-E1**: `Cell` previously derived `PartialEq` including a
//! `dirty: bool` field. Since `clear()` set `dirty=false` and `Cell::new()`
//! set `dirty=true`, every rendered cell was always "different" — the
//! scheduler therefore never saw zero damage and idled at 60 fps for a
//! static cow. The fix is a manual `PartialEq` comparing only `ch, fg, bg,
//! alpha` — and dropping the `dirty` field entirely.
//!
//! **Perf (T1/D1/D3):** damage is tracked *incrementally* inside `set()`,
//! comparing the incoming cell against `front` (the last drawn frame). The
//! bitmap + reused damage list make `compute_damage()` `O(changed cells)`,
//! `O(1)` amortized per cell, and **zero-allocation when the frame is
//! static** (no `HashSet` per frame). `clear()` marks nothing dirty; only
//! subsequent `set()`s against the still-old front do (G1). `swap()` =
//! `mem::swap(back, front)` and resets the trackers.

/// RGBA color with 8-bit channels. Alpha is 0 = transparent, 255 = opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// One cell in the framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub alpha: u8,
}

impl Cell {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ch: ' ',
            fg: Color::WHITE,
            bg: Color::TRANSPARENT,
            alpha: 0,
        }
    }

    #[must_use]
    pub const fn new(ch: char, fg: Color) -> Self {
        Self {
            ch,
            fg,
            bg: Color::TRANSPARENT,
            alpha: 255,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::empty()
    }
}

/// Double-buffered framebuffer. The front buffer is what we last *drew*;
/// the back buffer is what we're building for the *next* frame.
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub back: Vec<Cell>,
    pub front: Vec<Cell>,
    /// Per-cell dirty flag: `true` iff `back[i] != front[i]` since the last
    /// `clear()`/`swap()`. Indexed the same as `back`/`front`.
    dirty: Vec<bool>,
    /// Reused list of `(x, y)` damage cells for the current frame. Emptied on
    /// `clear()`/`swap()`; appended to by `set()`.
    damage_list: Vec<(usize, usize)>,
}

impl FrameBuffer {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let sz = width.saturating_mul(height);
        Self {
            width,
            height,
            back: vec![Cell::empty(); sz],
            front: vec![Cell::empty(); sz],
            dirty: vec![false; sz],
            damage_list: Vec::with_capacity(sz),
        }
    }

    /// Replace the back buffer with empty cells (does *not* touch front).
    ///
    /// Marks **nothing** dirty — only cells written via `set()` afterwards
    /// become damaged (G1). Resets the incremental trackers.
    pub fn clear(&mut self) {
        for cell in &mut self.back {
            *cell = Cell::empty();
        }
        self.dirty.fill(false);
        self.damage_list.clear();
    }

    /// Write a cell into the back buffer at `(x, y)`. Returns `true` if the
    /// cell was within bounds.
    ///
    /// If the new cell differs from `front[i]` (the last drawn frame), the
    /// cell is marked dirty and recorded in the damage list (O(changed),
    /// no allocation when nothing changes).
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let i = y * self.width + x;
        self.back[i] = cell;
        if cell != self.front[i] && !self.dirty[i] {
            self.dirty[i] = true;
            self.damage_list.push((x, y));
        }
        true
    }

    /// Read a cell from the front buffer.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Cell {
        if x >= self.width || y >= self.height {
            return Cell::empty();
        }
        self.front[y * self.width + x]
    }

    /// Read a cell from the **back** buffer (the frame currently being built).
    ///
    /// This is what the renderer should read so the *just-rendered* frame is
    /// drawn rather than the stale last-swapped one (BUG-A stale-frame fix).
    #[must_use]
    pub fn get_back(&self, x: usize, y: usize) -> Cell {
        if x >= self.width || y >= self.height {
            return Cell::empty();
        }
        self.back[y * self.width + x]
    }

    /// Resize. Both buffers are reset to empty and the trackers cleared.
    pub fn resize(&mut self, width: usize, height: usize) {
        let sz = width.saturating_mul(height);
        self.width = width;
        self.height = height;
        self.back = vec![Cell::empty(); sz];
        self.front = vec![Cell::empty(); sz];
        self.dirty = vec![false; sz];
        self.damage_list.clear();
    }

    /// Return the set of `(x, y)` cells that changed in the back buffer since
    /// the last `clear()`/`swap()`. This is **`O(changed cells)`** and
    /// **zero-alloc when the frame is static** (no `HashSet`).
    #[must_use]
    pub fn compute_damage(&self) -> Vec<(usize, usize)> {
        self.damage_list.clone()
    }

    /// Swap buffers: `front` becomes the frame just built in `back`, and `back`
    /// is repopulated as a copy of that displayed frame so the next frame is
    /// built on top of it.
    ///
    /// Damage is measured by comparing each `set()` against `front` (the last
    /// displayed frame). Resetting the trackers here means the next frame's
    /// `set()` calls measure damage against the freshly-displayed front buffer
    /// (G1), and `get_back` returns the displayed content until overwritten —
    /// which is what `render_damage` reads to draw the current (non-stale)
    /// frame (BUG-A).
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.back, &mut self.front);
        self.back.clone_from(&self.front);
        self.dirty.fill(false);
        self.damage_list.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Cell ──────────────────────────────────────────────────────

    #[test]
    fn cell_partial_eq_compares_content_not_identity() {
        let a = Cell::new('x', Color::WHITE);
        let b = Cell::new('x', Color::WHITE);
        assert_eq!(a, b, "identical cells must be equal");
        // Different character
        let c = Cell::new('y', Color::WHITE);
        assert_ne!(a, c, "different chars must not be equal");
        // Different color
        let d = Cell::new('x', Color::BLACK);
        assert_ne!(a, d, "different colors must not be equal");
        // Different alpha
        let e = Cell {
            ch: 'x',
            fg: Color::WHITE,
            bg: Color::TRANSPARENT,
            alpha: 128,
        };
        assert_ne!(a, e, "different alpha must not be equal");
    }

    #[test]
    fn cell_empty_is_transparent() {
        let cell = Cell::empty();
        assert_eq!(cell.ch, ' ');
        assert_eq!(cell.fg, Color::WHITE);
        assert_eq!(cell.bg, Color::TRANSPARENT);
        assert_eq!(cell.alpha, 0);
    }

    #[test]
    fn cell_new_is_opaque() {
        let cell = Cell::new('X', Color::rgb(10, 20, 30));
        assert_eq!(cell.ch, 'X');
        assert_eq!(cell.fg, Color::rgb(10, 20, 30));
        assert_eq!(cell.bg, Color::TRANSPARENT);
        assert_eq!(cell.alpha, 255);
    }

    // ── Color ─────────────────────────────────────────────────────

    #[test]
    fn color_constants_values() {
        assert_eq!(
            Color::BLACK,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            }
        );
        assert_eq!(
            Color::WHITE,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(
            Color::TRANSPARENT,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            }
        );
    }

    #[test]
    fn color_rgb_sets_alpha_255() {
        let c = Color::rgb(100, 150, 200);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_rgba_preserves_alpha() {
        let c = Color::rgba(100, 150, 200, 42);
        assert_eq!(c.a, 42);
    }

    // ── FrameBuffer construction ──────────────────────────────────

    #[test]
    fn framebuffer_zero_size() {
        let fb = FrameBuffer::new(0, 0);
        assert_eq!(fb.width, 0);
        assert_eq!(fb.height, 0);
        assert!(fb.back.is_empty());
        assert!(fb.front.is_empty());
    }

    #[test]
    fn framebuffer_normal_size() {
        let fb = FrameBuffer::new(80, 24);
        assert_eq!(fb.width, 80);
        assert_eq!(fb.height, 24);
        assert_eq!(fb.back.len(), 80 * 24);
        assert_eq!(fb.front.len(), 80 * 24);
        // All cells should be empty
        for cell in &fb.back {
            assert_eq!(*cell, Cell::empty());
        }
    }

    #[test]
    fn framebuffer_1x1() {
        let mut fb = FrameBuffer::new(1, 1);
        assert!(fb.set(0, 0, Cell::new('x', Color::WHITE)));
        assert!(!fb.set(1, 0, Cell::new('x', Color::WHITE)));
        assert!(!fb.set(0, 1, Cell::new('x', Color::WHITE)));
        assert!(!fb.set(1, 1, Cell::new('x', Color::WHITE)));
    }

    // ── set/get ───────────────────────────────────────────────────

    #[test]
    fn set_writes_to_back_buffer() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(2, 1, Cell::new('A', Color::WHITE));
        // Before swap: front is empty, back has the cell
        assert_eq!(fb.get(2, 1).ch, ' ', "front should be empty before swap");
        // After swap: front gets the cell
        fb.swap();
        assert_eq!(
            fb.get(2, 1).ch,
            'A',
            "after swap, front should have the cell"
        );
    }

    #[test]
    fn get_reads_from_front_buffer() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(2, 1, Cell::new('A', Color::WHITE));
        fb.swap();
        // After swap, front has the cell
        assert_eq!(fb.get(2, 1).ch, 'A');
    }

    #[test]
    fn get_back_reads_back_buffer() {
        let mut fb = FrameBuffer::new(4, 4);
        // Set a cell in back, before any swap.
        fb.set(2, 1, Cell::new('A', Color::WHITE));
        // Before swap: back has 'A', front is empty.
        assert_eq!(fb.get_back(2, 1).ch, 'A', "back should have the cell");
        assert_eq!(fb.get(2, 1).ch, ' ', "front should be empty before swap");
        // Swap: back (empty, old front) <-> front (has 'A').
        fb.swap();
        // After swap: both front and back agree.
        assert_eq!(fb.get_back(2, 1).ch, 'A');
        assert_eq!(fb.get(2, 1).ch, 'A');
        // Now write a NEW cell to back without swapping.
        fb.set(0, 0, Cell::new('B', Color::WHITE));
        // back has 'B' at (0,0); front still has 'A' at (2,1) and empty elsewhere.
        assert_eq!(fb.get_back(0, 0).ch, 'B');
        assert_eq!(
            fb.get(0, 0).ch,
            ' ',
            "front must not see the unswapped back change"
        );
        assert_eq!(fb.get_back(2, 1).ch, 'A');
    }

    #[test]
    fn out_of_bounds_set_returns_false() {
        let mut fb = FrameBuffer::new(2, 2);
        assert!(!fb.set(2, 0, Cell::new('x', Color::WHITE)));
        assert!(!fb.set(0, 2, Cell::new('x', Color::WHITE)));
        assert!(!fb.set(99, 99, Cell::new('x', Color::WHITE)));
        // In bounds
        assert!(fb.set(0, 0, Cell::new('x', Color::WHITE)));
        assert!(fb.set(1, 1, Cell::new('x', Color::WHITE)));
    }

    #[test]
    fn out_of_bounds_get_returns_empty() {
        let fb = FrameBuffer::new(2, 2);
        assert_eq!(fb.get(2, 0), Cell::empty());
        assert_eq!(fb.get(0, 2), Cell::empty());
        assert_eq!(fb.get(99, 99), Cell::empty());
    }

    // ── swap ──────────────────────────────────────────────────────

    #[test]
    fn swap_exchanges_buffers() {
        let mut fb = FrameBuffer::new(2, 1);
        fb.set(0, 0, Cell::new('A', Color::WHITE));
        fb.swap();
        assert_eq!(
            fb.get(0, 0).ch,
            'A',
            "after swap, front should have the cell"
        );
        // Copy-swap: `back` mirrors the displayed frame so the next build
        // starts from it; it is NOT empty.
        assert_eq!(fb.back[0].ch, 'A', "after swap, back mirrors front");
    }

    #[test]
    fn double_swap_restores() {
        let mut fb = FrameBuffer::new(2, 1);
        fb.set(0, 0, Cell::new('A', Color::WHITE));
        fb.swap();
        fb.swap();
        // After two swaps, back should have original content
        assert_eq!(fb.back[0].ch, 'A');
    }

    // ── clear ─────────────────────────────────────────────────────

    #[test]
    fn clear_empties_back_buffer() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.set(3, 3, Cell::new('b', Color::BLACK));
        fb.clear();
        for cell in &fb.back {
            assert_eq!(*cell, Cell::empty(), "cleared cell must be empty");
        }
    }

    #[test]
    fn clear_does_not_affect_front() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        // Front still has 'a'
        assert_eq!(fb.get(0, 0).ch, 'a');
    }

    #[test]
    fn clear_marks_nothing_dirty() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(1, 1, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        // clear() must not record damage on its own; only later sets do.
        assert!(
            fb.compute_damage().is_empty(),
            "clear() must mark nothing dirty"
        );
    }

    // ── resize ────────────────────────────────────────────────────

    #[test]
    fn resize_resets_both_buffers() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.set(1, 1, Cell::new('b', Color::WHITE));
        fb.swap();
        fb.resize(4, 4);
        assert_eq!(fb.width, 4);
        assert_eq!(fb.height, 4);
        assert_eq!(fb.back.len(), 16);
        assert_eq!(fb.front.len(), 16);
        // Both buffers should be empty
        for cell in &fb.back {
            assert_eq!(*cell, Cell::empty());
        }
        for cell in &fb.front {
            assert_eq!(*cell, Cell::empty());
        }
    }

    #[test]
    fn resize_to_smaller_invalidates_old_positions() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(3, 3, Cell::new('x', Color::WHITE));
        fb.swap();
        fb.resize(2, 2);
        // Old position (3,3) is now out of bounds
        assert_eq!(fb.get(3, 3), Cell::empty());
        assert!(!fb.set(3, 3, Cell::new('y', Color::WHITE)));
    }

    #[test]
    fn resize_to_same_size_works() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.resize(4, 4);
        assert_eq!(fb.width, 4);
        assert_eq!(fb.height, 4);
        // After resize, buffers are reset
        assert_eq!(fb.get(0, 0), Cell::empty());
    }

    // ── compute_damage (incremental, O(changed), no HashSet) ──────

    #[test]
    fn damage_empty_buffers() {
        let fb = FrameBuffer::new(3, 3);
        assert!(fb.compute_damage().is_empty());
    }

    #[test]
    fn damage_after_set_and_swap() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(1, 1, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        // clear() marks nothing dirty; overwriting the displayed cell with a
        // *different* value produces damage (G1: only subsequent sets do).
        fb.set(1, 1, Cell::new('b', Color::BLACK));
        let dmg = fb.compute_damage();
        assert!(dmg.contains(&(1, 1)));
        assert_eq!(dmg.len(), 1);
    }

    #[test]
    fn damage_same_content_no_damage() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(1, 1, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        fb.set(1, 1, Cell::new('a', Color::WHITE));
        let dmg = fb.compute_damage();
        assert!(
            dmg.is_empty(),
            "same content should produce no damage, got {dmg:?}"
        );
    }

    #[test]
    fn damage_different_color_detected() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        fb.set(0, 0, Cell::new('a', Color::BLACK));
        let dmg = fb.compute_damage();
        assert!(dmg.contains(&(0, 0)));
    }

    #[test]
    fn damage_all_cells_changed() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.set(1, 0, Cell::new('b', Color::WHITE));
        fb.set(0, 1, Cell::new('c', Color::WHITE));
        fb.set(1, 1, Cell::new('d', Color::WHITE));
        fb.swap();
        fb.clear();
        fb.set(0, 0, Cell::new('w', Color::BLACK));
        fb.set(1, 0, Cell::new('x', Color::BLACK));
        fb.set(0, 1, Cell::new('y', Color::BLACK));
        fb.set(1, 1, Cell::new('z', Color::BLACK));
        let dmg = fb.compute_damage();
        assert_eq!(dmg.len(), 4, "all 4 cells should be damaged");
        assert!(dmg.contains(&(0, 0)));
        assert!(dmg.contains(&(1, 0)));
        assert!(dmg.contains(&(0, 1)));
        assert!(dmg.contains(&(1, 1)));
    }

    #[test]
    fn damage_partial_change() {
        let mut fb = FrameBuffer::new(3, 3);
        // Fill all 9 cells in back, then swap so front = all 'a'
        for y in 0..3 {
            for x in 0..3 {
                fb.set(x, y, Cell::new('a', Color::WHITE));
            }
        }
        fb.swap();
        // Now back is empty (old front), front has all 'a'
        // Re-fill back with all 'a' (matching front), then change only (1,1)
        for y in 0..3 {
            for x in 0..3 {
                fb.set(x, y, Cell::new('a', Color::WHITE));
            }
        }
        fb.set(1, 1, Cell::new('b', Color::WHITE));
        let dmg = fb.compute_damage();
        assert_eq!(dmg.len(), 1, "only one cell changed, got {}", dmg.len());
        assert!(dmg.contains(&(1, 1)));
    }

    #[test]
    fn damage_not_duplicated_on_repeat_set() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(1, 1, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        // Multiple identical sets on the same dirty cell must record it once.
        fb.set(1, 1, Cell::new('b', Color::BLACK));
        fb.set(1, 1, Cell::new('b', Color::BLACK));
        fb.set(1, 1, Cell::new('c', Color::WHITE));
        let dmg = fb.compute_damage();
        assert_eq!(dmg.len(), 1, "cell recorded once, got {}", dmg.len());
        assert!(dmg.contains(&(1, 1)));
    }

    #[test]
    fn damage_reset_by_swap() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.swap();
        fb.clear();
        fb.set(0, 0, Cell::new('b', Color::BLACK));
        assert_eq!(fb.compute_damage().len(), 1);
        // swap resets the tracker; next frame starts clean.
        fb.swap();
        assert!(fb.compute_damage().is_empty());
    }

    // ── BUG-E1 invariant: identical cells → 0 damage ──────────────

    #[test]
    fn bug_e1_regression_identical_cells_zero_damage() {
        let mut fb = FrameBuffer::new(80, 24);
        // Fill back with a pattern
        for y in 0..24 {
            for x in 0..80 {
                let ch = ((x + y) % 26) as u8 + b'a';
                fb.set(x, y, Cell::new(ch as char, Color::WHITE));
            }
        }
        fb.swap();
        fb.clear();
        // Re-fill with the exact same pattern
        for y in 0..24 {
            for x in 0..80 {
                let ch = ((x + y) % 26) as u8 + b'a';
                fb.set(x, y, Cell::new(ch as char, Color::WHITE));
            }
        }
        let dmg = fb.compute_damage();
        assert!(
            dmg.is_empty(),
            "BUG-E1 regression: identical content should produce 0 damage, got {}",
            dmg.len()
        );
    }

    #[test]
    fn bug_e1_regression_single_cell_change() {
        let mut fb = FrameBuffer::new(80, 24);
        for y in 0..24 {
            for x in 0..80 {
                fb.set(x, y, Cell::new('x', Color::WHITE));
            }
        }
        fb.swap();
        fb.clear();
        for y in 0..24 {
            for x in 0..80 {
                fb.set(x, y, Cell::new('x', Color::WHITE));
            }
        }
        // Change one cell
        fb.set(40, 12, Cell::new('y', Color::WHITE));
        let dmg = fb.compute_damage();
        assert_eq!(dmg.len(), 1, "only one cell changed");
        assert!(dmg.contains(&(40, 12)));
    }

    // ── Clone ─────────────────────────────────────────────────────

    #[test]
    fn framebuffer_clone_independent() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(0, 0, Cell::new('A', Color::WHITE));
        fb.swap();
        let mut clone = fb.clone();
        clone.set(0, 0, Cell::new('B', Color::BLACK));
        // Original unchanged — front still has 'A'
        assert_eq!(fb.get(0, 0).ch, 'A');
        // Clone's front also has 'A' (set writes to back, get reads front)
        assert_eq!(clone.get(0, 0).ch, 'A');
        // But clone's back has 'B'
        assert_eq!(clone.back[0].ch, 'B');
    }
}
