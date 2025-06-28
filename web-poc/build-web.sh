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
cargo build --target wasm32-unknown-unknown --release --lib

# Generate JS bindings
echo "Generating bindings..."
wasm-bindgen ../target/wasm32-unknown-unknown/release/mathypad_web_poc.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

# Optional: Optimize WASM file size with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM..."
    wasm-opt -O2 pkg/mathypad_web_poc_bg.wasm -o pkg/mathypad_web_poc_bg.wasm
fi

# Copy files to root directory to match production structure
echo "Copying files to root directory for local development..."
cp pkg/mathypad_web_poc.js .
cp pkg/mathypad_web_poc_bg.wasm .

echo "Build complete! Open index.html in a web server to run."
echo ""
echo "You can use Python's built-in server:"
echo "  python3 -m http.server 8000"
echo "Then open http://localhost:8000 in your browser."