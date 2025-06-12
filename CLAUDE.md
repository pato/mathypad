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
