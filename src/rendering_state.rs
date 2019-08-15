use crate::util::*;
use array_macro::array;
use cgmath::prelude::*;
use cgmath::{vec2, vec3, Vector2, Vector3};
use ndarray::prelude::*;
use std::ops::Range;
use wad_gfx::Sprite;

const TAU: f32 = 2. * ::std::f32::consts::PI;
const PROJECTION_PLANE_WIDTH: f32 = 320.;
const FOV: f32 = 90. * TAU / 360.;
const PROJECTION_PLANE_HALF_WIDTH: f32 = PROJECTION_PLANE_WIDTH / 2.;

pub struct RenderingState<'a> {
    distance_to_projection_plane: f32,
    framebuffer: &'a mut ArrayViewMut2<'a, u8>,
    h_open: Vec<Range<i32>>,
    v_open: [Range<i32>; 320],
}

impl<'a> RenderingState<'a> {
    pub fn new(framebuffer: &'a mut ArrayViewMut2<'a, u8>) -> RenderingState<'a> {
        RenderingState {
            distance_to_projection_plane: PROJECTION_PLANE_HALF_WIDTH / (FOV / 2.).tan(),
            framebuffer,
            h_open: vec![0..320],
            v_open: array![0..200; 320],
        }
    }

    fn project(&self, p: Vector3<f32>) -> Vector2<f32> {
        let w = 1. / p.z;

        vec2(
            160. + self.distance_to_projection_plane * p.x * w,
            100. - self.distance_to_projection_plane * p.y * w,
        )
    }

    pub fn is_complete(&self) -> bool {
        self.h_open.is_empty()
    }

    fn apply_horizontal_clipping(&mut self, r: Range<i32>) -> Vec<Range<i32>> {
        let mut to_render = vec![];
        let mut clipped = vec![];

        for c in self.h_open.drain(..).into_iter() {
            let i = intersect(c.clone(), r.clone());

            if is_empty(&i) {
                clipped.push(c);
            } else {
                if c.start < i.start {
                    clipped.push(c.start..i.start);
                }
                if i.end < c.end {
                    clipped.push(i.end..c.end);
                }

                to_render.push(i);
            }
        }

        self.h_open = clipped;
        to_render
    }

    fn horizontally_clip(&self, r: Range<i32>) -> Vec<Range<i32>> {
        let mut to_render = vec![];

        for c in self.h_open.iter() {
            let i = intersect(c.clone(), r.clone());

            if !is_empty(&i) {
                to_render.push(i);
            }
        }

        to_render
    }

    fn clip_near(
        a: Vector2<f32>,
        b: Vector2<f32>,
    ) -> Option<(Vector2<f32>, f32, Vector2<f32>, f32)> {
        const CLIP_NEAR: f32 = 10.;

        if a.y <= CLIP_NEAR && b.y <= CLIP_NEAR {
            return None;
        }

        let d = b - a;
        let len = d.magnitude();

        // a.y + i * d.y = CLIP_NEAR
        // i * d.y = CLIP_NEAR - a.y
        // i = (CLIP_NEAR - a.y) / d.y
        let intersection_u = (CLIP_NEAR - a.y) / d.y;
        let intersection_p = a + d * intersection_u;

        if a.y < CLIP_NEAR {
            return Some((intersection_p, intersection_u * len, b, len));
        } else if b.y < CLIP_NEAR {
            return Some((a, 0., intersection_p, intersection_u * len));
        }

        Some((a, 0., b, len))
    }

    fn draw_texture_col(
        &mut self,
        x: i32,
        top: f32,
        bottom: i32,
        scale: f32,
        texture: &Sprite,
        column: u32,
    ) {
        if bottom <= top as i32 {
            return;
        }

        for span in texture.col(column) {
            let span_y_top = top + span.top as f32 * scale;
            let span_y_bottom = top + scale * (span.top as u32 + span.pixels.len() as u32) as f32;
            let dy = span_y_bottom - span_y_top;

            let y_range =
                span_y_top.round() as i32..std::cmp::min(span_y_bottom.round() as i32, bottom);

            // Vertical clipping
            // let y_range = intersect(y_range, 0..200); // Redundant
            let y_range = intersect(y_range, self.v_open[x as usize].clone());

            for y in y_range {
                let s = (y as f32 - span_y_top) / dy * span.pixels.len() as f32;
                self.framebuffer[[y as usize, x as usize]] = span.pixels[s as usize];
            }
        }

        let rendered_to = top + texture.height() as f32 * scale;
        if bottom >= rendered_to as i32 {
            self.draw_texture_col(x, rendered_to, bottom, scale, texture, column);
        }
    }

    pub fn wall(
        &mut self,
        floor: f32,
        ceil: f32,
        a: Vector2<f32>,
        b: Vector2<f32>,
        texture: &Sprite,
    ) {
        let (a, ua, b, ub) = match Self::clip_near(a, b) {
            None => return,
            Some(x) => x,
        };

        let za = a.y;
        let zb = b.y;

        let fa = self.project(vec3(a.x, floor, a.y));
        let ca = self.project(vec3(a.x, ceil, a.y));
        let fb = self.project(vec3(b.x, floor, b.y));
        let cb = self.project(vec3(b.x, ceil, b.y));

        let d_ceil = cb - ca;
        let d_floor = fb - fa;

        let scale_a = self.distance_to_projection_plane / a.y;
        let scale_b = self.distance_to_projection_plane / b.y;
        let dscale = scale_b - scale_a;

        let x_range = fa.x.round() as i32..fb.x.round() as i32;
        // let x_ranges = vec![intersect(x_range, 0..320)];
        let x_ranges = self.apply_horizontal_clipping(x_range);

        for x in x_ranges.into_iter().flatten() {
            let t = (x as f32 - fa.x) / d_floor.x;

            let top = ca.y + d_ceil.y * t;
            let bottom = fa.y + d_floor.y * t;

            // Perspective correct interpolation of u coordinate
            // TODO: Derive from fundamental geometry
            let u = ((1. - t) * ua / za + t * ub / zb) / ((1. - t) / za + t / zb);

            let u = (u.round() as i32).rem_euclid(texture.width() as i32);

            self.draw_texture_col(x, top, bottom as _, scale_a + dscale * t, texture, u as u32);

            // TODO Yield visplanes
        }
    }

    pub fn portal(
        &mut self,
        floor: f32,
        ceil: f32,
        a: Vector2<f32>,
        b: Vector2<f32>,
        upper: &Option<(f32, f32, Sprite)>,
        lower: &Option<(f32, f32, Sprite)>,
    ) {
        let (a, ua, b, ub) = match Self::clip_near(a, b) {
            None => return,
            Some(x) => x,
        };

        let za = a.y;
        let zb = b.y;

        let fa = self.project(vec3(a.x, floor, a.y));
        let ca = self.project(vec3(a.x, ceil, a.y));
        let fb = self.project(vec3(b.x, floor, b.y));
        let cb = self.project(vec3(b.x, ceil, b.y));

        let d_ceil = cb - ca;
        let d_floor = fb - fa;

        let scale_a = self.distance_to_projection_plane / a.y;
        let scale_b = self.distance_to_projection_plane / b.y;
        let dscale = scale_b - scale_a;

        let x_range = fa.x.round() as i32..fb.x.round() as i32;
        // let x_ranges = vec![intersect(x_range, 0..320)];
        let x_ranges = self.horizontally_clip(x_range);

        for x in x_ranges.into_iter().flatten() {
            let t = (x as f32 - fa.x) / d_floor.x;

            let top = ca.y + d_ceil.y * t;
            let bottom = fa.y + d_floor.y * t;
            let scale = scale_a + dscale * t;

            let u = ((1. - t) * ua / za + t * ub / zb) / ((1. - t) / za + t / zb);

            if let Some((top, bottom, texture)) = upper {
                let u = (u.round() as i32).rem_euclid(texture.width() as i32);
                self.draw_texture_col(
                    x,
                    100. - top * scale,
                    (100. - bottom * scale) as _,
                    scale,
                    texture,
                    u as u32,
                );
            }

            if let Some((top, bottom, texture)) = lower {
                let u = (u.round() as i32).rem_euclid(texture.width() as i32);
                self.draw_texture_col(
                    x,
                    100. - top * scale,
                    (100. - bottom * scale) as _,
                    scale,
                    texture,
                    u as u32,
                );
            }

            // TODO Yield visplanes

            self.v_open[x as usize] =
                intersect(self.v_open[x as usize].clone(), top as _..bottom as _);
        }
    }
}
