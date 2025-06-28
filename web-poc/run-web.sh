#!/bin/bash
set -e

echo "🚀 Building and running Mathypad Web POC..."
echo

# Build the WASM version
echo "📦 Building WASM..."
./build-web.sh

echo
echo "🌐 Starting web server..."

# Kill any existing server on port 8080
pkill -f "python3 -m http.server 8080" 2>/dev/null || true

# Start the web server in the background
python3 -m http.server 8080 --directory . > /dev/null 2>&1 &
SERVER_PID=$!

# Wait a moment for server to start
sleep 2

# Check if server started successfully
if ! curl -s http://localhost:8080 > /dev/null; then
    echo "❌ Failed to start web server"
    exit 1
fi

echo "✅ Web server started on http://localhost:8080"
echo

# Try to open the webpage in the default browser
if command -v open &> /dev/null; then
    # macOS
    echo "🌐 Opening http://localhost:8080 in your browser..."
    open http://localhost:8080
elif command -v xdg-open &> /dev/null; then
    # Linux
    echo "🌐 Opening http://localhost:8080 in your browser..."
    xdg-open http://localhost:8080
elif command -v start &> /dev/null; then
    # Windows
    echo "🌐 Opening http://localhost:8080 in your browser..."
    start http://localhost:8080
else
    echo "🌐 Please open http://localhost:8080 in your browser"
fi

echo
echo "📝 Server is running in the background (PID: $SERVER_PID)"
echo "   To stop the server: kill $SERVER_PID"
echo "   Or press Ctrl+C if running in foreground"
echo
echo "🔧 To see server logs: tail -f server.log"
echo "💡 To rebuild only: ./build-web.sh"

# If running interactively, wait for Ctrl+C to stop the server
if [[ -t 0 ]]; then
    echo
    echo "Press Ctrl+C to stop the server..."
    trap "echo '🛑 Stopping server...'; kill $SERVER_PID 2>/dev/null || true; exit 0" INT
    wait $SERVER_PID
fi