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

    pub fn wall(
        &mut self,
        floor: f32,
        ceil: f32,
        a: Vector2<f32>,
        b: Vector2<f32>,
        texture: &Sprite,
    ) {
        let mut fa = vec3(a.x, floor, a.y);
        let mut ca = vec3(a.x, ceil, a.y);
        let mut fb = vec3(b.x, floor, b.y);
        let mut cb = vec3(b.x, ceil, b.y);

        const CLIP_NEAR: f32 = 10.;

        if fa.z <= CLIP_NEAR && fb.z <= CLIP_NEAR {
            return;
        }

        let mut ua = 0.;
        let mut ub = ua + (b - a).magnitude();

        if fa.z < CLIP_NEAR {
            let d = fb - fa;
            let u = (CLIP_NEAR - fa.z) / d.z;

            let x = fa.x + u * d.x;
            fa.x = x;
            ca.x = x;

            fa.z = CLIP_NEAR;
            ca.z = CLIP_NEAR;

            ua = ua + (ub - ua) * u;
        }

        if fb.z < CLIP_NEAR {
            let d = fa - fb;
            let u = (CLIP_NEAR - fb.z) / d.z;

            let x = fb.x + u * d.x;
            fb.x = x;
            cb.x = x;

            fb.z = CLIP_NEAR;
            cb.z = CLIP_NEAR;

            ub = ub + (ua - ub) * u;
        }

        let za = fa.z;
        let zb = fb.z;

        let fa = self.project(fa);
        let ca = self.project(ca);
        let fb = self.project(fb);
        let cb = self.project(cb);

        let d_ceil = cb - ca;
        let d_floor = fb - fa;

        let v_top = 0.;
        let v_bottom = ceil - floor;

        let x_range = fa.x.round() as i32..fb.x.round() as i32;
        // let x_ranges = vec![intersect(x_range, 0..320)];
        let x_ranges = self.apply_horizontal_clipping(x_range);

        for x in x_ranges.into_iter().flatten() {
            let t = (x as f32 - fa.x) / d_floor.x;

            let top = ca.y + d_ceil.y * t;
            let bottom = fa.y + d_floor.y * t;
            let height = bottom - top;

            // Perspective correct interpolation of u coordinate
            // TODO: Derive from fundamental geometry
            let u = ((1. - t) * ua / za + t * ub / zb) / ((1. - t) / za + t / zb);

            let u = (u.round() as i32).rem_euclid(texture.width() as i32);

            // HITCH! Transparency does not make sense in the first rendering pass!
            // Solid walls must be treated differently from transparent walls!
            for span in texture.col(u as u32) {
                // Revisit. Clean up. FIXME: Does not work with different v ranges
                let span_y_top = top + height * (span.top as f32 / texture.height() as f32);
                let span_y_bottom = top
                    + height
                        * ((span.top as u32 + span.pixels.len() as u32) as f32
                            / texture.height() as f32);
                let dy = span_y_bottom - span_y_top;

                let y_range = span_y_top.round() as i32..span_y_bottom.round() as i32;

                // Vertical clipping
                // let y_range = intersect(y_range, 0..200); // Redundant
                let y_range = intersect(y_range, self.v_open[x as usize].clone());

                for y in y_range {
                    let s = (y as f32 - span_y_top) / dy * span.pixels.len() as f32;
                    self.framebuffer[[y as usize, x as usize]] = span.pixels[s as usize];
                }
            }

            // TODO Yield visplanes

            // Vertical clipping. Walls (not portals) always cover the full height.
            self.v_open[x as usize] = 0..0;
        }
    }

    pub fn portal(
        &mut self,
        floor: f32,
        ceil: f32,
        a: Vector2<f32>,
        b: Vector2<f32>,
        _upper: &Option<(f32, f32, Sprite)>,
        _lower: &Option<(f32, f32, Sprite)>,
    ) {
        let mut fa = vec3(a.x, floor, a.y);
        let mut ca = vec3(a.x, ceil, a.y);
        let mut fb = vec3(b.x, floor, b.y);
        let mut cb = vec3(b.x, ceil, b.y);

        const CLIP_NEAR: f32 = 10.;

        if fa.z <= CLIP_NEAR && fb.z <= CLIP_NEAR {
            return;
        }

        let mut ua = 0.;
        let mut ub = ua + (b - a).magnitude();

        if fa.z < CLIP_NEAR {
            let d = fb - fa;
            let u = (CLIP_NEAR - fa.z) / d.z;

            let x = fa.x + u * d.x;
            fa.x = x;
            ca.x = x;

            fa.z = CLIP_NEAR;
            ca.z = CLIP_NEAR;

            ua = ua + (ub - ua) * u;
        }

        if fb.z < CLIP_NEAR {
            let d = fa - fb;
            let u = (CLIP_NEAR - fb.z) / d.z;

            let x = fb.x + u * d.x;
            fb.x = x;
            cb.x = x;

            fb.z = CLIP_NEAR;
            cb.z = CLIP_NEAR;

            ub = ub + (ua - ub) * u;
        }

        let za = fa.z;
        let zb = fb.z;

        let fa = self.project(fa);
        let ca = self.project(ca);
        let fb = self.project(fb);
        let cb = self.project(cb);

        let d_ceil = cb - ca;
        let d_floor = fb - fa;

        let v_top = 0.;
        let v_bottom = ceil - floor;

        let x_range = fa.x.round() as i32..fb.x.round() as i32;
        // let x_ranges = vec![intersect(x_range, 0..320)];
        let x_ranges = self.horizontally_clip(x_range);

        for x in x_ranges.into_iter().flatten() {
            let t = (x as f32 - fa.x) / d_floor.x;

            let top = ca.y + d_ceil.y * t;
            let bottom = fa.y + d_floor.y * t;
            let height = bottom - top;

            // Perspective correct interpolation of u coordinate
            // TODO: Derive from fundamental geometry
            let u = ((1. - t) * ua / za + t * ub / zb) / ((1. - t) / za + t / zb);

            // let u = (u.round() as i32).rem_euclid(texture.width() as i32);

            // // HITCH! Transparency does not make sense in the first rendering pass!
            // // Solid walls must be treated differently from transparent walls!
            // for span in texture.col(u as u32) {
            //     // Revisit. Clean up. FIXME: Does not work with different v ranges
            //     let span_y_top = top + height * (span.top as f32 / texture.height() as f32);
            //     let span_y_bottom = top
            //         + height
            //             * ((span.top as u32 + span.pixels.len() as u32) as f32
            //                 / texture.height() as f32);
            //     let dy = span_y_bottom - span_y_top;

            //     let y_range = span_y_top.round() as i32..span_y_bottom.round() as i32;

            //     // Vertical clipping
            //     // let y_range = intersect(y_range, 0..200); // Redundant
            //     let y_range = intersect(y_range, self.v_open[x as usize].clone());

            //     for y in y_range {
            //         let s = (y as f32 - span_y_top) / dy * span.pixels.len() as f32;
            //         self.framebuffer[[y as usize, x as usize]] = span.pixels[s as usize];
            //     }
            // }

            // TODO Yield visplanes

            self.v_open[x as usize] =
                intersect(self.v_open[x as usize].clone(), top as _..bottom as _);

            // self.v_open[x as usize] = 0..0;
        }
    }
}
