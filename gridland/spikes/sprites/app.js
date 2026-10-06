import { rng, randomGenome, inherit, drawCharacter, JOBS, renderScene, lineup } from "./sprites.js";

const SIZES = [16, 32];
const SHOW = 64; // css px per tile, both sizes
const state = { seed: Number(new URLSearchParams(location.search).get("seed")) || 7, night: false, anim: true, t: 0 };

function buildPeople(seed) {
  const r = rng(seed);
  const elderPartner = randomGenome(r);
  const grand = randomGenome(r);
  const parentA = inherit(grand, elderPartner, r);
  const parentB = randomGenome(r);
  const kids = [0, 1, 2].map(() => inherit(parentA, parentB, r));
  const jobs = JOBS.map(() => randomGenome(r));
  return { grand, parentA, parentB, kids, jobs, jobA: r.pick(JOBS), jobB: r.pick(JOBS), kidJobs: kids.map(() => r.pick(JOBS)) };
}

let people = buildPeople(state.seed);

function canvasEl(scale) {
  const c = document.createElement("canvas");
  c.style.width = "0"; // set after drawing
  c.dataset.scale = scale;
  return c;
}
function fit(c, cssScale) { c.style.width = c.width * cssScale + "px"; c.style.height = c.height * cssScale + "px"; }

function panel(host, size, title) {
  const col = document.createElement("div");
  col.className = "col";
  col.innerHTML = `<h3><b>${size} px</b> per tile${title ? " · " + title : ""}</h3>`;
  host.appendChild(col);
  return col;
}

function row(col, size, sprites, labels) {
  const wrap = document.createElement("div");
  wrap.className = "row";
  const c = canvasEl();
  lineup(c, size, sprites);
  fit(c, SHOW / size);
  wrap.appendChild(c);
  if (labels) {
    const l = document.createElement("div");
    l.className = "labels";
    l.innerHTML = labels.map((t) => `<span>${t}</span>`).join("");
    wrap.appendChild(l);
  }
  col.appendChild(wrap);
}

function render() {
  const frame = state.anim ? Math.floor(state.t * 3) % 2 : 0;
  const P = people;
  document.getElementById("seed").textContent = `seed ${state.seed}`;

  // family
  const fam = document.getElementById("family"); fam.innerHTML = "";
  for (const size of SIZES) {
    const col = panel(fam, size);
    const ch = (g, stage, job, mood = "happy") => drawCharacter(g, { size, stage, job, mood, frame });
    row(col, size, [ch(P.grand, "elder", P.jobA, "neutral"), ch(P.parentA, "adult", P.jobA), ch(P.parentB, "adult", P.jobB), ...P.kids.map((k) => ch(k, "child"))],
      ["grandparent", "parent", "partner", "child 1", "child 2", "child 3"]);
    row(col, size, [ch(P.parentA, "adult", P.jobA, "neutral"), ch(P.parentB, "adult", P.jobB, "neutral"), ...P.kids.map((k, i) => ch(k, "adult", P.kidJobs[i], "neutral"))],
      ["parent", "partner", ...P.kidJobs.map((j, i) => `${i + 1}: ${j}`)]);
  }

  // jobs
  const jobs = document.getElementById("jobs"); jobs.innerHTML = "";
  for (const size of SIZES) {
    const col = panel(jobs, size);
    const sprites = JOBS.map((j, i) => drawCharacter(P.jobs[i], { size, stage: "adult", job: j, mood: "neutral", frame }));
    row(col, size, sprites.slice(0, 6), JOBS.slice(0, 6));
    row(col, size, sprites.slice(6), JOBS.slice(6));
  }

  // moods
  const moods = document.getElementById("moods"); moods.innerHTML = "";
  for (const size of SIZES) {
    const col = panel(moods, size);
    for (const [g, stage, job] of [[P.parentA, "adult", P.jobA], [P.kids[0], "child", null], [P.grand, "elder", P.jobA]]) {
      const s = (mood, f) => drawCharacter(g, { size, stage, job, mood, frame: f });
      row(col, size, [s("happy", 0), s("neutral", 0), s("sad", 0), s("neutral", 0), s("neutral", 1)],
        stage === "adult" ? ["happy", "neutral", "sad", "walk 1", "walk 2"] : null);
    }
  }

  // scene
  const scene = document.getElementById("scene"); scene.innerHTML = "";
  for (const size of SIZES) {
    const col = panel(scene, size, `${size === 16 ? "the 64×64 world would be 1,024 px wide" : "the 64×64 world would be 2,048 px wide"}`);
    const ch = (g, stage, job, mood) => drawCharacter(g, { size, stage, job, mood, frame });
    const cast = [
      { sprite: ch(P.grand, "elder", P.jobA, "neutral"), x: 4, y: 3 },
      { sprite: ch(P.parentA, "adult", P.jobA, "happy"), x: 6, y: 4 },
      { sprite: ch(P.parentB, "adult", P.jobB, "happy"), x: 4, y: 5 },
      { sprite: ch(P.kids[0], "child", null, "happy"), x: 5, y: 5 },
      { sprite: ch(P.kids[1], "child", null, "happy"), x: 6, y: 5 },
      { sprite: ch(P.jobs[11], "adult", "fisherman", "neutral"), x: 1, y: 3 },
      { sprite: ch(P.jobs[5], "adult", "farmer", "neutral"), x: 1, y: 6 },
      { sprite: ch(P.jobs[7], "adult", "toolmaker", "sad"), x: 8, y: 5 },
      { sprite: ch(P.jobs[1], "adult", "builder", "neutral"), x: 9, y: 3 },
    ];
    const big = document.createElement("canvas");
    renderScene(big, size, cast, { t: state.t, night: state.night });
    fit(big, SHOW / size);
    col.appendChild(big);
    const actual = document.createElement("div");
    actual.className = "actual";
    const small = document.createElement("canvas");
    renderScene(small, size, cast, { t: state.t, night: state.night });
    fit(small, 1);
    actual.appendChild(small);
    const cap = document.createElement("div");
    cap.textContent = `1× (${size} css px per tile)`;
    actual.appendChild(cap);
    col.appendChild(actual);
  }
}

document.getElementById("reroll").onclick = () => { state.seed++; people = buildPeople(state.seed); render(); };
document.getElementById("night").onclick = (e) => { state.night = !state.night; e.target.classList.toggle("on", state.night); render(); };
document.getElementById("anim").onclick = (e) => { state.anim = !state.anim; e.target.classList.toggle("on", state.anim); render(); };

render();
let last = performance.now();
setInterval(() => {
  if (!state.anim) return;
  const now = performance.now();
  state.t += (now - last) / 1000; last = now;
  render();
}, 333);
