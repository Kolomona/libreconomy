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

    // Camera following state
    this.followEntityId = null;        // Entity to follow (null = no following)
    this.followSpeed = 0.08;           // Lerp speed (0-1, lower = smoother)
    this.followDeadzone = 30;          // Pixels before camera starts moving
    this.followLookAhead = 50;         // Predict entity position based on velocity
    this.smoothPosition = {            // Smoothed target position
      x: this.x,
      y: this.y
    };
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

      // Stop following when user manually pans
      if (this.isFollowing()) {
        this.stopFollowing();
        console.log('Camera following stopped (manual pan)');
      }
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

  // Update camera following (call before apply() in draw loop)
  update(ecsWorld) {
    // Early exit if not following anything
    if (this.followEntityId === null) {
      return;
    }

    // Check if followed entity still exists
    const entities = allEntitiesQuery(ecsWorld);
    if (!entities.includes(this.followEntityId)) {
      this.followEntityId = null;  // Stop following if entity disappeared
      return;
    }

    // Get entity position
    const targetX = Position.x[this.followEntityId];
    const targetY = Position.y[this.followEntityId];

    // Validate position (handle NaN or undefined)
    if (isNaN(targetX) || isNaN(targetY) || targetX === undefined || targetY === undefined) {
      return;  // Don't move camera if entity has invalid position
    }

    // Calculate look-ahead offset based on entity velocity (if moving)
    let offsetX = 0;
    let offsetY = 0;

    if (Velocity && this.followLookAhead > 0) {
      const vx = Velocity.vx[this.followEntityId] || 0;
      const vy = Velocity.vy[this.followEntityId] || 0;
      const speed = Math.sqrt(vx * vx + vy * vy);

      if (speed > 0.1) {  // Only apply look-ahead if entity is moving
        offsetX = (vx / speed) * this.followLookAhead;
        offsetY = (vy / speed) * this.followLookAhead;
      }
    }

    // Target position with look-ahead
    const desiredX = targetX + offsetX;
    const desiredY = targetY + offsetY;

    // Smooth target position (first-order filter)
    const alpha = 0.15;  // Smoothing for target (higher = snappier)
    this.smoothPosition.x += (desiredX - this.smoothPosition.x) * alpha;
    this.smoothPosition.y += (desiredY - this.smoothPosition.y) * alpha;

    // Calculate distance from camera to smooth target
    const dx = this.smoothPosition.x - this.x;
    const dy = this.smoothPosition.y - this.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    // Apply deadzone - only move if outside deadzone radius
    if (distance > this.followDeadzone) {
      // Lerp camera toward smooth target
      this.x += dx * this.followSpeed;
      this.y += dy * this.followSpeed;

      // Clamp camera to world bounds
      this.clampToBounds();
    }
  }

  // Start following an entity
  startFollowing(entityId) {
    this.followEntityId = entityId;

    // Initialize smooth position to entity position (prevents initial jump)
    if (entityId !== null) {
      const x = Position.x[entityId];
      const y = Position.y[entityId];
      if (!isNaN(x) && !isNaN(y)) {
        this.smoothPosition.x = x;
        this.smoothPosition.y = y;
      }
    }
  }

  // Stop following
  stopFollowing() {
    this.followEntityId = null;
  }

  // Check if currently following an entity
  isFollowing() {
    return this.followEntityId !== null;
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
    // Round camera position to eliminate sub-pixel rendering artifacts
    // This keeps smooth camera movement (stored as floats) while ensuring
    // pixel-perfect rendering (rendered at integer coordinates)
    translate(-Math.round(this.x), -Math.round(this.y));
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
