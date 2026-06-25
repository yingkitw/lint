use lint::ConfigBuilder;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let large_file = temp_dir.path().join("large.rs");

    let mut content = String::new();
    for i in 0..10_000 {
        content.push_str(&format!("fn function_{}() {{\n    println!(\"Hello\");\n}}\n", i));
    }
    std::fs::write(&large_file, &content)?;

    let config = ConfigBuilder::new()
        .paths(vec![large_file.clone()])
        .max_line_length(Some(100))
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
        ])
        .build();

    let start = Instant::now();
    let results = lint::lint_files(&config)?;
    let elapsed = start.elapsed();

    println!("Linted {} bytes in {:?}", content.len(), elapsed);
    println!("Found {} issues in {}", results[0].messages.len(), large_file.display());

    Ok(())
}
