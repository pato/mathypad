//! WASM entry points for web builds

#[cfg(all(feature = "gui", target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

/// Entry point for WASM builds
#[cfg(all(feature = "gui", target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn main() {
    use crate::gui::MathypadGuiApp;

    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        // Get the canvas element from the DOM
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("mathypad_canvas"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("Failed to find canvas element with id 'mathypad_canvas'");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(MathypadGuiApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner.
        let loading = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        if let Some(loading) = loading {
            let _ = loading.set_attribute("style", "display: none;");
        }

        if let Err(e) = start_result {
            panic!("Failed to start eframe: {e:?}");
        }
    });
}
