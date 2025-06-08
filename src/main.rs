//! Binary entry point for mathypad

use clap::{Arg, Command};
use mathypad::{run_interactive_mode, run_one_shot_mode};
use std::error::Error;

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