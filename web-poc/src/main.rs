#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use mathypad_web_poc::MathypadPocApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("Mathypad Web POC"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Mathypad Web POC",
        native_options,
        Box::new(|cc| Ok(Box::new(MathypadPocApp::new(cc)))),
    )
}

// For WASM builds, main is handled in lib.rs
#[cfg(target_arch = "wasm32")]
fn main() {
    // This will never be called for WASM builds
}