use crate::{util::*, Input};
use cgmath::prelude::*;
use cgmath::{vec2, vec3, Vector2, Vector3};
use ndarray::prelude::*;
use wad::Wad;
use wad_gfx::Sprite;

const TAU: f32 = 2. * ::std::f32::consts::PI;
const PROJECTION_PLANE_WIDTH: f32 = 320.;
const FOV: f32 = 60. * TAU / 360.;
const PROJECTION_PLANE_HALF_WIDTH: f32 = PROJECTION_PLANE_WIDTH / 2.;

pub struct State<'a> {
    distance_to_projection_plane: f32,
    playpal: &'a [u8],
    titlepic: Sprite<'a>,
    wall: Sprite<'a>,
}

impl<'a> State<'a> {
    pub fn new(wad: &Wad) -> State {
        State {
            distance_to_projection_plane: PROJECTION_PLANE_HALF_WIDTH / (FOV / 2.).tan(),
            playpal: wad.by_id(b"PLAYPAL").unwrap(),
            titlepic: Sprite::new(wad.by_id(b"TITLEPIC").unwrap()),
            wall: Sprite::new(wad.by_id(b"WALL62_1").unwrap()),
        }
    }

    fn project(&self, p: Vector3<f32>) -> Vector2<f32> {
        let w = 1. / p.z;

        vec2(
            160. + self.distance_to_projection_plane * p.x * w,
            100. - self.distance_to_projection_plane * p.y * w,
        )
    }

    fn wall(
        &self,
        screen: &mut ArrayViewMut2<u8>,
        floor: f32,
        ceil: f32,
        a: Vector2<f32>,
        b: Vector2<f32>,
        texture: &Sprite,
    ) {
        let fa = vec3(a.x, floor, a.y);
        let ca = vec3(a.x, ceil, a.y);
        let fb = vec3(b.x, floor, b.y);
        let cb = vec3(b.x, ceil, b.y);

        let za = fa.z;
        let zb = fb.z;

        let fa = self.project(fa);
        let ca = self.project(ca);
        let fb = self.project(fb);
        let cb = self.project(cb);

        line(screen, fa, fb, 0);
        line(screen, ca, cb, 0);

        line(screen, fa, ca, 0);
        line(screen, fb, cb, 0);

        let d_ceil = cb - ca;
        let d_floor = fb - fa;

        let ua = 0.;
        let ub = 128.;

        let v_top = 0.;
        let v_bottom = 128.;

        let x_range = fa.x.round() as i32..fb.x.round() as i32;
        let x_range = intersect(x_range, 0..320);

        for x in x_range {
            let t = (x as f32 - fa.x) / d_floor.x;

            let top = ca.y + d_ceil.y * t;
            let bottom = fa.y + d_floor.y * t;
            let height = bottom - top;

            // Perspective correct interpolation of u coordinate
            // TODO: Derive from fundamental geometry
            let u = ((1. - t) * ua / za + t * ub / zb) / ((1. - t) / za + t / zb);

            let u = u.round() as i32 % texture.width() as i32;

            for span in texture.col(u as u32) {
                // Revisit. Clean up. FIXME: Does not work with different v ranges
                let span_y_top = top + height * (span.top as f32 / texture.height() as f32);
                let span_y_bottom = top
                    + height
                        * ((span.top as u32 + span.pixels.len() as u32) as f32
                            / texture.height() as f32);
                let dy = span_y_bottom - span_y_top;

                let y_range = span_y_top.round() as i32..span_y_bottom.round() as i32;
                let y_range = intersect(y_range, 0..200);

                for y in y_range {
                    let s = (y as f32 - span_y_top) / dy * span.pixels.len() as f32;
                    screen[[y as usize, x as usize]] = span.pixels[s as usize];
                }
            }
        }
    }

    pub fn render(&self, Input { buf, pal, .. }: Input) {
        pal.clone_from_slice(&self.playpal[0..768]);

        let mut screen = ArrayViewMut2::from_shape((200, 320), buf).unwrap();
        put_sprite(&mut screen, 0, 0, &self.titlepic);

        // put_sprite(&mut screen, 160, 150, &self.wall);

        self.wall(
            &mut screen,
            -20.,
            12.,
            vec2(-16., 100.),
            vec2(16., 50.),
            &self.wall,
        );
    }
}
