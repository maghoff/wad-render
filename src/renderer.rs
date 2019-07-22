pub struct State {
    titlepic: Vec<u8>,
}

impl State {
    pub fn new(wad_slice: &[u8]) -> State {
        let wad = wad::parse_wad(Vec::from(wad_slice)).unwrap();

        let titlepic = wad.by_id(b"TITLEPIC").unwrap();

        State {
            titlepic: Vec::from(titlepic),
        }
    }
}

pub fn render(state: &State, crate::Input { buf, .. }: crate::Input) {
    for y in 0..200 {
        for x in 0..320 {
            buf[y * 320 + x] = state.titlepic[1288 + x * 209 + y];
        }
    }
}
