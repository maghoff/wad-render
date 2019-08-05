"use strict";

const WIDTH = 320;
const HEIGHT = 200;
const FRAME_BYTE_SIZE = WIDTH * HEIGHT * 4;

function fpsControls(dom, pos, dir, update) {
    dom.addEventListener("click", ev => {
        ev.preventDefault();
        ev.stopPropagation();
        dom.requestPointerLock();
    });

    function mousemove(ev) {
        ev.preventDefault();
        ev.stopPropagation();

        const d = dir();
        const ang = ev.movementX / 90;

        update(
            pos(),
            {
                x: d.x * Math.cos(ang) - d.y * Math.sin(ang),
                y: d.x * Math.sin(ang) + d.y * Math.cos(ang),
            }
        )
    }

    const held = {
        'w': false,
        's': false,
        'a': false,
        'd': false,
    };

    let animating = false;
    let prevTimer = null;
    function animate() {
        if (animating) return;
        animating = true;
        prevTimer = performance.now();
        requestAnimationFrame(animationFrame);
    }

    function animationFrame(timer) {
        const fwd = (held['w'] ? 1 : 0) + (held['s'] ? -1 : 0);
        const rig = (held['d'] ? 1 : 0) + (held['a'] ? -1 : 0);
        if (fwd == 0 && rig == 0) {
            animating = false;
            return;
        }

        const dt = timer - prevTimer;
        const l = dt * 0.3;

        const p = pos();
        const d = dir();
        const s = { x: -d.y, y: d.x };
        update(
            {
                x: p.x + fwd * l * d.x + rig * l * s.x,
                y: p.y + fwd * l * d.y + rig * l * s.y,
            },
            d
        )

        prevTimer = timer;
        requestAnimationFrame(animationFrame);
    }

    function keydown(ev) {
        const k = ev.key.toLowerCase();
        if (k != 'w' && k != 's' && k != 'a' && k != 'd') return;

        ev.preventDefault();
        ev.stopPropagation();

        held[k] = true;

        animate();
    }

    function keyup(ev) {
        const k = ev.key.toLowerCase();
        if (k != 'w' && k != 's' && k != 'a' && k != 'd') return;

        ev.preventDefault();
        ev.stopPropagation();

        held[k] = false;

        animate();
    }

    function lockChangeAlert() {
        const el = document.pointerLockElement || document.mozPointerLockElement;

        if (el === dom) {
            document.addEventListener("mousemove", mousemove, false);
            document.addEventListener("keydown", keydown, false);
            document.addEventListener("keyup", keyup, false);
        } else {
            document.removeEventListener("mousemove", mousemove, false);
            document.removeEventListener("keydown", keydown, false);
            document.removeEventListener("keyup", keyup, false);
            held['w'] = held['s'] = held['a'] = held['d'] = false;
        }
    }

    document.addEventListener('pointerlockchange', lockChangeAlert, false);
    document.addEventListener('mozpointerlockchange', lockChangeAlert, false);
}

function initCanvas() {
    const canvas = document.getElementById('screen');
    return canvas.getContext('2d');
}

function copyArrayBuffer(arrayBuffer, buffer, ptr) {
    const src = new Uint8ClampedArray(arrayBuffer);
    const dst = new Uint8ClampedArray(buffer, ptr, arrayBuffer.byteLength);
    dst.set(src);
}

function renderFrame(mod, state, screen, focusPoint, direction) {
    mod.render(
        state,
        screen.ptr,
        focusPoint.x, focusPoint.y,
        direction.x, direction.y
    );

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
    const wadObject = mod.parse_wad(wadPtr, wad.byteLength);

    const state = mod.init(wadObject);

    // --- --- ---

    let focusPoint = { x: 0, y: 0 };
    let direction = { x: 0, y: 1 };

    let pendingRender = false;
    function render(_timestamp) {
        pendingRender = false;
        renderFrame(mod, state, screen, focusPoint, direction);
    }

    function scheduleRender() {
        if (pendingRender) return;
        pendingRender = true;
        window.requestAnimationFrame(render);
    }

    function updateCamera(newFocusPoint, newDirection) {
        focusPoint.x = newFocusPoint.x;
        focusPoint.y = newFocusPoint.y;
        direction.x = newDirection.x;
        direction.y = newDirection.y;
        scheduleRender();
    }

    fpsControls(
        document.getElementById('screen'),
        () => focusPoint,
        () => direction,
        (focusPoint, direction) => {
            updateCamera(focusPoint, direction);
        }
    );

    // --- --- ---

    renderFrame(mod, state, screen, focusPoint, direction);
}

init()
    .catch(ex => { alert(ex); throw ex; });
