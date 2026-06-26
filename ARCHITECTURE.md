# ARCHITECTURE.md

Module relationships, data flow, and deployment topology for the `lint` project.

## 1. Module Overview

```
src/
├── main.rs           # CLI binary: argument parsing, command dispatch, output rendering
├── mcp_main.rs       # MCP binary: thin wrapper to start McpServer
├── lib.rs            # Library root: public API re-exports, convenience functions
├── config.rs         # Config / ConfigBuilder: serde-aware configuration model
├── linter.rs         # Linter: file discovery, rule execution, cache integration
├── rules.rs          # UniversalRule trait + built-in universal rules
├── language_rules.rs # LanguageRule trait + 18+ language-specific rules
├── output.rs         # LintResult / LintMessage / Severity / Fix structs
├── cache.rs          # Cache / CacheEntry: filesystem-backed result cache
└── mcp.rs            # McpServer: JSON-RPC over TCP, tool dispatch
```

## 2. Data Flow

### 2.1 CLI Invocation

```
main.rs
  │
  ├─→ clap parses CLI args → Commands enum
  │
  ├─→ load_config_file() reads .lint.json (if present)
  │
  ├─→ merge_configs() merges base + local + CLI overrides
  │
  ├─→ run_lint_and_print(config, cache, RunOptions)
  │     │
  │     ├─→ lint_files(config) or lint_files_with_cache(config, cache)
  │     │       │
  │     │       └─→ Linter::run()
  │     │               │
  │     │               ├─→ lint_path()  → file discovery (WalkBuilder)
  │     │               │       │
  │     │               │       └─→ lint_file() per file
  │     │               │               │
  │     │               │               ├─→ cache lookup (fast path)
  │     │               │               │
  │     │               │               ├─→ read file content
  │     │               │               │
  │     │               │               ├─→ RuleSet.check()   (universal rules)
  │     │               │               ├─→ LanguageRuleSet.check() (language rules)
  │     │               │               │
  │     │               │               ├─→ parse_suppressions()
  │     │               │               ├─→ filter suppressed messages
  │     │               │               ├─→ detect unused suppressions
  │     │               │               │
  │     │               │               └─→ cache insert (on miss)
  │     │               │
  │     │               └─→ Vec<LintResult>
  │     │
  │     ├─→ apply fixes (if --fix / --fix-only)
  │     │
  │     ├─→ filter baseline (if --baseline)
  │     │
  │     ├─→ render_results(format, quiet, show_source)
  │     │       │
  │     │       └─→ format-specific renderer (text/json/sarif/etc.)
  │     │
  │     └─→ print or write to --output-file
  │
  └─→ exit code (0 = clean, 1 = errors)
```

### 2.2 MCP Request Flow

```
mcp_main.rs
  │
  └─→ McpServer::run()
          │
          ├─→ TcpListener accepts connection
          │
          └─→ per-connection tokio task
                  │
                  ├─→ read JSON-RPC request
                  │
                  ├─→ dispatch method:
                  │       lint_files  → ConfigBuilder + lint_files()
                  │       list_rules  → static rule metadata
                  │
                  └─→ write JSON-RPC response
```

## 3. Key Structs and Traits

### 3.1 Configuration

- **`Config`** (`config.rs`): The canonical runtime configuration. Derived from builder or deserialized from JSON. Fields: `paths`, `ignore_patterns`, `max_line_length`, `rule_set`, `output_format`, `cache_strategy`, `show_source`, `no_gitignore`, etc.
- **`ConfigBuilder`** (`config.rs`): Fluent builder with defaults. Supports chaining for programmatic use.

### 3.2 Linting Engine

- **`Linter`** (`linter.rs`): Owns `Config`, `RuleSet`, `LanguageRuleSet`, optional `Cache`, and `Gitignore`. `run()` returns `Vec<LintResult>`.
- **`Rule`** (`rules.rs`): Universal rule trait (`name`, `category`, `check`, `supports_extension`).
- **`RuleSet`** (`rules.rs`): Collection of `Box<dyn Rule>`. Built-in rules: line-length, trailing-whitespace, no-todo, no-empty-file, no-consecutive-empty-lines, no-tabs, final-newline, no-mixed-line-endings.
- **`LanguageRule`** (`language_rules.rs`): Language-specific rule trait. 18+ implementations covering JS/TS, Python, Go, Java, Rust, Ruby, PHP, Swift, Kotlin, Dart, C#, Shell, SQL, Lua, Scala, R, Zig, HTML, CSS.
- **`LanguageRuleSet`** (`language_rules.rs`): Collection of `Box<dyn LanguageRule>`.

### 3.3 Output Model

- **`LintResult`** (`output.rs`): Per-file result containing `file_path`, `messages`, `file_content`.
- **`LintMessage`** (`output.rs`): Single diagnostic with `line`, `column`, `severity`, `message`, `rule`, `suggestion`, `fix`.
- **`Fix`** (`output.rs`): Auto-fix descriptor with `line`, `replacement`, `is_safe`.
- **`Severity`** (`output.rs`): `Error`, `Warning`, `Info`.

### 3.4 Cache

- **`Cache`** (`cache.rs`): In-memory HashMap keyed by `PathBuf`. Persists to JSON.
- **`CacheEntry`** (`cache.rs`): `mtime`, `size`, `content_hash` (optional), `messages`.

## 4. Deployment Topology

### 4.1 CLI Binary (`lint`)

- Single statically-linked binary.
- No runtime dependencies beyond the OS.
- Reads config from cwd upward (auto-discovery).
- Writes cache to `.lint_cache.json` (or custom location).

### 4.2 MCP Binary (`lint-mcp`)

- Standalone TCP server.
- Deployed alongside AI/IDE tools that speak MCP.
- Stateless: each request builds its own `Config` and `Linter`.

### 4.3 Library (`liblint`)

- Published to crates.io.
- Consumed as a normal Cargo dependency.
- Public API surface: `lint_files`, `lint_files_with_cache`, `Config`, `ConfigBuilder`, `Linter`, `Rule`, `RuleSet`, `LintResult`, `LintMessage`, `Severity`, `McpServer`, `LanguageRule`, `LanguageRuleSet`.

## 5. Concurrency Model

- **File-level parallelism**: `rayon` parallel iterator over discovered files in `Linter::run()`.
- **Cache synchronization**: `Arc<Mutex<Cache>>` shared across threads.
- **MCP server**: `tokio` async with one task per TCP connection.

## 6. Extension Points

- **New universal rule**: Implement `Rule`, add to `Linter::new_with_cache()`.
- **New language rule**: Implement `LanguageRule`, add to `LanguageRuleSet::new()`.
- **New output format**: Add variant to `OutputFormat`, add arm to `render_results()`.
- **Custom rules**: JSON definition with `name`, `pattern`, `message`, `severity`, `suggestion`, `extensions` loaded at runtime via `CustomRule`.
