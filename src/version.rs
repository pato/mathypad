//! Version tracking functionality for mathypad

use std::fs;

const VERSION_FILE: &str = "version";

/// Initialize version tracking by creating ~/.mathypad directory (but don't update version yet)
pub fn init_version_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let mathypad_dir = get_mathypad_dir()?;

    // Create the ~/.mathypad directory if it doesn't exist
    if !mathypad_dir.exists() {
        fs::create_dir_all(&mathypad_dir)?;
    }

    Ok(())
}

/// Update the stored version to the current version (call after welcome screen is dismissed)
pub fn update_stored_version() -> Result<(), Box<dyn std::error::Error>> {
    let mathypad_dir = get_mathypad_dir()?;
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

/// Get changelog content between the stored version and current version
/// If no stored version exists (first run), show the latest version's changelog
pub fn get_changelog_since_version() -> Option<String> {
    let current_version = get_current_version();

    match get_stored_version() {
        Some(stored_version) => {
            // If versions are the same, no changelog to show
            if version_compare(current_version, &stored_version) <= 0 {
                return None;
            }
            extract_changelog_between_versions(&stored_version, current_version)
        }
        None => {
            // First run - show the latest version's changelog
            extract_latest_version_changelog()
        }
    }
}

/// Extract the changelog for just the latest version (for first-time users)
fn extract_latest_version_changelog() -> Option<String> {
    let changelog = include_str!("../CHANGELOG.md");
    let lines: Vec<&str> = changelog.lines().collect();

    let mut result = Vec::new();
    let mut found_first_version = false;

    for line in lines {
        // Check if this line is a version header
        if line.starts_with("## [") {
            if found_first_version {
                // We've found the second version header, stop collecting
                break;
            } else {
                // This is the first version header (latest version)
                found_first_version = true;
                result.push(line);
                continue;
            }
        }

        // If we're collecting the first version, add all non-version lines
        if found_first_version {
            result.push(line);
        }
    }

    if found_first_version && !result.is_empty() {
        Some(result.join("\n"))
    } else {
        None
    }
}

/// Extract changelog sections between two versions from the embedded CHANGELOG.md
fn extract_changelog_between_versions(from_version: &str, to_version: &str) -> Option<String> {
    let changelog = include_str!("../CHANGELOG.md");
    let lines: Vec<&str> = changelog.lines().collect();

    let mut result = Vec::new();
    let mut in_range = false;
    let mut found_to_version = false;

    for line in lines {
        // Check if this line is a version header
        if line.starts_with("## [") {
            if let Some(version) = extract_version_from_header(line) {
                // If we find the "to" version, start collecting
                if version == to_version {
                    in_range = true;
                    found_to_version = true;
                    result.push(line);
                    continue;
                }

                // If we find the "from" version while collecting, stop
                if in_range && version == from_version {
                    break;
                }

                // If we're in range but hit another version, include it
                if in_range {
                    result.push(line);
                    continue;
                }
            }
        }

        // If we're in range, collect all lines
        if in_range {
            result.push(line);
        }
    }

    if found_to_version && !result.is_empty() {
        Some(result.join("\n"))
    } else {
        None
    }
}

/// Extract version number from a changelog header line like "## [0.1.9] - 2025-06-19"
fn extract_version_from_header(line: &str) -> Option<String> {
    if let Some(start) = line.find('[') {
        if let Some(end) = line.find(']') {
            if end > start {
                return Some(line[start + 1..end].to_string());
            }
        }
    }
    None
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

    #[test]
    fn test_extract_version_from_header() {
        assert_eq!(
            extract_version_from_header("## [0.1.9] - 2025-06-19"),
            Some("0.1.9".to_string())
        );
        assert_eq!(
            extract_version_from_header("## [1.0.0] - 2025-01-01"),
            Some("1.0.0".to_string())
        );
        assert_eq!(extract_version_from_header("### Not a version"), None);
        assert_eq!(extract_version_from_header("## No brackets"), None);
    }

    #[test]
    fn test_extract_latest_version_changelog() {
        // This tests the function with the actual embedded changelog
        let result = extract_latest_version_changelog();
        assert!(result.is_some(), "Should find the latest version changelog");

        let changelog = result.unwrap();
        // Should start with a version header
        assert!(
            changelog.starts_with("## ["),
            "Should start with version header"
        );
        // Should contain some content
        assert!(changelog.len() > 20, "Should have substantial content");
    }
}
