use crate::{util::*, Input};
use array_macro::array;
use cgmath::prelude::*;
use cgmath::{vec2, vec3, Vector2, Vector3};
use ndarray::prelude::*;
use std::ops::Range;
use wad::Wad;
use wad_gfx::Sprite;

const TAU: f32 = 2. * ::std::f32::consts::PI;
const PROJECTION_PLANE_WIDTH: f32 = 320.;
const FOV: f32 = 90. * TAU / 360.;
const PROJECTION_PLANE_HALF_WIDTH: f32 = PROJECTION_PLANE_WIDTH / 2.;
const EYE_HEIGHT: f32 = 40.;

pub struct State<'a> {
    playpal: &'a [u8],
    texture_provider: TextureProvider<'a>,
    map: wad_map::Map,
}

pub struct BspTraverser<'a> {
    nodes: &'a [wad_map::Node],
    pos: Vector2<f32>,
    state: Vec<wad_map::Child>,
}

impl<'a> BspTraverser<'a> {
    fn new(nodes: &'a [wad_map::Node], pos: Vector2<f32>) -> BspTraverser<'a> {
        BspTraverser {
            nodes,
            pos,
            state: vec![((nodes.len() - 1) as u16).into()],
        }
    }
}

impl<'a> Iterator for BspTraverser<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self.state.pop()? {
            wad_map::Child::Subsector(s) => Some(s),
            wad_map::Child::Subnode(n) => {
                let node = &self.nodes[n as usize];

                let view = self.pos - vec2(node.x as f32, node.y as f32);
                let left = node.dy as f32 * view.x;
                let right = view.y * node.dx as f32;

                let is_right_side = right < left;

                if is_right_side {
                    self.state.push(node.left_child.clone());
                    self.state.push(node.right_child.clone());
                } else {
                    self.state.push(node.right_child.clone());
                    self.state.push(node.left_child.clone());
                }
                self.next()
            }
        }
    }
}

impl<'a> State<'a> {
    pub fn new(wad: &Wad) -> State {
        State {
            playpal: wad.by_id(b"PLAYPAL").unwrap(),
            texture_provider: TextureProvider::new(wad.as_slice()),
            map: wad_map::read_map(&wad.as_slice(), "E1M1").unwrap(),
        }
    }

    pub fn svg_from_map(&self) -> String {
        let mut buf = String::new();
        generate_svg(&mut buf, &self.map).unwrap();
        buf
    }

    pub fn spawn_point(&self) -> (Vector2<f32>, Vector2<f32>) {
        let spawn_thing = &self.map.things.iter().find(|&x| x.thing_type == 1).unwrap();
        let ang = spawn_thing.ang as f32 / 360. * TAU;
        (
            vec2(spawn_thing.x as _, spawn_thing.y as _),
            vec2(ang.cos(), ang.sin()),
        )
    }

    fn floor_height_at(&self, pos: Vector2<f32>) -> f32 {
        for subsector in BspTraverser::new(&self.map.nodes, pos) {
            let subsector = &self.map.subsectors[subsector as usize];

            let start = subsector.first_seg as usize;
            let end = start + subsector.seg_count as usize;

            for line_segment in &self.map.line_segments[start..end] {
                let linedef = &self.map.linedefs[line_segment.linedef as usize];

                let a = &self.map.vertexes[line_segment.start_vertex as usize];
                let b = &self.map.vertexes[line_segment.end_vertex as usize];

                let a = vec2(a.x as f32, a.y as f32);
                let b = vec2(b.x as f32, b.y as f32);

                let reverse = line_segment.direction != 0;
                let right_side = ((pos - a).perp_dot(b - a) > 0.) ^ reverse;

                let front_sidedef = if right_side {
                    linedef.right_sidedef
                } else {
                    linedef.left_sidedef
                };

                if let Some(front_sidedef) = front_sidedef {
                    let front_sidedef = &self.map.sidedefs[front_sidedef as usize];

                    let front_sector = front_sidedef.sector_id;
                    let front_sector = &self.map.sectors[front_sector as usize];

                    return front_sector.floor_height as f32 + EYE_HEIGHT;
                }
            }
        }
        unreachable!()
    }

    pub fn render(
        &mut self,
        Input {
            buf, pal, pos, dir, ..
        }: Input,
    ) {
        pal.clone_from_slice(&self.playpal[0..768]);

        let mut screen = ArrayViewMut2::from_shape((200, 320), buf).unwrap();
        fill(&mut screen, 0);

        let mut rendering_state = RenderingState::new(&mut screen);

        // Mysterious rotation matrix:
        let transform = cgmath::Matrix2::new(dir.y, dir.x, -dir.x, dir.y);

        let camera_y = self.floor_height_at(pos);

        for subsector in BspTraverser::new(&self.map.nodes, pos) {
            let subsector = &self.map.subsectors[subsector as usize];

            let start = subsector.first_seg as usize;
            let end = start + subsector.seg_count as usize;

            for line_segment in &self.map.line_segments[start..end] {
                let linedef = &self.map.linedefs[line_segment.linedef as usize];

                let a = &self.map.vertexes[line_segment.start_vertex as usize];
                let b = &self.map.vertexes[line_segment.end_vertex as usize];

                let a = vec2(a.x as f32, a.y as f32);
                let b = vec2(b.x as f32, b.y as f32);

                let reverse = line_segment.direction != 0;
                let right_side = ((pos - a).perp_dot(b - a) > 0.) ^ reverse;

                let (front_sidedef, back_sidedef) = if right_side {
                    (linedef.right_sidedef, linedef.left_sidedef)
                } else {
                    (linedef.left_sidedef, linedef.right_sidedef)
                };

                let a = transform * (a - pos);
                let b = transform * (b - pos);

                let portal = front_sidedef.is_some() && back_sidedef.is_some();

                if portal {
                    // TODO: Push deferred rendering instructions on a stack somewhere
                    // if texture == b"-\0\0\0\0\0\0\0" {
                    //     continue;
                    // }
                } else {
                    if let Some(front_sidedef) = front_sidedef {
                        let front_sidedef = &self.map.sidedefs[front_sidedef as usize];

                        let front_sector = front_sidedef.sector_id;
                        let front_sector = &self.map.sectors[front_sector as usize];

                        let texture = &front_sidedef.middle_texture;
                        let texture = &self.texture_provider.texture(texture);

                        let floor = front_sector.floor_height as f32 - camera_y;
                        let ceil = front_sector.ceil_height as f32 - camera_y;

                        rendering_state.wall(floor, ceil, a, b, texture);

                        if rendering_state.is_complete() {
                            return;
                        }
                    }
                }
            }
        }
    }
}

struct RenderingState<'a> {
    distance_to_projection_plane: f32,
    framebuffer: &'a mut ArrayViewMut2<'a, u8>,
    h_open: Vec<Range<i32>>,
    v_open: [Range<i32>; 320],
}

impl<'a> RenderingState<'a> {
    fn new(framebuffer: &'a mut ArrayViewMut2<'a, u8>) -> RenderingState<'a> {
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

    fn is_complete(&self) -> bool {
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

    fn wall(&mut self, floor: f32, ceil: f32, a: Vector2<f32>, b: Vector2<f32>, texture: &Sprite) {
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
}
