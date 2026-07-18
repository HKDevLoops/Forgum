use crate::framebuffer::{Cell, Color, FrameBuffer};

pub trait Shader {
    fn apply(&self, fb: &mut FrameBuffer);
}

#[derive(Debug)]
pub struct CrtShader {
    pub scanline_intensity: f32,
    pub bloom_radius: u32,
}

impl Default for CrtShader {
    fn default() -> Self {
        Self {
            scanline_intensity: 0.15,
            bloom_radius: 1,
        }
    }
}

impl Shader for CrtShader {
    fn apply(&self, fb: &mut FrameBuffer) {
        for y in 0..fb.height {
            if y % 2 == 1 {
                for x in 0..fb.width {
                    let cell = fb.get(x, y);
                    let (r, g, b) = (cell.fg.r, cell.fg.g, cell.fg.b);
                    let factor = 1.0 - self.scanline_intensity;
                    let new_cell = Cell::new(
                        cell.ch,
                        Color::rgb(
                            (r as f32 * factor) as u8,
                            (g as f32 * factor) as u8,
                            (b as f32 * factor) as u8,
                        ),
                    );
                    fb.set(x, y, new_cell);
                }
            }
        }

        if self.bloom_radius > 0 {
            apply_bloom(fb, self.bloom_radius);
        }
    }
}

fn apply_bloom(fb: &mut FrameBuffer, _radius: u32) {
    for y in 0..fb.height {
        for x in 0..fb.width {
            let cell = fb.get(x, y);
            let (r, g, b) = (cell.fg.r, cell.fg.g, cell.fg.b);
            if r > 200 && g > 200 && b > 200 {
                for dy in 0i32..=1 {
                    for dx in 0i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0
                            && ny >= 0
                            && (nx as usize) < fb.width
                            && (ny as usize) < fb.height
                        {
                            let neighbor = fb.get(nx as usize, ny as usize);
                            let (nr, ng, nb) = (neighbor.fg.r, neighbor.fg.g, neighbor.fg.b);
                            fb.set(
                                nx as usize,
                                ny as usize,
                                Cell::new(
                                    neighbor.ch,
                                    Color::rgb(
                                        nr.saturating_add(30),
                                        ng.saturating_add(30),
                                        nb.saturating_add(30),
                                    ),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct NoopShader;

impl Shader for NoopShader {
    fn apply(&self, _fb: &mut FrameBuffer) {}
}

pub fn create_shader(name: &str) -> Box<dyn Shader> {
    match name {
        "crt" => Box::new(CrtShader::default()),
        _ => Box::new(NoopShader),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crt_scanline_dimmers_odd_rows() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(2, 1, Cell::new('X', Color::rgb(255, 255, 255)));
        let shader = CrtShader {
            scanline_intensity: 0.15,
            bloom_radius: 0,
        };
        shader.apply(&mut fb);
        fb.swap();
        let cell = fb.get(2, 1);
        assert!(cell.fg.r < 255);
        assert!(cell.fg.g < 255);
        assert!(cell.fg.b < 255);
    }

    #[test]
    fn crt_even_rows_unchanged() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(2, 0, Cell::new('X', Color::rgb(255, 255, 255)));
        fb.swap();
        let original = fb.get(2, 0);
        // Re-set on back for shader to process
        fb.set(2, 0, Cell::new('X', Color::rgb(255, 255, 255)));
        let shader = CrtShader {
            scanline_intensity: 0.15,
            bloom_radius: 0,
        };
        shader.apply(&mut fb);
        fb.swap();
        let after = fb.get(2, 0);
        assert_eq!(after.ch, original.ch, "even row char should be unchanged");
        assert_eq!(after.fg, original.fg, "even row color should be unchanged");
    }

    #[test]
    fn crt_bloom_increases_neighbor_brightness() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(0, 0, Cell::new('X', Color::rgb(255, 255, 255)));
        let shader = CrtShader::default();
        shader.apply(&mut fb);
        fb.swap();
        let neighbor = fb.get(1, 1);
        assert!(neighbor.fg.r > 30);
        assert!(neighbor.fg.g > 30);
        assert!(neighbor.fg.b > 30);
    }

    #[test]
    fn noop_shader_preserves_content() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(0, 0, Cell::new('X', Color::rgb(255, 0, 0)));
        let original = fb.get(0, 0);
        NoopShader.apply(&mut fb);
        assert_eq!(fb.get(0, 0), original);
    }

    #[test]
    fn create_shader_crt_applies_effect() {
        let s = create_shader("crt");
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(2, 1, Cell::new('X', Color::rgb(200, 200, 200)));
        let before = fb.back.clone();
        s.apply(&mut fb);
        assert_ne!(fb.back, before, "CRT shader should modify the framebuffer");
    }

    #[test]
    fn create_shader_unknown_returns_noop() {
        let n = create_shader("unknown");
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(0, 0, Cell::new('X', Color::rgb(255, 0, 0)));
        let original = fb.get(0, 0);
        n.apply(&mut fb);
        assert_eq!(fb.get(0, 0), original);
    }

    #[test]
    fn crt_scanline_intensity_configurable() {
        let mut fb_default = FrameBuffer::new(10, 5);
        let mut fb_strong = FrameBuffer::new(10, 5);
        fb_default.set(2, 1, Cell::new('X', Color::rgb(200, 200, 200)));
        fb_strong.set(2, 1, Cell::new('X', Color::rgb(200, 200, 200)));

        CrtShader {
            scanline_intensity: 0.15,
            bloom_radius: 0,
        }
        .apply(&mut fb_default);
        CrtShader {
            scanline_intensity: 0.5,
            bloom_radius: 0,
        }
        .apply(&mut fb_strong);

        fb_default.swap();
        fb_strong.swap();

        let cell_default = fb_default.get(2, 1);
        let cell_strong = fb_strong.get(2, 1);
        assert!(cell_strong.fg.r < cell_default.fg.r);
    }

    #[test]
    fn crt_scanline_zero_intensity_odd_rows_unchanged() {
        let mut fb = FrameBuffer::new(10, 5);
        let original = Cell::new('A', Color::rgb(100, 150, 200));
        fb.set(3, 1, original);
        fb.swap();
        let original = fb.get(3, 1);
        fb.set(3, 1, original);
        let shader = CrtShader {
            scanline_intensity: 0.0,
            bloom_radius: 0,
        };
        shader.apply(&mut fb);
        fb.swap();
        let cell = fb.get(3, 1);
        assert_eq!(
            cell.ch, original.ch,
            "char should be unchanged at zero intensity"
        );
        assert_eq!(
            cell.fg, original.fg,
            "color should be unchanged at zero intensity"
        );
    }

    #[test]
    fn crt_scanline_full_intensity_odd_rows_zeroed_fg() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(5, 1, Cell::new('B', Color::rgb(180, 180, 180)));
        let shader = CrtShader {
            scanline_intensity: 1.0,
            bloom_radius: 0,
        };
        shader.apply(&mut fb);
        fb.swap();
        let cell = fb.get(5, 1);
        assert_eq!(cell.fg.r, 0, "red should be zero at full intensity");
        assert_eq!(cell.fg.g, 0, "green should be zero at full intensity");
        assert_eq!(cell.fg.b, 0, "blue should be zero at full intensity");
    }

    #[test]
    fn noop_shader_does_not_modify_framebuffer() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(0, 0, Cell::new('C', Color::rgb(50, 100, 150)));
        fb.set(4, 3, Cell::new('D', Color::rgb(200, 100, 50)));
        let before_back = fb.back.clone();
        let noop = NoopShader;
        noop.apply(&mut fb);
        assert_eq!(
            fb.back, before_back,
            "NoopShader should not modify framebuffer"
        );
    }
}
