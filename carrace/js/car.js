// Player car module
const Car = {
    // Position and movement
    x: 0,              // Horizontal position (-1 to 1 = road width)
    z: 0,              // Position along track
    speed: 0,          // Current speed

    // Physics constants
    maxSpeed: 15000,   // Maximum speed (segment units per second)
    accel: 10000,      // Acceleration rate
    braking: 15000,    // Braking rate
    decel: 8000,       // Natural deceleration
    offRoadDecel: 15000, // Deceleration when off-road
    offRoadLimit: 0.4, // Max speed multiplier when off-road
    steering: 3,       // Steering sensitivity
    centrifugal: 0.3,  // Centrifugal force on curves

    // State
    steerDirection: 0, // -1 left, 0 center, 1 right
    isOffRoad: false,

    init() {
        this.reset();
    },

    reset() {
        this.x = 0;
        this.z = 0;
        this.speed = 0;
        this.steerDirection = 0;
        this.isOffRoad = false;
    },

    update(dt, input) {
        const segment = Road.getSegment(this.z);
        const speedPercent = this.speed / this.maxSpeed;

        // Check if off-road
        this.isOffRoad = Math.abs(this.x) > 1;

        // Acceleration / Braking
        if (input.keys.up) {
            this.speed += this.accel * dt;
        } else if (input.keys.down) {
            this.speed -= this.braking * dt;
        } else {
            // Natural deceleration
            this.speed -= this.decel * dt;
        }

        // Off-road slowdown
        if (this.isOffRoad) {
            this.speed -= this.offRoadDecel * dt;
            if (this.speed > this.maxSpeed * this.offRoadLimit) {
                this.speed = this.maxSpeed * this.offRoadLimit;
            }
        }

        // Clamp speed
        this.speed = Math.max(0, Math.min(this.speed, this.maxSpeed));

        // Steering
        if (input.keys.left) {
            this.x -= this.steering * speedPercent * dt;
            this.steerDirection = -1;
        } else if (input.keys.right) {
            this.x += this.steering * speedPercent * dt;
            this.steerDirection = 1;
        } else {
            this.steerDirection = 0;
        }

        // Centrifugal force from curves
        this.x += (segment.curve * speedPercent * this.centrifugal * dt);

        // Clamp horizontal position
        this.x = Math.max(-2, Math.min(2, this.x));

        // Move along track
        this.z += this.speed * dt;

        // Wrap around track
        while (this.z >= Road.trackLength) {
            this.z -= Road.trackLength;
        }

        // Check collisions with roadside sprites
        this.checkSpriteCollisions(segment);

        // Check collisions with other cars
        this.checkCarCollisions(segment);
    },

    checkSpriteCollisions(segment) {
        for (let i = 0; i < segment.sprites.length; i++) {
            const sprite = segment.sprites[i];
            const spriteW = 0.15; // Collision width

            // Check if player overlaps with sprite
            if (Math.abs(this.x - sprite.offset) < spriteW + 0.2) {
                // Collision! Slow down significantly
                this.speed *= 0.3;

                // Push player away from sprite
                if (this.x < sprite.offset) {
                    this.x = sprite.offset - spriteW - 0.2;
                } else {
                    this.x = sprite.offset + spriteW + 0.2;
                }
            }
        }
    },

    checkCarCollisions(segment) {
        for (let i = 0; i < segment.cars.length; i++) {
            const car = segment.cars[i];
            const carW = 0.3;

            if (Math.abs(this.x - car.offset) < carW) {
                // Collision with other car
                this.speed *= 0.5;

                // Bounce off
                if (this.x < car.offset) {
                    this.x -= 0.5;
                } else {
                    this.x += 0.5;
                }
            }
        }
    },

    render(ctx) {
        const sprite = Sprites.cache['player'];
        if (!sprite) return;

        // Car position at bottom center of screen
        const x = Render.width / 2;
        const y = Render.height - 20;

        // Bounce effect based on speed
        const bounce = Math.sin(Date.now() * 0.02) * (this.speed / this.maxSpeed) * 2;

        // Tilt based on steering
        ctx.save();
        ctx.translate(x, y + bounce);

        // Slight rotation for steering effect
        const tilt = this.steerDirection * 0.1;
        ctx.rotate(tilt);

        // Draw car
        const scale = 1.5;
        const w = sprite.width * scale;
        const h = sprite.height * scale;
        ctx.drawImage(sprite, -w / 2, -h, w, h);

        ctx.restore();
    },

    // Get speed in MPH for display
    getSpeedMPH() {
        return Math.round(this.speed / 100);
    }
};

// AI Traffic cars
const Traffic = {
    cars: [],
    maxCars: 10,
    colors: ['blue', 'green', 'yellow', 'purple', 'orange'],

    init() {
        this.cars = [];
        this.spawnCars();
    },

    spawnCars() {
        // Spawn cars along the track
        for (let i = 0; i < this.maxCars; i++) {
            this.cars.push({
                z: Math.random() * Road.trackLength,
                offset: -0.8 + Math.random() * 1.6, // Random lane
                speed: 5000 + Math.random() * 5000,  // Random speed
                color: this.colors[Math.floor(Math.random() * this.colors.length)]
            });
        }
    },

    update(dt, playerZ) {
        // Clear cars from all segments
        for (let i = 0; i < Road.segments.length; i++) {
            Road.segments[i].cars = [];
        }

        // Update each car and add to appropriate segment
        for (let i = 0; i < this.cars.length; i++) {
            const car = this.cars[i];

            // Move car
            car.z += car.speed * dt;

            // Wrap around
            if (car.z >= Road.trackLength) {
                car.z -= Road.trackLength;
            }

            // Add to road segment for rendering
            const segment = Road.getSegment(car.z);
            segment.cars.push(car);
        }
    },

    reset() {
        this.cars = [];
        this.spawnCars();
    }
};
