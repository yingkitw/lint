# SPEC.md

Scope, requirements, and quality bar for the `lint` project.

## 1. Scope

`lint` is a fast, multi-language linter with three interfaces:

- **CLI**: Primary interface for local development and CI.
- **Library**: Rust crate API for embedding in other tools.
- **MCP Server**: Model Context Protocol server for AI-driven workflows.

It lints source files for style and correctness issues, supports auto-fixes, and produces output in multiple formats for integration with editors and CI systems.

## 2. Functional Requirements

### 2.1 Core Linting

- Scan files and directories recursively.
- Apply universal rules (line length, trailing whitespace, TODO detection, etc.).
- Apply language-specific rules based on file extension.
- Respect `.gitignore` and user-configured ignore patterns.
- Parse suppression comments (`lint: ignore`, `lint: disable/enable`, `lint: ignore-file`).
- Detect unused suppression comments.
- Support per-file ignore patterns and severity overrides via config.

### 2.2 Auto-Fix

- Apply safe fixes automatically when `--fix` is passed.
- Support unsafe fixes behind `--unsafe-fixes`.
- Preview fixes with `--diff` without writing to disk.
- Filter fixable rules via `--fixable` / `--unfixable`.
- Add suppression comments automatically with `--add-noqa`.

### 2.3 Output Formats

- `text` (default): Human-readable with source context and caret underlines.
- `json`: Machine-readable JSON.
- `markdown`: Markdown-formatted diagnostic list.
- `github`: GitHub Actions workflow commands.
- `sarif`: SARIF for GitHub Advanced Security.
- `junit`: JUnit XML for CI integration.
- `gitlab`: GitLab Code Quality report.
- `concise`: One-line-per-violation.
- `grouped`: Grouped by rule.

### 2.4 Caching

- Optional file-system cache to skip unchanged files.
- Metadata-based caching (mtime + size).
- Content-based caching (SHA-256 hash).
- Custom cache file location.

### 2.5 Configuration

- JSON config file (`.lint.json`) with auto-discovery.
- Base config extension via `extends`.
- CLI arguments override config values.
- Config builder API for programmatic use.

### 2.6 MCP Server

- Expose `lint_files` and `list_rules` tools via JSON-RPC over TCP.
- Configurable host and port.

## 3. Non-Functional Requirements

### 3.1 Performance

- Multi-file parallel processing via `rayon`.
- Regex compilation once per rule (not per file check).
- O(1) ignore pattern lookups via `HashSet`.
- Cache fast path to avoid disk reads on unchanged files.

### 3.2 Compatibility

- Rust edition 2024.
- Supports 20+ programming languages and file types.
- Works on macOS, Linux, and Windows.

### 3.3 Reliability

- All features covered by unit and integration tests.
- `cargo test` passes before any PR is merged.
- `cargo clippy --all-targets --all-features` is clean.

## 4. Quality Bar

- No compiler warnings in release builds.
- No Clippy warnings (or explicitly suppressed with justification).
- Every public API function has a doc comment.
- Every new feature has an integration test.
- No dead code or unused imports.

## 5. Technical Stack

| Component | Choice |
|---|---|
| Language | Rust (edition 2024) |
| CLI Parser | `clap` (derive feature) |
| Serialization | `serde` + `serde_json` |
| Async Runtime | `tokio` (MCP server only) |
| Regex Engine | `regex` crate |
| Parallelism | `rayon` |
| File Watching | `notify` |
| Gitignore | `ignore` crate |
| Glob Matching | `glob` crate |
| Colors | `colored` crate |
| TOML Parsing | `toml` crate |
| Error Handling | `anyhow` |
| Testing | Built-in `test` + `tempfile` |

## 6. Out of Scope

- AST-based linting (the project is regex-based by design).
- Plugin system for dynamically loaded rules (planned for future).
- Git-aware linting (`--changed`, `--staged`) (planned for future).
- IDE/language server protocol (LSP) integration (planned for future).
