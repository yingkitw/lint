use lint::{ConfigBuilder, OutputFormat};
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_lint_files_empty_config() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples")])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(!results.is_empty());
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
    assert!(!results[0].messages.is_empty());
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
    let config = ConfigBuilder::default().build();
    assert_eq!(config.max_line_length, Some(100));
    assert_eq!(config.rule_set.enabled_rules.len(), 4);
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

#[test]
fn test_lint_with_custom_rules() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let custom_rules_path = temp_dir.path().join("custom_rules.json");
    let mut custom_rules_file = std::fs::File::create(&custom_rules_path)?;
    let rules_json = r#"[
        {
            "name": "no-eval",
            "pattern": "\\beval\\(",
            "message": "eval() is dangerous",
            "severity": "Error",
            "suggestion": "Use a safer alternative",
            "extensions": ["js"]
        }
    ]"#;
    custom_rules_file.write_all(rules_json.as_bytes())?;
    custom_rules_file.flush()?;

    let test_file = temp_dir.path().join("test.js");
    std::fs::write(&test_file, "eval(x);\n")?;

    let config = ConfigBuilder::new()
        .paths(vec![test_file.clone()])
        .custom_rules(Some(custom_rules_path))
        .enabled_rules(vec!["no-eval".to_string()])
        .build();

    let results = lint::lint_files(&config)?;
    assert_eq!(results.len(), 1);
    assert!(results[0].messages.iter().any(|m| m.rule == "no-eval"));

    Ok(())
}

#[test]
fn test_apply_fixes_removes_trailing_whitespace() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "let x = 5;   \nlet y = 10;\n")?;

    let config = ConfigBuilder::new()
        .paths(vec![test_file.clone()])
        .enabled_rules(vec!["trailing-whitespace".to_string()])
        .build();

    let mut results = lint::lint_files(&config)?;
    assert_eq!(results.len(), 1);
    assert!(!results[0].messages.is_empty());

    assert!(results[0].apply_fixes());
    assert_eq!(results[0].file_content, "let x = 5;\nlet y = 10;\n");

    Ok(())
}

#[test]
fn test_json_output_snapshot() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.py");
    std::fs::write(&test_file, "print('hello')\n")?;

    let config = ConfigBuilder::new()
        .paths(vec![test_file.clone()])
        .enabled_rules(vec![])
        .output_format(OutputFormat::Json)
        .build();

    let results = lint::lint_files(&config)?;
    let json = serde_json::to_string_pretty(&results)?;

    assert!(json.contains("file_path"));
    assert!(json.contains("messages"));
    assert!(json.contains("no-print"));
    assert!(json.contains("severity"));
    assert!(json.contains("line"));
    assert!(json.contains("column"));
    assert!(json.contains("rule"));
    assert!(json.contains("suggestion"));

    Ok(())
}

#[test]
fn test_trailing_whitespace_exact_message() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "let x = 5;   \n")?;

    let config = ConfigBuilder::new()
        .paths(vec![test_file.clone()])
        .enabled_rules(vec!["trailing-whitespace".to_string()])
        .build();

    let results = lint::lint_files(&config)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].messages.len(), 1);

    let msg = &results[0].messages[0];
    assert_eq!(msg.line, 1);
    assert_eq!(msg.severity, lint::Severity::Warning);
    assert_eq!(msg.rule, "trailing-whitespace");
    assert_eq!(msg.message, "Trailing whitespace detected");
    assert!(
        msg.suggestion
            .as_ref()
            .unwrap()
            .contains("Delete spaces/tabs")
    );
    assert!(msg.fix.is_some());
    assert_eq!(msg.fix.as_ref().unwrap().replacement, "let x = 5;");

    Ok(())
}
