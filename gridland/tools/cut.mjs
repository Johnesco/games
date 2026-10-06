#!/usr/bin/env node
// Gridland build pipeline. Every design change ships as a numbered, frozen,
// playable build under builds/build-N/, recorded in builds.json, which the
// diary page (../index.html) renders.
//
//   node tools/cut.mjs cut --title "Short headline" --notes "Paragraph." [--notes "…"] [--known "…"]
//       wasm-pack build → cargo test → probe → copy www/ to builds/build-N/ → append builds.json
//   node tools/cut.mjs backfill
//       (re)create the historical builds listed in tools/history.json from git
//   node tools/cut.mjs probe [commit]
//       print the probe summary for the working tree (or a past commit)
//
// After `cut`: commit, then tag:  git tag gridland-build-N
// See ../CLAUDE.md for the full process.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const BUILDS = path.join(ROOT, "builds");
const MANIFEST = path.join(ROOT, "builds.json");
const HISTORY = path.join(ROOT, "tools", "history.json");
const PROBE_SRC = path.join(ROOT, "examples", "probe.rs");
// Shared target dir so historical probes reuse compiled dependencies.
// git archive run from a subdirectory only includes that subdirectory, so run it from the repo root.
const REPO = path.resolve(ROOT, "..");
const PROBE_TARGET = path.join(ROOT, "target", "probe-history");

const sh = (cmd, args, opts = {}) =>
  execFileSync(cmd, args, { stdio: ["ignore", "pipe", "inherit"], encoding: "utf8", maxBuffer: 1 << 28, ...opts });

const loadManifest = () =>
  fs.existsSync(MANIFEST) ? JSON.parse(fs.readFileSync(MANIFEST, "utf8")) : { builds: [] };

function saveManifest(m) {
  m.builds.sort((a, b) => a.n - b.n);
  m.latest = m.builds.length ? m.builds[m.builds.length - 1].n : null;
  m.generated = today();
  fs.writeFileSync(MANIFEST, JSON.stringify(m, null, 2) + "\n");
}

const today = () => new Date().toISOString().slice(0, 10);

// --- probe -----------------------------------------------------------------

// Fields every build's stats() has had since build 1.
const FINAL_FIELDS = ["bots", "homes", "berries", "trees", "fires", "complaints",
  "logs_chopped_total", "berries_cooked_total", "avg_mood"];

function summarise(raw) {
  const runs = raw.runs.map((r) => {
    const samples = r.samples;
    const last = samples[samples.length - 1];
    const final = {};
    for (const f of FINAL_FIELDS) final[f] = f in last ? last[f] : null;
    return {
      seed: r.seed,
      population: samples.map((s) => s.bots),
      berries: samples.map((s) => s.berries),
      final,
    };
  });
  const mean = {};
  for (const f of FINAL_FIELDS) {
    const vals = runs.map((r) => r.final[f]).filter((v) => v !== null);
    mean[f] = vals.length ? Math.round((vals.reduce((a, b) => a + b, 0) / vals.length) * 10) / 10 : null;
  }
  return { ticks: raw.ticks, sample_every: raw.sample_every, mean, runs };
}

function runProbe(crateDir) {
  console.log(`  probe: ${path.relative(ROOT, crateDir) || "."}`);
  const out = sh("cargo", ["run", "--release", "--quiet", "--example", "probe"], {
    cwd: crateDir,
    env: { ...process.env, CARGO_TARGET_DIR: PROBE_TARGET, RUSTFLAGS: "-Awarnings" },
  });
  return summarise(JSON.parse(out.trim().split("\n").pop()));
}

// Extract `<commit>:gridland/<sub>` into dir. The tar is written into dir and
// extracted with cwd=dir, so no path ever crosses the Windows/MSYS boundary.
function gitExtract(commit, sub, dir) {
  fs.mkdirSync(dir, { recursive: true });
  const tar = path.join(dir, ".extract.tar");
  sh("git", ["archive", "--format=tar", "-o", tar, `${commit}:gridland${sub ? "/" + sub : ""}`], { cwd: REPO });
  sh("tar", ["-xf", ".extract.tar"], { cwd: dir });
  fs.rmSync(tar);
  return dir;
}

function probeCommit(commit) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "gridland-src-"));
  try {
    gitExtract(commit, "", tmp);
    fs.mkdirSync(path.join(tmp, "examples"), { recursive: true });
    fs.copyFileSync(PROBE_SRC, path.join(tmp, "examples", "probe.rs"));
    return runProbe(tmp);
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

// --- build copies ------------------------------------------------------------

// wasm-pack writes pkg/.gitignore = "*"; inside builds/ it would hide the build from git.
const dropPkgIgnore = (dir) => fs.rmSync(path.join(dir, "pkg", ".gitignore"), { force: true });

function stampBuildTag(dir, n) {
  const p = path.join(dir, "index.html");
  const html = fs.readFileSync(p, "utf8");
  const tag = /<a class="build-tag"[^>]*>[^<]*<\/a>/;
  if (!tag.test(html)) throw new Error("www/index.html has no build-tag link to stamp");
  fs.writeFileSync(p, html.replace(tag, `<a class="build-tag" href="../../">build ${n} &middot; diary</a>`));
}

// --- commands ------------------------------------------------------------------

function backfill() {
  const history = JSON.parse(fs.readFileSync(HISTORY, "utf8")).builds;
  const m = loadManifest();
  for (const h of history) {
    console.log(`build ${h.n} ← ${h.commit}`);
    const dir = path.join(BUILDS, `build-${h.n}`);
    fs.rmSync(dir, { recursive: true, force: true });
    gitExtract(h.commit, "www", dir);
    dropPkgIgnore(dir);
    const entry = { ...h, play: `builds/build-${h.n}/`, recovered: true, probe: probeCommit(h.commit) };
    m.builds = m.builds.filter((b) => b.n !== h.n).concat(entry);
  }
  saveManifest(m);
  console.log(`builds.json: ${m.builds.length} builds, latest ${m.latest}`);
}

function cut(opts) {
  if (!opts.title || !opts.notes.length) throw new Error('cut needs --title "…" and at least one --notes "…"');
  const m = loadManifest();
  const n = (m.latest ?? 0) + 1;
  const dir = path.join(BUILDS, `build-${n}`);
  if (fs.existsSync(dir)) throw new Error(`${path.relative(ROOT, dir)} already exists`);

  console.log("1/4 wasm-pack build");
  sh("wasm-pack", ["build", "--target", "web", "--out-dir", "www/pkg", "--release"], { cwd: ROOT, stdio: "inherit" });
  console.log("2/4 cargo test --release");
  sh("cargo", ["test", "--release"], { cwd: ROOT, stdio: "inherit" });
  console.log("3/4 probe");
  const probe = runProbe(ROOT);
  console.log(`4/4 freeze www/ → builds/build-${n}/`);
  fs.cpSync(path.join(ROOT, "www"), dir, { recursive: true });
  dropPkgIgnore(dir);
  stampBuildTag(dir, n);

  const head = sh("git", ["rev-parse", "--short", "HEAD"], { cwd: ROOT }).trim();
  const dirty = sh("git", ["status", "--porcelain", "--", "."], { cwd: ROOT }).trim() !== "";
  m.builds.push({
    n, date: today(), title: opts.title, notes: opts.notes,
    ...(opts.known.length ? { known: opts.known } : {}),
    // The build's own commit doesn't exist yet; record what it was cut on top of.
    base: head, base_dirty: dirty,
    play: `builds/build-${n}/`, probe,
  });
  saveManifest(m);
  console.log(`\nbuild ${n} cut. Next: commit, then  git tag gridland-build-${n}`);
}

function parseArgs(argv) {
  const o = { notes: [], known: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--title") o.title = argv[++i];
    else if (a === "--notes") o.notes.push(argv[++i]);
    else if (a === "--known") o.known.push(argv[++i]);
    else (o._ ??= []).push(a);
  }
  return o;
}

const [cmd, ...rest] = process.argv.slice(2);
const opts = parseArgs(rest);
if (cmd === "backfill") backfill();
else if (cmd === "cut") cut(opts);
else if (cmd === "probe") console.log(JSON.stringify(opts._?.[0] ? probeCommit(opts._[0]) : runProbe(ROOT), null, 2));
else {
  console.error("usage: node tools/cut.mjs cut|backfill|probe   (see header comment)");
  process.exit(1);
}
