// Car — ES module
// Pure kinematic car: position + heading + speed. No physics engine.
// Visual: procedural low-poly car meshes.

import * as THREE from 'three';
import { getTerrainInfo } from './track.js';

// ── Kinematic constants ─────────────────────────────────────────────
const RIDE_HEIGHT = 0.8;  // car center above road surface

// ── Driving presets ─────────────────────────────────────────────────
const PRESETS = {
    casual:  { label:'Casual',  accel:16, brake:20, topSpeed:30, turnSpeed:1.8 },
    touring: { label:'Touring', accel:20, brake:25, topSpeed:42, turnSpeed:2.0 },
    sport:   { label:'Sport',   accel:25, brake:30, topSpeed:52, turnSpeed:2.2 },
    race:    { label:'Race',    accel:30, brake:35, topSpeed:65, turnSpeed:2.4 },
    drift:   { label:'Drift',   accel:22, brake:25, topSpeed:48, turnSpeed:2.5 },
};
const tuning = { ...PRESETS.touring };

// ── Visual constants ────────────────────────────────────────────────
const WHEEL_RADIUS = 0.6;
const SUSP_REST    = 0.5;

const WHEEL_OFFSETS = [
    new THREE.Vector3(-0.9, -0.2, -1.3),   // FL
    new THREE.Vector3( 0.9, -0.2, -1.3),   // FR
    new THREE.Vector3(-0.9, -0.2,  1.3),   // RL
    new THREE.Vector3( 0.9, -0.2,  1.3),   // RR
];

const BODY_STYLES = [
    { name: 'sedan',  hoodH: 0.22, hoodL: 1.0, cabinH: 0.50, cabinL: 1.2, cabinY: 0.55, cabinZ: 0.0 },
    { name: 'sport',  hoodH: 0.18, hoodL: 1.2, cabinH: 0.38, cabinL: 1.0, cabinY: 0.45, cabinZ: 0.1 },
    { name: 'suv',    hoodH: 0.25, hoodL: 0.9, cabinH: 0.60, cabinL: 1.3, cabinY: 0.62, cabinZ: -0.05 },
];

// ── Reusable temp objects for terrain-aware physics ─────────────────
const _fwdSurf = new THREE.Vector3();
const _lookMat = new THREE.Matrix4();
const _zero    = new THREE.Vector3();
const _yAxis   = new THREE.Vector3(0, 1, 0);

// Reusable temp vector
const _wv = new THREE.Vector3();

// ═════════════════════════════════════════════════════════════════════
// Car factory — visual mesh + kinematic state (no physics body)
// ═════════════════════════════════════════════════════════════════════
export function createCarInstance(scene, world, color = 0xcc0000, bodyStyleIndex = 0) {

    // ── Visual chassis ──────────────────────────────────────────────
    const carMesh = new THREE.Group();
    const mainMat = new THREE.MeshPhongMaterial({ color, shininess: 80, specular: 0x444444 });
    const darkColor = new THREE.Color(color).offsetHSL(0, 0, -0.15);
    const darkMat = new THREE.MeshPhongMaterial({ color: darkColor, shininess: 60 });
    const glassMat = new THREE.MeshPhongMaterial({
        color: 0x4488cc, transparent: true, opacity: 0.45, shininess: 120, specular: 0x888888,
    });
    const bodyStyle = BODY_STYLES[bodyStyleIndex % BODY_STYLES.length];

    const lower = new THREE.Mesh(new THREE.BoxGeometry(2.0, 0.35, 3.6), mainMat);
    lower.position.y = -0.05;
    lower.castShadow = true;
    carMesh.add(lower);

    const hood = new THREE.Mesh(new THREE.BoxGeometry(1.8, bodyStyle.hoodH, bodyStyle.hoodL), mainMat);
    hood.position.set(0, 0.15, -1.1);
    hood.castShadow = true;
    carMesh.add(hood);

    const cabin = new THREE.Mesh(new THREE.BoxGeometry(1.6, bodyStyle.cabinH, bodyStyle.cabinL), mainMat);
    cabin.position.set(0, bodyStyle.cabinY, bodyStyle.cabinZ);
    cabin.castShadow = true;
    carMesh.add(cabin);

    const trunk = new THREE.Mesh(new THREE.BoxGeometry(1.8, 0.25, 0.9), mainMat);
    trunk.position.set(0, 0.15, 1.15);
    trunk.castShadow = true;
    carMesh.add(trunk);

    const ws = new THREE.Mesh(new THREE.PlaneGeometry(1.5, bodyStyle.cabinH * 0.85), glassMat);
    ws.position.set(0, bodyStyle.cabinY + 0.05, bodyStyle.cabinZ - bodyStyle.cabinL / 2 - 0.01);
    ws.rotation.x = -0.3;
    carMesh.add(ws);

    const rw = new THREE.Mesh(new THREE.PlaneGeometry(1.5, bodyStyle.cabinH * 0.7), glassMat);
    rw.position.set(0, bodyStyle.cabinY, bodyStyle.cabinZ + bodyStyle.cabinL / 2 + 0.01);
    rw.rotation.x = 0.35;
    rw.rotation.y = Math.PI;
    carMesh.add(rw);

    [-1.01, 1.01].forEach(x => {
        const skirt = new THREE.Mesh(new THREE.BoxGeometry(0.08, 0.22, 3.4), darkMat);
        skirt.position.set(x, -0.1, 0);
        carMesh.add(skirt);
    });

    const bumperF = new THREE.Mesh(new THREE.BoxGeometry(2.0, 0.18, 0.12), darkMat);
    bumperF.position.set(0, -0.12, -1.83);
    carMesh.add(bumperF);

    const bumperR = new THREE.Mesh(new THREE.BoxGeometry(2.0, 0.18, 0.12), darkMat);
    bumperR.position.set(0, -0.12, 1.83);
    carMesh.add(bumperR);

    const hlGeo = new THREE.BoxGeometry(0.3, 0.15, 0.08);
    const hlMat = new THREE.MeshPhongMaterial({ color: 0xffffee, emissive: 0xaaaa66, emissiveIntensity: 0.6 });
    [-0.65, 0.65].forEach(x => {
        const hl = new THREE.Mesh(hlGeo, hlMat);
        hl.position.set(x, 0.0, -1.84);
        carMesh.add(hl);
    });

    const blGeo = new THREE.BoxGeometry(0.3, 0.12, 0.06);
    const blOffMat = new THREE.MeshPhongMaterial({ color: 0x660000, emissive: 0x220000 });
    const blOnMat  = new THREE.MeshPhongMaterial({ color: 0xff0000, emissive: 0xff0000, emissiveIntensity: 0.8 });
    const brakeLightMats = { off: blOffMat, on: blOnMat };
    const brakeLights = [];
    [-0.65, 0.65].forEach(x => {
        const bl = new THREE.Mesh(blGeo, blOffMat);
        bl.position.set(x, 0.0, 1.84);
        carMesh.add(bl);
        brakeLights.push(bl);
    });

    scene.add(carMesh);

    // ── Visual wheels ───────────────────────────────────────────────
    const wheelMeshes = [];
    for (let i = 0; i < 4; i++) {
        const wg = new THREE.Group();
        const tire = new THREE.Mesh(
            new THREE.CylinderGeometry(WHEEL_RADIUS, WHEEL_RADIUS, 0.25, 16),
            new THREE.MeshPhongMaterial({ color: 0x1a1a1a, shininess: 30 }),
        );
        tire.rotation.z = Math.PI / 2;
        wg.add(tire);

        const rim = new THREE.Mesh(
            new THREE.CylinderGeometry(WHEEL_RADIUS * 0.6, WHEEL_RADIUS * 0.6, 0.27, 8),
            new THREE.MeshPhongMaterial({ color: 0xaaaaaa, shininess: 100 }),
        );
        rim.rotation.z = Math.PI / 2;
        wg.add(rim);

        const hub = new THREE.Mesh(
            new THREE.CylinderGeometry(WHEEL_RADIUS * 0.2, WHEEL_RADIUS * 0.2, 0.29, 6),
            new THREE.MeshPhongMaterial({ color: 0x666666, shininess: 60 }),
        );
        hub.rotation.z = Math.PI / 2;
        wg.add(hub);

        wg.castShadow = true;
        scene.add(wg);
        wheelMeshes.push(wg);
    }

    // ── Kinematic state ─────────────────────────────────────────────
    const pos = new THREE.Vector3(0, 3, 0);

    // chassisBody shim — lets race.js / main3d.js read position/velocity
    // without knowing the car is kinematic
    const chassisBody = {
        position: pos,
        velocity: new THREE.Vector3(),
        quaternion: new THREE.Quaternion(),
        angularVelocity: { x: 0, y: 0, z: 0, set() {} },
        addEventListener() {},
        pointToWorldFrame(local, result) {
            _wv.set(local.x, local.y, local.z);
            _wv.applyQuaternion(chassisBody.quaternion);
            _wv.add(pos);
            result.x = _wv.x; result.y = _wv.y; result.z = _wv.z;
        },
    };

    return {
        chassisBody, carMesh, wheelMeshes, brakeLights, brakeLightMats,
        // Kinematic fields (mutated directly by update functions)
        heading: 0,
        speed: 0,
    };
}

// ═════════════════════════════════════════════════════════════════════
// Kinematic update — the ONLY movement logic. No physics engine.
// ═════════════════════════════════════════════════════════════════════
export function applyCarControlsTo(car, input, dt) {
    if (!dt) dt = 1 / 60;
    const t = tuning;

    // ── Terrain at current position ──────────────────────────────────
    const terrain = getTerrainInfo(
        car.chassisBody.position.x,
        car.chassisBody.position.z,
    );
    const N = terrain.normal;

    // Forward / right from current heading (for force projections)
    const fwdX0 = -Math.sin(car.heading);
    const fwdZ0 = -Math.cos(car.heading);
    const rightX = -fwdZ0;
    const rightZ = fwdX0;

    // ── Acceleration / braking ───────────────────────────────────────
    if (input.up)        car.speed += t.accel * dt;
    else if (input.down) car.speed -= t.brake * dt;

    // Natural drag (speed-proportional — rolling resistance)
    car.speed -= car.speed * 2.0 * dt;

    // ── Slope gravity (hills affect speed) ───────────────────────────
    // Project world gravity onto surface plane, then dot with forward.
    // g = (0, -9.82, 0);  g·N = -9.82*Ny
    // g_surface = g - (g·N)*N  →  XZ components = 9.82*Ny*(Nx, Nz)
    const gDotN  = -9.82 * N.y;
    const gSurfX = -gDotN * N.x;
    const gSurfZ = -gDotN * N.z;
    car.speed += (gSurfX * fwdX0 + gSurfZ * fwdZ0) * dt;

    // Clamp
    car.speed = Math.max(-8, Math.min(t.topSpeed, car.speed));
    if (Math.abs(car.speed) < 0.1 && !input.up && !input.down) car.speed = 0;

    // Handbrake: strong deceleration
    if (input.handbrake) car.speed *= Math.pow(0.3, dt);

    // ── Banking assist (banked curves help steer) ────────────────────
    // Lateral component of surface gravity → natural heading correction.
    // heading_rate = lateral_accel / speed  (centripetal formula v/r = a/v)
    const gLateral = gSurfX * rightX + gSurfZ * rightZ;
    if (Math.abs(car.speed) > 2) {
        car.heading += gLateral / Math.max(Math.abs(car.speed), 5) * dt;
    }

    // ── Steering (no turn at zero speed) ─────────────────────────────
    let steer = 0;
    if (input.left)  steer =  1;
    if (input.right) steer = -1;
    const speedFactor = Math.min(Math.abs(car.speed) / 5, 1);
    car.heading += steer * t.turnSpeed * speedFactor * dt;

    // ── Move (using updated heading) ─────────────────────────────────
    const fwdX = -Math.sin(car.heading);
    const fwdZ = -Math.cos(car.heading);
    car.chassisBody.position.x += fwdX * car.speed * dt;
    car.chassisBody.position.z += fwdZ * car.speed * dt;
    car.chassisBody.position.y = terrain.height + RIDE_HEIGHT;

    // ── Update shim velocity ─────────────────────────────────────────
    car.chassisBody.velocity.set(fwdX * car.speed, 0, fwdZ * car.speed);

    // ── Visual tilt: orient car to match surface ─────────────────────
    // Project forward onto surface plane → car pitches on hills, banks on curves
    const fwdDotN = fwdX * N.x + fwdZ * N.z;
    _fwdSurf.set(fwdX - fwdDotN * N.x, -fwdDotN * N.y, fwdZ - fwdDotN * N.z);
    if (_fwdSurf.lengthSq() > 0.001) {
        _fwdSurf.normalize();
        _lookMat.lookAt(_zero, _fwdSurf, N);
        car.chassisBody.quaternion.setFromRotationMatrix(_lookMat);
    } else {
        car.chassisBody.quaternion.setFromAxisAngle(_yAxis, car.heading);
    }

    // Brake lights
    const blMat = (input.down || input.handbrake) ? car.brakeLightMats.on : car.brakeLightMats.off;
    car.brakeLights.forEach(bl => { bl.material = blMat; });
}

// ═════════════════════════════════════════════════════════════════════
// Visual sync — copy kinematic state to Three.js meshes
// ═════════════════════════════════════════════════════════════════════
export function syncCarVisualsFor(car) {
    car.carMesh.position.copy(car.chassisBody.position);
    car.carMesh.quaternion.copy(car.chassisBody.quaternion);

    for (let i = 0; i < 4; i++) {
        const off = WHEEL_OFFSETS[i];
        _wv.copy(off).applyQuaternion(car.chassisBody.quaternion).add(car.chassisBody.position);
        car.wheelMeshes[i].position.set(_wv.x, _wv.y - SUSP_REST, _wv.z);
        car.wheelMeshes[i].quaternion.copy(car.chassisBody.quaternion);
    }
}

// ═════════════════════════════════════════════════════════════════════
// Hold / reset / recover
// ═════════════════════════════════════════════════════════════════════
export function holdCar(car) {
    car.speed = 0;
    car.chassisBody.velocity.set(0, 0, 0);
}

export function resetCarInstance(car, position, quaternion) {
    if (position) {
        car.chassisBody.position.set(position.x, position.y, position.z);
    }
    if (quaternion) {
        // Extract yaw from quaternion
        const q = new THREE.Quaternion(quaternion.x, quaternion.y, quaternion.z, quaternion.w);
        const e = new THREE.Euler().setFromQuaternion(q, 'YXZ');
        car.heading = e.y;
        car.chassisBody.quaternion.copy(q);
    }
    car.speed = 0;
    car.chassisBody.velocity.set(0, 0, 0);
}

// ═════════════════════════════════════════════════════════════════════
// Player car wrappers (backward-compatible exports)
// ═════════════════════════════════════════════════════════════════════
let playerCar = null;

export function createCar(scene, world) {
    playerCar = createCarInstance(scene, world, 0xcc0000);
    return playerCar;
}

export function setDrivingPreset(name) {
    const preset = PRESETS[name];
    if (!preset) return;
    Object.assign(tuning, preset);
}

export function getChassisBody()  { return playerCar ? playerCar.chassisBody : null; }
export function getSpeedMPH()     { return playerCar ? Math.round(Math.abs(playerCar.speed) * 2.237) : 0; }
export function getTopSpeed()     { return tuning.topSpeed; }

// Stubs for backward compatibility (main3d.js imports these)
export function postPhysicsStabilize() {}
export const carMaterial = null;
