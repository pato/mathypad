//! File operations abstraction for different platforms

use std::path::Path;

/// Trait for file operations - allows different backends (native fs, web storage, etc.)
pub trait FileOperations {
    type Error;
    
    /// Save content to a file path
    fn save_content(&self, path: &Path, content: &str) -> Result<(), Self::Error>;
    
    /// Load content from a file path
    fn load_content(&self, path: &Path) -> Result<String, Self::Error>;
}

/// Serialize text lines into a single string for file storage
pub fn serialize_lines(lines: &[String]) -> String {
    lines.join("\n")
}

/// Deserialize file content into individual text lines
pub fn deserialize_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
        vec![String::new()]
    } else {
        content.lines().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_lines() {
        let lines = vec!["line1".to_string(), "line2".to_string(), "line3".to_string()];
        let content = serialize_lines(&lines);
        assert_eq!(content, "line1\nline2\nline3");
    }

    #[test]
    fn test_serialize_empty_lines() {
        let lines = Vec::<String>::new();
        let content = serialize_lines(&lines);
        assert_eq!(content, "");
    }

    #[test]
    fn test_deserialize_lines() {
        let content = "line1\nline2\nline3";
        let lines = deserialize_lines(content);
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_deserialize_empty_content() {
        let lines = deserialize_lines("");
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn test_round_trip() {
        let original_lines = vec!["5 + 3".to_string(), "line1 * 2".to_string()];
        let content = serialize_lines(&original_lines);
        let restored_lines = deserialize_lines(&content);
        assert_eq!(original_lines, restored_lines);
    }
}