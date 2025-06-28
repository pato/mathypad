use eframe::egui;
use egui::{Color32, FontId, TextEdit, TextStyle, ScrollArea};
use mathypad_core::core::MathypadCore;

/// The main application state
pub struct MathypadPocApp {
    /// Core calculation engine
    core: MathypadCore,
    /// The position of the separator (percentage of window width for left panel)
    separator_position: f32,
}

impl Default for MathypadPocApp {
    fn default() -> Self {
        let core = MathypadCore::new();

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

    /// Render the code editor with proper line numbers and syntax highlighting
    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        let content = self.core.get_content();
        
        // Calculate line count and max line number width
        let line_count = if content.is_empty() {
            1
        } else {
            content.lines().count().max(1)
        };
        
        // Fixed width for line numbers to ensure alignment
        let line_number_width = 50.0;
        
        ui.horizontal(|ui| {
            // Line numbers column
            self.render_line_numbers(ui, line_count, line_number_width);
            
            // Main editor
            self.render_main_editor(ui, content);
        });
    }
    
    /// Render the line numbers column
    fn render_line_numbers(&self, ui: &mut egui::Ui, line_count: usize, width: f32) {
        // Create line numbers text
        let line_numbers: String = (1..=line_count)
            .map(|i| format!("{:3}", i))
            .collect::<Vec<_>>()
            .join("\n");
        
        let mut line_numbers_text = line_numbers;
        
        ui.add_sized(
            [width, ui.available_height()],
            TextEdit::multiline(&mut line_numbers_text)
                .font(FontId::monospace(14.0))
                .interactive(false)
                .frame(false)
                .desired_width(width)
        );
    }
    
    /// Render the main editor with syntax highlighting
    fn render_main_editor(&mut self, ui: &mut egui::Ui, mut content: String) {
        let response = ui.add_sized(
            [ui.available_width(), ui.available_height()],
            TextEdit::multiline(&mut content)
                .font(FontId::monospace(14.0))
                .frame(false)
        );
        
        // Update core state if content changed
        if response.changed() {
            self.core.set_content(&content);
        }
    }
    
    
    /// Render the results panel with aligned line numbers
    fn render_results_panel(&self, ui: &mut egui::Ui) {
        let content = self.core.get_content();
        let line_count = if content.is_empty() {
            1
        } else {
            content.lines().count().max(1)
        };
        
        // Use same fixed width as editor
        let line_number_width = 50.0;
        
        ui.horizontal(|ui| {
            // Use the same line number rendering as the editor
            self.render_line_numbers(ui, line_count, line_number_width);
            
            // Results column - create multiline text to match editor layout
            let results_text: String = (0..line_count)
                .map(|i| {
                    if i < self.core.results.len() {
                        if let Some(res) = &self.core.results[i] {
                            res.clone()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            let mut results_display = results_text;
            
            ui.add_sized(
                [ui.available_width(), ui.available_height()],
                TextEdit::multiline(&mut results_display)
                    .font(FontId::monospace(14.0))
                    .interactive(false)
                    .frame(false)
                    .text_color(Color32::from_rgb(100, 200, 100))
            );
        });
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

                // Custom code editor with proper line numbers and syntax highlighting
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_code_editor(ui);
                });
            });

        // Right panel - Results  
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Results");
            ui.separator();

            // Results display with aligned line numbers
            ScrollArea::vertical().show(ui, |ui| {
                self.render_results_panel(ui);
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
