// Main entry — ES module
// Three.js renderer, kinematic car, game loop, chase camera

import * as THREE from 'three';
import * as CANNON from 'cannon-es';
import { initInput, getInput } from './input.js';
import { createEnvironment } from './environment.js';
import { createCar, getChassisBody, getSpeedMPH, getTopSpeed, setDrivingPreset } from './car3d.js';
import { createTrack } from './track.js';
import { initRace, updateRace, syncAllRaceCars, restartRace, recoverPlayerCar, getRaceHUD } from './race.js';
import { initAudio, updateAudio, playImpact, suspendAudio, resumeAudio, isDriftingCheck } from './audio.js';
import { createParticleSystem, emitSmoke, updateParticles } from './particles.js';

// ── Three.js setup ───────────────────────────────────────────────────
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(
    75,
    window.innerWidth / window.innerHeight,
    0.1,
    5000,
);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;
renderer.setClearColor(0x88aadd);
document.getElementById('game-canvas').appendChild(renderer.domElement);

window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
});

// ── cannon-es world (environment + track visuals only, not used for car) ─
const world = new CANNON.World();
world.gravity.set(0, -9.82, 0);

// ── Build scene ──────────────────────────────────────────────────────
const env = createEnvironment(scene, world);
createTrack(scene, world);
const playerCar = createCar(scene, world);
initInput();

// ── Initialize race (positions all cars on grid) ─────────────────────
initRace(playerCar, scene, world);

// ── Particles ────────────────────────────────────────────────────────
createParticleSystem(scene, camera);

// ── Audio init on first user gesture ─────────────────────────────────
let audioStarted = false;
function tryInitAudio() {
    if (audioStarted) return;
    audioStarted = true;
    initAudio();
}
document.addEventListener('keydown', tryInitAudio, { once: false });
document.addEventListener('click', tryInitAudio, { once: false });

document.addEventListener('visibilitychange', () => {
    if (document.hidden) suspendAudio();
    else resumeAudio();
});

// (Collision sound removed — car is kinematic, no physics collisions)

// ── Chase camera state ───────────────────────────────────────────────
const camPos = new THREE.Vector3(0, 5, -12);
const camLookAt = new THREE.Vector3();
let shakeIntensity = 0;

const CAM_DISTANCE = 12;
const CAM_HEIGHT = 4;
const CAM_LOOK_AHEAD = 10;
const CAM_POS_SMOOTH = 0.06;
const CAM_LOOK_SMOOTH = 0.10;
const BASE_FOV = 75;

// ── HUD elements ─────────────────────────────────────────────────────
const speedEl = document.getElementById('speed');
const debugEl = document.getElementById('debug');
const raceInfoEl = document.getElementById('race-info');
const overlayEl = document.getElementById('race-overlay');

// ── Driving preset selector ─────────────────────────────────────────
const presetSelect = document.getElementById('driving-preset');
presetSelect.addEventListener('change', () => setDrivingPreset(presetSelect.value));

// ── Clock ────────────────────────────────────────────────────────────
const clock = new THREE.Clock();

// ── Game loop ────────────────────────────────────────────────────────
function animate() {
    requestAnimationFrame(animate);

    const dt = Math.min(clock.getDelta(), 0.1);

    // Input
    const input = getInput();

    // Space = recover car upright on track
    if (input.recover) {
        recoverPlayerCar();
        input.recover = false;
    }

    // R = restart race
    if (input.reset) {
        restartRace();
        input.reset = false;
    }

    // Race update: moves player (kinematic) + AI (spline-following)
    updateRace(dt, input);

    // Sync all car visuals (player + AI)
    syncAllRaceCars();

    // Drift detection + audio + particles
    const chassis = getChassisBody();
    const drifting = isDriftingCheck(chassis);
    const speed = chassis ? chassis.velocity.length() : 0;
    updateAudio(speed, getTopSpeed(), drifting);

    if (drifting && playerCar.wheelMeshes) {
        for (let wi = 2; wi <= 3; wi++) {
            emitSmoke(playerCar.wheelMeshes[wi].position, Math.min(speed / 30, 1));
        }
    }
    updateParticles(dt);

    // Camera
    updateCamera(dt);

    // Move shadow camera to follow car
    if (chassis && env.dirLight) {
        env.dirLight.target.position.copy(chassis.position);
        env.dirLight.target.updateMatrixWorld();
        env.dirLight.position.set(
            chassis.position.x + 50,
            80,
            chassis.position.z + 30,
        );
    }

    // HUD
    updateRaceHUD();

    renderer.render(scene, camera);
}

function updateCamera(dt) {
    const chassis = getChassisBody();
    if (!chassis) return;

    const pos = chassis.position;
    const speed = chassis.velocity.length();
    const speedRatio = Math.min(speed / 40, 1);

    // Car forward direction from chassis quaternion (-Z = travel direction)
    const forward = new THREE.Vector3(0, 0, -1);
    const q = new THREE.Quaternion(
        chassis.quaternion.x,
        chassis.quaternion.y,
        chassis.quaternion.z,
        chassis.quaternion.w,
    );
    forward.applyQuaternion(q);

    // Speed-adaptive distance / height
    const dist = CAM_DISTANCE + speedRatio * 4;
    const height = CAM_HEIGHT + speedRatio * 2;

    // Target camera position: behind and above car
    const targetPos = new THREE.Vector3(
        pos.x - forward.x * dist,
        pos.y + height,
        pos.z - forward.z * dist,
    );

    // Ground clamp — don't go below road surface
    if (targetPos.y < pos.y + 2) targetPos.y = pos.y + 2;

    // Target look-at: ahead of car
    const lookAhead = CAM_LOOK_AHEAD + speedRatio * 6;
    const targetLook = new THREE.Vector3(
        pos.x + forward.x * lookAhead,
        pos.y + 1.5,
        pos.z + forward.z * lookAhead,
    );

    // Lerp
    camPos.lerp(targetPos, CAM_POS_SMOOTH);
    camLookAt.lerp(targetLook, CAM_LOOK_SMOOTH);

    camera.position.copy(camPos);

    // Collision shake
    if (shakeIntensity > 0.01) {
        camera.position.x += (Math.random() - 0.5) * shakeIntensity;
        camera.position.y += (Math.random() - 0.5) * shakeIntensity * 0.5;
        camera.position.z += (Math.random() - 0.5) * shakeIntensity;
        shakeIntensity *= 0.85;
    }

    camera.lookAt(camLookAt);

    // Speed FOV effect
    const targetFOV = BASE_FOV + speedRatio * 10;
    camera.fov = THREE.MathUtils.lerp(camera.fov, targetFOV, 0.03);
    camera.updateProjectionMatrix();
}

function updateRaceHUD() {
    const hud = getRaceHUD();
    speedEl.textContent = getSpeedMPH() + ' MPH';
    raceInfoEl.textContent = `Lap ${Math.min(hud.playerLap + 1, hud.totalLaps)}/${hud.totalLaps} | P${hud.playerPosition}`;

    if (hud.state === 0) { // COUNTDOWN
        overlayEl.textContent = hud.countdown > 0 ? Math.ceil(hud.countdown) : 'GO!';
        overlayEl.style.display = 'flex';
    } else if (hud.state === 2) { // FINISHED
        overlayEl.textContent = hud.playerPosition === 1 ? 'YOU WIN!' : `P${hud.playerPosition} — Race Over`;
        overlayEl.style.display = 'flex';
    } else {
        overlayEl.style.display = 'none';
    }

    const chassis = getChassisBody();
    if (chassis) {
        const p = chassis.position;
        const v = chassis.velocity;
        debugEl.textContent =
            `pos  ${p.x.toFixed(1)} ${p.y.toFixed(1)} ${p.z.toFixed(1)}\n` +
            `vel  ${v.length().toFixed(1)} m/s`;
    }
}

// Go
animate();
