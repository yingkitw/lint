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

- [ ] **Inline suppression comments (noqa)**
  - Ruff/Flake8/Pylint support `# noqa: rule-name` to suppress a rule on a specific line
  - Without this, users cannot suppress false positives — table stakes for any real linter
  - **Goal**: `let x = 5; // lint: ignore=line-length` suppresses the rule on that line only

- [ ] **Glob/wildcard path support**
  - Users expect `lint lint "src/**/*.rs"` — currently only exact paths work
  - **Goal**: Expand glob patterns in paths before linting

- [ ] **Watch mode (`--watch`)**
  - Ruff/ESLint have `--watch` to re-run on file changes
  - Big UX win for development workflows
  - **Goal**: `lint lint . --watch` re-lints on file changes using `notify` crate

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

## Done

- [x] Comprehensive unit and integration test coverage (92 lib + 8 bin + 17 advanced + 10 basic = 127 tests)
- [x] MCP server implementation
- [x] Multi-language rule support
- [x] `--fix` auto-fix capability (trailing whitespace)
- [x] Custom rules loaded from JSON file
- [x] `--config` CLI option wired up
- [x] `ignore_patterns` respected during directory traversal
- [x] Audit: removed unused dependencies (`glob`, `thiserror`) from `Cargo.toml`
- [x] Audit: fixed all `cargo clippy` warnings (collapsed nested ifs, replaced `len()` comparisons with `is_empty()`, simplified `map_or`)
