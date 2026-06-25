use clap::{Parser, Subcommand};
use colored::Colorize;
use lint::{Config, ConfigBuilder, OutputFormat};
use notify::Watcher;
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
        #[arg(value_name = "PATHS")]
        paths: Vec<PathBuf>,

        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,

        #[arg(short, long, value_name = "LENGTH")]
        max_line_length: Option<usize>,

        #[arg(short = 'r', long)]
        rules: Option<Vec<String>>,

        #[arg(long)]
        fix: bool,

        #[arg(long)]
        watch: bool,

        #[arg(long)]
        cache: bool,

        #[arg(long)]
        quiet: bool,

        #[arg(long, value_name = "N")]
        max_warnings: Option<usize>,

        #[arg(long, value_name = "WHEN")]
        color: Option<String>,

        #[arg(long, value_name = "RULES", value_delimiter = ',')]
        select: Option<Vec<String>>,

        #[arg(long, value_name = "RULES", value_delimiter = ',')]
        ignore: Option<Vec<String>>,

        #[arg(long)]
        print_files: bool,

        #[arg(long, value_name = "PATH")]
        output_file: Option<PathBuf>,

        #[arg(long)]
        stdin: bool,

        #[arg(long, value_name = "FILENAME")]
        stdin_filename: Option<String>,

        #[arg(long)]
        exit_zero: bool,

        #[arg(long)]
        statistics: bool,
    },
    ListRules,
    Version,
}

#[allow(clippy::too_many_arguments)]
fn run_lint_and_print(
    config: &lint::Config,
    fix: bool,
    output_format: &lint::OutputFormat,
    cache: Option<&std::sync::Mutex<lint::cache::Cache>>,
    quiet: bool,
    max_warnings: Option<usize>,
    output_file: Option<&PathBuf>,
    exit_zero: bool,
    statistics: bool,
) -> anyhow::Result<i32> {
    let cache_arc = cache.map(|c| std::sync::Arc::new(std::sync::Mutex::new(c.lock().unwrap().clone())));
    let mut results = if let Some(ref c) = cache_arc {
        lint::lint_files_with_cache(config, Some(c.clone()))?
    } else {
        lint::lint_files(config)?
    };

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

    if let Some(path) = output_file {
        colored::control::set_override(false);
        let output = render_results(&results, output_format.clone(), quiet);
        colored::control::unset_override();
        std::fs::write(path, output)?;
    } else {
        print_results(&results, output_format.clone(), quiet);
    }

    if statistics && !quiet {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for result in &results {
            for msg in &result.messages {
                *counts.entry(msg.rule.clone()).or_insert(0) += 1;
            }
        }
        if !counts.is_empty() {
            let mut pairs: Vec<_> = counts.into_iter().collect();
            pairs.sort_by(|a, b| b.1.cmp(&a.1));
            println!("\n{}", "Statistics:".bold());
            for (rule, count) in pairs {
                println!("  {}: {}", rule, count);
            }
        }
    }

    let has_errors = results.iter().any(|r| r.has_errors());
    let warning_count: usize = results
        .iter()
        .map(|r| r.messages.iter().filter(|m| m.severity == lint::Severity::Warning).count())
        .sum();
    let max_warnings_exceeded = max_warnings.is_some_and(|max| warning_count > max);

    Ok(if exit_zero {
        0
    } else if has_errors || max_warnings_exceeded {
        1
    } else {
        0
    })
}

fn load_config_file(path: &PathBuf) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;
    let mut config: Config = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?;

    if let Some(ref extends) = config.extends {
        let base_path = if std::path::Path::new(extends).is_absolute() {
            PathBuf::from(extends)
        } else if let Some(parent) = path.parent() {
            parent.join(extends)
        } else {
            PathBuf::from(extends)
        };
        let base_config = load_config_file(&base_path)?;
        config = merge_configs(base_config, config);
    }

    Ok(config)
}

fn merge_configs(base: Config, local: Config) -> Config {
    let mut ignore_patterns = base.ignore_patterns;
    for p in &local.ignore_patterns {
        if !ignore_patterns.contains(p) {
            ignore_patterns.push(p.clone());
        }
    }

    let mut per_file_ignores = base.per_file_ignores;
    for (k, v) in local.per_file_ignores {
        per_file_ignores.insert(k, v);
    }

    let mut severity_overrides = base.severity_overrides;
    for (k, v) in local.severity_overrides {
        severity_overrides.insert(k, v);
    }

    Config {
        paths: if local.paths.is_empty() { base.paths } else { local.paths },
        ignore_patterns,
        max_line_length: local.max_line_length.or(base.max_line_length),
        rule_set: lint::config::RuleSetConfig {
            enabled_rules: if local.rule_set.enabled_rules.is_empty() {
                base.rule_set.enabled_rules
            } else {
                local.rule_set.enabled_rules
            },
            custom_rules_path: local.rule_set.custom_rules_path.or(base.rule_set.custom_rules_path),
        },
        output_format: local.output_format,
        per_file_ignores,
        severity_overrides,
        extends: local.extends,
    }
}

fn expand_globs(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut expanded = Vec::new();
    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains('*') || path_str.contains('?') || path_str.contains('[') {
            match glob::glob(&path_str) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        expanded.push(entry);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: invalid glob pattern {}: {}", path.display(), e);
                }
            }
        } else {
            expanded.push(path);
        }
    }
    expanded
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
            watch,
            cache,
            quiet,
            max_warnings,
            color,
            select,
            ignore,
            print_files,
            output_file,
            stdin,
            stdin_filename,
            exit_zero,
            statistics,
        } => {
            match color.as_deref() {
                Some("never") | Some("no") => colored::control::set_override(false),
                Some("always") | Some("yes") => colored::control::set_override(true),
                _ => {}
            }
            let mut config = if let Some(config_path) = &cli.config {
                load_config_file(config_path)?
            } else {
                ConfigBuilder::new().build()
            };

            let stdin_temp_path: Option<PathBuf> = if stdin {
                use std::io::Read;
                let mut content = String::new();
                std::io::stdin().read_to_string(&mut content)?;
                let suffix = stdin_filename
                    .as_ref()
                    .and_then(|f| std::path::Path::new(f).extension())
                    .and_then(|e| e.to_str())
                    .unwrap_or("rs");
                let temp_path = std::env::temp_dir().join(format!("lint_stdin_{}.{}"
                    , std::process::id()
                    , suffix));
                std::fs::write(&temp_path, content)?;
                Some(temp_path)
            } else {
                None
            };

            if let Some(ref temp_path) = stdin_temp_path {
                config.paths = vec![temp_path.clone()];
            } else if paths.is_empty() {
                config.paths = vec![PathBuf::from(".")];
            } else {
                config.paths = expand_globs(paths);
            }

            if let Some(length) = max_line_length {
                config.max_line_length = Some(length);
            }

            if let Some(rules) = rules {
                config.rule_set.enabled_rules = rules;
            } else {
                if let Some(select) = select {
                    for rule in select {
                        if !config.rule_set.enabled_rules.contains(&rule) {
                            config.rule_set.enabled_rules.push(rule);
                        }
                    }
                }
                if let Some(ignore) = ignore {
                    config.rule_set.enabled_rules.retain(|r| !ignore.contains(r));
                }
            }

            if print_files {
                let linter = lint::Linter::new(&config);
                for path in linter.list_files() {
                    println!("{}", path.display());
                }
                return Ok(());
            }

            let output_format = match output.as_deref() {
                Some("json") => OutputFormat::Json,
                Some("markdown") => OutputFormat::Markdown,
                Some("github") => OutputFormat::Github,
                Some("sarif") => OutputFormat::Sarif,
                _ => config.output_format.clone(),
            };
            config.output_format = output_format.clone();

            let cache_path = std::path::PathBuf::from(".lint_cache.json");
            let cache = if cache {
                match lint::cache::Cache::load(&cache_path) {
                    Ok(c) => Some(std::sync::Mutex::new(c)),
                    Err(_) => Some(std::sync::Mutex::new(lint::cache::Cache::new())),
                }
            } else {
                None
            };

            if watch {
                println!("Watching for changes... Press Ctrl+C to stop.");
                run_lint_and_print(&config, fix, &output_format, cache.as_ref(), quiet, max_warnings, output_file.as_ref(), exit_zero, statistics)?;

                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = notify::recommended_watcher(move |res| {
                    if let Ok(event) = res {
                        let _ = tx.send(event);
                    }
                })?;

                for path in &config.paths {
                    if path.is_dir() {
                        watcher.watch(path, notify::RecursiveMode::Recursive)?;
                    } else if let Some(parent) = path.parent() {
                        watcher.watch(parent, notify::RecursiveMode::NonRecursive)?;
                    }
                }

                for event in rx {
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        run_lint_and_print(&config, fix, &output_format, cache.as_ref(), quiet, max_warnings, output_file.as_ref(), exit_zero, statistics)?;
                    }
                }
            } else {
                let exit_code = run_lint_and_print(&config, fix, &output_format, cache.as_ref(), quiet, max_warnings, output_file.as_ref(), exit_zero, statistics)?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
            }

            if let Some(ref c) = cache
                && let Err(e) = c.lock().unwrap().save(&cache_path)
            {
                eprintln!("Warning: failed to save cache: {}", e);
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

fn severity_to_github_command(severity: lint::Severity) -> &'static str {
    match severity {
        lint::Severity::Error => "error",
        lint::Severity::Warning => "warning",
        lint::Severity::Info => "notice",
    }
}

fn escape_github_command(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn render_github(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    for result in results {
        let file = result.file_path.display().to_string();
        let file = escape_github_command(&file);
        for msg in &result.messages {
            let level = severity_to_github_command(msg.severity);
            out.push_str(&format!(
                "::{level} file={file},line={line},col={col},title={title}::{message}\n",
                level = level,
                file = file,
                line = msg.line,
                col = msg.column,
                title = escape_github_command(&msg.rule),
                message = escape_github_command(&msg.message),
            ));
        }
    }
    out
}

fn render_sarif(results: &[lint::LintResult]) -> String {
    use serde_json::json;

    let mut rules_seen = std::collections::HashSet::new();
    let mut sarif_results = Vec::new();

    for result in results {
        for msg in &result.messages {
            let level = match msg.severity {
                lint::Severity::Error => "error",
                lint::Severity::Warning => "warning",
                lint::Severity::Info => "note",
            };

            rules_seen.insert(msg.rule.clone());

            sarif_results.push(json!({
                "ruleId": msg.rule,
                "level": level,
                "message": {
                    "text": msg.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": result.file_path.to_string_lossy()
                        },
                        "region": {
                            "startLine": msg.line,
                            "startColumn": msg.column
                        }
                    }
                }]
            }));
        }
    }

    let rules: Vec<_> = rules_seen.into_iter().map(|r| {
        json!({
            "id": r,
            "defaultConfiguration": {
                "level": "warning"
            }
        })
    }).collect();

    let doc = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "lint",
                    "informationUri": "https://github.com/yingkitw/lint",
                    "rules": rules
                }
            },
            "results": sarif_results
        }]
    });

    serde_json::to_string_pretty(&doc).unwrap()
}

fn render_results(results: &[lint::LintResult], format: OutputFormat, quiet: bool) -> String {
    match format {
        OutputFormat::Json => {
            let filtered: Vec<_> = if quiet {
                results
                    .iter()
                    .map(|r| {
                        let mut cr = r.clone();
                        cr.messages.retain(|m| m.severity == lint::Severity::Error);
                        cr
                    })
                    .collect()
            } else {
                results.to_vec()
            };
            serde_json::to_string_pretty(&filtered).unwrap()
        }
        OutputFormat::Markdown => {
            let mut out = String::new();
            for result in results {
                let relevant: Vec<_> = if quiet {
                    result.messages.iter().filter(|m| m.severity == lint::Severity::Error).collect()
                } else {
                    result.messages.iter().collect()
                };
                if !relevant.is_empty() {
                    out.push_str(&format!("# {}\n", result.file_path.display()));
                    for msg in relevant {
                        out.push_str(&format!(
                            "- **Line {}**: [{}] {} (`{}`)\n",
                            msg.line,
                            msg.severity.as_str(),
                            msg.message,
                            msg.rule
                        ));
                        if let Some(suggestion) = &msg.suggestion {
                            out.push_str(&format!("  - **Fix**: {}\n", suggestion));
                        }
                    }
                    out.push('\n');
                }
            }
            out
        }
        OutputFormat::Github => {
            if quiet {
                let filtered: Vec<_> = results
                    .iter()
                    .map(|r| {
                        let mut cr = r.clone();
                        cr.messages.retain(|m| m.severity == lint::Severity::Error);
                        cr
                    })
                    .collect();
                render_github(&filtered)
            } else {
                render_github(results)
            }
        }
        OutputFormat::Sarif => render_sarif(results),
        OutputFormat::Text => {
            let mut out = String::new();
            let mut total_errors = 0;
            let mut total_warnings = 0;
            let mut total_infos = 0;

            for result in results {
                let relevant: Vec<_> = if quiet {
                    result.messages.iter().filter(|m| m.severity == lint::Severity::Error).collect()
                } else {
                    result.messages.iter().collect()
                };

                if relevant.is_empty() {
                    if !quiet && result.messages.is_empty() {
                        out.push_str(&format!("{}: No issues found\n", result.file_path.display()));
                    }
                } else {
                    out.push_str(&format!("{}\n", format!("{}", result.file_path.display()).bold()));
                    for msg in relevant {
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

                        out.push_str(&format!(
                            "  {}:{} {}: {}\n",
                            msg.line,
                            msg.column,
                            prefix.color(color),
                            msg.message
                        ));

                        let lines: Vec<&str> = result.file_content.lines().collect();
                        if let Some(source_line) = lines.get(msg.line.saturating_sub(1)) {
                            out.push_str(&format!("    | {}\n", source_line));
                            let caret = "    |".to_string() + &" ".repeat(msg.column.saturating_sub(1)) + "^";
                            out.push_str(&format!("{}\n", caret.dimmed()));
                        }

                        if let Some(suggestion) = &msg.suggestion {
                            out.push_str(&format!("    {} help: {}\n", "→".dimmed(), suggestion.dimmed()));
                        }
                    }
                    out.push('\n');
                }
            }

            if !quiet {
                let summary = format!(
                    "Summary: {} errors, {} warnings, {} infos",
                    total_errors, total_warnings, total_infos
                );
                out.push_str(&format!("{}\n", summary.bold()));
            }
            out
        }
    }
}

fn print_results(results: &[lint::LintResult], format: OutputFormat, quiet: bool) {
    print!("{}", render_results(results, format, quiet));
}

fn list_rules() {
    println!("Available rules (generic, apply to all files):");
    println!("  [style]        line-length          - Lines exceeding max length (fix: break line)");
    println!("  [style]        trailing-whitespace   - Trailing spaces/tabs (fix: remove)");
    println!("  [correctness]  no-todo              - TODO/FIXME comments (fix: address or create issue)");
    println!();
    println!("Language-specific rules (auto-applied by file extension):");
    println!("  JS/TS:    no-console-log, no-var");
    println!("  Python:   no-print, python-style");
    println!("  Rust:     no-unwrap, no-expect");
    println!("  Java:     java-style (PascalCase, no System.out)");
    println!("  Go:       go-style");
    println!("  Ruby:     no-puts, ruby-style");
    println!("  PHP:      no-echo");
    println!("  Swift:    no-swift-print");
    println!("  Kotlin:   kotlin-style");
    println!("  Dart:     no-dart-print");
    println!("  C#:       no-csharp-console, csharp-style");
    println!("  Shell:    shell-echo-quote");
    println!("  SQL:      sql-no-select-star");
    println!("  Lua:      no-lua-print");
    println!("  Scala:    no-scala-println");
    println!("  R:        no-r-print");
    println!("  Zig:      no-zig-debug-print");
    println!("  HTML:     html-no-inline-style, html-img-alt");
    println!("  CSS:      css-avoid-important");
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
            "output_format": "Json",
            "per_file_ignores": {},
            "severity_overrides": {}
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
                watch,
                cache,
                quiet,
                max_warnings,
                color,
                select,
                ignore,
                print_files,
                output_file,
                stdin,
                stdin_filename,
                exit_zero,
                statistics,
            } => {
                assert_eq!(paths, vec![PathBuf::from("src/")]);
                assert_eq!(output, Some("json".to_string()));
                assert_eq!(max_line_length, None);
                assert_eq!(rules, None);
                assert!(!fix);
                assert!(!watch);
                assert!(!cache);
                assert!(!quiet);
                assert_eq!(max_warnings, None);
                assert_eq!(color, None);
                assert_eq!(select, None);
                assert_eq!(ignore, None);
                assert!(!print_files);
                assert_eq!(output_file, None);
                assert!(!stdin);
                assert_eq!(stdin_filename, None);
                assert!(!exit_zero);
                assert!(!statistics);
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

    #[test]
    fn test_expand_globs_no_pattern() {
        let paths = vec![PathBuf::from("src/main.rs"), PathBuf::from("lib.rs")];
        let expanded = expand_globs(paths.clone());
        assert_eq!(expanded, paths);
    }

    #[test]
    fn test_expand_globs_star_pattern() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path();
        std::fs::write(root.join("a.rs"), "")?;
        std::fs::write(root.join("b.rs"), "")?;
        std::fs::write(root.join("c.js"), "")?;

        let pattern = root.join("*.rs");
        let expanded = expand_globs(vec![pattern]);
        assert_eq!(expanded.len(), 2);

        Ok(())
    }

    #[test]
    fn test_cli_watch_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--watch"]);
        match cli.command {
            Commands::Lint { watch, .. } => {
                assert!(watch);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_github_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "github"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("github".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_quiet_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--quiet"]);
        match cli.command {
            Commands::Lint { quiet, .. } => {
                assert!(quiet);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_max_warnings_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--max-warnings", "10"]);
        match cli.command {
            Commands::Lint { max_warnings, .. } => {
                assert_eq!(max_warnings, Some(10));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_color_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--color", "never"]);
        match cli.command {
            Commands::Lint { color, .. } => {
                assert_eq!(color, Some("never".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_select_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--select", "no-todo"]);
        match cli.command {
            Commands::Lint { select, .. } => {
                assert_eq!(select, Some(vec!["no-todo".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_ignore_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--ignore", "line-length"]);
        match cli.command {
            Commands::Lint { ignore, .. } => {
                assert_eq!(ignore, Some(vec!["line-length".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_print_files_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--print-files"]);
        match cli.command {
            Commands::Lint { print_files, .. } => {
                assert!(print_files);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_file_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output-file", "results.json"]);
        match cli.command {
            Commands::Lint { output_file, .. } => {
                assert_eq!(output_file, Some(PathBuf::from("results.json")));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_sarif_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "sarif"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("sarif".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_exit_zero_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--exit-zero"]);
        match cli.command {
            Commands::Lint { exit_zero, .. } => {
                assert!(exit_zero);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_statistics_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--statistics"]);
        match cli.command {
            Commands::Lint { statistics, .. } => {
                assert!(statistics);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_stdin_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "--stdin"]);
        match cli.command {
            Commands::Lint { stdin, .. } => {
                assert!(stdin);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_stdin_filename_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "--stdin", "--stdin-filename", "test.rs"]);
        match cli.command {
            Commands::Lint { stdin, stdin_filename, .. } => {
                assert!(stdin);
                assert_eq!(stdin_filename, Some("test.rs".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_severity_to_github_command() {
        assert_eq!(severity_to_github_command(lint::Severity::Error), "error");
        assert_eq!(
            severity_to_github_command(lint::Severity::Warning),
            "warning"
        );
        assert_eq!(severity_to_github_command(lint::Severity::Info), "notice");
    }

    #[test]
    fn test_escape_github_command() {
        assert_eq!(escape_github_command("plain text"), "plain text");
        assert_eq!(escape_github_command("100% done"), "100%25 done");
        assert_eq!(escape_github_command("a\nb"), "a%0Ab");
        assert_eq!(escape_github_command("a\rb"), "a%0Db");
        assert_eq!(escape_github_command("a\r\nb%"), "a%0D%0Ab%25");
    }

    fn sample_result() -> lint::LintResult {
        let mut result = lint::LintResult::new(
            PathBuf::from("src/main.rs"),
            "let x = 5;   \n".to_string(),
        );
        result.add_message(lint::LintMessage::new(
            1,
            10,
            lint::Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            None,
        ));
        result.add_message(lint::LintMessage::new(
            2,
            1,
            lint::Severity::Error,
            "Line too long".to_string(),
            "line-length".to_string(),
            None,
        ));
        result.add_message(lint::LintMessage::new(
            3,
            1,
            lint::Severity::Info,
            "Found TODO".to_string(),
            "no-todo".to_string(),
            None,
        ));
        result
    }

    #[test]
    fn test_render_github_formats_each_severity() {
        let results = vec![sample_result()];
        let out = render_github(&results);

        assert!(
            out.contains("::warning file=src/main.rs,line=1,col=10,title=trailing-whitespace::Trailing whitespace\n"),
            "warning line missing/malformed: {out}"
        );
        assert!(
            out.contains("::error file=src/main.rs,line=2,col=1,title=line-length::Line too long\n"),
            "error line missing/malformed: {out}"
        );
        assert!(
            out.contains("::notice file=src/main.rs,line=3,col=1,title=no-todo::Found TODO\n"),
            "notice line missing/malformed: {out}"
        );
    }

    #[test]
    fn test_render_github_escapes_message() {
        let mut result =
            lint::LintResult::new(PathBuf::from("src/a.rs"), "x".to_string());
        result.add_message(lint::LintMessage::new(
            1,
            1,
            lint::Severity::Error,
            "Unexpected 100% usage\nnewline here".to_string(),
            "r1".to_string(),
            None,
        ));
        let out = render_github(&[result]);
        assert!(
            out.contains("::error file=src/a.rs,line=1,col=1,title=r1::Unexpected 100%25 usage%0Anewline here\n"),
            "message not escaped: {out}"
        );
    }

    #[test]
    fn test_render_github_empty_results() {
        assert_eq!(render_github(&[]), "");
        let clean = lint::LintResult::new(PathBuf::from("clean.rs"), "ok".to_string());
        assert_eq!(render_github(&[clean]), "");
    }

    #[test]
    fn test_render_sarif_produces_valid_json() {
        let mut result = lint::LintResult::new(PathBuf::from("src/main.rs"), "let x = 5;   \n".to_string());
        result.add_message(lint::LintMessage::new(
            1,
            10,
            lint::Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            Some("Remove trailing spaces".to_string()),
        ));
        let out = render_sarif(&[result]);
        assert!(out.contains("\"$schema\""));
        assert!(out.contains("\"version\": \"2.1.0\""));
        assert!(out.contains("\"ruleId\": \"trailing-whitespace\""));
        assert!(out.contains("\"level\": \"warning\""));
        assert!(out.contains("\"startLine\": 1"));
        assert!(out.contains("\"startColumn\": 10"));
        assert!(out.contains("\"uri\": \"src/main.rs\""));
    }

    #[test]
    fn test_merge_configs_local_overrides_base() {
        let base = ConfigBuilder::new()
            .max_line_length(Some(80))
            .enabled_rules(vec!["line-length".to_string()])
            .output_format(lint::OutputFormat::Json)
            .build();
        let local = ConfigBuilder::new()
            .max_line_length(Some(120))
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let merged = merge_configs(base, local);
        assert_eq!(merged.max_line_length, Some(120));
        assert_eq!(merged.rule_set.enabled_rules, vec!["trailing-whitespace"]);
        assert_eq!(merged.output_format, lint::OutputFormat::Text);
    }

    #[test]
    fn test_merge_configs_base_preserved_when_local_does_not_override() {
        let base = ConfigBuilder::new()
            .max_line_length(Some(80))
            .enabled_rules(vec!["line-length".to_string()])
            .build();
        let local = ConfigBuilder::new()
            .max_line_length(None)
            .enabled_rules(Vec::new())
            .build();

        let merged = merge_configs(base, local);
        assert_eq!(merged.max_line_length, Some(80));
        assert_eq!(merged.rule_set.enabled_rules, vec!["line-length"]);
    }

    #[test]
    fn test_merge_configs_ignore_patterns_concatenated() {
        let base = ConfigBuilder::new()
            .ignore_patterns(vec!["node_modules".to_string(), "dist".to_string()])
            .build();
        let local = ConfigBuilder::new()
            .ignore_patterns(vec!["vendor".to_string(), "node_modules".to_string()])
            .build();

        let merged = merge_configs(base, local);
        assert!(merged.ignore_patterns.contains(&"node_modules".to_string()));
        assert!(merged.ignore_patterns.contains(&"dist".to_string()));
        assert!(merged.ignore_patterns.contains(&"vendor".to_string()));
        assert_eq!(merged.ignore_patterns.len(), 3);
    }

    #[test]
    fn test_merge_configs_per_file_ignores_merged() {
        use std::collections::HashMap;
        let mut base_ignores = HashMap::new();
        base_ignores.insert("tests/**/*.rs".to_string(), vec!["line-length".to_string()]);
        let mut local_ignores = HashMap::new();
        local_ignores.insert("tests/**/*.rs".to_string(), vec!["no-todo".to_string()]);
        local_ignores.insert("src/**/*.rs".to_string(), vec!["line-length".to_string()]);

        let base = ConfigBuilder::new().per_file_ignores(base_ignores).build();
        let local = ConfigBuilder::new().per_file_ignores(local_ignores).build();

        let merged = merge_configs(base, local);
        assert_eq!(merged.per_file_ignores.get("tests/**/*.rs"), Some(&vec!["no-todo".to_string()]));
        assert_eq!(merged.per_file_ignores.get("src/**/*.rs"), Some(&vec!["line-length".to_string()]));
    }

    #[test]
    fn test_load_config_file_with_extends() -> anyhow::Result<()> {
        use std::io::Write;
        let base_config = ConfigBuilder::new()
            .max_line_length(Some(80))
            .enabled_rules(vec!["line-length".to_string()])
            .build();
        let mut base_file = tempfile::NamedTempFile::new()?;
        base_file.write_all(serde_json::to_string(&base_config)?.as_bytes())?;
        base_file.flush()?;

        let mut local_file = tempfile::NamedTempFile::new()?;
        let local_config = ConfigBuilder::new()
            .max_line_length(Some(120))
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .extends(Some(base_file.path().to_string_lossy().to_string()))
            .build();
        local_file.write_all(serde_json::to_string(&local_config)?.as_bytes())?;
        local_file.flush()?;

        let config = load_config_file(&local_file.path().to_path_buf())?;
        assert_eq!(config.max_line_length, Some(120));
        assert_eq!(config.rule_set.enabled_rules, vec!["trailing-whitespace"]);

        Ok(())
    }
}
