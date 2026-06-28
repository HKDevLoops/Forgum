//! Effects module — Phase 0 stub.
//!
//! Phase 0 only needs to prove that the render loop, guards, and damage
//! tracking all work. Real effects (aurora, plasma, particle pool, etc.)
//! land in Phase 3.
//!
//! For now, `render_static_cow` writes the literal cow text into the
//! framebuffer at row 0, leaving everything else empty.

use crate::framebuffer::{Cell, Color, FrameBuffer};

/// Write the cow text into `fb` at row 0, columns 0..text.len().
///
/// Phase 0: we use a single-color ASCII rendition. The exact cow art comes
/// in Phase 2 (cow file loading + speech bubble).
pub fn render_static_cow(fb: &mut FrameBuffer, cow_text: &str) {
    let fg = Color::WHITE;
    let mut x = 0usize;
    let mut y = 0usize;
    for ch in cow_text.chars() {
        if x >= fb.width {
            // Wrap to next row.
            x = 0;
            y = y.saturating_add(1);
        }
        if y >= fb.height {
            break;
        }
        if ch == '\n' {
            x = 0;
            y = y.saturating_add(1);
            continue;
        }
        let _ = fb.set(x, y, Cell::new(ch, fg));
        x = x.saturating_add(1);
    }
}

/// Returns the canonical "default" cow art for Phase 0. Replaced by a
/// proper cow-file parser in Phase 2.
#[must_use]
pub fn default_cow_text() -> &'static str {
    r#"        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_cow_does_not_overflow() {
        let mut fb = FrameBuffer::new(40, 10);
        render_static_cow(&mut fb, default_cow_text());
        // The render loop swaps after committing, so for test purposes we
        // mirror that: swap, then read from front.
        fb.swap();
        assert_eq!(fb.get(8, 0).ch, '\\');
        assert_eq!(fb.get(12, 0).ch, '^');
        assert_eq!(fb.get(15, 0).ch, '^');
        // Col 0 of subsequent rows should remain empty.
        assert_eq!(fb.get(0, 1).ch, ' ');
    }

    #[test]
    fn empty_text_is_safe() {
        let mut fb = FrameBuffer::new(10, 10);
        render_static_cow(&mut fb, "");
        assert!(fb.compute_damage().is_empty());
    }
}
