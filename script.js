"use strict";

const WIDTH = 320;
const HEIGHT = 200;
const FRAME_BYTE_SIZE = WIDTH * HEIGHT * 4;

function initCanvas() {
    const canvas = document.getElementById('screen');
    return canvas.getContext('2d');
}

function copyArrayBuffer(arrayBuffer, buffer, ptr) {
    const src = new Uint8ClampedArray(arrayBuffer);
    const dst = new Uint8ClampedArray(buffer, ptr, arrayBuffer.byteLength);
    dst.set(src);
}

function renderFrame(mod, state, screen) {
    mod.render(state, screen.ptr);

    const screenBuf = new Uint8ClampedArray(mod.memory.buffer, screen.ptr, FRAME_BYTE_SIZE);
    const img = new ImageData(screenBuf, WIDTH, HEIGHT);

    screen.ctx.putImageData(img, 0, 0);
}

async function init() {
    const [wasm, wad] = await Promise.all([
        WebAssembly.instantiateStreaming(fetch("wad_render.gc.wasm")),
        fetch("doom1.wad").then(x => x.arrayBuffer()),
    ]);

    const mod = wasm.instance.exports;

    let screen = {
        ctx: initCanvas(),
        ptr: mod.alloc(FRAME_BYTE_SIZE),
    };

    const wadPtr = mod.alloc(wad.byteLength);
    copyArrayBuffer(wad, mod.memory.buffer, wadPtr);

    const state = mod.init(wadPtr, wad.byteLength);

    renderFrame(mod, state, screen);
}

init()
    .catch(ex => { alert(ex); throw ex; });
