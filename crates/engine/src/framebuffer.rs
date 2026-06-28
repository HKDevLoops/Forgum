//! Double-buffered framebuffer.
//!
//! **The fix for BUG-E1**: `Cell` previously derived `PartialEq` including a
//! `dirty: bool` field. Since `clear()` set `dirty=false` and `Cell::new()`
//! set `dirty=true`, every rendered cell was always "different" — the
//! scheduler therefore never saw zero damage and idled at 60 fps for a
//! static cow. The fix is a manual `PartialEq` comparing only `ch, fg, bg,
//! alpha` — and dropping the `dirty` field entirely.

use std::collections::HashSet;

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
#[derive(Debug)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub back: Vec<Cell>,
    pub front: Vec<Cell>,
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
        }
    }

    /// Replace the back buffer with empty cells (does *not* touch front).
    pub fn clear(&mut self) {
        for cell in &mut self.back {
            *cell = Cell::empty();
        }
    }

    /// Write a cell into the back buffer at `(x, y)`. Returns `true` if the
    /// cell was within bounds.
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.back[y * self.width + x] = cell;
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

    /// Resize. Both buffers are reset to empty.
    pub fn resize(&mut self, width: usize, height: usize) {
        let sz = width.saturating_mul(height);
        self.width = width;
        self.height = height;
        self.back = vec![Cell::empty(); sz];
        self.front = vec![Cell::empty(); sz];
    }

    /// Compute the set of (x, y) cells where the back buffer differs from
    /// the front buffer. Returns an empty set when the buffers are identical
    /// (BUG-E1 invariant).
    #[must_use]
    pub fn compute_damage(&self) -> HashSet<(usize, usize)> {
        let mut dmg = HashSet::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;
                if self.back[i] != self.front[i] {
                    dmg.insert((x, y));
                }
            }
        }
        dmg
    }

    /// Swap buffers (back becomes the new front).
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.back, &mut self.front);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_partial_eq_ignores_dirty() {
        // The fix for BUG-E1: two cells with the same ch/fg/bg/alpha compare
        // equal, regardless of any internal bookkeeping.
        let a = Cell::new('x', Color::WHITE);
        let b = Cell::new('x', Color::WHITE);
        assert_eq!(a, b);
        let c = Cell::new('y', Color::WHITE);
        assert_ne!(a, c);
    }

    #[test]
    fn framebuffer_damage_is_empty_when_buffers_match() {
        let mut fb = FrameBuffer::new(4, 4);
        fb.set(1, 1, Cell::new('a', Color::WHITE));
        fb.swap(); // commit
        fb.clear(); // back is now empty; front has the 'a'
        assert!(!fb.compute_damage().is_empty());
        fb.set(1, 1, Cell::new('a', Color::WHITE)); // restore
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
    fn out_of_bounds_set_is_safe() {
        let mut fb = FrameBuffer::new(2, 2);
        assert!(!fb.set(99, 99, Cell::new('x', Color::WHITE)));
        assert!(!fb.set(2, 0, Cell::new('x', Color::WHITE)));
        assert!(fb.set(0, 0, Cell::new('x', Color::WHITE)));
    }

    #[test]
    fn resize_resets_buffers() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set(0, 0, Cell::new('a', Color::WHITE));
        fb.resize(4, 4);
        assert_eq!(fb.width, 4);
        assert_eq!(fb.height, 4);
        assert!(fb.compute_damage().is_empty());
    }
}
