import * as THREE from 'three';

let ctx = null;
let engineOsc1, engineOsc2, engineGain, engineFilter;
let screechNode, screechGain, screechFilter;
let windNode, windGain, windFilter;
let noiseBuf = null;

export function initAudio() {
    if (ctx) return;
    ctx = new (window.AudioContext || window.webkitAudioContext)();
    if (ctx.state === 'suspended') ctx.resume();

    const bufSize = ctx.sampleRate * 2;
    noiseBuf = ctx.createBuffer(1, bufSize, ctx.sampleRate);
    const data = noiseBuf.getChannelData(0);
    for (let i = 0; i < bufSize; i++) data[i] = Math.random() * 2 - 1;

    // Engine: sawtooth + triangle → lowpass → gain
    engineOsc1 = ctx.createOscillator();
    engineOsc1.type = 'sawtooth';
    engineOsc1.frequency.value = 40;

    engineOsc2 = ctx.createOscillator();
    engineOsc2.type = 'triangle';
    engineOsc2.frequency.value = 80;

    engineFilter = ctx.createBiquadFilter();
    engineFilter.type = 'lowpass';
    engineFilter.frequency.value = 150;
    engineFilter.Q.value = 2;

    engineGain = ctx.createGain();
    engineGain.gain.value = 0.05;

    engineOsc1.connect(engineFilter);
    engineOsc2.connect(engineFilter);
    engineFilter.connect(engineGain);
    engineGain.connect(ctx.destination);
    engineOsc1.start();
    engineOsc2.start();

    // Tire screech: noise → bandpass → gain
    screechNode = ctx.createBufferSource();
    screechNode.buffer = noiseBuf;
    screechNode.loop = true;

    screechFilter = ctx.createBiquadFilter();
    screechFilter.type = 'bandpass';
    screechFilter.frequency.value = 2000;
    screechFilter.Q.value = 4;

    screechGain = ctx.createGain();
    screechGain.gain.value = 0;

    screechNode.connect(screechFilter);
    screechFilter.connect(screechGain);
    screechGain.connect(ctx.destination);
    screechNode.start();

    // Wind: noise → lowpass → gain
    windNode = ctx.createBufferSource();
    windNode.buffer = noiseBuf;
    windNode.loop = true;

    windFilter = ctx.createBiquadFilter();
    windFilter.type = 'lowpass';
    windFilter.frequency.value = 400;
    windFilter.Q.value = 1;

    windGain = ctx.createGain();
    windGain.gain.value = 0;

    windNode.connect(windFilter);
    windFilter.connect(windGain);
    windGain.connect(ctx.destination);
    windNode.start();
}

export function updateAudio(speed, maxSpeed, isDrifting) {
    if (!ctx) return;

    const t = Math.min(speed / maxSpeed, 1);

    // Engine pitch and volume scale with speed
    engineOsc1.frequency.value = 40 + t * 120;
    engineOsc2.frequency.value = 80 + t * 240;
    engineFilter.frequency.value = 150 + t * 650;
    engineGain.gain.value = 0.05 + t * 0.20;

    // Tire screech when drifting
    const screechTarget = isDrifting ? 0.15 : 0;
    screechGain.gain.value += (screechTarget - screechGain.gain.value) * 0.3;
    screechFilter.frequency.value = 2000 + (isDrifting ? 2500 : 0);

    // Wind noise at high speed
    const windTarget = t > 0.4 ? (t - 0.4) * 0.12 : 0;
    windGain.gain.value += (windTarget - windGain.gain.value) * 0.2;
    windFilter.frequency.value = 300 + t * 500;
}

export function playImpact(intensity) {
    if (!ctx) return;
    const scaled = Math.min(intensity, 1);

    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(200 * scaled + 60, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(40, ctx.currentTime + 0.15);

    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0.2 * scaled, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.2);

    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + 0.25);

    // Noise burst
    const noise = ctx.createBufferSource();
    noise.buffer = noiseBuf;
    const nGain = ctx.createGain();
    nGain.gain.setValueAtTime(0.1 * scaled, ctx.currentTime);
    nGain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.12);
    noise.connect(nGain);
    nGain.connect(ctx.destination);
    noise.start();
    noise.stop(ctx.currentTime + 0.15);
}

export function suspendAudio() {
    if (ctx && ctx.state === 'running') ctx.suspend();
}

export function resumeAudio() {
    if (ctx && ctx.state === 'suspended') ctx.resume();
}

// Drift detection helper
const _fwd = new THREE.Vector3();
const _vel = new THREE.Vector3();

export function isDriftingCheck(chassisBody) {
    if (!chassisBody) return false;
    const speed = chassisBody.velocity.length();
    if (speed < 5) return false;

    _fwd.set(0, 0, -1);
    const q = chassisBody.quaternion;
    _fwd.applyQuaternion(new THREE.Quaternion(q.x, q.y, q.z, q.w));
    _fwd.y = 0;
    _fwd.normalize();

    _vel.set(chassisBody.velocity.x, 0, chassisBody.velocity.z).normalize();

    const dot = _fwd.dot(_vel);
    return dot < 0.94; // ~20 degree angle
}
