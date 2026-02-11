// Main game module
const Game = {
    // Game states
    STATE: {
        TITLE: 'title',
        PLAYING: 'playing',
        GAMEOVER: 'gameover'
    },

    state: 'title',
    canvas: null,
    ctx: null,
    lastTime: 0,

    // Camera settings
    camera: {
        x: 0,
        y: 1000,      // Camera height
        z: 0,
        depth: 100,   // Camera depth (affects FOV)
        playerZ: 500  // Distance from camera to player
    },

    // Race settings
    totalLaps: 3,
    currentLap: 1,
    lapStartZ: 0,
    hasPassedHalf: false,
    raceTime: 0,
    bestTime: null,

    init() {
        // Get canvas
        this.canvas = document.getElementById('game-canvas');
        this.ctx = this.canvas.getContext('2d');

        // Initialize all modules
        Render.init(this.canvas);
        Input.init();
        Background.init();
        Sprites.init();
        Road.init();
        Car.init();
        Traffic.init();

        // Load best time from localStorage
        this.bestTime = localStorage.getItem('retroRacerBest');

        // Start game loop
        this.lastTime = performance.now();
        requestAnimationFrame((t) => this.loop(t));
    },

    loop(currentTime) {
        // Calculate delta time (capped to prevent huge jumps)
        const dt = Math.min((currentTime - this.lastTime) / 1000, 0.1);
        this.lastTime = currentTime;

        // Update and render based on state
        switch (this.state) {
            case this.STATE.TITLE:
                this.updateTitle(dt);
                break;
            case this.STATE.PLAYING:
                this.updatePlaying(dt);
                break;
            case this.STATE.GAMEOVER:
                this.updateGameOver(dt);
                break;
        }

        // Continue loop
        requestAnimationFrame((t) => this.loop(t));
    },

    updateTitle(dt) {
        // Render a preview of the road
        this.renderGame();

        // Check for start
        if (Input.keys.enter) {
            this.startGame();
            Input.keys.enter = false;
        }
    },

    updatePlaying(dt) {
        // Update game time
        this.raceTime += dt;

        // Update car
        Car.update(dt, Input);

        // Update traffic
        Traffic.update(dt, Car.z);

        // Update camera to follow player
        this.camera.x = Car.x * Road.roadWidth;
        this.camera.z = Car.z - this.camera.playerZ;

        // Get current segment for camera height
        const segment = Road.getSegment(Car.z);
        const targetY = 1000 + segment.point.world.y;
        this.camera.y += (targetY - this.camera.y) * 0.1;

        // Check for lap completion
        this.checkLapProgress();

        // Update HUD
        this.updateHUD();

        // Render game
        this.renderGame();
    },

    updateGameOver(dt) {
        // Keep rendering but don't update
        this.renderGame();

        // Check for restart
        if (Input.keys.enter) {
            this.startGame();
            Input.keys.enter = false;
        }
    },

    renderGame() {
        // Clear canvas
        Render.clear();

        // Draw background layers
        Background.render(this.ctx, Car.x, Car.speed);

        // Draw road
        Road.render(this.ctx, Car, this.camera);

        // Draw player car
        Car.render(this.ctx);
    },

    checkLapProgress() {
        // Track lap progress
        const halfTrack = Road.trackLength / 2;

        // Check if passed halfway point
        if (!this.hasPassedHalf && Car.z > halfTrack) {
            this.hasPassedHalf = true;
        }

        // Check for lap completion (crossed start line after passing halfway)
        if (this.hasPassedHalf && Car.z < halfTrack && Car.z > 0) {
            this.currentLap++;
            this.hasPassedHalf = false;

            if (this.currentLap > this.totalLaps) {
                this.endRace();
            }
        }
    },

    updateHUD() {
        // Update speed display
        document.getElementById('speed').textContent = Car.getSpeedMPH() + ' MPH';

        // Update lap counter
        document.getElementById('lap').textContent =
            'LAP ' + Math.min(this.currentLap, this.totalLaps) + '/' + this.totalLaps;

        // Update time display
        document.getElementById('time').textContent = 'TIME ' + this.formatTime(this.raceTime);
    },

    formatTime(seconds) {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        const ms = Math.floor((seconds % 1) * 100);
        return mins + ':' + secs.toString().padStart(2, '0') + '.' + ms.toString().padStart(2, '0');
    },

    startGame() {
        // Hide title/gameover screens
        document.getElementById('title-screen').classList.add('hidden');
        document.getElementById('game-over').classList.add('hidden');

        // Reset game state
        this.state = this.STATE.PLAYING;
        this.currentLap = 1;
        this.hasPassedHalf = false;
        this.raceTime = 0;

        // Reset player and traffic
        Car.reset();
        Traffic.reset();

        // Reset camera
        this.camera.z = -this.camera.playerZ;
    },

    endRace() {
        this.state = this.STATE.GAMEOVER;

        // Check for best time
        if (!this.bestTime || this.raceTime < parseFloat(this.bestTime)) {
            this.bestTime = this.raceTime;
            localStorage.setItem('retroRacerBest', this.raceTime);
        }

        // Show game over screen
        const gameOver = document.getElementById('game-over');
        gameOver.classList.remove('hidden');
        document.getElementById('final-time').textContent =
            'Time: ' + this.formatTime(this.raceTime) +
            (this.bestTime === this.raceTime ? ' (NEW BEST!)' : '');
    }
};

// Start game when page loads
window.addEventListener('load', () => {
    Game.init();
});
