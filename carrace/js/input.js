// Input handling — ES module
// Keys: WASD / Arrows for driving, Space = brake, R = reset

const keys = {
    up: false,
    down: false,
    left: false,
    right: false,
    recover: false,
    reset: false,
};

function handleKey(e, pressed) {
    switch (e.code) {
        case 'ArrowUp':
        case 'KeyW':
            keys.up = pressed;
            e.preventDefault();
            break;
        case 'ArrowDown':
        case 'KeyS':
            keys.down = pressed;
            e.preventDefault();
            break;
        case 'ArrowLeft':
        case 'KeyA':
            keys.left = pressed;
            e.preventDefault();
            break;
        case 'ArrowRight':
        case 'KeyD':
            keys.right = pressed;
            e.preventDefault();
            break;
        case 'Space':
            keys.recover = pressed;
            e.preventDefault();
            break;
        case 'KeyR':
            keys.reset = pressed;
            e.preventDefault();
            break;
    }
}

export function initInput() {
    document.addEventListener('keydown', (e) => handleKey(e, true));
    document.addEventListener('keyup', (e) => handleKey(e, false));
}

export function getInput() {
    return keys;
}
