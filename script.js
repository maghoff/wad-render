"use strict";

const WIDTH = 320;
const HEIGHT = 200;
const FRAME_BYTE_SIZE = WIDTH * HEIGHT * 4;

function initCanvas(buffer, ptr) {
    const canvas = document.getElementById('screen');

    const screenBuf = new Uint8ClampedArray(buffer, ptr, FRAME_BYTE_SIZE);
    const img = new ImageData(screenBuf, WIDTH, HEIGHT);

    const ctx = canvas.getContext('2d');

    return { ctx, img };
}

function allocate(mod) {
    // Do all allocations up front, as they may invalidate mod.memory.buffer

    const screen = mod.alloc(FRAME_BYTE_SIZE);

    return { screen };
}

function renderFrame(mod, screen) {
    mod.render(screen.ptr);

    screen.ctx.putImageData(screen.img, 0, 0);
}

async function init() {
    const wasm = await WebAssembly.instantiateStreaming(fetch("wad_render.gc.wasm"));
    const mod = wasm.instance.exports;

    const ptr = allocate(mod);

    const { ctx, img } = initCanvas(mod.memory.buffer, ptr.screen);

    let screen = {
        ctx,
        img,
        ptr: ptr.screen,
    };

    renderFrame(mod, screen);
}

init()
    .catch(ex => { alert(ex); throw ex; });
