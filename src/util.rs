use std::cmp::{max, min};
use std::ops::Range;
use ndarray::prelude::*;
use wad_gfx::Sprite;

pub fn add(r: Range<i32>, d: i32) -> Range<i32> {
    (r.start + d)..(r.end + d)
}

pub fn intersect(a: Range<i32>, b: Range<i32>) -> Range<i32> {
    max(a.start, b.start)..min(a.end, b.end)
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
