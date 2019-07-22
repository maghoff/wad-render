extern crate wee_alloc;

use std::{mem, slice};

mod renderer;

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

pub struct Input<'a> {
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
pub fn init(wad: *mut u8, wad_sz: usize) -> *mut renderer::State {
    let wad_slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(mem::transmute(wad), wad_sz) };

    let state = Box::new(renderer::State::new(wad_slice));

    Box::leak(state) as _
}

#[no_mangle]
pub fn render(state: *mut renderer::State, screen_ptr: *mut u8) {
    let state = unsafe { Box::from_raw(state) };

    let screen_slice: &mut [u8] = unsafe {
        slice::from_raw_parts_mut(mem::transmute(screen_ptr), SCREEN_WIDTH * SCREEN_HEIGHT * 4)
    };

    let mut framebuf = [0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let input = Input { buf: &mut framebuf };

    renderer::render(&*state, input);

    for (dst, src) in screen_slice.chunks_exact_mut(4).zip(framebuf.iter_mut()) {
        dst[0] = *src;
        dst[1] = 0;
        dst[2] = 0;
        dst[3] = 255;
    }

    Box::leak(state);
}
