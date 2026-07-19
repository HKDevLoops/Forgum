use criterion::{criterion_group, criterion_main, Criterion};
use forgum_engine::framebuffer::{Cell, Color, FrameBuffer};
use forgum_engine::renderer::{AnsiRenderer, Renderer};

fn make_cell(ch: char) -> Cell {
    Cell::new(ch, Color::WHITE)
}

fn bench_damage_diff(c: &mut Criterion) {
    let mut fb = FrameBuffer::new(80, 24);
    for y in 0..24 {
        for x in 0..80 {
            if (x + y) % 2 == 0 {
                fb.set(x, y, make_cell('@'));
            }
        }
    }
    c.bench_function("compute_damage_50pct", |b| {
        b.iter(|| {
            let d = fb.compute_damage();
            criterion::black_box(d.len())
        })
    });
}

fn bench_render_region(c: &mut Criterion) {
    let mut fb = FrameBuffer::new(80, 24);
    for y in 0..24 {
        for x in 0..80 {
            fb.set(x, y, make_cell('@'));
        }
    }
    let dmg: Vec<(usize, usize)> = fb.compute_damage();
    let mut out = Vec::new();
    let mut r = AnsiRenderer::default();
    c.bench_function("render_damage_full", |b| {
        b.iter(|| {
            out.clear();
            r.render_damage(&mut out, &fb, &dmg).unwrap();
            criterion::black_box(out.len())
        })
    });
}

criterion_group!(benches, bench_damage_diff, bench_render_region);
criterion_main!(benches);
