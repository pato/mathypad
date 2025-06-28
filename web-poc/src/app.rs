use egui::{Color32, FontId, RichText, ScrollArea, TextEdit, TextStyle};
use eframe::egui;

/// The main application state
pub struct MathypadPocApp {
    /// The input text in the left panel
    input_text: String,
    /// The position of the separator (percentage of window width for left panel)
    separator_position: f32,
    /// Dummy results for display
    dummy_results: Vec<(usize, Option<String>)>,
}

impl Default for MathypadPocApp {
    fn default() -> Self {
        Self {
            input_text: "5 + 3\n10 kg to lb\nline1 * 2\nsin(30 degrees)\n$100/month * 12\n".to_string(),
            separator_position: 70.0,
            dummy_results: vec![
                (1, Some("8".to_string())),
                (2, Some("22.046 lb".to_string())),
                (3, Some("16".to_string())),
                (4, Some("0.5".to_string())),
                (5, Some("$1,200/year".to_string())),
            ],
        }
    }
}

impl MathypadPocApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts
        configure_fonts(&cc.egui_ctx);
        
        // Configure visuals for dark theme
        configure_visuals(&cc.egui_ctx);

        Default::default()
    }
}

impl eframe::App for MathypadPocApp {
    /// Called each time the UI needs repainting
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Calculate panel widths
        let total_width = ctx.screen_rect().width();
        let left_width = total_width * (self.separator_position / 100.0);

        // Left panel - Text input
        egui::SidePanel::left("text_panel")
            .resizable(true)
            .default_width(left_width)
            .width_range(200.0..=total_width - 200.0)
            .show(ctx, |ui| {
                ui.heading("Mathypad");
                ui.separator();
                
                // Text editor with line numbers
                ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Line numbers column
                        let line_count = self.input_text.lines().count().max(1);
                        ui.vertical(|ui| {
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            for i in 1..=line_count {
                                ui.label(
                                    RichText::new(format!("{:3} ", i))
                                        .color(Color32::from_gray(128))
                                        .monospace()
                                );
                            }
                        });
                        
                        // Text editor
                        ui.vertical(|ui| {
                            let response = ui.add(
                                TextEdit::multiline(&mut self.input_text)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .frame(false)
                            );
                            
                            // Ensure the editor gets focus
                            if response.has_focus() {
                                // In a real app, we'd do syntax highlighting here
                            }
                        });
                    });
                });
            });

        // Right panel - Results
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Results");
            ui.separator();
            
            // Results display
            ScrollArea::vertical().show(ui, |ui| {
                // Show dummy results aligned with line numbers
                let mut current_line = 1;
                for &(line_num, ref result) in &self.dummy_results {
                    // Add empty lines for spacing
                    while current_line < line_num {
                        ui.label(
                            RichText::new(format!("{:3} ", current_line))
                                .color(Color32::from_gray(128))
                                .monospace()
                        );
                        current_line += 1;
                    }
                    
                    // Show result
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{:3} ", line_num))
                                .color(Color32::from_gray(128))
                                .monospace()
                        );
                        if let Some(res) = result {
                            ui.label(
                                RichText::new(res)
                                    .color(Color32::from_rgb(100, 200, 100))
                                    .monospace()
                            );
                        }
                    });
                    current_line += 1;
                }
            });
        });

        // Note: Panel resizing is handled automatically by egui
        // The separator_position will be updated automatically when panels are resized
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Use monospace font for code
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(14.0, egui::FontFamily::Monospace),
    );
    
    // Set default font size
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(14.0, egui::FontFamily::Monospace),
    );
    
    ctx.set_style(style);
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Dark theme colors similar to terminal
    visuals.panel_fill = Color32::from_rgb(30, 30, 30);
    visuals.window_fill = Color32::from_rgb(30, 30, 30);
    visuals.faint_bg_color = Color32::from_rgb(35, 35, 35);
    
    // Text colors
    visuals.override_text_color = Some(Color32::from_rgb(200, 200, 200));
    
    // Selection colors
    visuals.selection.bg_fill = Color32::from_rgb(60, 90, 120);
    
    ctx.set_visuals(visuals);
}