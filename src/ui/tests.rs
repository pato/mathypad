//! UI snapshot tests using insta and ratatui TestBackend

use super::*;
use crate::{App, Mode};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};

/// Helper function to create a test terminal with a fixed size
fn create_test_terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(120, 30)).unwrap()
}

/// Helper function to render app and return terminal output
fn render_app_to_string(app: &App) -> String {
    let mut terminal = create_test_terminal();
    terminal.draw(|frame| ui(frame, app)).unwrap();
    format!("{}", terminal.backend())
}

/// Helper function to create an app with some sample content
fn create_sample_app() -> App {
    let mut app = App::default();
    app.text_lines = vec![
        "5 + 3".to_string(),
        "10 kg to lb".to_string(),
        "line1 * 2".to_string(),
        "sin(30 degrees)".to_string(),
    ];
    app.results = vec![
        Some("8".to_string()),
        Some("22.046 lb".to_string()),
        Some("16".to_string()),
        Some("0.5".to_string()),
    ];
    app.cursor_line = 1;
    app.cursor_col = 5;

    // Add some variables
    app.variables.insert("x".to_string(), "42".to_string());
    app.variables.insert("rate".to_string(), "5.5".to_string());

    app
}

#[test]
fn test_basic_ui_layout() {
    let app = App::default();
    let output = render_app_to_string(&app);
    assert_snapshot!("basic_ui_layout", output);
}

#[test]
fn test_ui_with_content() {
    let app = create_sample_app();
    let output = render_app_to_string(&app);
    assert_snapshot!("ui_with_content", output);
}

#[test]
fn test_ui_normal_mode() {
    let mut app = create_sample_app();
    app.mode = Mode::Normal;
    let output = render_app_to_string(&app);
    assert_snapshot!("ui_normal_mode", output);
}

#[test]
fn test_ui_insert_mode() {
    let mut app = create_sample_app();
    app.mode = Mode::Insert;
    let output = render_app_to_string(&app);
    assert_snapshot!("ui_insert_mode", output);
}

#[test]
fn test_ui_with_unsaved_changes() {
    let mut app = create_sample_app();
    app.has_unsaved_changes = true;
    let output = render_app_to_string(&app);
    assert_snapshot!("ui_with_unsaved_changes", output);
}

#[test]
fn test_ui_different_separator_position() {
    let mut app = create_sample_app();
    app.separator_position = 60; // 60/40 split instead of default 80/20
    let output = render_app_to_string(&app);
    assert_snapshot!("ui_different_separator_position", output);
}

#[test]
fn test_text_area_rendering() {
    let mut terminal = create_test_terminal();
    let app = create_sample_app();

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_text_area(frame, &app, area);
        })
        .unwrap();

    let output = format!("{}", terminal.backend());
    assert_snapshot!("text_area_rendering", output);
}

#[test]
fn test_results_panel_rendering() {
    let mut terminal = create_test_terminal();
    let app = create_sample_app();

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_results_panel(frame, &app, area);
        })
        .unwrap();

    let output = format!("{}", terminal.backend());
    assert_snapshot!("results_panel_rendering", output);
}

#[test]
fn test_syntax_highlighting_numbers() {
    let mut app = App::default();
    app.text_lines = vec![
        "123".to_string(),
        "3.14159".to_string(),
        "1,234,567.89".to_string(),
    ];
    app.results = vec![None, None, None];

    let output = render_app_to_string(&app);
    assert_snapshot!("syntax_highlighting_numbers", output);
}

#[test]
fn test_syntax_highlighting_operators() {
    let mut app = App::default();
    app.text_lines = vec![
        "5 + 3 - 2".to_string(),
        "10 * (4 / 2)".to_string(),
        "x = 42".to_string(),
    ];
    app.results = vec![None, None, None];

    let output = render_app_to_string(&app);
    assert_snapshot!("syntax_highlighting_operators", output);
}

#[test]
fn test_syntax_highlighting_units() {
    let mut app = App::default();
    app.text_lines = vec![
        "100 kg".to_string(),
        "50 miles per hour".to_string(),
        "25 degrees celsius".to_string(),
        "1 GiB".to_string(),
    ];
    app.results = vec![None, None, None, None];

    let output = render_app_to_string(&app);
    assert_snapshot!("syntax_highlighting_units", output);
}

#[test]
fn test_syntax_highlighting_keywords() {
    let mut app = App::default();
    app.text_lines = vec![
        "100 kg to lb".to_string(),
        "50 miles in km".to_string(),
        "25% of 200".to_string(),
    ];
    app.results = vec![None, None, None];

    let output = render_app_to_string(&app);
    assert_snapshot!("syntax_highlighting_keywords", output);
}

#[test]
fn test_syntax_highlighting_line_references() {
    let mut app = App::default();
    app.text_lines = vec![
        "100".to_string(),
        "line1 + 50".to_string(),
        "line2 * 2".to_string(),
        "line1 + line2 + line3".to_string(),
    ];
    app.results = vec![
        Some("100".to_string()),
        Some("150".to_string()),
        Some("300".to_string()),
        Some("550".to_string()),
    ];

    let output = render_app_to_string(&app);
    assert_snapshot!("syntax_highlighting_line_references", output);
}

#[test]
fn test_syntax_highlighting_variables() {
    let mut app = App::default();
    app.text_lines = vec![
        "x = 42".to_string(),
        "y = x * 2".to_string(),
        "result = x + y".to_string(),
    ];
    app.results = vec![None, None, None];
    app.variables.insert("x".to_string(), "42".to_string());
    app.variables.insert("y".to_string(), "84".to_string());
    app.variables
        .insert("result".to_string(), "126".to_string());

    let output = render_app_to_string(&app);
    assert_snapshot!("syntax_highlighting_variables", output);
}

#[test]
fn test_cursor_highlighting() {
    let mut app = App::default();
    app.text_lines = vec!["hello world".to_string(), "123 + 456".to_string()];
    app.results = vec![None, None];
    app.cursor_line = 0;
    app.cursor_col = 6; // Position cursor on 'w' in "world"

    let output = render_app_to_string(&app);
    assert_snapshot!("cursor_highlighting", output);
}

#[test]
fn test_cursor_at_end_of_line() {
    let mut app = App::default();
    app.text_lines = vec!["hello".to_string()];
    app.results = vec![None];
    app.cursor_line = 0;
    app.cursor_col = 5; // Position cursor at end of line

    let output = render_app_to_string(&app);
    assert_snapshot!("cursor_at_end_of_line", output);
}

#[test]
fn test_scrolled_content() {
    let mut app = App::default();
    // Create more lines than fit on screen
    app.text_lines = (0..50).map(|i| format!("line {} content", i + 1)).collect();
    app.results = vec![None; 50];
    app.scroll_offset = 10; // Scroll down 10 lines
    app.cursor_line = 15;
    app.cursor_col = 5;

    let output = render_app_to_string(&app);
    assert_snapshot!("scrolled_content", output);
}

#[test]
fn test_empty_results() {
    let mut app = App::default();
    app.text_lines = vec![
        "".to_string(),
        "invalid expression +++".to_string(),
        "".to_string(),
    ];
    app.results = vec![None, None, None];

    let output = render_app_to_string(&app);
    assert_snapshot!("empty_results", output);
}

#[test]
fn test_unsaved_dialog() {
    let mut app = create_sample_app();
    app.show_unsaved_dialog = true;
    app.has_unsaved_changes = true;

    let output = render_app_to_string(&app);
    assert_snapshot!("unsaved_dialog", output);
}

#[test]
fn test_save_as_dialog() {
    let mut app = create_sample_app();
    app.show_save_as_dialog = true;
    app.save_as_input = "my_notebook.pad".to_string();

    let output = render_app_to_string(&app);
    assert_snapshot!("save_as_dialog", output);
}

#[test]
fn test_save_as_dialog_placeholder() {
    let mut app = create_sample_app();
    app.show_save_as_dialog = true;
    app.save_as_input = ".pad".to_string(); // Default placeholder

    let output = render_app_to_string(&app);
    assert_snapshot!("save_as_dialog_placeholder", output);
}

#[test]
fn test_separator_indicator_hovering() {
    let mut app = create_sample_app();
    app.is_hovering_separator = true;

    let output = render_app_to_string(&app);
    assert_snapshot!("separator_indicator_hovering", output);
}

#[test]
fn test_separator_indicator_dragging() {
    let mut app = create_sample_app();
    app.is_dragging_separator = true;

    let output = render_app_to_string(&app);
    assert_snapshot!("separator_indicator_dragging", output);
}
