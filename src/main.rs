use clap::{Parser, Subcommand};
use colored::Colorize;
use lint::{ConfigBuilder, OutputFormat};
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
    },
    ListRules,
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint {
            paths,
            output,
            max_line_length,
            rules,
        } => {
            let mut builder = ConfigBuilder::new().paths(paths);

            if let Some(length) = max_line_length {
                builder = builder.max_line_length(Some(length));
            }

            if let Some(rules) = rules {
                builder = builder.enabled_rules(rules);
            }

            let output_format = match output.as_deref() {
                Some("json") => OutputFormat::Json,
                Some("markdown") => OutputFormat::Markdown,
                _ => OutputFormat::Text,
            };
            builder = builder.output_format(output_format.clone());

            let config = builder.build();
            let results = lint::lint_files(&config)?;

            print_results(&results, output_format);
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
