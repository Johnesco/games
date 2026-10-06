// Gridland sprite spike (docs/ART.md): procedural, code-drawn pixel art at
// 16 px and 32 px per tile, to compare before committing to a size or a
// renderer. Throwaway-quality structure, real-quality pixels.
//
// Characters are built from a genome (inherited) + age stage + job + mood.
// Every sprite is drawn at native resolution with integer fillRects, then
// gets an automatic coloured outline ("selout").

// ---------------------------------------------------------------- utilities

export function rng(seed) {
  // mulberry32
  let a = seed >>> 0;
  const f = () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  f.int = (n) => Math.floor(f() * n);
  f.pick = (arr) => arr[Math.floor(f() * arr.length)];
  return f;
}

const hex2rgb = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
const rgb2hex = (c) => "#" + c.map((v) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, "0")).join("");
export const shade = (h, f) => rgb2hex(hex2rgb(h).map((v) => v * f));
export const mix = (a, b, t) => { const A = hex2rgb(a), B = hex2rgb(b); return rgb2hex(A.map((v, i) => v + (B[i] - v) * t)); };

function pen(w, h) {
  const c = document.createElement("canvas");
  c.width = w; c.height = h;
  const ctx = c.getContext("2d");
  const fill = (x, y, ww, hh, col) => { if (ww > 0 && hh > 0) { ctx.fillStyle = col; ctx.fillRect(x, y, ww, hh); } };
  const px = (x, y, col) => fill(x, y, 1, 1, col);
  const disc = (cx, cy, r, col) => {
    for (let y = Math.floor(cy - r); y <= Math.ceil(cy + r); y++)
      for (let x = Math.floor(cx - r); x <= Math.ceil(cx + r); x++)
        if ((x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2 <= r * r) px(x, y, col);
  };
  return { c, ctx, fill, px, disc };
}

// Coloured outline: every transparent pixel touching an opaque one takes a
// dark version of that neighbour's colour.
function outline(c, strength = 0.42) {
  const ctx = c.getContext("2d");
  const { width: w, height: h } = c;
  const img = ctx.getImageData(0, 0, w, h), d = img.data, out = new Uint8ClampedArray(d);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const i = (y * w + x) * 4;
    if (d[i + 3] !== 0) continue;
    for (const [dx, dy] of [[0, 1], [1, 0], [-1, 0], [0, -1]]) {
      const nx = x + dx, ny = y + dy;
      if (nx < 0 || ny < 0 || nx >= w || ny >= h) continue;
      const j = (ny * w + nx) * 4;
      if (d[j + 3] === 0) continue;
      out[i] = d[j] * strength; out[i + 1] = d[j + 1] * strength; out[i + 2] = d[j + 2] * strength * 1.1; out[i + 3] = 255;
      break;
    }
  }
  img.data.set(out);
  ctx.putImageData(img, 0, 0);
  return c;
}

// ---------------------------------------------------------------- genome

export const SKIN = ["#f7d7ba", "#ecbd98", "#d9a06f", "#b97f52", "#8f5c3b", "#64402a"];
export const HAIR = ["#2a1f19", "#583824", "#8b5a2e", "#c2873e", "#e3c375", "#b5452c", "#3c3b4a", "#e6dfd0"];
export const EYES = ["#4a3324", "#3c74a8", "#4b7d3c", "#7652a0", "#2a2a2e"];
export const STYLES = ["short", "long", "bun", "spiky", "curly"];
export const JOBS = ["forager", "builder", "scout", "guardian", "socialite", "farmer",
  "hermit", "toolmaker", "cook", "digger", "healer", "fisherman"];
const CLOTH = {
  forager: "#5f8a3a", builder: "#a3652f", scout: "#3b78a8", guardian: "#913636",
  socialite: "#c4508f", farmer: "#5f7f96", hermit: "#5b5569", toolmaker: "#707079",
  cook: "#ece5d2", digger: "#7d5f3c", healer: "#3f9a82", fisherman: "#2f6a80", child: "#9fb4c8",
};
const TROUSERS = "#4a3b30", BOOTS = "#3a2a20", WOOD = "#8a5a32", METAL = "#b8bec8", STRAW = "#e2c36a";

export function randomGenome(r) {
  return { skin: r.int(SKIN.length), hair: r.int(HAIR.length), eyes: r.int(EYES.length), style: r.pick(STYLES), build: r.int(2) };
}

/// Child genome: each gene from one parent; skin may land between them.
export function inherit(a, b, r) {
  const from = (k) => (r() < 0.5 ? a[k] : b[k]);
  const lo = Math.min(a.skin, b.skin), hi = Math.max(a.skin, b.skin);
  return { skin: lo + r.int(hi - lo + 1), hair: from("hair"), eyes: from("eyes"), style: from("style"), build: from("build") };
}

const greyHair = (h) => mix(h, "#cfcfd4", 0.72);

// ---------------------------------------------------------------- characters

// Geometry per size and age stage. h* = head, b* = body, a = arm columns,
// l = leg columns. Units are native pixels.
const GEOM = {
  16: {
    child: { hx: 5, hy: 5, hw: 6, hh: 5, bx: 6, by: 10, bw: 4, bh: 3, ax: [5, 10], aw: 1, ay: 10, ah: 2, lx: [6, 8], lw: 2, ly: 13, lh: 2 },
    adult: { hx: 5, hy: 2, hw: 6, hh: 5, bx: 5, by: 7, bw: 6, bh: 5, ax: [4, 11], aw: 1, ay: 7, ah: 4, lx: [6, 8], lw: 2, ly: 12, lh: 3 },
    elder: { hx: 5, hy: 3, hw: 6, hh: 5, bx: 5, by: 8, bw: 6, bh: 4, ax: [4, 11], aw: 1, ay: 8, ah: 3, lx: [6, 8], lw: 2, ly: 12, lh: 3 },
  },
  32: {
    child: { hx: 10, hy: 10, hw: 12, hh: 10, bx: 11, by: 20, bw: 10, bh: 6, ax: [9, 21], aw: 2, ay: 21, ah: 5, lx: [12, 16], lw: 4, ly: 26, lh: 4 },
    adult: { hx: 10, hy: 3, hw: 12, hh: 11, bx: 9, by: 15, bw: 14, bh: 10, ax: [7, 23], aw: 2, ay: 16, ah: 9, lx: [11, 17], lw: 4, ly: 25, lh: 5 },
    elder: { hx: 10, hy: 5, hw: 12, hh: 11, bx: 9, by: 16, bw: 14, bh: 9, ax: [7, 23], aw: 2, ay: 17, ah: 8, lx: [11, 17], lw: 4, ly: 25, lh: 5 },
  },
};

/**
 * Draw one character. opts: { size: 16|32, stage, job, mood: happy|neutral|sad, frame: 0|1 }
 * Returns a canvas of size×size, feet on the bottom rows.
 */
export function drawCharacter(g, { size = 16, stage = "adult", job = "forager", mood = "neutral", frame = 0 }) {
  const G = GEOM[size][stage];
  const P = pen(size, size);
  const { fill, px } = P;
  const big = size === 32;
  const skin = SKIN[g.skin], skinS = shade(skin, 0.86);
  let hair = HAIR[g.hair];
  if (stage === "elder") hair = greyHair(hair);
  const cloth = stage === "child" ? mix(CLOTH.child, HAIR[g.hair], 0.18) : CLOTH[job];
  const clothS = shade(cloth, 0.8), clothH = shade(cloth, 1.12);
  const stocky = g.build === 1 && stage !== "child";
  const bx = G.bx - (stocky ? 1 : 0), bw = G.bw + (stocky ? 2 : 0);
  const cx = G.hx + G.hw / 2; // head centre line (x between two middle columns)

  // legs (frame 1 lifts the left leg)
  G.lx.forEach((lx, i) => {
    const lift = frame === 1 && i === 0 ? 1 : 0;
    const bootH = big ? 2 : 1;
    fill(lx, G.ly, G.lw, G.lh - bootH - lift, TROUSERS);
    if (big) fill(lx + G.lw - 1, G.ly, 1, G.lh - bootH - lift, shade(TROUSERS, 0.8));
    fill(lx - (big && i === 0 ? 1 : 0), G.ly + G.lh - bootH - lift, G.lw + (big ? 1 : 0), bootH, BOOTS);
  });

  // body
  fill(bx, G.by, bw, G.bh, cloth);
  fill(bx + bw - (big ? 3 : 1), G.by, big ? 3 : 1, G.bh, clothS);
  if (big) {
    fill(bx, G.by, 1, G.bh, clothH);
    // collar
    fill(cx - 1, G.by, 2, 2, skinS);
    // belt
    if (stage !== "child") {
      fill(bx, G.by + G.bh - 3, bw, 1, "#5a3a22");
      fill(cx - 1, G.by + G.bh - 3, 2, 1, "#d8b84a");
    }
    if (job === "cook" && stage !== "child") fill(bx + 2, G.by + 3, bw - 4, G.bh - 3, "#ffffff");
  } else if (job === "cook" && stage !== "child") {
    fill(bx + 1, G.by + 2, bw - 2, G.bh - 2, "#ffffff");
  }
  // neck (32)
  if (big && stage !== "child") fill(cx - 2, G.hy + G.hh, 4, 1, skinS);

  // arms
  G.ax.forEach((ax, i) => {
    const handH = big ? 2 : 1;
    fill(ax, G.ay, G.aw, G.ah - handH, i === 1 ? clothS : shade(cloth, 0.92));
    fill(ax, G.ay + G.ah - handH, G.aw, handH, skin);
  });

  // head
  fill(G.hx, G.hy, G.hw, G.hh, skin);
  fill(G.hx + G.hw - (big ? 2 : 1), G.hy, big ? 2 : 1, G.hh, skinS);
  if (big) {
    fill(G.hx, G.hy + G.hh - 1, G.hw, 1, shade(skin, 0.92));
    fill(G.hx - 1, G.hy + 5, 1, 2, skinS); // ears
    fill(G.hx + G.hw, G.hy + 5, 1, 2, shade(skin, 0.8));
  }

  // face
  const eye = EYES[g.eyes];
  if (big) {
    const ey = G.hy + 5, exL = G.hx + 2, exR = G.hx + G.hw - 4;
    const squint = stage === "elder";
    for (const ex of [exL, exR]) {
      fill(ex, ey + (squint ? 1 : 0), 2, squint ? 1 : 2, shade(eye, 0.75));
      if (!squint) px(ex, ey, "#ffffff");
    }
    // brows
    const brow = shade(stage === "elder" ? hair : HAIR[g.hair], 0.8);
    if (mood === "sad") {
      px(exL, ey - 2, brow); px(exL + 1, ey - 3, brow);
      px(exR + 1, ey - 2, brow); px(exR, ey - 3, brow);
    } else {
      fill(exL, ey - 2, 2, 1, brow); fill(exR, ey - 2, 2, 1, brow);
    }
    if (stage === "elder") { px(exL - 1, ey + 2, skinS); px(exR + 2, ey + 2, shade(skin, 0.8)); }
    // nose
    fill(cx - 1, G.hy + 7, 2, 1, shade(skin, 0.8));
    // mouth
    const lip = shade(skin, 0.55), my = G.hy + 8;
    if (mood === "happy") {
      px(cx - 2, my, lip); fill(cx - 1, my + 1, 2, 1, lip); px(cx + 1, my, lip);
      px(exL, ey + 2, mix(skin, "#e06070", 0.45)); px(exR + 1, ey + 2, mix(skin, "#e06070", 0.45));
    } else if (mood === "sad") {
      px(cx - 2, my + 1, lip); fill(cx - 1, my, 2, 1, lip); px(cx + 1, my + 1, lip);
    } else {
      fill(cx - 1, my + 1, 2, 1, lip);
    }
  } else {
    const ey = G.hy + 2;
    px(G.hx + 1, ey, shade(eye, 0.55)); px(G.hx + G.hw - 2, ey, shade(eye, 0.55));
    const lip = shade(skin, 0.62);
    if (mood === "happy") fill(cx - 1, ey + 2, 2, 1, lip);
    else if (mood === "sad") { px(cx - 2, ey + 2, lip); px(cx + 1, ey + 2, lip); }
    else px(cx - 1, ey + 2, shade(skin, 0.75));
  }

  // hair (hats replace or cover it)
  const hat = stage === "child" ? null : { farmer: "straw", cook: "toque", guardian: "helmet", scout: "cap", hermit: "hood" }[job];
  drawHair(P, G, g.style, hair, big, stage, hat);
  if (hat) drawHat(P, G, hat, cloth, big);

  // held items / marks
  if (stage === "elder") {
    const x = G.ax[1] + G.aw;
    fill(x, G.ay + (big ? 2 : 1), big ? 2 : 1, size - G.ay - (big ? 3 : 2), WOOD);
    fill(x - 1, G.ay + (big ? 2 : 1), big ? 2 : 1, 1, shade(WOOD, 1.2));
  } else if (stage === "adult") {
    drawTool(P, G, job, big, size, cloth);
  }
  return outline(P.c);
}

function drawHair(P, G, style, hair, big, stage, hat) {
  const { fill, px } = P;
  const hl = shade(hair, 1.25), dk = shade(hair, 0.8);
  const { hx, hy, hw } = G;
  if (hat === "hood" || hat === "helmet") return; // fully covered
  if (stage === "elder") {
    // thinning: sides and a few strands
    const k = big ? 2 : 1;
    fill(hx - (big ? 1 : 0), hy + (big ? 1 : 0), k, big ? 5 : 2, hair);
    fill(hx + hw - k + (big ? 1 : 0), hy + (big ? 1 : 0), k, big ? 5 : 2, dk);
    fill(hx + (big ? 2 : 1), hy - (big ? 1 : 0), hw - (big ? 4 : 2), 1, hair);
    return;
  }
  if (!big) {
    switch (style) {
      case "long": fill(hx - 1, hy - 1, hw + 2, 2, hair); fill(hx - 1, hy + 1, 1, 4, hair); fill(hx + hw, hy + 1, 1, 4, dk); break;
      case "bun": fill(hx, hy - 1, hw, 2, hair); px(hx, hy + 1, hair); px(hx + hw - 1, hy + 1, dk); fill(hx + 2, hy - 2, 2, 1, hair); break;
      case "spiky": fill(hx, hy, hw, 1, hair); px(hx, hy - 1, hair); px(hx + 2, hy - 1, hair); px(hx + 4, hy - 1, hair); px(hx + hw - 1, hy - 1, dk); break;
      case "curly": fill(hx - 1, hy - 1, hw + 2, 2, hair); px(hx - 1, hy + 1, hair); px(hx + hw, hy + 1, dk); px(hx - 1, hy + 2, hair); px(hx + hw, hy + 2, dk); px(hx + 1, hy - 2, hair); px(hx + 3, hy - 2, hair); break;
      default: fill(hx, hy - 1, hw, 2, hair); px(hx, hy + 1, hair); px(hx + hw - 1, hy + 1, dk);
    }
    px(hx + 1, hy - 1, hl);
    return;
  }
  const base = () => { fill(hx - 1, hy - 1, hw + 2, 4, hair); fill(hx - 1, hy + 3, 2, 3, hair); fill(hx + hw - 1, hy + 3, 2, 3, dk); };
  switch (style) {
    case "long": base(); fill(hx - 2, hy + 1, 2, 11, hair); fill(hx + hw, hy + 1, 2, 11, dk); break;
    case "bun": base(); fill(hx + 4, hy - 4, 4, 3, hair); px(hx + 5, hy - 4, hl); break;
    case "spiky":
      fill(hx - 1, hy, hw + 2, 3, hair); fill(hx - 1, hy + 3, 2, 2, hair); fill(hx + hw - 1, hy + 3, 2, 2, dk);
      // uneven tufts so it reads as hair, not a crown
      [[0, 2], [3, 3], [6, 1], [8, 3], [11, 2]].forEach(([dx, h], i) => fill(hx - 1 + dx, hy - h, 2, h, i === 4 ? dk : hair));
      break;
    case "curly":
      fill(hx - 2, hy - 2, hw + 4, 5, hair);
      for (let i = 0; i < 4; i++) fill(hx - 1 + i * 4, hy - 3, 2, 1, hair);
      fill(hx - 2, hy + 3, 2, 5, hair); fill(hx + hw, hy + 3, 2, 5, dk);
      px(hx - 3, hy + 4, hair); px(hx + hw + 2, hy + 5, dk);
      break;
    default: base();
  }
  fill(hx + 1, hy - 1, 4, 1, hl);
}

function drawHat(P, G, hat, cloth, big) {
  const { fill, px } = P;
  const { hx, hy, hw, hh } = G;
  const k = big ? 2 : 1;
  switch (hat) {
    case "straw":
      fill(hx - 2 * k, hy - (big ? 0 : 0), hw + 4 * k, k, STRAW);
      fill(hx, hy - 2 * k - (big ? 1 : 0), hw, 2 * k + (big ? 1 : 0), STRAW);
      fill(hx, hy - k, hw, big ? 1 : 0, "#b5452c"); // band
      fill(hx + hw - k, hy - 2 * k, k, 2 * k, shade(STRAW, 0.82));
      break;
    case "toque":
      fill(hx, hy - (big ? 6 : 3), hw, big ? 6 : 3, "#ffffff");
      fill(hx, hy - (big ? 1 : 1), hw, 1, "#dcdcd8");
      if (big) fill(hx + hw - 2, hy - 6, 2, 5, "#e8e8e4");
      break;
    case "helmet":
      fill(hx - (big ? 1 : 0), hy - (big ? 2 : 1), hw + (big ? 2 : 0), big ? 6 : 3, METAL);
      fill(hx + hw - k, hy - k, k + (big ? 1 : 0), big ? 6 : 3, shade(METAL, 0.8));
      fill(hx + 1, hy - (big ? 2 : 1), big ? 4 : 2, 1, shade(METAL, 1.15));
      if (big) fill(hx + hw / 2 - 1, hy + 4, 2, 4, shade(METAL, 0.9)); // nose guard
      break;
    case "cap":
      fill(hx, hy - k, hw, 2 * k, cloth);
      fill(hx + hw - k, hy + (big ? 1 : 0), 2 * k, k, shade(cloth, 0.75)); // brim
      px(hx + 1, hy - k - 1, "#e8e2c0"); if (big) fill(hx + 1, hy - 5, 1, 3, "#e8e2c0"); // feather
      break;
    case "hood":
      fill(hx - k, hy - k, hw + 2 * k, k + (big ? 1 : 0), cloth);
      fill(hx - k, hy, k, hh - (big ? 2 : 1), cloth);
      fill(hx + hw, hy, k, hh - (big ? 2 : 1), shade(cloth, 0.8));
      break;
  }
}

function drawTool(P, G, job, big, size, cloth) {
  const { fill, px } = P;
  const k = big ? 2 : 1;
  const hx = G.ax[1] + G.aw;        // just right of the right hand
  const hy = G.ay + G.ah - k;       // hand row
  switch (job) {
    case "toolmaker": fill(hx, hy - 5 * k, k, 6 * k, WOOD); fill(hx - k, hy - 6 * k, 3 * k, 2 * k, METAL); fill(hx + k, hy - 6 * k, k, 2 * k, shade(METAL, 0.8)); break;
    case "builder": fill(hx, hy - 3 * k, k, 4 * k, WOOD); fill(hx - k, hy - 4 * k, 3 * k, k + (big ? 1 : 0), shade(METAL, 0.85)); break;
    case "digger": fill(hx, hy - 6 * k, k, 7 * k, WOOD); fill(hx - (big ? 1 : 0), hy + k, k + (big ? 2 : 1), 2 * k, METAL); break;
    case "fisherman":
      for (let i = 0; i < (big ? 12 : 6); i++) px(hx + Math.floor(i / 3), hy - i, WOOD);
      px(hx + (big ? 4 : 2), hy - (big ? 11 : 5), "#e0e0e0");
      for (let i = 0; i < (big ? 8 : 4); i++) px(hx + (big ? 4 : 2), hy - (big ? 10 : 4) + i, "#cfd8dc");
      break;
    case "forager": fill(G.ax[0] - k, hy - k, 2 * k + (big ? 1 : 0), 2 * k, "#b08a4a"); fill(G.ax[0] - k, hy - k, 2 * k + (big ? 1 : 0), 1, "#d6b06a"); if (big) px(G.ax[0] - 1, hy, "#d6343f"); break;
    case "healer": {
      const cx = G.bx + Math.floor(G.bw / 2) - (big ? 1 : 0), cy = G.by + (big ? 3 : 1);
      if (big) { fill(cx, cy, 2, 6, "#ffffff"); fill(cx - 2, cy + 2, 6, 2, "#ffffff"); }
      else { px(cx, cy, "#ffffff"); px(cx - 1, cy + 1, "#ffffff"); px(cx, cy + 1, "#ffffff"); px(cx + 1, cy + 1, "#ffffff"); px(cx, cy + 2, "#ffffff"); }
      break;
    }
    case "socialite": {
      const fx = G.hx + G.hw - k, fy = G.hy - k;
      if (big) { fill(fx, fy, 3, 3, "#f06aa8"); px(fx + 1, fy + 1, "#ffe066"); fill(G.bx + 4, G.by + 1, G.bw - 8, 1, "#f4d06a"); }
      else { px(fx, fy, "#f06aa8"); px(G.bx + 2, G.by, "#f4d06a"); px(G.bx + 3, G.by, "#f4d06a"); }
      break;
    }
  }
}

// ---------------------------------------------------------------- terrain

const T = {
  grass: "#5b9b4a", grassD: "#4c8a3f", grassL: "#6fae58",
  water: "#3e72b6", waterD: "#335f9c", waterL: "#6ea3dc", foam: "#c3e2f4",
  sand: "#dfc489", sandD: "#c8aa6b",
  dirt: "#a7865a", dirtD: "#8b6c47", dirtL: "#c2a476",
  soil: "#7a5a37", soilD: "#5f4529", sprout: "#7cc35a",
  rock: "#8c909a", rockD: "#666a74", rockL: "#b5b9c2",
};

const tileHash = (x, y) => { let h = (x * 374761393 + y * 668265263) >>> 0; h = Math.imul(h ^ (h >>> 13), 1274126177); return (h ^ (h >>> 16)) >>> 0; };

/** Draw the ground layer of one tile at (ox, oy). nb(dx,dy) → neighbour type. */
function drawGround(P, s, type, ox, oy, nb, t) {
  const { fill, px } = P;
  const u = s / 16;
  const r = rng(tileHash(ox, oy));
  const ground = { water: "water", sand: "sand", path: "path", field: "field" }[type] || "grass";
  if (ground === "grass") {
    fill(ox, oy, s, s, T.grass);
    for (let i = 0; i < s * s / 22; i++) px(ox + r.int(s), oy + r.int(s), r() < 0.55 ? T.grassD : T.grassL);
    for (let i = 0; i < 2; i++) { // tufts
      const x = ox + 1 + r.int(s - 3 * u - 1), y = oy + 2 * u + r.int(s - 3 * u);
      px(x, y, T.grassD); px(x + u, y - u, T.grassD); px(x + 2 * u, y, T.grassD);
      if (u > 1) { px(x + 1, y - 1, T.grassL); px(x + 3, y - 1, T.grassL); }
    }
    if (type === "flowers") for (let i = 0; i < 3 + u; i++) {
      const x = ox + 1 + r.int(s - 2 * u - 1), y = oy + 1 + r.int(s - 2 * u - 1);
      const col = r.pick(["#f08cbe", "#f4d454", "#8fb2ee", "#ffffff"]);
      fill(x, y, u, u, col);
      if (u > 1) { px(x - 1, y, col); px(x + 2, y, col); px(x, y - 1, col); px(x, y + 2, col); fill(x, y, 2, 2, "#ffe58a"); }
    }
  } else if (ground === "water") {
    fill(ox, oy, s, s, T.water);
    for (let i = 0; i < s / 4; i++) px(ox + r.int(s), oy + r.int(s), T.waterD);
    // drifting highlights
    for (let i = 0; i < 2; i++) {
      const y = oy + ((r.int(s) + Math.floor(t * 2) * u) % s), x = ox + r.int(s - 3 * u);
      fill(x, y, 3 * u, u > 1 ? 1 : 1, T.waterL);
    }
    // shoreline foam toward any land neighbour
    const land = (dx, dy) => { const n = nb(dx, dy); return n && n !== "water"; };
    const fw = u * 2;
    if (land(0, -1)) { fill(ox, oy, s, fw, T.waterL); fill(ox, oy, s, u, T.foam); }
    if (land(0, 1)) { fill(ox, oy + s - fw, s, fw, T.waterL); fill(ox, oy + s - u, s, u, T.foam); }
    if (land(-1, 0)) { fill(ox, oy, fw, s, T.waterL); fill(ox, oy, u, s, T.foam); }
    if (land(1, 0)) { fill(ox + s - fw, oy, fw, s, T.waterL); fill(ox + s - u, oy, u, s, T.foam); }
  } else if (ground === "sand") {
    fill(ox, oy, s, s, T.sand);
    for (let i = 0; i < s * s / 18; i++) px(ox + r.int(s), oy + r.int(s), r() < 0.7 ? T.sandD : shade(T.sand, 1.08));
  } else if (ground === "path") {
    fill(ox, oy, s, s, T.dirt);
    for (let i = 0; i < s * s / 20; i++) px(ox + r.int(s), oy + r.int(s), r() < 0.6 ? T.dirtD : T.dirtL);
    // grassy edges toward non-path neighbours, jagged
    const edge = (horiz, x0, y0, inward) => {
      for (let i = 0; i < s; i++) {
        const d = (tileHash(ox + i, oy + (horiz ? y0 : x0)) % 3 === 0 ? 2 : 1) * u;
        if (horiz) fill(ox + i, inward > 0 ? oy : oy + s - d, 1, d, T.grass);
        else fill(inward > 0 ? ox : ox + s - d, oy + i, d, 1, T.grass);
      }
    };
    const isPath = (dx, dy) => ["path", "home"].includes(nb(dx, dy));
    if (!isPath(0, -1)) edge(true, 0, 0, 1);
    if (!isPath(0, 1)) edge(true, 0, s, -1);
    if (!isPath(-1, 0)) edge(false, 0, 0, 1);
    if (!isPath(1, 0)) edge(false, s, 0, -1);
  } else if (ground === "field") {
    fill(ox, oy, s, s, T.soil);
    for (let y = 0; y < s; y += 4 * u) {
      fill(ox, oy + y + 3 * u, s, u, T.soilD);
      for (let x = u; x < s; x += 3 * u) { px(ox + x, oy + y + u, T.sprout); if (u > 1) { px(ox + x + 1, oy + y + 1, T.sprout); px(ox + x, oy + y + 3, shade(T.sprout, 0.75)); } }
    }
  }
}

/** Objects that stand on a tile and may overlap the tile above (drawn y-sorted). */
function drawObject(P, s, type, ox, oy, t, night, seed) {
  const { fill, px, disc } = P;
  const u = s / 16, cx = ox + s / 2, base = oy + s;
  const r = rng(seed ?? tileHash(ox + 7, oy + 3));
  if (type === "tree") {
    disc(cx, base - 2 * u, 5 * u, shade(T.grass, 0.78)); // shadow
    fill(cx - u, base - 6 * u, 2 * u, 5 * u, "#6b4a2a");
    fill(cx, base - 6 * u, u, 5 * u, "#553a20");
    const cy = base - 11 * u;
    disc(cx, cy, 7 * u, "#2f6b33");
    disc(cx - u, cy - u, 6 * u, "#3f7f3a");
    disc(cx - 2.5 * u, cy - 2.5 * u, 3 * u, "#5c9a4a");
    for (let i = 0; i < 4 * u; i++) px(cx - 5 * u + r.int(10 * u), cy - 4 * u + r.int(8 * u), "#6fae58");
  } else if (type === "bush") {
    disc(cx, base - 4 * u, 5.5 * u, "#3c7a3a");
    disc(cx - u, base - 5 * u, 4 * u, "#4f8f45");
    for (let i = 0; i < 5; i++) { const x = cx - 4 * u + r.int(8 * u), y = base - 8 * u + r.int(6 * u); fill(x, y, u, u, "#d6343f"); if (u > 1) px(x, y, "#ff8a8a"); }
  } else if (type === "rock") {
    disc(cx, base - 4 * u, 6 * u, T.rock);
    disc(cx + 1.5 * u, base - 3 * u, 4.5 * u, T.rockD);
    disc(cx - 1.5 * u, base - 5.5 * u, 3 * u, T.rockL);
    fill(ox + 3 * u, base - u, 10 * u, u, shade(T.grass, 0.75));
  } else if (type === "home") {
    const wall = "#cfa676", wallS = "#b18a5c", roof = "#9a4636", roofS = "#7a3428";
    fill(ox + 2 * u, oy + 7 * u, 12 * u, 9 * u, wall);
    fill(ox + 11 * u, oy + 7 * u, 3 * u, 9 * u, wallS);
    for (let row = 0; row < 9 * u; row++) {
      const half = Math.floor(row / 1.15) + u;
      fill(cx - half, oy - u + row, half * 2, 1, row % (3 * u) === 0 ? roofS : roof);
    }
    fill(cx - u, base - 6 * u, 3 * u, 6 * u, "#5a3a22"); // door
    px(cx + u, base - 3 * u, "#d8b84a");
    const win = night ? "#ffd27a" : "#43597a";
    fill(ox + 3 * u, oy + 9 * u, 3 * u, 3 * u, win);
    if (!night) px(ox + 3 * u, oy + 9 * u, "#8fb2d8");
    fill(ox + 3 * u, oy + 10 * u + (u > 1 ? 1 : 0), 3 * u, u > 1 ? 1 : 0, "#5a3a22");
  } else if (type === "fire") {
    fill(cx - 5 * u, base - 3 * u, 10 * u, 2 * u, "#5a3a22");
    fill(cx - 4 * u, base - 4 * u, 8 * u, 2 * u, "#7a5232");
    const f = Math.floor(t * 8) % 3;
    const h = [8, 9, 7][f] * u, w = [3, 2.5, 3.5][f] * u;
    disc(cx, base - 4 * u - h * 0.35, w + u, "#d8482f");
    disc(cx, base - 4 * u - h * 0.4, w, "#f49a3c");
    disc(cx, base - 4 * u - h * 0.3, w * 0.55, "#ffe27a");
    fill(cx - u / 2, base - 4 * u - h, u, u, "#f49a3c");
    if (f === 1) px(cx + 2 * u, base - 4 * u - h - 2 * u, "#ffe27a");
  }
}

// ---------------------------------------------------------------- scene

export const SCENE = [
  "wwwwsggTTggT",
  "wwwsgg*gTTgg",
  "wwsggHpppgTg",
  "wsgggggbpggg",
  "sggggFggpHg*",
  "gggggggrpppg",
  "fff*ggggggpg",
  "fffgggggTgpg",
];
const LEGEND = { w: "water", s: "sand", g: "grass", p: "path", T: "tree", H: "home", F: "fire", f: "field", b: "bush", r: "rock", "*": "flowers" };
const OBJECTS = new Set(["tree", "home", "fire", "bush", "rock"]);

/**
 * Render the scene. cast: [{ sprite: canvas(size×size), x, y }] in tile coords.
 */
export function renderScene(canvas, s, cast, { t = 0, night = false } = {}) {
  const H = SCENE.length, W = SCENE[0].length;
  canvas.width = W * s; canvas.height = H * s;
  const ctx = canvas.getContext("2d");
  const P = { ctx, ...penOn(ctx) };
  const type = (x, y) => (y < 0 || y >= H || x < 0 || x >= W ? null : LEGEND[SCENE[y][x]]);
  for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
    drawGround(P, s, type(x, y), x * s, y * s, (dx, dy) => type(x + dx, y + dy), t);
  }
  // y-sorted object + character pass
  const items = [];
  for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) if (OBJECTS.has(type(x, y))) items.push({ y, x, obj: type(x, y) });
  for (const c of cast) items.push({ y: c.y + 0.5, x: c.x, sprite: c.sprite });
  items.sort((a, b) => a.y - b.y);
  for (const it of items) {
    if (it.obj) {
      // object sprites go through a scratch pen so they get outlined too
      const pad = s; // room above the tile for canopies and roofs
      const S = pen(s, s + pad);
      drawObject({ ...S, fill: (x, y, w, h, c) => S.fill(x, y + pad, w, h, c), px: (x, y, c) => S.px(x, y + pad, c), disc: (x, y, r, c) => S.disc(x, y + pad, r, c) },
        s, it.obj, 0, 0, t, night, tileHash(it.x, it.y));
      outline(S.c, 0.5);
      ctx.drawImage(S.c, it.x * s, it.y * s - pad);
    } else {
      ctx.drawImage(it.sprite, it.x * s, Math.floor(it.y - 0.5) * s);
    }
  }
  if (night) applyNight(ctx, s, W, H, type);
}

function penOn(ctx) {
  const fill = (x, y, w, h, col) => { if (w > 0 && h > 0) { ctx.fillStyle = col; ctx.fillRect(x, y, w, h); } };
  const px = (x, y, col) => fill(x, y, 1, 1, col);
  return { fill, px };
}

// Night as the new renderer would do it: a light map (ambient + warm radial
// lights), stepped to 2-pixel blocks for a pixel-art feel, multiplied over.
function applyNight(ctx, s, W, H, type) {
  const step = s / 8; // light-map cell size in art pixels
  const lw = Math.ceil((W * s) / step), lh = Math.ceil((H * s) / step);
  const L = document.createElement("canvas"); L.width = lw; L.height = lh;
  const l = L.getContext("2d");
  l.fillStyle = "#3a4670"; l.fillRect(0, 0, lw, lh);
  l.globalCompositeOperation = "lighter";
  const glow = (tx, ty, radiusTiles, col, alpha) => {
    const cx = ((tx + 0.5) * s) / step, cy = ((ty + 0.5) * s) / step, r = (radiusTiles * s) / step;
    const g = l.createRadialGradient(cx, cy, 0, cx, cy, r);
    g.addColorStop(0, col.replace("A", alpha)); g.addColorStop(1, col.replace("A", 0));
    l.fillStyle = g; l.fillRect(cx - r, cy - r, 2 * r, 2 * r);
  };
  for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
    if (type(x, y) === "fire") glow(x, y - 0.2, 3.6, "rgba(255,170,90,A)", 0.95);
    if (type(x, y) === "home") glow(x - 0.15, y + 0.1, 1.1, "rgba(255,200,120,A)", 0.55);
  }
  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.globalCompositeOperation = "multiply";
  ctx.drawImage(L, 0, 0, W * s, H * s);
  ctx.restore();
}

// A strip of grass tiles with characters standing on it, for the lineups.
export function lineup(canvas, s, sprites) {
  canvas.width = sprites.length * s; canvas.height = s;
  const ctx = canvas.getContext("2d");
  const P = { ctx, ...penOn(ctx) };
  sprites.forEach((sp, i) => {
    drawGround(P, s, "grass", i * s, 0, () => "grass", 0);
    ctx.fillStyle = "rgba(0,0,0,0.18)";
    ctx.fillRect(i * s + s * 0.28, s - s / 8, s * 0.44, s / 16);
    ctx.drawImage(sp, i * s, 0);
  });
}
