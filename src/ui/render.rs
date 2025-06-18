//! UI rendering functions

use crate::expression::{Token, TokenWithSpan, parse_expression_for_highlighting};
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

/// Helper function to split a number-with-unit token and color the parts differently with cursor support
fn split_and_color_number_with_unit_with_cursor<'a>(
    token_text: &'a str,
    spans: &mut Vec<Span<'a>>,
    cursor_offset: Option<usize>,
    base_style_number: Style,
    base_style_unit: Style,
) {
    // Find where the unit starts by finding the first alphabetic character or %
    let chars: Vec<char> = token_text.chars().collect();
    let mut unit_start = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_alphabetic() || ch == '%' {
            unit_start = i;
            break;
        }
    }

    if unit_start < chars.len() {
        // Split into number part and unit part
        let number_part = token_text[..unit_start].trim_end();
        let unit_part = &token_text[unit_start..];

        // Handle number part with potential cursor
        if !number_part.is_empty() {
            if let Some(cursor_offset) = cursor_offset {
                if cursor_offset < number_part.len() {
                    // Cursor is in the number part
                    let number_chars: Vec<char> = number_part.chars().collect();

                    if cursor_offset > 0 {
                        let before_cursor: String = number_chars[..cursor_offset].iter().collect();
                        spans.push(Span::styled(before_cursor, base_style_number));
                    }

                    let cursor_char = number_chars[cursor_offset];
                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        base_style_number.bg(Color::White).fg(Color::Black),
                    ));

                    if cursor_offset + 1 < number_chars.len() {
                        let after_cursor: String =
                            number_chars[cursor_offset + 1..].iter().collect();
                        spans.push(Span::styled(after_cursor, base_style_number));
                    }
                } else {
                    // Cursor is not in number part, color normally
                    spans.push(Span::styled(number_part, base_style_number));
                }
            } else {
                // No cursor, color normally
                spans.push(Span::styled(number_part, base_style_number));
            }
        }

        // Add any whitespace between number and unit
        if unit_start > number_part.len() {
            let whitespace = &token_text[number_part.len()..unit_start];
            if let Some(cursor_offset) = cursor_offset {
                if cursor_offset >= number_part.len() && cursor_offset < unit_start {
                    // Cursor is in the whitespace
                    let ws_cursor_offset = cursor_offset - number_part.len();
                    let ws_chars: Vec<char> = whitespace.chars().collect();

                    if ws_cursor_offset > 0 {
                        let before_cursor: String = ws_chars[..ws_cursor_offset].iter().collect();
                        spans.push(Span::raw(before_cursor));
                    }

                    if ws_cursor_offset < ws_chars.len() {
                        let cursor_char = ws_chars[ws_cursor_offset];
                        spans.push(Span::styled(
                            cursor_char.to_string(),
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));

                        if ws_cursor_offset + 1 < ws_chars.len() {
                            let after_cursor: String =
                                ws_chars[ws_cursor_offset + 1..].iter().collect();
                            spans.push(Span::raw(after_cursor));
                        }
                    }
                } else {
                    spans.push(Span::raw(whitespace));
                }
            } else {
                spans.push(Span::raw(whitespace));
            }
        }

        // Handle unit part with potential cursor
        if let Some(cursor_offset) = cursor_offset {
            if cursor_offset >= unit_start {
                // Cursor is in the unit part
                let unit_cursor_offset = cursor_offset - unit_start;
                let unit_chars: Vec<char> = unit_part.chars().collect();

                if unit_cursor_offset > 0 && unit_cursor_offset <= unit_chars.len() {
                    let before_cursor: String = unit_chars
                        [..unit_cursor_offset.min(unit_chars.len())]
                        .iter()
                        .collect();
                    spans.push(Span::styled(before_cursor, base_style_unit));
                }

                if unit_cursor_offset < unit_chars.len() {
                    let cursor_char = unit_chars[unit_cursor_offset];
                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        base_style_unit.bg(Color::White).fg(Color::Black),
                    ));

                    if unit_cursor_offset + 1 < unit_chars.len() {
                        let after_cursor: String =
                            unit_chars[unit_cursor_offset + 1..].iter().collect();
                        spans.push(Span::styled(after_cursor, base_style_unit));
                    }
                }
            } else {
                // Cursor is not in unit part, color normally
                spans.push(Span::styled(unit_part, base_style_unit));
            }
        } else {
            // No cursor, color normally
            spans.push(Span::styled(unit_part, base_style_unit));
        }
    } else {
        // No unit found, treat as number with potential cursor
        if let Some(cursor_offset) = cursor_offset {
            let token_chars: Vec<char> = token_text.chars().collect();

            if cursor_offset > 0 {
                let before_cursor: String = token_chars[..cursor_offset].iter().collect();
                spans.push(Span::styled(before_cursor, base_style_number));
            }

            if cursor_offset < token_chars.len() {
                let cursor_char = token_chars[cursor_offset];
                spans.push(Span::styled(
                    cursor_char.to_string(),
                    base_style_number.bg(Color::White).fg(Color::Black),
                ));

                if cursor_offset + 1 < token_chars.len() {
                    let after_cursor: String = token_chars[cursor_offset + 1..].iter().collect();
                    spans.push(Span::styled(after_cursor, base_style_number));
                }
            }
        } else {
            spans.push(Span::styled(token_text, base_style_number));
        }
    }
}

/// Helper function to split a number-with-unit token and color the parts differently
fn split_and_color_number_with_unit<'a>(token_text: &'a str, spans: &mut Vec<Span<'a>>) {
    // Find where the unit starts by finding the first alphabetic character or %
    let chars: Vec<char> = token_text.chars().collect();
    let mut unit_start = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_alphabetic() || ch == '%' {
            unit_start = i;
            break;
        }
    }

    if unit_start < chars.len() {
        // Split into number part and unit part
        let number_part = token_text[..unit_start].trim_end();
        let unit_part = &token_text[unit_start..];

        // Color number part as LightBlue
        if !number_part.is_empty() {
            spans.push(Span::styled(
                number_part,
                Style::default().fg(Color::LightBlue),
            ));
        }

        // Add any whitespace between number and unit
        if unit_start > number_part.len() {
            let whitespace = &token_text[number_part.len()..unit_start];
            spans.push(Span::raw(whitespace));
        }

        // Color unit part as Green
        spans.push(Span::styled(unit_part, Style::default().fg(Color::Green)));
    } else {
        // No unit found, color everything as number
        spans.push(Span::styled(
            token_text,
            Style::default().fg(Color::LightBlue),
        ));
    }
}

/// Convert tokens with spans to colored spans using Chumsky parser
fn tokens_to_colored_spans<'a>(
    text: &'a str,
    tokens: &[TokenWithSpan],
    variables: &HashMap<String, String>,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;

    for token_with_span in tokens {
        let start = token_with_span.start();
        let end = token_with_span.end();

        // Add any text before this token as unstyled
        if current_pos < start {
            spans.push(Span::raw(&text[current_pos..start]));
        }

        // Add the token with appropriate styling
        let token_text = &text[start..end];
        match &token_with_span.token {
            Token::Number(_) => {
                spans.push(Span::styled(
                    token_text,
                    Style::default().fg(Color::LightBlue),
                ));
            }
            Token::NumberWithUnit(_, _) => {
                // Legacy NumberWithUnit tokens (still used in some places)
                // Split NumberWithUnit to color number and unit parts differently
                split_and_color_number_with_unit(token_text, &mut spans);
            }
            Token::Plus
            | Token::Minus
            | Token::Multiply
            | Token::Divide
            | Token::LeftParen
            | Token::RightParen
            | Token::Assign => {
                spans.push(Span::styled(token_text, Style::default().fg(Color::Cyan)));
            }
            Token::To | Token::In | Token::Of => {
                spans.push(Span::styled(token_text, Style::default().fg(Color::Yellow)));
            }
            Token::LineReference(_) => {
                spans.push(Span::styled(
                    token_text,
                    Style::default().fg(Color::Magenta),
                ));
            }
            Token::Variable(var_name) => {
                let style = if variables.contains_key(var_name) {
                    Style::default().fg(Color::LightCyan)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(token_text, style));
            }
            Token::Unit(_) => {
                // Units should be colored green
                spans.push(Span::styled(token_text, Style::default().fg(Color::Green)));
            }
        }
        current_pos = end;
    }

    // Add any remaining text as unstyled
    if current_pos < text.len() {
        spans.push(Span::raw(&text[current_pos..]));
    }

    spans
}

/// Parse text and return colored spans for syntax highlighting
pub fn parse_colors<'a>(text: &'a str, variables: &'a HashMap<String, String>) -> Vec<Span<'a>> {
    // Use the Chumsky parser to get tokens with spans
    let tokens = parse_expression_for_highlighting(text);

    // Convert tokens to colored spans
    tokens_to_colored_spans(text, &tokens, variables)
}

/// Convert tokens with spans to colored spans with cursor highlighting
fn tokens_to_colored_spans_with_cursor<'a>(
    text: &'a str,
    tokens: &[TokenWithSpan],
    cursor_col: usize,
    variables: &HashMap<String, String>,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;

    for token_with_span in tokens {
        let start = token_with_span.start();
        let end = token_with_span.end();

        // Add any text before this token as unstyled (with potential cursor highlighting)
        if current_pos < start {
            let before_text = &text[current_pos..start];
            if cursor_col >= current_pos && cursor_col < start {
                // Cursor is in the text before this token
                let cursor_offset = cursor_col - current_pos;
                let chars: Vec<char> = before_text.chars().collect();

                if cursor_offset > 0 {
                    let before_cursor: String = chars[..cursor_offset].iter().collect();
                    spans.push(Span::raw(before_cursor));
                }

                if cursor_offset < chars.len() {
                    let cursor_char = chars[cursor_offset];
                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        Style::default().bg(Color::White).fg(Color::Black),
                    ));

                    if cursor_offset + 1 < chars.len() {
                        let after_cursor: String = chars[cursor_offset + 1..].iter().collect();
                        spans.push(Span::raw(after_cursor));
                    }
                } else {
                    // Cursor is at the end of this text segment
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(Color::White).fg(Color::Black),
                    ));
                }
            } else {
                spans.push(Span::raw(before_text));
            }
        }

        // Add the token with appropriate styling (and potential cursor highlighting)
        let token_text = &text[start..end];
        let cursor_offset = if cursor_col >= start && cursor_col < end {
            Some(cursor_col - start)
        } else {
            None
        };

        match &token_with_span.token {
            Token::Number(_) => {
                let base_style = Style::default().fg(Color::LightBlue);
                if let Some(cursor_offset) = cursor_offset {
                    let chars: Vec<char> = token_text.chars().collect();

                    if cursor_offset > 0 {
                        let before_cursor: String = chars[..cursor_offset].iter().collect();
                        spans.push(Span::styled(before_cursor, base_style));
                    }

                    let cursor_char = chars[cursor_offset];
                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        base_style.bg(Color::White).fg(Color::Black),
                    ));

                    if cursor_offset + 1 < chars.len() {
                        let after_cursor: String = chars[cursor_offset + 1..].iter().collect();
                        spans.push(Span::styled(after_cursor, base_style));
                    }
                } else {
                    spans.push(Span::styled(token_text, base_style));
                }
            }
            Token::NumberWithUnit(_, _) => {
                // Legacy NumberWithUnit tokens (still used in some places)
                // Split NumberWithUnit to color number and unit parts differently with cursor support
                split_and_color_number_with_unit_with_cursor(
                    token_text,
                    &mut spans,
                    cursor_offset,
                    Style::default().fg(Color::LightBlue), // Number style
                    Style::default().fg(Color::Green),     // Unit style
                );
            }
            _ => {
                let base_style = match &token_with_span.token {
                    Token::Plus
                    | Token::Minus
                    | Token::Multiply
                    | Token::Divide
                    | Token::LeftParen
                    | Token::RightParen
                    | Token::Assign => Style::default().fg(Color::Cyan),
                    Token::To | Token::In | Token::Of => Style::default().fg(Color::Yellow),
                    Token::LineReference(_) => Style::default().fg(Color::Magenta),
                    Token::Variable(var_name) => {
                        if variables.contains_key(var_name) {
                            Style::default().fg(Color::LightCyan)
                        } else {
                            Style::default()
                        }
                    }
                    Token::Unit(_) => Style::default().fg(Color::Green),
                    _ => Style::default(),
                };

                if let Some(cursor_offset) = cursor_offset {
                    let chars: Vec<char> = token_text.chars().collect();

                    if cursor_offset > 0 {
                        let before_cursor: String = chars[..cursor_offset].iter().collect();
                        spans.push(Span::styled(before_cursor, base_style));
                    }

                    let cursor_char = chars[cursor_offset];
                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        base_style.bg(Color::White).fg(Color::Black),
                    ));

                    if cursor_offset + 1 < chars.len() {
                        let after_cursor: String = chars[cursor_offset + 1..].iter().collect();
                        spans.push(Span::styled(after_cursor, base_style));
                    }
                } else {
                    spans.push(Span::styled(token_text, base_style));
                }
            }
        }

        current_pos = end;
    }

    // Add any remaining text as unstyled (with potential cursor highlighting)
    if current_pos < text.len() {
        let remaining_text = &text[current_pos..];
        if cursor_col >= current_pos {
            let cursor_offset = cursor_col - current_pos;
            let chars: Vec<char> = remaining_text.chars().collect();

            if cursor_offset > 0 && cursor_offset <= chars.len() {
                let before_cursor: String =
                    chars[..cursor_offset.min(chars.len())].iter().collect();
                spans.push(Span::raw(before_cursor));
            }

            if cursor_offset < chars.len() {
                let cursor_char = chars[cursor_offset];
                spans.push(Span::styled(
                    cursor_char.to_string(),
                    Style::default().bg(Color::White).fg(Color::Black),
                ));

                if cursor_offset + 1 < chars.len() {
                    let after_cursor: String = chars[cursor_offset + 1..].iter().collect();
                    spans.push(Span::raw(after_cursor));
                }
            } else if cursor_offset == chars.len() {
                // Cursor is at the end of the line
                spans.push(Span::styled(
                    " ",
                    Style::default().bg(Color::White).fg(Color::Black),
                ));
            }
        } else {
            spans.push(Span::raw(remaining_text));
        }
    } else if cursor_col == current_pos {
        // Cursor is at the end of the line after all tokens
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::White).fg(Color::Black),
        ));
    }

    spans
}

/// Parse text and return colored spans with cursor highlighting
pub fn parse_colors_with_cursor<'a>(
    text: &'a str,
    cursor_col: usize,
    variables: &'a HashMap<String, String>,
) -> Vec<Span<'a>> {
    // Use the Chumsky parser to get tokens with spans
    let tokens = parse_expression_for_highlighting(text);

    // Convert tokens to colored spans with cursor highlighting
    tokens_to_colored_spans_with_cursor(text, &tokens, cursor_col, variables)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_token_rendering() {
        use std::collections::HashMap;

        // Test that separate Number and Unit tokens are colored correctly
        let variables = HashMap::new();
        let spans = parse_colors("5 GiB + 100 MB", &variables);

        // Should have spans for: Number(5), Unit(GiB), Plus, Number(100), Unit(MB)
        // Plus any whitespace spans in between
        assert!(spans.len() >= 5);

        // Find the spans by their text content and verify colors
        let mut found_number = false;
        let mut found_unit = false;

        for span in &spans {
            match span.content.as_ref() {
                "5" => {
                    // Numbers should be light blue
                    found_number = true;
                    // Note: we can't easily test the exact color here due to ratatui's design
                    // but the test ensures the parsing works correctly
                }
                "GiB" | "MB" => {
                    // Units should be green
                    found_unit = true;
                }
                _ => {}
            }
        }

        assert!(
            found_number && found_unit,
            "Expected to find both number and unit spans"
        );
    }
}
