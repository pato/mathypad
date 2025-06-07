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
        .filter_map(|expr| {
            if let Some(unit_value) = parse_and_evaluate(&expr) {
                Some(unit_value.value)
            } else {
                // Fallback to old parser for expressions without units
                parse_and_evaluate_simple(&expr)
            }
        })
        .next()
}

fn parse_and_evaluate_simple(expr: &str) -> Option<f64> {
    let expr = expr.replace(" ", "");

    if expr.is_empty() {
        return None;
    }

    let mut tokens = Vec::new();
    let mut current_number = String::new();

    for ch in expr.chars() {
        match ch {
            '0'..='9' | '.' | ',' => {
                current_number.push(ch);
            }
            '+' | '-' | '*' | '/' | '(' | ')' => {
                if !current_number.is_empty() {
                    // Remove commas before parsing
                    let cleaned_number = current_number.replace(",", "");
                    if let Ok(num) = cleaned_number.parse::<f64>() {
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
        // Remove commas before parsing
        let cleaned_number = current_number.replace(",", "");
        if let Ok(num) = cleaned_number.parse::<f64>() {
            tokens.push(Token::Number(num));
        } else {
            return None;
        }
    }

    evaluate_tokens(&tokens)
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
                // Skip through the whole number (including commas and decimals)
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',') {
                    i += 1;
                }
                
                // Skip whitespace
                while i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
                
                // Check for unit
                if i < chars.len() && chars[i].is_ascii_alphabetic() {
                    let unit_start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '/') {
                        i += 1;
                    }
                    
                    let unit_str: String = chars[unit_start..i].iter().collect();
                    if parse_unit(&unit_str).is_none() && unit_str.to_lowercase() != "to" {
                        // Not a recognized unit, rewind
                        i = unit_start;
                    }
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
                if ch.is_ascii_alphabetic() {
                    let unit_start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '/') {
                        i += 1;
                    }
                    
                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        has_operator = true;
                        prev_was_operator = true;
                    } else if parse_unit(&word).is_some() {
                        // Valid unit, continue
                        prev_was_operator = false;
                    } else {
                        // Invalid word, break
                        break;
                    }
                } else {
                    // If we encounter any other character, check if what we have so far is valid
                    break;
                }
            }
        }
    }
    
    // Must have balanced parentheses, at least one number, and if it has operators, must end properly
    paren_count == 0 && has_number && (!has_operator || !prev_was_operator)
}

fn parse_and_evaluate(expr: &str) -> Option<UnitValue> {
    let tokens = tokenize_with_units(expr)?;
    evaluate_tokens_with_units(&tokens)
}

fn tokenize_with_units(expr: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
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
                // Parse number (with potential commas)
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',') {
                    i += 1;
                }
                
                let number_str: String = chars[start..i].iter().collect();
                let cleaned_number = number_str.replace(",", "");
                let num = cleaned_number.parse::<f64>().ok()?;
                
                // Skip whitespace
                while i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
                
                // Look for unit
                if i < chars.len() && chars[i].is_ascii_alphabetic() {
                    let unit_start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '/') {
                        i += 1;
                    }
                    
                    let unit_str: String = chars[unit_start..i].iter().collect();
                    if let Some(unit) = parse_unit(&unit_str) {
                        tokens.push(Token::NumberWithUnit(num, unit));
                    } else {
                        tokens.push(Token::Number(num));
                        // Put back the unit characters - they might be part of something else
                        i = unit_start;
                    }
                } else {
                    tokens.push(Token::Number(num));
                }
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Multiply);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Divide);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LeftParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RightParen);
                i += 1;
            }
            't' | 'T' => {
                // Check for "to" keyword
                if i + 1 < chars.len() && chars[i + 1].to_lowercase().next() == Some('o') {
                    // Skip whitespace after "to"
                    i += 2;
                    while i < chars.len() && chars[i] == ' ' {
                        i += 1;
                    }
                    tokens.push(Token::To);
                } else {
                    return None; // Unexpected character
                }
            }
            _ => {
                if ch.is_ascii_alphabetic() {
                    // Could be a unit or part of "to"
                    let unit_start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '/') {
                        i += 1;
                    }
                    
                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if let Some(unit) = parse_unit(&word) {
                        // Standalone unit for conversion target
                        tokens.push(Token::NumberWithUnit(1.0, unit));
                    } else {
                        return None; // Unexpected word
                    }
                } else {
                    return None; // Unexpected character
                }
            }
        }
    }

    Some(tokens)
}

#[derive(Debug, Clone)]
struct UnitValue {
    value: f64,
    unit: Option<Unit>,
}

impl UnitValue {
    fn new(value: f64, unit: Option<Unit>) -> Self {
        UnitValue { value, unit }
    }
    
    fn to_unit(&self, target_unit: &Unit) -> Option<UnitValue> {
        match &self.unit {
            Some(current_unit) => {
                if current_unit.unit_type() == target_unit.unit_type() {
                    let base_value = current_unit.to_base_value(self.value);
                    let converted_value = target_unit.from_base_value(base_value);
                    Some(UnitValue::new(converted_value, Some(target_unit.clone())))
                } else {
                    None // Can't convert between different unit types
                }
            }
            None => None, // No unit to convert from
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    NumberWithUnit(f64, Unit),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    To, // for conversions like "to KiB"
}

#[derive(Debug, Clone, PartialEq)]
enum Unit {
    // Time units (base: seconds)
    Second,
    Minute,
    Hour,
    Day,
    
    // Data units (base 10)
    Byte,
    KB, // Kilobyte
    MB, // Megabyte  
    GB, // Gigabyte
    TB, // Terabyte
    
    // Data units (base 2)
    KiB, // Kibibyte
    MiB, // Mebibyte
    GiB, // Gibibyte
    TiB, // Tebibyte
    
    // Derived units
    BytesPerSecond,
    KBPerSecond,
    MBPerSecond,
    GBPerSecond,
    TBPerSecond,
    KiBPerSecond,
    MiBPerSecond,
    GiBPerSecond,
    TiBPerSecond,
}

impl Unit {
    fn to_base_value(&self, value: f64) -> f64 {
        match self {
            // Time units (convert to seconds)
            Unit::Second => value,
            Unit::Minute => value * 60.0,
            Unit::Hour => value * 3600.0,
            Unit::Day => value * 86400.0,
            
            // Data units base 10 (convert to bytes)
            Unit::Byte => value,
            Unit::KB => value * 1_000.0,
            Unit::MB => value * 1_000_000.0,
            Unit::GB => value * 1_000_000_000.0,
            Unit::TB => value * 1_000_000_000_000.0,
            
            // Data units base 2 (convert to bytes)
            Unit::KiB => value * 1_024.0,
            Unit::MiB => value * 1_048_576.0,
            Unit::GiB => value * 1_073_741_824.0,
            Unit::TiB => value * 1_099_511_627_776.0,
            
            // Rate units (convert to bytes per second)
            Unit::BytesPerSecond => value,
            Unit::KBPerSecond => value * 1_000.0,
            Unit::MBPerSecond => value * 1_000_000.0,
            Unit::GBPerSecond => value * 1_000_000_000.0,
            Unit::TBPerSecond => value * 1_000_000_000_000.0,
            Unit::KiBPerSecond => value * 1_024.0,
            Unit::MiBPerSecond => value * 1_048_576.0,
            Unit::GiBPerSecond => value * 1_073_741_824.0,
            Unit::TiBPerSecond => value * 1_099_511_627_776.0,
        }
    }
    
    fn from_base_value(&self, base_value: f64) -> f64 {
        match self {
            // Time units (from seconds)
            Unit::Second => base_value,
            Unit::Minute => base_value / 60.0,
            Unit::Hour => base_value / 3600.0,
            Unit::Day => base_value / 86400.0,
            
            // Data units base 10 (from bytes)
            Unit::Byte => base_value,
            Unit::KB => base_value / 1_000.0,
            Unit::MB => base_value / 1_000_000.0,
            Unit::GB => base_value / 1_000_000_000.0,
            Unit::TB => base_value / 1_000_000_000_000.0,
            
            // Data units base 2 (from bytes)
            Unit::KiB => base_value / 1_024.0,
            Unit::MiB => base_value / 1_048_576.0,
            Unit::GiB => base_value / 1_073_741_824.0,
            Unit::TiB => base_value / 1_099_511_627_776.0,
            
            // Rate units (from bytes per second)
            Unit::BytesPerSecond => base_value,
            Unit::KBPerSecond => base_value / 1_000.0,
            Unit::MBPerSecond => base_value / 1_000_000.0,
            Unit::GBPerSecond => base_value / 1_000_000_000.0,
            Unit::TBPerSecond => base_value / 1_000_000_000_000.0,
            Unit::KiBPerSecond => base_value / 1_024.0,
            Unit::MiBPerSecond => base_value / 1_048_576.0,
            Unit::GiBPerSecond => base_value / 1_073_741_824.0,
            Unit::TiBPerSecond => base_value / 1_099_511_627_776.0,
        }
    }
    
    fn unit_type(&self) -> UnitType {
        match self {
            Unit::Second | Unit::Minute | Unit::Hour | Unit::Day => UnitType::Time,
            Unit::Byte | Unit::KB | Unit::MB | Unit::GB | Unit::TB |
            Unit::KiB | Unit::MiB | Unit::GiB | Unit::TiB => UnitType::Data,
            Unit::BytesPerSecond | Unit::KBPerSecond | Unit::MBPerSecond | Unit::GBPerSecond | Unit::TBPerSecond |
            Unit::KiBPerSecond | Unit::MiBPerSecond | Unit::GiBPerSecond | Unit::TiBPerSecond => UnitType::DataRate,
        }
    }
    
    fn display_name(&self) -> &'static str {
        match self {
            Unit::Second => "s",
            Unit::Minute => "min",
            Unit::Hour => "h",
            Unit::Day => "day",
            Unit::Byte => "B",
            Unit::KB => "KB",
            Unit::MB => "MB", 
            Unit::GB => "GB",
            Unit::TB => "TB",
            Unit::KiB => "KiB",
            Unit::MiB => "MiB",
            Unit::GiB => "GiB",
            Unit::TiB => "TiB",
            Unit::BytesPerSecond => "B/s",
            Unit::KBPerSecond => "KB/s",
            Unit::MBPerSecond => "MB/s",
            Unit::GBPerSecond => "GB/s",
            Unit::TBPerSecond => "TB/s",
            Unit::KiBPerSecond => "KiB/s",
            Unit::MiBPerSecond => "MiB/s",
            Unit::GiBPerSecond => "GiB/s",
            Unit::TiBPerSecond => "TiB/s",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UnitType {
    Time,
    Data,
    DataRate,
}

fn parse_unit(text: &str) -> Option<Unit> {
    match text.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(Unit::Second),
        "min" | "minute" | "minutes" => Some(Unit::Minute),
        "h" | "hr" | "hour" | "hours" => Some(Unit::Hour),
        "d" | "day" | "days" => Some(Unit::Day),
        
        "b" | "byte" | "bytes" => Some(Unit::Byte),
        "kb" => Some(Unit::KB),
        "mb" => Some(Unit::MB),
        "gb" => Some(Unit::GB),
        "tb" => Some(Unit::TB),
        
        "kib" => Some(Unit::KiB),
        "mib" => Some(Unit::MiB),
        "gib" => Some(Unit::GiB),
        "tib" => Some(Unit::TiB),
        
        "b/s" | "bytes/s" | "bps" => Some(Unit::BytesPerSecond),
        "kb/s" | "kbps" => Some(Unit::KBPerSecond),
        "mb/s" | "mbps" => Some(Unit::MBPerSecond),
        "gb/s" | "gbps" => Some(Unit::GBPerSecond),
        "tb/s" | "tbps" => Some(Unit::TBPerSecond),
        "kib/s" | "kibps" => Some(Unit::KiBPerSecond),
        "mib/s" | "mibps" => Some(Unit::MiBPerSecond),
        "gib/s" | "gibps" => Some(Unit::GiBPerSecond),
        "tib/s" | "tibps" => Some(Unit::TiBPerSecond),
        
        _ => None,
    }
}

fn evaluate_tokens_with_units(tokens: &[Token]) -> Option<UnitValue> {
    if tokens.is_empty() {
        return None;
    }
    
    // Handle conversion expressions like "1 GiB to KiB"
    for i in 0..tokens.len().saturating_sub(2) {
        if let (Token::NumberWithUnit(value, from_unit), Token::To) = (&tokens[i], &tokens[i + 1]) {
            // Look for target unit in the remaining tokens
            for j in (i + 2)..tokens.len() {
                if let Token::NumberWithUnit(_, to_unit) = &tokens[j] {
                    let unit_value = UnitValue::new(*value, Some(from_unit.clone()));
                    return unit_value.to_unit(to_unit);
                }
            }
        }
    }
    
    // Handle simple arithmetic with units
    let mut operator_stack = Vec::new();
    let mut value_stack = Vec::new();
    
    for token in tokens {
        match token {
            Token::Number(n) => {
                value_stack.push(UnitValue::new(*n, None));
            }
            Token::NumberWithUnit(value, unit) => {
                value_stack.push(UnitValue::new(*value, Some(unit.clone())));
            }
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide => {
                while let Some(top_op) = operator_stack.last() {
                    if precedence_unit(token) <= precedence_unit(top_op) {
                        let op = operator_stack.pop().unwrap();
                        if !apply_operator_with_units(&mut value_stack, &op) {
                            return None;
                        }
                    } else {
                        break;
                    }
                }
                operator_stack.push(token.clone());
            }
            Token::LeftParen => {
                operator_stack.push(token.clone());
            }
            Token::RightParen => {
                while let Some(op) = operator_stack.pop() {
                    if matches!(op, Token::LeftParen) {
                        break;
                    }
                    if !apply_operator_with_units(&mut value_stack, &op) {
                        return None;
                    }
                }
            }
            _ => {}
        }
    }
    
    while let Some(op) = operator_stack.pop() {
        if !apply_operator_with_units(&mut value_stack, &op) {
            return None;
        }
    }
    
    if value_stack.len() == 1 {
        value_stack.pop()
    } else {
        None
    }
}

fn precedence_unit(token: &Token) -> i32 {
    match token {
        Token::Plus | Token::Minus => 1,
        Token::Multiply | Token::Divide => 2,
        _ => 0,
    }
}

fn apply_operator_with_units(stack: &mut Vec<UnitValue>, op: &Token) -> bool {
    if stack.len() < 2 {
        return false;
    }
    
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    
    let result = match op {
        Token::Plus => {
            // Addition: units must be compatible
            match (&a.unit, &b.unit) {
                (Some(unit_a), Some(unit_b)) => {
                    if unit_a.unit_type() == unit_b.unit_type() {
                        let base_a = unit_a.to_base_value(a.value);
                        let base_b = unit_b.to_base_value(b.value);
                        let result_base = base_a + base_b;
                        let result_value = unit_a.from_base_value(result_base);
                        UnitValue::new(result_value, Some(unit_a.clone()))
                    } else {
                        return false;
                    }
                }
                (None, None) => UnitValue::new(a.value + b.value, None),
                _ => return false, // Can't add number with unit and number without unit
            }
        }
        Token::Minus => {
            // Subtraction: units must be compatible
            match (&a.unit, &b.unit) {
                (Some(unit_a), Some(unit_b)) => {
                    if unit_a.unit_type() == unit_b.unit_type() {
                        let base_a = unit_a.to_base_value(a.value);
                        let base_b = unit_b.to_base_value(b.value);
                        let result_base = base_a - base_b;
                        let result_value = unit_a.from_base_value(result_base);
                        UnitValue::new(result_value, Some(unit_a.clone()))
                    } else {
                        return false;
                    }
                }
                (None, None) => UnitValue::new(a.value - b.value, None),
                _ => return false,
            }
        }
        Token::Multiply => {
            // Multiplication: special cases for units
            match (&a.unit, &b.unit) {
                (Some(Unit::GiB), Some(Unit::Second)) | (Some(Unit::Second), Some(Unit::GiB)) => {
                    // GiB * s = GiB (total data transferred)
                    UnitValue::new(a.value * b.value, Some(Unit::GiB))
                }
                (Some(Unit::GiBPerSecond), Some(Unit::Second)) | (Some(Unit::Second), Some(Unit::GiBPerSecond)) => {
                    // GiB/s * s = GiB
                    UnitValue::new(a.value * b.value, Some(Unit::GiB))
                }
                (Some(data_unit), Some(Unit::Second)) | (Some(Unit::Second), Some(data_unit)) 
                    if data_unit.unit_type() == UnitType::Data => {
                    // Data * time = data (total transferred)
                    UnitValue::new(a.value * b.value, Some(data_unit.clone()))
                }
                (Some(rate_unit), Some(Unit::Second)) | (Some(Unit::Second), Some(rate_unit))
                    if rate_unit.unit_type() == UnitType::DataRate => {
                    // Rate * time = data
                    let data_unit = match rate_unit {
                        Unit::BytesPerSecond => Unit::Byte,
                        Unit::KBPerSecond => Unit::KB,
                        Unit::MBPerSecond => Unit::MB,
                        Unit::GBPerSecond => Unit::GB,
                        Unit::TBPerSecond => Unit::TB,
                        Unit::KiBPerSecond => Unit::KiB,
                        Unit::MiBPerSecond => Unit::MiB,
                        Unit::GiBPerSecond => Unit::GiB,
                        Unit::TiBPerSecond => Unit::TiB,
                        _ => return false,
                    };
                    UnitValue::new(a.value * b.value, Some(data_unit))
                }
                (Some(unit), None) | (None, Some(unit)) => {
                    // Number * unit = unit
                    UnitValue::new(a.value * b.value, Some(unit.clone()))
                }
                (None, None) => UnitValue::new(a.value * b.value, None),
                _ => return false, // Unsupported unit combination
            }
        }
        Token::Divide => {
            match (&a.unit, &b.unit) {
                (Some(data_unit), Some(Unit::Second))
                    if data_unit.unit_type() == UnitType::Data => {
                    // Data / time = rate
                    let rate_unit = match data_unit {
                        Unit::Byte => Unit::BytesPerSecond,
                        Unit::KB => Unit::KBPerSecond,
                        Unit::MB => Unit::MBPerSecond,
                        Unit::GB => Unit::GBPerSecond,
                        Unit::TB => Unit::TBPerSecond,
                        Unit::KiB => Unit::KiBPerSecond,
                        Unit::MiB => Unit::MiBPerSecond,
                        Unit::GiB => Unit::GiBPerSecond,
                        Unit::TiB => Unit::TiBPerSecond,
                        _ => return false,
                    };
                    UnitValue::new(a.value / b.value, Some(rate_unit))
                }
                (Some(unit), None) => {
                    // unit / number = unit
                    UnitValue::new(a.value / b.value, Some(unit.clone()))
                }
                (None, None) => {
                    if b.value == 0.0 {
                        return false;
                    }
                    UnitValue::new(a.value / b.value, None)
                }
                _ => return false,
            }
        }
        _ => return false,
    };
    
    stack.push(result);
    true
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

fn format_number_with_commas(num: i64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 && *ch != '-' {
            result.push(',');
        }
        result.push(*ch);
    }
    
    result
}

fn format_decimal_with_commas(num: f64) -> String {
    let formatted = format!("{:.3}", num);
    let parts: Vec<&str> = formatted.split('.').collect();
    
    if parts.len() == 2 {
        let integer_part = parts[0].parse::<i64>().unwrap_or(0);
        let decimal_part = parts[1];
        format!("{}.{}", format_number_with_commas(integer_part), decimal_part)
    } else {
        formatted
    }
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
                    format_number_with_commas(*value as i64)
                } else {
                    format_decimal_with_commas(*value)
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

