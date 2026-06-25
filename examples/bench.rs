use lint::ConfigBuilder;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir)?;

    let file_count = 100;
    let lines_per_file = 500;

    for i in 0..file_count {
        let mut content = String::new();
        for j in 0..lines_per_file {
            content.push_str(&format!(
                "fn function_{}_{}() {{ println!(\"Hello\"); }}\n",
                i, j
            ));
        }
        std::fs::write(src_dir.join(format!("file_{}.rs", i)), content)?;
    }

    let config = ConfigBuilder::new()
        .paths(vec![src_dir.clone()])
        .max_line_length(Some(50))
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
        ])
        .build();

    let start = Instant::now();
    let results = lint::lint_files(&config)?;
    let elapsed = start.elapsed();

    let total_files = results.len();
    let total_issues: usize = results.iter().map(|r| r.messages.len()).sum();

    println!(
        "Linted {} files ({} lines each) in {:?}",
        total_files, lines_per_file, elapsed
    );
    println!("Found {} total issues", total_issues);

    Ok(())
}
