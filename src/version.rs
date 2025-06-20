//! Version tracking functionality for mathypad

use std::fs;

const VERSION_FILE: &str = "version";

/// Initialize version tracking by creating ~/.mathypad directory and writing current version
pub fn init_version_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let mathypad_dir = get_mathypad_dir()?;

    // Create the ~/.mathypad directory if it doesn't exist
    if !mathypad_dir.exists() {
        fs::create_dir_all(&mathypad_dir)?;
    }

    // Write the current version to the version file
    let version_file = mathypad_dir.join(VERSION_FILE);
    let current_version = env!("CARGO_PKG_VERSION");
    fs::write(&version_file, current_version)?;

    Ok(())
}

/// Get the stored version from ~/.mathypad/version
pub fn get_stored_version() -> Option<String> {
    let mathypad_dir = get_mathypad_dir().ok()?;
    let version_file = mathypad_dir.join(VERSION_FILE);

    if version_file.exists() {
        fs::read_to_string(&version_file)
            .ok()?
            .trim()
            .to_string()
            .into()
    } else {
        None
    }
}

/// Get the current version from Cargo.toml
pub fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Check if this is a first run (no version file exists)
pub fn is_first_run() -> bool {
    get_stored_version().is_none()
}

/// Check if this is a newer version than what's stored
pub fn is_newer_version() -> bool {
    match get_stored_version() {
        Some(stored) => {
            let current = get_current_version();
            version_compare(current, &stored) > 0
        }
        None => true, // First run counts as newer version
    }
}

/// Get the ~/.mathypad directory path
fn get_mathypad_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home_dir.join(".mathypad"))
}

/// Simple version comparison (assumes semantic versioning x.y.z)
/// Returns: -1 if a < b, 0 if a == b, 1 if a > b
fn version_compare(a: &str, b: &str) -> i32 {
    let parse_version =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|part| part.parse().ok()).collect() };

    let version_a = parse_version(a);
    let version_b = parse_version(b);

    let max_len = version_a.len().max(version_b.len());

    for i in 0..max_len {
        let a_part = version_a.get(i).copied().unwrap_or(0);
        let b_part = version_b.get(i).copied().unwrap_or(0);

        match a_part.cmp(&b_part) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => continue,
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert_eq!(version_compare("1.0.0", "1.0.0"), 0);
        assert_eq!(version_compare("1.0.1", "1.0.0"), 1);
        assert_eq!(version_compare("1.0.0", "1.0.1"), -1);
        assert_eq!(version_compare("1.1.0", "1.0.9"), 1);
        assert_eq!(version_compare("2.0.0", "1.9.9"), 1);
        assert_eq!(version_compare("1.0", "1.0.0"), 0);
    }
}
