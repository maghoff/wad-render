use cgmath::prelude::*;
use cgmath::Vector2;
use ndarray::prelude::*;
use std::cmp::{max, min};
use std::ops::Range;
use wad_gfx::Sprite;

pub fn add(r: Range<i32>, d: i32) -> Range<i32> {
    (r.start + d)..(r.end + d)
}

pub fn intersect(a: Range<i32>, b: Range<i32>) -> Range<i32> {
    max(a.start, b.start)..min(a.end, b.end)
}

pub fn is_empty(r: &Range<i32>) -> bool {
    r.end <= r.start
}

pub fn put_sprite(trg: &mut ArrayViewMut2<u8>, pos_x: i16, pos_y: i16, sprite: &Sprite) {
    let (height, width) = trg.dim();

    let (top, left) = sprite.origin();
    let origin = (left as i32, top as i32); // Flip xy

    // Position sprite origin at given coordinates
    let offset = (pos_x as i32 - origin.0, pos_y as i32 - origin.1);

    let x_range = 0..sprite.width() as i32; // Sprite dimension
    let x_range = add(x_range, offset.0); // Position on canvas
    let x_range = intersect(x_range, 0..width as i32); // Clip to canvas

    for x in x_range {
        for span in sprite.col((x - offset.0) as _) {
            let y_offset = offset.1 + span.top as i32;

            let span_range = 0..span.pixels.len() as i32;
            let span_range = add(span_range, y_offset);
            let span_range = intersect(span_range, 0..height as i32);

            for y in span_range {
                trg[[y as usize, x as usize]] = span.pixels[(y - y_offset) as usize];
            }
        }
    }
}

pub fn point(trg: &mut ArrayViewMut2<u8>, p: Vector2<f32>, col: u8) {
    let p: Vector2<i32> = p.cast().unwrap();
    let on_screen = p.x >= 0 && p.x < 320 && p.y >= 0 && p.y < 200;
    if on_screen {
        trg[[p.y as usize, p.x as usize]] = col;
    }
}

pub fn line(trg: &mut ArrayViewMut2<u8>, a: Vector2<f32>, b: Vector2<f32>, col: u8) {
    let d = b - a;
    for i in 0..500 {
        let p = a + i as f32 * d / 500.;
        point(trg, p, col);
    }
}

pub fn fill(trg: &mut ArrayViewMut2<u8>, col: u8) {
    for x in trg.iter_mut() {
        *x = col;
    }
}
