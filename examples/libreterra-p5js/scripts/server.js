#!/usr/bin/env node
// Simple HTTP server for libreterra-p5js development
// Usage: node server.js [port]

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || process.argv[2] || 8000;
const ROOT_DIR = path.join(__dirname, '..');

// MIME types for common file extensions
const MIME_TYPES = {
  '.html': 'text/html',
  '.css': 'text/css',
  '.js': 'application/javascript',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.wasm': 'application/wasm',
  '.txt': 'text/plain',
  '.md': 'text/markdown'
};

const server = http.createServer((req, res) => {
  // Get file path from URL
  let filePath = path.join(ROOT_DIR, req.url === '/' ? 'index.html' : req.url);

  // Security: prevent directory traversal
  if (!filePath.startsWith(ROOT_DIR)) {
    res.writeHead(403, { 'Content-Type': 'text/plain' });
    res.end('403 Forbidden');
    return;
  }

  // Get file extension for MIME type
  const ext = path.extname(filePath).toLowerCase();
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';

  // Read and serve the file
  fs.readFile(filePath, (err, data) => {
    if (err) {
      if (err.code === 'ENOENT') {
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end('404 Not Found');
      } else {
        res.writeHead(500, { 'Content-Type': 'text/plain' });
        res.end('500 Internal Server Error');
      }
      console.error(`Error serving ${req.url}:`, err.message);
    } else {
      res.writeHead(200, {
        'Content-Type': contentType,
        'Cross-Origin-Opener-Policy': 'same-origin',
        'Cross-Origin-Embedder-Policy': 'require-corp'
      });
      res.end(data);
      console.log(`[${new Date().toISOString()}] ${req.method} ${req.url} - 200`);
    }
  });
});

server.listen(PORT, () => {
  console.log('════════════════════════════════════════════════════════════');
  console.log('  libreterra-p5js Development Server (Node.js)');
  console.log('════════════════════════════════════════════════════════════');
  console.log('');
  console.log(`  Server running at http://localhost:${PORT}`);
  console.log('  Press Ctrl+C to stop');
  console.log('');
  console.log('════════════════════════════════════════════════════════════');
  console.log('');
});

// Handle server errors
server.on('error', (err) => {
  if (err.code === 'EADDRINUSE') {
    console.error(`\nERROR: Port ${PORT} is already in use.`);
    console.error(`Try a different port: node server.js <port>`);
  } else {
    console.error('\nServer error:', err);
  }
  process.exit(1);
});

// Graceful shutdown
process.on('SIGINT', () => {
  console.log('\n\nShutting down server...');
  server.close(() => {
    console.log('Server stopped.');
    process.exit(0);
  });
});
