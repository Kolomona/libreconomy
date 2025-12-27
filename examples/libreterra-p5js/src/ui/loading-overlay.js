// Loading overlay for terrain generation and app initialization
// Shows progress bar with percentage or indeterminate spinner

class LoadingOverlay {
  constructor() {
    this.visible = false;
    this.message = '';
    this.progress = -1;  // -1 = indeterminate, 0-100 = percentage
  }

  show(message, progress = -1) {
    this.visible = true;
    this.message = message;
    this.progress = progress;
  }

  hide() {
    this.visible = false;
  }

  render(p5Instance) {
    if (!this.visible) return;

    const p = p5Instance;

    // Dark semi-transparent background
    p.push();
    p.fill(0, 0, 0, 180);
    p.noStroke();
    p.rect(0, 0, p.width, p.height);

    // Center position
    const centerX = p.width / 2;
    const centerY = p.height / 2;

    // White container box
    p.fill(40, 40, 40);
    p.stroke(100, 100, 100);
    p.strokeWeight(2);
    p.rectMode(p.CENTER);
    p.rect(centerX, centerY, 400, 150, 10);

    // Message text
    p.fill(255);
    p.noStroke();
    p.textAlign(p.CENTER, p.CENTER);
    p.textSize(16);
    p.text(this.message, centerX, centerY - 30);

    // Progress bar (if percentage mode)
    if (this.progress >= 0) {
      const barWidth = 360;
      const barHeight = 30;
      const barX = centerX - barWidth / 2;
      const barY = centerY + 10;

      // Switch to CORNER mode for progress bar
      p.rectMode(p.CORNER);

      // Background bar
      p.fill(60, 60, 60);
      p.noStroke();
      p.rect(barX, barY, barWidth, barHeight, 5);

      // Filled portion
      const fillWidth = (this.progress / 100) * barWidth;
      p.fill(76, 175, 80);  // Green
      p.rect(barX, barY, fillWidth, barHeight, 5);

      // Border
      p.noFill();
      p.stroke(100, 100, 100);
      p.strokeWeight(1);
      p.rect(barX, barY, barWidth, barHeight, 5);

      // Percentage text
      p.fill(255);
      p.noStroke();
      p.textSize(14);
      p.text(`${Math.round(this.progress)}%`, centerX, barY + barHeight / 2);
    } else {
      // Indeterminate spinner
      p.push();
      p.translate(centerX, centerY + 20);
      p.rotate(p.frameCount * 0.05);
      p.noFill();
      p.stroke(76, 175, 80);
      p.strokeWeight(4);
      p.arc(0, 0, 40, 40, 0, p.PI * 1.5);
      p.pop();
    }

    p.pop();
  }
}

console.log('✓ loading-overlay.js loaded successfully');
