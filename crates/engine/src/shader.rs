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
    fn crt_does_not_panic() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.set(0, 0, Cell::new('X', Color::rgb(255, 255, 255)));
        let shader = CrtShader::default();
        shader.apply(&mut fb);
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
    fn create_shader_returns_correct_types() {
        let s = create_shader("crt");
        let mut fb = FrameBuffer::new(4, 4);
        s.apply(&mut fb);

        let n = create_shader("unknown");
        let mut fb2 = FrameBuffer::new(4, 4);
        n.apply(&mut fb2);
    }
}
