// Default story text
const defaultStory = `I had scarcely thought of the theater for some years, when Kean arrived in this country; and it was more from curiosity than from any other motive, that I went to see, for the first time, the great actor of the age. I was soon lost to the recollection of being in a theater, or looking upon a great display of the "mimic art." The simplicity, earnestness, and sincerity of his acting made me forgetful of the fiction, and bore me away with the power of reality and truth. If this be acting, said I, as I returned home, I may as well make the theater my school, and henceforward study nature at second hand.
How can I describe one who is almost as full of beauties as nature itself,—who grows upon us the more we become acquainted with him, and makes us sensible that the first time we saw him in any part, however much he may have moved us, we had but a partial apprehension of the many excellences of his acting? We cease to consider it as a mere amusement. It is an intellectual feast; and he who goes to it with a disposition and capacity to relish it, will receive from it more nourishment for his mind, than he would be likely to do in many other ways in twice the time. Our faculties are opened and enlivened by it; our reflections and recollections are of an elevated kind; and the voice which is sounding in our ears, long after we have left him, creates an inward harmony which is for our good.
Kean, in truth, stands very much in that relation to other players whom we have seen, that Shakspeare does to other dramatists. One player is called classical; another makes fine points here, and another there; Kean makes more fine points than all of them together; but in him these are only little prominences, showing their bright heads above a beautifully undulated surface. A continual change is going on in him, partaking of the nature of the varying scenes he is passing through, and the many thoughts and feelings which are shifting within him.`;

// State
let blockedOut = false;

// DOM Elements
const storyEl = document.getElementById('story');
const poemEl = document.getElementById('poem');
const storyForm = document.getElementById('story-form');
const storyInput = document.getElementById('story-input');
const btnBlockout = document.getElementById('btn-blockout');
const btnClear = document.getElementById('btn-clear');
const btnReset = document.getElementById('btn-reset');

// Display story with each word wrapped in a span
function displayStory(text) {
  const words = text.split(/\s+/);
  const html = words
    .filter(word => word.trim() !== '')
    .map(word => `<span class="word">${word}</span>`)
    .join(' ');
  storyEl.innerHTML = html;
}

// Update poem with all marked words
function updatePoem() {
  const markedWords = document.querySelectorAll('.story span.marked');
  if (markedWords.length === 0) {
    poemEl.textContent = '(your poem appears here)';
  } else {
    const poemText = Array.from(markedWords)
      .map(span => span.textContent)
      .join(' ');
    poemEl.textContent = poemText;
  }
}

// Handle word click - toggle marked state
function handleWordClick(e) {
  const target = e.target;

  // Only handle clicks on word spans
  if (!target.classList.contains('word')) return;

  // Don't allow clicking on faded words
  if (target.classList.contains('faded')) return;

  // Remove blocked state if present, toggle marked
  target.classList.remove('blocked');
  target.classList.toggle('marked');

  updatePoem();
}

// Toggle blockout on unmarked words
function toggleBlockout() {
  const unmarkedWords = document.querySelectorAll('.story span.word:not(.marked)');

  if (blockedOut) {
    unmarkedWords.forEach(span => span.classList.remove('blocked'));
    blockedOut = false;
  } else {
    unmarkedWords.forEach(span => span.classList.add('blocked'));
    blockedOut = true;
  }
}

// Clear all marked words
function clearMarked() {
  const allWords = document.querySelectorAll('.story span.word');
  allWords.forEach(span => span.classList.remove('marked'));
  poemEl.textContent = '(your poem appears here)';
}

// Reset everything
function resetAll() {
  const allWords = document.querySelectorAll('.story span.word');
  allWords.forEach(span => {
    span.classList.remove('blocked', 'faded', 'marked');
  });
  poemEl.textContent = '(your poem appears here)';
  blockedOut = false;
}

// Handle new story submission
function handleSubmit(e) {
  e.preventDefault();
  const newText = storyInput.value.trim();
  if (newText) {
    displayStory(newText);
    storyInput.value = '';
    blockedOut = false;
    poemEl.textContent = '(your poem appears here)';
  }
}

// Event Listeners
storyEl.addEventListener('click', handleWordClick);
btnBlockout.addEventListener('click', toggleBlockout);
btnClear.addEventListener('click', clearMarked);
btnReset.addEventListener('click', resetAll);
storyForm.addEventListener('submit', handleSubmit);

// Initialize with default story
displayStory(defaultStory);
