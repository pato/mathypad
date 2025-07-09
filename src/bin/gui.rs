//! GUI binary entry point for mathypad (requires 'gui' feature)

#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
fn main() -> eframe::Result<()> {
    use mathypad::gui::MathypadGuiApp;

    env_logger::init(); // Log to stderr (if you want to see logs from eframe)

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mathypad",
        options,
        Box::new(|cc| Ok(Box::new(MathypadGuiApp::new(cc)))),
    )
}

#[cfg(all(feature = "gui", target_arch = "wasm32"))]
fn main() {
    eprintln!("Error: mathypad-gui binary is not available for WASM builds");
    eprintln!("Use the library's WASM export or web-poc crate instead");
    std::process::exit(1);
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("Error: mathypad-gui requires the 'gui' feature to be enabled");
    eprintln!("Build with: cargo build --features gui --bin mathypad-gui");
    std::process::exit(1);
}
