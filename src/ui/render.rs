//! UI rendering functions

use crate::expression::parse_line_reference;
use crate::units::parse_unit;
use crate::{App, Mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::collections::HashMap;

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

/// Main UI layout and rendering
pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(f.area());

    render_text_area(f, app, chunks[0]);
    render_results_panel(f, app, chunks[1]);

    // Render dialogs on top if needed
    if app.show_unsaved_dialog {
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
    };

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let visible_height = inner_area.height as usize;
    let start_line = app.scroll_offset;
    let end_line = (start_line + visible_height).min(app.text_lines.len());

    let mut lines = Vec::new();
    for (i, line_text) in app.text_lines[start_line..end_line].iter().enumerate() {
        let line_num = start_line + i + 1;
        let line_num_str = format!("{:4} ", line_num);

        let mut spans = vec![Span::styled(
            line_num_str,
            Style::default().fg(Color::DarkGray),
        )];

        if start_line + i == app.cursor_line {
            // Parse with cursor highlighting
            spans.extend(parse_colors_with_cursor(
                line_text,
                app.cursor_col,
                &app.variables,
            ));
        } else {
            spans.extend(parse_colors(line_text, &app.variables));
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
    let end_line = (start_line + visible_height).min(app.results.len());

    let mut lines = Vec::new();
    for (i, result) in app.results[start_line..end_line].iter().enumerate() {
        let line_num = start_line + i + 1;
        let line_num_str = format!("{:4} ", line_num);

        let mut spans = vec![Span::styled(
            line_num_str,
            Style::default().fg(Color::DarkGray),
        )];

        if let Some(value) = result {
            // Get animation state for this line
            let line_index = start_line + i;
            let color = if let Some(animation) = app.get_result_animation(line_index) {
                // Apply fade-in animation by adjusting color intensity
                let opacity = animation.opacity();
                animate_color(Color::Green, opacity)
            } else {
                Color::Green
            };

            spans.push(Span::styled(value.clone(), Style::default().fg(color)));
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner_area);
}

/// Parse text and return colored spans for syntax highlighting
pub fn parse_colors<'a>(text: &'a str, variables: &'a HashMap<String, String>) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while current_pos < chars.len() {
        if chars[current_pos].is_ascii_alphabetic() {
            // Handle potential units, keywords, and line references first
            let start_pos = current_pos;

            while current_pos < chars.len()
                && (chars[current_pos].is_ascii_alphabetic()
                    || chars[current_pos].is_ascii_digit()
                    || chars[current_pos] == '/')
            {
                current_pos += 1;
            }

            let word_text: String = chars[start_pos..current_pos].iter().collect();

            // Check if it's a valid unit, keyword, line reference, or variable
            if parse_line_reference(&word_text).is_some() {
                spans.push(Span::styled(word_text, Style::default().fg(Color::Magenta)));
            } else if word_text.to_lowercase() == "to"
                || word_text.to_lowercase() == "in"
                || word_text.to_lowercase() == "of"
            {
                spans.push(Span::styled(word_text, Style::default().fg(Color::Yellow)));
            } else if parse_unit(&word_text).is_some() {
                spans.push(Span::styled(word_text, Style::default().fg(Color::Green)));
            } else if variables.contains_key(&word_text) {
                // Highlight variables that are defined
                spans.push(Span::styled(
                    word_text,
                    Style::default().fg(Color::LightCyan),
                ));
            } else {
                spans.push(Span::raw(word_text));
            }
        } else if chars[current_pos].is_ascii_digit() || chars[current_pos] == '.' {
            // Handle numbers
            let start_pos = current_pos;
            let mut has_digit = false;
            let mut has_dot = false;

            while current_pos < chars.len() {
                let ch = chars[current_pos];
                if ch.is_ascii_digit() {
                    has_digit = true;
                    current_pos += 1;
                } else if ch == '.' && !has_dot {
                    has_dot = true;
                    current_pos += 1;
                } else if ch == ',' {
                    current_pos += 1;
                } else {
                    break;
                }
            }

            if has_digit {
                let number_text: String = chars[start_pos..current_pos].iter().collect();
                spans.push(Span::styled(
                    number_text,
                    Style::default().fg(Color::LightBlue),
                ));
            } else {
                spans.push(Span::raw(chars[start_pos].to_string()));
                current_pos = start_pos + 1;
            }
        } else if chars[current_pos] == '%' {
            // Handle percentage symbol as a unit
            spans.push(Span::styled(
                "%".to_string(),
                Style::default().fg(Color::Green),
            ));
            current_pos += 1;
        } else if "+-*/()=".contains(chars[current_pos]) {
            // Handle operators (including assignment)
            spans.push(Span::styled(
                chars[current_pos].to_string(),
                Style::default().fg(Color::Cyan),
            ));
            current_pos += 1;
        } else {
            // Handle other characters
            spans.push(Span::raw(chars[current_pos].to_string()));
            current_pos += 1;
        }
    }

    spans
}

/// Parse text and return colored spans with cursor highlighting
pub fn parse_colors_with_cursor<'a>(
    text: &'a str,
    cursor_col: usize,
    variables: &'a HashMap<String, String>,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut char_index = 0; // Track character position for cursor

    while current_pos < chars.len() {
        if chars[current_pos].is_ascii_alphabetic() {
            // Handle potential units, keywords, and line references first
            let start_pos = current_pos;
            let start_char_index = char_index;

            while current_pos < chars.len()
                && (chars[current_pos].is_ascii_alphabetic()
                    || chars[current_pos].is_ascii_digit()
                    || chars[current_pos] == '/')
            {
                current_pos += 1;
                char_index += 1;
            }

            let word_text: String = chars[start_pos..current_pos].iter().collect();

            // Determine the style for this word
            let style = if parse_line_reference(&word_text).is_some() {
                Style::default().fg(Color::Magenta)
            } else if word_text.to_lowercase() == "to"
                || word_text.to_lowercase() == "in"
                || word_text.to_lowercase() == "of"
            {
                Style::default().fg(Color::Yellow)
            } else if parse_unit(&word_text).is_some() {
                Style::default().fg(Color::Green)
            } else if variables.contains_key(&word_text) {
                Style::default().fg(Color::LightCyan)
            } else {
                Style::default()
            };

            // Check if cursor is within this word
            if cursor_col >= start_char_index && cursor_col < char_index {
                // Split the word to highlight the cursor character
                let cursor_offset = cursor_col - start_char_index;
                let word_chars: Vec<char> = word_text.chars().collect();

                if cursor_offset > 0 {
                    let before: String = word_chars[..cursor_offset].iter().collect();
                    spans.push(Span::styled(before, style));
                }

                let cursor_char = word_chars[cursor_offset];
                spans.push(Span::styled(
                    cursor_char.to_string(),
                    style.bg(Color::White).fg(Color::Black),
                ));

                if cursor_offset + 1 < word_chars.len() {
                    let after: String = word_chars[cursor_offset + 1..].iter().collect();
                    spans.push(Span::styled(after, style));
                }
            } else {
                spans.push(Span::styled(word_text, style));
            }
        } else if chars[current_pos].is_ascii_digit() || chars[current_pos] == '.' {
            // Handle numbers
            let start_pos = current_pos;
            let start_char_index = char_index;
            let mut has_digit = false;
            let mut has_dot = false;

            while current_pos < chars.len() {
                let ch = chars[current_pos];
                if ch.is_ascii_digit() {
                    has_digit = true;
                    current_pos += 1;
                    char_index += 1;
                } else if ch == '.' && !has_dot {
                    has_dot = true;
                    current_pos += 1;
                    char_index += 1;
                } else if ch == ',' {
                    current_pos += 1;
                    char_index += 1;
                } else {
                    break;
                }
            }

            if has_digit {
                let number_text: String = chars[start_pos..current_pos].iter().collect();
                let style = Style::default().fg(Color::LightBlue);

                // Check if cursor is within this number
                if cursor_col >= start_char_index && cursor_col < char_index {
                    let cursor_offset = cursor_col - start_char_index;
                    let number_chars: Vec<char> = number_text.chars().collect();

                    if cursor_offset > 0 {
                        let before: String = number_chars[..cursor_offset].iter().collect();
                        spans.push(Span::styled(before, style));
                    }

                    let cursor_char = number_chars[cursor_offset];
                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        style.bg(Color::White).fg(Color::Black),
                    ));

                    if cursor_offset + 1 < number_chars.len() {
                        let after: String = number_chars[cursor_offset + 1..].iter().collect();
                        spans.push(Span::styled(after, style));
                    }
                } else {
                    spans.push(Span::styled(number_text, style));
                }
            } else {
                let ch = chars[start_pos];
                if cursor_col == char_index {
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().bg(Color::White).fg(Color::Black),
                    ));
                } else {
                    spans.push(Span::raw(ch.to_string()));
                }
                current_pos = start_pos + 1;
                char_index += 1;
            }
        } else {
            // Handle single characters (operators, punctuation, etc.)
            let ch = chars[current_pos];
            let style = if ch == '%' {
                Style::default().fg(Color::Green)
            } else if "+-*/()=".contains(ch) {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            if cursor_col == char_index {
                spans.push(Span::styled(
                    ch.to_string(),
                    style.bg(Color::White).fg(Color::Black),
                ));
            } else {
                spans.push(Span::styled(ch.to_string(), style));
            }

            current_pos += 1;
            char_index += 1;
        }
    }

    // If cursor is at the end of the line, add a space with cursor background
    if cursor_col == char_index {
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
            Span::styled("Ctrl+Q", Style::default().fg(Color::Red)),
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
