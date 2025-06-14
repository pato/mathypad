# Important commands

- `cargo check` - to compile and check the code
- `cargo test` - run all the tests
- `cargo test SPECIFIC_TEST -- --nocapture` - run one specific test (replace
  SPECIFIC_TEST with the desired test)
- `cargo fmt` - format all the code
- `cargo clippy` - check the code for lint errors
- `cargo clippy --fix --allow-dirty` - use clippy to try and automatically fix
  lint errors

# General workflow

- Make the changes requested, making sure to always add unit tests for new
  functionality
- When applicable, also run the tool in one-shot mode to make sure it works
- Always make sure there are no regressions in other tests at the end
- Once you are done, make sure to run `cargo fmt` to format all the code properly
- We want to be very careful whenever we have to change existing tests to make
  sure we are no introducing any regressions in behavior, if you are in doubt
  ask the human for verification if you are going to have to change an existing
  test in a non-trivial way

## Testing

Since the main application is a TUI that requires interactive tty, you will not
be able to run it directly. Instead you should rely on the various testing
suites we have for simulating interactions, and multi-line notebooks. We prefer
to havefast unit tests that cover all the various typse of operations, edge
cases, conversions, and behaviors of mathypad.

## Code Quality Standards
- Compiler warnings should be treated as errors and fixed, not ignored
- Linter warnings should be treated as errors and fixed, not ignored
- Code should be thoroughly tested with appropriate unit tests
- Follow standard Rust idioms including error handling

# Project Structure

## Directory Layout

```
mathypad2/
├── src/                    # Main source code
│   ├── bin/               # Binary entry points
│   │   └── main.rs       # CLI entry point (handles both TUI and one-shot modes)
│   ├── expression/        # Expression parsing and evaluation
│   │   ├── chumsky_parser.rs  # Parser implementation using Chumsky
│   │   ├── evaluator.rs       # Expression evaluation logic
│   │   ├── parser.rs          # Expression parsing utilities
│   │   ├── tests.rs           # Expression module tests
│   │   └── tokens.rs          # Token definitions for parsing
│   ├── ui/               # Terminal UI components
│   │   ├── events.rs     # Event handling and interactive mode
│   │   └── render.rs     # UI rendering using Ratatui
│   ├── units/            # Unit system and conversions
│   │   ├── parser.rs     # Unit parsing functionality
│   │   ├── tests.rs      # Unit system tests
│   │   ├── types.rs      # Unit type definitions and conversions
│   │   └── value.rs      # Unit value representation
│   ├── app.rs            # Core application state and logic
│   ├── cli.rs            # CLI functionality for one-shot mode
│   ├── integration_tests.rs  # End-to-end integration tests
│   ├── lib.rs            # Library root, exports public APIs
│   └── mode.rs           # Vim-like editing modes (Insert/Normal)
├── screenshots/           # Application screenshots
├── web/                  # Web-related files
├── Cargo.toml            # Project configuration
├── CLAUDE.md             # This file - AI assistant instructions
├── README.md             # Project documentation
├── LICENSE               # MIT license
├── CHANGELOG.md          # Version history
├── cliff.toml            # Changelog generation config
└── todo.md               # Project tasks
```

## Key Components

### Core Application (`app.rs`)
- Manages the application state including text lines, cursor position, and scroll offset
- Handles variable storage and retrieval
- Implements mode switching between Insert and Normal modes
- Contains the main application logic for text editing operations

### Expression Module (`expression/`)
- **`tokens.rs`**: Defines the token types used in parsing (numbers, operators, functions, etc.)
- **`parser.rs`**: Expression parsing utilities and helper functions
- **`chumsky_parser.rs`**: Main parser implementation using the Chumsky parser combinator library
- **`evaluator.rs`**: Evaluates parsed expressions and performs calculations
- **`tests.rs`**: Comprehensive unit tests for expression parsing and evaluation

### Units Module (`units/`)
- **`types.rs`**: Defines unit types, dimensions, and conversion factors
- **`value.rs`**: Represents values with units and implements unit operations
- **`parser.rs`**: Parses unit expressions and handles unit conversions
- **`tests.rs`**: Tests for unit parsing, conversion, and arithmetic

### UI Module (`ui/`)
- **`events.rs`**: Handles keyboard events and manages the interactive TUI mode
- **`render.rs`**: Renders the terminal UI using Ratatui, displays lines and results

### CLI Module (`cli.rs`)
- Implements one-shot mode for evaluating expressions from command line
- Handles non-interactive expression evaluation

### Mode Module (`mode.rs`)
- Defines Vim-like editing modes (Insert and Normal)
- Simple enum for mode state management

## Test Organization

1. **Unit Tests**: Co-located with code using `#[cfg(test)]` blocks
   - Each module contains its own unit tests
   - Focus on testing individual components in isolation

2. **Integration Tests**: Located in `src/integration_tests.rs`
   - Tests end-to-end functionality
   - Verifies correct interaction between parser, evaluator, and UI

3. **Test Helpers**: Defined in `lib.rs` under `test_helpers` module
   - `evaluate_test_expression()`: Helper for testing expression evaluation
   - `evaluate_with_unit_info()`: Helper for testing unit conversions

## Key Features

- **TUI Calculator**: Interactive terminal-based calculator with live evaluation
- **Unit Conversions**: Comprehensive unit system with automatic conversions
- **Expression Parser**: Robust mathematical expression parser using Chumsky
- **Vim-like Editing**: Modal editing with Insert and Normal modes
- **Variable Support**: Store and reference variables in calculations
- **Line References**: Reference values from other lines with automatic updates when referenced lines change
- **One-shot Mode**: CLI mode for quick calculations without TUI
