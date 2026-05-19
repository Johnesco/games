// Track — ES module
// Diverse closed-loop circuit with banked turns, hills, guard rails,
// CANNON.Heightfield road physics, CANNON.Box rail physics, procedural asphalt texture.

import * as THREE from 'three';
import * as CANNON from 'cannon-es';

// ── Track constants ────────────────────────────────────────────────────
const HALF_WIDTH    = 12;         // 24 m total road width (8 cars × 3 m)
const N_SAMPLES     = 200;        // visual cross-sections (smooth rendering)
const N_PHYSICS     = 50;         // physics cross-sections (fewer edges = no jitter)
const BANK_MAX_DEG  = 18;         // max banking angle in degrees
const K_MAX         = 0.025;      // curvature at which banking saturates
const SMOOTH_RADIUS = 8;          // visual banking smoothing radius
const PHYS_SMOOTH   = 2;          // physics banking smoothing radius (proportional)

// Guard rail constants
const RAIL_HEIGHT    = 1.2;       // metres
const RAIL_THICKNESS = 0.5;
const RAIL_INTERVAL  = 2;         // physics box every Nth sample

// ── Ground material (exported for contact material setup) ─────────────
export const groundMaterial = new CANNON.Material('ground');

// ── Module state ───────────────────────────────────────────────────────
let trackCurve = null;
let trackMesh  = null;
let trackBody  = null;
let startPos   = null;
let startQuat  = null;

// ── Heightfield data (for getTerrainInfo export) ────────────────────
let hfGrid = null;
let hfMeta = null;
const _terrainNormal = new THREE.Vector3(0, 1, 0);

// ═══════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════

export function createTrack(scene, world) {
    trackCurve = buildCurve();

    // ── Fine banking (visual + guard rails) ──────────────────────────
    const bankAngles = computeBankingArray(trackCurve, N_SAMPLES);
    const smoothed   = smoothArray(bankAngles, SMOOTH_RADIUS);

    const visual = buildTrackGeometry(trackCurve, smoothed, N_SAMPLES);

    // ── Road physics: HEIGHTFIELD (fast grid collision) ──────────────
    buildTrackHeightfield(trackCurve, smoothed, world);

    // ── Road visual mesh (fine — detailed rendering) ─────────────────
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(visual.positions, 3));
    geo.setAttribute('uv',       new THREE.BufferAttribute(visual.uvs, 2));
    geo.setAttribute('normal',   new THREE.BufferAttribute(visual.normals, 3));
    geo.setIndex(new THREE.BufferAttribute(visual.indices, 1));
    geo.computeVertexNormals();

    const mat = new THREE.MeshPhongMaterial({
        map: createTrackTexture(),
        side: THREE.DoubleSide,
        shininess: 8,
    });
    trackMesh = new THREE.Mesh(geo, mat);
    trackMesh.receiveShadow = true;
    scene.add(trackMesh);

    // ── Guard rails (visual uses fine, physics uses own sampling) ────
    buildGuardRails(scene, world, trackCurve, smoothed);

    // ── Start line visual ────────────────────────────────────────────
    buildStartLine(scene, trackCurve);

    // ── Start transform ──────────────────────────────────────────────
    computeStartTransform(trackCurve);

    return { mesh: trackMesh, body: trackBody, curve: trackCurve };
}

export function getTrackStart() {
    return { position: startPos, quaternion: startQuat };
}

// ═══════════════════════════════════════════════════════════════════════
// Track path — diverse circuit
// ═══════════════════════════════════════════════════════════════════════

function buildCurve() {
    const pts = [
        // Oval: counter-clockwise, start/finish on flat straight (+X direction)
        new THREE.Vector3(-80, 4, -50),     // t=0, start/finish

        new THREE.Vector3( 80, 4, -50),     // end of start straight

        // Right turn (east end)
        new THREE.Vector3( 130, 4, -25),
        new THREE.Vector3( 140, 4,   0),    // apex
        new THREE.Vector3( 130, 4,  25),

        // Hill straight (traveling -X)
        new THREE.Vector3( 40, 4,  50),     // approaching hill
        new THREE.Vector3(  0, 10, 50),     // hill peak
        new THREE.Vector3(-40, 4,  50),     // past hill

        // Left turn (west end)
        new THREE.Vector3(-130, 4,  25),
        new THREE.Vector3(-140, 4,   0),    // apex
        new THREE.Vector3(-130, 4, -25),

        // Transition back to start straight
        new THREE.Vector3(-100, 4, -50),
    ];

    return new THREE.CatmullRomCurve3(pts, true, 'catmullrom', 0.5);
}

// ═══════════════════════════════════════════════════════════════════════
// Frame / curvature / banking helpers
// ═══════════════════════════════════════════════════════════════════════

function getFrame(curve, t) {
    const T = curve.getTangentAt(t).normalize();
    const worldUp = new THREE.Vector3(0, 1, 0);
    const R = new THREE.Vector3().crossVectors(T, worldUp).normalize();
    if (R.lengthSq() < 0.001) R.set(1, 0, 0);
    const U = new THREE.Vector3().crossVectors(R, T).normalize();
    return { T, R, U };
}

function getCurvature(curve, t) {
    const dt = 0.005;
    const t0 = ((t - dt) % 1 + 1) % 1;
    const t1 = ((t + dt) % 1 + 1) % 1;
    const T0 = curve.getTangentAt(t0);
    const T1 = curve.getTangentAt(t1);

    const a0 = Math.atan2(T0.x, T0.z);
    const a1 = Math.atan2(T1.x, T1.z);
    let da = a1 - a0;
    if (da >  Math.PI) da -= 2 * Math.PI;
    if (da < -Math.PI) da += 2 * Math.PI;

    const arcLen = curve.getLength() * 2 * dt;
    return da / arcLen;
}

function bankingAngle(curvature) {
    const absK = Math.abs(curvature);
    const ratio = Math.min(absK / K_MAX, 1);
    const ss = ratio * ratio * (3 - 2 * ratio);
    const angle = ss * (BANK_MAX_DEG * Math.PI / 180);
    return -Math.sign(curvature) * angle;
}

function computeBankingArray(curve, n) {
    const arr = new Float32Array(n);
    for (let i = 0; i < n; i++) {
        arr[i] = bankingAngle(getCurvature(curve, i / n));
    }
    return arr;
}

function smoothArray(arr, radius) {
    const n = arr.length;
    const out = new Float32Array(n);
    for (let i = 0; i < n; i++) {
        let sum = 0, wSum = 0;
        for (let j = -radius; j <= radius; j++) {
            const idx = ((i + j) % n + n) % n;
            const w = radius + 1 - Math.abs(j);
            sum += arr[idx] * w;
            wSum += w;
        }
        out[i] = sum / wSum;
    }
    return out;
}

function applyBanking(T, R, U, angle) {
    const quat = new THREE.Quaternion().setFromAxisAngle(T, angle);
    return {
        R: R.clone().applyQuaternion(quat),
        U: U.clone().applyQuaternion(quat),
    };
}

// ═══════════════════════════════════════════════════════════════════════
// Road surface geometry
// ═══════════════════════════════════════════════════════════════════════

function buildTrackGeometry(curve, bankAngles, n) {
    const vCount = (n + 1) * 2;
    const positions = new Float32Array(vCount * 3);
    const uvs       = new Float32Array(vCount * 2);
    const normals   = new Float32Array(vCount * 3);

    const lengths = curve.getLengths(n);

    for (let i = 0; i <= n; i++) {
        const t = (i % n) / n;
        const { T, R, U } = getFrame(curve, t);
        const bAngle = bankAngles[i % n];
        const { R: bR, U: bU } = applyBanking(T, R, U, bAngle);

        const center = curve.getPointAt(t);
        const left   = center.clone().add(bR.clone().multiplyScalar(-HALF_WIDTH));
        const right  = center.clone().add(bR.clone().multiplyScalar( HALF_WIDTH));

        const vi = i * 2;
        positions[vi * 3]     = left.x;
        positions[vi * 3 + 1] = left.y;
        positions[vi * 3 + 2] = left.z;
        positions[(vi + 1) * 3]     = right.x;
        positions[(vi + 1) * 3 + 1] = right.y;
        positions[(vi + 1) * 3 + 2] = right.z;

        const v = lengths[i] / 20;
        uvs[vi * 2]         = 0;    uvs[vi * 2 + 1]     = v;
        uvs[(vi + 1) * 2]   = 1;    uvs[(vi + 1) * 2 + 1] = v;

        normals[vi * 3]     = bU.x;  normals[vi * 3 + 1] = bU.y;  normals[vi * 3 + 2] = bU.z;
        normals[(vi + 1) * 3] = bU.x; normals[(vi + 1) * 3 + 1] = bU.y; normals[(vi + 1) * 3 + 2] = bU.z;
    }

    const triCount = n * 2;
    const indices  = new Uint32Array(triCount * 3);
    for (let i = 0; i < n; i++) {
        const a = i * 2, b = a + 1, c = a + 2, d = a + 3;
        const idx = i * 6;
        indices[idx]     = a;  indices[idx + 1] = b;  indices[idx + 2] = c;
        indices[idx + 3] = b;  indices[idx + 4] = d;  indices[idx + 5] = c;
    }

    return { positions, uvs, indices, normals };
}

// ═══════════════════════════════════════════════════════════════════════
// Heightfield generation — fast grid-based ground collision
// ═══════════════════════════════════════════════════════════════════════

function buildTrackHeightfield(curve, bankAngles, world) {
    // Heightfield covers a generous area around the track
    const ELEM   = 2;      // metres per grid cell
    const HALF_X = 200;    // ±200 m in X
    const HALF_Z = 120;    // ±120 m in Z
    const NX     = Math.floor(HALF_X * 2 / ELEM) + 1;  // 201
    const NZ     = Math.floor(HALF_Z * 2 / ELEM) + 1;  // 121

    // Pre-sample the curve (position, right vector, banking)
    const NS = 400;
    const samples = [];
    for (let k = 0; k < NS; k++) {
        const t = k / NS;
        const p = curve.getPointAt(t);
        const { R } = getFrame(curve, t);
        const bIdx = Math.floor(t * bankAngles.length) % bankAngles.length;
        samples.push({ x: p.x, y: p.y, z: p.z, rx: R.x, rz: R.z, bank: bankAngles[bIdx] });
    }

    // Build height matrix: data[i][j], i = X axis, j = Z axis (after rotation)
    const data = [];
    for (let i = 0; i < NX; i++) {
        const row = [];
        const wx = -HALF_X + i * ELEM;
        for (let j = 0; j < NZ; j++) {
            const wz = HALF_Z - j * ELEM;  // j increases → Z decreases (rotation convention)
            row.push(sampleHeight(wx, wz, samples));
        }
        data.push(row);
    }

    // Store for terrain queries (used by getTerrainInfo)
    hfGrid = data;
    hfMeta = { ELEM, HALF_X, HALF_Z, NX, NZ };

    const hfShape = new CANNON.Heightfield(data, { elementSize: ELEM });
    trackBody = new CANNON.Body({ mass: 0, material: groundMaterial });
    trackBody.addShape(hfShape);
    trackBody.position.set(-HALF_X, 0, HALF_Z);
    trackBody.quaternion.setFromEuler(-Math.PI / 2, 0, 0);
    world.addBody(trackBody);
}

function sampleHeight(wx, wz, samples) {
    // Find closest curve sample
    let bestD2 = Infinity, bestIdx = 0;
    for (let k = 0; k < samples.length; k++) {
        const dx = samples[k].x - wx;
        const dz = samples[k].z - wz;
        const d2 = dx * dx + dz * dz;
        if (d2 < bestD2) { bestD2 = d2; bestIdx = k; }
    }

    const s = samples[bestIdx];
    const dist = Math.sqrt(bestD2);

    // Lateral offset from centerline (projected onto right vector)
    const dx = wx - s.x;
    const dz = wz - s.z;
    const lateral = dx * s.rx + dz * s.rz;

    let h;
    const hw = HALF_WIDTH;
    if (Math.abs(lateral) <= hw) {
        // On track: centerline height + banking
        h = s.y + lateral * Math.sin(s.bank);
    } else if (Math.abs(lateral) <= hw + 6) {
        // Shoulder: fade banking to flat centerline height
        const edgeH = s.y + Math.sign(lateral) * hw * Math.sin(s.bank);
        const t = (Math.abs(lateral) - hw) / 6;
        const ss = t * t * (3 - 2 * t);  // smoothstep
        h = edgeH * (1 - ss) + s.y * ss;
    } else {
        // Off track: flat at centerline height
        h = s.y;
    }

    // Far from any track: blend down to ground level (Y=0)
    const fadeStart = hw + 15;
    const fadeEnd   = hw + 50;
    if (dist > fadeStart) {
        const t = Math.min((dist - fadeStart) / (fadeEnd - fadeStart), 1);
        h = h * (1 - t * t);
    }

    return Math.max(h, 0);
}

// ═══════════════════════════════════════════════════════════════════════
// Procedural asphalt texture
// ═══════════════════════════════════════════════════════════════════════

function createTrackTexture() {
    const size = 512;
    const canvas = document.createElement('canvas');
    canvas.width  = size;
    canvas.height = size;
    const ctx = canvas.getContext('2d');

    ctx.fillStyle = '#3a3a3a';
    ctx.fillRect(0, 0, size, size);

    for (let i = 0; i < 12000; i++) {
        const x = Math.random() * size;
        const y = Math.random() * size;
        const b = Math.floor(Math.random() * 30 + 40);
        ctx.fillStyle = `rgb(${b},${b},${b})`;
        ctx.fillRect(x, y, 1, 1);
    }

    // Edge lines (white)
    ctx.fillStyle = 'rgba(255,255,255,0.9)';
    ctx.fillRect(Math.round(0.03 * size), 0, Math.round(0.03 * size), size);
    ctx.fillRect(Math.round(0.94 * size), 0, Math.round(0.03 * size), size);

    // Dashed center line
    ctx.fillStyle = 'rgba(255,255,255,0.85)';
    const cx0 = Math.round(0.49 * size), cw = Math.round(0.02 * size);
    for (let y = 0; y < size; y += 60) {
        ctx.fillRect(cx0, y, cw, 30);
    }

    const tex = new THREE.CanvasTexture(canvas);
    tex.wrapS = THREE.ClampToEdgeWrapping;
    tex.wrapT = THREE.RepeatWrapping;
    return tex;
}

// ═══════════════════════════════════════════════════════════════════════
// Guard rails — visual wall strips + physics box bodies
// ═══════════════════════════════════════════════════════════════════════

function buildGuardRails(scene, world, curve, bankAngles) {
    // Visual: smooth wall mesh on each edge
    const wallMat = new THREE.MeshPhongMaterial({
        color: 0xbbbbbb,
        side: THREE.DoubleSide,
        shininess: 15,
    });
    buildWallMesh(scene, curve, bankAngles, -1, wallMat);
    buildWallMesh(scene, curve, bankAngles, +1, wallMat);

    // Physics: box bodies at intervals on each edge
    buildWallBodies(world, curve, bankAngles);
}

function buildWallMesh(scene, curve, bankAngles, side, material) {
    const n = N_SAMPLES;
    const vCount = (n + 1) * 2;
    const positions = new Float32Array(vCount * 3);

    for (let i = 0; i <= n; i++) {
        const t = (i % n) / n;
        const { T, R, U } = getFrame(curve, t);
        const bAngle = bankAngles[i % n];
        const { R: bR } = applyBanking(T, R, U, bAngle);

        const center = curve.getPointAt(t);
        const edge   = center.clone().add(bR.clone().multiplyScalar(side * HALF_WIDTH));

        const vi = i * 2;
        // Bottom vertex — at road edge
        positions[vi * 3]     = edge.x;
        positions[vi * 3 + 1] = edge.y;
        positions[vi * 3 + 2] = edge.z;
        // Top vertex — RAIL_HEIGHT above edge
        positions[(vi + 1) * 3]     = edge.x;
        positions[(vi + 1) * 3 + 1] = edge.y + RAIL_HEIGHT;
        positions[(vi + 1) * 3 + 2] = edge.z;
    }

    const triCount = n * 2;
    const indices  = new Uint32Array(triCount * 3);
    for (let i = 0; i < n; i++) {
        const a = i * 2, b = a + 1, c = a + 2, d = a + 3;
        const idx = i * 6;
        indices[idx]     = a;  indices[idx + 1] = b;  indices[idx + 2] = c;
        indices[idx + 3] = b;  indices[idx + 4] = d;  indices[idx + 5] = c;
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    geo.computeVertexNormals();

    const mesh = new THREE.Mesh(geo, material);
    mesh.receiveShadow = true;
    scene.add(mesh);
}

function buildWallBodies(world, curve, bankAngles) {
    const railCount = Math.floor(N_SAMPLES / RAIL_INTERVAL);
    const totalLen  = curve.getLength();
    const railLen   = (totalLen / railCount) * 1.15;   // 15 % overlap
    const railHalf  = new CANNON.Vec3(RAIL_THICKNESS / 2, RAIL_HEIGHT / 2, railLen / 2);
    const railShape = new CANNON.Box(railHalf);         // shared shape

    for (let i = 0; i < railCount; i++) {
        const sampleIdx = i * RAIL_INTERVAL;
        const t = sampleIdx / N_SAMPLES;

        const { T, R, U } = getFrame(curve, t);
        const bAngle = bankAngles[sampleIdx];
        const { R: bR } = applyBanking(T, R, U, bAngle);

        const center = curve.getPointAt(t);

        // Box orientation: z-axis = tangent, y-axis ≈ world up
        const m4 = new THREE.Matrix4();
        m4.lookAt(new THREE.Vector3(), T, new THREE.Vector3(0, 1, 0));
        const q3 = new THREE.Quaternion().setFromRotationMatrix(m4);
        const cq = new CANNON.Quaternion(q3.x, q3.y, q3.z, q3.w);

        for (const side of [-1, 1]) {
            const edge = center.clone().add(bR.clone().multiplyScalar(side * HALF_WIDTH));

            const body = new CANNON.Body({ mass: 0 });
            body.addShape(railShape);
            body.position.set(
                edge.x + R.x * side * RAIL_THICKNESS / 2,
                edge.y + RAIL_HEIGHT / 2,
                edge.z + R.z * side * RAIL_THICKNESS / 2,
            );
            body.quaternion.copy(cq);
            world.addBody(body);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Start position / quaternion
// ═══════════════════════════════════════════════════════════════════════

function computeStartTransform(curve) {
    const center  = curve.getPointAt(0);
    const tangent = curve.getTangentAt(0).normalize();

    startPos = new CANNON.Vec3(center.x, center.y + 2, center.z);

    const m = new THREE.Matrix4();
    m.lookAt(new THREE.Vector3(), tangent, new THREE.Vector3(0, 1, 0));
    const q = new THREE.Quaternion().setFromRotationMatrix(m);
    startQuat = new CANNON.Quaternion(q.x, q.y, q.z, q.w);
}

// ═══════════════════════════════════════════════════════════════════════
// Start-line visual — checkered pattern at t=0
// ═══════════════════════════════════════════════════════════════════════

function buildStartLine(scene, curve) {
    const center  = curve.getPointAt(0);
    const tangent = curve.getTangentAt(0).normalize();

    // Checkered texture (8x2 grid)
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 32;
    const ctx = canvas.getContext('2d');
    const cols = 8, rows = 2;
    const cw = canvas.width / cols, ch = canvas.height / rows;
    for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
            ctx.fillStyle = (r + c) % 2 === 0 ? '#ffffff' : '#111111';
            ctx.fillRect(c * cw, r * ch, cw, ch);
        }
    }
    const tex = new THREE.CanvasTexture(canvas);

    const geo = new THREE.PlaneGeometry(HALF_WIDTH * 2, 2);
    const mat = new THREE.MeshPhongMaterial({
        map: tex,
        side: THREE.DoubleSide,
        depthWrite: false,
        polygonOffset: true,
        polygonOffsetFactor: -1,
    });
    const mesh = new THREE.Mesh(geo, mat);

    // Position on road surface
    mesh.position.set(center.x, center.y + 0.05, center.z);

    // Orient: lay flat on road, long axis perpendicular to tangent
    mesh.rotation.set(-Math.PI / 2, Math.atan2(tangent.x, tangent.z), 0);

    scene.add(mesh);
}

// ═══════════════════════════════════════════════════════════════════════
// Grid positions — 2-wide starting grid at t=0
// ═══════════════════════════════════════════════════════════════════════

export function getTrackCurve() {
    return trackCurve;
}

export function getGridPositions(count) {
    if (!trackCurve) return [];

    const center  = trackCurve.getPointAt(0);
    const tangent = trackCurve.getTangentAt(0).normalize();
    const up      = new THREE.Vector3(0, 1, 0);
    const right   = new THREE.Vector3().crossVectors(tangent, up).normalize();

    // Orientation quaternion (facing along tangent)
    const m4 = new THREE.Matrix4();
    m4.lookAt(new THREE.Vector3(), tangent, up);
    const q3 = new THREE.Quaternion().setFromRotationMatrix(m4);
    const quat = new CANNON.Quaternion(q3.x, q3.y, q3.z, q3.w);

    const ROW_SPACING = 8;   // metres along -tangent (behind)
    const COL_SPACING = 5;   // metres along right axis

    const positions = [];
    for (let i = 0; i < count; i++) {
        const row = Math.floor(i / 2);
        const col = (i % 2 === 0) ? -1 : 1; // left, right

        const pos = new CANNON.Vec3(
            center.x - tangent.x * row * ROW_SPACING + right.x * col * COL_SPACING * 0.5,
            center.y + 2,
            center.z - tangent.z * row * ROW_SPACING + right.z * col * COL_SPACING * 0.5,
        );

        positions.push({ position: pos, quaternion: quat });
    }

    return positions;
}

// ═══════════════════════════════════════════════════════════════════════
// Terrain query — bilinear height + surface normal from heightfield
// ═══════════════════════════════════════════════════════════════════════

export function getTerrainInfo(wx, wz) {
    if (!hfGrid || !hfMeta) {
        _terrainNormal.set(0, 1, 0);
        return { height: 0, normal: _terrainNormal };
    }

    const { ELEM, HALF_X, HALF_Z, NX, NZ } = hfMeta;

    // World coords → grid coords
    // wx = -HALF_X + i * ELEM  →  i = (wx + HALF_X) / ELEM
    // wz = HALF_Z  - j * ELEM  →  j = (HALF_Z - wz) / ELEM
    const fi = (wx + HALF_X) / ELEM;
    const fj = (HALF_Z - wz) / ELEM;

    // Clamp to grid bounds
    const i0 = Math.max(0, Math.min(NX - 2, Math.floor(fi)));
    const j0 = Math.max(0, Math.min(NZ - 2, Math.floor(fj)));
    const i1 = i0 + 1;
    const j1 = j0 + 1;

    // Fractional position within cell
    const fx = Math.max(0, Math.min(1, fi - i0));
    const fz = Math.max(0, Math.min(1, fj - j0));

    // Four corner heights
    const h00 = hfGrid[i0][j0];
    const h10 = hfGrid[i1][j0];
    const h01 = hfGrid[i0][j1];
    const h11 = hfGrid[i1][j1];

    // Bilinear interpolation
    const height = h00 * (1 - fx) * (1 - fz)
                 + h10 * fx * (1 - fz)
                 + h01 * (1 - fx) * fz
                 + h11 * fx * fz;

    // Surface normal via finite differences
    // dh/dx: height change per metre in +X direction (i axis)
    const dhdx = ((h10 - h00) * (1 - fz) + (h11 - h01) * fz) / ELEM;
    // dh/dz: height change per metre in +Z direction (j axis is -Z, so negate)
    const dhdz = -((h01 - h00) * (1 - fx) + (h11 - h10) * fx) / ELEM;

    // Normal = (-dhdx, 1, -dhdz) normalized
    _terrainNormal.set(-dhdx, 1, -dhdz).normalize();

    return { height, normal: _terrainNormal };
}
