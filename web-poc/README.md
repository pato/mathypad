# Mathypad Web POC

This is a proof-of-concept web UI for Mathypad using the Egui framework. It demonstrates the visual layout and text editing experience without any calculation logic.

## Features

- ✅ **Two-panel layout** with resizable separator
- ✅ **Text editor** with line numbers and syntax highlighting 
- ✅ **Dark theme** matching the TUI appearance
- ✅ **Dummy results** showing potential functionality
- ✅ **Responsive design** working on different screen sizes
- ✅ **WASM build** for web deployment

## Project Structure

```
web-poc/
├── src/
│   ├── main.rs              # Native entry point
│   ├── lib.rs               # WASM entry point
│   └── app.rs               # Main application logic
├── index.html               # Web page hosting the app
├── build-web.sh             # WASM build script
├── pkg/                     # Generated WASM files (after build)
├── Cargo.toml              # Dependencies
└── README.md               # This file
```

## Running the POC

### Option 1: Native Desktop App

```bash
# From the web-poc directory
cargo run --bin mathypad-web-poc
```

### Option 2: Web Version

1. **Build the WASM version:**
   ```bash
   ./build-web.sh
   ```

2. **Start a local web server:**
   ```bash
   python3 -m http.server 8080
   ```

3. **Open in your browser:**
   http://localhost:8080

## Development

### Prerequisites

- Rust 2024 edition
- For WASM builds: `wasm-bindgen-cli` (installed automatically by build script)

### Building

- **Native:** `cargo build`
- **WASM:** `./build-web.sh`

### Architecture

The app uses Egui for cross-platform UI rendering:

- **Native:** Uses eframe with OpenGL backend
- **Web:** Compiles to WASM and renders on HTML5 Canvas

## Key Components

### `MathypadPocApp`

Main application state containing:
- Input text for the left panel
- Separator position (resizable)
- Dummy results for display

### Two-Panel Layout

- **Left Panel:** Text editor with line numbers
- **Right Panel:** Results display with dummy data
- **Separator:** Resizable divider between panels

### Styling

- Dark theme matching TUI colors
- Monospace font for calculator feel  
- Syntax highlighting for basic tokens

## Next Steps

Once this POC is validated:

1. **Extract core logic** from main mathypad into shared library
2. **Replace dummy results** with real calculation engine
3. **Add advanced features** (copy/paste, file operations, etc.)
4. **Deploy as production** web application

## Benefits of This Approach

- ✅ **Risk-free validation** - No impact on existing codebase
- ✅ **Cross-platform** - Same code runs native and web
- ✅ **Modern UI framework** - Egui provides excellent developer experience
- ✅ **Performance** - Compiled Rust + WASM for web efficiency