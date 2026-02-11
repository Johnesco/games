// Sprite generation and rendering module
const Sprites = {
    cache: {},

    init() {
        // Generate all sprites programmatically (pixel art style)
        this.generateTree();
        this.generateBush();
        this.generatePost();
        this.generateBillboard();
        this.generatePlayerCar();
        this.generateAICars();
    },

    generateTree() {
        const canvas = document.createElement('canvas');
        canvas.width = 64;
        canvas.height = 128;
        const ctx = canvas.getContext('2d');

        // Tree trunk
        ctx.fillStyle = '#4a2c0a';
        ctx.fillRect(26, 80, 12, 48);

        // Tree foliage (layered circles for pine tree look)
        ctx.fillStyle = '#0a5f0a';
        ctx.beginPath();
        ctx.moveTo(32, 0);
        ctx.lineTo(8, 80);
        ctx.lineTo(56, 80);
        ctx.closePath();
        ctx.fill();

        // Darker shade
        ctx.fillStyle = '#084408';
        ctx.beginPath();
        ctx.moveTo(32, 0);
        ctx.lineTo(8, 80);
        ctx.lineTo(32, 70);
        ctx.closePath();
        ctx.fill();

        this.cache['tree'] = canvas;
    },

    generateBush() {
        const canvas = document.createElement('canvas');
        canvas.width = 48;
        canvas.height = 32;
        const ctx = canvas.getContext('2d');

        // Bush body
        ctx.fillStyle = '#228b22';
        ctx.beginPath();
        ctx.ellipse(24, 20, 22, 14, 0, 0, Math.PI * 2);
        ctx.fill();

        // Highlights
        ctx.fillStyle = '#32cd32';
        ctx.beginPath();
        ctx.ellipse(18, 16, 8, 6, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.beginPath();
        ctx.ellipse(32, 18, 6, 5, 0, 0, Math.PI * 2);
        ctx.fill();

        this.cache['bush'] = canvas;
    },

    generatePost() {
        const canvas = document.createElement('canvas');
        canvas.width = 16;
        canvas.height = 64;
        const ctx = canvas.getContext('2d');

        // Post body
        ctx.fillStyle = '#fff';
        ctx.fillRect(5, 0, 6, 64);

        // Red stripes
        ctx.fillStyle = '#ff0000';
        for (let y = 0; y < 64; y += 16) {
            ctx.fillRect(5, y, 6, 8);
        }

        this.cache['post'] = canvas;
    },

    generateBillboard() {
        const canvas = document.createElement('canvas');
        canvas.width = 128;
        canvas.height = 96;
        const ctx = canvas.getContext('2d');

        // Posts
        ctx.fillStyle = '#444';
        ctx.fillRect(10, 48, 8, 48);
        ctx.fillRect(110, 48, 8, 48);

        // Billboard frame
        ctx.fillStyle = '#333';
        ctx.fillRect(0, 0, 128, 56);

        // Billboard content area
        ctx.fillStyle = '#fff';
        ctx.fillRect(4, 4, 120, 48);

        // Text
        ctx.fillStyle = '#ff0000';
        ctx.font = 'bold 20px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('RACE!', 64, 32);

        this.cache['billboard'] = canvas;
    },

    generatePlayerCar() {
        const canvas = document.createElement('canvas');
        canvas.width = 80;
        canvas.height = 48;
        const ctx = canvas.getContext('2d');

        // Car body (red sports car from above/behind)
        ctx.fillStyle = '#cc0000';
        ctx.fillRect(8, 8, 64, 32);

        // Cockpit
        ctx.fillStyle = '#222';
        ctx.fillRect(20, 12, 40, 16);

        // Windshield reflection
        ctx.fillStyle = '#6688cc';
        ctx.fillRect(22, 14, 36, 8);

        // Wheels
        ctx.fillStyle = '#111';
        ctx.fillRect(2, 8, 10, 12);
        ctx.fillRect(68, 8, 10, 12);
        ctx.fillRect(2, 28, 10, 12);
        ctx.fillRect(68, 28, 10, 12);

        // Rear lights
        ctx.fillStyle = '#ff4444';
        ctx.fillRect(8, 36, 12, 4);
        ctx.fillRect(60, 36, 12, 4);

        // Spoiler
        ctx.fillStyle = '#990000';
        ctx.fillRect(10, 40, 60, 6);

        // White stripes
        ctx.fillStyle = '#fff';
        ctx.fillRect(38, 8, 4, 32);

        this.cache['player'] = canvas;
    },

    generateAICars() {
        const colors = [
            { name: 'blue', main: '#0066cc', dark: '#004499' },
            { name: 'green', main: '#00aa00', dark: '#007700' },
            { name: 'yellow', main: '#ccaa00', dark: '#997700' },
            { name: 'purple', main: '#9900cc', dark: '#660099' },
            { name: 'orange', main: '#ff6600', dark: '#cc4400' }
        ];

        colors.forEach(color => {
            const canvas = document.createElement('canvas');
            canvas.width = 80;
            canvas.height = 48;
            const ctx = canvas.getContext('2d');

            // Car body
            ctx.fillStyle = color.main;
            ctx.fillRect(8, 8, 64, 32);

            // Cockpit
            ctx.fillStyle = '#222';
            ctx.fillRect(20, 12, 40, 16);

            // Windshield
            ctx.fillStyle = '#6688cc';
            ctx.fillRect(22, 14, 36, 8);

            // Wheels
            ctx.fillStyle = '#111';
            ctx.fillRect(2, 8, 10, 12);
            ctx.fillRect(68, 8, 10, 12);
            ctx.fillRect(2, 28, 10, 12);
            ctx.fillRect(68, 28, 10, 12);

            // Rear section
            ctx.fillStyle = color.dark;
            ctx.fillRect(10, 36, 60, 10);

            this.cache['car_' + color.name] = canvas;
        });
    },

    render(ctx, type, x, y, scale) {
        const sprite = this.cache[type];
        if (!sprite) return;

        const w = sprite.width * scale * 0.5;
        const h = sprite.height * scale * 0.5;

        // Don't render if too small or off-screen
        if (w < 2 || y < 0 || x < -w || x > Render.width + w) return;

        ctx.drawImage(sprite, x - w / 2, y - h, w, h);
    },

    // Get sprite dimensions for collision detection
    getSize(type) {
        const sprite = this.cache[type];
        if (!sprite) return { width: 0, height: 0 };
        return { width: sprite.width, height: sprite.height };
    }
};
