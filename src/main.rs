use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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
    results: Vec<Option<f64>>,
}

impl Default for App {
    fn default() -> App {
        App {
            text_lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            results: vec![None],
        }
    }
}

impl App {
    fn insert_char(&mut self, c: char) {
        if self.cursor_line < self.text_lines.len() {
            self.text_lines[self.cursor_line].insert(self.cursor_col, c);
            self.cursor_col += 1;
            self.update_result(self.cursor_line);
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_line < self.text_lines.len() && self.cursor_col > 0 {
            self.text_lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
            self.update_result(self.cursor_line);
        }
    }

    fn new_line(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            let current_line = self.text_lines[self.cursor_line].clone();
            let (left, right) = current_line.split_at(self.cursor_col);
            self.text_lines[self.cursor_line] = left.to_string();
            self.text_lines
                .insert(self.cursor_line + 1, right.to_string());
            self.results.insert(self.cursor_line + 1, None);
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.update_result(self.cursor_line - 1);
            self.update_result(self.cursor_line);
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

    fn update_result(&mut self, line_index: usize) {
        if line_index < self.text_lines.len() && line_index < self.results.len() {
            let line = &self.text_lines[line_index];
            self.results[line_index] = evaluate_expression(line);
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

fn evaluate_expression(text: &str) -> Option<f64> {
    // Find the longest valid mathematical expression in the text
    find_math_expressions(text)
        .into_iter()
        .filter_map(|expr| parse_and_evaluate(&expr))
        .next()
}

fn find_math_expressions(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut expressions = Vec::new();
    
    for start in 0..chars.len() {
        if chars[start].is_ascii_digit() {
            for end in start + 1..=chars.len() {
                let candidate = chars[start..end].iter().collect::<String>();
                if is_valid_math_expression(&candidate) {
                    expressions.push(candidate);
                }
            }
        }
    }
    
    // Sort by length descending to get the longest expression first
    expressions.sort_by(|a, b| b.len().cmp(&a.len()));
    expressions
}

fn is_valid_math_expression(expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return false;
    }
    
    let mut has_number = false;
    let mut has_operator = false;
    let mut paren_count = 0;
    let mut prev_was_operator = true; // Start as true to allow leading numbers
    
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            ' ' => {
                i += 1;
                continue;
            }
            '0'..='9' => {
                has_number = true;
                prev_was_operator = false;
                // Skip through the whole number
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                continue;
            }
            '.' => {
                if prev_was_operator {
                    return false; // Can't start with decimal point
                }
                i += 1;
            }
            '+' | '-' | '*' | '/' => {
                if prev_was_operator && ch != '-' {
                    return false; // Two operators in a row (except minus for negation)
                }
                has_operator = true;
                prev_was_operator = true;
                i += 1;
            }
            '(' => {
                paren_count += 1;
                prev_was_operator = true;
                i += 1;
            }
            ')' => {
                paren_count -= 1;
                if paren_count < 0 {
                    return false;
                }
                prev_was_operator = false;
                i += 1;
            }
            _ => {
                // If we encounter any other character, check if what we have so far is valid
                break;
            }
        }
    }
    
    // Must have balanced parentheses, at least one number, and if it has operators, must end properly
    paren_count == 0 && has_number && (!has_operator || !prev_was_operator)
}

fn parse_and_evaluate(expr: &str) -> Option<f64> {
    let expr = expr.replace(" ", "");

    if expr.is_empty() {
        return None;
    }

    let mut tokens = Vec::new();
    let mut current_number = String::new();

    for ch in expr.chars() {
        match ch {
            '0'..='9' | '.' => {
                current_number.push(ch);
            }
            '+' | '-' | '*' | '/' | '(' | ')' => {
                if !current_number.is_empty() {
                    if let Ok(num) = current_number.parse::<f64>() {
                        tokens.push(Token::Number(num));
                    } else {
                        return None;
                    }
                    current_number.clear();
                }
                tokens.push(match ch {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    _ => return None,
                });
            }
            _ => return None,
        }
    }

    if !current_number.is_empty() {
        if let Ok(num) = current_number.parse::<f64>() {
            tokens.push(Token::Number(num));
        } else {
            return None;
        }
    }

    evaluate_tokens(&tokens)
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
}

fn evaluate_tokens(tokens: &[Token]) -> Option<f64> {
    if tokens.is_empty() {
        return None;
    }

    let mut output = Vec::new();
    let mut operators = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => output.push(*n),
            Token::LeftParen => operators.push(token.clone()),
            Token::RightParen => {
                while let Some(op) = operators.pop() {
                    if matches!(op, Token::LeftParen) {
                        break;
                    }
                    if !apply_operator(&mut output, &op) {
                        return None;
                    }
                }
            }
            op => {
                while let Some(top_op) = operators.last() {
                    if matches!(top_op, Token::LeftParen) || precedence(op) > precedence(top_op) {
                        break;
                    }
                    let op_to_apply = operators.pop().unwrap();
                    if !apply_operator(&mut output, &op_to_apply) {
                        return None;
                    }
                }
                operators.push(op.clone());
            }
        }
    }

    while let Some(op) = operators.pop() {
        if !apply_operator(&mut output, &op) {
            return None;
        }
    }

    if output.len() == 1 {
        Some(output[0])
    } else {
        None
    }
}

fn precedence(token: &Token) -> i32 {
    match token {
        Token::Plus | Token::Minus => 1,
        Token::Multiply | Token::Divide => 2,
        _ => 0,
    }
}

fn apply_operator(output: &mut Vec<f64>, op: &Token) -> bool {
    if output.len() < 2 {
        return false;
    }

    let b = output.pop().unwrap();
    let a = output.pop().unwrap();

    let result = match op {
        Token::Plus => a + b,
        Token::Minus => a - b,
        Token::Multiply => a * b,
        Token::Divide => {
            if b == 0.0 {
                return false;
            }
            a / b
        }
        _ => return false,
    };

    output.push(result);
    true
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
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(f.area());

    render_text_area(f, app, chunks[0]);
    render_results_panel(f, app, chunks[1]);
}

fn render_text_area(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Mathypad").borders(Borders::ALL);

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

fn render_results_panel(f: &mut Frame, app: &App, area: Rect) {
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

        match result {
            Some(value) => {
                let result_text = if value.fract() == 0.0 {
                    format!("{}", *value as i64)
                } else {
                    format!("{:.3}", value)
                };
                spans.push(Span::styled(result_text, Style::default().fg(Color::Green)));
            }
            None => {}
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner_area);
}

