import * as THREE from 'three';

const MAX_PARTICLES = 200;
const particles = [];
let instancedMesh = null;
let camera = null;

const _dummy = new THREE.Object3D();
const _color = new THREE.Color();

export function createParticleSystem(scene, cam) {
    camera = cam;
    const geo = new THREE.PlaneGeometry(1, 1);
    const mat = new THREE.MeshBasicMaterial({
        color: 0xdddddd,
        transparent: true,
        opacity: 0.2,
        depthWrite: false,
        side: THREE.DoubleSide,
    });
    instancedMesh = new THREE.InstancedMesh(geo, mat, MAX_PARTICLES);
    instancedMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    instancedMesh.frustumCulled = false;

    // Hide all instances initially
    _dummy.scale.set(0, 0, 0);
    _dummy.updateMatrix();
    for (let i = 0; i < MAX_PARTICLES; i++) {
        instancedMesh.setMatrixAt(i, _dummy.matrix);
    }
    instancedMesh.instanceMatrix.needsUpdate = true;

    scene.add(instancedMesh);
}

export function emitSmoke(position, intensity) {
    if (!instancedMesh) return;
    const count = Math.min(1 + Math.floor(intensity * 2), 3);
    for (let i = 0; i < count; i++) {
        if (particles.length >= MAX_PARTICLES) return;
        particles.push({
            x: position.x + (Math.random() - 0.5) * 0.5,
            y: position.y,
            z: position.z + (Math.random() - 0.5) * 0.5,
            vx: (Math.random() - 0.5) * 0.5,
            vy: 1.0 + Math.random() * 0.8,
            vz: (Math.random() - 0.5) * 0.5,
            life: 1.0,
            decay: 0.8 + Math.random() * 0.4,
            size: 0.3 + Math.random() * 0.2,
        });
    }
}

export function updateParticles(dt) {
    if (!instancedMesh) return;

    for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i];
        p.life -= p.decay * dt;
        if (p.life <= 0) {
            particles.splice(i, 1);
            continue;
        }
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.z += p.vz * dt;
        p.vy *= 0.98;
        p.size += dt * 1.2;
    }

    for (let i = 0; i < MAX_PARTICLES; i++) {
        if (i < particles.length) {
            const p = particles[i];
            _dummy.position.set(p.x, p.y, p.z);
            if (camera) {
                _dummy.lookAt(camera.position);
            }
            const s = p.size;
            _dummy.scale.set(s, s, s);
            _dummy.updateMatrix();
            instancedMesh.setMatrixAt(i, _dummy.matrix);

            const gray = 0.7 + p.life * 0.3;
            _color.setRGB(gray, gray, gray);
            instancedMesh.setColorAt(i, _color);
        } else {
            _dummy.scale.set(0, 0, 0);
            _dummy.updateMatrix();
            instancedMesh.setMatrixAt(i, _dummy.matrix);
        }
    }

    instancedMesh.instanceMatrix.needsUpdate = true;
    if (instancedMesh.instanceColor) instancedMesh.instanceColor.needsUpdate = true;
}
