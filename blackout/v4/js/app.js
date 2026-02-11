// Default story text
const defaultStory = `Whether I shall turn out to be the hero of my own life, or whether that
station will be held by anybody else, these pages must show. To begin my
life with the beginning of my life, I record that I was born (as I have
been informed and believe) on a Friday, at twelve o’clock at night.
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

// DOM Elements
const storyEl = document.getElementById('story');
const poemEl = document.getElementById('poem');
const storyForm = document.getElementById('story-form');
const storyInput = document.getElementById('story-input');
const btnBlockout = document.getElementById('btn-blockout');
const btnClear = document.getElementById('btn-clear');
const btnReset = document.getElementById('btn-reset');

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

// Event Listeners
storyEl.addEventListener('click', handleTokenClick);
btnBlockout.addEventListener('click', toggleBlockout);
btnClear.addEventListener('click', clearMarked);
btnReset.addEventListener('click', resetAll);
storyForm.addEventListener('submit', handleSubmit);

// Initialize with default story
displayStory(defaultStory);

// Apply paper noise texture
generateNoiseTexture();
