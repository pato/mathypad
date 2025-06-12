//! Binary entry point for mathypad

use clap::{Arg, Command, crate_version};
use mathypad::{run_interactive_mode_with_file, run_one_shot_mode};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check for "--" separator first (one-shot mode)
    if let Some(dash_pos) = args.iter().position(|arg| arg == "--") {
        let expression_parts: Vec<String> = args.iter().skip(dash_pos + 1).cloned().collect();
        if !expression_parts.is_empty() {
            let expression = expression_parts.join(" ");
            run_one_shot_mode(&expression)?;
            return Ok(());
        }
    }

    // Use clap for file argument and help/version
    let matches = Command::new("mathypad")
        .version(crate_version!())
        .about("A mathematical notepad with unit conversion support")
        .arg(
            Arg::new("file")
                .help("File to open")
                .value_name("FILE")
                .index(1)
                .required(false),
        )
        .after_help("Examples:\n  mathypad                      # Start empty interactive mode\n  mathypad calculations.txt     # Open file in interactive mode\n  mathypad -- \"100 GB to GiB\"   # One-shot calculation")
        .get_matches();

    // Check if we have a file to open
    let file_path = matches.get_one::<String>("file").map(PathBuf::from);

    // Run the interactive TUI mode with optional file
    run_interactive_mode_with_file(file_path)
}
