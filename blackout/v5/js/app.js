// Default story text
const defaultStory = `Whether I shall turn out to be the hero of my own life, or whether that
station will be held by anybody else, these pages must show. To begin my
life with the beginning of my life, I record that I was born (as I have
been informed and believe) on a Friday, at twelve o'clock at night.
It was remarked that the clock began to strike, and I began to cry,
simultaneously.

In consideration of the day and hour of my birth, it was declared by
the nurse, and by some sage women in the neighbourhood who had taken a
lively interest in me several months before there was any possibility
of our becoming personally acquainted, first, that I was destined to be
unlucky in life; and secondly, that I was privileged to see ghosts and
spirits; both these gifts inevitably attaching, as they believed, to
all unlucky infants of either gender, born towards the small hours on a
Friday night.

I need say nothing here, on the first head, because nothing can show
better than my history whether that prediction was verified or falsified
by the result. On the second branch of the question, I will only remark,
that unless I ran through that part of my inheritance while I was still
a baby, I have not come into it yet. But I do not at all complain of
having been kept out of this property; and if anybody else should be in
the present enjoyment of it, he is heartily welcome to keep it.`;

// State
let blockedOut = false;
let blockoutInProgress = false;
let currentStoryText = defaultStory;

// DOM Elements
const storyEl = document.getElementById('story');
const poemEl = document.getElementById('poem');
const storyForm = document.getElementById('story-form');
const storyInput = document.getElementById('story-input');
const btnBlockout = document.getElementById('btn-blockout');
const btnClear = document.getElementById('btn-clear');
const btnReset = document.getElementById('btn-reset');
const btnSave = document.getElementById('btn-save');
const btnLoad = document.getElementById('btn-load');
const btnExport = document.getElementById('btn-export');
const btnImport = document.getElementById('btn-import');
const btnImage = document.getElementById('btn-image');
const btnShare = document.getElementById('btn-share');
const savePanel = document.getElementById('save-panel');
const loadPanel = document.getElementById('load-panel');
const saveName = document.getElementById('save-name');
const saveConfirm = document.getElementById('save-confirm');
const savedList = document.getElementById('saved-list');
const fileInput = document.getElementById('file-input');
const toastEl = document.getElementById('toast');
const sharedBanner = document.getElementById('shared-banner');
const bannerDismiss = document.getElementById('banner-dismiss');

const STORAGE_KEY = 'blackout-poems';

/**
 * Tokenize text into words, punctuation, and spaces
 * Each token is an object with type and value
 */
function tokenize(text) {
  // Match: words (letters/numbers) | punctuation | whitespace
  const regex = /([a-zA-Z0-9]+)|([.,!?;:'"()\u2014\-\[\]{}\u201c\u201d])|(\s+)/g;
  const tokens = [];
  let match;

  while ((match = regex.exec(text)) !== null) {
    if (match[1]) {
      tokens.push({ type: 'word', value: match[1] });
    } else if (match[2]) {
      tokens.push({ type: 'punctuation', value: match[2] });
    } else if (match[3]) {
      tokens.push({ type: 'space', value: match[3] });
    }
  }

  return tokens;
}

/**
 * Display story with each token wrapped in a span
 * Spaces are rendered as-is (not clickable)
 * Words and punctuation are clickable tokens
 */
function displayStory(text) {
  currentStoryText = text;
  const tokens = tokenize(text);
  const html = tokens.map(token => {
    if (token.type === 'space') {
      // Spaces are just text, not clickable
      return token.value;
    }
    // Words and punctuation are clickable tokens
    return `<span class="token ${token.type}">${token.value}</span>`;
  }).join('');

  storyEl.innerHTML = html;
}

/**
 * Update poem with all marked tokens (words and punctuation)
 */
function updatePoem() {
  const markedTokens = document.querySelectorAll('.story span.token.marked');

  if (markedTokens.length === 0) {
    poemEl.textContent = '(your poem appears here)';
  } else {
    const poemText = Array.from(markedTokens)
      .map(span => span.textContent)
      .join(' ');
    poemEl.textContent = poemText;
  }
}

/**
 * Handle token click - toggle marked state
 */
function handleTokenClick(e) {
  const target = e.target;

  // Only handle clicks on token spans
  if (!target.classList.contains('token')) return;

  // Don't allow clicking on faded tokens
  if (target.classList.contains('faded')) return;

  // Don't allow clicking during blockout animation
  if (blockoutInProgress) return;

  // Remove blocked state and animation classes if present
  target.classList.remove('blocked', 'animate-in', 'animate-out');
  target.classList.toggle('marked');

  updatePoem();
}

/**
 * Toggle blockout on unmarked tokens with staggered animation
 */
function toggleBlockout() {
  if (blockoutInProgress) return;

  const unmarkedTokens = Array.from(
    document.querySelectorAll('.story span.token:not(.marked)')
  );

  if (unmarkedTokens.length === 0) return;

  blockoutInProgress = true;
  btnBlockout.disabled = true;

  // Auto-cap delay so total stagger doesn't exceed ~3 seconds
  const baseDelay = Math.min(12, 3000 / unmarkedTokens.length);

  if (blockedOut) {
    // Unblock: reveal in reverse order
    const total = unmarkedTokens.length;
    unmarkedTokens.reverse().forEach((span, i) => {
      const jitter = Math.random() * 25;
      const delay = baseDelay * i + jitter;

      setTimeout(() => {
        span.classList.add('animate-out');
        span.addEventListener('animationend', function handler() {
          span.removeEventListener('animationend', handler);
          span.classList.remove('blocked', 'animate-out');
        });

        // Last token: mark animation complete
        if (i === total - 1) {
          setTimeout(() => {
            blockoutInProgress = false;
            btnBlockout.disabled = false;
          }, 200);
        }
      }, delay);
    });
    blockedOut = false;
  } else {
    // Block: sweep forward
    const total = unmarkedTokens.length;
    unmarkedTokens.forEach((span, i) => {
      const jitter = Math.random() * 25;
      const delay = baseDelay * i + jitter;

      setTimeout(() => {
        span.classList.add('blocked', 'animate-in');
        span.addEventListener('animationend', function handler() {
          span.removeEventListener('animationend', handler);
          span.classList.remove('animate-in');
        });

        // Last token: mark animation complete
        if (i === total - 1) {
          setTimeout(() => {
            blockoutInProgress = false;
            btnBlockout.disabled = false;
          }, 200);
        }
      }, delay);
    });
    blockedOut = true;
  }
}

/**
 * Clear all marked tokens
 */
function clearMarked() {
  const allTokens = document.querySelectorAll('.story span.token');
  allTokens.forEach(span => {
    span.classList.remove('marked', 'animate-in', 'animate-out');
  });
  poemEl.textContent = '(your poem appears here)';
}

/**
 * Reset everything
 */
function resetAll() {
  const allTokens = document.querySelectorAll('.story span.token');
  allTokens.forEach(span => {
    span.classList.remove('blocked', 'faded', 'marked', 'animate-in', 'animate-out');
  });
  poemEl.textContent = '(your poem appears here)';
  blockedOut = false;
  blockoutInProgress = false;
  btnBlockout.disabled = false;
}

/**
 * Handle new story submission
 */
function handleSubmit(e) {
  e.preventDefault();
  const newText = storyInput.value.trim();
  if (newText) {
    displayStory(newText);
    storyInput.value = '';
    blockedOut = false;
    blockoutInProgress = false;
    btnBlockout.disabled = false;
    poemEl.textContent = '(your poem appears here)';
  }
}

/**
 * Generate a subtle noise texture and apply to the story section
 * Uses a temporary canvas to create a tiling data URI
 */
function generateNoiseTexture() {
  const size = 128;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d');
  const imageData = ctx.createImageData(size, size);
  const data = imageData.data;

  for (let i = 0; i < data.length; i += 4) {
    const value = Math.random() * 20 + 240; // light noise
    data[i] = value;     // R
    data[i + 1] = value; // G
    data[i + 2] = value; // B
    data[i + 3] = 12;    // very low alpha for subtlety
  }

  ctx.putImageData(imageData, 0, 0);
  const dataURI = canvas.toDataURL('image/png');

  const storySection = document.querySelector('.story-section');
  if (storySection) {
    const existing = storySection.style.backgroundImage;
    storySection.style.backgroundImage = `url("${dataURI}"), ${existing || 'none'}`;
  }
}

// ──────────────────────────────────────
// Serialization
// ──────────────────────────────────────

/**
 * Read current DOM state into { story, states }
 * states maps 1:1 to non-space tokens (0=normal, 1=marked, 2=blocked)
 */
function serializeState() {
  const tokens = document.querySelectorAll('.story span.token');
  const states = Array.from(tokens).map(span => {
    if (span.classList.contains('marked')) return 1;
    if (span.classList.contains('blocked')) return 2;
    return 0;
  });
  return { story: currentStoryText, states };
}

/**
 * Restore state from { story, states }
 */
function deserializeState(data) {
  displayStory(data.story);
  const tokens = document.querySelectorAll('.story span.token');
  if (data.states && data.states.length === tokens.length) {
    tokens.forEach((span, i) => {
      if (data.states[i] === 1) span.classList.add('marked');
      else if (data.states[i] === 2) span.classList.add('blocked');
    });
    // Check if any tokens are blocked to set blockedOut flag
    blockedOut = data.states.some(s => s === 2);
  }
  blockoutInProgress = false;
  btnBlockout.disabled = false;
  updatePoem();
}

// ──────────────────────────────────────
// Toast
// ──────────────────────────────────────

let toastTimeout = null;

function showToast(msg, duration) {
  duration = duration || 2500;
  toastEl.textContent = msg;
  toastEl.classList.add('show');
  clearTimeout(toastTimeout);
  toastTimeout = setTimeout(() => toastEl.classList.remove('show'), duration);
}

// ──────────────────────────────────────
// Panel toggle
// ──────────────────────────────────────

function togglePanel(panel) {
  const isOpen = panel.classList.contains('open');
  // Close all panels first
  savePanel.classList.remove('open');
  loadPanel.classList.remove('open');
  if (!isOpen) {
    panel.classList.add('open');
  }
}

// ──────────────────────────────────────
// Feature 1: localStorage Save/Load
// ──────────────────────────────────────

function getSavedPoems() {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY)) || [];
  } catch (_) {
    return [];
  }
}

function setSavedPoems(poems) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(poems));
}

function savePoem() {
  const name = saveName.value.trim() || 'Untitled';
  const data = serializeState();
  const poems = getSavedPoems();
  poems.push({
    name: name,
    date: new Date().toISOString(),
    story: data.story,
    states: data.states
  });
  setSavedPoems(poems);
  saveName.value = '';
  savePanel.classList.remove('open');
  showToast('Poem saved');
}

function renderSavedList() {
  const poems = getSavedPoems();
  if (poems.length === 0) {
    savedList.innerHTML = '<p class="empty-message">No saved poems yet.</p>';
    return;
  }
  savedList.innerHTML = poems.map((p, i) => {
    const date = new Date(p.date).toLocaleDateString();
    return `<div class="saved-item">
      <div class="saved-info">
        <div class="saved-name">${escapeHtml(p.name)}</div>
        <div class="saved-date">${date}</div>
      </div>
      <div class="saved-actions">
        <button class="load-btn" data-index="${i}">Load</button>
        <button class="delete-btn" data-index="${i}">Delete</button>
      </div>
    </div>`;
  }).join('');
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function handleSavedListClick(e) {
  const target = e.target;
  const index = parseInt(target.dataset.index, 10);
  if (isNaN(index)) return;

  const poems = getSavedPoems();
  if (target.classList.contains('load-btn')) {
    if (poems[index]) {
      deserializeState(poems[index]);
      loadPanel.classList.remove('open');
      showToast('Poem loaded');
    }
  } else if (target.classList.contains('delete-btn')) {
    poems.splice(index, 1);
    setSavedPoems(poems);
    renderSavedList();
    showToast('Poem deleted');
  }
}

// ──────────────────────────────────────
// Feature 2: JSON Export/Import
// ──────────────────────────────────────

function exportJSON() {
  const data = serializeState();
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'blackout-poem.json';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
  showToast('Exported as JSON');
}

function importJSON(file) {
  const reader = new FileReader();
  reader.onload = function (e) {
    try {
      const data = JSON.parse(e.target.result);
      if (!data.story || !Array.isArray(data.states)) {
        showToast('Invalid poem file');
        return;
      }
      deserializeState(data);
      showToast('Poem imported');
    } catch (_) {
      showToast('Could not read file');
    }
  };
  reader.readAsText(file);
}

// ──────────────────────────────────────
// Feature 3: Image Export (canvas)
// ──────────────────────────────────────

function exportImage() {
  const data = serializeState();
  const tokens = tokenize(data.story);

  const canvasWidth = 800;
  const padding = 40;
  const fontSize = 18;
  const lineHeight = fontSize * 2;
  const font = fontSize + 'px Georgia';
  const textColor = '#2a2520';
  const bgColor = '#faf8f3';
  const markedBg = 'rgba(255, 230, 0, 0.45)';
  const blockedBg = '#1a1a1a';

  // Measure pass: calculate lines and height
  const measure = document.createElement('canvas');
  const mctx = measure.getContext('2d');
  mctx.font = font;

  // Build flat list of drawable tokens (non-space) with their widths
  // Also track spaces for word-wrap logic
  const drawTokens = [];
  let stateIdx = 0;
  for (const tok of tokens) {
    if (tok.type === 'space') {
      drawTokens.push({ type: 'space', value: tok.value });
    } else {
      const w = mctx.measureText(tok.value).width;
      const state = (data.states && stateIdx < data.states.length) ? data.states[stateIdx] : 0;
      drawTokens.push({ type: tok.type, value: tok.value, width: w, state: state });
      stateIdx++;
    }
  }

  // Layout: word-wrap tokens
  const maxX = canvasWidth - padding * 2;
  const lines = [];
  let line = [];
  let x = 0;
  const spaceWidth = mctx.measureText(' ').width;

  for (const dt of drawTokens) {
    if (dt.type === 'space') {
      // Check if space contains a newline
      if (dt.value.indexOf('\n') !== -1) {
        lines.push(line);
        line = [];
        x = 0;
        // Multiple newlines = blank lines
        const nlCount = (dt.value.match(/\n/g) || []).length;
        for (let n = 1; n < nlCount; n++) {
          lines.push([]);
        }
      } else {
        x += spaceWidth;
      }
    } else {
      if (x + dt.width > maxX && line.length > 0) {
        lines.push(line);
        line = [];
        x = 0;
      }
      line.push({ value: dt.value, width: dt.width, state: dt.state, x: x });
      x += dt.width + spaceWidth;
    }
  }
  if (line.length > 0) lines.push(line);

  // Canvas height
  const titleHeight = 50;
  const canvasHeight = titleHeight + lines.length * lineHeight + padding * 2;

  // Draw
  const canvas = document.createElement('canvas');
  canvas.width = canvasWidth;
  canvas.height = canvasHeight;
  const ctx = canvas.getContext('2d');

  // Background
  ctx.fillStyle = bgColor;
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  // Title
  ctx.font = 'bold 22px Georgia';
  ctx.fillStyle = textColor;
  ctx.textBaseline = 'top';
  ctx.fillText('Blackout Poetry', padding, padding);

  // Tokens
  ctx.font = font;
  ctx.textBaseline = 'alphabetic';

  lines.forEach((lineTokens, li) => {
    const y = titleHeight + padding + li * lineHeight + fontSize;
    lineTokens.forEach(tok => {
      const tx = padding + tok.x;
      if (tok.state === 1) {
        // Marked: yellow rect + text
        ctx.fillStyle = markedBg;
        ctx.fillRect(tx - 2, y - fontSize + 2, tok.width + 4, fontSize + 4);
        ctx.fillStyle = textColor;
        ctx.fillText(tok.value, tx, y);
      } else if (tok.state === 2) {
        // Blocked: dark rect only
        ctx.fillStyle = blockedBg;
        ctx.fillRect(tx - 2, y - fontSize + 2, tok.width + 4, fontSize + 4);
      } else {
        // Normal
        ctx.fillStyle = textColor;
        ctx.fillText(tok.value, tx, y);
      }
    });
  });

  // Download
  canvas.toBlob(function (blob) {
    if (!blob) return;
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'blackout-poem.png';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    showToast('Image saved');
  }, 'image/png');
}

// ──────────────────────────────────────
// Feature 4: URL Sharing
// ──────────────────────────────────────

/**
 * Base64url encode/decode (no padding)
 */
function base64urlEncode(uint8) {
  let binary = '';
  for (let i = 0; i < uint8.length; i++) {
    binary += String.fromCharCode(uint8[i]);
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function base64urlDecode(str) {
  str = str.replace(/-/g, '+').replace(/_/g, '/');
  while (str.length % 4) str += '=';
  const binary = atob(str);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Compress with CompressionStream, fallback to uncompressed
 */
async function compressData(str) {
  const encoded = new TextEncoder().encode(str);
  if (typeof CompressionStream === 'undefined') {
    return base64urlEncode(encoded);
  }
  try {
    const cs = new CompressionStream('deflate');
    const writer = cs.writable.getWriter();
    writer.write(encoded);
    writer.close();
    const reader = cs.readable.getReader();
    const chunks = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
    }
    let totalLength = 0;
    for (const c of chunks) totalLength += c.length;
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const c of chunks) {
      result.set(c, offset);
      offset += c.length;
    }
    return base64urlEncode(result);
  } catch (_) {
    return base64urlEncode(encoded);
  }
}

/**
 * Decompress with DecompressionStream, fallback to uncompressed
 */
async function decompressData(b64) {
  const bytes = base64urlDecode(b64);
  if (typeof DecompressionStream === 'undefined') {
    return new TextDecoder().decode(bytes);
  }
  try {
    const ds = new DecompressionStream('deflate');
    const writer = ds.writable.getWriter();
    writer.write(bytes);
    writer.close();
    const reader = ds.readable.getReader();
    const chunks = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
    }
    let totalLength = 0;
    for (const c of chunks) totalLength += c.length;
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const c of chunks) {
      result.set(c, offset);
      offset += c.length;
    }
    return new TextDecoder().decode(result);
  } catch (_) {
    // Fallback: may be uncompressed
    return new TextDecoder().decode(bytes);
  }
}

async function shareLink() {
  const data = serializeState();
  const compact = data.story + '\n' + data.states.join('');
  const encoded = await compressData(compact);
  const url = window.location.origin + window.location.pathname + '?poem=' + encoded;

  if (url.length > 8000) {
    showToast('Warning: URL is very long and may not work in all browsers', 4000);
  }

  try {
    await navigator.clipboard.writeText(url);
    showToast('Share link copied to clipboard');
  } catch (_) {
    // Fallback for older browsers
    const input = document.createElement('input');
    input.value = url;
    document.body.appendChild(input);
    input.select();
    document.execCommand('copy');
    document.body.removeChild(input);
    showToast('Share link copied to clipboard');
  }
}

async function restoreFromURL() {
  const params = new URLSearchParams(window.location.search);
  const encoded = params.get('poem');
  if (!encoded) return false;

  try {
    const raw = await decompressData(encoded);
    const nlIndex = raw.indexOf('\n');
    if (nlIndex === -1) return false;

    const story = raw.substring(0, nlIndex);
    const statesStr = raw.substring(nlIndex + 1);
    const states = Array.from(statesStr).map(Number);

    deserializeState({ story, states });
    sharedBanner.style.display = '';
    return true;
  } catch (_) {
    return false;
  }
}

// ──────────────────────────────────────
// Event Listeners
// ──────────────────────────────────────

storyEl.addEventListener('click', handleTokenClick);
btnBlockout.addEventListener('click', toggleBlockout);
btnClear.addEventListener('click', clearMarked);
btnReset.addEventListener('click', resetAll);
storyForm.addEventListener('submit', handleSubmit);

// Save/Load panel toggles
btnSave.addEventListener('click', () => togglePanel(savePanel));
btnLoad.addEventListener('click', () => {
  renderSavedList();
  togglePanel(loadPanel);
});

// Save confirm
saveConfirm.addEventListener('click', savePoem);
saveName.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    e.preventDefault();
    savePoem();
  }
});

// Load/Delete from saved list
savedList.addEventListener('click', handleSavedListClick);

// JSON Export/Import
btnExport.addEventListener('click', exportJSON);
btnImport.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', (e) => {
  if (e.target.files.length > 0) {
    importJSON(e.target.files[0]);
    fileInput.value = '';
  }
});

// Image export
btnImage.addEventListener('click', exportImage);

// Share
btnShare.addEventListener('click', shareLink);

// Dismiss shared banner
bannerDismiss.addEventListener('click', () => {
  sharedBanner.style.display = 'none';
});

// ──────────────────────────────────────
// Initialize
// ──────────────────────────────────────

(async function init() {
  // Try restoring from URL first
  const restored = await restoreFromURL();
  if (!restored) {
    displayStory(defaultStory);
  }
  generateNoiseTexture();
})();
