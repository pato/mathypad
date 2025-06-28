use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, ScrollArea, TextEdit, TextStyle};
use mathypad_core::core::{
    MathypadCore,
    highlighting::{HighlightType, highlight_expression},
};

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
                .desired_width(width),
        );
    }

    /// Render the main editor with syntax highlighting
    fn render_main_editor(&mut self, ui: &mut egui::Ui, mut content: String) {
        let original_content = content.clone();

        // Use a stable ID to help egui maintain widget state consistently
        let text_edit_id = ui.make_persistent_id("mathypad_editor");

        // Try minimal custom layouter with stable behavior
        let variables = self.core.variables.clone(); // Clone to avoid borrow issues
        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            // Only highlight if string is not empty and looks stable
            if string.is_empty() {
                // Empty string case
                let mut job = LayoutJob::default();
                job.append("", 0.0, TextFormat::simple(FontId::monospace(14.0), Color32::from_rgb(200, 200, 200)));
                ui.fonts(|f| f.layout_job(job))
            } else {
                // Apply highlighting
                let highlighted_spans = highlight_expression(string, &variables);
                let mut job = LayoutJob::default();
                
                for span in highlighted_spans {
                    let color = match span.highlight_type {
                        // Elegant dark colors for light background - inspired by GitHub/VSCode light themes
                        HighlightType::Number => Color32::from_rgb(9, 134, 88),        // Dark green (GitHub numbers)
                        HighlightType::Unit => Color32::from_rgb(0, 92, 197),          // Dark blue (GitHub keywords)
                        HighlightType::LineReference => Color32::from_rgb(181, 118, 20), // Dark orange/amber
                        HighlightType::Keyword => Color32::from_rgb(215, 58, 73),      // Dark red (GitHub keywords)
                        HighlightType::Operator => Color32::from_rgb(36, 41, 47),      // Very dark gray (almost black)
                        HighlightType::Variable => Color32::from_rgb(111, 66, 193),    // Dark purple
                        HighlightType::Function => Color32::from_rgb(102, 57, 186),    // Dark violet
                        HighlightType::Normal => Color32::from_rgb(36, 41, 47),        // Dark gray
                    };
                    let format = TextFormat::simple(FontId::monospace(14.0), color);
                    job.append(&span.text, 0.0, format);
                }
                
                ui.fonts(|f| f.layout_job(job))
            }
        };

        let response = ui.add(
            TextEdit::multiline(&mut content)
                .id(text_edit_id)
                .code_editor()
                .frame(false)
                .desired_width(ui.available_width())
                .desired_rows(25)
                .layouter(&mut layouter),
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
                    .text_color(Color32::from_rgb(100, 200, 100)),
            );
        });
    }
}

impl eframe::App for MathypadPocApp {
    /// Called each time the UI needs repainting
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if we're on mobile (narrow screen)
        let total_width = ctx.screen_rect().width();
        let is_mobile = total_width < 600.0;

        if is_mobile {
            // Mobile layout: Stack vertically
            self.render_mobile_layout(ctx);
        } else {
            // Desktop layout: Side by side panels
            self.render_desktop_layout(ctx, total_width);
        }
    }
}

impl MathypadPocApp {
    /// Render mobile-friendly vertical layout
    fn render_mobile_layout(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("📱 Mathypad");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("Results stay visible while typing!");
                });
            });
            ui.separator();

            // Split screen vertically for mobile - Results FIRST, then editor
            let available_height = ui.available_height();
            let results_height = available_height * 0.4; // 40% for results (top)

            // Results section FIRST (top) - visible when keyboard is open
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), results_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.label("📊 Results:");
                    ScrollArea::vertical()
                        .max_height(results_height - 30.0)
                        .show(ui, |ui| {
                            self.render_results_panel(ui);
                        });
                },
            );

            ui.separator();

            // Editor section (bottom) - gets covered by keyboard but that's OK
            ui.label("📝 Input:");
            ScrollArea::vertical().show(ui, |ui| {
                self.render_code_editor(ui);
            });
        });
    }

    /// Render desktop layout with resizable panels
    fn render_desktop_layout(&mut self, ctx: &egui::Context, total_width: f32) {
        // Calculate panel widths
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
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Detect mobile and adjust font sizes accordingly
    let is_mobile = ctx.screen_rect().width() < 600.0;
    let base_font_size = if is_mobile { 16.0 } else { 14.0 }; // Larger on mobile
    let heading_size = if is_mobile { 20.0 } else { 18.0 };

    // Use monospace font for code
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(base_font_size, egui::FontFamily::Monospace),
    );

    // Set default font size
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(base_font_size, egui::FontFamily::Monospace),
    );

    // Larger headings
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(heading_size, egui::FontFamily::Proportional),
    );

    // Improve spacing for mobile
    if is_mobile {
        style.spacing.item_spacing.y = 8.0; // More vertical spacing
        style.spacing.button_padding = egui::Vec2::new(12.0, 8.0); // Bigger touch targets
        style.spacing.menu_margin = egui::Margin::same(8.0);
    }

    ctx.set_style(style);
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();

    // Clean light theme colors
    visuals.panel_fill = Color32::from_rgb(248, 249, 250);     // Very light gray
    visuals.window_fill = Color32::from_rgb(255, 255, 255);    // Pure white
    visuals.faint_bg_color = Color32::from_rgb(246, 248, 250); // Slightly darker gray

    // Text colors for light theme
    visuals.override_text_color = Some(Color32::from_rgb(36, 41, 47)); // Dark gray text

    // Selection colors for light theme
    visuals.selection.bg_fill = Color32::from_rgb(0, 92, 197); // Blue selection

    ctx.set_visuals(visuals);
}
