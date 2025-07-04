//! UI rendering functions

use crate::{App, Mode};
use mathypad_core::core::highlighting::{HighlightType, highlight_expression};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::collections::HashMap;

/// Convert a HighlightType to a ratatui Color using shared color system
fn highlight_type_to_color(highlight_type: &HighlightType) -> Color {
    let (r, g, b) = highlight_type.rgb_color();
    Color::Rgb(r, g, b)
}

/// Animate a color by interpolating its intensity based on opacity
fn animate_color(base_color: Color, opacity: f32) -> Color {
    match base_color {
        Color::Green => {
            // Fade from dark green to bright green
            let intensity = (opacity * 255.0) as u8;
            Color::Rgb(0, intensity, 0)
        }
        Color::Red => {
            let intensity = (opacity * 255.0) as u8;
            Color::Rgb(intensity, 0, 0)
        }
        Color::Blue => {
            let intensity = (opacity * 255.0) as u8;
            Color::Rgb(0, 0, intensity)
        }
        Color::Yellow => {
            let intensity = (opacity * 255.0) as u8;
            Color::Rgb(intensity, intensity, 0)
        }
        Color::Cyan => {
            let intensity = (opacity * 255.0) as u8;
            Color::Rgb(0, intensity, intensity)
        }
        Color::Magenta => {
            let intensity = (opacity * 255.0) as u8;
            Color::Rgb(intensity, 0, intensity)
        }
        _ => base_color, // For other colors, just return as-is
    }
}

/// Create a flash effect color that brightens based on opacity
fn create_flash_color(opacity: f32) -> Color {
    // Create a bright flash effect that fades from white to normal
    let intensity = (opacity * 255.0) as u8;
    Color::Rgb(255, 255, intensity.max(200)) // Bright white/yellow flash
}

/// Main UI layout and rendering
pub fn ui(f: &mut Frame, app: &App) {
    // Check if we need to reserve space for command line
    let main_area = if app.mode == Mode::Command {
        // Reserve one line at the bottom for command line
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main content area
                Constraint::Length(1), // Command line
            ])
            .split(f.area());

        // Render command line first
        render_command_line(f, app, vertical_chunks[1]);

        vertical_chunks[0] // Use the main content area
    } else {
        f.area() // Use the full area
    };

    let text_percentage = app.separator_position;
    let results_percentage = 100 - app.separator_position;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(text_percentage),
            Constraint::Percentage(results_percentage),
        ])
        .split(main_area);

    render_text_area(f, app, chunks[0]);
    render_results_panel(f, app, chunks[1]);

    // Render separator visual feedback if hovering or dragging
    if app.is_dragging_separator || app.is_hovering_separator {
        render_separator_indicator(f, app, f.area());
    }

    // Render dialogs on top if needed
    if app.show_welcome_dialog {
        render_welcome_dialog(f, app, f.area());
    } else if app.show_unsaved_dialog {
        render_unsaved_dialog(f, app, f.area());
    } else if app.show_save_as_dialog {
        render_save_as_dialog(f, app, f.area());
    }
}

/// Render the main text editing area
pub fn render_text_area(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.has_unsaved_changes {
        "Mathypad * "
    } else {
        "Mathypad"
    };
    let block = match app.mode {
        Mode::Insert => Block::default().title(title).borders(Borders::ALL),
        Mode::Normal => Block::default()
            .title(title)
            .borders(Borders::ALL)
            .title_bottom(" NORMAL "),
        Mode::Command => Block::default()
            .title(title)
            .borders(Borders::ALL)
            .title_bottom(" COMMAND "),
    };

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let visible_height = inner_area.height as usize;
    let start_line = app.scroll_offset;
    let end_line = (start_line + visible_height).min(app.core.text_lines.len());

    let mut lines = Vec::new();
    for (i, line_text) in app.core.text_lines[start_line..end_line].iter().enumerate() {
        let line_num = start_line + i + 1;
        let line_num_str = format!("{:4} ", line_num);
        let line_index = start_line + i;

        let mut spans = vec![Span::styled(
            line_num_str,
            Style::default().fg(Color::DarkGray),
        )];

        // Check if this line has a copy flash animation for the text area (not result area)
        let line_style = if let Some(animation) = app.get_copy_flash_animation(line_index) {
            // Only flash if this was a text area copy (not result area)
            if line_index < app.copy_flash_is_result.len() && !app.copy_flash_is_result[line_index]
            {
                let opacity = animation.opacity();
                Style::default().bg(create_flash_color(opacity))
            } else {
                Style::default()
            }
        } else {
            Style::default()
        };

        if start_line + i == app.core.cursor_line {
            // Parse with cursor highlighting
            let mut colored_spans =
                parse_colors_with_cursor(line_text, app.core.cursor_col, &app.core.variables);
            // Apply flash background to all spans if flashing
            if line_style.bg.is_some() {
                for span in &mut colored_spans {
                    span.style = span.style.patch(line_style);
                }
            }
            spans.extend(colored_spans);
        } else {
            let mut colored_spans = parse_colors(line_text, &app.core.variables);
            // Apply flash background to all spans if flashing
            if line_style.bg.is_some() {
                for span in &mut colored_spans {
                    span.style = span.style.patch(line_style);
                }
            }
            spans.extend(colored_spans);
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner_area);
}

/// Render the results panel
pub fn render_results_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Results").borders(Borders::ALL);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let visible_height = inner_area.height as usize;
    let start_line = app.scroll_offset;
    let end_line = (start_line + visible_height).min(app.core.results.len());

    let mut lines = Vec::new();
    for (i, result) in app.core.results[start_line..end_line].iter().enumerate() {
        let line_num = start_line + i + 1;
        let line_num_str = format!("{:4} ", line_num);
        let line_index = start_line + i;

        let mut spans = vec![Span::styled(
            line_num_str,
            Style::default().fg(Color::DarkGray),
        )];

        // Check if this line has a copy flash animation for the results area (not text area)
        let flash_style = if let Some(animation) = app.get_copy_flash_animation(line_index) {
            // Only flash if this was a result area copy (not text area)
            if line_index < app.copy_flash_is_result.len() && app.copy_flash_is_result[line_index] {
                let opacity = animation.opacity();
                Some(Style::default().bg(create_flash_color(opacity)))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(value) = result {
            // Get animation state for this line
            let color = if let Some(animation) = app.get_result_animation(line_index) {
                // Apply fade-in animation by adjusting color intensity
                let opacity = animation.opacity();
                animate_color(Color::Green, opacity)
            } else {
                Color::Green
            };

            let mut result_style = Style::default().fg(color);
            // Apply flash background if flashing
            if let Some(flash) = flash_style {
                result_style = result_style.patch(flash);
            }

            spans.push(Span::styled(value.clone(), result_style));
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner_area);
}

/// Parse text and return colored spans for syntax highlighting using shared logic
pub fn parse_colors<'a>(text: &'a str, variables: &'a HashMap<String, String>) -> Vec<Span<'a>> {
    let highlighted_spans = highlight_expression(text, variables);

    highlighted_spans
        .into_iter()
        .map(|span| {
            let color = highlight_type_to_color(&span.highlight_type);
            if color == Color::Reset {
                Span::raw(span.text)
            } else {
                Span::styled(span.text, Style::default().fg(color))
            }
        })
        .collect()
}

/// Parse text and return colored spans with cursor highlighting using shared logic
pub fn parse_colors_with_cursor<'a>(
    text: &'a str,
    cursor_col: usize,
    variables: &'a HashMap<String, String>,
) -> Vec<Span<'a>> {
    let highlighted_spans = highlight_expression(text, variables);
    let mut spans = Vec::new();
    let mut char_index = 0; // Track character position for cursor

    for highlighted_span in highlighted_spans {
        let span_text = highlighted_span.text;
        let span_start = char_index;
        let span_end = char_index + span_text.chars().count();
        let base_color = highlight_type_to_color(&highlighted_span.highlight_type);

        // Check if cursor is within this span
        if cursor_col >= span_start && cursor_col < span_end {
            // Split the span to highlight the cursor character
            let cursor_offset = cursor_col - span_start;
            let span_chars: Vec<char> = span_text.chars().collect();

            if cursor_offset > 0 {
                let before: String = span_chars[..cursor_offset].iter().collect();
                if base_color == Color::Reset {
                    spans.push(Span::raw(before));
                } else {
                    spans.push(Span::styled(before, Style::default().fg(base_color)));
                }
            }

            let cursor_char = span_chars[cursor_offset];
            let cursor_style = Style::default().bg(Color::White).fg(Color::Black);
            spans.push(Span::styled(cursor_char.to_string(), cursor_style));

            if cursor_offset + 1 < span_chars.len() {
                let after: String = span_chars[cursor_offset + 1..].iter().collect();
                if base_color == Color::Reset {
                    spans.push(Span::raw(after));
                } else {
                    spans.push(Span::styled(after, Style::default().fg(base_color)));
                }
            }
        } else {
            // Normal span without cursor
            if base_color == Color::Reset {
                spans.push(Span::raw(span_text));
            } else {
                spans.push(Span::styled(span_text, Style::default().fg(base_color)));
            }
        }

        char_index = span_end;
    }

    // Handle cursor at end of line
    if cursor_col >= char_index {
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::White).fg(Color::Black),
        ));
    }

    spans
}

/// Render the unsaved changes confirmation dialog
pub fn render_unsaved_dialog(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::Clear;

    // Calculate dialog size and position (centered)
    let dialog_width = 60;
    let dialog_height = 8;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background for the dialog
    f.render_widget(Clear, dialog_area);

    // Create the dialog block
    let block = Block::default()
        .title(" Unsaved Changes ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    // Create the dialog content
    let filename = app
        .file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled");

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "You have unsaved changes in ",
                Style::default().fg(Color::White),
            ),
            Span::styled(filename, Style::default().fg(Color::Yellow)),
            Span::styled(".", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
            Span::styled(" - Save and quit", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::styled(" - Quit without saving", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::styled("    - Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog_area);
}

/// Render a visual indicator for the separator when dragging
pub fn render_separator_indicator(f: &mut Frame, app: &App, area: Rect) {
    // Calculate the layout split to get the exact separator position
    let text_percentage = app.separator_position;
    let results_percentage = 100 - app.separator_position;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(text_percentage),
            Constraint::Percentage(results_percentage),
        ])
        .split(area);

    // The separator should be at the boundary between the two panels
    // We want to draw it exactly where the new layout boundary will be
    let separator_x = chunks[0].x + chunks[0].width;

    // Calculate the inner area (excluding borders) to determine where to draw the line
    // Both panels have the same border structure, so we only need to calculate one
    let panel_block = Block::default().borders(Borders::ALL);
    let inner_area = panel_block.inner(chunks[0]);

    // Use the inner area to determine the vertical bounds for the separator line
    // Extend one character up and down to cover the border corners for a cleaner look
    let separator_start_y = inner_area.y.saturating_sub(1);
    let separator_end_y = (inner_area.y + inner_area.height + 1).min(area.y + area.height);

    // Only draw if the separator position is within the valid area
    if separator_x >= area.x && separator_x < area.x + area.width {
        // Draw the separator line only within the content area (respecting borders)
        for y_offset in 0..(separator_end_y - separator_start_y) {
            let separator_area = Rect {
                x: separator_x.saturating_sub(1), // Position it just before the boundary
                y: separator_start_y + y_offset,
                width: 1,
                height: 1,
            };

            // Use different visual styles for hovering vs dragging
            let (separator_char, color) = if app.is_dragging_separator {
                ("▐", Color::Yellow) // Bright yellow when actively dragging
            } else {
                ("┃", Color::LightCyan) // Subtle cyan when just hovering
            };

            let separator_widget = Paragraph::new(separator_char).style(Style::default().fg(color));
            f.render_widget(separator_widget, separator_area);
        }
    }
}

/// Render the save as dialog
pub fn render_save_as_dialog(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::Clear;

    // Calculate dialog size and position (centered)
    let dialog_width = 60;
    let dialog_height = 6;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background for the dialog
    f.render_widget(Clear, dialog_area);

    // Create the dialog block
    let block = Block::default()
        .title(" Save As ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    // Create the dialog content with text input
    let input_display = if app.save_as_input == ".pad" {
        "[filename].pad".to_string()
    } else {
        app.save_as_input.clone()
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Filename: ", Style::default().fg(Color::White)),
            Span::styled(
                input_display,
                if app.save_as_input == ".pad" {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Yellow).bg(Color::Blue)
                },
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::styled(" - Save    ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::styled(" - Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog_area);
}

/// Render the welcome screen dialog showing changelog for new version
pub fn render_welcome_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Get the actual version information
    let changelog_content = crate::version::get_changelog_since_version().unwrap_or_else(|| {
        "Welcome to mathypad!\n\nThis appears to be your first time running this version."
            .to_string()
    });

    let current_version = crate::version::get_current_version();
    let stored_version = crate::version::get_stored_version();

    render_welcome_dialog_with_content(
        f,
        app,
        area,
        &changelog_content,
        current_version,
        stored_version.as_deref(),
    )
}

/// Render the welcome screen dialog with specific content (for testing)
pub fn render_welcome_dialog_with_content(
    f: &mut Frame,
    app: &App,
    area: Rect,
    changelog_content: &str,
    current_version: &str,
    stored_version: Option<&str>,
) {
    use ratatui::widgets::Clear;

    // Calculate dialog size and position (larger than other dialogs for changelog content)
    let dialog_width = 100.min(area.width.saturating_sub(4));
    let dialog_height = 25.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background
    f.render_widget(Clear, dialog_area);

    let is_first_run = stored_version.is_none();

    // Create the welcome dialog
    let block = Block::default()
        .title(format!(" Welcome to mathypad v{} ", current_version))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    // Prepare the content lines
    let header_lines = if is_first_run {
        vec![
            Line::from(vec![
                Span::styled("Welcome to mathypad! ", Style::default().fg(Color::Green)),
                Span::styled(
                    "Thank you for trying it out.",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "What's in this version:",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(""),
        ]
    } else {
        let stored = stored_version.unwrap_or("unknown");
        vec![
            Line::from(vec![
                Span::styled("Welcome! ", Style::default().fg(Color::Green)),
                Span::styled(
                    format!("You've updated from v{} to v{}", stored, current_version),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "What's new:",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(""),
        ]
    };

    // Split changelog into lines and apply scroll offset
    let changelog_lines: Vec<Line> = changelog_content
        .lines()
        .map(|line| {
            if line.starts_with("## ") {
                // Version headers in bright yellow
                Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
            } else if line.starts_with("### ") {
                // Section headers in cyan
                Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
            } else if line.starts_with("- ") {
                // Bullet points in green
                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
            } else {
                // Regular text
                Line::from(Span::styled(line, Style::default().fg(Color::White)))
            }
        })
        .collect();

    // Combine header and changelog lines
    let mut all_lines = header_lines;
    all_lines.extend(changelog_lines);

    // Calculate layout: reserve space for footer (3 lines: empty, instructions, scroll indicator)
    let inner_area = block.inner(dialog_area);
    let footer_height = 3; // Empty line + instructions + scroll indicator
    let content_height = inner_area.height as usize;
    let scrollable_height = content_height.saturating_sub(footer_height);

    // Apply scroll offset
    let total_lines = all_lines.len();
    let max_scroll = total_lines.saturating_sub(scrollable_height);
    let scroll_offset = app.welcome_scroll_offset.min(max_scroll);

    let visible_lines: Vec<Line> = all_lines
        .into_iter()
        .skip(scroll_offset)
        .take(scrollable_height)
        .collect();

    // Create content area (excludes footer)
    let content_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: scrollable_height as u16,
    };

    // Create footer area (bottom 3 lines)
    let footer_area = Rect {
        x: inner_area.x,
        y: inner_area.y + scrollable_height as u16,
        width: inner_area.width,
        height: footer_height as u16,
    };

    // Render the main dialog block
    f.render_widget(block, dialog_area);

    // Render scrollable content
    let content_paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    f.render_widget(content_paragraph, content_area);

    // Render footer with instructions (always visible at bottom)
    let footer_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::styled(" scroll  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::styled(" or ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::styled(" close", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![if total_lines > scrollable_height {
            Span::styled(
                format!("({}/{})", scroll_offset + 1, max_scroll + 1),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::raw("")
        }]),
    ];

    let footer_paragraph = Paragraph::new(footer_lines).wrap(Wrap { trim: false });
    f.render_widget(footer_paragraph, footer_area);

    // Render scrollbar if there's content to scroll
    if total_lines > scrollable_height {
        render_scrollbar(
            f,
            dialog_area,
            scroll_offset,
            total_lines,
            scrollable_height,
        );
    }
}

/// Render a scrollbar on the right side of a dialog
fn render_scrollbar(
    f: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    total_lines: usize,
    visible_height: usize,
) {
    // Calculate scrollbar dimensions
    let scrollbar_x = area.x + area.width - 1; // Right edge of the dialog
    let scrollbar_y = area.y + 1; // Start below the top border
    let scrollbar_height = area.height.saturating_sub(2); // Exclude top and bottom borders

    if scrollbar_height == 0 {
        return;
    }

    // Calculate scroll thumb position and size
    let content_height = total_lines.max(1);
    let thumb_size =
        ((visible_height as f32 / content_height as f32) * scrollbar_height as f32).max(1.0) as u16;
    let max_thumb_position = scrollbar_height.saturating_sub(thumb_size);

    let thumb_position = if content_height <= visible_height {
        0
    } else {
        let scroll_ratio = scroll_offset as f32 / (content_height - visible_height) as f32;
        (scroll_ratio * max_thumb_position as f32) as u16
    };

    // Draw the scrollbar track (background)
    for y in 0..scrollbar_height {
        let track_area = Rect {
            x: scrollbar_x,
            y: scrollbar_y + y,
            width: 1,
            height: 1,
        };

        let track_char = if y >= thumb_position && y < thumb_position + thumb_size {
            "█" // Solid block for thumb
        } else {
            "░" // Light shade for track
        };

        let track_color = if y >= thumb_position && y < thumb_position + thumb_size {
            Color::Cyan // Bright color for thumb
        } else {
            Color::DarkGray // Subtle color for track
        };

        let scrollbar_widget = Paragraph::new(track_char).style(Style::default().fg(track_color));
        f.render_widget(scrollbar_widget, track_area);
    }
}

/// Render the command line at the bottom of the screen
pub fn render_command_line(f: &mut Frame, app: &App, area: Rect) {
    // Create spans for the command line with cursor highlighting
    let mut spans = Vec::new();

    // Add the command line text with cursor highlighting
    let chars: Vec<char> = app.command_line.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i == app.command_cursor {
            // Highlight cursor position
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(Color::White).fg(Color::Black),
            ));
        } else {
            // Normal character
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(Color::White),
            ));
        }
    }

    // If cursor is at the end, add a space with cursor highlighting
    if app.command_cursor >= chars.len() {
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::White).fg(Color::Black),
        ));
    }

    // Create the command line paragraph
    let command_line =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black).fg(Color::White));

    f.render_widget(command_line, area);
}
