import init, { Gridland } from "./pkg/gridland.js";

const canvas = document.getElementById("world");
const ctx = canvas.getContext("2d");
const statsEl = document.getElementById("stats");
const botsEl = document.getElementById("bots");
const logEl = document.getElementById("log");
const bubblesEl = document.getElementById("bubbles");
const worldView = document.getElementById("world-view");

let world = null;
let memory = null;
let speed = 2;
let tool = "select";
let selectedBotId = null;
let zoom = 1;
let baseW = 512;
let baseH = 512;
let tileSizeCss = 8; // css pixels per tile at current zoom

async function boot() {
  const wasm = await init();
  memory = wasm.memory;
  const seed = (Math.random() * 0xffffffff) >>> 0;
  world = new Gridland(seed);
  window.__gl = world;
  baseW = world.canvas_w();
  baseH = world.canvas_h();
  canvas.width = baseW;
  canvas.height = baseH;
  applyZoom();
  setupUI();
  requestAnimationFrame(loop);
  setInterval(refreshPanels, 200);
  setInterval(refreshBubbles, 160);
}

function applyZoom() {
  const w = baseW * zoom;
  const h = baseH * zoom;
  worldView.style.width = w + "px";
  worldView.style.height = h + "px";
  tileSizeCss = world.tile_size() * zoom;
}

function setupUI() {
  document.querySelectorAll(".toolbar button[data-speed]").forEach((btn) => {
    btn.addEventListener("click", () => {
      document
        .querySelectorAll(".toolbar button[data-speed]")
        .forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      speed = parseInt(btn.dataset.speed, 10);
    });
  });

  document.querySelectorAll(".toolbar button[data-zoom]").forEach((btn) => {
    btn.addEventListener("click", () => {
      document
        .querySelectorAll(".toolbar button[data-zoom]")
        .forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      zoom = parseInt(btn.dataset.zoom, 10);
      applyZoom();
      // If selected bot, scroll it into view
      if (selectedBotId !== null) {
        scrollSelectedIntoView();
      }
      refreshBubbles();
    });
  });

  document.querySelectorAll(".toolbar button[data-tool]").forEach((btn) => {
    btn.addEventListener("click", () => {
      document
        .querySelectorAll(".toolbar button[data-tool]")
        .forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      tool = btn.dataset.tool;
      canvas.style.cursor =
        tool === "select" ? "pointer" : "crosshair";
    });
  });

  canvas.addEventListener("click", (e) => {
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const px = Math.floor((e.clientX - rect.left) * scaleX);
    const py = Math.floor((e.clientY - rect.top) * scaleY);
    const tilePx = world.tile_size();
    const tx = Math.floor(px / tilePx);
    const ty = Math.floor(py / tilePx);

    if (tool === "select") {
      const id = world.click_select(px, py);
      selectedBotId = id >= 0 ? id : null;
      refreshPanels();
      refreshBubbles();
    } else if (tool === "berry") {
      world.drop_food(tx, ty);
    } else if (tool === "rock") {
      world.drop_rock(tx, ty);
    } else if (tool === "fire") {
      world.drop_fire(tx, ty);
    } else if (tool === "clear") {
      world.clear_tile(tx, ty);
    }
  });

  document.getElementById("btn-reseed").addEventListener("click", () => {
    const seed = (Math.random() * 0xffffffff) >>> 0;
    world = new Gridland(seed);
    window.__gl = world;
    selectedBotId = null;
    bubblesEl.innerHTML = "";
    botsEl.innerHTML = "";
    botRowEls.clear();
    refreshPanels();
  });

  // Delegated click on the residents list. Tapping the row header selects +
  // expands. Tapping the already-expanded bot's header collapses (deselect).
  // Clicks inside the detail body are ignored so e.g. scrolling the memory
  // list doesn't collapse the row.
  botsEl.addEventListener("click", (e) => {
    if (e.target.closest(".bot-detail")) return;
    const li = e.target.closest("li[data-id]");
    if (!li) return;
    const id = parseInt(li.dataset.id, 10);
    if (Number.isNaN(id)) return;
    if (selectedBotId === id) {
      world.clear_selection();
      selectedBotId = null;
    } else if (world.select_by_id(id)) {
      selectedBotId = id;
      scrollSelectedIntoView();
    }
    refreshPanels();
  });
}

function scrollSelectedIntoView() {
  const info = world.selected_info();
  if (info === "null") return;
  const b = JSON.parse(info);
  const scroller = document.getElementById("world-scroll");
  const targetX = b.x * tileSizeCss + tileSizeCss / 2 - scroller.clientWidth / 2;
  const targetY = b.y * tileSizeCss + tileSizeCss / 2 - scroller.clientHeight / 2;
  scroller.scrollTo({ left: targetX, top: targetY, behavior: "smooth" });
}

function loop() {
  if (world) {
    for (let i = 0; i < speed; i++) world.tick();
    world.render();
    const ptr = world.buffer_ptr();
    const len = world.buffer_len();
    const bytes = new Uint8ClampedArray(memory.buffer, ptr, len);
    const imgData = new ImageData(bytes, world.canvas_w(), world.canvas_h());
    ctx.putImageData(imgData, 0, 0);
  }
  requestAnimationFrame(loop);
}

// Track rendered bubbles by bot id so we can update positions without flicker.
const bubbleEls = new Map(); // id -> { el, lastThought, ttl }

function refreshBubbles() {
  if (!world) return;
  const bubbles = JSON.parse(world.bubbles());
  const seen = new Set();
  for (const b of bubbles) {
    seen.add(b.id);
    let rec = bubbleEls.get(b.id);
    if (!rec) {
      const el = document.createElement("div");
      el.className = "bubble";
      bubblesEl.appendChild(el);
      rec = { el, lastThought: "" };
      bubbleEls.set(b.id, rec);
    }
    if (b.thought !== rec.lastThought) {
      rec.el.innerHTML = `<span class="who">${escapeHtml(b.name)}:</span>${escapeHtml(b.thought)}`;
      rec.lastThought = b.thought;
      // re-trigger pop animation
      rec.el.style.animation = "none";
      // eslint-disable-next-line no-unused-expressions
      rec.el.offsetWidth;
      rec.el.style.animation = "";
    }
    rec.el.classList.toggle("selected", !!b.selected);
    rec.el.classList.toggle("fading", b.ttl < 25 && !b.selected);

    // Position above the bot's head
    const cx = b.x * tileSizeCss + tileSizeCss / 2;
    const cy = b.y * tileSizeCss - 4;
    rec.el.style.left = cx + "px";
    rec.el.style.top = cy + "px";
  }
  // Remove stale bubble elements
  for (const [id, rec] of bubbleEls.entries()) {
    if (!seen.has(id)) {
      rec.el.remove();
      bubbleEls.delete(id);
    }
  }
}

let prevSelectedForScroll = null;

// Keep stable DOM nodes across refreshes. Rebuilding innerHTML every 200ms
// would otherwise blow away an in-flight click on a row.
const botRowEls = new Map(); // id -> { li, row, swatch, name, idLine, status, thought, detail }

function setText(el, t) {
  if (el.textContent !== t) el.textContent = t;
}

function reconcileBots(bots, info) {
  const seen = new Set();
  for (const b of bots) {
    seen.add(b.id);
    let rec = botRowEls.get(b.id);
    const isSelected = info && info.id === b.id;

    if (!rec) {
      const li = document.createElement("li");
      li.dataset.id = b.id;

      const row = document.createElement("div");
      row.className = "bot-row";

      const swatch = document.createElement("span");
      swatch.className = "swatch";
      swatch.style.background = `hsl(${(b.id * 47) % 360} 70% 60%)`;

      const identity = document.createElement("div");
      identity.className = "identity";

      const primary = document.createElement("div");
      primary.className = "primary";
      const name = document.createElement("span");
      name.className = "name";
      const idLine = document.createElement("span");
      idLine.className = "id-line";
      primary.append(name, idLine);

      const secondary = document.createElement("div");
      secondary.className = "secondary";
      const status = document.createElement("span");
      status.className = "status";
      const thought = document.createElement("span");
      thought.className = "thought-snip";
      secondary.append(status, thought);

      identity.append(primary, secondary);
      row.append(swatch, identity);
      li.append(row);

      rec = { li, row, name, idLine, status, thought, detail: null };
      botRowEls.set(b.id, rec);
      botsEl.append(li);
    }

    setText(rec.name, b.name);
    setText(rec.idLine, `${b.job} #${b.id}`);
    setText(rec.status, b.goal);
    if (b.thought) {
      setText(rec.thought, `\u201C${b.thought}\u201D`);
      rec.thought.style.display = "";
    } else {
      rec.thought.style.display = "none";
    }

    const wantClass = isSelected ? "expanded active" : "";
    if (rec.li.className !== wantClass) rec.li.className = wantClass;
    const wantTitle = isSelected ? "tap to collapse" : "tap to expand";
    if (rec.row.title !== wantTitle) rec.row.title = wantTitle;

    if (isSelected && info) {
      if (!rec.detail) {
        rec.detail = document.createElement("div");
        rec.detail.className = "bot-detail";
        rec.li.append(rec.detail);
      }
      // Preserve scroll positions of inner scrollable lists across rerenders.
      const memScroll = rec.detail.querySelector(".mem-list")?.scrollTop ?? 0;
      const recentScroll = rec.detail.querySelector(".recent-list")?.scrollTop ?? 0;
      rec.detail.innerHTML = renderBotDetail(info);
      const mem = rec.detail.querySelector(".mem-list");
      if (mem) mem.scrollTop = memScroll;
      const rec2 = rec.detail.querySelector(".recent-list");
      if (rec2) rec2.scrollTop = recentScroll;
    } else if (rec.detail) {
      rec.detail.remove();
      rec.detail = null;
    }
  }

  for (const [id, rec] of botRowEls.entries()) {
    if (!seen.has(id)) {
      rec.li.remove();
      botRowEls.delete(id);
    }
  }
}

function refreshPanels() {
  if (!world) return;
  const stats = JSON.parse(world.stats());
  const moodColor =
    stats.avg_mood > 0 ? "#7ee0a1" : stats.avg_mood < -10 ? "#ff7878" : "#d8e1ea";
  const complaintsColor = stats.complaints > 0 ? "#e6b45a" : "#d8e1ea";
  const stressColor =
    stats.avg_stress > 60 ? "#ff7878" : stats.avg_stress > 35 ? "#e6b45a" : "#7ee0a1";
  const thirstColor =
    stats.avg_thirst > 60 ? "#ff7878" : stats.avg_thirst > 35 ? "#e6b45a" : "#7ee0a1";
  const warmthColor =
    stats.avg_warmth < 30 ? "#8ad0ff" : stats.avg_warmth < 60 ? "#e6b45a" : "#7ee0a1";
  const weatherGlyph =
    stats.weather === "rain" ? "\u2614" :
    stats.weather === "clearing" ? "\u26c5" : "\u2600";
  const nightGlyph = stats.night ? " \u263e" : " \u2600";
  statsEl.innerHTML = `
    tick <b>${stats.tick}</b> &nbsp;·&nbsp;
    ${weatherGlyph}${nightGlyph} &nbsp;·&nbsp;
    bots <b>${stats.bots}</b> &nbsp;·&nbsp;
    berries <b>${stats.berries}</b> (cooked <b>${stats.cooked}</b>) &nbsp;·&nbsp;
    mushrooms <b>${stats.mushrooms}</b> &nbsp;·&nbsp;
    trees <b>${stats.trees}</b> &nbsp;·&nbsp;
    saplings <b>${stats.saplings}</b> &nbsp;·&nbsp;
    homes <b>${stats.homes}</b> &nbsp;·&nbsp;
    fires <b>${stats.fires}</b> &nbsp;·&nbsp;
    logs <b>${stats.logs}</b> · stones <b>${stats.stones}</b> &nbsp;·&nbsp;
    paths <b>${stats.paths}</b> · puddles <b>${stats.puddles}</b> &nbsp;·&nbsp;
    shrines <b>${stats.shrines}</b> · graves <b>${stats.graves}</b> &nbsp;·&nbsp;
    fields <b>${stats.fields || 0}</b> · fish <b>${stats.fish_tiles || 0}</b> &nbsp;·&nbsp;
    chatting <b>${stats.chatting}</b> &nbsp;·&nbsp;
    hauling <b>${stats.hauling}</b> &nbsp;·&nbsp;
    toolmakers <b>${stats.toolmakers}</b> (<b>${stats.axes}</b> axes) &nbsp;·&nbsp;
    cooks <b>${stats.cooks}</b> · diggers <b>${stats.diggers}</b> · healers <b>${stats.healers}</b> · fishermen <b>${stats.fishermen || 0}</b> &nbsp;·&nbsp;
    complaints <b style="color:${complaintsColor}">${stats.complaints}</b> &nbsp;·&nbsp;
    mood <b style="color:${moodColor}">${stats.avg_mood.toFixed(1)}</b> &nbsp;·&nbsp;
    thirst <b style="color:${thirstColor}">${stats.avg_thirst.toFixed(0)}</b> &nbsp;·&nbsp;
    stress <b style="color:${stressColor}">${stats.avg_stress.toFixed(0)}</b> &nbsp;·&nbsp;
    warmth <b style="color:${warmthColor}">${stats.avg_warmth.toFixed(0)}</b>
  `;

  const bots = JSON.parse(world.bots_summary());
  const infoRaw = world.selected_info();
  const info = infoRaw === "null" ? null : JSON.parse(infoRaw);
  if (info) {
    selectedBotId = info.id;
    // Pass a lightweight id→name map so relationship rows can render names.
    const nameById = {};
    for (const b of bots) nameById[b.id] = b.name;
    info._nameById = nameById;
  } else if (selectedBotId !== null) {
    // Selection cleared (e.g. clicked empty grass).
    selectedBotId = null;
  }

  reconcileBots(bots, info);

  // When selection changes, bring the row into view within the scrolling panel.
  if (selectedBotId !== prevSelectedForScroll) {
    if (selectedBotId !== null) {
      const activeLi = botsEl.querySelector(
        `li[data-id="${selectedBotId}"]`
      );
      activeLi?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
    prevSelectedForScroll = selectedBotId;
  }

  const log = JSON.parse(world.event_log());
  logEl.innerHTML = log
    .slice()
    .reverse()
    .map((l) => `<li>${escapeHtml(l)}</li>`)
    .join("");
}

function renderBotDetail(b) {
  const homeLabel = b.home_name ? `"${b.home_name}"` : `home`;
  const home = b.home
    ? `${homeLabel} at (${b.home[0]}, ${b.home[1]})`
    : `no home yet`;

  // Build a quick id→name map from the bots_summary JSON so relationship
  // rows can show who is who rather than a cryptic #7.
  const nameById = b._nameById || {};
  const relations = b.relations
    .slice()
    .sort((a, z) => z.affinity - a.affinity)
    .slice(0, 5)
    .map((r) => {
      const sign = r.affinity > 0 ? "+" : "";
      const color = r.affinity > 0 ? "#7ee0a1" : r.affinity < 0 ? "#ff7878" : "#8a96a4";
      const label = nameById[r.id] ? escapeHtml(nameById[r.id]) : `#${r.id}`;
      return `<span style="color:${color}">${label} ${sign}${r.affinity}</span>`;
    })
    .join(" · ") || '<span style="color:#8a96a4">no one yet</span>';

  const mem = b.memory
    .slice(-6)
    .reverse()
    .map((m) => `<li>${escapeHtml(m.kind)} @ (${m.x},${m.y})</li>`)
    .join("") || '<li style="color:#8a96a4">empty</li>';

  const recent = b.recent
    .slice(-6)
    .reverse()
    .map((t) => `<li>&ldquo;${escapeHtml(t)}&rdquo;</li>`)
    .join("") || '<li style="color:#8a96a4">quiet</li>';

  const chatBlock = b.chatting
    ? `<div class="chat-line">chatting with <b>${escapeHtml(
        b.chatting.name || `#${b.chatting.id}`
      )}</b> <span style="color:#8a96a4">· ${b.chatting.ticks}t</span></div>`
    : "";

  const giftsBlock =
    b.gifts_given > 0 || b.gifts_received > 0
      ? `<div class="gift-line">gifts <b>${b.gifts_given}</b> given · <b>${b.gifts_received}</b> received</div>`
      : "";

  const toolBlock =
    b.job === "toolmaker" || b.has_tool > 0 || b.trees_chopped > 0
      ? `<div class="tool-line">${
          b.has_tool > 0
            ? `stone axe ready <b>\u00d7${b.has_tool}</b>`
            : `<span style="color:#8a96a4">no tool \u2014 needs a rock</span>`
        } &nbsp;\u00b7&nbsp; felled <b>${b.trees_chopped}</b></div>`
      : "";

  const carryKind = b.carrying && b.carrying.kind ? b.carrying.kind : "nothing";
  const carryColor = b.carrying && b.carrying.color
    ? `rgb(${b.carrying.color[0]},${b.carrying.color[1]},${b.carrying.color[2]})`
    : "#8a96a4";
  const carryBlock =
    carryKind !== "nothing" || b.deliveries > 0 || b.berries_cooked > 0
      ? `<div class="tool-line">${
          carryKind !== "nothing"
            ? `carrying <b style="color:${carryColor}">${escapeHtml(carryKind)}</b>`
            : `<span style="color:#8a96a4">empty-handed</span>`
        } &nbsp;\u00b7&nbsp; delivered <b>${b.deliveries}</b>${
          b.berries_cooked > 0 ? ` &nbsp;\u00b7&nbsp; cooked <b>${b.berries_cooked}</b>` : ""
        }${
          b.reputation !== 0 ? ` &nbsp;\u00b7&nbsp; rep <b>${b.reputation > 0 ? "+" : ""}${b.reputation}</b>` : ""
        }</div>`
      : "";

  return `
    <div class="detail-sub">${escapeHtml(b.dominant)} · ${escapeHtml(b.goal)}</div>

    <div class="thought-bubble">${escapeHtml(b.thought)}</div>

    ${chatBlock}

    <div class="bars">
      ${bar("hunger", b.hunger)}
      ${bar("thirst", b.thirst)}
      ${bar("energy", b.energy)}
      ${bar("warmth", b.warmth)}
      ${bar("social", b.social)}
      ${bar("boredom", b.boredom)}
      ${bar("stress", b.stress)}
      ${moodBar(b.mood)}
    </div>

    <div class="traits">
      <div><b>curiosity</b> ${b.traits.curiosity.toFixed(2)}</div>
      <div><b>sociability</b> ${b.traits.sociability.toFixed(2)}</div>
      <div><b>aggression</b> ${b.traits.aggression.toFixed(2)}</div>
      <div><b>industry</b> ${b.traits.industriousness.toFixed(2)}</div>
      <div><b>bravery</b> ${b.traits.bravery.toFixed(2)}</div>
      <div><b>age</b> ${b.age}t</div>
    </div>

    <div class="meta">${home} · at (${b.x}, ${b.y}) · step cadence ${b.speed}t</div>

    ${toolBlock}

    ${carryBlock}

    ${giftsBlock}

    <div class="section-sub">Relationships</div>
    <div style="font-size:11px; margin-top:4px">${relations}</div>

    <div class="section-sub">Memory</div>
    <ul class="mem-list">${mem}</ul>

    <div class="section-sub">Recent thoughts</div>
    <ul class="recent-list">${recent}</ul>
  `;
}

// Map a (kind, value) pair to a color reflecting how good/bad this state is.
// Green = the bot is fine on this axis, amber = creeping toward trouble,
// red/pink = actively suffering. Bar WIDTH still shows raw magnitude so
// you can eyeball "how hungry is this bot" independently of the color.
function healthColor(kind, value) {
  if (kind === "energy" || kind === "warmth") {
    // low = bad (tired / frozen)
    if (value < 30) return "#ff7878";
    if (value < 60) return "#e6b45a";
    return "#7ee0a1";
  }
  if (kind === "hunger" || kind === "boredom" || kind === "thirst" || kind === "stress") {
    // high = bad
    if (value < 40) return "#7ee0a1";
    if (value < 70) return "#e6b45a";
    return "#ff7878";
  }
  if (kind === "social") {
    // high social drive = lonely
    if (value < 40) return "#7ee0a1";
    if (value < 60) return "#e6b45a";
    return "#e68cc8";
  }
  return "#8ad0ff";
}

function bar(kind, value) {
  const w = Math.max(0, Math.min(100, value));
  const color = healthColor(kind, value);
  return `
    <div class="bar ${kind}">
      <span>${kind}</span>
      <span class="track"><span class="fill" style="width:${w}%; background:${color}"></span></span>
      <span style="color:${color}">${value.toFixed(0)}</span>
    </div>
  `;
}

function moodBar(value) {
  // Mood is signed in [-100, 100] — render as a centred bar so negative
  // mood extends left-of-centre in red and positive extends right in green.
  const half = Math.max(0, Math.min(50, Math.abs(value) / 2));
  const color = value <= -20 ? "#ff7878" : value >= 20 ? "#7ee0a1" : "#8a96a4";
  // Fill either from 50% leftwards (negative) or from 50% rightwards (positive).
  const left = value < 0 ? 50 - half : 50;
  return `
    <div class="bar mood">
      <span>mood</span>
      <span class="track">
        <span class="tick-center"></span>
        <span class="fill" style="left:${left}%; width:${half}%; background:${color}"></span>
      </span>
      <span style="color:${color}">${value.toFixed(0)}</span>
    </div>
  `;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

boot();
