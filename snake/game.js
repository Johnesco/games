import kaplay from "https://esm.sh/kaplay@next";

const k = kaplay({
  background: [20, 20, 30],
  width: 800,
  height: 600,
  letterbox: true,
});

const CELL = 40;
const COLS = 20;
const ROWS = 15;
const TICK = 0.12;

const sameCell = (a, b) => a.x === b.x && a.y === b.y;

k.scene("play", () => {
  let snake = [k.vec2(5, 7), k.vec2(4, 7), k.vec2(3, 7)];
  let dir = k.vec2(1, 0);
  let queued = dir;
  let food = randomFoodCell();
  let score = 0;

  function randomFoodCell() {
    while (true) {
      const c = k.vec2(k.randi(0, COLS), k.randi(0, ROWS));
      if (!snake.some(s => sameCell(s, c))) return c;
    }
  }

  let foodObj = null;
  function renderFood() {
    if (foodObj) foodObj.destroy();
    foodObj = k.add([
      k.rect(CELL - 4, CELL - 4),
      k.pos(food.x * CELL + 2, food.y * CELL + 2),
      k.color(240, 80, 80),
    ]);
  }

  const segObjs = [];
  function renderSnake() {
    segObjs.forEach(o => o.destroy());
    segObjs.length = 0;
    snake.forEach((cell, i) => {
      segObjs.push(k.add([
        k.rect(CELL - 2, CELL - 2),
        k.pos(cell.x * CELL + 1, cell.y * CELL + 1),
        k.color(i === 0 ? 120 : 70, 230, i === 0 ? 160 : 130),
      ]));
    });
  }

  const scoreLabel = k.add([
    k.text("Score: 0", { size: 20 }),
    k.pos(16, 8),
    k.color(220, 220, 220),
  ]);

  renderFood();
  renderSnake();

  k.onKeyPress(["left", "a"],  () => { if (queued.x !==  1) queued = k.vec2(-1, 0); });
  k.onKeyPress(["right", "d"], () => { if (queued.x !== -1) queued = k.vec2( 1, 0); });
  k.onKeyPress(["up", "w"],    () => { if (queued.y !==  1) queued = k.vec2(0, -1); });
  k.onKeyPress(["down", "s"],  () => { if (queued.y !== -1) queued = k.vec2(0,  1); });

  k.loop(TICK, () => {
    dir = queued;
    const head = snake[0];
    const newHead = k.vec2(head.x + dir.x, head.y + dir.y);

    if (newHead.x < 0 || newHead.x >= COLS || newHead.y < 0 || newHead.y >= ROWS) {
      return k.go("over", score);
    }
    const ate = sameCell(food, newHead);
    const body = ate ? snake : snake.slice(0, -1);
    if (body.some(s => sameCell(s, newHead))) {
      return k.go("over", score);
    }

    snake.unshift(newHead);
    if (ate) {
      score++;
      scoreLabel.text = "Score: " + score;
      food = randomFoodCell();
      renderFood();
    } else {
      snake.pop();
    }
    renderSnake();
  });
});

k.scene("over", (score) => {
  k.add([
    k.text(`Game over\nScore: ${score}\n\nPress space to play again`, {
      size: 28,
      align: "center",
    }),
    k.pos(k.center()),
    k.anchor("center"),
    k.color(240, 200, 80),
  ]);
  k.onKeyPress("space", () => k.go("play"));
});

k.go("play");
