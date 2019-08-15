use cgmath::{vec2, Vector2};

pub struct BspTraverser<'a> {
    nodes: &'a [wad_map::Node],
    pos: Vector2<f32>,
    state: Vec<wad_map::Child>,
}

impl<'a> BspTraverser<'a> {
    pub fn new(nodes: &'a [wad_map::Node], pos: Vector2<f32>) -> BspTraverser<'a> {
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
