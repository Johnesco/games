// Race — ES module
// AI opponents follow the track spline directly (no physics steering).
// Race state machine, lap tracking, position calculation.

import * as THREE from 'three';
import * as CANNON from 'cannon-es';
import { createCarInstance, applyCarControlsTo, syncCarVisualsFor, resetCarInstance } from './car3d.js';
import { getTrackCurve, getGridPositions } from './track.js';

// ── AI constants ─────────────────────────────────────────────────────
const AI_COLORS = [0x2255cc, 0x22aa44, 0xdd8800, 0xaa22cc, 0x22aaaa, 0xcc2255, 0x8866ff];
const AI_NAMES  = ['Blue', 'Green', 'Orange', 'Purple', 'Cyan', 'Pink', 'Violet'];
const AI_SPEEDS = [26, 28, 29, 30, 31, 32, 34];           // m/s spread for competition
const AI_LANE_OFFSETS = [-6, -4, -2, 0, 2, 4, 6];         // metres from centerline
const AI_HEIGHT_OFFSET = 0.75;            // chassis center above road surface

// ── Race state ───────────────────────────────────────────────────────
const STATES = { COUNTDOWN: 0, RACING: 1, FINISHED: 2 };
let state = STATES.COUNTDOWN;
let countdown = 3.0;
let raceTime = 0;
const totalLaps = 3;
let finishOrder = [];

// ── Track data ───────────────────────────────────────────────────────
let trackLength = 0;

// ── Racers array (player + AI) ──────────────────────────────────────
let racers = [];
let playerRacer = null;

// ── Curve sample cache (for player lap tracking) ────────────────────
const N_CURVE_SAMPLES = 400;
let curveSamples = null;

function buildCurveSamples(curve) {
    curveSamples = [];
    for (let i = 0; i < N_CURVE_SAMPLES; i++) {
        const t = i / N_CURVE_SAMPLES;
        curveSamples.push({ point: curve.getPointAt(t), t });
    }
}

function findClosestT(position) {
    let bestDist = Infinity;
    let bestT = 0;
    const px = position.x, pz = position.z;
    for (let i = 0; i < curveSamples.length; i++) {
        const s = curveSamples[i];
        const dx = s.point.x - px;
        const dz = s.point.z - pz;
        const dist = dx * dx + dz * dz;
        if (dist < bestDist) {
            bestDist = dist;
            bestT = s.t;
        }
    }
    return bestT;
}

// ═══════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════

export function initRace(playerCar, scene, world) {
    const curve = getTrackCurve();
    buildCurveSamples(curve);
    trackLength = curve.getLength();

    // Create AI cars
    const aiCars = [];
    for (let i = 0; i < 7; i++) {
        const car = createCarInstance(scene, world, AI_COLORS[i]);
        aiCars.push({
            car,
            name: AI_NAMES[i],
            targetSpeed: AI_SPEEDS[i],
            trackT: 0,
            prevT: 0,
            lap: 0,
            isPlayer: false,
            // Spline-following state
            curveT: 0,
            currentSpeed: 0,
            laneOffset: AI_LANE_OFFSETS[i],
        });
    }

    // Player racer
    playerRacer = {
        car: playerCar,
        name: 'Player',
        targetSpeed: 0,
        trackT: 0,
        prevT: 0,
        lap: 0,
        isPlayer: true,
    };
    racers = [playerRacer, ...aiCars];

    // Position all cars on the starting grid
    const gridSlots = getGridPositions(8);
    for (let i = 0; i < racers.length; i++) {
        const slot = gridSlots[i];
        resetCarInstance(racers[i].car, slot.position, slot.quaternion);
    }

    // Init race state
    state = STATES.COUNTDOWN;
    countdown = 3.0;
    raceTime = 0;
    finishOrder = [];

    // Init track positions
    for (const r of racers) {
        const t = findClosestT(r.car.chassisBody.position);
        r.trackT = t;
        r.prevT = t;
        r.lap = 0;
        if (!r.isPlayer) {
            r.curveT = t;
            r.currentSpeed = 0;
        }
    }

    return { aiCars, racers };
}

export function updateRace(dt, playerInput) {
    const curve = getTrackCurve();

    switch (state) {
        case STATES.COUNTDOWN:
            // Hold all cars with brakes
            for (const r of racers) {
                for (let i = 0; i < 4; i++) r.car.vehicle.setBrake(100, i);
                r.car.vehicle.applyEngineForce(0, 2);
                r.car.vehicle.applyEngineForce(0, 3);
            }
            countdown -= dt;
            if (countdown <= 0) {
                state = STATES.RACING;
                raceTime = 0;
                for (const r of racers) {
                    for (let i = 0; i < 4; i++) r.car.vehicle.setBrake(0, i);
                }
            }
            break;

        case STATES.RACING:
            raceTime += dt;

            // Player: normal physics controls
            applyCarControlsTo(playerRacer.car, playerInput);

            // AI: advance along curve (no physics controls)
            for (const r of racers) {
                if (r.isPlayer) continue;
                advanceAICurveT(r, dt, curve);
            }

            // Lap progress
            for (const r of racers) {
                r.prevT = r.trackT;
                if (r.isPlayer) {
                    r.trackT = findClosestT(r.car.chassisBody.position);
                } else {
                    r.trackT = r.curveT;
                }
                if (r.prevT > 0.85 && r.trackT < 0.15) r.lap++;
                if (r.prevT < 0.15 && r.trackT > 0.85) r.lap = Math.max(0, r.lap - 1);
            }

            // Win condition
            for (const r of racers) {
                if (r.lap >= totalLaps && !finishOrder.includes(r.name)) {
                    finishOrder.push(r.name);
                }
            }
            if (finishOrder.length > 0 && state === STATES.RACING) {
                state = STATES.FINISHED;
                const sorted = getRacePositionsSorted();
                for (const r of sorted) {
                    if (!finishOrder.includes(r.name)) finishOrder.push(r.name);
                }
            }
            break;

        case STATES.FINISHED:
            // Player can still drive
            applyCarControlsTo(playerRacer.car, playerInput);

            // AI: decelerate to stop
            for (const r of racers) {
                if (r.isPlayer) continue;
                advanceAICurveT(r, dt, curve, true);
            }
            break;
    }
}

export function syncAllRaceCars() {
    const curve = getTrackCurve();

    // AI cars: override position from curve (during racing/finished)
    if (state === STATES.RACING || state === STATES.FINISHED) {
        for (const r of racers) {
            if (r.isPlayer) continue;
            placeAIOnCurve(r, curve);
        }
    }

    // Sync visuals for all cars
    for (const r of racers) {
        syncCarVisualsFor(r.car);
    }
}

export function recoverPlayerCar() {
    if (!playerRacer) return;
    const curve = getTrackCurve();
    const t = findClosestT(playerRacer.car.chassisBody.position);
    const point = curve.getPointAt(t);
    const tangent = curve.getTangentAt(t).normalize();

    const pos = new CANNON.Vec3(point.x, point.y + 2, point.z);
    const m = new THREE.Matrix4();
    m.lookAt(new THREE.Vector3(), tangent, new THREE.Vector3(0, 1, 0));
    const q = new THREE.Quaternion().setFromRotationMatrix(m);
    const quat = new CANNON.Quaternion(q.x, q.y, q.z, q.w);

    resetCarInstance(playerRacer.car, pos, quat);
}

export function restartRace() {
    const gridSlots = getGridPositions(racers.length);
    for (let i = 0; i < racers.length; i++) {
        const slot = gridSlots[i];
        resetCarInstance(racers[i].car, slot.position, slot.quaternion);
    }

    state = STATES.COUNTDOWN;
    countdown = 3.0;
    raceTime = 0;
    finishOrder = [];

    for (const r of racers) {
        const t = findClosestT(r.car.chassisBody.position);
        r.trackT = t;
        r.prevT = t;
        r.lap = 0;
        if (!r.isPlayer) {
            r.curveT = t;
            r.currentSpeed = 0;
        }
    }
}

export function getRaceHUD() {
    const playerPos = getPlayerPosition();
    return {
        state,
        countdown,
        playerLap: playerRacer ? playerRacer.lap : 0,
        totalLaps,
        playerPosition: playerPos,
        totalCars: racers.length,
        raceTime,
        finishOrder,
    };
}

// ═══════════════════════════════════════════════════════════════════════
// AI spline following
// ═══════════════════════════════════════════════════════════════════════

function advanceAICurveT(racer, dt, curve, stopping) {
    const t = racer.curveT;

    // Sample curvature ahead at multiple distances — use worst
    let worstK = 0;
    for (const ahead of [0.02, 0.04, 0.07]) {
        const t1 = (t + ahead) % 1;
        const t2 = (t + ahead + 0.015) % 1;
        const tan1 = curve.getTangentAt(t1);
        const tan2 = curve.getTangentAt(t2);
        const dot = tan1.x * tan2.x + tan1.z * tan2.z;
        const k = 1 - Math.max(0, dot);
        if (k > worstK) worstK = k;
    }

    // Target speed: full on straights, reduced in curves
    let targetSpeed;
    if (stopping) {
        targetSpeed = 0;
    } else {
        targetSpeed = racer.targetSpeed;
        if (worstK > 0.005) {
            targetSpeed *= Math.max(0.35, 1 - worstK * 5);
        }
    }

    // Smooth acceleration / deceleration
    const accel = racer.currentSpeed < targetSpeed ? 14 : 22;
    const diff = targetSpeed - racer.currentSpeed;
    racer.currentSpeed += Math.sign(diff) * Math.min(Math.abs(diff), accel * dt);
    racer.currentSpeed = Math.max(0, racer.currentSpeed);

    // Advance t
    if (trackLength > 0) {
        racer.curveT = (racer.curveT + racer.currentSpeed * dt / trackLength) % 1;
    }
}

function placeAIOnCurve(racer, curve) {
    const t = racer.curveT;
    const point = curve.getPointAt(t);
    const tangent = curve.getTangentAt(t).normalize();

    // Right vector = tangent × worldUp
    const right = _v1.set(0, 0, 0);
    right.crossVectors(tangent, _up);
    if (right.lengthSq() < 0.001) right.set(1, 0, 0);
    right.normalize();

    // Banking: compute curvature sign for visual lean
    const dt2 = 0.005;
    const tA = ((t - dt2) % 1 + 1) % 1;
    const tB = (t + dt2) % 1;
    const tanA = curve.getTangentAt(tA);
    const tanB = curve.getTangentAt(tB);
    const crossY = tanA.x * tanB.z - tanA.z * tanB.x;
    const bankAngle = Math.max(-0.25, Math.min(0.25, crossY * 8));

    // Position: curve center + lateral offset + height offset
    const body = racer.car.chassisBody;
    body.position.set(
        point.x + right.x * racer.laneOffset,
        point.y + AI_HEIGHT_OFFSET,
        point.z + right.z * racer.laneOffset,
    );

    // Quaternion: car -Z = tangent (matching physics forward convention)
    _mat4.lookAt(_origin, tangent, _up);
    _q3.setFromRotationMatrix(_mat4);

    // Apply banking roll around tangent axis
    _qBank.setFromAxisAngle(tangent, bankAngle);
    _q3.premultiply(_qBank);

    body.quaternion.set(_q3.x, _q3.y, _q3.z, _q3.w);

    // Velocity matches speed along tangent (so collisions feel right)
    body.velocity.set(
        tangent.x * racer.currentSpeed,
        tangent.y * racer.currentSpeed,
        tangent.z * racer.currentSpeed,
    );

    // No spin
    body.angularVelocity.set(0, 0, 0);
}

// Reusable temp objects for placeAIOnCurve
const _v1 = new THREE.Vector3();
const _origin = new THREE.Vector3();
const _up = new THREE.Vector3(0, 1, 0);
const _mat4 = new THREE.Matrix4();
const _q3 = new THREE.Quaternion();
const _qBank = new THREE.Quaternion();

// ═══════════════════════════════════════════════════════════════════════
// Position calculation
// ═══════════════════════════════════════════════════════════════════════

function getRacePositionsSorted() {
    return [...racers].sort((a, b) => {
        const progressA = a.lap + a.trackT;
        const progressB = b.lap + b.trackT;
        return progressB - progressA;
    });
}

function getPlayerPosition() {
    if (!playerRacer) return 1;
    const sorted = getRacePositionsSorted();
    for (let i = 0; i < sorted.length; i++) {
        if (sorted[i].isPlayer) return i + 1;
    }
    return 1;
}
