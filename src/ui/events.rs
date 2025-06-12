//! Event handling and main TUI loop

use super::render::ui;
use crate::{App, Mode, TICK_RATE_MS};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    error::Error,
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

/// Run the interactive TUI mode
pub fn run_interactive_mode() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char('w')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            if app.mode == Mode::Insert {
                                app.delete_word();
                            }
                        }
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                        }
                        _ => match app.mode {
                            Mode::Insert => {
                                handle_insert_mode(&mut app, key.code);
                            }
                            Mode::Normal => {
                                handle_normal_mode(&mut app, key.code);
                            }
                        },
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Run the interactive TUI mode with an optional file to load
pub fn run_interactive_mode_with_file(file_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = if let Some(path) = file_path {
        load_app_from_file(path)?
    } else {
        App::default()
    };

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char('w')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            if app.mode == Mode::Insert {
                                app.delete_word();
                            }
                        }
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                        }
                        _ => match app.mode {
                            Mode::Insert => {
                                handle_insert_mode(&mut app, key.code);
                            }
                            Mode::Normal => {
                                handle_normal_mode(&mut app, key.code);
                            }
                        },
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Load an App from a file
fn load_app_from_file(path: PathBuf) -> Result<App, Box<dyn Error>> {
    let contents = fs::read_to_string(&path)?;
    let mut app = App::default();

    // Clear the default empty line if we have file content
    if !contents.trim().is_empty() {
        app.text_lines.clear();
        app.results.clear();
    }

    // Split the contents into lines and load them into the app
    for line in contents.lines() {
        app.text_lines.push(line.to_string());
        app.results.push(None);
    }

    // If the file is empty, ensure we have at least one empty line
    if app.text_lines.is_empty() {
        app.text_lines.push(String::new());
        app.results.push(None);
    }

    // Recalculate all lines
    app.recalculate_all();

    Ok(app)
}

/// Handle key events in insert mode
fn handle_insert_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => {
            app.insert_char(c);
        }
        KeyCode::Enter => {
            app.new_line();
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        KeyCode::Up => {
            app.move_cursor_up();
        }
        KeyCode::Down => {
            app.move_cursor_down();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        _ => {}
    }
}

/// Handle key events in normal mode (vim-like)
fn handle_normal_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('h') => {
            app.move_cursor_left();
        }
        KeyCode::Char('j') => {
            app.move_cursor_down();
        }
        KeyCode::Char('k') => {
            app.move_cursor_up();
        }
        KeyCode::Char('l') => {
            app.move_cursor_right();
        }
        KeyCode::Char('i') => {
            app.mode = Mode::Insert;
        }
        KeyCode::Char('a') => {
            app.move_cursor_right();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('A') => {
            // Move to end of line
            if app.cursor_line < app.text_lines.len() {
                app.cursor_col = app.text_lines[app.cursor_line].len();
            }
            app.mode = Mode::Insert;
        }
        KeyCode::Char('I') => {
            app.cursor_col = 0;
            app.mode = Mode::Insert;
        }
        KeyCode::Char('o') => {
            // Insert new line below and enter insert mode
            if app.cursor_line < app.text_lines.len() {
                app.cursor_col = app.text_lines[app.cursor_line].len();
            }
            app.new_line();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('O') => {
            // Insert new line above and enter insert mode
            app.text_lines.insert(app.cursor_line, String::new());
            app.results.insert(app.cursor_line, None);
            app.cursor_col = 0;
            app.mode = Mode::Insert;
        }
        // Allow arrow keys in normal mode too
        KeyCode::Up => {
            app.move_cursor_up();
        }
        KeyCode::Down => {
            app.move_cursor_down();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        _ => {}
    }
}
