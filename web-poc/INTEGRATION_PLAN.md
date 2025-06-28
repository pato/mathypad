# Mathypad Core Integration Plan

## Overview

This plan outlines how to extract the core mathypad logic into a shared library that both TUI and web UI can use, ensuring zero code duplication while maintaining full compatibility with existing functionality.

## Phase 1: Create Shared Core Library (1-2 hours)

### Step 1.1: Create mathypad-core subcrate
```bash
# Add to workspace in root Cargo.toml
[workspace]
members = [".", "web-poc", "mathypad-core"]
```

### Step 1.2: Set up mathypad-core structure
```
mathypad-core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── expression/     # Moved from main crate
│   ├── units/          # Moved from main crate  
│   ├── core/           # New core abstractions
│   │   ├── mod.rs
│   │   ├── state.rs    # Core app state
│   │   ├── highlighting.rs # Syntax highlighting abstraction
│   │   └── file_ops.rs # File operation traits
│   └── test_helpers.rs # Shared test utilities
```

### Step 1.3: Move existing modules

1. **Copy `src/expression/` → `mathypad-core/src/expression/`**
   - No changes needed - already UI-agnostic
   - Update imports in `lib.rs`

2. **Copy `src/units/` → `mathypad-core/src/units/`** 
   - No changes needed - already UI-agnostic
   - Update imports in `lib.rs`

3. **Copy `src/test_helpers.rs` → `mathypad-core/src/test_helpers.rs`**
   - These are used by expression and units tests

### Step 1.4: Create core abstractions

#### `mathypad-core/src/core/state.rs`
```rust
use std::collections::HashMap;
use crate::expression::evaluator;

/// Core application state shared between TUI and web
#[derive(Debug, Clone)]
pub struct MathypadCore {
    pub text_lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub results: Vec<Option<String>>,
    pub variables: HashMap<String, String>,
}

impl MathypadCore {
    pub fn new() -> Self { /* ... */ }
    pub fn update_result(&mut self, line_index: usize) { /* ... */ }
    pub fn recalculate_all(&mut self) { /* ... */ }
    pub fn insert_char(&mut self, ch: char) { /* ... */ }
    pub fn delete_char(&mut self) { /* ... */ }
    // ... other core text manipulation methods
}
```

#### `mathypad-core/src/core/highlighting.rs`
```rust
/// UI-agnostic syntax highlighting
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedSpan {
    pub text: String,
    pub highlight_type: HighlightType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HighlightType {
    Number,
    Unit, 
    LineReference,
    Keyword,
    Operator,
    Variable,
    Function,
    Normal,
}

pub fn highlight_expression(text: &str, variables: &HashMap<String, String>) -> Vec<HighlightedSpan> {
    // Extract logic from current TUI render.rs
}
```

#### `mathypad-core/src/core/file_ops.rs`
```rust
use std::path::Path;

/// Trait for file operations - allows different backends
pub trait FileOperations {
    type Error;
    
    fn save_content(&self, path: &Path, content: &str) -> Result<(), Self::Error>;
    fn load_content(&self, path: &Path) -> Result<String, Self::Error>;
}

/// Content serialization utilities
pub fn serialize_lines(lines: &[String]) -> String {
    lines.join("\n")
}

pub fn deserialize_lines(content: &str) -> Vec<String> {
    content.lines().map(|s| s.to_string()).collect()
}
```

## Phase 2: Update Main TUI Crate (30-45 minutes)

### Step 2.1: Update main Cargo.toml
```toml
[dependencies]
mathypad-core = { path = "mathypad-core" }
# Remove redundant dependencies now in core
```

### Step 2.2: Update imports in main crate
```rust
// src/lib.rs
pub use mathypad_core::{expression, units, test_helpers};

// src/app.rs  
use mathypad_core::core::{MathypadCore, FileOperations};
use mathypad_core::core::file_ops::{serialize_lines, deserialize_lines};

pub struct App {
    pub core: MathypadCore,
    // TUI-specific fields
    pub mode: Mode,
    pub scroll_offset: usize,
    pub animations: Vec<Option<ResultAnimation>>,
    // ... other TUI-only state
}
```

### Step 2.3: Implement native file operations
```rust
// src/app.rs
use std::fs;

struct NativeFileSystem;

impl FileOperations for NativeFileSystem {
    type Error = std::io::Error;
    
    fn save_content(&self, path: &Path, content: &str) -> Result<(), Self::Error> {
        fs::write(path, content)
    }
    
    fn load_content(&self, path: &Path) -> Result<String, Self::Error> {
        fs::read_to_string(path)
    }
}
```

### Step 2.4: Update syntax highlighting in render.rs
```rust
// src/ui/render.rs
use mathypad_core::core::highlighting::{highlight_expression, HighlightType};

fn create_syntax_highlighted_line(/* ... */) -> Line<'_> {
    let highlighted = highlight_expression(text, variables);
    
    let spans: Vec<Span> = highlighted.into_iter().map(|span| {
        let style = match span.highlight_type {
            HighlightType::Number => Style::default().fg(Color::Cyan),
            HighlightType::Unit => Style::default().fg(Color::Green),
            // ... map all highlight types to ratatui styles
        };
        Span::styled(span.text, style)
    }).collect();
    
    Line::from(spans)
}
```

### Step 2.5: Update App methods to delegate to core
```rust
impl App {
    pub fn insert_char(&mut self, ch: char) {
        self.core.insert_char(ch);
        // TUI-specific logic (animations, etc.)
    }
    
    pub fn save(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let fs = NativeFileSystem;
        let content = serialize_lines(&self.core.text_lines);
        fs.save_content(path, &content)
    }
}
```

## Phase 3: Update Web POC (45-60 minutes)

### Step 3.1: Update web-poc Cargo.toml
```toml
[dependencies]
mathypad-core = { path = "../mathypad-core" }
# Add core dependency
```

### Step 3.2: Create web-specific app state
```rust
// web-poc/src/app.rs
use mathypad_core::core::MathypadCore;
use mathypad_core::core::highlighting::{highlight_expression, HighlightType};

pub struct MathypadPocApp {
    pub core: MathypadCore,
    // Web-specific UI state
    pub separator_position: f32,
}

impl MathypadPocApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        configure_visuals(&cc.egui_ctx);
        
        Self {
            core: MathypadCore::new(),
            separator_position: 70.0,
        }
    }
}
```

### Step 3.3: Implement real calculation in web UI
```rust
impl eframe::App for MathypadPocApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left panel - text input with real evaluation
        egui::SidePanel::left("text_panel")
            .show(ctx, |ui| {
                // Real text editor that updates core state
                let mut text = self.core.text_lines.join("\n");
                let response = ui.add(
                    TextEdit::multiline(&mut text)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                );
                
                if response.changed() {
                    self.core.text_lines = text.lines().map(|s| s.to_string()).collect();
                    self.core.recalculate_all();
                }
            });
            
        // Right panel - real results  
        egui::CentralPanel::default().show(ctx, |ui| {
            for (i, result) in self.core.results.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{:3} ", i + 1));
                    if let Some(res) = result {
                        ui.label(RichText::new(res).color(Color32::from_rgb(100, 200, 100)));
                    }
                });
            }
        });
    }
}
```

### Step 3.4: Add syntax highlighting to web UI
```rust
// In the text editor section
if response.has_focus() {
    // Apply syntax highlighting using shared logic
    let highlighted = highlight_expression(&text, &self.core.variables);
    // Convert to egui RichText styling
}
```

### Step 3.5: Implement web file operations (future)
```rust
// For future implementation
struct WebFileSystem;

impl FileOperations for WebFileSystem {
    type Error = String; // Or custom web error type
    
    fn save_content(&self, path: &Path, content: &str) -> Result<(), Self::Error> {
        // Use localStorage or File API
        web_sys::window()
            .unwrap()
            .local_storage()
            .unwrap()
            .unwrap()
            .set_item(&path.to_string_lossy(), content)
            .map_err(|e| format!("{:?}", e))
    }
    
    fn load_content(&self, path: &Path) -> Result<String, Self::Error> {
        // Load from localStorage
        // ...
    }
}
```

## Phase 4: Testing and Validation (30-45 minutes)

### Step 4.1: Verify TUI functionality unchanged
```bash
# Run all existing tests
cargo test --workspace

# Test TUI manually
cargo run --bin mathypad

# Test one-shot mode
cargo run -- -- "100 + 50"
```

### Step 4.2: Test web POC with real calculations
```bash
cd web-poc
./run-web.sh
# Verify:
# - Text input works
# - Calculations appear in results panel  
# - Line references work
# - Unit conversions work
```

### Step 4.3: Verify no regressions
- All 231 existing tests still pass
- TUI behavior identical to before
- Web POC now has real calculation functionality

## Benefits of This Approach

### ✅ **Zero Duplication**
- Single implementation of expression parsing
- Single implementation of unit conversions  
- Single implementation of mathematical evaluation
- Shared syntax highlighting logic

### ✅ **Maintainability**
- Bug fixes benefit both UIs
- New features automatically available to both
- Single source of truth for mathematical operations

### ✅ **Compatibility**
- Existing TUI functionality unchanged
- All tests continue to pass
- No breaking changes to public APIs

### ✅ **Performance**
- No runtime overhead from abstractions
- Compile-time optimizations preserved
- WASM benefits from Rust's performance

### ✅ **Future-Proof**
- Easy to add new UI backends
- Clean separation of concerns
- Testable core logic independent of UI

## Timeline Estimate

- **Phase 1**: 1-2 hours (core library creation)
- **Phase 2**: 30-45 minutes (TUI updates)  
- **Phase 3**: 45-60 minutes (web POC integration)
- **Phase 4**: 30-45 minutes (testing)

**Total**: 2.5-4 hours for complete integration

## Rollback Plan

If issues arise:
1. Keep original code intact during migration
2. Use feature flags to switch between old/new implementations
3. All changes are additive - can revert by removing mathypad-core dependency

This plan ensures a smooth transition to shared core logic while maintaining full compatibility and adding real calculation functionality to the web POC.