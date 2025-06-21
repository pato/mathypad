# Important commands

- `cargo check` - to compile and check the code
- `cargo test` - run all the tests
- `cargo test SPECIFIC_TEST -- --nocapture` - run one specific test (replace
  SPECIFIC_TEST with the desired test)
- `cargo fmt` - format all the code
- `cargo clippy --all-targets --all-features -- -D warnings` - check the code for lint errors
- `cargo clippy --fix --allow-dirty` - use clippy to try and automatically fix
  lint errors

# Running one shot mode

If you want to test the one shot mode you should use
commands like: `cargo run -- -- "100 GiB / 10 minutes"` (it's important to have the double escape "-- --").

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

## Important notes

- Mathypad runs in raw mode, so if we ever need to exit it cannot be through
  'std::process::exit(0)' without first reseting raw mode, otherwise it will
  mess up the terminal
- Because Mathypad runs in a TUI, we cannot print text in the UI mode otherwise
  it will not render properly and mess up the TUI
- You cannot run mathypad in TUI mode directly (i.e., 'cargo run' will not work
  unless you are using one-shot mode). So do not try to run it in TUI mode.
  Instead you can rely on the UI snapshot tests

## Testing

Since the main application is a TUI that requires interactive tty, you will not
be able to run it directly. Instead you should rely on the various testing
suites we have for simulating interactions, and multi-line notebooks. We prefer
to havefast unit tests that cover all the various typse of operations, edge
cases, conversions, and behaviors of mathypad. It is important that all tests
cases pass, it is not acceptable to have even one failure. If you are really
stuck trying to get tests to pass, you should ask for human input.

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

# UI Snapshot Testing

Mathypad uses snapshot testing with `cargo-insta` to ensure UI consistency and catch visual regressions.

## Overview

UI snapshot tests capture the exact visual output of the terminal UI and compare it against stored "golden" snapshots. Tests are in `src/ui/tests.rs` and use a fixed 120x30 terminal size for consistency.

## Running Tests

```bash
# Run all UI snapshot tests
cargo test ui::

# Run specific snapshot test
cargo test ui::tests::test_basic_ui_layout
```

## Managing Snapshot Changes

When you make UI changes, snapshot tests will fail. This is expected.

### Review Changes (RECOMMENDED)
```bash
cargo insta review
```
Interactive mode: `a` (accept), `r` (reject), `s` (skip)

### Accept All Changes (Use with Caution)
```bash
cargo insta accept
```

### Check What Needs Review
```bash
cargo insta pending-snapshots
```

## Essential Workflow

1. Make UI changes
2. Run `cargo test ui::` 
3. Use `cargo insta review` to examine diffs
4. Accept expected changes, reject unexpected ones
5. Commit both code and updated snapshots

## Writing New Snapshot Tests

- Use `create_test_terminal()` helper for consistent 120x30 size
- Use `assert_snapshot!("test_name", output)` 
- Include realistic test data in your App instance
- Test edge cases (empty states, dialogs, different modes)

## Key Points

- **Always review diffs** - don't blindly accept with `cargo insta accept`
- **Commit `.snap` files** - ignore `.snap.new` and `.snap.old` files  
- **Snapshot failures catch regressions** - investigate unexpected changes
- **Use descriptive test names** - they become snapshot file names
- Tests cover: layout, syntax highlighting, dialogs, cursor states, scrolling

## Common Issues

- **New test failing**: Run once to generate snapshot, then `cargo insta accept`
- **Size mismatch**: Ensure all tests use `create_test_terminal()` helper
- **CI failures**: Commit snapshot files and ensure no platform differences
