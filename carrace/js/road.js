// Road generation and rendering module
const Road = {
    segments: [],
    segmentLength: 200,      // Length of each road segment
    roadWidth: 2000,         // Half-width of the road
    rumbleLength: 3,         // Segments per rumble strip
    trackLength: null,       // Total track length (set after building)
    lanes: 3,

    // Colors for road rendering
    colors: {
        sky: '#72d7ee',
        tree: '#005108',
        fog: '#005108',
        light: {
            road: '#6b6b6b',
            grass: '#10aa10',
            rumble: '#fff',
            lane: '#ccc'
        },
        dark: {
            road: '#696969',
            grass: '#009a00',
            rumble: '#bb0000',
            lane: '#696969'
        }
    },

    // Curve and hill intensity constants
    CURVE: {
        NONE: 0,
        EASY: 2,
        MEDIUM: 4,
        HARD: 6
    },

    HILL: {
        NONE: 0,
        LOW: 20,
        MEDIUM: 40,
        HIGH: 60
    },

    init() {
        this.segments = [];
        this.buildTrack();
        this.trackLength = this.segments.length * this.segmentLength;
    },

    buildTrack() {
        // Build a varied track with curves and hills

        // Starting straight
        this.addStraight(50);

        // First set of curves
        this.addCurve(25, this.CURVE.EASY, this.HILL.NONE);
        this.addHill(25, this.HILL.LOW);
        this.addCurve(25, this.CURVE.MEDIUM, this.HILL.NONE);

        // Long straight with hills
        this.addStraight(25);
        this.addHill(50, this.HILL.MEDIUM);
        this.addDownhill(30, this.HILL.LOW);

        // S-curves
        this.addCurve(30, -this.CURVE.HARD, this.HILL.NONE);
        this.addCurve(30, this.CURVE.HARD, this.HILL.NONE);

        // Big hill section
        this.addHill(40, this.HILL.HIGH);
        this.addCurve(25, this.CURVE.MEDIUM, this.HILL.LOW);
        this.addDownhill(30, this.HILL.MEDIUM);

        // Final stretch
        this.addCurve(25, -this.CURVE.EASY, this.HILL.NONE);
        this.addStraight(50);

        // Add sprites to segments
        this.addSprites();

        // Mark start/finish
        this.segments[0].color = { ...this.colors.dark, rumble: '#fff', lane: '#fff' };
    },

    addSegment(curve, y) {
        const n = this.segments.length;
        this.segments.push({
            index: n,
            point: {
                world: { x: 0, y: y, z: n * this.segmentLength },
                camera: { x: 0, y: 0, z: 0 },
                screen: { x: 0, y: 0, w: 0, scale: 0 }
            },
            curve: curve,
            sprites: [],
            cars: [],
            color: Math.floor(n / this.rumbleLength) % 2 ? this.colors.dark : this.colors.light
        });
    },

    addStraight(num) {
        for (let i = 0; i < num; i++) {
            this.addSegment(0, this.getLastY());
        }
    },

    addCurve(num, curve, hill) {
        for (let i = 0; i < num; i++) {
            this.addSegment(curve, this.getLastY() + hill);
        }
    },

    addHill(num, height) {
        for (let i = 0; i < num; i++) {
            this.addSegment(0, this.getLastY() + height);
        }
    },

    addDownhill(num, height) {
        for (let i = 0; i < num; i++) {
            this.addSegment(0, this.getLastY() - height);
        }
    },

    getLastY() {
        return this.segments.length > 0 ? this.segments[this.segments.length - 1].point.world.y : 0;
    },

    addSprites() {
        // Add roadside sprites (trees, signs, etc.)
        for (let i = 0; i < this.segments.length; i++) {
            const segment = this.segments[i];

            // Add trees on both sides periodically
            if (i % 15 === 0) {
                segment.sprites.push({ type: 'tree', offset: -1.5 - Math.random() * 0.5 });
                segment.sprites.push({ type: 'tree', offset: 1.5 + Math.random() * 0.5 });
            }

            // Add bushes more frequently
            if (i % 8 === 0) {
                const side = Math.random() > 0.5 ? 1 : -1;
                segment.sprites.push({ type: 'bush', offset: side * (1.2 + Math.random() * 0.3) });
            }

            // Add posts along the road
            if (i % 25 === 0) {
                segment.sprites.push({ type: 'post', offset: -1.1 });
                segment.sprites.push({ type: 'post', offset: 1.1 });
            }

            // Occasional billboards
            if (i % 80 === 40) {
                segment.sprites.push({ type: 'billboard', offset: Math.random() > 0.5 ? -2 : 2 });
            }
        }
    },

    getSegment(z) {
        if (z < 0) z += this.trackLength;
        return this.segments[Math.floor(z / this.segmentLength) % this.segments.length];
    },

    render(ctx, player, camera) {
        const baseSegment = this.getSegment(camera.z);
        const baseIndex = baseSegment.index;
        let maxY = Render.height;
        let x = 0;
        let dx = -(baseSegment.curve * (camera.z % this.segmentLength) / this.segmentLength);

        // Draw from back to front
        for (let n = 0; n < 300; n++) {
            const index = (baseIndex + n) % this.segments.length;
            const segment = this.segments[index];
            const looped = index < baseIndex;
            const fog = this.exponentialFog(n, 100);

            // Project segment points
            const p1 = segment.point;
            const cameraZ = camera.z - (looped ? this.trackLength : 0);

            this.projectPoint(p1, camera.x - x, camera.y, cameraZ, camera.depth);
            x += dx;
            dx += segment.curve;

            // Skip if behind camera or above screen
            if (p1.camera.z <= camera.depth || p1.screen.y >= maxY) continue;

            // Get next segment for rendering
            const nextIndex = (index + 1) % this.segments.length;
            const nextSegment = this.segments[nextIndex];
            const p2 = nextSegment.point;
            const nextLooped = nextIndex < baseIndex;
            const nextCameraZ = camera.z - (nextLooped ? this.trackLength : 0);

            this.projectPoint(p2, camera.x - x - dx, camera.y, nextCameraZ, camera.depth);

            if (p2.screen.y >= p1.screen.y) continue;

            // Render road segment
            this.renderSegment(
                ctx,
                Render.width, this.lanes,
                p1.screen.x, p1.screen.y, p1.screen.w,
                p2.screen.x, p2.screen.y, p2.screen.w,
                segment.color,
                fog
            );

            maxY = p2.screen.y;
        }

        // Render sprites back to front
        for (let n = 299; n >= 0; n--) {
            const index = (baseIndex + n) % this.segments.length;
            const segment = this.segments[index];

            for (let i = 0; i < segment.sprites.length; i++) {
                const sprite = segment.sprites[i];
                const spriteScale = segment.point.screen.scale;
                const spriteX = segment.point.screen.x + (spriteScale * sprite.offset * this.roadWidth);
                const spriteY = segment.point.screen.y;

                Sprites.render(ctx, sprite.type, spriteX, spriteY, spriteScale);
            }

            // Render AI cars in this segment
            for (let i = 0; i < segment.cars.length; i++) {
                const car = segment.cars[i];
                const carScale = segment.point.screen.scale;
                const carX = segment.point.screen.x + (carScale * car.offset * this.roadWidth);
                const carY = segment.point.screen.y;

                Sprites.render(ctx, 'car_' + car.color, carX, carY, carScale * 1.2);
            }
        }
    },

    projectPoint(point, cameraX, cameraY, cameraZ, cameraDepth) {
        const transX = point.world.x - cameraX;
        const transY = point.world.y - cameraY;
        const transZ = point.world.z - cameraZ;

        point.camera = { x: transX, y: transY, z: transZ };

        if (transZ > 0) {
            point.screen.scale = cameraDepth / transZ;
            point.screen.x = Math.round(Render.width / 2 + point.camera.x * point.screen.scale);
            point.screen.y = Math.round(Render.height / 2 - point.camera.y * point.screen.scale);
            point.screen.w = Math.round(point.screen.scale * this.roadWidth);
        }
    },

    renderSegment(ctx, width, lanes, x1, y1, w1, x2, y2, w2, color, fog) {
        const r = Render;

        // Grass
        r.rect(0, y2, width, y1 - y2, this.fogColor(color.grass, fog));

        // Road
        r.polygon(
            x1 - w1, y1, x1 + w1, y1,
            x2 + w2, y2, x2 - w2, y2,
            this.fogColor(color.road, fog)
        );

        // Rumble strips
        const rumbleW1 = w1 / 5;
        const rumbleW2 = w2 / 5;
        r.polygon(
            x1 - w1 - rumbleW1, y1, x1 - w1, y1,
            x2 - w2, y2, x2 - w2 - rumbleW2, y2,
            this.fogColor(color.rumble, fog)
        );
        r.polygon(
            x1 + w1, y1, x1 + w1 + rumbleW1, y1,
            x2 + w2 + rumbleW2, y2, x2 + w2, y2,
            this.fogColor(color.rumble, fog)
        );

        // Lane markers
        if (color.lane) {
            const laneW1 = w1 / 30;
            const laneW2 = w2 / 30;
            const laneX1 = w1 * 2 / lanes;
            const laneX2 = w2 * 2 / lanes;

            for (let i = 1; i < lanes; i++) {
                const lane1 = x1 - w1 + laneX1 * i;
                const lane2 = x2 - w2 + laneX2 * i;
                r.polygon(
                    lane1 - laneW1, y1, lane1 + laneW1, y1,
                    lane2 + laneW2, y2, lane2 - laneW2, y2,
                    this.fogColor(color.lane, fog)
                );
            }
        }
    },

    exponentialFog(distance, density) {
        return 1 / Math.pow(Math.E, (distance / density));
    },

    fogColor(color, fog) {
        if (fog >= 1) return color;

        // Parse the color
        const r = parseInt(color.slice(1, 3), 16);
        const g = parseInt(color.slice(3, 5), 16);
        const b = parseInt(color.slice(5, 7), 16);

        // Fog color (greenish for trees)
        const fogR = 0;
        const fogG = 81;
        const fogB = 8;

        // Interpolate
        const newR = Math.round(r * fog + fogR * (1 - fog));
        const newG = Math.round(g * fog + fogG * (1 - fog));
        const newB = Math.round(b * fog + fogB * (1 - fog));

        return '#' +
            newR.toString(16).padStart(2, '0') +
            newG.toString(16).padStart(2, '0') +
            newB.toString(16).padStart(2, '0');
    }
};
