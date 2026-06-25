use lint::{ConfigBuilder, OutputFormat};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_lint_with_custom_max_length() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .max_line_length(Some(30))
        .enabled_rules(vec!["line-length".to_string()])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(results[0].has_warnings());
}

#[test]
fn test_lint_with_multiple_rules() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
            "no-todo".to_string(),
        ])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(results[0].has_warnings());
    assert!(results[0].messages.iter().any(|m| m.rule == "line-length"
        || m.rule == "trailing-whitespace"
        || m.rule == "no-todo"));
}

#[test]
fn test_lint_with_markdown_output() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples/test_file.rs")])
        .enabled_rules(vec![])
        .output_format(OutputFormat::Markdown)
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(config.output_format, OutputFormat::Markdown);
}

#[test]
fn test_lint_empty_directory() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let empty_path = temp_dir.path().join("empty");
    std::fs::create_dir(&empty_path)?;

    let config = ConfigBuilder::new()
        .paths(vec![empty_path])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config)?;
    assert!(results.is_empty());

    Ok(())
}

#[test]
fn test_lint_nonexistent_path() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("nonexistent/path/file.rs")])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_lint_with_clean_file() -> anyhow::Result<()> {
    let mut file = NamedTempFile::new()?;
    let content = "fn main() {\n    println!(\"Hello\");\n}";
    file.write_all(content.as_bytes())?;
    file.flush()?;

    let config = ConfigBuilder::new()
        .paths(vec![file.path().to_path_buf()])
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
            "no-todo".to_string(),
        ])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].has_errors());
    assert!(!results[0].has_warnings());

    Ok(())
}

#[test]
fn test_lint_severity_levels() -> anyhow::Result<()> {
    let mut file = NamedTempFile::new()?;
    let content = "fn main() {\n    let x = 5;   \n    // TODO: fix\n}";
    file.write_all(content.as_bytes())?;
    file.flush()?;

    let config = ConfigBuilder::new()
        .paths(vec![file.path().to_path_buf()])
        .max_line_length(Some(50))
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
            "no-todo".to_string(),
        ])
        .build();

    let results = lint::lint_files(&config).unwrap();

    let has_warning = results[0].has_warnings();
    let has_info = results[0]
        .messages
        .iter()
        .any(|m| m.severity == lint::Severity::Info);

    assert!(has_warning);
    assert!(has_info);

    Ok(())
}

#[test]
fn test_config_builder_chain() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("src")])
        .max_line_length(Some(120))
        .ignore_patterns(vec!["target".to_string()])
        .enabled_rules(vec!["line-length".to_string()])
        .output_format(OutputFormat::Json)
        .build();

    assert_eq!(config.paths.len(), 1);
    assert_eq!(config.max_line_length, Some(120));
    assert_eq!(config.ignore_patterns.len(), 1);
    assert_eq!(config.rule_set.enabled_rules.len(), 1);
}

#[test]
fn test_output_format_variants() {
    let text_format = OutputFormat::Text;
    let json_format = OutputFormat::Json;
    let markdown_format = OutputFormat::Markdown;

    assert!(matches!(text_format, OutputFormat::Text));
    assert!(matches!(json_format, OutputFormat::Json));
    assert!(matches!(markdown_format, OutputFormat::Markdown));
}

#[test]
fn test_multiple_files_linting() {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples")])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_lint_csharp_file() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("test.cs");
    std::fs::write(&path, "Console.WriteLine(\"hello\");")?;

    let config = ConfigBuilder::new()
        .paths(vec![path.clone()])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].messages.iter().any(|m| m.rule == "no-csharp-console"));

    Ok(())
}

#[test]
fn test_lint_html_file() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("test.html");
    std::fs::write(&path, r#"<img src="x.png">"#)?;

    let config = ConfigBuilder::new()
        .paths(vec![path.clone()])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].messages.iter().any(|m| m.rule == "html-img-alt"
        || m.rule == "html-no-inline-style"));

    Ok(())
}

#[test]
fn test_lint_suggestions_present() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("test.js");
    std::fs::write(&path, "console.log('hello');")?;

    let config = ConfigBuilder::new()
        .paths(vec![path.clone()])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    let msg = &results[0].messages[0];
    assert!(msg.suggestion.is_some(), "Fix suggestion should be present");
    assert!(!msg.suggestion.as_ref().unwrap().is_empty());

    Ok(())
}

#[test]
fn test_lint_sql_file() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("test.sql");
    std::fs::write(&path, "SELECT * FROM users WHERE id = 1;")?;

    let config = ConfigBuilder::new()
        .paths(vec![path.clone()])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].messages.iter().any(|m| m.rule == "sql-no-select-star"));

    Ok(())
}

#[test]
fn test_lint_shell_file() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("test.sh");
    std::fs::write(&path, "echo $HOME\n")?;

    let config = ConfigBuilder::new()
        .paths(vec![path.clone()])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].messages.iter().any(|m| m.rule == "shell-echo-quote"));

    Ok(())
}

#[test]
fn test_lint_respects_ignore_patterns() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();

    std::fs::create_dir_all(root.join("src"))?;
    std::fs::create_dir_all(root.join("node_modules").join("pkg"))?;

    std::fs::write(root.join("src/main.rs"), "fn main() {}\n")?;
    std::fs::write(
        root.join("node_modules/pkg/index.js"),
        "console.log('hello');\n",
    )?;

    let config = ConfigBuilder::new()
        .paths(vec![root.to_path_buf()])
        .ignore_patterns(vec!["node_modules".to_string()])
        .enabled_rules(vec![])
        .build();

    let results = lint::lint_files(&config)?;

    let paths: Vec<_> = results.iter().map(|r| r.file_path.clone()).collect();
    assert!(paths.iter().any(|p| p.ends_with("src/main.rs")));
    assert!(!paths.iter().any(|p| p.to_string_lossy().contains("node_modules")));

    Ok(())
}

#[test]
fn test_lint_detects_errors_with_custom_rule() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let custom_rules_path = temp_dir.path().join("rules.json");
    std::fs::write(
        &custom_rules_path,
        r#"[{"name": "no-danger","pattern": "danger","message": "Dangerous code","severity": "Error","suggestion": "Remove it","extensions": ["js"]}]"#,
    )?;

    let test_file = temp_dir.path().join("test.js");
    std::fs::write(&test_file, "danger();\n")?;

    let config = ConfigBuilder::new()
        .paths(vec![test_file.clone()])
        .custom_rules(Some(custom_rules_path))
        .enabled_rules(vec!["no-danger".to_string()])
        .build();

    let results = lint::lint_files(&config)?;
    assert_eq!(results.len(), 1);
    assert!(results[0].has_errors());
    assert_eq!(results[0].messages[0].severity, lint::Severity::Error);

    Ok(())
}
