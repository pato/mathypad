//! Binary entry point for mathypad

use clap::{Arg, Command, ValueHint, crate_version};
use mathypad::{run_interactive_mode_with_file, run_one_shot_mode, version};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // Check for one-shot mode first (before clap parsing to preserve existing behavior)
    if let Some(expression) = extract_one_shot_expression() {
        return run_one_shot_mode(&expression);
    }

    let matches = build_cli().get_matches();

    // Handle completions flag
    if let Some(shell) = matches.get_one::<String>("completions") {
        print_completion_script(shell);
        return Ok(());
    }

    // Handle changelog flags
    if matches.get_flag("changelog") || matches.get_flag("whats-new") {
        print_changelog();
        return Ok(());
    }

    // Initialize version tracking (create ~/.mathypad and write current version)
    if let Err(e) = version::init_version_tracking() {
        eprintln!("Warning: Could not initialize version tracking: {}", e);
    }

    // Extract file path and run interactive mode
    let file_path = matches.get_one::<String>("file").map(PathBuf::from);
    run_interactive_mode_with_file(file_path)
}

/// Extract one-shot expression if "--" separator is present
fn extract_one_shot_expression() -> Option<String> {
    let mut args = std::env::args();

    // Find the "--" separator position
    let dash_pos = args.position(|arg| arg == "--")?;

    // Skip the program name and arguments before "--"
    let remaining_args: Vec<String> = std::env::args().skip(dash_pos + 1).collect();

    if remaining_args.is_empty() {
        None
    } else {
        Some(remaining_args.join(" "))
    }
}

/// Build the CLI command structure
fn build_cli() -> Command {
    Command::new("mathypad")
        .version(crate_version!())
        .about("A mathematical notepad with unit conversion support")
        .arg(
            Arg::new("help_alt")
                .short('?')
                .action(clap::ArgAction::Help)
                .help("Print help"),
        )
        .arg(
            Arg::new("completions")
                .long("completions")
                .help("Generate shell completion files")
                .value_name("SHELL")
                .value_parser(["bash", "zsh", "fish"]),
        )
        .arg(
            Arg::new("changelog")
                .long("changelog")
                .action(clap::ArgAction::SetTrue)
                .help("Show the changelog"),
        )
        .arg(
            Arg::new("whats-new")
                .long("whats-new")
                .action(clap::ArgAction::SetTrue)
                .help("Show what's new (alias for --changelog)"),
        )
        .arg(
            Arg::new("file")
                .help("File to open")
                .value_name("FILE")
                .index(1)
                .value_hint(ValueHint::FilePath),
        )
        .after_help(
            "Examples:\n\
             \x20 mathypad                      # Start empty interactive mode\n\
             \x20 mathypad calculations.pad     # Open file in interactive mode\n\
             \x20 mathypad -- \"100 GB to GiB\"   # One-shot calculation\n\
             \x20 eval \"$(mathypad --completions bash)\"  # Enable bash completions",
        )
}

/// Print the completion script for the specified shell
/// Note: shell parameter is guaranteed to be valid by clap's value_parser
fn print_completion_script(shell: &str) {
    let completion_script = match shell {
        "bash" => include_str!("../../completions/mathypad.bash"),
        "zsh" => include_str!("../../completions/mathypad.zsh"),
        "fish" => include_str!("../../completions/mathypad.fish"),
        _ => unreachable!("clap should prevent invalid shell values"),
    };

    print!("{}", completion_script);
}

/// Print the changelog/what's new information
fn print_changelog() {
    // Include the changelog directly from the repository
    let changelog = include_str!("../../CHANGELOG.md");
    println!("{}", changelog);
}
