use lint::{ConfigBuilder, OutputFormat};
use std::path::PathBuf;

#[test]
fn test_lint_files_empty_config() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples")])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(results.len() >= 1);
}

#[test]
fn test_lint_files_with_rules() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
        ])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].messages.len() > 0);
}

#[test]
fn test_lint_files_single_file() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].file_path, PathBuf::from("examples/test_file.rs"));
}

#[test]
fn test_lint_files_max_line_length() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .max_line_length(Some(50))
        .enabled_rules(vec!["line-length".to_string()])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(!results[0].messages.is_empty());
    assert!(results[0].has_warnings());
}

#[test]
fn test_create_default_config() {
    let config = lint::create_default_config();
    assert_eq!(config.max_line_length, Some(100));
    assert_eq!(config.rule_set.enabled_rules.len(), 2);
}

#[test]
fn test_lint_with_output_format_json() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .enabled_rules(vec![])
        .output_format(OutputFormat::Json)
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);

    let json = serde_json::to_string(&results[0]).unwrap();
    assert!(json.contains("file_path"));
    assert!(json.contains("messages"));
}
