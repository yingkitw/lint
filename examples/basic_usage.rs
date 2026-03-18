use lint::{ConfigBuilder, OutputFormat};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .paths(vec![PathBuf::from("examples")])
        .max_line_length(Some(80))
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
            "no-todo".to_string(),
        ])
        .output_format(OutputFormat::Text)
        .build();

    let results = lint::lint_files(&config)?;

    println!("Linting results:");
    for result in &results {
        println!("\nFile: {}", result.file_path.display());
        if result.messages.is_empty() {
            println!("  ✓ No issues found");
        } else {
            for message in &result.messages {
                println!(
                    "  [{}] Line {}: {}",
                    message.severity.as_str(),
                    message.line,
                    message.message
                );
            }
        }
    }

    Ok(())
}
