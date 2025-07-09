#!/bin/bash
set -e

echo "🚀 Building Mathypad Web GUI..."
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src/gui" ]; then
    echo "❌ Error: Please run this script from the mathypad root directory"
    echo "   Expected to find: Cargo.toml and src/gui/ directory"
    exit 1
fi

# Install required tools if not present
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Add wasm32 target if not already added
rustup target add wasm32-unknown-unknown

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf pkg/
mkdir -p pkg/

# Build the WASM binary with GUI feature enabled
echo "🔨 Building WASM with GUI feature..."
if ! RUSTFLAGS="-C panic=abort" cargo build --target wasm32-unknown-unknown --release --lib --features gui; then
    echo "❌ WASM build failed!"
    exit 1
fi

# Verify WASM file exists
WASM_FILE="target/wasm32-unknown-unknown/release/mathypad.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo "❌ WASM file not found at $WASM_FILE"
    exit 1
fi

# Generate JS bindings
echo "🔗 Generating bindings..."
if ! wasm-bindgen "$WASM_FILE" \
    --out-dir pkg \
    --target web \
    --no-typescript; then
    echo "❌ wasm-bindgen failed!"
    exit 1
fi

# Verify generated files exist
if [ ! -f "pkg/mathypad.js" ] || [ ! -f "pkg/mathypad_bg.wasm" ]; then
    echo "❌ Generated WASM files not found!"
    exit 1
fi

# Optimize WASM file size if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM for size..."
    wasm-opt -Oz --enable-bulk-memory --enable-sign-ext pkg/mathypad_bg.wasm -o pkg/mathypad_bg.wasm
else
    echo "💡 Installing wasm-opt for smaller bundle sizes..."
    cargo install wasm-opt
    echo "⚡ Optimizing WASM for size..."
    wasm-opt -Oz --enable-bulk-memory --enable-sign-ext pkg/mathypad_bg.wasm -o pkg/mathypad_bg.wasm
fi

# Show file sizes
echo "📊 WASM bundle size:"
ls -lh pkg/mathypad_bg.wasm pkg/mathypad.js | awk '{print "   " $9 ": " $5}'

echo ""
echo "✅ WASM build completed successfully!"
echo ""
echo "📁 Generated files:"
echo "   pkg/mathypad.js       - JavaScript bindings"
echo "   pkg/mathypad_bg.wasm  - WebAssembly binary"
echo ""
echo "💡 Next steps:"
echo "   - Copy these files to your web server"
echo "   - Ensure proper MIME types for .wasm files"
echo "   - Use './deploy.sh' for automatic deployment"