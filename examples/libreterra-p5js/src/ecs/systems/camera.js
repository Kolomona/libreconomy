// Camera system for panning, zooming, and transforming coordinates

class CameraSystem {
  constructor() {
    this.x = CONFIG.WORLD_WIDTH / 2;
    this.y = CONFIG.WORLD_HEIGHT / 2;
    this.zoom = CONFIG.CAMERA.INITIAL_ZOOM;

    // Pan state
    this.isDragging = false;
    this.dragStartX = 0;
    this.dragStartY = 0;
    this.dragStartCamX = 0;
    this.dragStartCamY = 0;

    // Double-click detection
    this.lastClickTime = 0;
    this.doubleClickDelay = 300; // ms
  }

  // Handle mouse pressed
  handleMousePressed() {
    this.isDragging = true;
    this.dragStartX = mouseX;
    this.dragStartY = mouseY;
    this.dragStartCamX = this.x;
    this.dragStartCamY = this.y;

    // Check for double-click
    const currentTime = millis();
    if (currentTime - this.lastClickTime < this.doubleClickDelay) {
      this.centerOn(this.x, this.y);
    }
    this.lastClickTime = currentTime;
  }

  // Handle mouse dragged
  handleMouseDragged() {
    if (this.isDragging) {
      const dx = (mouseX - this.dragStartX) / this.zoom;
      const dy = (mouseY - this.dragStartY) / this.zoom;

      this.x = this.dragStartCamX - dx;
      this.y = this.dragStartCamY - dy;

      // Clamp to world bounds
      this.clampToBounds();
    }
  }

  // Handle mouse released
  handleMouseReleased() {
    this.isDragging = false;
  }

  // Handle mouse wheel for zooming
  handleMouseWheel(event) {
    const zoomFactor = 1 + (CONFIG.CAMERA.ZOOM_SPEED * Math.sign(-event.delta));
    const newZoom = this.zoom * zoomFactor;

    // Clamp zoom
    if (newZoom >= CONFIG.CAMERA.MIN_ZOOM && newZoom <= CONFIG.CAMERA.MAX_ZOOM) {
      // Zoom toward mouse position
      const worldX = this.screenToWorldX(mouseX);
      const worldY = this.screenToWorldY(mouseY);

      this.zoom = newZoom;

      // Adjust camera position to zoom toward mouse
      const newWorldX = this.screenToWorldX(mouseX);
      const newWorldY = this.screenToWorldY(mouseY);

      this.x += (worldX - newWorldX);
      this.y += (worldY - newWorldY);

      this.clampToBounds();
    }

    // Prevent default scrolling
    return false;
  }

  // Center camera on a world position
  centerOn(worldX, worldY) {
    this.x = worldX;
    this.y = worldY;
    this.clampToBounds();
  }

  // Clamp camera to world bounds
  clampToBounds() {
    const halfScreenWidth = (width / 2) / this.zoom;
    const halfScreenHeight = (height / 2) / this.zoom;

    this.x = constrain(this.x, halfScreenWidth, CONFIG.WORLD_WIDTH - halfScreenWidth);
    this.y = constrain(this.y, halfScreenHeight, CONFIG.WORLD_HEIGHT - halfScreenHeight);
  }

  // Apply camera transform to p5.js canvas
  apply() {
    push();
    translate(width / 2, height / 2);
    scale(this.zoom);
    translate(-this.x, -this.y);
  }

  // Reset camera transform
  reset() {
    pop();
  }

  // Convert screen coordinates to world coordinates
  screenToWorldX(screenX) {
    return this.x + (screenX - width / 2) / this.zoom;
  }

  screenToWorldY(screenY) {
    return this.y + (screenY - height / 2) / this.zoom;
  }

  // Convert world coordinates to screen coordinates
  worldToScreenX(worldX) {
    return (worldX - this.x) * this.zoom + width / 2;
  }

  worldToScreenY(worldY) {
    return (worldY - this.y) * this.zoom + height / 2;
  }

  // Get visible bounds in world coordinates
  getVisibleBounds() {
    const halfWidth = (width / 2) / this.zoom;
    const halfHeight = (height / 2) / this.zoom;

    return {
      minX: Math.max(0, this.x - halfWidth),
      maxX: Math.min(CONFIG.WORLD_WIDTH, this.x + halfWidth),
      minY: Math.max(0, this.y - halfHeight),
      maxY: Math.min(CONFIG.WORLD_HEIGHT, this.y + halfHeight),
    };
  }

  // Update UI overlay
  updateUI() {
    document.getElementById('cam-x').textContent = Math.round(this.x);
    document.getElementById('cam-y').textContent = Math.round(this.y);
    document.getElementById('cam-zoom').textContent = this.zoom.toFixed(2);
  }
}
