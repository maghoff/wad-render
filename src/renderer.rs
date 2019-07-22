use crate::util::*;
use ndarray::prelude::*;
use wad::Wad;
use wad_gfx::Sprite;

pub struct State<'a> {
    playpal: &'a [u8],
    titlepic: Sprite<'a>,
}

impl<'a> State<'a> {
    pub fn new(wad: &Wad) -> State {
        let playpal = wad.by_id(b"PLAYPAL").unwrap();
        let titlepic = Sprite::new(wad.by_id(b"TITLEPIC").unwrap());

        State { playpal, titlepic }
    }
}

pub fn render(state: &State, crate::Input { buf, pal, .. }: crate::Input) {
    pal.clone_from_slice(&state.playpal[0..768]);

    let mut screen = ArrayViewMut2::from_shape((200, 320), buf).unwrap();
    put_sprite(&mut screen, 0, 0, &state.titlepic);
}
