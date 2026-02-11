// Parallax background rendering module
const Background = {
    layers: [],

    init() {
        // Create background layers
        this.createSky();
        this.createMountains();
        this.createHills();
    },

    createSky() {
        const canvas = document.createElement('canvas');
        canvas.width = 640;
        canvas.height = 240;
        const ctx = canvas.getContext('2d');

        // Sky gradient
        const gradient = ctx.createLinearGradient(0, 0, 0, 240);
        gradient.addColorStop(0, '#1a1a4a');
        gradient.addColorStop(0.4, '#4a4a8a');
        gradient.addColorStop(0.7, '#8a6aa0');
        gradient.addColorStop(1, '#ffaa66');
        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, 640, 240);

        // Sun
        ctx.fillStyle = '#ffdd44';
        ctx.beginPath();
        ctx.arc(500, 180, 40, 0, Math.PI * 2);
        ctx.fill();

        // Sun glow
        ctx.fillStyle = 'rgba(255, 220, 100, 0.3)';
        ctx.beginPath();
        ctx.arc(500, 180, 60, 0, Math.PI * 2);
        ctx.fill();

        // Clouds
        ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
        this.drawCloud(ctx, 80, 60, 1);
        this.drawCloud(ctx, 250, 90, 0.8);
        this.drawCloud(ctx, 450, 50, 1.2);
        this.drawCloud(ctx, 580, 100, 0.7);

        this.layers.push({ canvas, speed: 0.001, y: 0 });
    },

    drawCloud(ctx, x, y, scale) {
        ctx.beginPath();
        ctx.ellipse(x, y, 30 * scale, 15 * scale, 0, 0, Math.PI * 2);
        ctx.ellipse(x + 25 * scale, y + 5, 25 * scale, 12 * scale, 0, 0, Math.PI * 2);
        ctx.ellipse(x - 20 * scale, y + 3, 20 * scale, 10 * scale, 0, 0, Math.PI * 2);
        ctx.fill();
    },

    createMountains() {
        const canvas = document.createElement('canvas');
        canvas.width = 1280;
        canvas.height = 120;
        const ctx = canvas.getContext('2d');

        // Far mountains (darker)
        ctx.fillStyle = '#2a3a4a';
        ctx.beginPath();
        ctx.moveTo(0, 120);
        for (let x = 0; x <= 1280; x += 80) {
            const height = 40 + Math.sin(x * 0.01) * 30 + Math.random() * 20;
            ctx.lineTo(x, 120 - height);
        }
        ctx.lineTo(1280, 120);
        ctx.closePath();
        ctx.fill();

        // Near mountains (lighter)
        ctx.fillStyle = '#3a4a5a';
        ctx.beginPath();
        ctx.moveTo(0, 120);
        for (let x = 0; x <= 1280; x += 60) {
            const height = 30 + Math.sin(x * 0.015 + 1) * 25 + Math.random() * 15;
            ctx.lineTo(x, 120 - height);
        }
        ctx.lineTo(1280, 120);
        ctx.closePath();
        ctx.fill();

        this.layers.push({ canvas, speed: 0.002, y: 120 });
    },

    createHills() {
        const canvas = document.createElement('canvas');
        canvas.width = 1280;
        canvas.height = 60;
        const ctx = canvas.getContext('2d');

        // Rolling hills
        ctx.fillStyle = '#1a6a1a';
        ctx.beginPath();
        ctx.moveTo(0, 60);
        for (let x = 0; x <= 1280; x += 40) {
            const height = 20 + Math.sin(x * 0.02) * 15 + Math.random() * 10;
            ctx.lineTo(x, 60 - height);
        }
        ctx.lineTo(1280, 60);
        ctx.closePath();
        ctx.fill();

        // Lighter patches
        ctx.fillStyle = '#2a8a2a';
        ctx.beginPath();
        ctx.moveTo(0, 60);
        for (let x = 0; x <= 1280; x += 50) {
            const height = 10 + Math.sin(x * 0.025 + 2) * 10;
            ctx.lineTo(x, 60 - height);
        }
        ctx.lineTo(1280, 60);
        ctx.closePath();
        ctx.fill();

        this.layers.push({ canvas, speed: 0.005, y: 180 });
    },

    render(ctx, playerX, speed) {
        // Render each layer with parallax scrolling
        for (let i = 0; i < this.layers.length; i++) {
            const layer = this.layers[i];

            // Calculate horizontal offset based on player position and layer speed
            const offset = (playerX * layer.speed * 1000) % layer.canvas.width;

            // Draw the layer twice for seamless scrolling
            ctx.drawImage(layer.canvas, -offset, layer.y);
            ctx.drawImage(layer.canvas, layer.canvas.width - offset, layer.y);

            // Handle negative offset
            if (offset < 0) {
                ctx.drawImage(layer.canvas, -layer.canvas.width - offset, layer.y);
            }
        }
    }
};
