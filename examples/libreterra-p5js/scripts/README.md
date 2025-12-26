# libreterra-p5js Server Scripts

This directory contains scripts for starting a development server to run the libreterra-p5js simulation.

## Quick Start

### Option 1: Bash Script (Recommended)

Uses Python's built-in HTTP server:

```bash
./scripts/start-server.sh
```

To use a different port:

```bash
PORT=3000 ./scripts/start-server.sh
```

### Option 2: Node.js Server

If you have Node.js installed:

```bash
node scripts/server.js
```

Or with a custom port:

```bash
node scripts/server.js 3000
```

### Option 3: NPX (No Installation Required)

If you have Node.js/npm:

```bash
npx http-server -p 8000
```

### Option 4: Python Directly

```bash
python3 -m http.server 8000
```

## Accessing the Simulation

Once the server is running, open your browser to:

```
http://localhost:8000
```

## Notes

- The server must be run from the `examples/libreterra-p5js` directory or use these scripts which handle path resolution
- WASM files require proper MIME types, which all these server options provide
- Press Ctrl+C to stop the server
