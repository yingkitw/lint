# Lint

A versatile linting tool with multiple interfaces: CLI, MCP (Model Context Protocol), and library.

## Features

- **CLI Interface**: Run linting from the command line
- **MCP Server**: Expose linting capabilities via MCP protocol
- **Library API**: Use linting functionality in your Rust projects

## Installation

```bash
cargo install --path .
```

## Usage

### CLI

Lint files or directories:

```bash
# Lint current directory
lint lint .

# `check` is an alias for `lint`
lint check .

# Lint specific files
lint lint src/main.rs src/lib.rs

# Use glob patterns
lint lint "src/**/*.rs"

# Specify output format (text, json, markdown, github, sarif, junit, concise, gitlab)
lint lint . --output json

# --output-format is an alias for --output
lint lint . --output-format markdown

# Set maximum line length
lint lint . --max-line-length 120

# Enable specific rules
lint lint . -r line-length -r trailing-whitespace

# Use a configuration file
lint lint . --config .lint.json

# Config is auto-discovered: place .lint.json in your project root
# and run lint without --config

# .gitignore patterns are automatically respected when walking directories

# Auto-fix fixable issues
lint lint . --fix

# Preview fixes without writing changes
lint lint . --diff

# Apply fixes but don't report remaining violations
lint lint . --fix-only

# Auto-add suppression comments to all violations
lint lint . --add-noqa

# Ignore all suppression comments (useful for auditing)
lint lint . --ignore-suppressions

# Ignore all ignore patterns (lint everything including node_modules)
lint lint . --no-ignore

# Watch for changes and re-lint
lint lint . --watch

# Use cache to skip unchanged files
lint lint . --cache

# Use a custom cache file location
lint lint . --cache --cache-location /tmp/lint_cache.json

# Use content-based caching (more accurate, slower)
lint lint . --cache --cache-strategy content

# Only show errors (suppress warnings and infos)
lint lint . --quiet

# Fail if more than 10 warnings
lint lint . --max-warnings 10

# Disable colored output
lint lint . --color never

# Add a rule to the default set
lint lint . --select no-todo

# Remove a rule from the default set
lint lint . --ignore line-length

# Enable the final-newline rule
lint lint . --select final-newline

# Enable the no-mixed-line-endings rule
lint lint . --select no-mixed-line-endings

# Enable all built-in rules
lint lint . --select-all

# Enable all rules in a category
lint lint . --select style
lint lint . --select security
lint lint . --select correctness,style

# Ignore all rules in a category
lint lint . --ignore security
lint lint . --ignore style

# Only lint files that have uncommitted changes (fast pre-commit hook)
lint lint . --changed

# Only lint staged files
lint lint . --staged

# Load a plugin rule pack
lint lint . --plugin security

# Load multiple plugins
lint lint . --plugin security --plugin javascript

# Don't respect .gitignore (lint generated files too)
lint lint . --no-gitignore

# Force exclude ignored files even when explicitly passed
lint lint . --force-exclude

# Disable source context in output
lint lint . --no-show-source

# List files that would be linted (without linting)
lint lint . --print-files

# Write JSON results to a file
lint lint . --output json --output-file results.json

# Generate SARIF output for GitHub Advanced Security
lint lint . --output sarif --output-file results.sarif.json

# Generate JUnit XML for CI integration
lint lint . --output junit --output-file results.junit.xml

# Generate GitLab Code Quality report
lint lint . --output gitlab --output-file gl-code-quality-report.json

# Concise one-line-per-violation output
lint lint . --output concise

# Show progress bar while linting (useful for large repos)
lint lint . --progress

# Show violations without failing the build
lint lint . --exit-zero

# Show effective configuration after merging all sources
lint lint . --show-settings

# Enable specific rules
lint lint . --select no-tabs --select no-consecutive-empty-lines

# Show per-rule violation statistics
lint lint . --statistics

# Exclude specific paths or patterns
lint lint . --exclude vendor --exclude "*.min.js"

# Don't fail when a glob pattern doesn't match any files
lint lint "nonexistent/**/*.rs" --no-error-on-unmatched-pattern

# Treat warnings as errors (exit 1 on any warning)
lint lint . --deny-warnings

# Exit non-zero even if all violations were fixed
lint lint . --fix --exit-non-zero-on-fix

# Lint code from stdin (useful for editor integrations)
echo 'let x = 5;   ' | lint lint --stdin --stdin-filename test.rs

# Validate configuration file (catches unknown rules, plugins, etc.)
lint lint . --validate-config

# Set maximum allowed nesting depth (default: 4)
lint lint . --max-nesting-depth 3

# Set maximum function length in lines (default: 50)
lint lint . --max-function-lines 30

# Detect out-of-order imports (supports Rust, JS/TS, Python, C/C++, Go)
lint lint . --rules sort-imports

# List available rules
lint list-rules

# Explain what a specific rule does
lint explain line-length

# Show version
lint version

# Generate a default configuration file
lint init
```

**Exit codes**: `0` if no issues found, `1` if any errors detected or if warnings exceed `--max-warnings`.

### Config Extends

Share a base configuration across projects:

```json
{
  "extends": ".lint.base.json",
  "max_line_length": 120,
  "rule_set": {
    "enabled_rules": ["line-length", "trailing-whitespace", "no-todo"]
  }
}
```

Values in the local config override the base. Collections like `ignore_patterns`, `per_file_ignores`, and `severity_overrides` are merged.

### Per-Directory Configuration

Place a `.lint.toml` or `.lint.json` in any subdirectory to override settings for files in that tree:

```toml
# src/.lint.toml — stricter rules for source code
max_line_length = 80
enabled_rules = ["line-length", "trailing-whitespace", "no-todo", "no-tabs"]
```

```toml
# tests/.lint.toml — relaxed rules for tests
max_line_length = 120
ignore_patterns = ["test_data"]
```

Per-directory configs are merged with the base config. Supported overrides: `max_line_length`, `enabled_rules`, `plugins`, `ignore_patterns`, `per_file_ignores`, `severity_overrides`.

### Unused Suppression Detection

Enable the `unused-suppression` rule to detect suppression comments that don't actually suppress any violations (useful for cleanup after refactoring):

```bash
lint lint . --rules line-length,trailing-whitespace,unused-suppression
```

This reports warnings like:

```
Unused suppression comment: `lint: ignore=line-length`
```

### MCP Server

Run the MCP server:

```bash
cargo run --bin lint-mcp -- --host 127.0.0.1 --port 8080
```

The server exposes the following tools:

- `lint_files`: Lint specified files and return issues
- `list_rules`: List all available linting rules

### LSP Server

Run the Language Server Protocol (LSP) server for real-time diagnostics in editors:

```bash
cargo run --bin lint-lsp -- --stdio
```

Or install and use with your editor:

```bash
cargo install --path .
# Then configure your editor to run `lint-lsp --stdio`
```

Supported LSP methods:

- `initialize` / `initialized`
- `textDocument/didOpen` — lint opened files and publish diagnostics
- `textDocument/didChange` — re-lint on change and publish diagnostics
- `textDocument/didClose` — clear diagnostics
- `shutdown` / `exit`

### Pre-commit Hook

Use `lint` with [pre-commit](https://pre-commit.com/):

```yaml
repos:
  - repo: https://github.com/yingkitw/lint
    rev: v0.1.3
    hooks:
      - id: lint
```

To lint only changed files (faster for large repos):

```yaml
repos:
  - repo: https://github.com/yingkitw/lint
    rev: v0.1.3
    hooks:
      - id: lint
        args: ['--changed']
```

To lint only staged files:

```yaml
repos:
  - repo: https://github.com/yingkitw/lint
    rev: v0.1.3
    hooks:
      - id: lint
        args: ['--staged']
```

### Library

Use as a library in your Rust project:

```rust
use lint::{ConfigBuilder, OutputFormat};

fn main() -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .paths(vec!["src".into()])
        .max_line_length(Some(100))
        .enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
        ])
        .output_format(OutputFormat::Json)
        .build();

    let results = lint::lint_files(&config)?;

    for result in results {
        println!("File: {}", result.file_path.display());
        for message in result.messages {
            println!("  {}: {}", message.severity.as_str(), message.message);
        }
    }

    Ok(())
}
```

## Available Rules

All violations include **fix suggestions** to help resolve issues. Use `--output text` to see `→ help:` messages, or `--output markdown` for `**Fix**:` blocks.

### Universal Rules
- `line-length`: Lines exceeding max length → break line, extract variable, or use continuation
- `trailing-whitespace`: Trailing spaces/tabs → remove
- `no-todo`: TODO/FIXME comments → address or create tracking issue

### JavaScript/TypeScript Rules
- `no-console-log`: console.log/warn/error → use logger
- `no-var`: var usage → use let/const
- `missing-semicolon`: Missing semicolons

### Python Rules
- `no-print`: print() → use logging module
- `python-style`: PEP 8 (PascalCase classes, snake_case functions)

### Go Rules
- `go-style`: Exported functions need documentation

### Java Rules
- `java-style`: PascalCase classes, no System.out → use SLF4J
- `missing-semicolon`: Missing semicolons

### Rust Rules
- `no-unwrap`: .unwrap() → use ? or match
- `no-expect`: .expect() → use ? or match
- `missing-semicolon`: Missing semicolons

### Ruby Rules
- `no-puts`: puts → use Logger
- `ruby-style`: CamelCase classes, attr_writer for setters

### PHP Rules
- `no-echo`: echo → use error_log or return JSON

### Swift Rules
- `no-swift-print`: print() → use OSLog

### Kotlin Rules
- `kotlin-style`: PascalCase classes, println → use slf4j

### Dart Rules
- `no-dart-print`: print() → use debugPrint or logging package

### C# Rules
- `no-csharp-console`: Console.WriteLine → use ILogger
- `csharp-style`: PascalCase classes

### Shell Rules
- `shell-echo-quote`: Unquoted variables in echo → quote with `"$VAR"`

### SQL Rules
- `sql-no-select-star`: SELECT * → list explicit columns

### Lua Rules
- `no-lua-print`: print() → use logging or remove

### Scala Rules
- `no-scala-println`: println() → use slf4j

### R Rules
- `no-r-print`: print() → use message() or cat()

### Zig Rules
- `no-zig-debug-print`: std.debug.print → remove or use std.log

### HTML Rules
- `html-no-inline-style`: Inline style= → move to CSS class
- `html-img-alt`: img without alt → add alt for accessibility

### CSS Rules
- `css-avoid-important`: !important → increase selector specificity

## Configuration

Create a custom configuration:

```rust
let config = ConfigBuilder::new()
    .paths(vec!["src".into(), "tests".into()])
    .ignore_patterns(vec![
        "node_modules".to_string(),
        "target".to_string(),
        ".git".to_string(),
    ])
    .max_line_length(Some(120))
    .enabled_rules(vec![
        "line-length".to_string(),
        "trailing-whitespace".to_string(),
    ])
    .custom_rules(Some("custom_rules.json".into()))
    .output_format(OutputFormat::Text)
    .build();
```

### Custom Rules

Define your own rules in a JSON file:

```json
[
  {
    "name": "no-debugger",
    "pattern": "\\bdebugger\\b",
    "message": "Debugger statement found",
    "severity": "Error",
    "suggestion": "Remove debugger before committing",
    "extensions": ["js", "ts"]
  }
]
```

- `name`: Unique rule identifier
- `pattern`: Regular expression to match
- `message`: Error/warning message
- `severity`: `Error`, `Warning`, or `Info`
- `suggestion`: Optional fix hint
- `extensions`: Optional list of file extensions to apply the rule to

### Suppressing Rules Inline

Suppress a rule on a specific line:

```rust
let x = 5; // lint: ignore=line-length
```

Suppress all rules on a line:

```rust
let x = 5; // lint: ignore
```

### Block Suppressions

Disable a rule for a block of code:

```rust
// lint: disable=line-length
const LONG_CONFIG: &str = "some very long configuration string that exceeds normal limits";
// lint: enable=line-length
```

Disable all rules for a block:

```rust
// lint: disable
const GENERATED_DATA: &str = "...generated content...";
// lint: enable
```

### File-Level Ignore

Ignore all rules for an entire file (must be on the first line):

```rust
// lint: ignore-file
// This file is auto-generated
const DATA: &str = "...";
```

Ignore a specific rule for the entire file:

```rust
// lint: ignore-file=line-length
// Test files often have long lines
```

Works in any language — the suppression comment is language-agnostic.

### Per-File Ignore Patterns

Disable specific rules for files matching a glob pattern via config:

```json
{
  "per_file_ignores": {
    "tests/**/*.rs": ["line-length"],
    "gen/**/*.js": ["line-length", "trailing-whitespace"]
  }
}
```

### Rule Severity Override

Override the default severity of any rule in your config:

```json
{
  "severity_overrides": {
    "line-length": "Error",
    "no-todo": "Warning"
  }
}
```

Valid severities: `Error`, `Warning`, `Info`.

## Supported File Extensions

- **Rust**: `.rs`
- **JavaScript/TypeScript**: `.js`, `.ts`, `.jsx`, `.tsx`
- **Python**: `.py`
- **Java**: `.java`
- **Go**: `.go`
- **C/C++**: `.c`, `.cpp`, `.h`, `.hpp`
- **Ruby**: `.rb`
- **PHP**: `.php`
- **Swift**: `.swift`
- **Kotlin**: `.kt`
- **Dart**: `.dart`
- **C#**: `.cs`
- **Shell**: `.sh`, `.bash`
- **SQL**: `.sql`
- **Lua**: `.lua`
- **Scala**: `.scala`
- **R**: `.r`
- **Zig**: `.zig`
- **HTML**: `.html`, `.htm`
- **CSS**: `.css`, `.scss`, `.sass`

## Output Formats

- **Text**: Human-readable output (default)
- **Json**: Machine-readable JSON format
- **Markdown**: Markdown-formatted output
- **GitHub**: GitHub Actions workflow commands (`::error file=...::...`)

## Benchmarking

Run the built-in performance benchmark:

```bash
cargo run --example bench
```

This generates 100 files (500 lines each) and measures linting throughput. The linter processes multiple files in parallel using `rayon` for better performance on multi-core machines.

## Contributing

If you find this project helpful, please consider giving it a star ⭐️

Feedback, issues, and pull requests are welcome! Feel free to open an issue for bug reports, feature requests, or questions.

## License

Apache-2.0
