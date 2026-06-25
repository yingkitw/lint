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

# Lint specific files
lint lint src/main.rs src/lib.rs

# Specify output format (text, json, markdown)
lint lint . --output json

# Set maximum line length
lint lint . --max-line-length 120

# Enable specific rules
lint lint . -r line-length -r trailing-whitespace

# Use a configuration file
lint lint . --config .lint.json

# Auto-fix fixable issues
lint lint . --fix

# List available rules
lint list-rules

# Show version
lint version
```

**Exit codes**: `0` if no issues found, `1` if any errors detected (warnings and infos do not affect the exit code).

### MCP Server

Run the MCP server:

```bash
cargo run --bin lint-mcp -- --host 127.0.0.1 --port 8080
```

The server exposes the following tools:

- `lint_files`: Lint specified files and return issues
- `list_rules`: List all available linting rules

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

## Benchmarking

Run the built-in performance benchmark:

```bash
cargo run --example bench
```

This generates a 10,000-line file and measures linting throughput.

## Contributing

If you find this project helpful, please consider giving it a star ⭐️

Feedback, issues, and pull requests are welcome! Feel free to open an issue for bug reports, feature requests, or questions.

## License

Apache-2.0
