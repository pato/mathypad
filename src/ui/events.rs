//! Event handling and main TUI loop

use super::render::ui;
use crate::{App, Mode, TICK_RATE_MS};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

// Global storage for the current text lines for panic recovery
static PANIC_RECOVERY_TEXT: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

/// Initialize the panic recovery system
fn init_panic_recovery() {
    let _ = PANIC_RECOVERY_TEXT.set(Arc::new(Mutex::new(Vec::new())));
}

/// Update the text lines for panic recovery
fn update_panic_recovery_text(text_lines: &[String]) {
    if let Some(recovery_text) = PANIC_RECOVERY_TEXT.get() {
        if let Ok(mut stored_text) = recovery_text.lock() {
            stored_text.clear();
            stored_text.extend_from_slice(text_lines);
        }
    }
}

/// Save the current text to recovery file during panic
fn save_recovery_file() -> Result<(), Box<dyn Error>> {
    let recovery_text = PANIC_RECOVERY_TEXT
        .get()
        .ok_or("Panic recovery not initialized")?;

    let text_lines = recovery_text
        .lock()
        .map_err(|_| "Failed to lock recovery text")?;

    if text_lines.is_empty() {
        return Ok(()); // Nothing to save
    }

    // Create ~/.mathypad directory if it doesn't exist
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let mathypad_dir = home_dir.join(".mathypad");
    fs::create_dir_all(&mathypad_dir)?;

    // Save to recovery file
    let recovery_path = mathypad_dir.join("recovered.pad");
    let mut file = fs::File::create(recovery_path)?;

    for line in text_lines.iter() {
        writeln!(file, "{}", line)?;
    }

    file.flush()?;
    Ok(())
}

/// Cleanup function to restore terminal state
fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = io::stdout().flush();
}

/// Set up panic hook to ensure terminal cleanup and error logging
fn setup_panic_hook() {
    // Get the original panic hook
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Clean up the terminal first
        cleanup_terminal();

        // Try to save recovery file
        let recovery_saved = save_recovery_file().is_ok();

        // Write panic info to error log file
        if let Err(e) = write_panic_to_log(panic_info) {
            eprintln!("Failed to write panic log: {}", e);
        }

        // Print a user-friendly message
        eprintln!("\n💀 Mathypad encountered an unexpected error and had to exit.");
        eprintln!("📄 Error details have been saved to: ~/.mathypad/error.log");
        if recovery_saved {
            eprintln!("💾 Your work has been saved to: ~/.mathypad/recovered.pad");
        }
        eprintln!(
            "🐛 Please consider reporting this issue at: https://github.com/pato/mathypad/issues"
        );
        eprintln!();

        // Call the original hook for any additional handling
        original_hook(panic_info);
    }));
}

/// Write panic information to error log file
fn write_panic_to_log(panic_info: &std::panic::PanicHookInfo) -> Result<(), Box<dyn Error>> {
    // Create ~/.mathypad directory if it doesn't exist
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let mathypad_dir = home_dir.join(".mathypad");
    fs::create_dir_all(&mathypad_dir)?;

    // Create or append to error log
    let error_log_path = mathypad_dir.join("error.log");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(error_log_path)?;

    // Write timestamp and panic info
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    writeln!(file, "\n=== PANIC AT {} (Unix timestamp) ===", timestamp)?;
    writeln!(file, "{}", panic_info)?;

    // Add some system info
    writeln!(file, "\nSystem Info:")?;
    writeln!(file, "  OS: {}", std::env::consts::OS)?;
    writeln!(file, "  Arch: {}", std::env::consts::ARCH)?;
    if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
        writeln!(file, "  Mathypad Version: {}", version)?;
    }

    // Add backtrace if available
    if let Some(location) = panic_info.location() {
        writeln!(
            file,
            "  Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )?;
    }

    writeln!(file, "=== END PANIC ===\n")?;
    file.flush()?;

    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
/// Test function to simulate a panic (for testing panic handling)
pub fn test_panic_handling() {
    setup_panic_hook();
    panic!("This is a test panic to verify error handling works correctly");
}

/// Run the interactive TUI mode
pub fn run_interactive_mode() -> Result<(), Box<dyn Error>> {
    let app = App::default();
    run_event_loop(app)
}

/// Run the main event loop with the given app
fn run_event_loop(mut app: App) -> Result<(), Box<dyn Error>> {
    // Initialize panic recovery system
    init_panic_recovery();

    // Set up panic hook to ensure terminal cleanup
    setup_panic_hook();

    // Initialize recovery state with current app
    update_panic_recovery_text(&app.core.text_lines);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Check if this is a newer version and show welcome screen if needed
    if crate::version::is_newer_version() {
        app.show_welcome_dialog = true;
    }

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Check if we have active animations to determine timeout
        let has_active_animations = app
            .result_animations
            .iter()
            .any(|anim| anim.as_ref().is_some_and(|a| !a.is_complete()))
            || app
                .copy_flash_animations
                .iter()
                .any(|anim| anim.as_ref().is_some_and(|a| !a.is_complete()));

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
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
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
                            } else if app.show_welcome_dialog {
                                // Dismiss the welcome dialog and update stored version
                                app.show_welcome_dialog = false;
                                app.welcome_scroll_offset = 0;
                                // Update the stored version now that user has seen the welcome screen
                                if let Err(e) = crate::version::update_stored_version() {
                                    eprintln!("Warning: Could not update stored version: {}", e);
                                }
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
                            } else if app.show_welcome_dialog {
                                // Handle welcome dialog input (scrolling)
                                handle_welcome_dialog_input(&mut app, key.code);
                            } else if !app.show_unsaved_dialog {
                                // Only handle normal input if we're not showing any dialog
                                match app.mode {
                                    Mode::Insert => {
                                        handle_insert_mode(&mut app, key.code);
                                    }
                                    Mode::Normal => {
                                        handle_normal_mode(&mut app, key.code);
                                    }
                                    Mode::Command => {
                                        if handle_command_mode(&mut app, key.code) {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(&mut app, mouse, terminal.size()?.width);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate || has_active_animations {
            // Update animations on each tick or when animations are active
            app.update_animations();

            // Update panic recovery with current text state
            update_panic_recovery_text(&app.core.text_lines);

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
    let app = if let Some(path) = file_path {
        load_app_from_file(path)?
    } else {
        App::default()
    };
    run_event_loop(app)
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
        app.core.text_lines.clear();
        app.core.results.clear();
        app.result_animations.clear();
    }

    // Split the contents into lines and load them into the app
    for line in contents.lines() {
        app.core.text_lines.push(line.to_string());
        app.core.results.push(None);
        app.result_animations.push(None);
    }

    // If the file is empty, ensure we have at least one empty line
    if app.core.text_lines.is_empty() {
        app.core.text_lines.push(String::new());
        app.core.results.push(None);
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
        KeyCode::Esc => {
            // Switch to normal mode
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}

/// Handle key events in normal mode (vim-like)
pub fn handle_normal_mode(app: &mut App, key: KeyCode) {
    // Check if we have a pending command
    if let Some(pending_cmd) = app.pending_normal_command {
        app.pending_normal_command = None; // Clear pending command

        match (pending_cmd, key) {
            // 'dd' - delete line
            ('d', KeyCode::Char('d')) => {
                app.delete_line();
                return;
            }
            // 'gg' - go to beginning of file
            ('g', KeyCode::Char('g')) => {
                app.core.cursor_line = 0;
                app.core.cursor_col = 0;
                app.scroll_offset = 0;
                return;
            }
            // 'dw' - delete word forward
            ('d', KeyCode::Char('w')) => {
                app.delete_word_forward();
                return;
            }
            // 'db' - delete word backward
            ('d', KeyCode::Char('b')) => {
                app.delete_word_backward();
                return;
            }
            // 'dW' - delete WORD forward
            ('d', KeyCode::Char('W')) => {
                app.delete_word_forward_big();
                return;
            }
            // 'dB' - delete WORD backward
            ('d', KeyCode::Char('B')) => {
                app.delete_word_backward_big();
                return;
            }
            _ => {
                // Invalid command sequence, ignore and process the key normally
            }
        }
    }

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
        KeyCode::Char('w') => {
            app.move_word_forward();
        }
        KeyCode::Char('b') => {
            app.move_word_backward();
        }
        KeyCode::Char('W') => {
            app.move_word_forward_big();
        }
        KeyCode::Char('B') => {
            app.move_word_backward_big();
        }
        KeyCode::Char('x') => {
            app.delete_char_at_cursor();
        }
        KeyCode::Char('d') => {
            // Start a delete command
            app.pending_normal_command = Some('d');
        }
        KeyCode::Char('g') => {
            // Start a 'g' command (for 'gg')
            app.pending_normal_command = Some('g');
        }
        KeyCode::Char('0') => {
            // Go to beginning of line
            app.core.cursor_col = 0;
        }
        KeyCode::Char('$') => {
            // Go to end of line
            if app.core.cursor_line < app.core.text_lines.len() {
                app.core.cursor_col = app.core.text_lines[app.core.cursor_line].chars().count();
            }
        }
        KeyCode::Char('G') => {
            // Go to end of file
            app.core.cursor_line = app.core.text_lines.len().saturating_sub(1);
            app.core.cursor_col = 0;
            // Adjust scroll to show the last line
            let visible_height = 25; // Approximate, this could be made dynamic
            if app.core.cursor_line >= app.scroll_offset + visible_height {
                app.scroll_offset = app.core.cursor_line.saturating_sub(visible_height - 1);
            }
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
            if app.core.cursor_line < app.core.text_lines.len() {
                app.core.cursor_col = app.core.text_lines[app.core.cursor_line].chars().count();
            }
            app.mode = Mode::Insert;
        }
        KeyCode::Char('I') => {
            app.core.cursor_col = 0;
            app.mode = Mode::Insert;
        }
        KeyCode::Char('o') => {
            // Insert new line below and enter insert mode
            if app.core.cursor_line < app.core.text_lines.len() {
                app.core.cursor_col = app.core.text_lines[app.core.cursor_line].chars().count();
            }
            app.new_line();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('O') => {
            // Insert new line above and enter insert mode
            app.core
                .text_lines
                .insert(app.core.cursor_line, String::new());
            app.core.results.insert(app.core.cursor_line, None);
            app.core.cursor_col = 0;
            app.mode = Mode::Insert;
        }
        KeyCode::Char(':') => {
            // Enter command mode
            app.mode = Mode::Command;
            app.command_line = ":".to_string();
            app.command_cursor = 1;
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

/// Handle key events in command mode (vim-like)
/// Returns true if the application should quit
pub fn handle_command_mode(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char(c) => {
            // Insert character at cursor position
            let mut chars: Vec<char> = app.command_line.chars().collect();
            chars.insert(app.command_cursor, c);
            app.command_line = chars.into_iter().collect();
            app.command_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.command_cursor > 0 {
                let mut chars: Vec<char> = app.command_line.chars().collect();
                chars.remove(app.command_cursor - 1);
                app.command_line = chars.into_iter().collect();
                app.command_cursor -= 1;
            }
        }
        KeyCode::Left => {
            if app.command_cursor > 0 {
                app.command_cursor -= 1;
            }
        }
        KeyCode::Right => {
            if app.command_cursor < app.command_line.chars().count() {
                app.command_cursor += 1;
            }
        }
        KeyCode::Home => {
            app.command_cursor = 0;
        }
        KeyCode::End => {
            app.command_cursor = app.command_line.chars().count();
        }
        KeyCode::Enter => {
            // Execute command and exit command mode
            if execute_command(app) {
                // Command requested quit
                return true;
            }
            app.mode = Mode::Normal;
            app.command_line.clear();
            app.command_cursor = 0;
        }
        KeyCode::Esc => {
            // Cancel command and return to normal mode
            app.mode = Mode::Normal;
            app.command_line.clear();
            app.command_cursor = 0;
        }
        _ => {}
    }
    false // Default: don't quit
}

/// Execute a vim-like command
/// Returns true if the application should quit
fn execute_command(app: &mut App) -> bool {
    let command = app.command_line.trim();

    // Commands must start with ':'
    if !command.starts_with(':') {
        return false;
    }

    // Safely remove the ':' prefix using char boundary
    let command = if let Some(stripped) = command.strip_prefix(':') {
        stripped
    } else {
        return false;
    };
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return false;
    }

    match parts[0] {
        "q" | "quit" => {
            // Quit command
            if app.has_unsaved_changes {
                app.show_unsaved_dialog = true;
                return false; // Don't quit yet, show dialog
            } else {
                return true; // Signal to quit
            }
        }
        "q!" | "quit!" => {
            // Force quit command - exit without saving, no confirmation
            return true; // Signal to quit immediately
        }
        "cq" => {
            // Quit with error code - exit without saving, no confirmation
            return true; // Signal to quit immediately
        }
        "w" | "write" => {
            // Write/save command
            if parts.len() > 1 {
                // Save to specific file: :w filename.pad
                let filename = parts[1..].join(" ");
                let filename = if filename.ends_with(".pad") {
                    filename
                } else {
                    format!("{}.pad", filename)
                };

                match app.save_as(PathBuf::from(filename)) {
                    Ok(_) => {} // Successfully saved
                    Err(e) => {
                        eprintln!("Save failed: {}", e);
                    }
                }
            } else {
                // Save to current file or show save as dialog
                if app.file_path.is_some() {
                    match app.save() {
                        Ok(_) => {} // Successfully saved
                        Err(e) => {
                            eprintln!("Save failed: {}", e);
                        }
                    }
                } else {
                    // No file path, show save as dialog
                    app.show_save_as_dialog = true;
                    app.save_as_input = ".pad".to_string();
                }
            }
        }
        "wq" => {
            // Save and quit
            if parts.len() > 1 {
                // Save to specific file and quit: :wq filename.pad
                let filename = parts[1..].join(" ");
                let filename = if filename.ends_with(".pad") {
                    filename
                } else {
                    format!("{}.pad", filename)
                };

                match app.save_as(PathBuf::from(filename)) {
                    Ok(_) => return true, // Signal to quit
                    Err(e) => {
                        eprintln!("Save failed: {}", e);
                    }
                }
            } else {
                // Save to current file and quit
                if app.file_path.is_some() {
                    match app.save() {
                        Ok(_) => return true, // Signal to quit
                        Err(e) => {
                            eprintln!("Save failed: {}", e);
                        }
                    }
                } else {
                    // No file path, show save as dialog and set quit flag
                    app.show_save_as_dialog = true;
                    app.save_as_input = ".pad".to_string();
                    app.save_as_and_quit = true;
                }
            }
        }
        _ => {
            // Unknown command, ignore
        }
    }

    false // Default: don't quit
}

/// Handle mouse events for dragging the separator and copying content
fn handle_mouse_event(app: &mut App, mouse: MouseEvent, terminal_width: u16) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if app.is_mouse_over_separator(mouse.column, terminal_width) {
                app.start_dragging_separator();
                app.set_separator_hover(true);
            } else {
                app.set_separator_hover(false);

                // Check for double-click to copy content
                if app.is_double_click(mouse.column, mouse.row) {
                    handle_double_click_copy(app, mouse.column, mouse.row, terminal_width);
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if app.is_dragging_separator {
                app.stop_dragging_separator();
                // Check if still hovering after release
                app.set_separator_hover(app.is_mouse_over_separator(mouse.column, terminal_width));
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.is_dragging_separator {
                app.update_separator_position(mouse.column, terminal_width);
            }
        }
        MouseEventKind::Moved => {
            // Update hover state when mouse moves
            let is_over_separator = app.is_mouse_over_separator(mouse.column, terminal_width);
            app.set_separator_hover(is_over_separator);
        }
        _ => {}
    }
}

/// Handle double-click to copy text or result
fn handle_double_click_copy(app: &mut App, mouse_x: u16, mouse_y: u16, terminal_width: u16) {
    use ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        widgets::{Block, Borders},
    };

    // Recreate the same layout calculation as the render function
    let terminal_area = Rect {
        x: 0,
        y: 0,
        width: terminal_width,
        height: 50, // Height doesn't matter for our calculation
    };

    let text_percentage = app.separator_position;
    let results_percentage = 100 - app.separator_position;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(text_percentage),
            Constraint::Percentage(results_percentage),
        ])
        .split(terminal_area);

    // Determine which panel was clicked
    let (is_results_panel, panel_area) = if mouse_x < chunks[0].x + chunks[0].width {
        (false, chunks[0])
    } else {
        (true, chunks[1])
    };

    // Calculate the inner area (content area) for the clicked panel
    let block = Block::default().borders(Borders::ALL);
    let inner_area = block.inner(panel_area);

    // Check if click is within the content area
    if mouse_x >= inner_area.x
        && mouse_x < inner_area.x + inner_area.width
        && mouse_y >= inner_area.y
        && mouse_y < inner_area.y + inner_area.height
    {
        // Calculate which line was clicked within the content area
        let content_line = (mouse_y - inner_area.y) as usize;
        let line_index = app.scroll_offset + content_line;

        if is_results_panel {
            // Clicked in results area - copy the result
            if line_index < app.core.results.len() {
                if let Some(result) = app.core.results[line_index].clone() {
                    if let Err(e) = app.copy_to_clipboard(&result, line_index, true) {
                        eprintln!("Copy failed: {}", e);
                    }
                }
            }
        } else {
            // Clicked in text area - copy the line content
            if line_index < app.core.text_lines.len() {
                let text = app.core.text_lines[line_index].clone();
                if !text.trim().is_empty() {
                    if let Err(e) = app.copy_to_clipboard(&text, line_index, false) {
                        eprintln!("Copy failed: {}", e);
                    }
                }
            }
        }
    }
}

/// Handle key events for save as dialog input
/// Returns true if the application should exit
fn handle_save_as_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char(c) => {
            // If input is just ".pad", replace it with the character + ".pad"
            if app.save_as_input == ".pad" {
                app.save_as_input = format!("{}.pad", c);
            } else {
                // Insert character before the ".pad" extension
                if app.save_as_input.ends_with(".pad") {
                    let base = app
                        .save_as_input
                        .strip_suffix(".pad")
                        .unwrap_or(&app.save_as_input);
                    app.save_as_input = format!("{}{}.pad", base, c);
                } else {
                    // Fallback: just append the character
                    app.save_as_input.push(c);
                }
            }
            false
        }
        KeyCode::Backspace => {
            if app.save_as_input.ends_with(".pad") && app.save_as_input.len() > 4 {
                // Remove character before the ".pad" extension
                // Use strip_suffix to safely remove the ".pad" extension
                let base = app
                    .save_as_input
                    .strip_suffix(".pad")
                    .unwrap_or(&app.save_as_input);
                if base.is_empty() {
                    app.save_as_input = ".pad".to_string();
                } else {
                    // Safely remove the last character using char boundaries
                    let mut chars = base.chars();
                    chars.next_back(); // Remove last char
                    let new_base: String = chars.collect();
                    app.save_as_input = if new_base.is_empty() {
                        ".pad".to_string()
                    } else {
                        format!("{}.pad", new_base)
                    };
                }
            } else if app.save_as_input == ".pad" {
                // Don't allow deleting the extension when it's just ".pad"
            } else {
                // Fallback: normal backspace
                app.save_as_input.pop();
            }
            false
        }
        KeyCode::Enter => {
            // Ensure filename has .pad extension before saving
            if !app.save_as_input.ends_with(".pad") && !app.save_as_input.is_empty() {
                app.save_as_input.push_str(".pad");
            }

            // Save with the entered filename
            match app.save_as_from_dialog() {
                Ok(should_quit) => should_quit,
                Err(e) => {
                    eprintln!("Save failed: {}", e);
                    false
                }
            }
        }
        _ => false,
    }
}

/// Handle key events for welcome dialog input (scrolling)
fn handle_welcome_dialog_input(app: &mut App, key: KeyCode) {
    // Get the changelog content to calculate max scroll
    let changelog_content = crate::version::get_changelog_since_version().unwrap_or_else(|| {
        "Welcome to mathypad!\n\nThis appears to be your first time running this version."
            .to_string()
    });

    // Calculate the total number of lines (header + changelog)
    let header_line_count = 4; // "Welcome! ...", "", "What's new:", ""
    let changelog_line_count = changelog_content.lines().count();
    let total_lines = header_line_count + changelog_line_count;

    // Calculate scrollable height (matches calculation in render.rs)
    // Dialog height is 25, minus 2 for borders, minus 3 for footer = 20 lines of scrollable content
    let dialog_height: usize = 25;
    let inner_height = dialog_height.saturating_sub(2); // Remove borders
    let footer_height = 3; // Empty line + instructions + scroll indicator
    let scrollable_height = inner_height.saturating_sub(footer_height);
    let max_scroll = total_lines.saturating_sub(scrollable_height);

    match key {
        KeyCode::Up => {
            if app.welcome_scroll_offset > 0 {
                app.welcome_scroll_offset -= 1;
            }
        }
        KeyCode::Down => {
            if app.welcome_scroll_offset < max_scroll {
                app.welcome_scroll_offset += 1;
            }
        }
        KeyCode::PageUp => {
            // Scroll up by half a screen
            let scroll_amount = (scrollable_height / 2).max(1);
            app.welcome_scroll_offset = app.welcome_scroll_offset.saturating_sub(scroll_amount);
        }
        KeyCode::PageDown => {
            // Scroll down by half a screen
            let scroll_amount = (scrollable_height / 2).max(1);
            app.welcome_scroll_offset = (app.welcome_scroll_offset + scroll_amount).min(max_scroll);
        }
        KeyCode::Home => {
            // Go to top
            app.welcome_scroll_offset = 0;
        }
        KeyCode::End => {
            // Go to bottom
            app.welcome_scroll_offset = max_scroll;
        }
        KeyCode::Enter => {
            // Enter also closes the dialog and updates stored version
            app.show_welcome_dialog = false;
            app.welcome_scroll_offset = 0;
            // Update the stored version now that user has seen the welcome screen
            if let Err(e) = crate::version::update_stored_version() {
                eprintln!("Warning: Could not update stored version: {}", e);
            }
        }
        _ => {
            // Ignore other keys
        }
    }
}
