#!/bin/bash
set -e

echo "Building Mathypad Web POC..."

# Install wasm-bindgen-cli if not already installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Add wasm32 target if not already added
rustup target add wasm32-unknown-unknown

# Build the WASM binary (library only)
echo "Building WASM..."
RUSTFLAGS="-C panic=abort" cargo build --target wasm32-unknown-unknown --release --lib

# Generate JS bindings
echo "Generating bindings..."
wasm-bindgen ../target/wasm32-unknown-unknown/release/mathypad_web_poc.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

# Optimize WASM file size with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM for size..."
    # Use -Oz for maximum size optimization
    wasm-opt -Oz --enable-bulk-memory --enable-sign-ext pkg/mathypad_web_poc_bg.wasm -o pkg/mathypad_web_poc_bg.wasm
    echo "WASM optimization complete"
else
    echo "💡 Installing wasm-opt for smaller bundle sizes..."
    cargo install wasm-opt
    echo "Optimizing WASM for size..."
    wasm-opt -Oz --enable-bulk-memory --enable-sign-ext pkg/mathypad_web_poc_bg.wasm -o pkg/mathypad_web_poc_bg.wasm
    echo "WASM optimization complete"
fi

# Copy files to root directory to match production structure
echo "Copying files to root directory for local development..."
cp pkg/mathypad_web_poc.js .
cp pkg/mathypad_web_poc_bg.wasm .

# Show bundle sizes
echo ""
echo "📊 Bundle sizes:"
ls -lh mathypad_web_poc_bg.wasm mathypad_web_poc.js | awk '{print "   " $9 ": " $5}'
echo ""
echo "📦 Gzipped sizes (typical web serving):"
gzip -c mathypad_web_poc_bg.wasm | wc -c | awk '{printf "   WASM: %.0f KB\n", $1/1024}'
gzip -c mathypad_web_poc.js | wc -c | awk '{printf "   JS: %.0f KB\n", $1/1024}'
echo ""
echo "✅ Build complete! Open index.html in a web server to run."
echo ""
echo "You can use Python's built-in server:"
echo "  python3 -m http.server 8000"
echo "Then open http://localhost:8000 in your browser."