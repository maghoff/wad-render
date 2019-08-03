#![feature(euclidean_division)]

extern crate wee_alloc;

use std::{mem, slice};

mod renderer;
mod util;

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

pub struct Input<'a> {
    pal: &'a mut [u8; 768],
    buf: &'a mut [u8; 320 * 200],
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub fn alloc(size: usize) -> *mut u8 {
    let block = vec![0u8; size].into_boxed_slice();
    Box::leak(block).as_mut_ptr()
}

#[no_mangle]
pub fn parse_wad(wad: *mut u8, wad_sz: usize) -> *mut wad::Wad {
    let wad_slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(mem::transmute(wad), wad_sz) };

    let wad = Box::new(wad::parse_wad(Vec::from(wad_slice)).unwrap());

    Box::leak(wad) as _
}

#[no_mangle]
pub fn init<'a>(wad: *mut wad::Wad) -> *mut renderer::State<'a> {
    let wad: &'a wad::Wad = unsafe { &*wad };

    let state = Box::new(renderer::State::new(wad));

    Box::leak(state) as _
}

#[no_mangle]
pub fn render(state: *mut renderer::State, screen_ptr: *mut u8) {
    let mut state = unsafe { Box::from_raw(state) };

    let screen_slice: &mut [u8] = unsafe {
        slice::from_raw_parts_mut(mem::transmute(screen_ptr), SCREEN_WIDTH * SCREEN_HEIGHT * 4)
    };

    let mut pal = [0; 768];
    for i in 0..256 {
        pal[i*3+0] = i as u8;
        pal[i*3+1] = 0;
        pal[i*3+2] = 0;
    }

    let mut framebuf = [0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let input = Input { pal: &mut pal, buf: &mut framebuf };

    state.render(input);

    for (dst, src) in screen_slice.chunks_exact_mut(4).zip(framebuf.iter_mut()) {
        let col = *src as usize;
        dst[0] = pal[col * 3 + 0];
        dst[1] = pal[col * 3 + 1];
        dst[2] = pal[col * 3 + 2];
        dst[3] = 255;
    }

    Box::leak(state);
}
