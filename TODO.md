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

## Brainstorming (from competitive intelligence round 8)

- [x] **`--exit-zero` CLI flag**
  - Ruff supports `--exit-zero` to always exit with code 0 even if violations are found
  - Useful for CI pipelines where you want to see lint output without failing the build
  - **Goal**: `lint lint . --exit-zero` always returns 0
  - **Status**: Added `exit_zero: bool` to `Commands::Lint`. Wired up in `run_lint_and_print` to return 0 regardless of errors or max-warnings exceeded. CLI parsing test added.

- [x] **Show source context in text output**
  - Ruff shows the offending source line with a caret underline in text output
  - **Goal**: Text output includes the source line and a caret pointing to the column
  - **Status**: Text output now shows the offending source line and a caret underline (`| ` + source + `| ` + `^` at column). Uses `result.file_content.lines()` to extract the source line. No new tests needed (existing tests use JSON/GitHub/SARIF output).

- [x] **`--statistics` CLI flag**
  - Ruff supports `--statistics` to show per-rule violation counts after linting
  - **Goal**: `lint lint . --statistics` prints a sorted table of rule names and their violation counts
  - **Status**: Added `statistics: bool` to `Commands::Lint`. `run_lint_and_print` collects per-rule counts from all results, sorts by count descending, and prints a summary table. CLI parsing test added.

- [x] **`--show-fixes` CLI flag**
  - Ruff supports `--show-fixes` to list files that were modified by `--fix`
  - **Goal**: `lint lint . --fix --show-fixes` prints the list of files that were fixed after linting
  - **Status**: Added `show_fixes: bool` to `Commands::Lint`. After applying fixes, `run_lint_and_print` tracks fixed files and prints them if `show_fixes` is true. CLI parsing test added.

- [x] **`--exclude` CLI flag**
  - Ruff and ESLint support `--exclude` to skip files/patterns at CLI level
  - **Goal**: `lint lint . --exclude vendor --exclude "*.min.js"` skips matching files
  - **Status**: Added `exclude: Option<Vec<String>>` to `Commands::Lint`. Patterns merged into `config.ignore_patterns` before linting. CLI parsing test added.

- [x] **JUnit XML output format**
  - Ruff supports JUnit XML output for CI integration (Jenkins, Azure DevOps, GitLab)
  - **Goal**: `lint lint . --output junit` produces JUnit XML for test result ingestion
  - **Status**: Added `Junit` variant to `OutputFormat`. `render_junit()` generates valid JUnit XML with `<testsuite>` and `<testcase>` elements. XML escaping via `escape_xml()`. CLI parsing test added.

- [x] **Additional generic rules**
  - Current generic rules are limited (line-length, trailing-whitespace, no-todo)
  - **Goal**: Add `no-empty-file`, `no-consecutive-empty-lines`, `no-tabs` rules
  - **Status**: Added `NoEmptyFileRule`, `NoConsecutiveEmptyLinesRule`, and `NoTabsRule` to `rules.rs`. `NoTabsRule` includes auto-fix (replace tabs with 4 spaces). All wired into `linter.rs` available rules. Unit tests added.

- [x] **`--show-settings` CLI flag**
  - Ruff supports `--show-settings` to print the effective configuration
  - **Goal**: `lint lint . --show-settings` prints merged config as JSON and exits
  - **Status**: Added `show_settings: bool` to `Commands::Lint`. After config merging, prints config as pretty JSON and returns `Ok(())`. CLI parsing test added.

- [x] **Config file auto-discovery**
  - Ruff, ESLint, and Biome all auto-discover config files in current and parent directories
  - **Goal**: Running `lint lint .` without `--config` automatically finds `.lint.json` in the project root
  - **Status**: Added `find_config_file()` that walks from `current_dir()` up to root looking for `.lint.json`. CLI `--config` still takes precedence. Unit test added.

- [x] **Concise output format**
  - Ruff supports `--output-format concise` for compact one-line-per-violation output
  - **Goal**: `lint lint . --output concise` shows `file:line:col [severity] message (rule)` per violation
  - **Status**: Added `Concise` variant to `OutputFormat`. `render_concise()` generates one-line format. CLI parsing test added.

- [x] **`--cache-location` CLI flag**
  - Ruff supports `--cache-dir` to customize the cache directory
  - **Goal**: `lint lint . --cache --cache-location /tmp/lint_cache.json` uses a custom cache path
  - **Status**: Added `cache_location: Option<PathBuf>` to `Commands::Lint`. Cache path defaults to `.lint_cache.json` but can be overridden. CLI parsing test added.

- [x] **`--diff` CLI flag**
  - Ruff supports `--diff` to preview fixes without writing changes
  - **Goal**: `lint lint . --diff` shows a line-by-line diff of proposed fixes without modifying files
  - **Status**: Added `diff: bool` to `Commands::Lint`. When `diff` is true, `run_lint_and_print` applies fixes in-memory, prints `---/+++` diff header and changed lines (`-` old, `+` new), but never writes files. CLI parsing test added.

- [x] **`--fix-only` CLI flag**
  - Ruff supports `--fix-only` to apply fixes without reporting or failing on remaining violations
  - **Goal**: `lint lint . --fix-only` applies safe fixes, suppresses all output and exit codes for leftover violations
  - **Status**: Added `fix_only: bool` to `Commands::Lint`. When `fix_only` is true, fixes are applied, results are not rendered/printed, and exit code is always 0. CLI parsing test added.

- [x] **`--add-noqa` CLI flag**
  - Ruff supports `--add-noqa` to automatically add `# noqa` suppression comments to failing lines
  - **Goal**: `lint lint . --add-noqa` appends `// lint: ignore=rule-name` to each offending line and writes the files
  - **Status**: Added `add_noqa: bool` to `Commands::Lint`. After linting, for each message, appends suppression directive to the line if not already present. Writes modified files. CLI parsing test added.

- [x] **`explain` subcommand**
  - Ruff supports `ruff rule <code>` to explain what a specific rule does
  - **Goal**: `lint explain line-length` shows name, category, and description of the rule
  - **Status**: Added `Explain { rule: String }` to `Commands`. `explain_rule()` searches generic and language-specific rules and prints details. Added `description()` to `Rule` and `LanguageRule` traits. `get_rules()` added to `LanguageRuleSet`. CLI parsing test added.

- [x] **`--ignore-suppressions` CLI flag**
  - Ruff supports `--ignore-noqa` to ignore all `# noqa` suppression comments
  - **Goal**: `lint lint . --ignore-suppressions` reports all violations even if suppressed by inline comments
  - **Status**: Added `ignore_suppressions: bool` to `Config`, `ConfigBuilder`, and `Commands::Lint`. In `linter.rs`, all suppression filtering (file-level, line-level, block-level, unused-suppression) is skipped when this flag is true. CLI parsing test added.

- [x] **`check` subcommand alias**
  - Ruff uses `ruff check` as the primary linting command
  - **Goal**: `lint check .` is an alias for `lint lint .`
  - **Status**: Added `#[command(alias = "check")]` to the `Lint` variant in `Commands`. CLI parsing test added.

- [x] **GitLab Code Quality output format**
  - Ruff supports GitLab Code Quality report format for CI integration
  - **Goal**: `lint lint . --output gitlab` produces a GitLab-compatible JSON report
  - **Status**: Added `Gitlab` variant to `OutputFormat`. `render_gitlab()` generates JSON array with `description`, `check_name`, `fingerprint`, `severity`, and `location` fields. CLI parsing test added.

- [x] **Additional generic rules (batch 2)**
  - **Goal**: Add `final-newline` and `no-mixed-line-endings` rules
  - **Status**: Added `FinalNewlineRule` (detects missing trailing newline, includes fix) and `NoMixedLineEndingsRule` (detects mixed CRLF/LF). Both added to default enabled rules and `linter.rs` rule set. Unit tests added. Integration test updated.

- [x] **`init` subcommand**
  - Ruff supports `ruff .` with implicit config generation, but explicit `init` is common in tools like ESLint (`eslint --init`)
  - **Goal**: `lint init` generates a default `.lint.json` in the current directory
  - **Status**: Added `Init` to `Commands`. `init_config()` writes a pretty-printed default config JSON. If `.lint.json` already exists, prints a message and exits cleanly. CLI parsing test added.

- [x] **`--output-format` alias**
  - Ruff uses `--output-format` instead of `--output`
  - **Goal**: `lint lint . --output-format json` is equivalent to `--output json`
  - **Status**: Added `visible_alias = "output-format"` to the `output` argument in `Commands::Lint`. CLI parsing test added.

- [x] **`--select-all` CLI flag**
  - Ruff supports `--select ALL` to enable all rules
  - **Goal**: `lint lint . --select-all` enables all built-in generic rules
  - **Status**: Added `select_all: bool` to `Commands::Lint`. When true, all 8 generic rule names are added to `config.rule_set.enabled_rules`. CLI parsing test added.

- [x] **Rule categories / severity-based grouping**
  - Ruff organizes rules into categories (E = errors, W = warnings, F = Pyflakes, etc.)
  - **Goal**: Rules have category prefixes (e.g., `style:line-length`, `bug:no-todo`)
  - **Status**: Added `category(&self) -> &str` to `Rule` and `LanguageRule` traits with default implementations. `LineLengthRule`, `TrailingWhitespaceRule`, `SemicolonRule` → `style`. `NoTodoRule` and all language rules → `correctness`. `list_rules()` now shows `[category]` prefix. Unit tests added for both `Rule` and `LanguageRule` categories.

## Brainstorming (from competitive intelligence round 7)

- [x] **SARIF output format**
  - Ruff and Biome support SARIF (Static Analysis Results Interchange Format) for CI integration
  - GitHub and Azure DevOps can render SARIF artifacts directly in PRs
  - **Goal**: `lint lint . --output sarif` produces SARIF JSON for upload to GitHub Advanced Security
  - **Status**: Added `Sarif` variant to `OutputFormat`. `render_sarif()` generates valid SARIF 2.1.0 JSON with tool info, rules catalog, and results with locations. `--output sarif` wired up in CLI. CLI parsing test and rendering test added.

- [x] **Stdin linting (`--stdin`, `--stdin-filename`)**
  - ESLint supports `eslint --stdin --stdin-filename=myfile.js` for piping code
  - Ruff supports `ruff check --stdin-filename myfile.py - < myfile.py`
  - **Goal**: `echo "let x = 5;" | lint lint --stdin --stdin-filename test.rs`
  - Useful for editor integrations (Vim, Emacs, VS Code)
  - **Status**: Added `stdin: bool` and `stdin_filename: Option<String>` to `Commands::Lint`. When `--stdin` is set, content is read from stdin, written to a temp file with the extension from `--stdin-filename` (for language detection), and linted. `--stdin-filename` defaults to `.rs` extension. CLI parsing tests added.

- [x] **Output to file (`--output-file`)**
  - Ruff supports `--output-file results.json` to write output to a file instead of stdout
  - **Goal**: `lint lint . --output json --output-file results.json`
  - **Status**: Added `output_file: Option<PathBuf>` to `Commands::Lint`. Refactored `print_results` into `render_results` (returns `String`) and `print_results` (prints to stdout). `run_lint_and_print` writes rendered output to file when `--output-file` is set, disabling colors automatically. CLI parsing test added.

## Brainstorming (from competitive intelligence round 6)

- [x] **`--select` / `--ignore` rule filtering at CLI**
  - Ruff uses `--select E,W` and `--ignore E501` for fine-grained rule control
  - Our current `--rules` flag replaces the entire enabled set
  - **Goal**: `--select line-length` adds a rule; `--ignore no-todo` removes a rule from defaults
  - **Status**: Added `select: Option<Vec<String>>` and `ignore: Option<Vec<String>>` to `Commands::Lint`. `--rules` still replaces the entire set. If `--rules` is not specified, `--select` appends to defaults and `--ignore` removes from defaults. Supports comma-delimited lists (e.g., `--select no-todo,no-console-log`). CLI parsing tests added.

- [x] **Config `extends` for shareable configs**
  - ESLint and Biome support extending other config files (`"extends": "./base.json"`)
  - Useful for monorepos and team-wide base configs
  - **Goal**: `{"extends": ".lint.base.json", "max_line_length": 120}` merges base + local
  - **Status**: Added `extends: Option<String>` to `Config` and `ConfigBuilder`. `load_config_file()` recursively loads base configs and calls `merge_configs()`. Scalars are overridden by local, collections are merged (ignore_patterns concatenated and deduplicated, per_file_ignores and severity_overrides merged with local winning). Relative paths resolved against config file parent directory. Unit tests added.

- [x] **`--print-files` / `--list-files` CLI flag**
  - Oxlint added `--print-files` to list files that would be linted without actually linting them
  - Useful for debugging `ignore_patterns` and glob expansion
  - **Goal**: `lint lint . --print-files` prints one path per line, then exits
  - **Status**: Added `print_files: bool` to `Commands::Lint`. `Linter::list_files()` method walks config paths, applies `is_ignored()` and `should_lint_file()` filters, and returns matching paths. `--print-files` prints paths and exits early before linting. CLI parsing test added.

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

- [x] Comprehensive unit and integration test coverage (132 lib + 55 bin + 18 advanced + 10 basic = 215 tests)
- [x] Source context in text output (offending line + caret underline)
- [x] `--statistics` CLI flag (per-rule violation counts)
- [x] `--show-fixes` CLI flag (list files modified by --fix)
- [x] `--exclude` CLI flag (exclude file patterns at CLI level)
- [x] JUnit XML output format (CI integration)
- [x] Additional generic rules (`no-empty-file`, `no-consecutive-empty-lines`, `no-tabs`, `final-newline`, `no-mixed-line-endings`)
- [x] `--show-settings` CLI flag (print effective config as JSON)
- [x] Config file auto-discovery (`find_config_file()`)
- [x] Concise output format (`--output concise`)
- [x] `--cache-location` CLI flag (custom cache path)
- [x] `--diff` CLI flag (preview fixes without writing)
- [x] `--fix-only` CLI flag (apply fixes, suppress reporting)
- [x] `--add-noqa` CLI flag (auto-add suppression comments)
- [x] `explain` subcommand (describe a rule)
- [x] `--ignore-suppressions` CLI flag (ignore suppression comments)
- [x] `check` subcommand alias for `lint`
- [x] GitLab Code Quality output format (`--output gitlab`)
- [x] `init` subcommand (generate default config)
- [x] `--output-format` alias for `--output`
- [x] `--select-all` CLI flag (enable all built-in rules)
- [x] `--no-ignore` CLI flag (disable all ignore patterns)
- [x] VCS integration (auto-respect `.gitignore` patterns)
- [x] `--cache-strategy` CLI flag (`metadata` or `content`)
- [x] `--no-error-on-unmatched-pattern` CLI flag
- [x] Rule categories (`style`, `correctness`, `custom`)
- [x] File count in text summary ("Summary: X errors, Y warnings, Z infos in N files")
- [x] Fix indicator `[*]` in text output for fixable issues
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
- [x] `--select` / `--ignore` CLI flags (add/remove rules from defaults)
- [x] `--print-files` CLI flag (list files that would be linted)
- [x] `--output-file` CLI flag (write results to file)
- [x] `--stdin` / `--stdin-filename` CLI flags (lint from stdin)
- [x] SARIF output format (`--output sarif`)
- [x] `--exit-zero` CLI flag (always return 0)
- [x] Config `extends` for shareable configs
- [x] Audit: removed unused dependencies (`glob`, `thiserror`) from `Cargo.toml`
- [x] Audit: fixed all `cargo clippy` warnings (collapsed nested ifs, replaced `len()` comparisons with `is_empty()`, simplified `map_or`)

## Brainstorming (from competitive intelligence round 8)

- [x] **`--no-ignore` CLI flag**
  - ESLint supports `--no-ignore` to disable all ignore patterns (both `.eslintignore` and `ignorePatterns` in config)
  - **Goal**: `lint lint . --no-ignore` lints all files, including those in `node_modules`, `.git`, etc.
  - **Status**: Added `no_ignore: bool` to `Commands::Lint`. When true, `config.ignore_patterns.clear()` is called after applying `--exclude`, so even explicit excludes are ignored. CLI parsing test added.

- [x] **`--no-error-on-unmatched-pattern` CLI flag**
  - ESLint supports `--no-error-on-unmatched-pattern` to avoid failing when a glob doesn't match any files
  - **Goal**: `lint lint "nonexistent/**/*.rs" --no-error-on-unmatched-pattern` exits 0 instead of erroring
  - **Status**: Added `no_error_on_unmatched_pattern: bool` to `Commands::Lint`. Our linter already silently skips non-existent paths and empty globs without erroring, so this flag is a no-op for compatibility with ESLint users. CLI parsing test added.

- [x] **VCS integration (respect `.gitignore`)**
  - Biome has VCS integration that automatically respects `.gitignore` patterns
  - **Goal**: `lint lint .` automatically skips files listed in `.gitignore` without explicit config
  - **Status**: `Linter` now parses `.gitignore` using `ignore::gitignore::GitignoreBuilder` and stores it in a `gitignore` field. `is_ignored()` checks `.gitignore` patterns in addition to `config.ignore_patterns`. The `ignore::Walk` already respected `.gitignore` for directory traversal; this adds explicit path checks. Unit test added.

- [x] **`--cache-strategy` CLI flag**
  - ESLint supports `--cache-strategy` with `metadata` (default) or `content` options
  - **Goal**: `lint lint . --cache --cache-strategy content` hashes file contents for cache keying
  - **Status**: Added `CacheStrategy` enum (`Metadata`/`Content`) to `config.rs`. Added `cache_strategy` field to `Config`, `ConfigBuilder`, and `Commands::Lint`. `lint_file()` in `linter.rs` uses `DefaultHasher` to hash content when `Content` strategy is active. `CacheEntry` now stores optional `content_hash`. `get_by_hash()` and `insert_with_hash()` added to `Cache`. CLI parsing test added.
