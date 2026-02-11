// Rendering utilities module
const Render = {
    ctx: null,
    width: 640,
    height: 480,

    init(canvas) {
        this.ctx = canvas.getContext('2d');
        canvas.width = this.width;
        canvas.height = this.height;
    },

    clear() {
        this.ctx.clearRect(0, 0, this.width, this.height);
    },

    // Draw a filled polygon (used for road segments)
    polygon(x1, y1, x2, y2, x3, y3, x4, y4, color) {
        this.ctx.fillStyle = color;
        this.ctx.beginPath();
        this.ctx.moveTo(x1, y1);
        this.ctx.lineTo(x2, y2);
        this.ctx.lineTo(x3, y3);
        this.ctx.lineTo(x4, y4);
        this.ctx.closePath();
        this.ctx.fill();
    },

    // Draw a rectangle
    rect(x, y, width, height, color) {
        this.ctx.fillStyle = color;
        this.ctx.fillRect(x, y, width, height);
    },

    // Draw a sprite (image or generated)
    sprite(sprite, x, y, width, height) {
        if (sprite instanceof HTMLImageElement || sprite instanceof HTMLCanvasElement) {
            this.ctx.drawImage(sprite, x - width / 2, y - height, width, height);
        }
    },

    // Draw text
    text(text, x, y, color = '#fff', size = 16, align = 'center') {
        this.ctx.fillStyle = color;
        this.ctx.font = `${size}px "Courier New", monospace`;
        this.ctx.textAlign = align;
        this.ctx.fillText(text, x, y);
    },

    // Project 3D coordinates to 2D screen
    project(point, cameraX, cameraY, cameraZ, cameraDepth) {
        const transX = point.world.x - cameraX;
        const transY = point.world.y - cameraY;
        const transZ = point.world.z - cameraZ;

        point.camera = {
            x: transX,
            y: transY,
            z: transZ
        };

        point.screen = {
            scale: cameraDepth / transZ,
            x: Math.round(this.width / 2 + point.camera.x * point.screen?.scale || 0),
            y: Math.round(this.height / 2 - point.camera.y * point.screen?.scale || 0),
            w: Math.round(point.screen?.scale || 0)
        };

        point.screen.scale = cameraDepth / transZ;
        point.screen.x = Math.round(this.width / 2 + point.camera.x * point.screen.scale);
        point.screen.y = Math.round(this.height / 2 - point.camera.y * point.screen.scale);
        point.screen.w = Math.round(point.screen.scale * Road.roadWidth);
    }
};
