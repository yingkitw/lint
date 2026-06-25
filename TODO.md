# Lint Project TODO

## In Progress

## High Priority

- [x] **Fix `ignore_patterns` not being used by linter**
  - `Config` stores `ignore_patterns` with defaults (`node_modules`, `target`, `.git`)
  - `linter.rs` never checks them when walking directories
  - Files in ignored directories are still linted
  - **Goal**: Respect `ignore_patterns` during directory traversal
  - **Status**: Fixed in `linter.rs` with `is_ignored()` method. Unit + integration tests added.

- [x] **Wire up CLI `--config` option**
  - `main.rs` defines `--config FILE` but never reads or uses it
  - Should load a JSON config file and merge with CLI args
  - **Goal**: `--config` actually loads configuration from file
  - **Status**: Fixed in `main.rs` with `load_config_file()` helper. CLI overrides take precedence. Unit tests added.

## Medium Priority

- [x] **Remove dead `Format` enum from `output.rs`**
  - `Format` is defined but never used; `OutputFormat` from `config.rs` is used everywhere
  - `lib.rs` re-exports it alongside the real `OutputFormat`
  - **Goal**: Eliminate confusing duplicate type
  - **Status**: Removed from `output.rs` and `lib.rs`.

- [x] **Add CLI argument parsing tests**
  - `main.rs` has test coverage for `load_config_file` but not full CLI argument parsing
  - `TEST_COVERAGE.md` lists this as a future improvement
  - **Goal**: Test CLI subcommands and flag parsing end-to-end
  - **Status**: Added tests for `Cli::parse_from` with `lint`, `list-rules`, `version` subcommands and `--config` flag.

## High Priority (from competitive intelligence: Ruff, ESLint, Clippy)

- [x] **Inline suppression comments (noqa)**
  - Ruff/Flake8/Pylint support `# noqa: rule-name` to suppress a rule on a specific line
  - Without this, users cannot suppress false positives — table stakes for any real linter
  - **Goal**: `let x = 5; // lint: ignore=line-length` suppresses the rule on that line only
  - **Status**: Implemented in `linter.rs` with `is_line_suppressed()` helper. Supports `lint: ignore=rule-name` (specific) and `lint: ignore` (blanket). Unit + integration tests added.

- [x] **Glob/wildcard path support**
  - Users expect `lint lint "src/**/*.rs"` — currently only exact paths work
  - **Goal**: Expand glob patterns in paths before linting
  - **Status**: Added `expand_globs()` helper in `main.rs` using the `glob` crate. `*` and `?` patterns are expanded before linting. Non-glob paths pass through unchanged. Unit tests added.

- [x] **Watch mode (`--watch`)**
  - Ruff/ESLint have `--watch` to re-run on file changes
  - Big UX win for development workflows
  - **Goal**: `lint lint . --watch` re-lints on file changes using `notify` crate
  - **Status**: Added `--watch` flag, `run_lint_and_print()` helper, and watch loop using `notify` crate. Re-lints on create/modify/remove events. CLI parsing test added.

## Low Priority / Brainstorming

- [x] Exit non-zero when lint errors are found
  - **Status**: `main.rs` now exits with code 1 when any result contains `Severity::Error` messages. Integration test with custom Error rule added.
- [x] Add `--fix` / auto-fix capability
  - Messages already contain `suggestion` but there's no way to apply them
  - **Status**: Added `Fix` struct and `with_fix()` builder to `LintMessage`. `TrailingWhitespaceRule` now emits fixes. `--fix` CLI flag applies fixes and writes files back. Unit + integration tests added.
- [x] Load and apply `custom_rules_path`
  - `Config` has `custom_rules_path` but it's never read
  - **Status**: Implemented `CustomRule` and `CustomRuleDefinition` in `rules.rs`. Wired up in `linter.rs`. Unit + integration tests added.
- [x] Property-based tests with `proptest`
  - **Status**: Added manual property-style invariant tests: `LineLengthRule` never flags short lines, `TrailingWhitespaceRule` never flags clean lines, `NoTodoRule` never flags clean content, fixes never increase file size. Zero new dependencies.
- [x] Benchmarks for large file performance
  - **Status**: Added `examples/bench.rs` that generates a 10,000-line file and measures linting time. Zero new dependencies.
- [x] Snapshot tests for CLI output
  - **Status**: Added `test_json_output_snapshot` and `test_trailing_whitespace_exact_message` integration tests that assert exact expected output fields and values. Zero new dependencies.

## Brainstorming (from competitive intelligence round 6)

- [ ] **`--select` / `--ignore` rule filtering at CLI**
  - Ruff uses `--select E,W` and `--ignore E501` for fine-grained rule control
  - Our current `--rules` flag replaces the entire enabled set
  - **Goal**: `--select line-length` adds a rule; `--ignore no-todo` removes a rule from defaults

- [ ] **Config `extends` for shareable configs**
  - ESLint and Biome support extending other config files (`"extends": "./base.json"`)
  - Useful for monorepos and team-wide base configs
  - **Goal**: `{"extends": ".lint.base.json", "max_line_length": 120}` merges base + local

- [ ] **`--print-files` / `--list-files` CLI flag**
  - Oxlint added `--print-files` to list files that would be linted without actually linting them
  - Useful for debugging `ignore_patterns` and glob expansion
  - **Goal**: `lint lint . --print-files` prints one path per line, then exits

## Brainstorming (from competitive intelligence round 5)

- [x] **Unused suppression detection (`lint: unused-suppression`)**
  - Ruff has `RUF100` (unused-noqa) which detects suppression comments that don't actually suppress any violations
  - This happens when code is refactored and the suppression becomes unnecessary
  - **Goal**: A new rule or post-processing step that reports `// lint: ignore=rule-name` comments where no violation would have occurred
  - **Status**: Implemented as opt-in post-processing in `lint_file()`. `SuppressionDirective` struct tracks inline and block suppressions. `parse_suppression_directives()` parses all directives with ranges. Only runs when `unused-suppression` is in `enabled_rules`. Unit tests added.

## Brainstorming (from competitive intelligence round 4)

- [x] **File-level ignore directive (`lint: ignore-file`)**
  - Deno lint supports `// deno-lint-ignore-file` at the top of a file to ignore all rules for that file
  - Useful for generated code, vendor files, or legacy files you don't want to modify
  - **Goal**: `// lint: ignore-file` or `// lint: ignore-file=line-length` suppresses rules for the entire file
  - **Status**: Implemented `parse_file_level_ignore()` in `linter.rs`. Checked on first line only. Empty vec = ignore all rules; non-empty = skip specific rules during rule checking. Unit tests added.

- [x] **`--quiet` / `--max-warnings` CLI flags**
  - Many linters support `--quiet` to suppress all but errors, and `--max-warnings N` to fail if warnings exceed N
  - **Goal**: `lint lint . --quiet` shows only errors; `lint lint . --max-warnings 10` exits non-zero if >10 warnings
  - **Status**: Added `quiet: bool` and `max_warnings: Option<usize>` to `Commands::Lint`. `print_results` filters messages to only errors when quiet. `run_lint_and_print` counts warnings and returns exit code 1 if max_warnings exceeded. Supports all output formats (text, json, markdown, github). CLI parsing tests added.

- [x] **`--color` / `--no-color` CLI flags**
  - Ruff added `--color` to force colored output; most tools support `--no-color` for piping
  - **Goal**: `lint lint . --no-color` produces plain text even when stdout is a TTY
  - **Status**: Added `color: Option<String>` to `Commands::Lint`. Supports `never`/`no` and `always`/`yes` values. Uses `colored::control::set_override()` to force color on/off. CLI parsing test added.

## Brainstorming (from competitive intelligence round 3)

- [x] **Block suppression comments (`lint: disable=rule-name`)**
  - Ruff v0.15 stabilized `ruff: disable` / `ruff: enable` block comments
  - Currently we only support per-line suppression (`// lint: ignore=rule-name`)
  - **Goal**: `// lint: disable=line-length` suppresses the rule until `// lint: enable=line-length`
  - Useful for generated code blocks, test data, or config blobs where many lines violate a rule
  - **Status**: Implemented `parse_block_suppressions()` in `linter.rs`. Tracks `all_disabled` flag and `disabled_rules` set across lines. Integrated with existing `is_line_suppressed()` inline check. Supports both specific rule disable/enable and blanket disable/enable. Unit tests added.

## Brainstorming (from competitive intelligence round 2)

- [x] **Parallel linting with `rayon`**
  - ESLint v9.34.0 added multithread linting (`--concurrency`), seeing 1.3x–3x speedups
  - Rust's `rayon` crate makes parallel file processing trivial
  - **Goal**: `lint lint . --jobs 4` processes files in parallel worker threads
  - **Status**: Parallelized `Linter::run()` using `rayon`. Each top-level path is linted in parallel. `Linter::run()` changed from `&mut self` to `&self`. Benchmark example updated for multi-file workloads.

- [x] **Per-file ignore patterns**
  - Ruff's `per-file-ignores` disables specific rules for specific file patterns
  - Useful for test files (`tests/**`) or generated code (`gen/**`)
  - **Goal**: Config field like `{"tests/**/*.rs": ["line-length"]}` to skip rules per pattern
  - **Status**: Added `per_file_ignores: HashMap<String, Vec<String>>` to `Config` and `ConfigBuilder`. Filtering applied in `linter.rs` after inline suppression check, using `glob::Pattern` for glob matching. Unit tests added.

- [x] **File caching (`--cache`)**
  - ESLint's `--cache` only re-lints changed files, dramatically improving CI performance
  - Store hashes of file contents, skip unchanged files on subsequent runs
  - **Goal**: `lint lint . --cache` stores `.lint_cache.json` and skips unchanged files
  - **Status**: Added `cache.rs` module with `Cache` struct (mtime+size keying). `Linter` accepts optional `Arc<Mutex<Cache>>`. `--cache` CLI flag loads `.lint_cache.json`, skips unchanged files, and writes cache back. Unit tests for hit/miss/save/load added.

- [x] **GitHub Actions annotation output**
  - Many linters output `::error file={file},line={line}::{message}` for CI integration
  - Makes errors appear inline in PR diffs
  - **Goal**: `lint lint . --output github` produces GitHub Actions workflow commands
  - **Status**: Added `Github` variant to `OutputFormat`. `render_github()`, `escape_github_command()`, and `severity_to_github_command()` in `main.rs`. CLI parsing test and rendering tests added.

- [x] **Rule severity override in config**
  - Currently severity is hardcoded per rule (e.g., `line-length` → Warning, `no-todo` → Info)
  - Users may want `line-length: Error` in their config
  - **Goal**: Config supports `"severity_overrides": {"line-length": "Error"}`
  - **Status**: Added `severity_overrides: HashMap<String, Severity>` to `Config` and `ConfigBuilder`. Overrides applied in `linter.rs` after per-file ignore filtering. Unit tests added.

## Done

- [x] Comprehensive unit and integration test coverage (114 lib + 20 bin + 18 advanced + 10 basic = 162 tests)
- [x] MCP server implementation
- [x] Multi-language rule support
- [x] `--fix` auto-fix capability (trailing whitespace)
- [x] Custom rules loaded from JSON file
- [x] `--config` CLI option wired up
- [x] `ignore_patterns` respected during directory traversal
- [x] Inline suppression comments (`// lint: ignore=rule-name`)
- [x] Block suppression comments (`// lint: disable=rule-name` / `// lint: enable=rule-name`)
- [x] File-level ignore directive (`// lint: ignore-file`)
- [x] Unused suppression detection (`unused-suppression` opt-in rule)
- [x] Glob/wildcard path expansion (`src/**/*.rs`)
- [x] Watch mode (`--watch`)
- [x] Parallel file linting with `rayon`
- [x] Per-file ignore patterns (`{"tests/**/*.rs": ["line-length"]}`)
- [x] File caching (`--cache`, `.lint_cache.json`)
- [x] GitHub Actions annotation output (`--output github`)
- [x] Rule severity override in config (`"severity_overrides": {"line-length": "Error"}`)
- [x] `--quiet` / `--max-warnings` CLI flags
- [x] `--color` CLI flag (`never`/`always`)
- [x] Audit: removed unused dependencies (`glob`, `thiserror`) from `Cargo.toml`
- [x] Audit: fixed all `cargo clippy` warnings (collapsed nested ifs, replaced `len()` comparisons with `is_empty()`, simplified `map_or`)
