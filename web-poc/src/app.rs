use eframe::egui;
use egui::{Color32, FontId, TextEdit, TextStyle, ScrollArea};
use egui::text::{LayoutJob, TextFormat};
use mathypad_core::core::{MathypadCore, highlighting::{highlight_expression, HighlightType}};

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

    /// Calculate line count consistently for both editor and results  
    fn calculate_line_count(&self) -> usize {
        // Now that core properly manages lines, just use the text_lines count
        self.core.text_lines.len()
    }

    /// Convert mathypad HighlightType to egui Color32 with light theme colors
    fn highlight_type_to_color(&self, highlight_type: &HighlightType) -> Color32 {
        // Use darker colors suitable for light background instead of the bright colors
        // designed for dark terminal backgrounds
        match highlight_type {
            HighlightType::Number => Color32::from_rgb(0, 100, 180),        // Dark blue (readable on white)
            HighlightType::Unit => Color32::from_rgb(0, 120, 60),           // Dark green  
            HighlightType::LineReference => Color32::from_rgb(150, 50, 150), // Dark magenta
            HighlightType::Keyword => Color32::from_rgb(180, 100, 0),       // Dark orange (instead of bright yellow)
            HighlightType::Operator => Color32::from_rgb(0, 150, 150),      // Dark cyan
            HighlightType::Variable => Color32::from_rgb(50, 100, 150),     // Dark blue-gray
            HighlightType::Function => Color32::from_rgb(100, 50, 180),     // Dark purple
            HighlightType::Normal => Color32::from_rgb(50, 50, 50),         // Dark gray (readable on white)
        }
    }

    /// Create a LayoutJob with mathypad syntax highlighting
    fn create_highlighted_layout_job(&self, text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();
        
        // Use mathypad's existing highlighting function - no code duplication!
        let highlighted_spans = highlight_expression(text, &self.core.variables);
        
        for span in highlighted_spans {
            let color = self.highlight_type_to_color(&span.highlight_type);
            let format = TextFormat::simple(
                FontId::monospace(14.0),
                color
            );
            
            job.append(&span.text, 0.0, format);
        }
        
        job
    }

    /// Render the code editor with proper line numbers and syntax highlighting
    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        let content = self.core.get_content();
        let line_count = self.calculate_line_count();
        
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
        let original_content = content.clone();
        
        // Pre-create the highlighted layout job to avoid borrowing issues
        let highlighted_layout = self.create_highlighted_layout_job(&content);
        let content_for_comparison = content.clone();
        
        // Create a custom layouter for syntax highlighting
        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            // If the string matches our content, use our pre-computed highlighting
            if string == content_for_comparison {
                ui.fonts(|f| f.layout_job(highlighted_layout.clone()))
            } else {
                // For different content (intermediate states), create simple highlighting
                let mut simple_job = LayoutJob::default();
                simple_job.append(
                    string,
                    0.0,
                    TextFormat::simple(FontId::monospace(14.0), Color32::WHITE)
                );
                ui.fonts(|f| f.layout_job(simple_job))
            }
        };
        
        let response = ui.add(
            TextEdit::multiline(&mut content)
                .code_editor()
                .frame(false)
                .desired_width(ui.available_width())
                .desired_rows(25) // Minimum rows to ensure space
                .layouter(&mut layouter)
        );
        
        // Update core state if content changed
        if response.changed() {
            self.smart_update_content(&original_content, &content);
        }
    }
    
    /// Smart content update that preserves cursor position when possible
    fn smart_update_content(&mut self, old_content: &str, new_content: &str) {
        // For now, use a simple heuristic: if the new content just has more newlines
        // at the end, it's likely the user pressed Enter at the end
        if new_content.len() > old_content.len() {
            let diff = &new_content[old_content.len()..];
            if diff == "\n" && old_content.is_empty() {
                // Special case: empty pad + Enter = add first new line
                // Use the new method that handles line references
                self.core.update_content_with_line_references(new_content);
                // Try to set cursor to second line
                if self.core.text_lines.len() >= 2 {
                    self.core.cursor_line = 1;
                    self.core.cursor_col = 0;
                }
                return;
            }
        }
        
        // Use the new method that handles line reference updates
        self.core.update_content_with_line_references(new_content);
    }
    
    
    /// Render the results panel with aligned line numbers
    fn render_results_panel(&self, ui: &mut egui::Ui) {
        let line_count = self.calculate_line_count();
        
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
