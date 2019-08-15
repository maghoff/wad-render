use crate::{bsp_traverser::*, rendering_state::*, util::*, Input};
use cgmath::{vec2, Vector2};
use ndarray::prelude::*;
use wad::Wad;

const TAU: f32 = 2. * ::std::f32::consts::PI;
const EYE_HEIGHT: f32 = 40.;

pub struct State<'a> {
    playpal: &'a [u8],
    texture_provider: TextureProvider<'a>,
    map: wad_map::Map,
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
                    let front_sidedef = &self.map.sidedefs[front_sidedef.unwrap() as usize];
                    let back_sidedef = &self.map.sidedefs[back_sidedef.unwrap() as usize];

                    let front_sector = &self.map.sectors[front_sidedef.sector_id as usize];
                    let back_sector = &self.map.sectors[back_sidedef.sector_id as usize];

                    let _ = self
                        .texture_provider
                        .load_texture(&front_sidedef.upper_texture);
                    let _ = self
                        .texture_provider
                        .load_texture(&front_sidedef.lower_texture);

                    let upper = if front_sector.ceil_height > back_sector.ceil_height {
                        self.texture_provider
                            .get_texture(&front_sidedef.upper_texture)
                            .map(|texture| {
                                (
                                    front_sector.ceil_height as f32 - camera_y,
                                    std::cmp::max(
                                        back_sector.ceil_height,
                                        front_sector.floor_height,
                                    ) as f32
                                        - camera_y,
                                    texture,
                                )
                            })
                    } else {
                        None
                    };

                    let lower = if front_sector.floor_height < back_sector.floor_height {
                        self.texture_provider
                            .get_texture(&front_sidedef.lower_texture)
                            .map(|texture| {
                                (
                                    std::cmp::min(
                                        back_sector.floor_height,
                                        front_sector.ceil_height,
                                    ) as f32
                                        - camera_y,
                                    front_sector.floor_height as f32 - camera_y,
                                    texture,
                                )
                            })
                    } else {
                        None
                    };

                    let floor = std::cmp::max(front_sector.floor_height, back_sector.floor_height)
                        as f32
                        - camera_y;
                    let ceil = std::cmp::min(front_sector.ceil_height, back_sector.ceil_height)
                        as f32
                        - camera_y;

                    rendering_state.portal(floor, ceil, a, b, &upper, &lower);

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
                        self.texture_provider.load_texture(texture).unwrap();
                        let texture = &self.texture_provider.get_texture(texture).unwrap();

                        let floor = front_sector.floor_height as f32 - camera_y;
                        let ceil = front_sector.ceil_height as f32 - camera_y;

                        rendering_state.wall(floor, ceil, a, b, texture);
                    }
                }
            }

            if rendering_state.is_complete() {
                return;
            }
        }
    }
}
