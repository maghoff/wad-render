extern crate wee_alloc;

use std::{mem, slice};

mod renderer;

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub fn alloc(size: usize) -> *mut u8 {
    let block = Vec::<u8>::with_capacity(size).into_boxed_slice();
    Box::leak(block).as_mut_ptr()
}

#[no_mangle]
pub fn basic() -> usize {
    let a = vec![1, 2, 3];
    a.len()
}

#[no_mangle]
pub fn render(screen_ptr: *mut u8) {
    let screen_slice: &mut [u8] = unsafe {
        slice::from_raw_parts_mut(mem::transmute(screen_ptr), SCREEN_WIDTH * SCREEN_HEIGHT * 4)
    };

    let mut framebuf = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
    renderer::render(&mut framebuf);

    for (dst, src) in screen_slice.chunks_exact_mut(4).zip(framebuf.iter_mut()) {
        dst[0] = *src;
        dst[1] = 0;
        dst[2] = 0;
        dst[3] = 255;
    }
}
