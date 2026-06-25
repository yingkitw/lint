use clap::{Parser, Subcommand};
use colored::Colorize;
use lint::{Config, ConfigBuilder, OutputFormat};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lint")]
#[command(about = "A versatile linting tool with multiple interfaces", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Lint {
        #[arg(required = true, value_name = "PATHS")]
        paths: Vec<PathBuf>,

        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,

        #[arg(short, long, value_name = "LENGTH")]
        max_line_length: Option<usize>,

        #[arg(short = 'r', long)]
        rules: Option<Vec<String>>,

        #[arg(long)]
        fix: bool,
    },
    ListRules,
    Version,
}

fn load_config_file(path: &PathBuf) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;
    let config: Config = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?;
    Ok(config)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint {
            paths,
            output,
            max_line_length,
            rules,
            fix,
        } => {
            let mut config = if let Some(config_path) = &cli.config {
                load_config_file(config_path)?
            } else {
                ConfigBuilder::new().build()
            };

            config.paths = paths;

            if let Some(length) = max_line_length {
                config.max_line_length = Some(length);
            }

            if let Some(rules) = rules {
                config.rule_set.enabled_rules = rules;
            }

            let output_format = match output.as_deref() {
                Some("json") => OutputFormat::Json,
                Some("markdown") => OutputFormat::Markdown,
                _ => config.output_format.clone(),
            };
            config.output_format = output_format.clone();

            let mut results = lint::lint_files(&config)?;

            if fix {
                let mut fixed_count = 0;
                for result in &mut results {
                    if result.apply_fixes() {
                        std::fs::write(&result.file_path, &result.file_content)?;
                        fixed_count += 1;
                    }
                }
                if fixed_count > 0 {
                    println!("Fixed {} file(s)", fixed_count);
                }
            }

            print_results(&results, output_format);

            if results.iter().any(|r| r.has_errors()) {
                std::process::exit(1);
            }
        }
        Commands::ListRules => {
            list_rules();
        }
        Commands::Version => {
            println!("lint {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

fn print_results(results: &[lint::LintResult], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(results).unwrap());
        }
        OutputFormat::Markdown => {
            for result in results {
                println!("# {}", result.file_path.display());
                for msg in &result.messages {
                    println!(
                        "- **Line {}**: [{}] {} (`{}`)",
                        msg.line,
                        msg.severity.as_str(),
                        msg.message,
                        msg.rule
                    );
                    if let Some(suggestion) = &msg.suggestion {
                        println!("  - **Fix**: {}", suggestion);
                    }
                }
                println!();
            }
        }
        OutputFormat::Text => {
            let mut total_errors = 0;
            let mut total_warnings = 0;
            let mut total_infos = 0;

            for result in results {
                if result.messages.is_empty() {
                    println!("{}: No issues found", result.file_path.display());
                } else {
                    println!("{}", format!("{}", result.file_path.display()).bold());
                    for msg in &result.messages {
                        let (prefix, color) = match msg.severity {
                            lint::Severity::Error => {
                                total_errors += 1;
                                ("error", colored::Color::Red)
                            }
                            lint::Severity::Warning => {
                                total_warnings += 1;
                                ("warning", colored::Color::Yellow)
                            }
                            lint::Severity::Info => {
                                total_infos += 1;
                                ("info", colored::Color::Cyan)
                            }
                        };

                        println!(
                            "  {}:{} {}: {}",
                            msg.line,
                            msg.column,
                            prefix.color(color),
                            msg.message
                        );

                        if let Some(suggestion) = &msg.suggestion {
                            println!("    {} help: {}", "→".dimmed(), suggestion.dimmed());
                        }
                    }
                }
                println!();
            }

            println!(
                "{}",
                format!(
                    "Summary: {} errors, {} warnings, {} infos",
                    total_errors, total_warnings, total_infos
                )
                .bold()
            );
        }
    }
}

fn list_rules() {
    println!("Available rules (generic, apply to all files):");
    println!("  line-length          - Lines exceeding max length (fix: break line)");
    println!("  trailing-whitespace   - Trailing spaces/tabs (fix: remove)");
    println!("  no-todo              - TODO/FIXME comments (fix: address or create issue)");
    println!();
    println!("Language-specific rules (auto-applied by file extension):");
    println!("  JS/TS:    no-console-log, no-var");
    println!("  Python:  no-print, python-style");
    println!("  Rust:    no-unwrap, no-expect");
    println!("  Java:    java-style (PascalCase, no System.out)");
    println!("  Go:      go-style");
    println!("  Ruby:    no-puts, ruby-style");
    println!("  PHP:     no-echo");
    println!("  Swift:   no-swift-print");
    println!("  Kotlin:  kotlin-style");
    println!("  Dart:    no-dart-print");
    println!("  C#:      no-csharp-console, csharp-style");
    println!("  Shell:   shell-echo-quote");
    println!("  SQL:     sql-no-select-star");
    println!("  Lua:     no-lua-print");
    println!("  Scala:   no-scala-println");
    println!("  R:       no-r-print");
    println!("  Zig:     no-zig-debug-print");
    println!("  HTML:    html-no-inline-style, html-img-alt");
    println!("  CSS:     css-avoid-important");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_config_file_success() -> anyhow::Result<()> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        let config_json = r#"{
            "paths": ["src"],
            "ignore_patterns": ["target"],
            "max_line_length": 120,
            "rule_set": {
                "enabled_rules": ["line-length"],
                "custom_rules_path": null
            },
            "output_format": "Json"
        }"#;
        temp_file.write_all(config_json.as_bytes())?;
        temp_file.flush()?;

        let config = load_config_file(&temp_file.path().to_path_buf())?;
        assert_eq!(config.paths, vec![PathBuf::from("src")]);
        assert_eq!(config.max_line_length, Some(120));
        assert_eq!(config.rule_set.enabled_rules, vec!["line-length".to_string()]);
        assert_eq!(config.output_format, OutputFormat::Json);

        Ok(())
    }

    #[test]
    fn test_load_config_file_not_found() {
        let result = load_config_file(&PathBuf::from("nonexistent_config.json"));
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Failed to read config file"));
    }

    #[test]
    fn test_load_config_file_invalid_json() -> anyhow::Result<()> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(b"not json")?;
        temp_file.flush()?;

        let result = load_config_file(&temp_file.path().to_path_buf());
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Failed to parse config file"));

        Ok(())
    }

    #[test]
    fn test_cli_lint_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "json"]);
        match cli.command {
            Commands::Lint {
                paths,
                output,
                max_line_length,
                rules,
                fix,
            } => {
                assert_eq!(paths, vec![PathBuf::from("src/")]);
                assert_eq!(output, Some("json".to_string()));
                assert_eq!(max_line_length, None);
                assert_eq!(rules, None);
                assert!(!fix);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_lint_parsing_with_config() {
        let cli = Cli::parse_from(["lint", "--config", ".lint.json", "lint", "src/"]);
        assert_eq!(cli.config, Some(PathBuf::from(".lint.json")));
        match cli.command {
            Commands::Lint { paths, .. } => {
                assert_eq!(paths, vec![PathBuf::from("src/")]);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_list_rules_parsing() {
        let cli = Cli::parse_from(["lint", "list-rules"]);
        match cli.command {
            Commands::ListRules => {}
            _ => panic!("Expected ListRules command"),
        }
    }

    #[test]
    fn test_cli_version_parsing() {
        let cli = Cli::parse_from(["lint", "version"]);
        match cli.command {
            Commands::Version => {}
            _ => panic!("Expected Version command"),
        }
    }

    #[test]
    fn test_cli_fix_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--fix"]);
        match cli.command {
            Commands::Lint { fix, .. } => {
                assert!(fix);
            }
            _ => panic!("Expected Lint command"),
        }
    }
}
