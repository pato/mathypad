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

        // Check if we have active animations to determine timeout
        let has_active_animations = app
            .result_animations
            .iter()
            .any(|anim| anim.as_ref().map_or(false, |a| !a.is_complete()));

        let timeout = if has_active_animations {
            // Use a shorter timeout during animations for smooth rendering
            Duration::from_millis(16) // ~60 FPS
        } else {
            // Use normal timeout when no animations are running
            tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0))
        };

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Check if we're showing the unsaved dialog
                            if app.show_unsaved_dialog {
                                // In dialog: Ctrl+Q means quit without saving
                                break;
                            } else if app.has_unsaved_changes {
                                // Show unsaved changes dialog
                                app.show_unsaved_dialog = true;
                            } else {
                                // No unsaved changes, exit immediately
                                break;
                            }
                        }
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Check if we're showing the unsaved dialog
                            if app.show_unsaved_dialog {
                                // In dialog: Ctrl+C means quit without saving
                                break;
                            } else if app.has_unsaved_changes {
                                // Show unsaved changes dialog
                                app.show_unsaved_dialog = true;
                            } else {
                                // No unsaved changes, exit immediately
                                break;
                            }
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
                        KeyCode::Char('s')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            if app.show_save_as_dialog {
                                // In save as dialog: Ctrl+S means confirm save
                                match app.save_as_from_dialog() {
                                    Ok(should_quit) => {
                                        if should_quit {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Save failed: {}", e);
                                    }
                                }
                            } else if app.show_unsaved_dialog {
                                // In unsaved dialog: Ctrl+S means save and quit
                                if app.file_path.is_some() {
                                    if let Err(e) = app.save() {
                                        eprintln!("Save failed: {}", e);
                                    } else {
                                        // Save succeeded, exit
                                        break;
                                    }
                                } else {
                                    // No filename, show save as dialog
                                    app.show_unsaved_dialog = false;
                                    app.show_save_as_dialog(true);
                                }
                            } else {
                                // Normal save operation
                                if app.file_path.is_some() {
                                    if let Err(e) = app.save() {
                                        eprintln!("Save failed: {}", e);
                                    }
                                } else {
                                    // No filename, show save as dialog
                                    app.show_save_as_dialog(false);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if app.show_save_as_dialog {
                                // Dismiss the save as dialog
                                app.show_save_as_dialog = false;
                                app.save_as_and_quit = false;
                            } else if app.show_unsaved_dialog {
                                // Dismiss the unsaved changes dialog
                                app.show_unsaved_dialog = false;
                            } else {
                                app.mode = Mode::Normal;
                            }
                        }
                        _ => {
                            if app.show_save_as_dialog {
                                // Handle text input for save as dialog
                                if handle_save_as_input(&mut app, key.code) {
                                    break;
                                }
                            } else if !app.show_unsaved_dialog {
                                // Only handle normal input if we're not showing any dialog
                                match app.mode {
                                    Mode::Insert => {
                                        handle_insert_mode(&mut app, key.code);
                                    }
                                    Mode::Normal => {
                                        handle_normal_mode(&mut app, key.code);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate || has_active_animations {
            // Update animations on each tick or when animations are active
            app.update_animations();
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

        // Check if we have active animations to determine timeout
        let has_active_animations = app
            .result_animations
            .iter()
            .any(|anim| anim.as_ref().map_or(false, |a| !a.is_complete()));

        let timeout = if has_active_animations {
            // Use a shorter timeout during animations for smooth rendering
            Duration::from_millis(16) // ~60 FPS
        } else {
            // Use normal timeout when no animations are running
            tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0))
        };

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Check if we're showing the unsaved dialog
                            if app.show_unsaved_dialog {
                                // In dialog: Ctrl+Q means quit without saving
                                break;
                            } else if app.has_unsaved_changes {
                                // Show unsaved changes dialog
                                app.show_unsaved_dialog = true;
                            } else {
                                // No unsaved changes, exit immediately
                                break;
                            }
                        }
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Check if we're showing the unsaved dialog
                            if app.show_unsaved_dialog {
                                // In dialog: Ctrl+C means quit without saving
                                break;
                            } else if app.has_unsaved_changes {
                                // Show unsaved changes dialog
                                app.show_unsaved_dialog = true;
                            } else {
                                // No unsaved changes, exit immediately
                                break;
                            }
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
                        KeyCode::Char('s')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            if app.show_save_as_dialog {
                                // In save as dialog: Ctrl+S means confirm save
                                match app.save_as_from_dialog() {
                                    Ok(should_quit) => {
                                        if should_quit {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Save failed: {}", e);
                                    }
                                }
                            } else if app.show_unsaved_dialog {
                                // In unsaved dialog: Ctrl+S means save and quit
                                if app.file_path.is_some() {
                                    if let Err(e) = app.save() {
                                        eprintln!("Save failed: {}", e);
                                    } else {
                                        // Save succeeded, exit
                                        break;
                                    }
                                } else {
                                    // No filename, show save as dialog
                                    app.show_unsaved_dialog = false;
                                    app.show_save_as_dialog(true);
                                }
                            } else {
                                // Normal save operation
                                if app.file_path.is_some() {
                                    if let Err(e) = app.save() {
                                        eprintln!("Save failed: {}", e);
                                    }
                                } else {
                                    // No filename, show save as dialog
                                    app.show_save_as_dialog(false);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if app.show_save_as_dialog {
                                // Dismiss the save as dialog
                                app.show_save_as_dialog = false;
                                app.save_as_and_quit = false;
                            } else if app.show_unsaved_dialog {
                                // Dismiss the unsaved changes dialog
                                app.show_unsaved_dialog = false;
                            } else {
                                app.mode = Mode::Normal;
                            }
                        }
                        _ => {
                            if app.show_save_as_dialog {
                                // Handle text input for save as dialog
                                if handle_save_as_input(&mut app, key.code) {
                                    break;
                                }
                            } else if !app.show_unsaved_dialog {
                                // Only handle normal input if we're not showing any dialog
                                match app.mode {
                                    Mode::Insert => {
                                        handle_insert_mode(&mut app, key.code);
                                    }
                                    Mode::Normal => {
                                        handle_normal_mode(&mut app, key.code);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate || has_active_animations {
            // Update animations on each tick or when animations are active
            app.update_animations();
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

/// Load an App from a file, creating the file if it doesn't exist
fn load_app_from_file(path: PathBuf) -> Result<App, Box<dyn Error>> {
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // File doesn't exist, create it with empty content
            // We don't actually write the file yet - it will be created on first save
            String::new()
        }
        Err(e) => return Err(Box::new(e)),
    };
    
    let mut app = App::default();

    // Clear the default empty line if we have file content
    if !contents.trim().is_empty() {
        app.text_lines.clear();
        app.results.clear();
        app.result_animations.clear();
    }

    // Split the contents into lines and load them into the app
    for line in contents.lines() {
        app.text_lines.push(line.to_string());
        app.results.push(None);
        app.result_animations.push(None);
    }

    // If the file is empty, ensure we have at least one empty line
    if app.text_lines.is_empty() {
        app.text_lines.push(String::new());
        app.results.push(None);
        app.result_animations.push(None);
    }

    // Recalculate all lines
    app.recalculate_all();

    // Set the file path and mark as saved (for existing files) or unsaved (for new files)
    app.set_file_path(Some(path.clone()));
    
    // If the file didn't exist, mark it as having unsaved changes so it gets created on save
    if !path.exists() {
        app.has_unsaved_changes = true;
    }

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
/// Handle key events for save as dialog input
/// Returns true if the application should exit
fn handle_save_as_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char(c) => {
            app.save_as_input.push(c);
            false
        }
        KeyCode::Backspace => {
            app.save_as_input.pop();
            false
        }
        KeyCode::Enter => {
            // Save with the entered filename
            match app.save_as_from_dialog() {
                Ok(should_quit) => should_quit,
                Err(e) => {
                    eprintln!("Save failed: {}", e);
                    false
                }
            }
        }
        _ => false
    }
}