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

- [x] Comprehensive unit and integration test coverage (137 lib + 86 bin + 18 advanced + 10 basic = 251 tests)
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
- [x] `--deny-warnings` CLI flag
- [x] `--exit-non-zero-on-fix` CLI flag
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
- [x] `--no-cache` CLI flag (skip cache reads/writes)
- [x] `--no-config` CLI flag (ignore `.lint.json` and auto-discovered config)
- [x] `--stdin-file-path` CLI flag (alias for `--stdin-filename`, used for per-file ignores and output path)
- [x] Grouped output format (`--output grouped`)
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

## Brainstorming (from competitive intelligence round 9)

- [x] **`--deny-warnings` CLI flag**
  - oxlint supports `--deny-warnings` to ensure warnings produce a non-zero exit code
  - **Goal**: `lint lint . --deny-warnings` exits 1 if any warnings are found, even without errors
  - **Status**: Added `deny_warnings: bool` to `Commands::Lint` and `run_lint_and_print()`. When true, the exit code logic treats any warning count > 0 as a failure (exit 1), regardless of whether errors exist. CLI parsing test added.

- [x] **`--exit-non-zero-on-fix` CLI flag**
  - Ruff supports `--exit-non-zero-on-fix` to exit 1 if any violations were found, even if all were fixed
  - **Goal**: `lint lint . --fix --exit-non-zero-on-fix` exits 1 when fixes were applied
  - **Status**: Added `exit_non_zero_on_fix: bool` to `Commands::Lint` and `run_lint_and_print()`. When true, the exit code is 1 if any fixes were applied during the run, useful for CI enforcing that fixed code must be committed. CLI parsing test added.

## Brainstorming (from competitive intelligence round 10)

- [x] **`--no-cache` CLI flag**
  - Ruff and oxlint support `--no-cache` to explicitly disable caching even if a config file enables it
  - **Goal**: `lint lint . --no-cache` skips cache reads/writes entirely
  - Useful for debugging or when cache state is suspected to be stale
  - **Status**: Added `no_cache: bool` to `Commands::Lint`. When true, cache is set to `None` so no cache file is loaded or saved. CLI parsing test added.

- [x] **`--no-config` CLI flag**
  - ESLint supports `--no-eslintrc` to skip config file discovery and use only CLI-provided settings
  - **Goal**: `lint lint . --no-config` ignores `.lint.json` and any auto-discovered config
  - Useful for quick one-off runs with only CLI arguments
  - **Status**: Added `no_config: bool` to `Commands::Lint`. When true, `ConfigBuilder::new().build()` is used directly, bypassing `--config` and auto-discovery. CLI parsing test added.

- [x] **`--stdin-file-path` CLI flag**
  - Biome and Ruff use `--stdin-file-path` to determine language and apply per-file overrides when linting stdin
  - **Goal**: `echo '...' | lint lint --stdin --stdin-file-path foo.rs` treats the input as Rust for rule selection
  - **Status**: Added `visible_alias = "stdin-file-path"` to the existing `--stdin-filename` argument. Added `stdin_file_path` to `Config` and `ConfigBuilder`. `Linter::effective_path()` replaces temp path with `stdin_file_path` for `LintResult.file_path` and `per_file_ignores` matching when the temp file name starts with `lint_stdin_`. CLI parsing and linter unit tests added.

- [x] **Grouped output format**
  - Ruff supports `--output-format=grouped` which groups violations by file path instead of a flat list
  - **Goal**: `lint lint . --output grouped` shows all issues per file together
  - Useful when scanning long output to focus on one file at a time
  - **Status**: Added `Grouped` variant to `OutputFormat`. `render_grouped()` groups messages by file path with bold filename headers, colored severity prefixes, line/column numbers, messages, and dimmed rule names. Clean files are skipped. CLI parsing test and rendering test added.

## Brainstorming (from competitive intelligence round 11)

- [x] **`--check` CLI flag**
  - Prettier supports `--check` to verify files are already formatted without writing changes
  - **Goal**: `lint lint . --check` exits 1 if any fixable violations exist, without modifying files
  - Useful for CI pipelines that want to enforce clean code without auto-fixing
  - **Status**: Added `check: bool` to `Commands::Lint`. When true, fixes are applied in-memory (like `--diff`) but not written to disk. A summary "Would fix N file(s)" is printed. Exit code is 1 if any fixes would be applied. CLI parsing test and behavior test added.

- [x] **Auto-fixes for remaining generic rules**
  - Currently only `trailing-whitespace`, `no-tabs`, and `final-newline` have auto-fixes
  - `no-consecutive-empty-lines`, `no-mixed-line-endings`, and `no-empty-file` lack fixes
  - **Goal**: All generic rules that can be safely auto-fixed emit `suggestion` and are applied by `--fix`
  - **Status**: Extended `apply_fixes()` to support full-content fixes via `line == 0` sentinel. Added `collapse_empty_lines()` helper. `NoConsecutiveEmptyLinesRule` now emits a full-content fix that collapses consecutive empty lines. `NoMixedLineEndingsRule` now emits a full-content fix that normalizes CRLF to LF. `NoEmptyFileRule` intentionally has no auto-fix (deleting a file or adding arbitrary content is not safe). Unit tests for fixes and full-content fix application added.

- [x] **`--max-diagnostics` CLI flag**
  - Ruff supports `--max-diagnostics N` to limit the number of violations shown per file or total
  - **Goal**: `lint lint . --max-diagnostics 50` truncates output after 50 messages with a "... N more" hint
  - Useful for large files with thousands of violations (e.g., generated code)
  - **Status**: Added `max_diagnostics: Option<usize>` to `Commands::Lint`. Extracted `truncate_diagnostics()` helper that limits total diagnostics across all files. When exceeded, prints `... N additional diagnostics hidden ...` after output. CLI parsing test and truncation behavior tests added.

- [x] **TOML configuration file support**
  - Biome and many Rust tools support `.biome.toml` in addition to JSON config
  - **Goal**: `lint lint .` auto-discovers `.lint.toml` and parses it with the same schema as `.lint.json`
  - TOML is more readable and comment-friendly than JSON for configuration
  - **Status**: Added `toml = "0.8"` dependency. `load_config_file()` detects `.toml` extension and uses `toml::from_str()` instead of `serde_json::from_str()`. Added `#[serde(default)]` to `Config` fields and `Default` impls for `CacheStrategy`, `RuleSetConfig`, and `OutputFormat` so TOML partial configs work. `find_config_file()` now searches for `.lint.toml` after `.lint.json`. Tests for loading and discovery added.

- [x] **Baseline file support**
  - Ruff supports `--baseline` to compare against a previous run and only show new issues
  - **Goal**: `lint lint . --baseline baseline.json` only reports violations not present in the baseline
  - Critical for incremental adoption in large legacy codebases
  - **Status**: Added `baseline: Option<PathBuf>` to `Commands::Lint`. `BaselineEntry` struct tracks file path, line, and rule for matching. `load_baseline()` parses JSON baseline files. `filter_baseline()` removes current violations that exactly match baseline entries (same file path, line, and rule). Applied after fixes but before rendering. CLI parsing test and filtering behavior tests added.

## Brainstorming (from competitive intelligence round 12)

- [x] **Safe/unsafe fix classification**
  - Ruff separates fixes into `safe` and `unsafe` categories
  - **Goal**: `--fix` only applies safe fixes; `--fix --unsafe-fixes` applies all fixes
  - Critical for CI automation where unsafe fixes could break code semantics
  - **Status**: Added `is_safe: bool` to `Fix` struct with serde default `true`. Added `--unsafe-fixes` CLI flag. In `run_lint_and_print()`, when `--fix` is used without `--unsafe-fixes`, unsafe fixes are filtered out from messages before applying. All existing fixes default to safe. CLI parsing test and filtering behavior test added.

- [x] **`--force-exclude` CLI flag**
  - Ruff supports `--force-exclude` to skip files even when explicitly passed as arguments
  - **Goal**: `lint lint node_modules/file.js --force-exclude` respects ignore patterns and skips the file
  - Useful when shell globbing accidentally matches ignored directories
  - **Status**: Added `force_exclude: bool` to `Config`, `ConfigBuilder`, and `Commands::Lint`. In `linter.rs`, `lint_path()` checks `is_ignored()` for explicitly passed files when `force_exclude` is true. CLI parsing test and behavior tests added.

- [ ] **Git-aware linting (`--changed`, `--staged`)**
  - Biome supports `--changed` and `--staged` to only lint files modified in git
  - **Goal**: `lint lint . --changed` only lints files with uncommitted changes; `--staged` only lints staged files
  - Dramatically speeds up pre-commit hooks and CI on large monorepos

- [x] **`--ext` CLI flag**
  - ESLint uses `--ext .js,.jsx,.ts` to filter which file extensions to lint
  - **Goal**: `lint lint . --ext rs,toml` only processes files with matching extensions
  - Useful when walking directories with many non-source files
  - **Status**: Added `ext: Option<Vec<String>>` to `Config`, `ConfigBuilder`, and `Commands::Lint`. Modified `should_lint_file()` to use configured extensions when `ext` is set, falling back to the hardcoded supported extensions list otherwise. CLI parsing test and behavior test added.

- [x] **`--verbose` / `--log-level` CLI flag**
  - Biome and Ruff support `--verbose` and `--log-level` for debugging config loading, file discovery, cache behavior
  - **Goal**: `lint lint . --verbose` prints debug info: config file found, cache hits/misses, files skipped and why
  - Essential for troubleshooting why a file is or isn't being linted
  - **Status**: Added `verbose: bool` to `Commands::Lint` and `run_lint_and_print()`. When true, prints file count, diagnostic count, cache status, and fix mode status before rendering results. CLI parsing test added.

- [ ] **Plugin system for custom rules**
  - Oxlint supports `--import-plugin`, `--jsdoc-plugin`, etc. for extending rule sets
  - **Goal**: Load additional rule packs as plugins (e.g., `--plugin security`, `--plugin react`)
  - Would enable ecosystem-specific rules without bloating the core binary

## Brainstorming (from competitive intelligence round 13)

- [x] **`--show-source` toggle**
  - Currently text output always shows source context (offending line + caret underline)
  - Some users may want cleaner output without source snippets
  - **Goal**: `--no-show-source` disables source context in text output
  - Simple to implement: add `show_source: bool` to config, thread through `render_results()`
  - **Status**: See round 14 for implementation details.

- [x] **`--line-length` CLI alias**
  - Ruff uses `--line-length N`; we only have `--max-line-length N`
  - **Goal**: `--line-length 120` is an alias for `--max-line-length 120`
  - Very simple: add `line_length` to CLI with `visible_alias = "max-line-length"` or duplicate handler
  - **Status**: Added `visible_alias = "line-length"` to the `max_line_length` CLI argument via clap. `--line-length 120` now parses identically to `--max-line-length 120`. CLI parsing test added.

- [x] **`--no-gitignore` CLI flag**
  - Currently `.gitignore` files are always respected via the `ignore` crate's `WalkBuilder`
  - **Goal**: `--no-gitignore` disables automatic .gitignore respect
  - Useful when you want to lint generated files that are gitignored
  - **Status**: Added `no_gitignore: bool` to `Config`, `ConfigBuilder`, and `Commands::Lint`. `WalkBuilder::git_ignore()` toggled in `linter.rs`. `GitignoreBuilder` skipped when `no_gitignore` is true. CLI parsing and behavior tests added.

- [x] **`--fixable` / `--unfixable` CLI flags**
  - Ruff supports `--fixable E,W` and `--unfixable I` to control which rules can be auto-fixed
  - **Goal**: `--fixable trailing-whitespace` only auto-fixes that rule; `--unfixable line-length` excludes it
  - Simple extension of the existing fix filtering logic
  - **Status**: Added `fixable: Option<Vec<String>>` and `unfixable: Option<Vec<String>>` to `Commands::Lint`. In `run_lint_and_print()`, when `--fixable` is used, only fixes for listed rules are retained; when `--unfixable` is used, fixes for listed rules are removed. Applied after unsafe fix filtering but before fix application. CLI parsing tests and behavior test added.

- [x] **`--preview` CLI flag**
  - Ruff uses `--preview` to enable experimental/nightly rules
  - **Goal**: `--preview` enables rules marked as experimental in the rule registry
  - Simple boolean flag that gates certain rules; future-proof for adding new rules
  - **Status**: Added `preview: bool` to `Config`, `ConfigBuilder`, and `Commands::Lint`. `merge_configs()` merges with OR logic (either config or CLI enables preview). `test_config_serialization` updated. CLI parsing test added.

## Brainstorming (from competitive intelligence round 14)

- [x] **`--show-source` toggle**
  - Currently text output always shows source context (offending line + caret underline)
  - Some users may want cleaner output without source snippets
  - **Goal**: `--no-show-source` disables source context in text output
  - Simple to implement: add `show_source: bool` to config, thread through `render_results()`
  - **Status**: Added `show_source: bool` to `Config` and `ConfigBuilder` with serde default `true`. Added `--no-show-source` CLI flag. Threaded `show_source` through `render_results()` and `print_results()`; source context and caret underline are conditionally rendered in `OutputFormat::Text`. `merge_configs()` uses AND logic (both must be true to show source). CLI parsing test added.

- [x] **`--no-gitignore` CLI flag**
  - Currently `.gitignore` files are always respected via the `ignore` crate's `WalkBuilder`
  - **Goal**: `--no-gitignore` disables automatic .gitignore respect
  - Useful when you want to lint generated files that are gitignored
  - **Status**: Added `no_gitignore: bool` to `Config`, `ConfigBuilder`, and `Commands::Lint`. `WalkBuilder::git_ignore()` toggled in `linter.rs`. `GitignoreBuilder` skipped when `no_gitignore` is true. CLI parsing and behavior tests added.

- [ ] **Git-aware linting (`--changed`, `--staged`)**
  - Biome supports `--changed` and `--staged` to only lint files modified in git
  - **Goal**: `lint lint . --changed` only lints files with uncommitted changes; `--staged` only lints staged files
  - Dramatically speeds up pre-commit hooks and CI on large monorepos
  - Requires adding `git2` or shelling out to `git status`

- [ ] **Plugin system for custom rules**
  - Oxlint supports `--import-plugin`, `--jsdoc-plugin`, etc. for extending rule sets
  - **Goal**: Load additional rule packs as plugins (e.g., `--plugin security`, `--plugin react`)
  - Would enable ecosystem-specific rules without bloating the core binary
  - Complex: requires dynamic loading or conditional compilation of rule modules

## Optimization round (performance pass)

- [x] **Bundle `run_lint_and_print` booleans into `RunOptions` struct**
  - Eliminated 15+ positional boolean parameters
  - Removed `#[allow(clippy::too_many_arguments)]`
- [x] **Reduce clones in hot paths**
  - `filter_baseline`: removed `.to_string()` on `to_string_lossy()`
  - `render_results`/`print_results`: take `&OutputFormat` instead of owned
  - `apply_fixes`: early-return without collecting fixes when none exist; avoid `raw_messages.clone()` in suppression handling
  - `statistics`: `HashMap<&str, usize>` instead of `HashMap<String, usize>`
  - `add_noqa`: short-circuit files with no messages
  - `diff mode`: only clone file content when `opts.diff` is true
  - Text renderer: `lines().collect()` moved outside per-message loop
  - JSON output: serialize slice directly when not quiet
- [x] **Cache fast path for content strategy**
  - Check metadata-based cache before reading file even when `cache_strategy == Content`
  - Avoids disk reads when mtime/size haven't changed
- [x] **O(1) ignore pattern lookups**
  - Replaced nested loops in `is_ignored()` with `HashSet<String>` stored in `Linter`
- [x] **Compile regexes once**
  - `NoTodoRule`: changed from unit struct to struct with pre-compiled `Regex`
  - All 18 language rules: converted on-the-fly `Regex::new` to `std::sync::LazyLock` statics
- [x] **Dead code removal**
  - Removed unused `create_default_config()` from `lib.rs`
- [x] **Clean clippy warnings**
  - `large_enum_variant` suppressed on `Commands` enum (CLI-only, constructed once)
  - `collapsible_if` suppressed where let-chains are needed
