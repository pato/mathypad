use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

struct App {
    text_lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            text_lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }
}

impl App {
    fn insert_char(&mut self, c: char) {
        if self.cursor_line < self.text_lines.len() {
            self.text_lines[self.cursor_line].insert(self.cursor_col, c);
            self.cursor_col += 1;
        }
    }
    
    fn delete_char(&mut self) {
        if self.cursor_line < self.text_lines.len() && self.cursor_col > 0 {
            self.text_lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        }
    }
    
    fn new_line(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            let current_line = self.text_lines[self.cursor_line].clone();
            let (left, right) = current_line.split_at(self.cursor_col);
            self.text_lines[self.cursor_line] = left.to_string();
            self.text_lines.insert(self.cursor_line + 1, right.to_string());
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }
    
    fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.text_lines[self.cursor_line].len());
        }
    }
    
    fn move_cursor_down(&mut self) {
        if self.cursor_line + 1 < self.text_lines.len() {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.text_lines[self.cursor_line].len());
        }
    }
    
    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }
    
    fn move_cursor_right(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            self.cursor_col = (self.cursor_col + 1).min(self.text_lines[self.cursor_line].len());
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::default();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            break;
                        }
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

fn parse_colors(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();
    
    while current_pos < chars.len() {
        if chars[current_pos].is_ascii_digit() || chars[current_pos] == '.' {
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
        } else {
            spans.push(Span::raw(chars[current_pos].to_string()));
            current_pos += 1;
        }
    }
    
    spans
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(f.area());
    
    render_text_area(f, app, chunks[0]);
    render_results_panel(f, chunks[1]);
}

fn render_text_area(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Mathematical Notes")
        .borders(Borders::ALL);
    
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
            spans.extend(parse_colors(before_cursor));
            spans.push(Span::styled("█", Style::default().fg(Color::White)));
            spans.extend(parse_colors(after_cursor));
        } else {
            spans.extend(parse_colors(line_text));
        }
        
        lines.push(Line::from(spans));
    }
    
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner_area);
}

fn render_results_panel(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Results")
        .borders(Borders::ALL);
    
    let paragraph = Paragraph::new("Equation results will appear here")
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}