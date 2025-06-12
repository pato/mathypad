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

/// Main UI layout and rendering
pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(f.area());

    render_text_area(f, app, chunks[0]);
    render_results_panel(f, app, chunks[1]);
}

/// Render the main text editing area
pub fn render_text_area(f: &mut Frame, app: &App, area: Rect) {
    let title = "Mathypad";
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
            let (before_cursor, after_cursor) = line_text.split_at(app.cursor_col);
            spans.extend(parse_colors(before_cursor, &app.variables));
            spans.push(Span::styled("█", Style::default().fg(Color::White)));
            spans.extend(parse_colors(after_cursor, &app.variables));
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
            spans.push(Span::styled(
                value.clone(),
                Style::default().fg(Color::Green),
            ));
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
            } else if word_text.to_lowercase() == "to" || word_text.to_lowercase() == "in" || word_text.to_lowercase() == "of" {
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
