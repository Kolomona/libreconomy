#!/bin/bash
# Start development server for libreterra-p5js

# Change to the libreterra-p5js directory
cd "$(dirname "$0")/.." || exit 1

PORT="${PORT:-8001}"

echo "════════════════════════════════════════════════════════════"
echo "  libreterra-p5js Development Server"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if port is in use and kill the process
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "  Port $PORT is in use. Killing existing process..."
    PID=$(lsof -Pi :$PORT -sTCP:LISTEN -t)
    kill -9 $PID 2>/dev/null
    sleep 1
    echo "  ✓ Process killed (PID: $PID)"
    echo ""
fi

echo "  Starting server on http://localhost:$PORT"
echo "  Press Ctrl+C to stop"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if Python 3 is available
if command -v python3 &> /dev/null; then
    echo "Using Python 3 http.server..."
    python3 -m http.server "$PORT"
elif command -v python &> /dev/null; then
    echo "Using Python http.server..."
    python -m http.server "$PORT"
else
    echo "ERROR: Python not found. Please install Python 3 to run the server."
    echo "Alternatively, you can use any other HTTP server like:"
    echo "  - npx http-server -p $PORT"
    echo "  - php -S localhost:$PORT"
    exit 1
fi
