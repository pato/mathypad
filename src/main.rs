use clap::{Arg, Command};
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

// Constants for better readability and maintainability
const TICK_RATE_MS: u64 = 250;
const MAX_INTEGER_FOR_FORMATTING: f64 = 1e15;
const FLOAT_EPSILON: f64 = f64::EPSILON;

struct App {
    text_lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
    results: Vec<Option<String>>,
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
            self.results[line_index] =
                evaluate_expression_with_context(line, &self.results, line_index);
        } else {
            // This should never happen in normal operation, but let's be defensive
            eprintln!(
                "Warning: Attempted to update result for invalid line index {}",
                line_index
            );
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("mathypad")
        .version("0.1.0")
        .about("A mathematical notepad with unit conversion support")
        .arg(
            Arg::new("expression")
                .help("Expression to evaluate (use -- before expression)")
                .last(true)
                .num_args(0..),
        )
        .get_matches();

    // Check if we have expressions to evaluate (one-shot mode)
    if let Some(expression_parts) = matches.get_many::<String>("expression") {
        let expression: String = expression_parts.cloned().collect::<Vec<String>>().join(" ");
        if !expression.is_empty() {
            run_one_shot_mode(&expression)?;
            return Ok(());
        }
    }

    // Run the interactive TUI mode
    run_interactive_mode()
}

fn run_one_shot_mode(expression: &str) -> Result<(), Box<dyn Error>> {
    // Print the expression with syntax highlighting
    print_formatted_expression(expression);

    // Evaluate the expression (no context for one-shot mode)
    if let Some(result) = evaluate_expression_with_context(expression, &[], 0) {
        println!(" = {}", result);
    } else {
        println!(" = (invalid expression)");
    }

    Ok(())
}

fn print_formatted_expression(text: &str) {
    // Use ANSI escape codes to print numbers in light blue and units in green
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

            // Check if it's a valid unit, keyword, or line reference
            if parse_line_reference(&word_text).is_some() {
                // Print line reference in magenta (ANSI color code 95)
                print!("\x1b[95m{}\x1b[0m", word_text);
            } else if word_text.to_lowercase() == "to" || word_text.to_lowercase() == "in" {
                // Print keywords in yellow (ANSI color code 93)
                print!("\x1b[93m{}\x1b[0m", word_text);
            } else if parse_unit(&word_text).is_some() {
                // Print units in green (ANSI color code 92)
                print!("\x1b[92m{}\x1b[0m", word_text);
            } else {
                print!("{}", word_text);
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
                // Print number in light blue (ANSI color code 94)
                print!("\x1b[94m{}\x1b[0m", number_text);
            } else {
                print!("{}", chars[start_pos]);
                current_pos = start_pos + 1;
            }
        } else if "+-*/()".contains(chars[current_pos]) {
            // Print operators in cyan (ANSI color code 96)
            print!("\x1b[96m{}\x1b[0m", chars[current_pos]);
            current_pos += 1;
        } else {
            print!("{}", chars[current_pos]);
            current_pos += 1;
        }
    }
}

fn run_interactive_mode() -> Result<(), Box<dyn Error>> {
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

fn evaluate_expression_with_context(
    text: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<String> {
    // Find the longest valid mathematical expression in the text
    let expressions = find_math_expressions(text);

    for expr in expressions {
        // Try unit-aware parsing first
        if let Some(unit_value) =
            parse_and_evaluate_with_context(&expr, previous_results, current_line)
        {
            return Some(unit_value.format());
        }
        // Then try simple parsing
        if let Some(simple_result) = parse_and_evaluate_simple(&expr) {
            let unit_value = UnitValue::new(simple_result, None);
            return Some(unit_value.format());
        }
    }

    None
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
    let mut expressions = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    // First check if the entire text is a valid math expression
    let trimmed_text = text.trim();
    if !trimmed_text.is_empty() && is_valid_math_expression(trimmed_text) {
        expressions.push(trimmed_text.to_string());
        // If the entire text is valid, don't look for sub-expressions
        return expressions;
    }

    // Early validation - check for obviously invalid expressions
    if has_invalid_expression_structure(trimmed_text) {
        return expressions; // Return empty - invalid expression
    }

    // Then look for sub-expressions only if the entire text is NOT valid
    for start in 0..chars.len() {
        if chars[start].is_ascii_digit() || chars[start] == '(' {
            for end in start + 1..=chars.len() {
                let candidate = chars[start..end].iter().collect::<String>();
                let trimmed_candidate = extract_math_portion(&candidate);

                if !trimmed_candidate.is_empty()
                    && is_valid_math_expression(&trimmed_candidate)
                    && trimmed_candidate != trimmed_text
                {
                    // Don't re-add the full text
                    expressions.push(trimmed_candidate);
                }
            }
        }
    }

    // Sort by complexity (length and operator count) descending
    expressions.sort_by(|a, b| {
        let complexity_a = a.len() + a.chars().filter(|c| "+-*/()".contains(*c)).count() * 2;
        let complexity_b = b.len() + b.chars().filter(|c| "+-*/()".contains(*c)).count() * 2;
        complexity_b.cmp(&complexity_a)
    });

    // Remove duplicates and sub-expressions
    let mut filtered_expressions = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for expr in &expressions {
        if seen.insert(expr.clone()) {
            let mut is_subexpression = false;
            for other_expr in &expressions {
                if other_expr != expr && other_expr.len() > expr.len() && other_expr.contains(expr)
                {
                    is_subexpression = true;
                    break;
                }
            }
            if !is_subexpression {
                filtered_expressions.push(expr.clone());
            }
        }
    }

    filtered_expressions
}

fn has_invalid_expression_structure(text: &str) -> bool {
    // Check if the text ends with an operator OR starts with an operator (except minus for negation)
    let text_ends_with_operator = {
        let last_char = text.chars().rev().find(|c| !c.is_whitespace());
        matches!(last_char, Some('+') | Some('-') | Some('*') | Some('/'))
    };

    let text_starts_with_operator = {
        let first_char = text.chars().find(|c| !c.is_whitespace());
        matches!(first_char, Some('*') | Some('/') | Some('+'))
    };

    // Check for unbalanced parentheses
    let has_unbalanced_parens = {
        let mut paren_count = 0;
        for ch in text.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        paren_count != 0
    };

    text_ends_with_operator || text_starts_with_operator || has_unbalanced_parens
}

fn extract_math_portion(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut math_end = 0;
    let mut found_digit = false;
    let mut i = 0;
    let mut last_was_operator = false;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '0'..='9' | '.' | ',' => {
                found_digit = true;
                math_end = i + 1;
                last_was_operator = false;
            }
            '+' | '-' | '*' | '/' => {
                if found_digit {
                    last_was_operator = true;
                    // Don't update math_end yet - wait for next operand
                }
            }
            '(' => {
                math_end = i + 1;
                last_was_operator = false;
            }
            ')' => {
                math_end = i + 1;
                last_was_operator = false;
                // Don't break here - continue to see if there are more operators
            }
            ' ' => {
                // Continue, space is okay
                // Don't change last_was_operator
            }
            _ => {
                if ch.is_ascii_alphabetic() {
                    // Check if this starts a complete known unit (with word boundaries)
                    let remaining = &text[i..];
                    let mut word_end = i;
                    for (j, word_char) in remaining.chars().enumerate() {
                        if word_char.is_ascii_alphabetic() || word_char == '/' {
                            word_end = i + j + 1;
                        } else {
                            break;
                        }
                    }
                    let potential_word = &text[i..word_end];

                    // Only treat as unit if it's a complete unit word, "to", "in", or line reference
                    if (parse_unit(potential_word).is_some()
                        || potential_word.to_lowercase() == "to"
                        || potential_word.to_lowercase() == "in"
                        || parse_line_reference(potential_word).is_some())
                        && (word_end >= text.len()
                            || !text.chars().nth(word_end).unwrap().is_ascii_alphabetic())
                    {
                        math_end = word_end;
                        last_was_operator = false;
                        // Skip to the end of this word
                        i = word_end;
                        continue;
                    } else {
                        // Unknown or partial word, end of math expression here
                        break;
                    }
                } else {
                    // Other character, end of math expression
                    break;
                }
            }
        }

        // If we just processed an operator, update math_end only if we find more content
        if last_was_operator && i + 1 < chars.len() {
            // Look ahead for more content
            let mut j = i + 1;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            if j < chars.len()
                && (chars[j].is_ascii_digit() || chars[j] == '(' || chars[j].is_ascii_alphabetic())
            {
                math_end = i + 1;
            }
        }

        i += 1;
    }

    chars[..math_end]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

fn is_valid_math_expression(expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return false;
    }

    let mut has_number = false;
    // let mut has_operator = false;
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
                while i < chars.len()
                    && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',')
                {
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
                    if parse_unit(&unit_str).is_none()
                        && unit_str.to_lowercase() != "to"
                        && unit_str.to_lowercase() != "in"
                        && parse_line_reference(&unit_str).is_none()
                    {
                        // Not a recognized unit or line reference, rewind
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
                    // For potential line references, also include digits
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" || word.to_lowercase() == "in" {
                        prev_was_operator = true;
                    } else if parse_line_reference(&word).is_some() {
                        // Valid line reference, acts like a number
                        has_number = true;
                        prev_was_operator = false;
                    } else if parse_unit(&word).is_some() {
                        // Valid unit, continue
                        prev_was_operator = false;
                    } else {
                        // Unknown word - treat as the end of the expression
                        // Check if what we have so far is valid
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
    paren_count == 0 && has_number && !prev_was_operator
}

fn parse_and_evaluate_with_context(
    expr: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    let tokens = tokenize_with_units(expr)?;
    evaluate_tokens_with_units_and_context(&tokens, previous_results, current_line)
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
                while i < chars.len()
                    && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',')
                {
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
                    // Check if it's actually "to" and not part of a longer word like "tb"
                    if i + 2 >= chars.len() || !chars[i + 2].is_ascii_alphabetic() {
                        // Skip whitespace after "to"
                        i += 2;
                        while i < chars.len() && chars[i] == ' ' {
                            i += 1;
                        }
                        tokens.push(Token::To);
                    } else {
                        // Fall through to general alphabetic handling
                        let unit_start = i;
                        while i < chars.len()
                            && (chars[i].is_ascii_alphabetic()
                                || chars[i].is_ascii_digit()
                                || chars[i] == '/')
                        {
                            i += 1;
                        }

                        let word: String = chars[unit_start..i].iter().collect();
                        if word.to_lowercase() == "to" {
                            tokens.push(Token::To);
                        } else if word.to_lowercase() == "in" {
                            tokens.push(Token::In);
                        } else if let Some(line_num) = parse_line_reference(&word) {
                            tokens.push(Token::LineReference(line_num));
                        } else if let Some(unit) = parse_unit(&word) {
                            tokens.push(Token::NumberWithUnit(1.0, unit));
                        } else {
                            return None;
                        }
                    }
                } else {
                    // Fall through to general alphabetic handling
                    let unit_start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if word.to_lowercase() == "in" {
                        tokens.push(Token::In);
                    } else if let Some(line_num) = parse_line_reference(&word) {
                        tokens.push(Token::LineReference(line_num));
                    } else if let Some(unit) = parse_unit(&word) {
                        tokens.push(Token::NumberWithUnit(1.0, unit));
                    } else {
                        return None;
                    }
                }
            }
            'i' | 'I' => {
                // Check for "in" keyword
                if i + 1 < chars.len() && chars[i + 1].to_lowercase().next() == Some('n') {
                    // Check if it's actually "in" and not part of a longer word
                    if i + 2 >= chars.len() || !chars[i + 2].is_ascii_alphabetic() {
                        // Skip whitespace after "in"
                        i += 2;
                        while i < chars.len() && chars[i] == ' ' {
                            i += 1;
                        }
                        tokens.push(Token::In);
                    } else {
                        // Fall through to general alphabetic handling
                        let unit_start = i;
                        while i < chars.len()
                            && (chars[i].is_ascii_alphabetic()
                                || chars[i].is_ascii_digit()
                                || chars[i] == '/')
                        {
                            i += 1;
                        }

                        let word: String = chars[unit_start..i].iter().collect();
                        if word.to_lowercase() == "to" {
                            tokens.push(Token::To);
                        } else if word.to_lowercase() == "in" {
                            tokens.push(Token::In);
                        } else if let Some(line_num) = parse_line_reference(&word) {
                            tokens.push(Token::LineReference(line_num));
                        } else if let Some(unit) = parse_unit(&word) {
                            tokens.push(Token::NumberWithUnit(1.0, unit));
                        } else {
                            return None;
                        }
                    }
                } else {
                    // Fall through to general alphabetic handling
                    let unit_start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if word.to_lowercase() == "in" {
                        tokens.push(Token::In);
                    } else if let Some(line_num) = parse_line_reference(&word) {
                        tokens.push(Token::LineReference(line_num));
                    } else if let Some(unit) = parse_unit(&word) {
                        tokens.push(Token::NumberWithUnit(1.0, unit));
                    } else {
                        return None;
                    }
                }
            }
            _ => {
                if ch.is_ascii_alphabetic() {
                    // Could be a unit, keyword, or line reference
                    let unit_start = i;

                    // For potential line references, also include digits
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if word.to_lowercase() == "in" {
                        tokens.push(Token::In);
                    } else if let Some(line_num) = parse_line_reference(&word) {
                        tokens.push(Token::LineReference(line_num));
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
                    let converted_value = target_unit.clone().from_base_value(base_value);
                    Some(UnitValue::new(converted_value, Some(target_unit.clone())))
                } else {
                    None // Can't convert between different unit types
                }
            }
            None => None, // No unit to convert from
        }
    }

    fn format(&self) -> String {
        let formatted_value =
            if self.value.fract() == 0.0 && self.value.abs() < MAX_INTEGER_FOR_FORMATTING {
                format_number_with_commas(self.value as i64)
            } else {
                format_decimal_with_commas(self.value)
            };

        match &self.unit {
            Some(unit) => format!("{} {}", formatted_value, unit.display_name()),
            None => formatted_value,
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
    To,                   // for conversions like "to KiB"
    In,                   // for conversions like "in KiB"
    LineReference(usize), // for referencing other lines like "line1", "line2"
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
    PB, // Petabyte
    EB, // Exabyte

    // Data units (base 2)
    KiB, // Kibibyte
    MiB, // Mebibyte
    GiB, // Gibibyte
    TiB, // Tebibyte
    PiB, // Pebibyte
    EiB, // Exbibyte

    // Request/Query count (base unit: requests)
    Request,
    Query,

    // Data rate units
    BytesPerSecond,
    KBPerSecond,
    MBPerSecond,
    GBPerSecond,
    TBPerSecond,
    PBPerSecond,
    EBPerSecond,
    KiBPerSecond,
    MiBPerSecond,
    GiBPerSecond,
    TiBPerSecond,
    PiBPerSecond,
    EiBPerSecond,

    // Request/Query rate units (base: requests per second)
    RequestsPerSecond,
    RequestsPerMinute,
    RequestsPerHour,
    QueriesPerSecond,
    QueriesPerMinute,
    QueriesPerHour,
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
            Unit::PB => value * 1_000_000_000_000_000.0,
            Unit::EB => value * 1_000_000_000_000_000_000.0,

            // Data units base 2 (convert to bytes)
            Unit::KiB => value * 1_024.0,
            Unit::MiB => value * 1_048_576.0,
            Unit::GiB => value * 1_073_741_824.0,
            Unit::TiB => value * 1_099_511_627_776.0,
            Unit::PiB => value * 1_125_899_906_842_624.0,
            Unit::EiB => value * 1_152_921_504_606_846_976.0,

            // Request/Query count (base unit: requests/queries)
            Unit::Request => value,
            Unit::Query => value, // Queries and requests are equivalent

            // Data rate units (convert to bytes per second)
            Unit::BytesPerSecond => value,
            Unit::KBPerSecond => value * 1_000.0,
            Unit::MBPerSecond => value * 1_000_000.0,
            Unit::GBPerSecond => value * 1_000_000_000.0,
            Unit::TBPerSecond => value * 1_000_000_000_000.0,
            Unit::PBPerSecond => value * 1_000_000_000_000_000.0,
            Unit::EBPerSecond => value * 1_000_000_000_000_000_000.0,
            Unit::KiBPerSecond => value * 1_024.0,
            Unit::MiBPerSecond => value * 1_048_576.0,
            Unit::GiBPerSecond => value * 1_073_741_824.0,
            Unit::TiBPerSecond => value * 1_099_511_627_776.0,
            Unit::PiBPerSecond => value * 1_125_899_906_842_624.0,
            Unit::EiBPerSecond => value * 1_152_921_504_606_846_976.0,

            // Request/Query rate units (convert to requests per second)
            Unit::RequestsPerSecond => value,
            Unit::RequestsPerMinute => value / 60.0,
            Unit::RequestsPerHour => value / 3600.0,
            Unit::QueriesPerSecond => value, // QPS = requests per second
            Unit::QueriesPerMinute => value / 60.0,
            Unit::QueriesPerHour => value / 3600.0,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_base_value(self, base_value: f64) -> f64 {
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
            Unit::PB => base_value / 1_000_000_000_000_000.0,
            Unit::EB => base_value / 1_000_000_000_000_000_000.0,

            // Data units base 2 (from bytes)
            Unit::KiB => base_value / 1_024.0,
            Unit::MiB => base_value / 1_048_576.0,
            Unit::GiB => base_value / 1_073_741_824.0,
            Unit::TiB => base_value / 1_099_511_627_776.0,
            Unit::PiB => base_value / 1_125_899_906_842_624.0,
            Unit::EiB => base_value / 1_152_921_504_606_846_976.0,

            // Request/Query count (from requests/queries)
            Unit::Request => base_value,
            Unit::Query => base_value,

            // Data rate units (from bytes per second)
            Unit::BytesPerSecond => base_value,
            Unit::KBPerSecond => base_value / 1_000.0,
            Unit::MBPerSecond => base_value / 1_000_000.0,
            Unit::GBPerSecond => base_value / 1_000_000_000.0,
            Unit::TBPerSecond => base_value / 1_000_000_000_000.0,
            Unit::PBPerSecond => base_value / 1_000_000_000_000_000.0,
            Unit::EBPerSecond => base_value / 1_000_000_000_000_000_000.0,
            Unit::KiBPerSecond => base_value / 1_024.0,
            Unit::MiBPerSecond => base_value / 1_048_576.0,
            Unit::GiBPerSecond => base_value / 1_073_741_824.0,
            Unit::TiBPerSecond => base_value / 1_099_511_627_776.0,
            Unit::PiBPerSecond => base_value / 1_125_899_906_842_624.0,
            Unit::EiBPerSecond => base_value / 1_152_921_504_606_846_976.0,

            // Request/Query rate units (from requests per second)
            Unit::RequestsPerSecond => base_value,
            Unit::RequestsPerMinute => base_value * 60.0,
            Unit::RequestsPerHour => base_value * 3600.0,
            Unit::QueriesPerSecond => base_value,
            Unit::QueriesPerMinute => base_value * 60.0,
            Unit::QueriesPerHour => base_value * 3600.0,
        }
    }

    fn unit_type(&self) -> UnitType {
        match self {
            Unit::Second | Unit::Minute | Unit::Hour | Unit::Day => UnitType::Time,
            Unit::Byte
            | Unit::KB
            | Unit::MB
            | Unit::GB
            | Unit::TB
            | Unit::PB
            | Unit::EB
            | Unit::KiB
            | Unit::MiB
            | Unit::GiB
            | Unit::TiB
            | Unit::PiB
            | Unit::EiB => UnitType::Data,
            Unit::Request | Unit::Query => UnitType::Request,
            Unit::BytesPerSecond
            | Unit::KBPerSecond
            | Unit::MBPerSecond
            | Unit::GBPerSecond
            | Unit::TBPerSecond
            | Unit::PBPerSecond
            | Unit::EBPerSecond
            | Unit::KiBPerSecond
            | Unit::MiBPerSecond
            | Unit::GiBPerSecond
            | Unit::TiBPerSecond
            | Unit::PiBPerSecond
            | Unit::EiBPerSecond => UnitType::DataRate,
            Unit::RequestsPerSecond
            | Unit::RequestsPerMinute
            | Unit::RequestsPerHour
            | Unit::QueriesPerSecond
            | Unit::QueriesPerMinute
            | Unit::QueriesPerHour => UnitType::RequestRate,
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
            Unit::PB => "PB",
            Unit::EB => "EB",
            Unit::KiB => "KiB",
            Unit::MiB => "MiB",
            Unit::GiB => "GiB",
            Unit::TiB => "TiB",
            Unit::PiB => "PiB",
            Unit::EiB => "EiB",
            Unit::Request => "req",
            Unit::Query => "query",
            Unit::BytesPerSecond => "B/s",
            Unit::KBPerSecond => "KB/s",
            Unit::MBPerSecond => "MB/s",
            Unit::GBPerSecond => "GB/s",
            Unit::TBPerSecond => "TB/s",
            Unit::PBPerSecond => "PB/s",
            Unit::EBPerSecond => "EB/s",
            Unit::KiBPerSecond => "KiB/s",
            Unit::MiBPerSecond => "MiB/s",
            Unit::GiBPerSecond => "GiB/s",
            Unit::TiBPerSecond => "TiB/s",
            Unit::PiBPerSecond => "PiB/s",
            Unit::EiBPerSecond => "EiB/s",
            Unit::RequestsPerSecond => "req/s",
            Unit::RequestsPerMinute => "req/min",
            Unit::RequestsPerHour => "req/h",
            Unit::QueriesPerSecond => "QPS",
            Unit::QueriesPerMinute => "QPM",
            Unit::QueriesPerHour => "QPH",
        }
    }

    fn to_rate_unit(&self) -> Result<Unit, ()> {
        match self {
            Unit::Byte => Ok(Unit::BytesPerSecond),
            Unit::KB => Ok(Unit::KBPerSecond),
            Unit::MB => Ok(Unit::MBPerSecond),
            Unit::GB => Ok(Unit::GBPerSecond),
            Unit::TB => Ok(Unit::TBPerSecond),
            Unit::PB => Ok(Unit::PBPerSecond),
            Unit::EB => Ok(Unit::EBPerSecond),
            Unit::KiB => Ok(Unit::KiBPerSecond),
            Unit::MiB => Ok(Unit::MiBPerSecond),
            Unit::GiB => Ok(Unit::GiBPerSecond),
            Unit::TiB => Ok(Unit::TiBPerSecond),
            Unit::PiB => Ok(Unit::PiBPerSecond),
            Unit::EiB => Ok(Unit::EiBPerSecond),
            Unit::Request => Ok(Unit::RequestsPerSecond),
            Unit::Query => Ok(Unit::QueriesPerSecond),
            _ => Err(()),
        }
    }

    fn to_data_unit(&self) -> Result<Unit, ()> {
        match self {
            Unit::BytesPerSecond => Ok(Unit::Byte),
            Unit::KBPerSecond => Ok(Unit::KB),
            Unit::MBPerSecond => Ok(Unit::MB),
            Unit::GBPerSecond => Ok(Unit::GB),
            Unit::TBPerSecond => Ok(Unit::TB),
            Unit::PBPerSecond => Ok(Unit::PB),
            Unit::EBPerSecond => Ok(Unit::EB),
            Unit::KiBPerSecond => Ok(Unit::KiB),
            Unit::MiBPerSecond => Ok(Unit::MiB),
            Unit::GiBPerSecond => Ok(Unit::GiB),
            Unit::TiBPerSecond => Ok(Unit::TiB),
            Unit::PiBPerSecond => Ok(Unit::PiB),
            Unit::EiBPerSecond => Ok(Unit::EiB),
            _ => Err(()),
        }
    }

    fn to_request_unit(&self) -> Result<Unit, ()> {
        match self {
            Unit::RequestsPerSecond => Ok(Unit::Request),
            Unit::RequestsPerMinute => Ok(Unit::Request),
            Unit::RequestsPerHour => Ok(Unit::Request),
            Unit::QueriesPerSecond => Ok(Unit::Query),
            Unit::QueriesPerMinute => Ok(Unit::Query),
            Unit::QueriesPerHour => Ok(Unit::Query),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UnitType {
    Time,
    Data,
    Request,
    DataRate,
    RequestRate,
}

fn parse_line_reference(text: &str) -> Option<usize> {
    let text_lower = text.to_lowercase();
    if let Some(number_part) = text_lower.strip_prefix("line") {
        if let Ok(line_num) = number_part.parse::<usize>() {
            if line_num > 0 {
                return Some(line_num - 1); // Convert to 0-based indexing
            }
        }
    }
    None
}

fn parse_unit(text: &str) -> Option<Unit> {
    match text.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(Unit::Second),
        "min" | "minute" | "minutes" => Some(Unit::Minute),
        "h" | "hr" | "hour" | "hours" => Some(Unit::Hour),
        "day" | "days" => Some(Unit::Day), // Remove single "d" to avoid conflicts

        "b" | "byte" | "bytes" => Some(Unit::Byte),
        "kb" => Some(Unit::KB),
        "mb" => Some(Unit::MB),
        "gb" => Some(Unit::GB),
        "tb" => Some(Unit::TB),
        "pb" => Some(Unit::PB),
        "eb" => Some(Unit::EB),

        "kib" => Some(Unit::KiB),
        "mib" => Some(Unit::MiB),
        "gib" => Some(Unit::GiB),
        "tib" => Some(Unit::TiB),
        "pib" => Some(Unit::PiB),
        "eib" => Some(Unit::EiB),

        "req" | "request" | "requests" => Some(Unit::Request),
        "query" | "queries" => Some(Unit::Query),

        "b/s" | "bytes/s" | "bps" => Some(Unit::BytesPerSecond),
        "kb/s" | "kbps" => Some(Unit::KBPerSecond),
        "mb/s" | "mbps" => Some(Unit::MBPerSecond),
        "gb/s" | "gbps" => Some(Unit::GBPerSecond),
        "tb/s" | "tbps" => Some(Unit::TBPerSecond),
        "pb/s" | "pbps" => Some(Unit::PBPerSecond),
        "eb/s" | "ebps" => Some(Unit::EBPerSecond),
        "kib/s" | "kibps" => Some(Unit::KiBPerSecond),
        "mib/s" | "mibps" => Some(Unit::MiBPerSecond),
        "gib/s" | "gibps" => Some(Unit::GiBPerSecond),
        "tib/s" | "tibps" => Some(Unit::TiBPerSecond),
        "pib/s" | "pibps" => Some(Unit::PiBPerSecond),
        "eib/s" | "eibps" => Some(Unit::EiBPerSecond),

        "req/s" | "requests/s" | "rps" => Some(Unit::RequestsPerSecond),
        "req/min" | "requests/min" | "rpm" => Some(Unit::RequestsPerMinute),
        "req/h" | "req/hour" | "requests/h" | "requests/hour" | "rph" => {
            Some(Unit::RequestsPerHour)
        }
        "qps" | "queries/s" | "queries/sec" => Some(Unit::QueriesPerSecond),
        "qpm" | "queries/min" | "queries/minute" => Some(Unit::QueriesPerMinute),
        "qph" | "queries/h" | "queries/hour" => Some(Unit::QueriesPerHour),

        _ => None,
    }
}

fn evaluate_tokens_with_units_and_context(
    tokens: &[Token],
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if tokens.is_empty() {
        return None;
    }

    // Handle simple conversion expressions like "1 GiB to KiB" (only if it's the entire expression)
    if tokens.len() == 3 {
        if let (
            Token::NumberWithUnit(value, from_unit),
            Token::To,
            Token::NumberWithUnit(_, to_unit),
        ) = (&tokens[0], &tokens[1], &tokens[2])
        {
            let unit_value = UnitValue::new(*value, Some(from_unit.clone()));
            return unit_value.to_unit(to_unit);
        }
    }

    // Check if we have an "in" or "to" conversion request at the end
    let mut target_unit_for_conversion = None;
    let mut evaluation_tokens = tokens;

    // Look for "in" or "to" followed by a unit at the end
    for i in 0..tokens.len().saturating_sub(1) {
        if let Token::In | Token::To = &tokens[i] {
            // Look for unit after "in" or "to"
            for j in (i + 1)..tokens.len() {
                if let Token::NumberWithUnit(_, unit) = &tokens[j] {
                    target_unit_for_conversion = Some(unit.clone());
                    evaluation_tokens = &tokens[..i]; // Evaluate everything before "in"/"to"
                    break;
                }
            }
            break;
        }
    }

    // Handle simple arithmetic with units
    let mut operator_stack = Vec::new();
    let mut value_stack = Vec::new();

    for token in evaluation_tokens {
        match token {
            Token::Number(n) => {
                value_stack.push(UnitValue::new(*n, None));
            }
            Token::NumberWithUnit(value, unit) => {
                value_stack.push(UnitValue::new(*value, Some(unit.clone())));
            }
            Token::LineReference(line_index) => {
                // Resolve line reference to its calculated result
                if let Some(line_result) =
                    resolve_line_reference(*line_index, previous_results, current_line)
                {
                    value_stack.push(line_result);
                } else {
                    return None; // Invalid or circular reference
                }
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
        let mut result = value_stack.pop().unwrap();

        // If we have a target unit for conversion, convert the result
        if let Some(target_unit) = target_unit_for_conversion {
            if let Some(converted) = result.to_unit(&target_unit) {
                result = converted;
            }
        }

        Some(result)
    } else {
        None
    }
}

fn resolve_line_reference(
    line_index: usize,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    // Prevent circular references
    if line_index >= current_line {
        return None;
    }

    // Check if the referenced line exists and has a result
    if line_index < previous_results.len() {
        if let Some(result_str) = &previous_results[line_index] {
            // Parse the result string back into a UnitValue
            return parse_result_string(result_str);
        }
    }

    None
}

fn parse_result_string(result_str: &str) -> Option<UnitValue> {
    // Parse a result string like "14 GiB" or "42" back into a UnitValue
    let parts: Vec<&str> = result_str.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    // Try to parse the first part as a number
    let number_str = parts[0].replace(",", ""); // Remove commas
    if let Ok(value) = number_str.parse::<f64>() {
        if parts.len() == 1 {
            // Just a number
            return Some(UnitValue::new(value, None));
        } else if parts.len() == 2 {
            // Number with unit
            if let Some(unit) = parse_unit(parts[1]) {
                return Some(UnitValue::new(value, Some(unit)));
            }
        }
    }

    None
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

                        // Choose the smaller unit (larger value) for the result
                        let result_unit = if unit_a.to_base_value(1.0) < unit_b.to_base_value(1.0) {
                            unit_a
                        } else {
                            unit_b
                        };
                        let result_value = result_unit.clone().from_base_value(result_base);
                        UnitValue::new(result_value, Some(result_unit.clone()))
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

                        // Choose the smaller unit (larger value) for the result
                        let result_unit = if unit_a.to_base_value(1.0) < unit_b.to_base_value(1.0) {
                            unit_a
                        } else {
                            unit_b
                        };
                        let result_value = result_unit.clone().from_base_value(result_base);
                        UnitValue::new(result_value, Some(result_unit.clone()))
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
                // Time * Rate = Data (convert time to seconds first)
                (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                    if time_unit.unit_type() == UnitType::Time
                        && rate_unit.unit_type() == UnitType::DataRate =>
                {
                    // Determine which value is time and which is rate
                    let (time_value, time_u, rate_value, rate_u) =
                        if time_unit.unit_type() == UnitType::Time {
                            (a.value, time_unit, b.value, rate_unit)
                        } else {
                            (b.value, time_unit, a.value, rate_unit)
                        };

                    // Convert time to seconds
                    let time_in_seconds = time_u.to_base_value(time_value);

                    // Rate * time = data
                    let data_unit = match rate_u.to_data_unit() {
                        Ok(unit) => unit,
                        Err(()) => return false,
                    };
                    UnitValue::new(rate_value * time_in_seconds, Some(data_unit))
                }
                // Time * RequestRate = Requests (convert time to seconds first)
                (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                    if time_unit.unit_type() == UnitType::Time
                        && rate_unit.unit_type() == UnitType::RequestRate =>
                {
                    // Determine which value is time and which is rate
                    let (time_value, time_u, rate_value, rate_u) =
                        if time_unit.unit_type() == UnitType::Time {
                            (a.value, time_unit, b.value, rate_unit)
                        } else {
                            (b.value, time_unit, a.value, rate_unit)
                        };

                    // Convert time to seconds
                    let time_in_seconds = time_u.to_base_value(time_value);

                    // RequestRate * time = requests
                    let request_unit = match rate_u.to_request_unit() {
                        Ok(unit) => unit,
                        Err(()) => return false,
                    };
                    UnitValue::new(rate_value * time_in_seconds, Some(request_unit))
                }
                // Data * Time = Data (total transferred) - for specific data units
                (Some(data_unit), Some(time_unit)) | (Some(time_unit), Some(data_unit))
                    if data_unit.unit_type() == UnitType::Data
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    UnitValue::new(a.value * b.value, Some(data_unit.clone()))
                }
                (Some(rate_unit), Some(Unit::Second)) | (Some(Unit::Second), Some(rate_unit))
                    if rate_unit.unit_type() == UnitType::DataRate =>
                {
                    let data_unit = match rate_unit.to_data_unit() {
                        Ok(unit) => unit,
                        Err(()) => return false,
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
                (Some(data_unit), Some(time_unit))
                    if data_unit.unit_type() == UnitType::Data
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // Data / time = rate
                    // Convert time to seconds first
                    let time_in_seconds = time_unit.to_base_value(b.value);
                    let rate_unit = match data_unit.to_rate_unit() {
                        Ok(unit) => unit,
                        Err(()) => return false,
                    };
                    UnitValue::new(a.value / time_in_seconds, Some(rate_unit))
                }
                (Some(request_unit), Some(time_unit))
                    if request_unit.unit_type() == UnitType::Request
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // Requests / time = request rate
                    // Convert time to seconds first
                    let time_in_seconds = time_unit.to_base_value(b.value);
                    let rate_unit = match request_unit.to_rate_unit() {
                        Ok(unit) => unit,
                        Err(()) => return false,
                    };
                    UnitValue::new(a.value / time_in_seconds, Some(rate_unit))
                }
                (Some(rate_unit), Some(time_unit))
                    if rate_unit.unit_type() == UnitType::RequestRate
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // RequestRate / time = RequestRate (rate per unit time)
                    // This is a more complex case - dividing a rate by time
                    // For now, we'll treat this as invalid
                    return false;
                }
                (Some(unit), None) => {
                    // unit / number = unit
                    if b.value.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    UnitValue::new(a.value / b.value, Some(unit.clone()))
                }
                (None, None) => {
                    if b.value.abs() < FLOAT_EPSILON {
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
            if b.abs() < f64::EPSILON {
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
        format!(
            "{}.{}",
            format_number_with_commas(integer_part),
            decimal_part
        )
    } else {
        formatted
    }
}

fn parse_colors(text: &str) -> Vec<Span> {
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

            // Check if it's a valid unit, keyword, or line reference
            if parse_line_reference(&word_text).is_some() {
                spans.push(Span::styled(word_text, Style::default().fg(Color::Magenta)));
            } else if word_text.to_lowercase() == "to" || word_text.to_lowercase() == "in" {
                spans.push(Span::styled(word_text, Style::default().fg(Color::Yellow)));
            } else if parse_unit(&word_text).is_some() {
                spans.push(Span::styled(word_text, Style::default().fg(Color::Green)));
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
        } else if "+-*/()".contains(chars[current_pos]) {
            // Handle operators and parentheses
            spans.push(Span::styled(
                chars[current_pos].to_string(),
                Style::default().fg(Color::Cyan),
            ));
            current_pos += 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to evaluate expressions for testing
    fn evaluate_test_expression(input: &str) -> Option<String> {
        evaluate_expression_with_context(input, &[], 0)
    }

    // Helper function to get unit conversion results for testing
    fn evaluate_with_unit_info(input: &str) -> Option<UnitValue> {
        let expressions = find_math_expressions(input);
        for expr in expressions {
            if let Some(result) = parse_and_evaluate_with_context(&expr, &[], 0) {
                return Some(result);
            }
        }
        None
    }

    #[test]
    fn test_basic_arithmetic() {
        // Basic operations
        assert_eq!(evaluate_test_expression("2 + 3"), Some("5".to_string()));
        assert_eq!(evaluate_test_expression("10 - 4"), Some("6".to_string()));
        assert_eq!(evaluate_test_expression("6 * 7"), Some("42".to_string()));
        assert_eq!(evaluate_test_expression("15 / 3"), Some("5".to_string()));

        // Order of operations
        assert_eq!(
            evaluate_test_expression("2 + 3 * 4"),
            Some("14".to_string())
        );
        assert_eq!(
            evaluate_test_expression("(2 + 3) * 4"),
            Some("20".to_string())
        );
        assert_eq!(
            evaluate_test_expression("10 - 2 * 3"),
            Some("4".to_string())
        );

        // Decimal numbers
        assert_eq!(evaluate_test_expression("1.5 + 2.5"), Some("4".to_string()));
        assert_eq!(
            evaluate_test_expression("3.14 * 2"),
            Some("6.280".to_string())
        );

        // Numbers with commas
        assert_eq!(
            evaluate_test_expression("1,000 + 500"),
            Some("1,500".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1,234,567 / 1000"),
            Some("1,234.567".to_string())
        );
    }

    #[test]
    fn test_inline_expressions() {
        // Test expressions within text
        assert_eq!(
            evaluate_test_expression("The result is 5 + 3"),
            Some("8".to_string())
        );
        assert_eq!(
            evaluate_test_expression("Cost: 100 * 12 dollars"),
            Some("1,200".to_string())
        );
        assert_eq!(
            evaluate_test_expression("Total (10 + 20) items"),
            Some("30".to_string())
        );
    }

    #[test]
    fn test_unit_conversions() {
        // Data unit conversions (base 2)
        let result = evaluate_with_unit_info("1 GiB to KiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1048576.0).abs() < 0.001);

        let result = evaluate_with_unit_info("1 TiB to GiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1024.0).abs() < 0.001);

        let result = evaluate_with_unit_info("2048 KiB to MiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 2.0).abs() < 0.001);

        // Data unit conversions (base 10)
        let result = evaluate_with_unit_info("1 GB to MB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1000.0).abs() < 0.001);

        let result = evaluate_with_unit_info("5000 MB to GB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 5.0).abs() < 0.001);

        // Time unit conversions
        let result = evaluate_with_unit_info("1 hour to minutes");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 60.0).abs() < 0.001);

        let result = evaluate_with_unit_info("120 seconds to minutes");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_arithmetic_with_units() {
        // Data rate * time = data
        assert_eq!(
            evaluate_test_expression("50 GiB/s * 2 s"),
            Some("100 GiB".to_string())
        );

        assert_eq!(
            evaluate_test_expression("1 hour * 10 GiB/s"),
            Some("36,000 GiB".to_string())
        );

        // Data / time = rate
        assert_eq!(
            evaluate_test_expression("100 GiB / 10 s"),
            Some("10 GiB/s".to_string())
        );

        // Same unit addition/subtraction
        assert_eq!(
            evaluate_test_expression("1 GiB + 512 MiB"),
            Some("1,536 MiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("2 hours + 30 minutes"),
            Some("150 min".to_string())
        );
    }

    #[test]
    fn test_mixed_unit_types() {
        // Base 10 vs Base 2 data units
        let result = evaluate_with_unit_info("1 GB to GiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        // 1 GB = 1,000,000,000 bytes = ~0.931 GiB
        assert!((unit_val.value - 0.9313225746).abs() < 0.0001);

        let result = evaluate_with_unit_info("1 GiB to GB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        // 1 GiB = 1,073,741,824 bytes = ~1.074 GB
        assert!((unit_val.value - 1.073741824).abs() < 0.0001);
    }

    #[test]
    fn test_complex_expressions() {
        // Complex arithmetic
        assert_eq!(
            evaluate_test_expression("(10 + 5) * 2 - 8 / 4"),
            Some("28".to_string())
        );
        assert_eq!(
            evaluate_test_expression("100 / (5 + 5) + 3 * 2"),
            Some("16".to_string())
        );

        // Large numbers with commas
        assert_eq!(
            evaluate_test_expression("1,000,000 + 500,000"),
            Some("1,500,000".to_string())
        );
        assert_eq!(
            evaluate_test_expression("2,500 * 1,000"),
            Some("2,500,000".to_string())
        );

        // Complex unit expressions
        assert_eq!(
            evaluate_test_expression("Transfer: 5 GiB/s * 10 minutes"),
            Some("3,000 GiB".to_string())
        );
    }

    #[test]
    fn test_edge_cases() {
        // Division by zero
        println!("Testing 5 / 0: {:?}", evaluate_test_expression("5 / 0"));
        assert_eq!(evaluate_test_expression("5 / 0"), None);

        // Invalid expressions
        let expressions = find_math_expressions("5 +");
        println!("Found expressions for '5 +': {:?}", expressions);
        for expr in &expressions {
            println!(
                "Expression '{}' is valid: {}",
                expr,
                is_valid_math_expression(expr)
            );
        }
        println!("Testing 5 +: {:?}", evaluate_test_expression("5 +"));
        assert_eq!(evaluate_test_expression("5 +"), None);
        assert_eq!(evaluate_test_expression("* 5"), None);
        assert_eq!(evaluate_test_expression("((5)"), None);

        // Empty or invalid input
        assert_eq!(evaluate_test_expression(""), None);
        assert_eq!(evaluate_test_expression("hello world"), None);

        // Incompatible unit operations
        assert_eq!(evaluate_test_expression("5 GiB + 10 seconds"), None);
        assert_eq!(evaluate_test_expression("1 hour - 500 MB"), None);
    }

    #[test]
    fn test_unit_recognition() {
        // Test different unit formats
        let result = evaluate_with_unit_info("1 GiB to kib");
        assert!(result.is_some());

        let result = evaluate_with_unit_info("60 minutes to h");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001);

        let result = evaluate_with_unit_info("1024 bytes to KiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_real_world_scenarios() {
        // File transfer calculations
        assert_eq!(
            evaluate_test_expression("Download: 100 MB/s * 5 minutes"),
            Some("30,000 MB".to_string())
        );

        // Storage calculations
        assert_eq!(
            evaluate_test_expression("Total storage: 2 TB + 500 GB"),
            Some("2,500 GB".to_string())
        );

        // Bandwidth calculations
        assert_eq!(
            evaluate_test_expression("Bandwidth used: 1,000 GiB / 1 hour"),
            Some("0.278 GiB/s".to_string())
        );

        // Data conversion scenarios
        let result = evaluate_with_unit_info("How many KiB in 5 MiB?");
        assert!(result.is_some()); // Will find "5 MiB" as a valid expression

        let result = evaluate_with_unit_info("5 MiB to KiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 5120.0).abs() < 0.001);
    }

    #[test]
    fn test_precision() {
        // Test decimal precision
        assert_eq!(
            evaluate_test_expression("1.234 + 2.567"),
            Some("3.801".to_string())
        );
        assert_eq!(
            evaluate_test_expression("10.5 / 3"),
            Some("3.500".to_string())
        );

        // Test with units requiring precision
        let result = evaluate_with_unit_info("1.5 GiB to MiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1536.0).abs() < 0.001);
    }

    #[test]
    fn test_whitespace_handling() {
        // Various whitespace formats
        assert_eq!(evaluate_test_expression("5+3"), Some("8".to_string()));
        assert_eq!(
            evaluate_test_expression("  5  +  3  "),
            Some("8".to_string())
        );
        assert_eq!(evaluate_test_expression("5 * 3"), Some("15".to_string()));

        // Units with whitespace
        let result = evaluate_with_unit_info("1   GiB   to   KiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1048576.0).abs() < 0.001);
    }

    #[test]
    fn test_in_keyword_conversions() {
        // Test "in" keyword for unit conversions after calculations
        assert_eq!(
            evaluate_test_expression("24 MiB * 32 in KiB"),
            Some("786,432 KiB".to_string())
        );

        // Test with different operations
        assert_eq!(
            evaluate_test_expression("1 GiB + 512 MiB in KiB"),
            Some("1,572,864 KiB".to_string())
        );

        // Test with time calculations (using scalar multiplication)
        assert_eq!(
            evaluate_test_expression("2 hours * 60 in minutes"),
            Some("7,200 min".to_string())
        );

        // Test with complex expressions
        assert_eq!(
            evaluate_test_expression("(1 GiB + 1 GiB) / 2 in MiB"),
            Some("1,024 MiB".to_string())
        );

        // Test mixed base units (base 10 to base 2)
        assert_eq!(
            evaluate_test_expression("1000 MB * 5 in GiB"),
            Some("4.657 GiB".to_string())
        );

        // Test rate calculations with time conversion
        assert_eq!(
            evaluate_test_expression("500 GiB / 10 seconds in MiB/s"),
            Some("51,200 MiB/s".to_string())
        );

        // Test simple unit conversion
        assert_eq!(
            evaluate_test_expression("1024 KiB in MiB"),
            Some("1 MiB".to_string())
        );

        // Test addition with conversion
        assert_eq!(
            evaluate_test_expression("1 hour + 30 minutes in minutes"),
            Some("90 min".to_string())
        );

        // Test invalid unit conversion (incompatible types)
        assert_eq!(evaluate_test_expression("5 GiB + 10 in seconds"), None);

        // Test that "in" without valid target unit falls back to regular calculation
        assert_eq!(evaluate_test_expression("5 + 3 in"), Some("8".to_string()));
    }

    #[test]
    fn test_to_keyword_with_expressions() {
        // Test "to" keyword with expressions (same functionality as "in")
        assert_eq!(
            evaluate_test_expression("12 GiB + 50 MiB to MiB"),
            Some("12,338 MiB".to_string())
        );

        // Test with multiplication
        assert_eq!(
            evaluate_test_expression("24 MiB * 32 to KiB"),
            Some("786,432 KiB".to_string())
        );

        // Test with division that creates a rate
        assert_eq!(
            evaluate_test_expression("1000 GiB / 10 seconds to MiB/s"),
            Some("102,400 MiB/s".to_string())
        );

        // Test complex expression
        assert_eq!(
            evaluate_test_expression("(2 TiB - 1 GiB) / 1024 to GiB"),
            Some("1.999 GiB".to_string())
        );

        // Test time calculations
        assert_eq!(
            evaluate_test_expression("3 hours + 45 minutes to minutes"),
            Some("225 min".to_string())
        );

        // Ensure simple "to" conversions still work (backward compatibility)
        assert_eq!(
            evaluate_test_expression("1 GiB to MiB"),
            Some("1,024 MiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("60 seconds to minutes"),
            Some("1 min".to_string())
        );
    }

    #[test]
    fn test_line_references() {
        // Test parsing line references
        assert_eq!(parse_line_reference("line1"), Some(0));
        assert_eq!(parse_line_reference("line5"), Some(4));
        assert_eq!(parse_line_reference("line123"), Some(122));
        assert_eq!(parse_line_reference("Line1"), Some(0)); // Case insensitive
        assert_eq!(parse_line_reference("LINE1"), Some(0)); // Case insensitive

        // Test invalid line references
        assert_eq!(parse_line_reference("line0"), None); // Line numbers start at 1
        assert_eq!(parse_line_reference("line"), None); // No number
        assert_eq!(parse_line_reference("lineabc"), None); // Invalid number
        assert_eq!(parse_line_reference("myline1"), None); // Doesn't start with "line"
        assert_eq!(parse_line_reference("1line"), None); // Doesn't start with "line"

        // Test line reference resolution with context
        let previous_results = vec![
            Some("10 GiB".to_string()),
            Some("5".to_string()),
            None,
            Some("1,024 MiB".to_string()),
        ];

        // Test valid line references
        assert_eq!(
            evaluate_expression_with_context("line1 + 4 GiB", &previous_results, 4),
            Some("14 GiB".to_string())
        );
        assert_eq!(
            evaluate_expression_with_context("line2 * 3", &previous_results, 4),
            Some("15".to_string())
        );
        assert_eq!(
            evaluate_expression_with_context("line4 to GiB", &previous_results, 4),
            Some("1 GiB".to_string())
        );

        // Test circular reference prevention
        assert_eq!(
            evaluate_expression_with_context("line1 + 2", &previous_results, 0),
            None
        ); // Can't reference self
        assert_eq!(
            evaluate_expression_with_context("line5 + 2", &previous_results, 4),
            None
        ); // Can't reference future lines

        // Test reference to line with no result
        assert_eq!(
            evaluate_expression_with_context("line3 + 5", &previous_results, 4),
            None
        ); // Line 3 has no result

        // Test complex expressions with line references
        assert_eq!(
            evaluate_expression_with_context("(line1 + line4) / 2", &previous_results, 4),
            Some("5,632 MiB".to_string())
        );
        assert_eq!(
            evaluate_expression_with_context("line1 * line2 to MiB", &previous_results, 4),
            Some("51,200 MiB".to_string())
        );
    }

    #[test]
    fn test_line_reference_parsing_edge_cases() {
        // Test result string parsing
        assert!(parse_result_string("10 GiB").is_some());
        assert!(parse_result_string("1,024").is_some());
        assert!(parse_result_string("42").is_some());
        assert!(parse_result_string("3.14 MiB/s").is_some());

        // Test invalid result strings
        assert!(parse_result_string("").is_none());
        assert!(parse_result_string("invalid").is_none());
        assert!(parse_result_string("GiB 10").is_none()); // Wrong order

        // Test line reference in tokenizer
        let tokens = tokenize_with_units("line1 + 5 GiB").unwrap();
        assert!(matches!(tokens[0], Token::LineReference(0)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::NumberWithUnit(5.0, Unit::GiB)));
    }

    #[test]
    fn test_qps_unit_parsing() {
        // Test QPS unit parsing
        assert_eq!(parse_unit("qps"), Some(Unit::QueriesPerSecond));
        assert_eq!(parse_unit("QPS"), Some(Unit::QueriesPerSecond));
        assert_eq!(parse_unit("queries/s"), Some(Unit::QueriesPerSecond));
        assert_eq!(parse_unit("queries/sec"), Some(Unit::QueriesPerSecond));
        assert_eq!(parse_unit("qpm"), Some(Unit::QueriesPerMinute));
        assert_eq!(parse_unit("queries/min"), Some(Unit::QueriesPerMinute));
        assert_eq!(parse_unit("queries/minute"), Some(Unit::QueriesPerMinute));
        assert_eq!(parse_unit("qph"), Some(Unit::QueriesPerHour));
        assert_eq!(parse_unit("queries/h"), Some(Unit::QueriesPerHour));
        assert_eq!(parse_unit("queries/hour"), Some(Unit::QueriesPerHour));

        // Test request rate unit parsing
        assert_eq!(parse_unit("req/s"), Some(Unit::RequestsPerSecond));
        assert_eq!(parse_unit("requests/s"), Some(Unit::RequestsPerSecond));
        assert_eq!(parse_unit("rps"), Some(Unit::RequestsPerSecond));
        assert_eq!(parse_unit("req/min"), Some(Unit::RequestsPerMinute));
        assert_eq!(parse_unit("requests/min"), Some(Unit::RequestsPerMinute));
        assert_eq!(parse_unit("rpm"), Some(Unit::RequestsPerMinute));
        assert_eq!(parse_unit("req/h"), Some(Unit::RequestsPerHour));
        assert_eq!(parse_unit("req/hour"), Some(Unit::RequestsPerHour));
        assert_eq!(parse_unit("requests/h"), Some(Unit::RequestsPerHour));
        assert_eq!(parse_unit("requests/hour"), Some(Unit::RequestsPerHour));
        assert_eq!(parse_unit("rph"), Some(Unit::RequestsPerHour));

        // Test request/query count unit parsing
        assert_eq!(parse_unit("req"), Some(Unit::Request));
        assert_eq!(parse_unit("request"), Some(Unit::Request));
        assert_eq!(parse_unit("requests"), Some(Unit::Request));
        assert_eq!(parse_unit("query"), Some(Unit::Query));
        assert_eq!(parse_unit("queries"), Some(Unit::Query));
    }

    #[test]
    fn test_qps_unit_conversions() {
        // Test QPS to other rate units
        let result = evaluate_with_unit_info("100 QPS to req/min");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 6000.0).abs() < 0.001); // 100 * 60 = 6000 req/min

        let result = evaluate_with_unit_info("1 QPS to QPH");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 3600.0).abs() < 0.001); // 1 * 3600 = 3600 QPH

        let result = evaluate_with_unit_info("7200 req/h to req/min");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 120.0).abs() < 0.001); // 7200 / 60 = 120 req/min

        let result = evaluate_with_unit_info("60 QPM to QPS");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001); // 60 / 60 = 1 QPS

        // Test cross-family conversions (QPS to RPS should work since they're equivalent)
        let result = evaluate_with_unit_info("100 QPS to req/s");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 100.0).abs() < 0.001); // Direct equivalence

        let result = evaluate_with_unit_info("150 req/min to QPM");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 150.0).abs() < 0.001); // Direct equivalence
    }

    #[test]
    fn test_qps_arithmetic_operations() {
        // Test QPS * time = total requests
        assert_eq!(
            evaluate_test_expression("25 QPS * 1 hour"),
            Some("90,000 query".to_string())
        );

        assert_eq!(
            evaluate_test_expression("100 QPS * 30 s"),
            Some("3,000 query".to_string())
        );

        assert_eq!(
            evaluate_test_expression("50 req/s * 2 minutes"),
            Some("6,000 req".to_string())
        );

        assert_eq!(
            evaluate_test_expression("1 hour * 10 req/min"),
            Some("36,000 req".to_string())
        );

        // Test requests / time = request rate
        assert_eq!(
            evaluate_test_expression("3600 queries / 1 hour"),
            Some("1 QPS".to_string())
        );

        assert_eq!(
            evaluate_test_expression("6000 req / 10 minutes"),
            Some("10 req/s".to_string())
        );

        assert_eq!(
            evaluate_test_expression("1200 requests / 20 s"),
            Some("60 req/s".to_string())
        );

        // Test QPS arithmetic with conversions
        assert_eq!(
            evaluate_test_expression("100 QPS * 30 minutes to req"),
            Some("180,000 req".to_string())
        );

        assert_eq!(
            evaluate_test_expression("5000 queries / 10 minutes to QPS"),
            Some("8.333 QPS".to_string())
        );

        // Test complex expressions
        assert_eq!(
            evaluate_test_expression("(100 QPS + 50 QPS) * 2 hours"),
            Some("1,080,000 query".to_string())
        );

        assert_eq!(
            evaluate_test_expression("10000 req / (5 minutes + 5 minutes)"),
            Some("16.667 req/s".to_string())
        );
    }

    #[test]
    fn test_qps_addition_subtraction() {
        // Test adding/subtracting same rate units
        assert_eq!(
            evaluate_test_expression("100 QPS + 50 QPS"),
            Some("150 QPS".to_string())
        );

        assert_eq!(
            evaluate_test_expression("200 req/min - 80 req/min"),
            Some("120 req/min".to_string())
        );

        // Test adding different rate units (should convert to common base)
        assert_eq!(
            evaluate_test_expression("100 QPS + 60 QPM"),
            Some("6,060 QPM".to_string())
        );

        assert_eq!(
            evaluate_test_expression("3600 QPH - 30 QPM"),
            Some("1,800 QPH".to_string())
        );

        // Test mixed request rate families
        assert_eq!(
            evaluate_test_expression("100 QPS + 100 req/s"),
            Some("200 req/s".to_string())
        );

        assert_eq!(
            evaluate_test_expression("1000 req/min + 500 QPM"),
            Some("1,500 QPM".to_string())
        );
    }

    #[test]
    fn test_qps_real_world_scenarios() {
        // Test realistic QPS scenarios
        assert_eq!(
            evaluate_test_expression("API load: 1000 QPS * 5 minutes"),
            Some("300,000 query".to_string())
        );

        assert_eq!(
            evaluate_test_expression("Peak traffic: 500 req/s * 1 hour"),
            Some("1,800,000 req".to_string())
        );

        assert_eq!(
            evaluate_test_expression("Daily requests: 86400 req / 1 day"),
            Some("1 req/s".to_string())
        );

        // Test load balancing scenarios
        assert_eq!(
            evaluate_test_expression("Total load: 250 QPS + 150 QPS + 100 QPS"),
            Some("500 QPS".to_string())
        );

        assert_eq!(
            evaluate_test_expression("Per server: 1500 QPS / 3"),
            Some("500 QPS".to_string())
        );

        // Test capacity planning
        assert_eq!(
            evaluate_test_expression("Monthly queries: 100 QPS * 30 days"),
            Some("259,200,000 query".to_string())
        );

        // Test rate conversions for monitoring
        assert_eq!(
            evaluate_test_expression("Monitor rate: 5000 req/min to req/s"),
            Some("83.333 req/s".to_string())
        );
    }

    #[test]
    fn test_qps_edge_cases() {
        // Test very small QPS rates
        assert_eq!(
            evaluate_test_expression("0.1 QPS * 10 s"),
            Some("1 query".to_string())
        );

        assert_eq!(
            evaluate_test_expression("1 query / 10 s"),
            Some("0.100 QPS".to_string())
        );

        // Test very large QPS rates
        assert_eq!(
            evaluate_test_expression("1000000 QPS * 1 s"),
            Some("1,000,000 query".to_string())
        );

        // Test fractional results
        assert_eq!(
            evaluate_test_expression("100 QPS / 3"),
            Some("33.333 QPS".to_string())
        );

        assert_eq!(
            evaluate_test_expression("1000 req / 7 minutes"),
            Some("2.381 req/s".to_string())
        );

        // Test zero and negative cases (should be valid mathematically)
        assert_eq!(
            evaluate_test_expression("0 QPS * 1 hour"),
            Some("0 query".to_string())
        );

        // Test incompatible unit operations (should fail)
        assert_eq!(evaluate_test_expression("100 QPS + 1 GiB"), None);
        assert_eq!(evaluate_test_expression("50 req/s - 10 seconds"), None);
        assert_eq!(evaluate_test_expression("1000 queries + 5 hours"), None);
    }

    #[test]
    fn test_qps_unit_display_names() {
        // Test that display names are correct for QPS units
        assert_eq!(Unit::QueriesPerSecond.display_name(), "QPS");
        assert_eq!(Unit::QueriesPerMinute.display_name(), "QPM");
        assert_eq!(Unit::QueriesPerHour.display_name(), "QPH");
        assert_eq!(Unit::RequestsPerSecond.display_name(), "req/s");
        assert_eq!(Unit::RequestsPerMinute.display_name(), "req/min");
        assert_eq!(Unit::RequestsPerHour.display_name(), "req/h");
        assert_eq!(Unit::Request.display_name(), "req");
        assert_eq!(Unit::Query.display_name(), "query");
    }

    #[test]
    fn test_qps_unit_type_classification() {
        // Test that QPS units are properly classified
        assert_eq!(Unit::QueriesPerSecond.unit_type(), UnitType::RequestRate);
        assert_eq!(Unit::QueriesPerMinute.unit_type(), UnitType::RequestRate);
        assert_eq!(Unit::QueriesPerHour.unit_type(), UnitType::RequestRate);
        assert_eq!(Unit::RequestsPerSecond.unit_type(), UnitType::RequestRate);
        assert_eq!(Unit::RequestsPerMinute.unit_type(), UnitType::RequestRate);
        assert_eq!(Unit::RequestsPerHour.unit_type(), UnitType::RequestRate);
        assert_eq!(Unit::Request.unit_type(), UnitType::Request);
        assert_eq!(Unit::Query.unit_type(), UnitType::Request);
    }

    #[test]
    fn test_large_data_unit_parsing() {
        // Test Petabyte unit parsing (base 10)
        assert_eq!(parse_unit("pb"), Some(Unit::PB));
        assert_eq!(parse_unit("PB"), Some(Unit::PB));

        // Test Exabyte unit parsing (base 10)
        assert_eq!(parse_unit("eb"), Some(Unit::EB));
        assert_eq!(parse_unit("EB"), Some(Unit::EB));

        // Test Pebibyte unit parsing (base 2)
        assert_eq!(parse_unit("pib"), Some(Unit::PiB));
        assert_eq!(parse_unit("PiB"), Some(Unit::PiB));

        // Test Exbibyte unit parsing (base 2)
        assert_eq!(parse_unit("eib"), Some(Unit::EiB));
        assert_eq!(parse_unit("EiB"), Some(Unit::EiB));

        // Test rate units
        assert_eq!(parse_unit("pb/s"), Some(Unit::PBPerSecond));
        assert_eq!(parse_unit("pbps"), Some(Unit::PBPerSecond));
        assert_eq!(parse_unit("eb/s"), Some(Unit::EBPerSecond));
        assert_eq!(parse_unit("ebps"), Some(Unit::EBPerSecond));
        assert_eq!(parse_unit("pib/s"), Some(Unit::PiBPerSecond));
        assert_eq!(parse_unit("pibps"), Some(Unit::PiBPerSecond));
        assert_eq!(parse_unit("eib/s"), Some(Unit::EiBPerSecond));
        assert_eq!(parse_unit("eibps"), Some(Unit::EiBPerSecond));
    }

    #[test]
    fn test_large_data_unit_conversions() {
        // Test TB to PB conversions (base 10)
        let result = evaluate_with_unit_info("1000 TB to PB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001); // 1000 TB = 1 PB

        let result = evaluate_with_unit_info("5 PB to TB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 5000.0).abs() < 0.001); // 5 PB = 5000 TB

        // Test PB to EB conversions (base 10)
        let result = evaluate_with_unit_info("1000 PB to EB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001); // 1000 PB = 1 EB

        let result = evaluate_with_unit_info("2 EB to PB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 2000.0).abs() < 0.001); // 2 EB = 2000 PB

        // Test TiB to PiB conversions (base 2)
        let result = evaluate_with_unit_info("1024 TiB to PiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001); // 1024 TiB = 1 PiB

        let result = evaluate_with_unit_info("3 PiB to TiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 3072.0).abs() < 0.001); // 3 PiB = 3072 TiB

        // Test PiB to EiB conversions (base 2)
        let result = evaluate_with_unit_info("1024 PiB to EiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 1.0).abs() < 0.001); // 1024 PiB = 1 EiB

        let result = evaluate_with_unit_info("2 EiB to PiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 2048.0).abs() < 0.001); // 2 EiB = 2048 PiB

        // Test mixed base conversions
        let result = evaluate_with_unit_info("1 PB to PiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        // 1 PB = 1,000,000,000,000,000 bytes = ~0.888 PiB
        assert!((unit_val.value - 0.8881784197).abs() < 0.0001);

        let result = evaluate_with_unit_info("1 EiB to EB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        // 1 EiB = 1,152,921,504,606,846,976 bytes = ~1.153 EB
        assert!((unit_val.value - 1.152921504606847).abs() < 0.0001);
    }

    #[test]
    fn test_large_data_unit_arithmetic() {
        // Test arithmetic with PB units
        assert_eq!(
            evaluate_test_expression("2 PB + 500 TB"),
            Some("2,500 TB".to_string())
        );

        assert_eq!(
            evaluate_test_expression("5 PB - 1000 TB"),
            Some("4,000 TB".to_string())
        );

        // Test arithmetic with EB units
        assert_eq!(
            evaluate_test_expression("1 EB + 200 PB"),
            Some("1,200 PB".to_string())
        );

        // Test arithmetic with PiB units
        assert_eq!(
            evaluate_test_expression("1 PiB + 512 TiB"),
            Some("1,536 TiB".to_string())
        );

        assert_eq!(
            evaluate_test_expression("2 EiB - 1024 PiB"),
            Some("1,024 PiB".to_string())
        );

        // Test rate calculations with large units
        assert_eq!(
            evaluate_test_expression("1 PB / 1 hour"),
            Some("0.000 PB/s".to_string())
        );

        assert_eq!(
            evaluate_test_expression("10 PB/s * 30 minutes"),
            Some("18,000 PB".to_string())
        );

        // Test very large transfers
        assert_eq!(
            evaluate_test_expression("1 EB/s * 1 second"),
            Some("1 EB".to_string())
        );

        assert_eq!(
            evaluate_test_expression("500 EiB / 1 day"),
            Some("0.006 EiB/s".to_string())
        );
    }

    #[test]
    fn test_large_data_unit_display_names() {
        // Test display names for large data units
        assert_eq!(Unit::PB.display_name(), "PB");
        assert_eq!(Unit::EB.display_name(), "EB");
        assert_eq!(Unit::PiB.display_name(), "PiB");
        assert_eq!(Unit::EiB.display_name(), "EiB");

        // Test display names for large rate units
        assert_eq!(Unit::PBPerSecond.display_name(), "PB/s");
        assert_eq!(Unit::EBPerSecond.display_name(), "EB/s");
        assert_eq!(Unit::PiBPerSecond.display_name(), "PiB/s");
        assert_eq!(Unit::EiBPerSecond.display_name(), "EiB/s");
    }

    #[test]
    fn test_large_data_unit_type_classification() {
        // Test that large data units are properly classified
        assert_eq!(Unit::PB.unit_type(), UnitType::Data);
        assert_eq!(Unit::EB.unit_type(), UnitType::Data);
        assert_eq!(Unit::PiB.unit_type(), UnitType::Data);
        assert_eq!(Unit::EiB.unit_type(), UnitType::Data);

        // Test that large rate units are properly classified
        assert_eq!(Unit::PBPerSecond.unit_type(), UnitType::DataRate);
        assert_eq!(Unit::EBPerSecond.unit_type(), UnitType::DataRate);
        assert_eq!(Unit::PiBPerSecond.unit_type(), UnitType::DataRate);
        assert_eq!(Unit::EiBPerSecond.unit_type(), UnitType::DataRate);
    }

    #[test]
    fn test_large_data_real_world_scenarios() {
        // Test data center storage scenarios
        assert_eq!(
            evaluate_test_expression("Data center: 50 PB + 10 EB"),
            Some("10,050 PB".to_string())
        );

        // Test backup scenarios
        assert_eq!(
            evaluate_test_expression("Backup rate: 100 TB/s * 8 hours"),
            Some("2,880,000 TB".to_string())
        );

        // Test very large data transfers
        assert_eq!(
            evaluate_test_expression("Transfer: 5 EiB to PB"),
            Some("5,764.608 PB".to_string())
        );

        // Test scientific computing scenarios
        assert_eq!(
            evaluate_test_expression("Dataset: 1.5 EB to TiB"),
            Some("1,364,242.053 TiB".to_string())
        );

        // Test network throughput
        assert_eq!(
            evaluate_test_expression("Network: 10 PB/s to TB/s"),
            Some("10,000 TB/s".to_string())
        );

        // Test storage capacity planning
        assert_eq!(
            evaluate_test_expression("Total capacity: 100 PiB + 50 EiB"),
            Some("51,300 PiB".to_string())
        );
    }

    #[test]
    fn test_large_data_edge_cases() {
        // Test very small fractions
        assert_eq!(
            evaluate_test_expression("0.001 PB to TB"),
            Some("1 TB".to_string())
        );

        assert_eq!(
            evaluate_test_expression("0.5 EiB to PiB"),
            Some("512 PiB".to_string())
        );

        // Test precision with large numbers
        assert_eq!(
            evaluate_test_expression("1024.5 PiB to EiB"),
            Some("1.000 EiB".to_string())
        );

        // Test cross-base conversions with precision
        let result = evaluate_with_unit_info("1.234567 EB to EiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        // Should be approximately 1.071 EiB
        assert!((unit_val.value - 1.071).abs() < 0.01);

        // Test incompatible operations (should fail)
        assert_eq!(evaluate_test_expression("1 PB + 5 hours"), None);
        assert_eq!(evaluate_test_expression("100 EiB - 50 QPS"), None);
        assert_eq!(evaluate_test_expression("1 EB * 1 query"), None);
    }
}
