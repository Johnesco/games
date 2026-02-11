// Car — ES module
// cannon-es RaycastVehicle + Three.js visual, full 6DOF
// Supports multiple car instances (player + AI)

import * as THREE from 'three';
import * as CANNON from 'cannon-es';

// ── Fixed geometry constants ─────────────────────────────────────────
const CHASSIS_HALF = { x: 1.0, y: 0.4, z: 1.8 };       // half-extents (m)
const MASS = 150;
const WHEEL_RADIUS = 0.4;
const SUSP_REST_LENGTH = 0.4;
const SUSP_TRAVEL = 0.5;

// ── Driving presets ─────────────────────────────────────────────────
const PRESETS = {
    casual: {
        label: 'Casual',
        engineForce:    600,
        brakeForce:     40,
        maxSteer:       0.50,
        topSpeed:       25,
        suspStiffness:  45,
        dampComp:       5.0,
        dampRel:        3.0,
        frictionSlip:   2.5,
        rollInfluence:  0.01,
        angularDamping: 0.6,
    },
    touring: {
        label: 'Touring',
        engineForce:    900,
        brakeForce:     45,
        maxSteer:       0.48,
        topSpeed:       35,
        suspStiffness:  50,
        dampComp:       4.5,
        dampRel:        2.5,
        frictionSlip:   2.0,
        rollInfluence:  0.02,
        angularDamping: 0.5,
    },
    sport: {
        label: 'Sport',
        engineForce:    1200,
        brakeForce:     50,
        maxSteer:       0.45,
        topSpeed:       45,
        suspStiffness:  60,
        dampComp:       4.4,
        dampRel:        2.3,
        frictionSlip:   1.8,
        rollInfluence:  0.04,
        angularDamping: 0.45,
    },
    race: {
        label: 'Race',
        engineForce:    1500,
        brakeForce:     55,
        maxSteer:       0.40,
        topSpeed:       55,
        suspStiffness:  65,
        dampComp:       4.2,
        dampRel:        2.2,
        frictionSlip:   1.5,
        rollInfluence:  0.05,
        angularDamping: 0.4,
    },
    drift: {
        label: 'Drift',
        engineForce:    1300,
        brakeForce:     35,
        maxSteer:       0.55,
        topSpeed:       40,
        suspStiffness:  40,
        dampComp:       3.5,
        dampRel:        2.0,
        frictionSlip:   1.0,
        rollInfluence:  0.08,
        angularDamping: 0.3,
    },
};

// Active tuning — starts as Touring, changed via setDrivingPreset()
const tuning = { ...PRESETS.touring };

// Wheel placement (local to chassis, -Z = front of car)
const WHEEL_POSITIONS = [
    new CANNON.Vec3(-0.9, 0, -1.3),  // 0 FL
    new CANNON.Vec3( 0.9, 0, -1.3),  // 1 FR
    new CANNON.Vec3(-0.9, 0,  1.3),  // 2 RL
    new CANNON.Vec3( 0.9, 0,  1.3),  // 3 RR
];

// ── Generic car factory ─────────────────────────────────────────────
export function createCarInstance(scene, world, color = 0xcc0000) {
    // ── Physics chassis ──────────────────────────────────────────────
    const chassisShape = new CANNON.Box(
        new CANNON.Vec3(CHASSIS_HALF.x, CHASSIS_HALF.y, CHASSIS_HALF.z)
    );
    const chassisBody = new CANNON.Body({ mass: MASS });
    chassisBody.addShape(chassisShape, new CANNON.Vec3(0, 0.1, 0));
    chassisBody.angularDamping = tuning.angularDamping;
    chassisBody.position.set(0, 2, 0);

    // ── RaycastVehicle ───────────────────────────────────────────────
    const vehicle = new CANNON.RaycastVehicle({
        chassisBody,
        indexRightAxis: 0,
        indexUpAxis: 1,
        indexForwardAxis: 2,
    });

    const wheelOptions = {
        radius: WHEEL_RADIUS,
        directionLocal: new CANNON.Vec3(0, -1, 0),
        suspensionStiffness: tuning.suspStiffness,
        suspensionRestLength: SUSP_REST_LENGTH,
        maxSuspensionTravel: SUSP_TRAVEL,
        frictionSlip: tuning.frictionSlip,
        dampingCompression: tuning.dampComp,
        dampingRelaxation: tuning.dampRel,
        rollInfluence: tuning.rollInfluence,
        axleLocal: new CANNON.Vec3(-1, 0, 0),
        chassisConnectionPointLocal: new CANNON.Vec3(),
        maxSuspensionForce: 100000,
        customSlidingRotationalSpeed: -30,
        useCustomSlidingRotationalSpeed: true,
    };

    WHEEL_POSITIONS.forEach(pos => {
        wheelOptions.chassisConnectionPointLocal.copy(pos);
        vehicle.addWheel(wheelOptions);
    });

    vehicle.addToWorld(world);

    // ── Visual chassis ───────────────────────────────────────────────
    const carMesh = new THREE.Group();

    const bodyW = CHASSIS_HALF.x * 2;
    const bodyH = CHASSIS_HALF.y * 2;
    const bodyL = CHASSIS_HALF.z * 2;

    const body = new THREE.Mesh(
        new THREE.BoxGeometry(bodyW, bodyH, bodyL),
        new THREE.MeshPhongMaterial({ color, shininess: 80, specular: 0x444444 }),
    );
    body.castShadow = true;
    carMesh.add(body);

    // Headlights (at -Z = physics forward)
    const hlGeo = new THREE.BoxGeometry(0.3, 0.15, 0.05);
    const hlMat = new THREE.MeshPhongMaterial({ color: 0xffffee, emissive: 0xaaaa66 });
    [-0.55, 0.55].forEach(x => {
        const hl = new THREE.Mesh(hlGeo, hlMat);
        hl.position.set(x, -0.05, -bodyL / 2 - 0.03);
        carMesh.add(hl);
    });

    // Brake lights (at +Z = physics rear)
    const blGeo = new THREE.BoxGeometry(0.3, 0.15, 0.05);
    const blOffMat = new THREE.MeshPhongMaterial({ color: 0x660000, emissive: 0x220000 });
    const blOnMat = new THREE.MeshPhongMaterial({ color: 0xff0000, emissive: 0xff0000, emissiveIntensity: 0.8 });
    const brakeLightMats = { off: blOffMat, on: blOnMat };
    const brakeLights = [];
    [-0.55, 0.55].forEach(x => {
        const bl = new THREE.Mesh(blGeo, blOffMat);
        bl.position.set(x, -0.05, bodyL / 2 + 0.03);
        carMesh.add(bl);
        brakeLights.push(bl);
    });

    scene.add(carMesh);

    // ── Visual wheels ────────────────────────────────────────────────
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

    return { chassisBody, vehicle, carMesh, wheelMeshes, brakeLights, brakeLightMats };
}

// ── Per-car controls (call BEFORE world.step) ────────────────────────
export function applyCarControlsTo(car, input, tuningOverride) {
    if (!car.vehicle) return;

    const t = tuningOverride || tuning;
    const speed = car.chassisBody.velocity.length();

    // ── Steering (speed-sensitive) ───────────────────────────────────
    let steer;
    if (typeof input.steerAmount === 'number') {
        // Continuous AI steering
        const speedFactor = Math.max(0.3, 1 - Math.min(speed / 40, 1) * 0.6);
        steer = input.steerAmount * t.maxSteer * speedFactor;
    } else {
        const steerDir = (input.left ? 1 : 0) + (input.right ? -1 : 0);
        const speedFactor = Math.max(0.3, 1 - Math.min(speed / 40, 1) * 0.6);
        steer = steerDir * t.maxSteer * speedFactor;
    }
    car.vehicle.setSteeringValue(steer, 0);
    car.vehicle.setSteeringValue(steer, 1);

    // ── Engine (rear-wheel drive) with speed-based power fade ────────
    const speedRatio = Math.min(speed / t.topSpeed, 1);
    const powerFade  = 1 - speedRatio * 0.9;
    let engineForce = 0;
    if (input.up)    engineForce =  t.engineForce * powerFade;
    if (input.down)  engineForce = -t.engineForce * 0.6 * powerFade;
    car.vehicle.applyEngineForce(engineForce, 2);
    car.vehicle.applyEngineForce(engineForce, 3);

    // ── Brakes ───────────────────────────────────────────────────────
    for (let i = 0; i < 4; i++) car.vehicle.setBrake(0, i);

    // Brake lights (on when reversing)
    const blMat = input.down ? car.brakeLightMats.on : car.brakeLightMats.off;
    car.brakeLights.forEach(bl => { bl.material = blMat; });
}

// ── Per-car visual sync (call AFTER world.step) ──────────────────────
export function syncCarVisualsFor(car) {
    if (!car.vehicle) return;

    car.carMesh.position.copy(car.chassisBody.position);
    car.carMesh.quaternion.copy(car.chassisBody.quaternion);

    for (let i = 0; i < 4; i++) {
        car.vehicle.updateWheelTransform(i);
        const t = car.vehicle.wheelInfos[i].worldTransform;
        car.wheelMeshes[i].position.copy(t.position);
        car.wheelMeshes[i].quaternion.copy(t.quaternion);
    }
}

// ── Per-car reset ────────────────────────────────────────────────────
export function resetCarInstance(car, position, quaternion) {
    if (!car.chassisBody) return;
    if (position) {
        car.chassisBody.position.copy(position);
    } else {
        car.chassisBody.position.set(0, 2, 0);
    }
    car.chassisBody.velocity.setZero();
    car.chassisBody.angularVelocity.setZero();
    if (quaternion) {
        car.chassisBody.quaternion.copy(quaternion);
    } else {
        car.chassisBody.quaternion.set(0, 0, 0, 1);
    }
}

// ── Player car — thin wrappers over generic functions ────────────────
let playerCar = null;

export function createCar(scene, world) {
    playerCar = createCarInstance(scene, world, 0xcc0000);
    return playerCar;
}

export function applyCarControls(input) {
    applyCarControlsTo(playerCar, input);
}

export function syncCarVisuals() {
    syncCarVisualsFor(playerCar);
}

export function resetCar(position, quaternion) {
    resetCarInstance(playerCar, position, quaternion);
}

// ── Driving preset switcher ──────────────────────────────────────────
export function setDrivingPreset(name) {
    const preset = PRESETS[name];
    if (!preset) return;
    Object.assign(tuning, preset);

    if (playerCar && playerCar.chassisBody) {
        playerCar.chassisBody.angularDamping = tuning.angularDamping;
    }
    if (playerCar && playerCar.vehicle) {
        for (let i = 0; i < playerCar.vehicle.wheelInfos.length; i++) {
            const w = playerCar.vehicle.wheelInfos[i];
            w.suspensionStiffness  = tuning.suspStiffness;
            w.dampingCompression   = tuning.dampComp;
            w.dampingRelaxation    = tuning.dampRel;
            w.frictionSlip         = tuning.frictionSlip;
            w.rollInfluence        = tuning.rollInfluence;
        }
    }
}

// ── Accessors ────────────────────────────────────────────────────────
export function getChassisBody() { return playerCar ? playerCar.chassisBody : null; }
export function getVehicle()     { return playerCar ? playerCar.vehicle : null; }

export function getSpeedMPH() {
    if (!playerCar || !playerCar.chassisBody) return 0;
    return Math.round(playerCar.chassisBody.velocity.length() * 2.237);
}
