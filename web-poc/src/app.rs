use eframe::egui;
use egui::{Color32, FontId, RichText, ScrollArea, TextEdit, TextStyle};
use mathypad_core::core::MathypadCore;
use mathypad_core::core::highlighting::{highlight_expression, HighlightType};

/// Convert a HighlightType to an egui Color32 using shared color system
fn highlight_type_to_color32(highlight_type: &HighlightType) -> Color32 {
    let (r, g, b) = highlight_type.rgb_color();
    Color32::from_rgb(r, g, b)
}

/// The main application state
pub struct MathypadPocApp {
    /// Core calculation engine
    core: MathypadCore,
    /// The position of the separator (percentage of window width for left panel)
    separator_position: f32,
}

impl Default for MathypadPocApp {
    fn default() -> Self {
        let mut core = MathypadCore::new();
        // Set up some example content
        core.set_content("5 + 3\n10 kg to lb\nline1 * 2\nsqrt(16)\n$100/month * 12");

        Self {
            core,
            separator_position: 70.0,
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
                        let line_count = self.core.text_lines.len();
                        ui.vertical(|ui| {
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            for i in 1..=line_count {
                                ui.label(
                                    RichText::new(format!("{:3} ", i))
                                        .color(Color32::from_gray(128))
                                        .monospace(),
                                );
                            }
                        });

                        // Syntax highlighted text display
                        ui.vertical(|ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                for (line_num, line_text) in self.core.text_lines.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        // Line number
                                        ui.label(
                                            RichText::new(format!("{:3} ", line_num + 1))
                                                .color(Color32::from_gray(128))
                                                .monospace(),
                                        );
                                        
                                        // Syntax highlighted line content
                                        let highlighted_spans = highlight_expression(line_text, &self.core.variables);
                                        for span in highlighted_spans {
                                            let color = highlight_type_to_color32(&span.highlight_type);
                                            ui.label(
                                                RichText::new(span.text)
                                                    .color(color)
                                                    .monospace(),
                                            );
                                        }
                                    });
                                }
                            });
                            
                            ui.separator();
                            
                            // Editable text area
                            let mut content = self.core.get_content();
                            let response = ui.add(
                                TextEdit::multiline(&mut content)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .frame(true),
                            );

                            // Update core state if content changed
                            if response.changed() {
                                self.core.set_content(&content);
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
                // Show real calculation results aligned with line numbers
                for (i, result) in self.core.results.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{:3} ", i + 1))
                                .color(Color32::from_gray(128))
                                .monospace(),
                        );
                        if let Some(res) = result {
                            ui.label(
                                RichText::new(res)
                                    .color(Color32::from_rgb(100, 200, 100))
                                    .monospace(),
                            );
                        }
                    });
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
