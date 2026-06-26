use clap::{Parser, Subcommand};
use colored::Colorize;
use lint::config::CacheStrategy;
use lint::{Config, ConfigBuilder, OutputFormat};
use notify::Watcher;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
#[allow(clippy::large_enum_variant)]
enum Commands {
    #[command(alias = "check")]
    Lint {
        #[arg(value_name = "PATHS")]
        paths: Vec<PathBuf>,

        #[arg(short, long, visible_alias = "output-format", value_name = "FORMAT")]
        output: Option<String>,

        #[arg(short, long, visible_alias = "line-length", value_name = "LENGTH")]
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

        #[arg(long, visible_alias = "stdin-file-path", value_name = "FILENAME")]
        stdin_filename: Option<String>,

        #[arg(long)]
        exit_zero: bool,

        #[arg(long)]
        statistics: bool,

        #[arg(long)]
        show_fixes: bool,

        #[arg(long, value_name = "PATTERN")]
        exclude: Option<Vec<String>>,

        #[arg(long)]
        show_settings: bool,

        #[arg(long, value_name = "PATH")]
        cache_location: Option<PathBuf>,

        #[arg(long)]
        diff: bool,

        #[arg(long)]
        fix_only: bool,

        #[arg(long)]
        add_noqa: bool,

        #[arg(long)]
        ignore_suppressions: bool,

        #[arg(long)]
        select_all: bool,

        #[arg(long)]
        no_ignore: bool,

        #[arg(long, value_name = "STRATEGY")]
        cache_strategy: Option<String>,

        #[arg(long)]
        no_error_on_unmatched_pattern: bool,

        #[arg(long)]
        deny_warnings: bool,

        #[arg(long)]
        exit_non_zero_on_fix: bool,

        #[arg(long)]
        no_cache: bool,

        #[arg(long)]
        no_config: bool,

        #[arg(long)]
        check: bool,

        #[arg(long, value_name = "N")]
        max_diagnostics: Option<usize>,

        #[arg(long, value_name = "PATH")]
        baseline: Option<PathBuf>,

        #[arg(long)]
        force_exclude: bool,

        #[arg(long, value_name = "EXT", use_value_delimiter = true)]
        ext: Option<Vec<String>>,

        #[arg(long)]
        verbose: bool,

        #[arg(long)]
        unsafe_fixes: bool,

        #[arg(long, value_name = "RULES", use_value_delimiter = true)]
        fixable: Option<Vec<String>>,

        #[arg(long, value_name = "RULES", use_value_delimiter = true)]
        unfixable: Option<Vec<String>>,

        #[arg(long)]
        preview: bool,

        #[arg(long)]
        no_show_source: bool,

        #[arg(long)]
        no_gitignore: bool,

        #[arg(long)]
        changed: bool,

        #[arg(long)]
        staged: bool,

        #[arg(long, value_name = "PLUGIN", use_value_delimiter = true)]
        plugin: Option<Vec<String>>,

        #[arg(long)]
        progress: bool,

        #[arg(long)]
        validate_config: bool,

        #[arg(long, value_name = "DEPTH")]
        max_nesting_depth: Option<usize>,

        #[arg(long, value_name = "LINES")]
        max_function_lines: Option<usize>,
    },
    ListRules,
    Version,
    Explain {
        rule: String,
    },
    Init,
}

struct RunOptions<'a> {
    fix: bool,
    fix_only: bool,
    diff: bool,
    check: bool,
    quiet: bool,
    verbose: bool,
    statistics: bool,
    show_fixes: bool,
    add_noqa: bool,
    exit_zero: bool,
    exit_non_zero_on_fix: bool,
    deny_warnings: bool,
    unsafe_fixes: bool,
    max_warnings: Option<usize>,
    max_diagnostics: Option<usize>,
    output_format: &'a lint::OutputFormat,
    output_file: Option<&'a PathBuf>,
    baseline: Option<&'a PathBuf>,
    fixable: Option<&'a [String]>,
    unfixable: Option<&'a [String]>,
    progress: bool,
    per_dir_configs: std::collections::HashMap<PathBuf, lint::Config>,
}

impl<'a> RunOptions<'a> {
    fn effective_fix(&self) -> bool {
        self.fix || self.fix_only
    }
}

fn run_lint_and_print(
    config: &lint::Config,
    cache: Option<Arc<Mutex<lint::cache::Cache>>>,
    opts: RunOptions,
) -> anyhow::Result<i32> {
    let progress_bar = if opts.progress {
        let linter =
            lint::Linter::new_with_per_dir_configs(config, None, opts.per_dir_configs.clone());
        let files = linter.list_files();
        let pb = indicatif::ProgressBar::new(files.len() as u64);
        pb.set_style(
            indicatif::ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {msg} {pos}/{len} ({per_sec})",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        pb.set_message("Linting files".to_string());
        Some(pb)
    } else {
        None
    };

    let linter =
        lint::Linter::new_with_per_dir_configs(config, cache.clone(), opts.per_dir_configs.clone());
    let mut results = linter.run()?;

    if let Some(pb) = progress_bar {
        pb.finish_with_message("Linting complete".to_string());
    }

    let effective_fix = opts.effective_fix();

    // When --fix is used without --unsafe-fixes, filter out unsafe fixes
    if effective_fix && !opts.unsafe_fixes {
        for result in &mut results {
            for msg in &mut result.messages {
                if let Some(ref fix) = msg.fix
                    && !fix.is_safe
                {
                    msg.fix = None;
                }
            }
        }
    }

    // Apply --fixable and --unfixable filtering
    if let Some(rules) = opts.fixable {
        let allowed: std::collections::HashSet<&String> = rules.iter().collect();
        for result in &mut results {
            for msg in &mut result.messages {
                if msg.fix.is_some() && !allowed.contains(&msg.rule) {
                    msg.fix = None;
                }
            }
        }
    }
    if let Some(rules) = opts.unfixable {
        let denied: std::collections::HashSet<&String> = rules.iter().collect();
        for result in &mut results {
            for msg in &mut result.messages {
                if msg.fix.is_some() && denied.contains(&msg.rule) {
                    msg.fix = None;
                }
            }
        }
    }

    let mut fixed_files = Vec::new();
    if effective_fix || opts.diff || opts.check {
        let mut fixed_count = 0;
        for result in &mut results {
            let original = if opts.diff {
                result.file_content.clone()
            } else {
                String::new()
            };
            if result.apply_fixes() {
                fixed_count += 1;
                fixed_files.push(result.file_path.display().to_string());
                if opts.diff {
                    let original_lines: Vec<&str> = original.lines().collect();
                    let fixed_lines: Vec<&str> = result.file_content.lines().collect();
                    println!("--- {}", result.file_path.display());
                    println!("+++ {}", result.file_path.display());
                    let max_lines = original_lines.len().max(fixed_lines.len());
                    for i in 0..max_lines {
                        let orig = original_lines.get(i);
                        let fixed = fixed_lines.get(i);
                        if orig != fixed {
                            if let Some(line) = orig {
                                println!("- {}", line);
                            }
                            if let Some(line) = fixed {
                                println!("+ {}", line);
                            }
                        }
                    }
                } else if !opts.check {
                    std::fs::write(&result.file_path, &result.file_content)?;
                }
            }
        }
        if fixed_count > 0 && !opts.diff && !opts.check {
            println!("Fixed {} file(s)", fixed_count);
        } else if opts.check && fixed_count > 0 {
            println!("Would fix {} file(s)", fixed_count);
        }
    }

    if opts.add_noqa {
        let mut noqa_count = 0;
        for result in &mut results {
            if result.messages.is_empty() {
                continue;
            }
            let mut lines: Vec<String> =
                result.file_content.lines().map(|s| s.to_string()).collect();
            let mut modified = false;
            for msg in &result.messages {
                let line_idx = msg.line.saturating_sub(1);
                if let Some(line) = lines.get_mut(line_idx) {
                    let code = lint::rules::code_from_name(&msg.rule);
                    let rule_ref = if code != msg.rule { code } else { &msg.rule };
                    let directive = format!(" // lint: ignore={}", rule_ref);
                    if !line.contains(&directive) {
                        line.push_str(&directive);
                        modified = true;
                    }
                }
            }
            if modified {
                result.file_content = lines.join("\n");
                if !result.file_content.ends_with('\n') {
                    result.file_content.push('\n');
                }
                std::fs::write(&result.file_path, &result.file_content)?;
                noqa_count += 1;
            }
        }
        if noqa_count > 0 {
            println!("Added suppression comments to {} file(s)", noqa_count);
        }
    }

    if opts.verbose && !opts.quiet {
        let total_files = results.len();
        let total_messages: usize = results.iter().map(|r| r.messages.len()).sum();
        println!(
            "Linting {} file(s), found {} diagnostic(s)",
            total_files, total_messages
        );
        if cache.is_some() {
            println!("Cache: enabled");
        } else {
            println!("Cache: disabled");
        }
        if opts.fix {
            println!("Fix mode: enabled");
        }
        if opts.fix_only {
            println!("Fix-only mode: enabled");
        }
    }

    if let Some(baseline_path) = opts.baseline {
        let baseline_entries = load_baseline(baseline_path)?;
        filter_baseline(&mut results, &baseline_entries);
    }

    let hidden_diagnostics = truncate_diagnostics(&mut results, opts.max_diagnostics);

    if !opts.fix_only {
        if let Some(path) = opts.output_file {
            colored::control::set_override(false);
            let output =
                render_results(&results, opts.output_format, opts.quiet, config.show_source);
            colored::control::unset_override();
            std::fs::write(path, output)?;
        } else {
            print_results(&results, opts.output_format, opts.quiet, config.show_source);
        }
    }

    if hidden_diagnostics > 0 && !opts.quiet && !opts.fix_only {
        println!(
            "... {} additional diagnostics hidden ...",
            hidden_diagnostics
        );
    }

    if opts.show_fixes && !fixed_files.is_empty() && !opts.quiet && !opts.fix_only {
        println!("\n{}", "Fixed files:".bold());
        for file in &fixed_files {
            println!("  {}", file);
        }
    }

    if opts.statistics && !opts.quiet && !opts.fix_only {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for result in &results {
            for msg in &result.messages {
                *counts.entry(&msg.rule).or_insert(0) += 1;
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
        .map(|r| {
            r.messages
                .iter()
                .filter(|m| m.severity == lint::Severity::Warning)
                .count()
        })
        .sum();
    let max_warnings_exceeded = opts.max_warnings.is_some_and(|max| warning_count > max);
    let any_fixes_applied = !fixed_files.is_empty();

    Ok(if opts.fix_only || opts.exit_zero {
        0
    } else if has_errors
        || max_warnings_exceeded
        || (opts.deny_warnings && warning_count > 0)
        || (opts.exit_non_zero_on_fix && any_fixes_applied)
        || (opts.check && any_fixes_applied)
    {
        1
    } else {
        0
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaselineEntry {
    file: String,
    line: usize,
    rule: String,
}

fn load_baseline(path: &PathBuf) -> anyhow::Result<Vec<BaselineEntry>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read baseline file {}: {}", path.display(), e))?;
    let entries: Vec<BaselineEntry> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse baseline file {}: {}", path.display(), e))?;
    Ok(entries)
}

fn filter_baseline(results: &mut [lint::LintResult], baseline: &[BaselineEntry]) {
    for result in results.iter_mut() {
        let file_str = result.file_path.to_string_lossy();
        result.messages.retain(|msg| {
            !baseline.iter().any(|entry| {
                entry.file == file_str && entry.line == msg.line && entry.rule == msg.rule
            })
        });
    }
}

fn truncate_diagnostics(results: &mut [lint::LintResult], max_diagnostics: Option<usize>) -> usize {
    let mut hidden = 0usize;
    if let Some(max) = max_diagnostics {
        let total: usize = results.iter().map(|r| r.messages.len()).sum();
        if total > max {
            let mut remaining = max;
            for result in results.iter_mut() {
                if result.messages.len() <= remaining {
                    remaining -= result.messages.len();
                } else {
                    hidden += result.messages.len() - remaining;
                    result.messages.truncate(remaining);
                    remaining = 0;
                }
                if remaining == 0 {
                    break;
                }
            }
        }
    }
    hidden
}

fn load_config_file(path: &PathBuf) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;
    let mut config: Config = if path.extension().is_some_and(|e| e == "toml") {
        toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?
    } else {
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?
    };

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
        paths: if local.paths.is_empty() {
            base.paths
        } else {
            local.paths
        },
        ignore_patterns,
        max_line_length: local.max_line_length.or(base.max_line_length),
        rule_set: lint::config::RuleSetConfig {
            enabled_rules: if local.rule_set.enabled_rules.is_empty() {
                base.rule_set.enabled_rules
            } else {
                local.rule_set.enabled_rules
            },
            custom_rules_path: local
                .rule_set
                .custom_rules_path
                .or(base.rule_set.custom_rules_path),
        },
        output_format: local.output_format,
        per_file_ignores,
        severity_overrides,
        extends: local.extends,
        ignore_suppressions: local.ignore_suppressions || base.ignore_suppressions,
        cache_strategy: if local.cache_strategy != CacheStrategy::Metadata {
            local.cache_strategy
        } else {
            base.cache_strategy
        },
        stdin_file_path: local.stdin_file_path.or(base.stdin_file_path),
        force_exclude: local.force_exclude || base.force_exclude,
        ext: local.ext.or(base.ext),
        preview: local.preview || base.preview,
        show_source: local.show_source && base.show_source,
        no_gitignore: local.no_gitignore || base.no_gitignore,
        plugins: {
            let mut merged = base.plugins;
            for p in local.plugins {
                if !merged.contains(&p) {
                    merged.push(p);
                }
            }
            merged
        },
        max_nesting_depth: local.max_nesting_depth.or(base.max_nesting_depth),
        max_function_lines: local.max_function_lines.or(base.max_function_lines),
    }
}

/// Discovers `.lint.toml` / `.lint.json` files in subdirectories of the given
/// paths and returns a map from directory path to the merged config.
fn discover_per_dir_configs(
    paths: &[PathBuf],
    base_config: &lint::Config,
) -> std::collections::HashMap<PathBuf, lint::Config> {
    let mut per_dir = std::collections::HashMap::new();
    for path in paths {
        let dir = if path.is_dir() {
            path.clone()
        } else if let Some(parent) = path.parent() {
            parent.to_path_buf()
        } else {
            continue;
        };
        discover_in_dir(&dir, base_config, &mut per_dir);
    }
    per_dir
}

fn discover_in_dir(
    dir: &std::path::Path,
    base_config: &lint::Config,
    per_dir: &mut std::collections::HashMap<PathBuf, lint::Config>,
) {
    let walker = ignore::WalkBuilder::new(dir)
        .git_ignore(!base_config.no_gitignore)
        .build();
    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(cfg) = load_dir_config(path) {
            let merged = merge_configs(base_config.clone(), cfg);
            per_dir.insert(path.to_path_buf(), merged);
        }
    }
}

fn load_dir_config(dir: &std::path::Path) -> Option<lint::Config> {
    let json = dir.join(".lint.json");
    if json.is_file()
        && let Ok(config) = load_config_file(&json)
    {
        return Some(config);
    }
    let toml = dir.join(".lint.toml");
    if toml.is_file()
        && let Ok(config) = load_config_file(&toml)
    {
        return Some(config);
    }
    None
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
            show_fixes,
            exclude,
            show_settings,
            cache_location,
            diff,
            fix_only,
            add_noqa,
            ignore_suppressions,
            select_all,
            no_ignore,
            cache_strategy,
            no_error_on_unmatched_pattern,
            deny_warnings,
            exit_non_zero_on_fix,
            no_cache,
            no_config,
            check,
            max_diagnostics,
            baseline,
            force_exclude,
            ext,
            verbose,
            unsafe_fixes,
            fixable,
            unfixable,
            preview,
            no_show_source,
            no_gitignore,
            changed,
            staged,
            plugin,
            progress,
            validate_config,
            max_nesting_depth,
            max_function_lines,
        } => {
            let _ = no_error_on_unmatched_pattern;
            match color.as_deref() {
                Some("never") | Some("no") => colored::control::set_override(false),
                Some("always") | Some("yes") => colored::control::set_override(true),
                _ => {}
            }
            let mut config = if no_config {
                ConfigBuilder::new().build()
            } else if let Some(config_path) = &cli.config {
                load_config_file(config_path)?
            } else if let Some(config_path) = find_config_file() {
                load_config_file(&config_path)?
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
                let temp_path = std::env::temp_dir().join(format!(
                    "lint_stdin_{}.{}",
                    std::process::id(),
                    suffix
                ));
                std::fs::write(&temp_path, content)?;
                Some(temp_path)
            } else {
                None
            };

            if let Some(ref temp_path) = stdin_temp_path {
                config.paths = vec![temp_path.clone()];
                config.stdin_file_path = stdin_filename.clone();
            } else if paths.is_empty() {
                config.paths = vec![PathBuf::from(".")];
            } else {
                config.paths = expand_globs(paths);
            }

            if changed || staged {
                let git_files = git_changed_files(changed, staged)?;
                if git_files.is_empty() {
                    config.paths.clear();
                } else {
                    let requested: std::collections::HashSet<PathBuf> =
                        config.paths.iter().cloned().collect();
                    let mut filtered = Vec::new();
                    for gf in &git_files {
                        let gf_canon = std::path::Path::new(gf).canonicalize().ok();
                        for req in &requested {
                            if req == gf {
                                filtered.push(gf.clone());
                                break;
                            }
                            if req.is_dir()
                                && let Some(ref c) = gf_canon
                                && let Ok(req_canon) = req.canonicalize()
                                && c.starts_with(&req_canon)
                            {
                                filtered.push(gf.clone());
                                break;
                            }
                        }
                    }
                    config.paths = filtered;
                }
            }

            if let Some(exclude_patterns) = exclude {
                config.ignore_patterns.extend(exclude_patterns);
            }

            if no_ignore {
                config.ignore_patterns.clear();
            }

            if let Some(strategy) = cache_strategy {
                config.cache_strategy = match strategy.as_str() {
                    "content" => CacheStrategy::Content,
                    _ => CacheStrategy::Metadata,
                };
            }

            config.ignore_suppressions = ignore_suppressions;
            config.force_exclude = force_exclude;
            config.ext = ext;
            config.preview = preview;
            config.show_source = !no_show_source;
            config.no_gitignore = no_gitignore;
            config.plugins = plugin.unwrap_or_default();
            config.max_nesting_depth = max_nesting_depth;
            config.max_function_lines = max_function_lines;

            if let Some(length) = max_line_length {
                config.max_line_length = Some(length);
            }

            if let Some(rules) = rules {
                config.rule_set.enabled_rules = expand_rule_names(rules);
            } else {
                if let Some(select) = select {
                    for rule in expand_rule_names(select) {
                        if !config.rule_set.enabled_rules.contains(&rule) {
                            config.rule_set.enabled_rules.push(rule);
                        }
                    }
                }
                if let Some(ignore) = ignore {
                    let expanded_ignore = expand_rule_names(ignore);
                    config
                        .rule_set
                        .enabled_rules
                        .retain(|r| !expanded_ignore.contains(r));
                }
            }

            if select_all {
                for rule in expand_rule_names(vec!["ALL".to_string()]) {
                    if !config.rule_set.enabled_rules.contains(&rule) {
                        config.rule_set.enabled_rules.push(rule);
                    }
                }
            }

            if validate_config {
                let errors = validate_config_errors(&config);
                if errors.is_empty() {
                    println!("Config is valid.");
                    return Ok(());
                } else {
                    for err in &errors {
                        eprintln!("Config error: {}", err);
                    }
                    std::process::exit(1);
                }
            }

            if show_settings {
                println!("{}", serde_json::to_string_pretty(&config)?);
                return Ok(());
            }

            if print_files {
                let linter = lint::Linter::new(&config);
                for path in linter.list_files() {
                    println!("{}", path.display());
                }
                return Ok(());
            }

            let per_dir_configs = discover_per_dir_configs(&config.paths, &config);

            let output_format = match output.as_deref() {
                Some("json") => OutputFormat::Json,
                Some("markdown") => OutputFormat::Markdown,
                Some("github") => OutputFormat::Github,
                Some("sarif") => OutputFormat::Sarif,
                Some("junit") => OutputFormat::Junit,
                Some("concise") => OutputFormat::Concise,
                Some("gitlab") => OutputFormat::Gitlab,
                Some("grouped") => OutputFormat::Grouped,
                _ => config.output_format.clone(),
            };
            config.output_format = output_format.clone();

            let cache_path =
                cache_location.unwrap_or_else(|| std::path::PathBuf::from(".lint_cache.json"));
            let cache: Option<std::sync::Arc<std::sync::Mutex<lint::cache::Cache>>> = if no_cache {
                None
            } else if cache {
                match lint::cache::Cache::load(&cache_path) {
                    Ok(c) => Some(std::sync::Arc::new(std::sync::Mutex::new(c))),
                    Err(_) => Some(std::sync::Arc::new(std::sync::Mutex::new(
                        lint::cache::Cache::new(),
                    ))),
                }
            } else {
                None
            };

            if watch {
                println!("Watching for changes... Press Ctrl+C to stop.");
                run_lint_and_print(
                    &config,
                    cache.clone(),
                    RunOptions {
                        fix,
                        fix_only,
                        diff,
                        check,
                        quiet,
                        verbose,
                        statistics,
                        show_fixes,
                        add_noqa,
                        exit_zero,
                        exit_non_zero_on_fix,
                        deny_warnings,
                        unsafe_fixes,
                        max_warnings,
                        max_diagnostics,
                        output_format: &output_format,
                        output_file: output_file.as_ref(),
                        baseline: baseline.as_ref(),
                        fixable: fixable.as_deref(),
                        unfixable: unfixable.as_deref(),
                        progress,
                        per_dir_configs: per_dir_configs.clone(),
                    },
                )?;

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
                        run_lint_and_print(
                            &config,
                            cache.clone(),
                            RunOptions {
                                fix,
                                fix_only,
                                diff,
                                check,
                                quiet,
                                verbose,
                                statistics,
                                show_fixes,
                                add_noqa,
                                exit_zero,
                                exit_non_zero_on_fix,
                                deny_warnings,
                                unsafe_fixes,
                                max_warnings,
                                max_diagnostics,
                                output_format: &output_format,
                                output_file: output_file.as_ref(),
                                baseline: baseline.as_ref(),
                                fixable: fixable.as_deref(),
                                unfixable: unfixable.as_deref(),
                                progress,
                                per_dir_configs: per_dir_configs.clone(),
                            },
                        )?;
                    }
                }
            } else {
                let exit_code = run_lint_and_print(
                    &config,
                    cache.clone(),
                    RunOptions {
                        fix,
                        fix_only,
                        diff,
                        check,
                        quiet,
                        verbose,
                        statistics,
                        show_fixes,
                        add_noqa,
                        exit_zero,
                        exit_non_zero_on_fix,
                        deny_warnings,
                        unsafe_fixes,
                        max_warnings,
                        max_diagnostics,
                        output_format: &output_format,
                        output_file: output_file.as_ref(),
                        baseline: baseline.as_ref(),
                        fixable: fixable.as_deref(),
                        unfixable: unfixable.as_deref(),
                        progress,
                        per_dir_configs: per_dir_configs.clone(),
                    },
                )?;
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
        Commands::Explain { rule } => {
            explain_rule(&rule)?;
        }
        Commands::Init => {
            init_config()?;
        }
    }

    Ok(())
}

/// Validates a loaded config and returns a list of human-readable errors.
fn validate_config_errors(config: &lint::Config) -> Vec<String> {
    let mut errors = Vec::new();
    let known_rules: std::collections::HashSet<String> =
        lint::rules::known_rules().into_iter().collect();
    let known_plugins: std::collections::HashSet<String> =
        lint::rules::known_plugins().into_iter().collect();

    // Check enabled rules
    for rule in &config.rule_set.enabled_rules {
        let resolved = lint::rules::name_from_code(rule);
        if !known_rules.contains(rule) && !known_rules.contains(resolved) {
            errors.push(format!("Unknown rule '{}' in enabled_rules", rule));
        }
    }

    // Check severity_overrides keys
    for rule in config.severity_overrides.keys() {
        let resolved = lint::rules::name_from_code(rule);
        if !known_rules.contains(rule) && !known_rules.contains(resolved) {
            errors.push(format!("Unknown rule '{}' in severity_overrides", rule));
        }
    }

    // Check plugins
    for plugin in &config.plugins {
        if !known_plugins.contains(plugin) {
            errors.push(format!("Unknown plugin '{}'", plugin));
        }
    }

    errors
}

#[cfg(test)]
fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_changed_files(changed: bool, staged: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if changed {
        let output = std::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD"])
            .output()?;
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if !line.is_empty() {
                    files.push(PathBuf::from(line));
                }
            }
        }
    }

    if staged {
        let output = std::process::Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output()?;
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if !line.is_empty() && !files.contains(&PathBuf::from(line)) {
                    files.push(PathBuf::from(line));
                }
            }
        }
    }

    Ok(files)
}

/// Expands any category names (e.g., "style", "security") or "ALL" into their
/// constituent rule names. Also resolves rule codes (e.g., "W001") to names.
/// Rule names that are not categories pass through.
fn expand_rule_names(names: Vec<String>) -> Vec<String> {
    let mut expanded = Vec::new();
    for name in names {
        if name.eq_ignore_ascii_case("ALL") {
            for rule in lint::rules::known_rules() {
                if !expanded.contains(&rule) {
                    expanded.push(rule);
                }
            }
            for rule in lint::language_rules::LanguageRuleSet::known_rules() {
                if !expanded.contains(&rule) {
                    expanded.push(rule);
                }
            }
        } else if lint::rules::is_category(&name) {
            for rule in lint::rules::category_rules(&name) {
                if !expanded.contains(&rule) {
                    expanded.push(rule);
                }
            }
        } else {
            let resolved = lint::rules::name_from_code(&name).to_string();
            if resolved != name && !expanded.contains(&resolved) {
                expanded.push(resolved);
            } else if !expanded.contains(&name) {
                expanded.push(name);
            }
        }
    }
    expanded
}

fn explain_rule(rule_name: &str) -> anyhow::Result<()> {
    let config = lint::ConfigBuilder::new().build();
    let linter = lint::Linter::new(&config);

    let resolved_name = lint::rules::name_from_code(rule_name);
    let lookup_name = if resolved_name != rule_name {
        resolved_name
    } else {
        rule_name
    };

    let found = linter
        .rule_set()
        .get_rules()
        .iter()
        .find(|r| r.name() == lookup_name)
        .map(|r| {
            (
                r.name(),
                r.category(),
                r.description(),
                r.has_fix(),
                r.code(),
            )
        })
        .or_else(|| {
            linter
                .language_rule_set()
                .get_rules()
                .iter()
                .find(|r| r.name() == lookup_name)
                .map(|r| {
                    (
                        r.name(),
                        r.category(),
                        r.description(),
                        r.has_fix(),
                        r.code(),
                    )
                })
        });

    if let Some((name, category, description, has_fix, code)) = found {
        println!("Rule: {}", name);
        println!("Code: {}", code);
        println!("Category: {}", category);
        println!("Description: {}", description);
        println!("Auto-fix: {}", if has_fix { "yes" } else { "no" });
        return Ok(());
    }

    anyhow::bail!("Unknown rule: {}", rule_name);
}

fn init_config() -> anyhow::Result<()> {
    let config_path = PathBuf::from(".lint.json");
    if config_path.exists() {
        println!(".lint.json already exists.");
        return Ok(());
    }

    let default_config = serde_json::json!({
        "paths": ["src"],
        "ignore_patterns": ["node_modules", "target", ".git"],
        "max_line_length": 100,
        "rule_set": {
            "enabled_rules": [
                "line-length",
                "trailing-whitespace",
                "final-newline",
                "no-mixed-line-endings"
            ],
            "custom_rules_path": null
        },
        "output_format": "Text",
        "per_file_ignores": {},
        "severity_overrides": {},
        "ignore_suppressions": false
    });

    std::fs::write(&config_path, serde_json::to_string_pretty(&default_config)?)?;
    println!("Created .lint.json");
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
            let code = lint::rules::code_from_name(&msg.rule);
            let title = if code != msg.rule {
                format!("{} ({})", code, msg.rule)
            } else {
                msg.rule.clone()
            };
            out.push_str(&format!(
                "::{level} file={file},line={line},col={col},title={title}::{message}\n",
                level = level,
                file = file,
                line = msg.line,
                col = msg.column,
                title = escape_github_command(&title),
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

            let code = lint::rules::code_from_name(&msg.rule);
            rules_seen.insert(code.to_string());

            sarif_results.push(json!({
                "ruleId": code,
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

    let rules: Vec<_> = rules_seen
        .into_iter()
        .map(|code| {
            let name = lint::rules::name_from_code(&code);
            let short_desc = if name != code {
                format!("{} ({})", code, name)
            } else {
                code.clone()
            };
            json!({
                "id": code,
                "shortDescription": {
                    "text": short_desc
                },
                "defaultConfiguration": {
                    "level": "warning"
                }
            })
        })
        .collect();

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

fn render_junit(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<testsuites>\n");

    let total_tests = results.len();
    let total_failures: usize = results.iter().map(|r| r.messages.len()).sum();

    out.push_str(&format!(
        "  <testsuite name=\"lint\" tests=\"{}\" failures=\"{}\" errors=\"0\">\n",
        total_tests, total_failures
    ));

    for result in results {
        let file = result.file_path.to_string_lossy();
        out.push_str(&format!(
            "    <testcase name=\"{}\" classname=\"{}\" />\n",
            escape_xml(&file),
            escape_xml(&file)
        ));
        for msg in &result.messages {
            let code = lint::rules::code_from_name(&msg.rule);
            out.push_str(&format!(
                "    <testcase name=\"{}\" classname=\"{}\">\n",
                escape_xml(&msg.rule),
                escape_xml(&file)
            ));
            out.push_str(&format!(
                "      <failure message=\"{}\" type=\"{}\">\n",
                escape_xml(&msg.message),
                escape_xml(code)
            ));
            out.push_str(&format!(
                "        {}:{}:{} - {} [{}]\n",
                escape_xml(&file),
                msg.line,
                msg.column,
                escape_xml(&msg.message),
                escape_xml(code)
            ));
            out.push_str("      </failure>\n");
            out.push_str("    </testcase>\n");
        }
    }

    out.push_str("  </testsuite>\n");
    out.push_str("</testsuites>\n");
    out
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn render_gitlab(results: &[lint::LintResult]) -> String {
    use serde_json::json;

    let mut issues = Vec::new();
    for result in results {
        for msg in &result.messages {
            let severity = match msg.severity {
                lint::Severity::Error => "blocker",
                lint::Severity::Warning => "major",
                lint::Severity::Info => "info",
            };
            let code = lint::rules::code_from_name(&msg.rule);
            issues.push(json!({
                "description": msg.message,
                "check_name": code,
                "fingerprint": format!("{}:{}:{}", result.file_path.display(), msg.line, msg.column),
                "severity": severity,
                "location": {
                    "path": result.file_path.to_string_lossy(),
                    "lines": {
                        "begin": msg.line,
                        "end": msg.line
                    }
                }
            }));
        }
    }
    serde_json::to_string_pretty(&issues).unwrap()
}

fn render_concise(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    for result in results {
        for msg in &result.messages {
            out.push_str(&format!(
                "{}:{}:{} [{}] {} ({})\n",
                result.file_path.display(),
                msg.line,
                msg.column,
                msg.severity.as_str(),
                msg.message,
                msg.rule,
            ));
        }
    }
    out
}

fn render_grouped(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    for result in results {
        if result.messages.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "{}\n",
            format!("{}", result.file_path.display()).bold()
        ));
        for msg in &result.messages {
            let (prefix, color) = match msg.severity {
                lint::Severity::Error => ("error", colored::Color::Red),
                lint::Severity::Warning => ("warning", colored::Color::Yellow),
                lint::Severity::Info => ("info", colored::Color::Blue),
            };
            out.push_str(&format!(
                "  {}:{}:{} {} {}\n",
                msg.line,
                msg.column,
                prefix.color(color),
                msg.message,
                format!("({})", msg.rule).dimmed(),
            ));
        }
        out.push('\n');
    }
    out
}

fn render_results(
    results: &[lint::LintResult],
    format: &OutputFormat,
    quiet: bool,
    show_source: bool,
) -> String {
    match format {
        OutputFormat::Json => {
            if quiet {
                let filtered: Vec<_> = results
                    .iter()
                    .map(|r| {
                        let mut cr = r.clone();
                        cr.messages.retain(|m| m.severity == lint::Severity::Error);
                        cr
                    })
                    .collect();
                serde_json::to_string_pretty(&filtered).unwrap()
            } else {
                serde_json::to_string_pretty(results).unwrap()
            }
        }
        OutputFormat::Markdown => {
            let mut out = String::new();
            for result in results {
                let relevant: Vec<_> = if quiet {
                    result
                        .messages
                        .iter()
                        .filter(|m| m.severity == lint::Severity::Error)
                        .collect()
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
        OutputFormat::Junit => render_junit(results),
        OutputFormat::Concise => render_concise(results),
        OutputFormat::Gitlab => render_gitlab(results),
        OutputFormat::Grouped => render_grouped(results),
        OutputFormat::Text => {
            let mut out = String::new();
            let mut total_errors = 0;
            let mut total_warnings = 0;
            let mut total_infos = 0;

            for result in results {
                let relevant: Vec<_> = if quiet {
                    result
                        .messages
                        .iter()
                        .filter(|m| m.severity == lint::Severity::Error)
                        .collect()
                } else {
                    result.messages.iter().collect()
                };

                if relevant.is_empty() {
                    if !quiet && result.messages.is_empty() {
                        out.push_str(&format!(
                            "{}: No issues found\n",
                            result.file_path.display()
                        ));
                    }
                } else {
                    out.push_str(&format!(
                        "{}\n",
                        format!("{}", result.file_path.display()).bold()
                    ));
                    let lines: Vec<&str> = result.file_content.lines().collect();
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

                        let fix_indicator = if msg.fix.is_some() { "[*] " } else { "" };
                        let code = lint::rules::code_from_name(&msg.rule);
                        out.push_str(&format!(
                            "  {}:{} {}{}[{}] {}: {}\n",
                            msg.line,
                            msg.column,
                            fix_indicator.dimmed(),
                            if fix_indicator.is_empty() { "" } else { " " },
                            code.dimmed(),
                            prefix.color(color),
                            msg.message
                        ));

                        #[allow(clippy::collapsible_if)]
                        if show_source {
                            let line_idx = msg.line.saturating_sub(1);
                            if line_idx > 0 {
                                if let Some(prev) = lines.get(line_idx - 1) {
                                    out.push_str(&format!("    | {}\n", prev.dimmed()));
                                }
                            }
                            if let Some(source_line) = lines.get(line_idx) {
                                out.push_str(&format!("    | {}\n", source_line));
                                let caret =
                                    format!("    |{}^", " ".repeat(msg.column.saturating_sub(1)));
                                out.push_str(&format!("{}\n", caret.dimmed()));
                            }
                            if let Some(next) = lines.get(line_idx + 1) {
                                out.push_str(&format!("    | {}\n", next.dimmed()));
                            }
                        }

                        if let Some(suggestion) = &msg.suggestion {
                            out.push_str(&format!(
                                "    {} help: {}\n",
                                "→".dimmed(),
                                suggestion.dimmed()
                            ));
                        }
                    }
                    out.push('\n');
                }
            }

            if !quiet {
                let file_count = results.len();
                let summary = format!(
                    "Summary: {} errors, {} warnings, {} infos in {} files",
                    total_errors, total_warnings, total_infos, file_count
                );
                out.push_str(&format!("{}\n", summary.bold()));
            }
            out
        }
    }
}

fn print_results(
    results: &[lint::LintResult],
    format: &OutputFormat,
    quiet: bool,
    show_source: bool,
) {
    print!("{}", render_results(results, format, quiet, show_source));
}

fn find_config_file() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let json_candidate = dir.join(".lint.json");
        if json_candidate.is_file() {
            return Some(json_candidate);
        }
        let toml_candidate = dir.join(".lint.toml");
        if toml_candidate.is_file() {
            return Some(toml_candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn list_rules() {
    let config = lint::ConfigBuilder::new().build();
    let linter = lint::Linter::new(&config);

    println!("Available rules (generic, apply to all files):");
    for rule in linter.rule_set().get_rules() {
        let fix_marker = if rule.has_fix() {
            " (has auto-fix)"
        } else {
            ""
        };
        println!(
            "  [{:13}] {} {}{}",
            rule.category(),
            rule.code(),
            rule.name(),
            fix_marker
        );
        if !rule.description().is_empty() {
            println!("                   {}", rule.description());
        }
    }

    println!();
    println!("Language-specific rules (auto-applied by file extension):");
    for rule in linter.language_rule_set().get_rules() {
        let fix_marker = if rule.has_fix() {
            " (has auto-fix)"
        } else {
            ""
        };
        println!(
            "  [{:13}] {} {}{}",
            rule.category(),
            rule.code(),
            rule.name(),
            fix_marker
        );
        if !rule.description().is_empty() {
            println!("                   {}", rule.description());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // Serialize tests that change the process current directory.
    static CURRENT_DIR_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
            "severity_overrides": {},
            "ignore_suppressions": false,
            "cache_strategy": "Metadata"
        }"#;
        temp_file.write_all(config_json.as_bytes())?;
        temp_file.flush()?;

        let config = load_config_file(&temp_file.path().to_path_buf())?;
        assert_eq!(config.paths, vec![PathBuf::from("src")]);
        assert_eq!(config.max_line_length, Some(120));
        assert_eq!(
            config.rule_set.enabled_rules,
            vec!["line-length".to_string()]
        );
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
                show_fixes,
                exclude,
                show_settings,
                cache_location,
                diff,
                fix_only,
                add_noqa,
                ignore_suppressions,
                select_all,
                no_ignore,
                cache_strategy,
                no_error_on_unmatched_pattern,
                deny_warnings,
                exit_non_zero_on_fix,
                no_cache,
                no_config,
                check,
                max_diagnostics,
                baseline,
                force_exclude,
                ext,
                verbose,
                unsafe_fixes,
                fixable,
                unfixable,
                preview,
                no_show_source,
                no_gitignore,
                changed,
                staged,
                plugin,
                progress,
                validate_config,
                max_nesting_depth,
                max_function_lines,
            } => {
                let _ = no_cache;
                let _ = no_config;
                let _ = progress;
                let _ = validate_config;
                let _ = max_nesting_depth;
                let _ = max_function_lines;
                let _ = check;
                let _ = max_diagnostics;
                let _ = baseline;
                let _ = force_exclude;
                let _ = ext;
                let _ = verbose;
                let _ = unsafe_fixes;
                let _ = fixable;
                let _ = unfixable;
                let _ = preview;
                let _ = no_show_source;
                let _ = no_gitignore;
                let _ = changed;
                let _ = staged;
                let _ = plugin;
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
                assert!(!show_fixes);
                assert_eq!(exclude, None);
                assert!(!show_settings);
                assert_eq!(cache_location, None);
                assert!(!diff);
                assert!(!fix_only);
                assert!(!add_noqa);
                assert!(!ignore_suppressions);
                assert!(!select_all);
                assert!(!no_ignore);
                assert_eq!(cache_strategy, None);
                assert!(!no_error_on_unmatched_pattern);
                assert!(!deny_warnings);
                assert!(!exit_non_zero_on_fix);
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
    fn test_cli_output_junit_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "junit"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("junit".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_concise_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "concise"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("concise".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_gitlab_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "gitlab"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("gitlab".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_grouped_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output", "grouped"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("grouped".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_output_format_alias_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--output-format", "json"]);
        match cli.command {
            Commands::Lint { output, .. } => {
                assert_eq!(output, Some("json".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_select_all_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--select-all"]);
        match cli.command {
            Commands::Lint { select_all, .. } => {
                assert!(select_all);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_no_ignore_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--no-ignore"]);
        match cli.command {
            Commands::Lint { no_ignore, .. } => {
                assert!(no_ignore);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_cache_strategy_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--cache-strategy", "content"]);
        match cli.command {
            Commands::Lint { cache_strategy, .. } => {
                assert_eq!(cache_strategy, Some("content".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_no_error_on_unmatched_pattern_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--no-error-on-unmatched-pattern"]);
        match cli.command {
            Commands::Lint {
                no_error_on_unmatched_pattern,
                ..
            } => {
                assert!(no_error_on_unmatched_pattern);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_deny_warnings_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--deny-warnings"]);
        match cli.command {
            Commands::Lint { deny_warnings, .. } => {
                assert!(deny_warnings);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_exit_non_zero_on_fix_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--exit-non-zero-on-fix"]);
        match cli.command {
            Commands::Lint {
                exit_non_zero_on_fix,
                ..
            } => {
                assert!(exit_non_zero_on_fix);
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
    fn test_cli_show_fixes_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--show-fixes"]);
        match cli.command {
            Commands::Lint { show_fixes, .. } => {
                assert!(show_fixes);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_exclude_parsing() {
        let cli = Cli::parse_from([
            "lint",
            "lint",
            "src/",
            "--exclude",
            "vendor",
            "--exclude",
            "*.min.js",
        ]);
        match cli.command {
            Commands::Lint { exclude, .. } => {
                assert_eq!(
                    exclude,
                    Some(vec!["vendor".to_string(), "*.min.js".to_string()])
                );
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_show_settings_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--show-settings"]);
        match cli.command {
            Commands::Lint { show_settings, .. } => {
                assert!(show_settings);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_cache_location_parsing() {
        let cli = Cli::parse_from([
            "lint",
            "lint",
            "src/",
            "--cache-location",
            "/tmp/lint_cache.json",
        ]);
        match cli.command {
            Commands::Lint { cache_location, .. } => {
                assert_eq!(cache_location, Some(PathBuf::from("/tmp/lint_cache.json")));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_diff_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--diff"]);
        match cli.command {
            Commands::Lint { diff, .. } => {
                assert!(diff);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_fix_only_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--fix-only"]);
        match cli.command {
            Commands::Lint { fix_only, .. } => {
                assert!(fix_only);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_add_noqa_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--add-noqa"]);
        match cli.command {
            Commands::Lint { add_noqa, .. } => {
                assert!(add_noqa);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_explain_parsing() {
        let cli = Cli::parse_from(["lint", "explain", "line-length"]);
        match cli.command {
            Commands::Explain { rule } => {
                assert_eq!(rule, "line-length");
            }
            _ => panic!("Expected Explain command"),
        }
    }

    #[test]
    fn test_cli_ignore_suppressions_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--ignore-suppressions"]);
        match cli.command {
            Commands::Lint {
                ignore_suppressions,
                ..
            } => {
                assert!(ignore_suppressions);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_check_alias_parsing() {
        let cli = Cli::parse_from(["lint", "check", "src/"]);
        match cli.command {
            Commands::Lint { paths, .. } => {
                assert_eq!(paths, vec![PathBuf::from("src/")]);
            }
            _ => panic!("Expected Lint command via check alias"),
        }
    }

    #[test]
    fn test_cli_init_parsing() {
        let cli = Cli::parse_from(["lint", "init"]);
        match cli.command {
            Commands::Init => {}
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_no_cache_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--no-cache"]);
        match cli.command {
            Commands::Lint { no_cache, .. } => {
                assert!(no_cache);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_no_config_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--no-config"]);
        match cli.command {
            Commands::Lint { no_config, .. } => {
                assert!(no_config);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_check_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--check"]);
        match cli.command {
            Commands::Lint { check, .. } => {
                assert!(check);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_find_config_file_discovers_lint_json() {
        let _guard = CURRENT_DIR_MUTEX.lock().unwrap();
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".lint.json");
        let mut file = std::fs::File::create(&config_path).unwrap();
        file.write_all(b"{\"paths\": [\"src\"]}").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let found = find_config_file();
        std::env::set_current_dir(original_dir).unwrap();

        assert!(found.is_some());
        let found_canon = found.unwrap().canonicalize().unwrap();
        let expected_canon = config_path.canonicalize().unwrap();
        assert_eq!(found_canon, expected_canon);
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
            Commands::Lint {
                stdin,
                stdin_filename,
                ..
            } => {
                assert!(stdin);
                assert_eq!(stdin_filename, Some("test.rs".to_string()));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_stdin_file_path_alias_parsing() {
        let cli = Cli::parse_from([
            "lint",
            "lint",
            "--stdin",
            "--stdin-file-path",
            "src/main.rs",
        ]);
        match cli.command {
            Commands::Lint {
                stdin,
                stdin_filename,
                ..
            } => {
                assert!(stdin);
                assert_eq!(stdin_filename, Some("src/main.rs".to_string()));
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
        let mut result =
            lint::LintResult::new(PathBuf::from("src/main.rs"), "let x = 5;   \n".to_string());
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
            out.contains("::warning file=src/main.rs,line=1,col=10,title=W002 (trailing-whitespace)::Trailing whitespace\n"),
            "warning line missing/malformed: {out}"
        );
        assert!(
            out.contains(
                "::error file=src/main.rs,line=2,col=1,title=W001 (line-length)::Line too long\n"
            ),
            "error line missing/malformed: {out}"
        );
        assert!(
            out.contains(
                "::notice file=src/main.rs,line=3,col=1,title=W003 (no-todo)::Found TODO\n"
            ),
            "notice line missing/malformed: {out}"
        );
    }

    #[test]
    fn test_render_github_escapes_message() {
        let mut result = lint::LintResult::new(PathBuf::from("src/a.rs"), "x".to_string());
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
    fn test_render_grouped_groups_by_file() {
        let mut r1 = lint::LintResult::new(PathBuf::from("a.rs"), "x".to_string());
        r1.add_message(lint::LintMessage::new(
            1,
            1,
            lint::Severity::Error,
            "e1".to_string(),
            "r1".to_string(),
            None,
        ));
        let mut r2 = lint::LintResult::new(PathBuf::from("b.rs"), "y".to_string());
        r2.add_message(lint::LintMessage::new(
            2,
            3,
            lint::Severity::Warning,
            "w1".to_string(),
            "r2".to_string(),
            None,
        ));
        let clean = lint::LintResult::new(PathBuf::from("c.rs"), "z".to_string());

        let out = render_grouped(&[r1, r2, clean]);
        assert!(out.contains("a.rs"), "missing a.rs: {out}");
        assert!(out.contains("b.rs"), "missing b.rs: {out}");
        assert!(!out.contains("c.rs"), "should not show clean file: {out}");
        assert!(out.contains("1:1:"), "missing line/col: {out}");
        assert!(out.contains("e1"), "missing message: {out}");
        assert!(out.contains("(r1)"), "missing rule: {out}");
    }

    #[test]
    fn test_render_sarif_produces_valid_json() {
        let mut result =
            lint::LintResult::new(PathBuf::from("src/main.rs"), "let x = 5;   \n".to_string());
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
        assert!(out.contains("\"ruleId\": \"W002\""));
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
        assert_eq!(
            merged.per_file_ignores.get("tests/**/*.rs"),
            Some(&vec!["no-todo".to_string()])
        );
        assert_eq!(
            merged.per_file_ignores.get("src/**/*.rs"),
            Some(&vec!["line-length".to_string()])
        );
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

    #[test]
    fn test_check_mode_does_not_write_and_returns_nonzero() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "let x = 5;   \n")?;

        let config = ConfigBuilder::new()
            .paths(vec![file_path.clone()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let exit_code = run_lint_and_print(
            &config,
            None,
            RunOptions {
                fix: false,
                fix_only: false,
                diff: false,
                check: true,
                quiet: true,
                verbose: false,
                statistics: false,
                show_fixes: false,
                add_noqa: false,
                exit_zero: false,
                exit_non_zero_on_fix: false,
                deny_warnings: false,
                unsafe_fixes: false,
                max_warnings: None,
                max_diagnostics: None,
                output_format: &lint::OutputFormat::Text,
                output_file: None,
                baseline: None,
                fixable: None,
                unfixable: None,
                progress: false,
                per_dir_configs: std::collections::HashMap::new(),
            },
        )?;

        assert_eq!(
            exit_code, 1,
            "check mode should exit 1 when fixable violations exist"
        );

        let content = std::fs::read_to_string(&file_path)?;
        assert_eq!(
            content, "let x = 5;   \n",
            "check mode should not modify files"
        );

        Ok(())
    }

    #[test]
    fn test_cli_max_diagnostics_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--max-diagnostics", "50"]);
        match cli.command {
            Commands::Lint {
                max_diagnostics, ..
            } => {
                assert_eq!(max_diagnostics, Some(50));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_truncate_diagnostics_limits_total() {
        let mut results = vec![
            lint::LintResult::new(PathBuf::from("a.rs"), "x".to_string()),
            lint::LintResult::new(PathBuf::from("b.rs"), "y".to_string()),
        ];
        for _ in 0..3 {
            results[0].add_message(lint::LintMessage::new(
                1,
                1,
                lint::Severity::Warning,
                "w1".to_string(),
                "r1".to_string(),
                None,
            ));
        }
        for _ in 0..3 {
            results[1].add_message(lint::LintMessage::new(
                1,
                1,
                lint::Severity::Warning,
                "w2".to_string(),
                "r2".to_string(),
                None,
            ));
        }

        let hidden = truncate_diagnostics(&mut results, Some(4));
        assert_eq!(results[0].messages.len(), 3);
        assert_eq!(results[1].messages.len(), 1);
        assert_eq!(hidden, 2);
    }

    #[test]
    fn test_truncate_diagnostics_no_limit() {
        let mut results = vec![lint::LintResult::new(
            PathBuf::from("a.rs"),
            "x".to_string(),
        )];
        for _ in 0..5 {
            results[0].add_message(lint::LintMessage::new(
                1,
                1,
                lint::Severity::Warning,
                "w".to_string(),
                "r".to_string(),
                None,
            ));
        }

        let hidden = truncate_diagnostics(&mut results, None);
        assert_eq!(results[0].messages.len(), 5);
        assert_eq!(hidden, 0);
    }

    #[test]
    fn test_load_config_file_toml() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join(".lint.toml");
        std::fs::write(
            &config_path,
            r#"
paths = ["src"]
max_line_length = 120

[rule_set]
enabled_rules = ["line-length", "no-todo"]
"#,
        )?;

        let config = load_config_file(&config_path)?;
        assert_eq!(config.max_line_length, Some(120));
        assert_eq!(config.paths, vec![PathBuf::from("src")]);
        assert!(
            config
                .rule_set
                .enabled_rules
                .contains(&"line-length".to_string())
        );
        assert!(
            config
                .rule_set
                .enabled_rules
                .contains(&"no-todo".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_find_config_file_discovers_lint_toml() {
        let _guard = CURRENT_DIR_MUTEX.lock().unwrap();
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".lint.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        file.write_all(b"paths = [\"src\"]\n").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let found = find_config_file();
        std::env::set_current_dir(original_dir).unwrap();

        assert!(found.is_some());
        assert_eq!(
            found.unwrap().file_name(),
            Some(std::ffi::OsStr::new(".lint.toml"))
        );
    }

    #[test]
    fn test_cli_baseline_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--baseline", "baseline.json"]);
        match cli.command {
            Commands::Lint { baseline, .. } => {
                assert_eq!(baseline, Some(PathBuf::from("baseline.json")));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_filter_baseline_removes_known_violations() {
        let mut results = vec![lint::LintResult::new(
            PathBuf::from("src/main.rs"),
            "let x = 5;\n".to_string(),
        )];
        results[0].add_message(lint::LintMessage::new(
            1,
            1,
            lint::Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            None,
        ));
        results[0].add_message(lint::LintMessage::new(
            2,
            1,
            lint::Severity::Warning,
            "Line too long".to_string(),
            "line-length".to_string(),
            None,
        ));

        let baseline = vec![BaselineEntry {
            file: "src/main.rs".to_string(),
            line: 1,
            rule: "trailing-whitespace".to_string(),
        }];

        filter_baseline(&mut results, &baseline);
        assert_eq!(results[0].messages.len(), 1);
        assert_eq!(results[0].messages[0].rule, "line-length");
    }

    #[test]
    fn test_filter_baseline_leaves_unknown_violations() {
        let mut results = vec![lint::LintResult::new(
            PathBuf::from("src/main.rs"),
            "let x = 5;\n".to_string(),
        )];
        results[0].add_message(lint::LintMessage::new(
            1,
            1,
            lint::Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            None,
        ));

        let baseline = vec![BaselineEntry {
            file: "src/other.rs".to_string(),
            line: 1,
            rule: "trailing-whitespace".to_string(),
        }];

        filter_baseline(&mut results, &baseline);
        assert_eq!(results[0].messages.len(), 1);
    }

    #[test]
    fn test_cli_force_exclude_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--force-exclude"]);
        match cli.command {
            Commands::Lint { force_exclude, .. } => {
                assert!(force_exclude);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_force_exclude_skips_ignored_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("package.js");
        std::fs::write(&file_path, "let x = 5;\n").unwrap();
        let mut full_path = temp_dir.path().to_path_buf();
        full_path.push("node_modules");
        std::fs::create_dir(&full_path).unwrap();
        let nested_file = full_path.join("package.js");
        std::fs::write(&nested_file, "let x = 5;\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![nested_file.clone()])
            .ignore_patterns(vec!["node_modules".to_string()])
            .force_exclude(true)
            .enabled_rules(vec!["line-length".to_string()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_force_exclude_disabled_allows_ignored_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut full_path = temp_dir.path().to_path_buf();
        full_path.push("node_modules");
        std::fs::create_dir(&full_path).unwrap();
        let nested_file = full_path.join("package.js");
        std::fs::write(&nested_file, "let x = 5;\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![nested_file.clone()])
            .ignore_patterns(vec!["node_modules".to_string()])
            .force_exclude(false)
            .enabled_rules(vec![])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_path, nested_file);
    }

    #[test]
    fn test_cli_ext_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--ext", "rs,toml"]);
        match cli.command {
            Commands::Lint { ext, .. } => {
                assert_eq!(ext, Some(vec!["rs".to_string(), "toml".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_ext_limits_file_extensions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("test.rs");
        let md_file = temp_dir.path().join("test.md");
        std::fs::write(&rs_file, "let x = 5;\n").unwrap();
        std::fs::write(&md_file, "# hello\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![temp_dir.path().to_path_buf()])
            .ext(Some(vec!["rs".to_string()]))
            .enabled_rules(vec!["line-length".to_string()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].file_path.file_stem(),
            Some(std::ffi::OsStr::new("test"))
        );
        assert_eq!(
            results[0].file_path.extension(),
            Some(std::ffi::OsStr::new("rs"))
        );
    }

    #[test]
    fn test_cli_verbose_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--verbose"]);
        match cli.command {
            Commands::Lint { verbose, .. } => {
                assert!(verbose);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_progress_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--progress"]);
        match cli.command {
            Commands::Lint { progress, .. } => {
                assert!(progress);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_unsafe_fixes_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--unsafe-fixes"]);
        match cli.command {
            Commands::Lint { unsafe_fixes, .. } => {
                assert!(unsafe_fixes);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_unsafe_fixes_filtering() {
        let mut results = vec![lint::LintResult::new(
            PathBuf::from("test.rs"),
            "let x = 5;   \n".to_string(),
        )];
        let mut msg = lint::LintMessage::new(
            1,
            1,
            lint::Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            None,
        );
        msg.fix = Some(lint::output::Fix {
            line: 1,
            replacement: "let x = 5;".to_string(),
            is_safe: false,
        });
        results[0].add_message(msg);

        // Simulate fix without unsafe_fixes
        for result in &mut results {
            for msg in &mut result.messages {
                if let Some(ref fix) = msg.fix
                    && !fix.is_safe
                {
                    msg.fix = None;
                }
            }
        }

        assert!(results[0].messages[0].fix.is_none());
    }

    #[test]
    fn test_cli_line_length_alias_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--line-length", "120"]);
        match cli.command {
            Commands::Lint {
                max_line_length, ..
            } => {
                assert_eq!(max_line_length, Some(120));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_fixable_parsing() {
        let cli = Cli::parse_from([
            "lint",
            "lint",
            "src/",
            "--fixable",
            "trailing-whitespace,line-length",
        ]);
        match cli.command {
            Commands::Lint { fixable, .. } => {
                assert_eq!(
                    fixable,
                    Some(vec![
                        "trailing-whitespace".to_string(),
                        "line-length".to_string()
                    ])
                );
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_unfixable_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--unfixable", "line-length"]);
        match cli.command {
            Commands::Lint { unfixable, .. } => {
                assert_eq!(unfixable, Some(vec!["line-length".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_fixable_filters_fixes() {
        let mut results = vec![lint::LintResult::new(
            PathBuf::from("test.rs"),
            "let x = 5;   \n".to_string(),
        )];
        let mut msg = lint::LintMessage::new(
            1,
            1,
            lint::Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            None,
        );
        msg.fix = Some(lint::output::Fix {
            line: 1,
            replacement: "let x = 5;".to_string(),
            is_safe: true,
        });
        results[0].add_message(msg);

        let allowed: std::collections::HashSet<String> =
            ["line-length".to_string()].into_iter().collect();
        for result in &mut results {
            for msg in &mut result.messages {
                if let Some(ref _fix) = msg.fix
                    && !allowed.contains(&msg.rule)
                {
                    msg.fix = None;
                }
            }
        }

        assert!(results[0].messages[0].fix.is_none());
    }

    #[test]
    fn test_cli_preview_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--preview"]);
        match cli.command {
            Commands::Lint { preview, .. } => {
                assert!(preview);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_no_show_source_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--no-show-source"]);
        match cli.command {
            Commands::Lint { no_show_source, .. } => {
                assert!(no_show_source);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_no_gitignore_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--no-gitignore"]);
        match cli.command {
            Commands::Lint { no_gitignore, .. } => {
                assert!(no_gitignore);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_no_gitignore_lints_gitignored_files() -> anyhow::Result<()> {
        use std::io::Write;
        let temp_dir = tempfile::tempdir()?;
        let gitignore_path = temp_dir.path().join(".gitignore");
        let mut gitignore = std::fs::File::create(&gitignore_path)?;
        gitignore.write_all(b"ignored.rs\n")?;

        let ignored_file = temp_dir.path().join("ignored.rs");
        std::fs::write(&ignored_file, "let x = 5;   \n")?;

        let config = ConfigBuilder::new()
            .paths(vec![temp_dir.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .no_gitignore(true)
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run()?;

        let ignored_result = results.iter().find(|r| r.file_path.ends_with("ignored.rs"));
        assert!(
            ignored_result.is_some(),
            "ignored.rs should be linted when --no-gitignore is used"
        );
        assert!(!ignored_result.unwrap().messages.is_empty());
        Ok(())
    }

    #[test]
    fn test_cli_changed_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--changed"]);
        match cli.command {
            Commands::Lint { changed, .. } => {
                assert!(changed);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_staged_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--staged"]);
        match cli.command {
            Commands::Lint { staged, .. } => {
                assert!(staged);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_git_changed_files_returns_changed() -> anyhow::Result<()> {
        let _guard = CURRENT_DIR_MUTEX.lock().unwrap();
        if !git_available() {
            return Ok(());
        }
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path();

        // init git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(repo_path)
            .output()?;

        let file = repo_path.join("test.rs");
        std::fs::write(&file, "let x = 5;\n")?;
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()?;
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(repo_path)
            .output()?;

        // modify file
        std::fs::write(&file, "let x = 5;   \n")?;

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(repo_path)?;
        let files = git_changed_files(true, false)?;
        std::env::set_current_dir(original_dir)?;

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], PathBuf::from("test.rs"));
        Ok(())
    }

    #[test]
    fn test_git_changed_files_returns_staged() -> anyhow::Result<()> {
        let _guard = CURRENT_DIR_MUTEX.lock().unwrap();
        if !git_available() {
            return Ok(());
        }
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(repo_path)
            .output()?;

        let file = repo_path.join("test.rs");
        std::fs::write(&file, "let x = 5;\n")?;
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()?;
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(repo_path)
            .output()?;

        // modify and stage
        std::fs::write(&file, "let x = 5;   \n")?;
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()?;

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(repo_path)?;
        let files = git_changed_files(false, true)?;
        std::env::set_current_dir(original_dir)?;

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], PathBuf::from("test.rs"));
        Ok(())
    }

    #[test]
    fn test_git_changed_files_empty_when_clean() -> anyhow::Result<()> {
        let _guard = CURRENT_DIR_MUTEX.lock().unwrap();
        if !git_available() {
            return Ok(());
        }
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(repo_path)?;
        let files = git_changed_files(true, false)?;
        std::env::set_current_dir(original_dir)?;

        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn test_cli_plugin_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--plugin", "security"]);
        match cli.command {
            Commands::Lint { plugin, .. } => {
                assert_eq!(plugin, Some(vec!["security".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_multiple_plugins_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--plugin", "security,javascript"]);
        match cli.command {
            Commands::Lint { plugin, .. } => {
                assert_eq!(
                    plugin,
                    Some(vec!["security".to_string(), "javascript".to_string()])
                );
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_plugin_security_adds_rules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.js");
        std::fs::write(&file_path, "eval(userInput);\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![file_path.clone()])
            .plugins(vec!["security".to_string()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].messages.iter().any(|m| m.rule == "unsafe-eval"));
    }

    #[test]
    fn test_plugin_javascript_filters_language_rules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let js_file = temp_dir.path().join("test.js");
        let py_file = temp_dir.path().join("test.py");
        std::fs::write(&js_file, "console.log('hello');\n").unwrap();
        std::fs::write(&py_file, "print('hello')\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![temp_dir.path().to_path_buf()])
            .plugins(vec!["javascript".to_string()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();

        let js_result = results.iter().find(|r| r.file_path.ends_with("test.js"));
        let py_result = results.iter().find(|r| r.file_path.ends_with("test.py"));

        assert!(js_result.is_some());
        assert!(py_result.is_some());

        // JS should have console-log rule
        assert!(
            js_result
                .unwrap()
                .messages
                .iter()
                .any(|m| m.rule == "no-console-log")
        );
        // Python should NOT have print rule because only javascript plugin is enabled
        assert!(
            !py_result
                .unwrap()
                .messages
                .iter()
                .any(|m| m.rule == "no-print")
        );
    }

    #[test]
    fn test_no_plugin_runs_all_language_rules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let js_file = temp_dir.path().join("test.js");
        let py_file = temp_dir.path().join("test.py");
        std::fs::write(&js_file, "console.log('hello');\n").unwrap();
        std::fs::write(&py_file, "print('hello')\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![temp_dir.path().to_path_buf()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();

        let js_result = results.iter().find(|r| r.file_path.ends_with("test.js"));
        let py_result = results.iter().find(|r| r.file_path.ends_with("test.py"));

        assert!(js_result.is_some());
        assert!(py_result.is_some());
        assert!(
            js_result
                .unwrap()
                .messages
                .iter()
                .any(|m| m.rule == "no-console-log")
        );
        assert!(
            py_result
                .unwrap()
                .messages
                .iter()
                .any(|m| m.rule == "no-print")
        );
    }

    #[test]
    fn test_cli_select_category_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--select", "style"]);
        match cli.command {
            Commands::Lint { select, .. } => {
                assert_eq!(select, Some(vec!["style".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_ignore_category_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--ignore", "security"]);
        match cli.command {
            Commands::Lint { ignore, .. } => {
                assert_eq!(ignore, Some(vec!["security".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_select_all_value_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--select", "ALL"]);
        match cli.command {
            Commands::Lint { select, .. } => {
                assert_eq!(select, Some(vec!["ALL".to_string()]));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_select_category_expands_to_rules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("test.rs");
        std::fs::write(&file, "let x = 5;\n\n\nlet y = 6;\n").unwrap();

        // Default rules don't include no-consecutive-empty-lines
        let config = ConfigBuilder::new()
            .paths(vec![file.clone()])
            .enabled_rules(vec!["line-length".to_string()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();
        let msgs = &results[0].messages;
        // Only line-length is enabled by default in this config
        assert!(!msgs.iter().any(|m| m.rule == "no-consecutive-empty-lines"));

        // Now use --select style to add all style rules
        let config2 = ConfigBuilder::new()
            .paths(vec![file])
            .enabled_rules(vec!["line-length".to_string()])
            .build();
        // Simulate what main() does: expand categories
        let expanded = expand_rule_names(vec!["style".to_string()]);
        let mut enabled = config2.rule_set.enabled_rules.clone();
        for rule in expanded {
            if !enabled.contains(&rule) {
                enabled.push(rule);
            }
        }
        let config2 = ConfigBuilder::new()
            .paths(vec![temp_dir.path().join("test.rs")])
            .enabled_rules(enabled)
            .build();

        let linter2 = lint::Linter::new(&config2);
        let results2 = linter2.run().unwrap();
        let msgs2 = &results2[0].messages;
        assert!(msgs2.iter().any(|m| m.rule == "no-consecutive-empty-lines"));
    }

    #[test]
    fn test_ignore_category_removes_rules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("test.rs");
        std::fs::write(&file, "password = 'secret'\n").unwrap();

        let config = ConfigBuilder::new()
            .paths(vec![file.clone()])
            .enabled_rules(vec!["hardcoded-secret".to_string()])
            .build();

        let linter = lint::Linter::new(&config);
        let results = linter.run().unwrap();
        assert!(
            results[0]
                .messages
                .iter()
                .any(|m| m.rule == "hardcoded-secret")
        );

        // Now ignore the security category
        let config2 = ConfigBuilder::new()
            .paths(vec![file])
            .enabled_rules(vec!["hardcoded-secret".to_string()])
            .build();
        let expanded_ignore = expand_rule_names(vec!["security".to_string()]);
        let mut enabled = config2.rule_set.enabled_rules.clone();
        enabled.retain(|r| !expanded_ignore.contains(r));
        let config2 = ConfigBuilder::new()
            .paths(vec![temp_dir.path().join("test.rs")])
            .enabled_rules(enabled)
            .build();

        let linter2 = lint::Linter::new(&config2);
        let results2 = linter2.run().unwrap();
        assert!(
            !results2[0]
                .messages
                .iter()
                .any(|m| m.rule == "hardcoded-secret")
        );
    }

    #[test]
    fn test_select_all_expands_to_every_rule() {
        let expanded = expand_rule_names(vec!["ALL".to_string()]);
        let all_generic = lint::rules::known_rules();
        let all_lang = lint::language_rules::LanguageRuleSet::known_rules();

        // Should include every generic rule
        for rule in &all_generic {
            assert!(
                expanded.contains(rule),
                "ALL should include generic rule: {}",
                rule
            );
        }

        // Should include every language rule
        for rule in &all_lang {
            assert!(
                expanded.contains(rule),
                "ALL should include language rule: {}",
                rule
            );
        }

        // Should not have duplicates
        let unique: std::collections::HashSet<_> = expanded.iter().collect();
        assert_eq!(
            unique.len(),
            expanded.len(),
            "ALL expansion should have no duplicates"
        );
    }

    #[test]
    fn test_validate_config_valid() {
        let config = ConfigBuilder::new()
            .enabled_rules(vec!["line-length".to_string(), "no-todo".to_string()])
            .plugins(vec!["security".to_string()])
            .build();
        let errors = validate_config_errors(&config);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_validate_config_unknown_rule() {
        let config = ConfigBuilder::new()
            .enabled_rules(vec!["not-a-real-rule".to_string()])
            .build();
        let errors = validate_config_errors(&config);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not-a-real-rule"));
    }

    #[test]
    fn test_validate_config_unknown_plugin() {
        let config = ConfigBuilder::new()
            .plugins(vec!["not-a-plugin".to_string()])
            .build();
        let errors = validate_config_errors(&config);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not-a-plugin"));
    }

    #[test]
    fn test_validate_config_unknown_severity_rule() {
        let mut config = ConfigBuilder::new().build();
        config
            .severity_overrides
            .insert("fake-rule".to_string(), lint::output::Severity::Error);
        let errors = validate_config_errors(&config);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("fake-rule"));
    }

    #[test]
    fn test_cli_validate_config_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--validate-config"]);
        match cli.command {
            Commands::Lint {
                validate_config, ..
            } => {
                assert!(validate_config);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_max_nesting_depth_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--max-nesting-depth", "5"]);
        match cli.command {
            Commands::Lint {
                max_nesting_depth, ..
            } => {
                assert_eq!(max_nesting_depth, Some(5));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_max_function_lines_parsing() {
        let cli = Cli::parse_from(["lint", "lint", "src/", "--max-function-lines", "30"]);
        match cli.command {
            Commands::Lint {
                max_function_lines, ..
            } => {
                assert_eq!(max_function_lines, Some(30));
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_expand_rule_names_resolves_codes() {
        let expanded = expand_rule_names(vec!["W001".to_string()]);
        assert!(expanded.contains(&"line-length".to_string()));
    }

    #[test]
    fn test_validate_config_accepts_rule_codes() {
        let config = ConfigBuilder::new()
            .enabled_rules(vec!["W001".to_string(), "E001".to_string()])
            .build();
        let errors = validate_config_errors(&config);
        assert!(
            errors.is_empty(),
            "Expected no errors for valid codes, got: {:?}",
            errors
        );
    }
}
