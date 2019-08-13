'use strict';

// https://www.sitepoint.com/how-to-translate-from-dom-to-svg-coordinates-and-back-again/
// translate page to SVG co-ordinate
function svgPoint(element, x, y) {
    const svg = document.querySelector("svg");
    const pt = svg.createSVGPoint();

    pt.x = x;
    pt.y = y;

    return pt.matrixTransform(element.getScreenCTM().inverse());
}

function draggable(node, callback) {
    let dragging = false;

    node.addEventListener("click", ev => {
        ev.preventDefault();
        ev.stopPropagation();
    });
    node.addEventListener("mousedown", ev => {
        ev.preventDefault();
        ev.stopPropagation();

        node.classList.add("drag");
        node.setCapture(true);
        dragging = true;
    });
    node.addEventListener("mousemove", ev => {
        if (!dragging) return;
        ev.preventDefault();
        ev.stopPropagation();

        const tr = svgPoint(node, ev.x, ev.y);
        callback(tr.x, tr.y);
    });
    node.addEventListener("mouseup", ev => {
        if (!dragging) return;
        ev.preventDefault();
        ev.stopPropagation();

        node.classList.remove("drag");
        dragging = false;
    });
}

function length(v) {
    return Math.sqrt(v.x * v.x + v.y * v.y);
}

function initCamera(cameraDom, initialState, callback) {
    const arrowSize = 64;

    const focusPoint = {
        x: initialState.focusPoint.x,
        y: initialState.focusPoint.y,
    };

    let direction = {
        x: initialState.direction.x,
        y: initialState.direction.y,
    };

    const targetPoint = {
        x: focusPoint.x + direction.x * 256,
        y: focusPoint.y + direction.y * 256,
    };

    const dom = {
        focus: cameraDom.querySelector(".camera--focus"),
        target: cameraDom.querySelector(".camera--target"),
        sightline: cameraDom.querySelector(".camera--sightline"),
        direction: cameraDom.querySelector(".camera--direction"),
        fovLeft: cameraDom.querySelector(".camera--fov-left"),
        fovRight: cameraDom.querySelector(".camera--fov-right"),
    };

    function updateDirection() {
        const dirVec = {
            x: targetPoint.x - focusPoint.x,
            y: targetPoint.y - focusPoint.y,
        };
        const len = Math.sqrt(dirVec.x * dirVec.x + dirVec.y * dirVec.y);
        direction = {
            x: dirVec.x / len,
            y: dirVec.y / len,
        };

        const offset = {
            x: direction.x * arrowSize,
            y: direction.y * arrowSize,
        };

        dom.direction.setAttribute("x1", focusPoint.x);
        dom.direction.setAttribute("y1", focusPoint.y);
        dom.direction.setAttribute("x2", focusPoint.x + offset.x);
        dom.direction.setAttribute("y2", focusPoint.y + offset.y);

        const TAU = Math.PI * 2;
        const projection_plane_width = 320.;
        const fov = 90. * TAU / 360.;
        const projection_plane_half_width = projection_plane_width / 2.;
        const distance_to_projection_plane = projection_plane_half_width / Math.tan(fov / 2.);

        const side = {
            x: -direction.y,
            y: direction.x,
        }

        dom.fovLeft.setAttribute("x1", focusPoint.x);
        dom.fovLeft.setAttribute("y1", focusPoint.y);
        dom.fovLeft.setAttribute("x2", focusPoint.x + direction.x * distance_to_projection_plane - side.x * projection_plane_half_width);
        dom.fovLeft.setAttribute("y2", focusPoint.y + direction.y * distance_to_projection_plane - side.y * projection_plane_half_width);

        dom.fovRight.setAttribute("x1", focusPoint.x);
        dom.fovRight.setAttribute("y1", focusPoint.y);
        dom.fovRight.setAttribute("x2", focusPoint.x + direction.x * distance_to_projection_plane + side.x * projection_plane_half_width);
        dom.fovRight.setAttribute("y2", focusPoint.y + direction.y * distance_to_projection_plane + side.y * projection_plane_half_width);
    }

    draggable(cameraDom.querySelector(".camera--target"), (x, y) => {
        targetPoint.x = x;
        targetPoint.y = y;

        dom.target.setAttribute("cx", x);
        dom.target.setAttribute("cy", y);
        dom.sightline.setAttribute("x2", x);
        dom.sightline.setAttribute("y2", y);
        updateDirection();

        callback(focusPoint, direction);
    });

    draggable(cameraDom.querySelector(".camera--focus"), (x, y) => {
        focusPoint.x = x;
        focusPoint.y = y;

        dom.focus.setAttribute("cx", x);
        dom.focus.setAttribute("cy", y);
        dom.sightline.setAttribute("x1", x);
        dom.sightline.setAttribute("y1", y);
        updateDirection();

        callback(focusPoint, direction);
    });

    function updateDom() {
        dom.focus.setAttribute("cx", focusPoint.x);
        dom.focus.setAttribute("cy", focusPoint.y);
        dom.sightline.setAttribute("x1", focusPoint.x);
        dom.sightline.setAttribute("y1", focusPoint.y);

        dom.target.setAttribute("cx", targetPoint.x);
        dom.target.setAttribute("cy", targetPoint.y);
        dom.sightline.setAttribute("x2", targetPoint.x);
        dom.sightline.setAttribute("y2", targetPoint.y);

        updateDirection();
    }
    updateDom();

    return (newFocusPoint, newDirection) => {
        const sightlineLength = Math.min(
            Math.max(
                length({
                    x: targetPoint.x - focusPoint.x,
                    y: targetPoint.y - focusPoint.y,
                }),
                128
            ),
            512
        );

        focusPoint.x = newFocusPoint.x;
        focusPoint.y = newFocusPoint.y;

        direction.x = newDirection.x;
        direction.y = newDirection.y;

        targetPoint.x = focusPoint.x + sightlineLength * direction.x;
        targetPoint.y = focusPoint.y + sightlineLength * direction.y;

        updateDom();
    };
}

function interactiveMap(svg, camera, updateCamera) {
    const u = initCamera(
        svg.querySelector(".camera"),
        camera,
        updateCamera
    );

    return {
        updateCamera: u
    }
}

export default interactiveMap;
