#![allow(unused)]

use cgmath::prelude::*;
use cgmath::Vector2;
use ndarray::prelude::*;
use std::cmp::{max, min};
use std::io::Write;
use std::ops::Range;
use wad_gfx::Sprite;
use wad_map::*;

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

use std::collections::HashMap;

pub struct TextureProvider<'a> {
    wad: wad::WadSlice<'a>,
    patch_provider: wad_gfx::EagerPatchProvider<'a>,
    texture_dir: wad_gfx::TextureDirectory<'a>,
    cache: HashMap<wad::EntryId, Vec<u8>>,
}

impl<'a> TextureProvider<'a> {
    pub fn new(wad: wad::WadSlice) -> TextureProvider {
        let pnames = wad
            .by_id(b"PNAMES")
            .unwrap()
            .iter()
            .map(|x| x.to_ascii_uppercase())
            .collect::<Vec<_>>();
        let pnames = wad_gfx::parse_pnames(&pnames);
        let texture_dir = wad.by_id(b"TEXTURE1").unwrap();

        TextureProvider {
            wad: wad.slice(..),
            patch_provider: wad_gfx::EagerPatchProvider::new(wad, pnames),
            texture_dir: wad_gfx::TextureDirectory::new(texture_dir),
            cache: HashMap::new(),
        }
    }

    fn find_texture(&self, id: wad::EntryId) -> Option<wad_gfx::Texture<'a>> {
        for i in 0..self.texture_dir.len() {
            let t = self.texture_dir.texture(i);
            if wad::EntryId::from_bytes(&t.name()) == id {
                return Some(t);
            }
        }

        None
    }

    pub fn texture(&mut self, id: impl Into<wad::EntryId>) -> Sprite {
        let id = id.into();
        let texture = self.find_texture(id).unwrap();
        let patch_provider = &self.patch_provider;

        let data = self
            .cache
            .entry(id)
            .or_insert_with(|| wad_gfx::render_texture(texture, patch_provider));

        Sprite::new(data)
    }
}

pub fn generate_svg(mut out: impl std::fmt::Write, map: &wad_map::Map) -> std::fmt::Result {
    let mut bbox = BoundingBox::from(&map.vertexes);
    bbox.grow(20);

    writeln!(
        out,
        r#"<svg viewBox="{} {} {} {}" xmlns="http://www.w3.org/2000/svg"><g id="map-root" transform="scale(1, -1)">"#,
        bbox.left(),
        -bbox.bottom(),
        bbox.width(),
        bbox.height()
    )?;

    writeln!(
        out,
        r##"
    <marker id="arrowhead"
        markerWidth="10" markerHeight="10"
        refX="5" refY="5"
        orient="auto"
        markerUnits="strokeWidth"
    >
        <path d="M3,3 l3,2 l-3,2" fill="none" stroke="#41f4a9" stroke-linecap="round"/>
    </marker>"##,
    )?;

    for linedef in &map.linedefs {
        let a = &map.vertexes[linedef.a as usize];
        let b = &map.vertexes[linedef.b as usize];

        let portal = linedef.left_sidedef.is_some() && linedef.right_sidedef.is_some();

        let class = if portal { r#" class="portal""# } else { "" };

        writeln!(
            out,
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}"{} />"#,
            a.x, a.y, b.x, b.y, class
        )?;
    }
    writeln!(
        out,
        r#"
    <g class="camera">
        <line class="camera--sightline" />
        <line class="camera--direction" marker-end="url(#arrowhead)" />
        <line class="camera--fov-left" />
        <line class="camera--fov-right" />
        <circle r="32" class="camera--focus" />
        <circle r="32" class="camera--target" />
    </g>
    </g></svg>
    "#
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn texture_provider() {
        let wad = wad::parse_wad(Vec::from(include_bytes!("../doom1.wad") as &[u8])).unwrap();
        let _ = TextureProvider::new(wad.as_slice());
    }

    #[test]
    fn svg() {
        let wad = wad::parse_wad(Vec::from(include_bytes!("../doom1.wad") as &[u8])).unwrap();
        let map = wad_map::read_map(&wad.as_slice(), "E1M1").unwrap();
        let mut buf = String::new();
        let _ = generate_svg(&mut buf, &map);
    }
}
