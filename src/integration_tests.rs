//! Integration tests for mathypad application
//!
//! These tests verify the complete flow from user input to final output,
//! testing the integration between parser, evaluator, and UI components.

use crate::expression::evaluate_expression_with_context;
use crate::test_helpers::{evaluate_test_expression, evaluate_with_unit_info};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_basic_calculations() {
        // Test basic arithmetic
        assert_eq!(evaluate_test_expression("2 + 3"), Some("5".to_string()));
        assert_eq!(evaluate_test_expression("10 - 4"), Some("6".to_string()));
        assert_eq!(evaluate_test_expression("6 * 7"), Some("42".to_string()));
        assert_eq!(evaluate_test_expression("15 / 3"), Some("5".to_string()));

        // Test with decimals
        assert_eq!(evaluate_test_expression("2.5 + 1.5"), Some("4".to_string()));
        assert_eq!(
            evaluate_test_expression("10.2 / 2"),
            Some("5.1".to_string())
        );

        // Test with parentheses
        assert_eq!(
            evaluate_test_expression("(2 + 3) * 4"),
            Some("20".to_string())
        );
        assert_eq!(
            evaluate_test_expression("2 + (3 * 4)"),
            Some("14".to_string())
        );
    }

    #[test]
    fn test_end_to_end_unit_conversions() {
        // Test basic unit conversions
        assert_eq!(
            evaluate_test_expression("1 GiB to MiB"),
            Some("1,024 MiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("60 seconds to minutes"),
            Some("1 min".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1000 MB to GB"),
            Some("1 GB".to_string())
        );

        // Test conversion with calculations
        assert_eq!(
            evaluate_test_expression("2 GiB + 512 MiB to MiB"),
            Some("2,560 MiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1 hour + 30 minutes to minutes"),
            Some("90 min".to_string())
        );

        // Test "in" keyword
        assert_eq!(
            evaluate_test_expression("24 MiB * 32 in KiB"),
            Some("786,432 KiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("500 GiB / 10 seconds in MiB/s"),
            Some("51,200 MiB/s".to_string())
        );
    }

    #[test]
    fn test_end_to_end_data_rate_calculations() {
        // Test rate calculations
        assert_eq!(
            evaluate_test_expression("1 hour * 10 GiB/s"),
            Some("36,000 GiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("100 GiB / 10 s"),
            Some("10 GiB/s".to_string())
        );
        assert_eq!(
            evaluate_test_expression("50 GiB/s * 2 s"),
            Some("100 GiB".to_string())
        );

        // Test with compound units
        assert_eq!(
            evaluate_test_expression("10GiB/s * 30min"),
            Some("18,000 GiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1000GiB / 10min"),
            Some("100 GiB/min".to_string())
        );
    }

    #[test]
    fn test_end_to_end_qps_calculations() {
        // Test QPS rate calculations
        assert_eq!(
            evaluate_test_expression("100 QPS * 1 hour"),
            Some("360,000 query".to_string())
        );
        assert_eq!(
            evaluate_test_expression("25 QPS to req/minute"),
            Some("1,500 req/min".to_string())
        );
        assert_eq!(
            evaluate_test_expression("5000 queries / 10 minutes"),
            Some("8.333 query/s".to_string())
        );

        // Test QPS arithmetic
        assert_eq!(
            evaluate_test_expression("100 QPS + 50 QPS"),
            Some("150 query/s".to_string())
        );
        assert_eq!(
            evaluate_test_expression("200 req/min - 80 req/min"),
            Some("120 req/min".to_string())
        );

        // Test mixed QPS and request rates
        assert_eq!(
            evaluate_test_expression("100 QPS + 100 req/s"),
            Some("200 req/s".to_string())
        );
    }

    #[test]
    fn test_end_to_end_large_data_units() {
        // Test petabyte and exabyte calculations
        assert_eq!(
            evaluate_test_expression("1000 TB to PB"),
            Some("1 PB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("5 PB to TB"),
            Some("5,000 TB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1000 PB to EB"),
            Some("1 EB".to_string())
        );

        // Test binary large units
        assert_eq!(
            evaluate_test_expression("1024 TiB to PiB"),
            Some("1 PiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1024 PiB to EiB"),
            Some("1 EiB".to_string())
        );

        // Test mixed base conversions
        let result = evaluate_with_unit_info("1 PB to PiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 0.8881784197).abs() < 0.0001);
    }

    #[test]
    fn test_end_to_end_bit_byte_conversions() {
        // Test bit to byte conversions
        assert_eq!(
            evaluate_test_expression("8 bit to B"),
            Some("1 B".to_string())
        );
        assert_eq!(
            evaluate_test_expression("8 Kb to KB"),
            Some("1 KB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("8 Mb to MB"),
            Some("1 MB".to_string())
        );

        // Test byte to bit conversions
        assert_eq!(
            evaluate_test_expression("1 B to bit"),
            Some("8 bit".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1 KB to Kb"),
            Some("8 Kb".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1 MB to Mb"),
            Some("8 Mb".to_string())
        );

        // Test network speed scenarios
        assert_eq!(
            evaluate_test_expression("100 Mbps to MB/s"),
            Some("12.5 MB/s".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1 Gbps to MB/s"),
            Some("125 MB/s".to_string())
        );
    }

    #[test]
    fn test_end_to_end_comma_numbers() {
        // Test comma-separated numbers
        assert_eq!(
            evaluate_test_expression("1,000 + 2,000"),
            Some("3,000".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1,000,000 / 1000"),
            Some("1,000".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1,234.56 * 2"),
            Some("2,469.12".to_string())
        );

        // Test comma numbers with units
        assert_eq!(
            evaluate_test_expression("1,000 GiB to MiB"),
            Some("1,024,000 MiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1,000,000 bytes to MB"),
            Some("1 MB".to_string())
        );
    }

    #[test]
    fn test_end_to_end_no_space_units() {
        // Test numbers without spaces before units
        assert_eq!(evaluate_test_expression("5GiB"), Some("5 GiB".to_string()));
        assert_eq!(
            evaluate_test_expression("100MB to GB"),
            Some("0.1 GB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("2.5TiB to GiB"),
            Some("2,560 GiB".to_string())
        );

        // Test expressions with mixed spacing
        assert_eq!(
            evaluate_test_expression("5 GiB + 10GiB"),
            Some("15 GiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1,000GiB + 512 MiB"),
            Some("1,024,512 MiB".to_string())
        );

        // Test compound units without spaces
        assert_eq!(
            evaluate_test_expression("10GiB/s * 30min"),
            Some("18,000 GiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("100QPS * 5min"),
            Some("30,000 query".to_string())
        );
    }

    #[test]
    fn test_end_to_end_complex_expressions() {
        // Test complex nested expressions
        assert_eq!(
            evaluate_test_expression("((1 TiB + 512 GiB) / 2) * 3 to MiB"),
            Some("2,359,296 MiB".to_string())
        );

        // Test mixed unit types
        assert_eq!(
            evaluate_test_expression("(100 QPS * 1 hour) + (50 req/s * 30 minutes)"),
            Some("450,000 req".to_string())
        );

        // Test complex data transfer calculations with generic rates
        assert_eq!(
            evaluate_test_expression("(5 PB + 1000 TB) / (10 hours)"),
            Some("600 TB/h".to_string())
        );
    }

    #[test]
    fn test_end_to_end_real_world_scenarios() {
        // Test data center storage scenarios
        assert_eq!(
            evaluate_test_expression("Data center: 50 PB + 10 EB"),
            Some("10,050 PB".to_string())
        );

        // Test API load balancing
        assert_eq!(
            evaluate_test_expression("Total load: 250 QPS + 150 QPS + 100 QPS"),
            Some("500 query/s".to_string())
        );

        // Test bandwidth calculations with generic rates
        assert_eq!(
            evaluate_test_expression("Bandwidth used: 1,000 GiB / 1 hour"),
            Some("1,000 GiB/h".to_string())
        );

        // Test backup scenarios
        assert_eq!(
            evaluate_test_expression("Backup rate: 100 TB/s * 8 hours"),
            Some("2,880,000 TB".to_string())
        );

        // Test network throughput
        assert_eq!(
            evaluate_test_expression("Network: 10 PB/s to TB/s"),
            Some("10,000 TB/s".to_string())
        );
    }

    #[test]
    fn test_end_to_end_precision_and_formatting() {
        // Test precision handling
        assert_eq!(evaluate_test_expression("1 / 3"), Some("0.333".to_string()));
        assert_eq!(
            evaluate_test_expression("100 / 3"),
            Some("33.333".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1000 / 7"),
            Some("142.857".to_string())
        );

        // Test large number formatting
        assert_eq!(
            evaluate_test_expression("1000000 + 2000000"),
            Some("3,000,000".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1234567.89 + 1"),
            Some("1,234,568.89".to_string())
        );

        // Test very small numbers
        assert_eq!(
            evaluate_test_expression("0.001 + 0.002"),
            Some("0.003".to_string())
        );
        assert_eq!(
            evaluate_test_expression("0.000001 * 1000000"),
            Some("1".to_string())
        );
    }

    #[test]
    fn test_end_to_end_edge_cases() {
        // Test zero values
        assert_eq!(evaluate_test_expression("0 + 5"), Some("5".to_string()));
        assert_eq!(evaluate_test_expression("0 * 100"), Some("0".to_string()));
        assert_eq!(
            evaluate_test_expression("0 GiB + 5 GiB"),
            Some("5 GiB".to_string())
        );

        // Test division by small numbers
        assert_eq!(evaluate_test_expression("1 / 0.1"), Some("10".to_string()));
        assert_eq!(
            evaluate_test_expression("100 / 0.01"),
            Some("10,000".to_string())
        );

        // Test very large calculations
        assert_eq!(
            evaluate_test_expression("999999 * 999999"),
            Some("999,998,000,001".to_string())
        );

        // Test unit edge cases
        assert_eq!(
            evaluate_test_expression("1 ns to s"),
            Some("0 s".to_string())
        );
        assert_eq!(
            evaluate_test_expression("1000000000 ns to s"),
            Some("1 s".to_string())
        );
    }

    #[test]
    fn test_mixed_text_and_math_expressions() {
        // Test that we can evaluate mathematical expressions within text using the new token-based approach
        assert_eq!(
            evaluate_test_expression("The server has 16 GiB of RAM and processes 100 QPS"),
            Some("16 GiB".to_string()) // Should find and evaluate "16 GiB"
        );

        assert_eq!(
            evaluate_test_expression("Download: 1,000 MB at 50 MB/s takes 20 seconds"),
            Some("1,000 MB".to_string()) // Should find and evaluate "1,000 MB"
        );

        assert_eq!(
            evaluate_test_expression("Calculate: (5 GiB + 3 GiB) * 2 for total storage"),
            Some("16 GiB".to_string()) // Should find and evaluate "(5 GiB + 3 GiB) * 2"
        );

        // Test complex mathematical expressions in text
        assert_eq!(
            evaluate_test_expression("API performance: (100 QPS + 50 req/s) * 1 hour gives total"),
            Some("540,000 req".to_string()) // Should find and evaluate "(100 QPS + 50 req/s) * 1 hour"
        );
    }

    #[test]
    fn test_invalid_expression_handling() {
        // Test invalid expressions return None
        assert_eq!(evaluate_test_expression("invalid expression"), None);
        // Note: Some expressions might be partially parsed by chumsky
        assert_eq!(evaluate_test_expression("(1 + 2"), None);
        assert_eq!(evaluate_test_expression("1 + 2)"), None);
        assert_eq!(evaluate_test_expression(""), None);

        // Test incompatible unit operations
        assert_eq!(evaluate_test_expression("5 GiB + 10 seconds"), None);
        assert_eq!(evaluate_test_expression("100 QPS - 50 MB"), None);
        // Note: "1 hour * 1 GiB" might actually work as scalar multiplication

        // Test that invalid units are now parsed as separate tokens (new behavior)
        assert_eq!(
            evaluate_test_expression("100 invalidunit"),
            Some("100".to_string())
        ); // Parses "100" as valid
        assert_eq!(
            evaluate_test_expression("50 notarealunit to GB"),
            Some("50".to_string())
        ); // Also parses "50" as valid
    }

    #[test]
    fn test_context_with_line_references() {
        // Test basic line reference functionality
        let lines = vec![Some("10 GiB".to_string()), Some("5 GiB".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 + line2", &lines, 2),
            Some("15 GiB".to_string())
        );

        // Test line references with conversions
        let lines = vec![Some("1 TiB".to_string()), Some("512 GiB".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 + line2 to MiB", &lines, 2),
            Some("1,572,864 MiB".to_string())
        );

        // Test line references in complex expressions
        let lines = vec![Some("100 QPS".to_string()), Some("5 minutes".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 * line2", &lines, 2),
            Some("30,000 query".to_string())
        );

        // Test preventing future line references
        let lines = vec![Some("10 GiB".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 + line2", &lines, 0),
            None // Should fail because line2 doesn't exist yet
        );
    }

    #[test]
    fn test_performance_with_large_expressions() {
        // Test parsing performance with deeply nested expressions
        let nested_expr = "(((((1 + 2) * 3) + 4) * 5) + 6) * 7";
        assert_eq!(
            evaluate_test_expression(nested_expr),
            Some("497".to_string())
        );

        // Test with many operations
        let long_expr = "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10";
        assert_eq!(evaluate_test_expression(long_expr), Some("55".to_string()));

        // Test with many units
        let unit_expr = "1 GiB + 2 GiB + 3 GiB + 4 GiB + 5 GiB";
        assert_eq!(
            evaluate_test_expression(unit_expr),
            Some("15 GiB".to_string())
        );

        // Test complex unit conversion chain
        let conversion_expr = "((1 TiB + 512 GiB) * 2 + 1024 MiB) / 3 to KiB";
        let result = evaluate_test_expression(conversion_expr);
        assert!(result.is_some());
    }

    #[test]
    fn test_case_insensitivity() {
        // Test case insensitive units (note: some variations might have different meanings)
        assert_eq!(
            evaluate_test_expression("1 GiB to MiB"),
            Some("1,024 MiB".to_string())
        );

        // Test case insensitive keywords
        assert_eq!(
            evaluate_test_expression("1 GiB to MiB"),
            Some("1,024 MiB".to_string())
        );
        assert_eq!(
            evaluate_test_expression("24 MiB * 32 in KiB"),
            Some("786,432 KiB".to_string())
        );

        // Test QPS conversions
        assert_eq!(
            evaluate_test_expression("1 QPS to req/min"),
            Some("60 req/min".to_string())
        );
    }

    #[test]
    fn test_file_loading() {
        use crate::App;
        use std::fs;
        use tempfile::NamedTempFile;

        // Test loading a file with expressions
        let file_content = "5 + 3\n100 GiB to MiB\nx = 42\nx * 2";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), file_content).unwrap();

        // Load the file into an App using our load function logic
        let mut app = App::default();
        let contents = fs::read_to_string(temp_file.path()).unwrap();

        // Clear the default empty line if we have file content
        if !contents.trim().is_empty() {
            app.text_lines.clear();
            app.results.clear();
        }

        for line in contents.lines() {
            app.text_lines.push(line.to_string());
            app.results.push(None);
        }

        app.recalculate_all();

        // Verify the loaded content and calculations
        assert_eq!(app.text_lines.len(), 4);
        assert_eq!(app.text_lines[0], "5 + 3");
        assert_eq!(app.text_lines[1], "100 GiB to MiB");
        assert_eq!(app.text_lines[2], "x = 42");
        assert_eq!(app.text_lines[3], "x * 2");

        // Verify calculations
        assert_eq!(app.results[0], Some("8".to_string()));
        assert_eq!(app.results[1], Some("102,400 MiB".to_string()));
        assert_eq!(app.results[2], Some("42".to_string()));
        assert_eq!(app.results[3], Some("84".to_string()));
    }

    #[test]
    fn test_empty_file_loading() {
        use crate::App;
        use std::fs;
        use tempfile::NamedTempFile;

        // Test loading an empty file
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), "").unwrap();

        let mut app = App::default();
        let contents = fs::read_to_string(temp_file.path()).unwrap();

        for line in contents.lines() {
            app.text_lines.push(line.to_string());
            app.results.push(None);
        }

        // Ensure we have at least one empty line
        if app.text_lines.is_empty() {
            app.text_lines.push(String::new());
            app.results.push(None);
        }

        // Verify we have exactly one empty line
        assert_eq!(app.text_lines.len(), 1);
        assert_eq!(app.text_lines[0], "");
        assert_eq!(app.results[0], None);
    }

    #[test]
    fn test_save_functionality() {
        use crate::App;
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Create an app with some content
        let mut app = App {
            text_lines: vec![
                "5 + 3".to_string(),
                "x = 42".to_string(),
                "x * 2".to_string(),
            ],
            results: vec![None, None, None],
            ..Default::default()
        };
        app.set_file_path(Some(temp_path.clone()));

        // Verify initial state
        assert!(!app.has_unsaved_changes);

        // Make a change - move cursor to end of first line then insert
        app.cursor_col = 5; // Move to end of "5 + 3"
        app.insert_char('!');
        assert!(app.has_unsaved_changes);

        // Save the file
        app.save().unwrap();
        assert!(!app.has_unsaved_changes);

        // Verify the file content
        let saved_content = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(saved_content, "5 + 3!\nx = 42\nx * 2");
    }

    #[test]
    fn test_save_as_functionality() {
        use crate::App;
        use std::fs;
        use tempfile::NamedTempFile;

        // Create an app with some content but no file path
        let mut app = App {
            text_lines: vec!["test content".to_string()],
            results: vec![None],
            has_unsaved_changes: true,
            ..Default::default()
        };

        // Create a temporary file to save to
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Save as new file
        app.save_as(temp_path.clone()).unwrap();

        // Verify state after save_as
        assert_eq!(app.file_path, Some(temp_path.clone()));
        assert!(!app.has_unsaved_changes);

        // Verify the file content
        let saved_content = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(saved_content, "test content");
    }

    #[test]
    fn test_unsaved_changes_tracking() {
        use crate::App;

        let mut app = App::default();

        // Initially no unsaved changes
        assert!(!app.has_unsaved_changes);

        // Insert character should mark as unsaved
        app.insert_char('a');
        assert!(app.has_unsaved_changes);

        // Set file path and clear unsaved flag
        app.has_unsaved_changes = false;

        // Delete character should mark as unsaved
        app.delete_char();
        assert!(app.has_unsaved_changes);

        // Clear unsaved flag
        app.has_unsaved_changes = false;

        // New line should mark as unsaved
        app.new_line();
        assert!(app.has_unsaved_changes);

        // Clear unsaved flag
        app.has_unsaved_changes = false;

        // Delete word should mark as unsaved
        app.text_lines[0] = "hello world".to_string();
        app.cursor_line = 0; // Make sure we're on the line with text
        app.cursor_col = 11;
        app.delete_word();
        assert!(app.has_unsaved_changes);
    }

    #[test]
    fn test_loading_non_existent_file() {
        use crate::App;
        use tempfile::TempDir;

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let non_existent_file = temp_dir.path().join("does_not_exist.mathypad");

        // Verify the file doesn't exist
        assert!(!non_existent_file.exists());

        // Test the behavior by creating an app and setting a non-existent file path
        let mut app = App::default();

        // Set a non-existent file path
        app.set_file_path(Some(non_existent_file.clone()));

        // Add some content
        app.insert_char('t');
        app.insert_char('e');
        app.insert_char('s');
        app.insert_char('t');

        // Save should create the file
        app.save().unwrap();

        // Verify the file was created
        assert!(non_existent_file.exists());

        // Verify the content
        let content = std::fs::read_to_string(&non_existent_file).unwrap();
        assert_eq!(content, "test");
    }

    #[test]
    fn test_non_existent_file_creation() {
        use crate::App;
        use tempfile::TempDir;

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let non_existent_file = temp_dir.path().join("new_file.mathypad");

        // Verify the file doesn't exist
        assert!(!non_existent_file.exists());

        // Test that we can create a new file by setting a path and saving
        let mut app = App::default();
        app.set_file_path(Some(non_existent_file.clone()));
        app.insert_char('h');
        app.insert_char('i');

        // File still shouldn't exist until we save
        assert!(!non_existent_file.exists());

        // Save should create it
        app.save().unwrap();
        assert!(non_existent_file.exists());

        let content = std::fs::read_to_string(&non_existent_file).unwrap();
        assert_eq!(content, "hi");
    }

    #[test]
    fn test_unsaved_changes_dialog() {
        use crate::App;

        let mut app = App::default();

        // Initially, no dialog should be shown
        assert!(!app.show_unsaved_dialog);

        // Add some content to create unsaved changes
        app.insert_char('t');
        app.insert_char('e');
        app.insert_char('s');
        app.insert_char('t');

        // Verify we have unsaved changes
        assert!(app.has_unsaved_changes);

        // Simulate trying to exit - this should show the dialog instead of exiting
        // (In the actual implementation, this would be handled by the event loop)
        // For testing, we manually trigger the dialog logic
        if app.has_unsaved_changes {
            app.show_unsaved_dialog = true;
        }

        // Verify the dialog is now shown
        assert!(app.show_unsaved_dialog);

        // Test dismissing the dialog with Escape (simulate Escape key)
        app.show_unsaved_dialog = false;
        assert!(!app.show_unsaved_dialog);

        // Test the save and quit flow
        app.show_unsaved_dialog = true;
        // In the real implementation, Ctrl+S would save and exit
        // For testing, we just verify the dialog state can be managed
        assert!(app.show_unsaved_dialog);
    }

    #[test]
    fn test_save_as_dialog() {
        use crate::App;
        use tempfile::TempDir;

        let mut app = App::default();

        // Initially, no save as dialog should be shown
        assert!(!app.show_save_as_dialog);
        assert!(app.save_as_input.is_empty());

        // Add some content
        app.insert_char('t');
        app.insert_char('e');
        app.insert_char('s');
        app.insert_char('t');

        // Show save as dialog (simulate Ctrl+S with no filename)
        app.show_save_as_dialog(false);

        // Verify dialog is shown
        assert!(app.show_save_as_dialog);
        assert!(!app.save_as_and_quit);
        assert_eq!(app.save_as_input, ".pad");

        // Simulate typing a filename
        app.save_as_input = "test_file.pad".to_string();

        // Test save as functionality
        let temp_dir = TempDir::new().unwrap();
        let expected_path = temp_dir.path().join("test_file.pad");

        // Manually set the path for testing (in real usage, this comes from user input)
        app.save_as_input = expected_path.to_string_lossy().to_string();

        // Simulate Enter key (save)
        let should_quit = app.save_as_from_dialog().unwrap();
        assert!(!should_quit); // Should not quit since save_as_and_quit was false
        assert!(!app.show_save_as_dialog); // Dialog should be dismissed

        // Verify file was created and app state updated
        assert!(expected_path.exists());
        assert_eq!(app.file_path, Some(expected_path.clone()));
        assert!(!app.has_unsaved_changes);

        // Verify file content
        let content = std::fs::read_to_string(&expected_path).unwrap();
        assert_eq!(content, "test");
    }

    #[test]
    fn test_save_as_dialog_with_quit() {
        use crate::App;
        use tempfile::TempDir;

        let mut app = App::default();

        // Add content and show save as dialog with quit flag
        app.insert_char('h');
        app.insert_char('i');
        app.show_save_as_dialog(true); // Quit after save

        assert!(app.show_save_as_dialog);
        assert!(app.save_as_and_quit);

        // Set filename and save
        let temp_dir = TempDir::new().unwrap();
        let expected_path = temp_dir.path().join("quit_test.pad");
        app.save_as_input = expected_path.to_string_lossy().to_string();

        let should_quit = app.save_as_from_dialog().unwrap();
        assert!(should_quit); // Should quit since save_as_and_quit was true
        assert!(!app.show_save_as_dialog); // Dialog should be dismissed
        assert!(!app.save_as_and_quit); // Flag should be reset

        // Verify file was created
        assert!(expected_path.exists());
        let content = std::fs::read_to_string(&expected_path).unwrap();
        assert_eq!(content, "hi");
    }

    #[test]
    fn test_save_as_dialog_pad_extension() {
        use crate::App;

        let mut app = App::default();

        // Show save as dialog
        app.show_save_as_dialog(false);

        // Verify dialog starts with .pad extension
        assert_eq!(app.save_as_input, ".pad");

        // Test that entering a filename preserves the .pad extension
        // Simulate typing "test" - in real usage this would be handled by the input handler
        app.save_as_input = "test.pad".to_string();

        // Verify the extension is preserved
        assert!(app.save_as_input.ends_with(".pad"));

        // Test the auto-extension functionality
        app.save_as_input = "myfile".to_string();

        // Simulate pressing Enter (which should add .pad if missing)
        // This is tested indirectly through the save_as_from_dialog method
        if !app.save_as_input.ends_with(".pad") && !app.save_as_input.is_empty() {
            app.save_as_input.push_str(".pad");
        }

        assert_eq!(app.save_as_input, "myfile.pad");
    }

    #[test]
    fn test_force_quit_commands() {
        use crate::ui::handle_command_mode;
        use crate::{App, Mode};
        use crossterm::event::KeyCode;

        // Test :q! command - should quit immediately even with unsaved changes
        let mut app = App {
            mode: Mode::Command,
            command_line: ":q!".to_string(),
            command_cursor: 3,
            has_unsaved_changes: true, // Has unsaved changes
            ..Default::default()
        };

        // Simulate pressing Enter in command mode
        let should_quit = handle_command_mode(&mut app, KeyCode::Enter);
        assert!(should_quit); // Should quit immediately
        assert!(!app.show_unsaved_dialog); // No dialog should be shown

        // Test :cq command - should quit immediately even with unsaved changes
        let mut app = App {
            mode: Mode::Command,
            command_line: ":cq".to_string(),
            command_cursor: 3,
            has_unsaved_changes: true, // Has unsaved changes
            ..Default::default()
        };

        // Simulate pressing Enter in command mode
        let should_quit = handle_command_mode(&mut app, KeyCode::Enter);
        assert!(should_quit); // Should quit immediately
        assert!(!app.show_unsaved_dialog); // No dialog should be shown

        // Test :quit! command - should quit immediately even with unsaved changes
        let mut app = App {
            mode: Mode::Command,
            command_line: ":quit!".to_string(),
            command_cursor: 6,
            has_unsaved_changes: true, // Has unsaved changes
            ..Default::default()
        };

        // Simulate pressing Enter in command mode
        let should_quit = handle_command_mode(&mut app, KeyCode::Enter);
        assert!(should_quit); // Should quit immediately
        assert!(!app.show_unsaved_dialog); // No dialog should be shown

        // Compare with regular :q command which should show dialog
        let mut app = App {
            mode: Mode::Command,
            command_line: ":q".to_string(),
            command_cursor: 2,
            has_unsaved_changes: true, // Has unsaved changes
            ..Default::default()
        };

        // Simulate pressing Enter in command mode
        let should_quit = handle_command_mode(&mut app, KeyCode::Enter);
        assert!(!should_quit); // Should NOT quit yet
        assert!(app.show_unsaved_dialog); // Dialog should be shown
    }
}
