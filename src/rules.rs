use crate::output::{LintMessage, Severity};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::sync::LazyLock;

pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn code(&self) -> &str {
        code_from_name(self.name())
    }
    fn category(&self) -> &str {
        "style"
    }
    fn description(&self) -> &str {
        ""
    }
    fn has_fix(&self) -> bool {
        false
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn url(&self) -> &str {
        ""
    }
    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage>;
}

/// Maps a built-in rule name to its short letter-number code.
/// Returns the name itself as a fallback for unknown/custom rules.
pub fn code_from_name(name: &str) -> &str {
    match name {
        "line-length" => "W001",
        "trailing-whitespace" => "W002",
        "no-todo" => "W003",
        "no-empty-file" => "E001",
        "no-consecutive-empty-lines" => "W004",
        "no-tabs" => "W005",
        "final-newline" => "W006",
        "no-mixed-line-endings" => "W007",
        "hardcoded-secret" => "S001",
        "unsafe-eval" => "S002",
        "sql-injection-risk" => "S003",
        "max-nesting-depth" => "E002",
        "max-function-lines" => "E003",
        "sort-imports" => "W008",
        // Language-specific
        "no-console-log" => "L001",
        "no-var" => "L002",
        "no-print" => "L003",
        "python-style" => "L004",
        "go-style" => "L005",
        "java-style" => "L006",
        "no-unwrap" => "L007",
        "no-expect" => "L008",
        "missing-semicolon" => "L009",
        "no-puts" => "L010",
        "ruby-style" => "L011",
        "no-echo" => "L012",
        "no-swift-print" => "L013",
        "kotlin-style" => "L014",
        "no-dart-print" => "L015",
        "no-csharp-console" => "L016",
        "csharp-style" => "L017",
        "shell-echo-quote" => "L018",
        "sql-no-select-star" => "L019",
        "no-lua-print" => "L020",
        "no-scala-println" => "L021",
        "no-r-print" => "L022",
        "no-zig-debug-print" => "L023",
        "html-no-inline-style" => "L024",
        "html-img-alt" => "L025",
        "css-avoid-important" => "L026",
        _ => name,
    }
}

/// Maps a short letter-number code back to the canonical rule name.
/// Returns the code itself as a fallback for unknown codes.
pub fn name_from_code(code: &str) -> &str {
    match code {
        "W001" => "line-length",
        "W002" => "trailing-whitespace",
        "W003" => "no-todo",
        "E001" => "no-empty-file",
        "W004" => "no-consecutive-empty-lines",
        "W005" => "no-tabs",
        "W006" => "final-newline",
        "W007" => "no-mixed-line-endings",
        "S001" => "hardcoded-secret",
        "S002" => "unsafe-eval",
        "S003" => "sql-injection-risk",
        "E002" => "max-nesting-depth",
        "E003" => "max-function-lines",
        "W008" => "sort-imports",
        "L001" => "no-console-log",
        "L002" => "no-var",
        "L003" => "no-print",
        "L004" => "python-style",
        "L005" => "go-style",
        "L006" => "java-style",
        "L007" => "no-unwrap",
        "L008" => "no-expect",
        "L009" => "missing-semicolon",
        "L010" => "no-puts",
        "L011" => "ruby-style",
        "L012" => "no-echo",
        "L013" => "no-swift-print",
        "L014" => "kotlin-style",
        "L015" => "no-dart-print",
        "L016" => "no-csharp-console",
        "L017" => "csharp-style",
        "L018" => "shell-echo-quote",
        "L019" => "sql-no-select-star",
        "L020" => "no-lua-print",
        "L021" => "no-scala-println",
        "L022" => "no-r-print",
        "L023" => "no-zig-debug-print",
        "L024" => "html-no-inline-style",
        "L025" => "html-img-alt",
        "L026" => "css-avoid-important",
        _ => code,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomRuleDefinition {
    pub name: String,
    pub pattern: String,
    pub message: String,
    pub severity: String,
    pub suggestion: Option<String>,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CustomRule {
    definition: CustomRuleDefinition,
    regex: Regex,
}

impl CustomRule {
    pub fn from_definition(def: CustomRuleDefinition) -> Result<Self, regex::Error> {
        let regex = Regex::new(&def.pattern)?;
        Ok(Self {
            definition: def,
            regex,
        })
    }
}

impl Rule for CustomRule {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn category(&self) -> &str {
        "custom"
    }

    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage> {
        if let Some(ref extensions) = self.definition.extensions {
            let ext_matches = file_path.extension().is_some_and(|ext| {
                let ext_str = ext.to_string_lossy();
                extensions.iter().any(|e| e == ext_str.as_ref())
            });
            if !ext_matches {
                return Vec::new();
            }
        }

        let mut messages = Vec::new();
        let severity = match self.definition.severity.as_str() {
            "Error" => Severity::Error,
            "Warning" => Severity::Warning,
            _ => Severity::Info,
        };

        for (line_num, line) in content.lines().enumerate() {
            if self.regex.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    0,
                    severity,
                    self.definition.message.clone(),
                    self.definition.name.clone(),
                    self.definition.suggestion.clone(),
                ));
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct LineLengthRule {
    pub max_length: usize,
}

impl Rule for LineLengthRule {
    fn name(&self) -> &str {
        "line-length"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Lines exceeding the configured maximum length."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.len() > self.max_length {
                messages.push(LintMessage::new(
                    line_num + 1,
                    self.max_length + 1,
                    self.default_severity(),
                    format!("Line exceeds maximum length of {} characters", self.max_length),
                    self.name().to_string(),
                    Some(format!(
                        "Break the line: extract to a variable, use line continuation (\\), or split at natural boundaries (e.g., after commas, operators). Max {} chars.",
                        self.max_length
                    )),
                ));
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct TrailingWhitespaceRule;

impl Rule for TrailingWhitespaceRule {
    fn name(&self) -> &str {
        "trailing-whitespace"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Trailing whitespace at the end of lines."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed = line.trim_end();
                let message = LintMessage::new(
                    line_num + 1,
                    line.len(),
                    self.default_severity(),
                    "Trailing whitespace detected".to_string(),
                    self.name().to_string(),
                    Some("Delete spaces/tabs at end of line. In most editors: place cursor at line end and press Backspace until clean.".to_string()),
                )
                .with_fix(trimmed.to_string());
                messages.push(message);
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct NoTodoRule {
    regex: Regex,
}

impl Default for NoTodoRule {
    fn default() -> Self {
        Self {
            regex: Regex::new(r"(?i)\b(TODO|FIXME|HACK)\b").unwrap(),
        }
    }
}

impl Rule for NoTodoRule {
    fn name(&self) -> &str {
        "no-todo"
    }

    fn category(&self) -> &str {
        "correctness"
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if self.regex.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find(|c: char| c.is_ascii_alphanumeric()).unwrap_or(0),
                    self.default_severity(),
                    "TODO/FIXME comment found".to_string(),
                    self.name().to_string(),
                    Some("Create a tracking issue and replace with issue reference, or implement the fix and remove the comment.".to_string()),
                ));
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct NoEmptyFileRule;

impl Rule for NoEmptyFileRule {
    fn name(&self) -> &str {
        "no-empty-file"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Files that are completely empty or contain only whitespace."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            vec![LintMessage::new(
                1,
                1,
                self.default_severity(),
                "Empty file detected".to_string(),
                self.name().to_string(),
                Some("Add content or delete the file if it is unused.".to_string()),
            )]
        } else {
            Vec::new()
        }
    }
}

fn collapse_empty_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut prev_empty = false;

    for line in lines {
        let is_empty = line.trim().is_empty();
        if is_empty && prev_empty {
            continue;
        }
        result.push(line);
        prev_empty = is_empty;
    }

    let mut output = result.join("\n");
    if content.ends_with('\n') || content.ends_with("\r\n") {
        output.push('\n');
    }
    output
}

#[derive(Debug, Clone)]
pub struct NoConsecutiveEmptyLinesRule;

impl Rule for NoConsecutiveEmptyLinesRule {
    fn name(&self) -> &str {
        "no-consecutive-empty-lines"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "More than one consecutive empty line."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut prev_empty = false;
        let mut had_consecutive = false;

        for (i, line) in lines.iter().enumerate() {
            let is_empty = line.trim().is_empty();
            if is_empty && prev_empty {
                messages.push(LintMessage::new(
                    i + 1,
                    1,
                    self.default_severity(),
                    "Consecutive empty lines detected".to_string(),
                    self.name().to_string(),
                    Some("Remove extra blank lines; files should not contain more than one consecutive empty line.".to_string()),
                ));
                had_consecutive = true;
            }
            prev_empty = is_empty;
        }

        if had_consecutive {
            let fixed = collapse_empty_lines(content);
            if let Some(first) = messages.first_mut() {
                first.fix = Some(crate::output::Fix {
                    line: 0,
                    replacement: fixed,
                    is_safe: true,
                });
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct NoTabsRule;

impl Rule for NoTabsRule {
    fn name(&self) -> &str {
        "no-tabs"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Tab characters used for indentation. Use spaces instead."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(col) = line.find('\t') {
                let fixed = line.replace('\t', "    ");
                messages.push(
                    LintMessage::new(
                        line_num + 1,
                        col + 1,
                        self.default_severity(),
                        "Tab character detected; use spaces for indentation".to_string(),
                        self.name().to_string(),
                        Some("Replace tabs with spaces. Most editors support 'Insert spaces instead of tabs' in settings.".to_string()),
                    )
                    .with_fix(fixed),
                );
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct FinalNewlineRule;

impl Rule for FinalNewlineRule {
    fn name(&self) -> &str {
        "final-newline"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Files that do not end with a newline character."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        if content.is_empty() {
            return Vec::new();
        }
        if !content.ends_with('\n') {
            vec![
                LintMessage::new(
                    content.lines().count().max(1),
                    1,
                    self.default_severity(),
                    "File does not end with a newline".to_string(),
                    self.name().to_string(),
                    Some("Add a final newline at the end of the file.".to_string()),
                )
                .with_fix("\n".to_string()),
            ]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoMixedLineEndingsRule;

impl Rule for NoMixedLineEndingsRule {
    fn name(&self) -> &str {
        "no-mixed-line-endings"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Mixed CRLF and LF line endings in the same file."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut has_crlf = false;
        let mut has_lf = false;
        // Scan raw bytes for line endings
        let bytes = content.as_bytes();
        for i in 0..bytes.len() {
            if bytes[i] == b'\r' {
                has_crlf = true;
            } else if bytes[i] == b'\n' && (i == 0 || bytes[i - 1] != b'\r') {
                has_lf = true;
            }
        }
        if has_crlf && has_lf {
            let fixed = content.replace("\r\n", "\n");
            vec![{
                let mut msg = LintMessage::new(
                    1,
                    1,
                    self.default_severity(),
                    "Mixed line endings detected (both CRLF and LF)".to_string(),
                    self.name().to_string(),
                    Some("Use consistent line endings. Prefer LF (\n) for cross-platform compatibility.".to_string()),
                );
                msg.fix = Some(crate::output::Fix {
                    line: 0,
                    replacement: fixed,
                    is_safe: true,
                });
                msg
            }]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaxNestingDepthRule {
    pub max_depth: usize,
}

impl Rule for MaxNestingDepthRule {
    fn name(&self) -> &str {
        "max-nesting-depth"
    }

    fn category(&self) -> &str {
        "correctness"
    }

    fn description(&self) -> &str {
        "Code blocks nested deeper than the configured maximum depth."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let mut depth: isize = 0;
        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            // Count brace closings before measuring depth for this line
            let closings = line.chars().filter(|c| *c == '}').count() as isize;
            depth -= closings;
            if depth < 0 {
                depth = 0;
            }
            if depth > self.max_depth as isize {
                messages.push(LintMessage::new(
                    line_num,
                    1,
                    self.default_severity(),
                    format!(
                        "Nesting depth of {} exceeds maximum {}",
                        depth, self.max_depth
                    ),
                    self.name().to_string(),
                    Some(
                        "Refactor to reduce nesting: extract into functions or use early returns."
                            .to_string(),
                    ),
                ));
            }
            // Count brace openings after measuring depth for this line
            let openings = line.chars().filter(|c| *c == '{').count() as isize;
            depth += openings;
        }
        messages
    }
}

#[derive(Debug, Clone)]
pub struct MaxFunctionLinesRule {
    pub max_lines: usize,
}

impl Rule for MaxFunctionLinesRule {
    fn name(&self) -> &str {
        "max-function-lines"
    }

    fn category(&self) -> &str {
        "correctness"
    }

    fn description(&self) -> &str {
        "Functions that exceed the configured maximum line count."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            // Detect function definitions across common languages
            // Pattern: function name(...), fn name(...), def name(...), etc.
            let is_func_start = line.trim().starts_with("fn ")
                || line.trim().starts_with("function ")
                || line.trim().starts_with("def ")
                || line.trim().starts_with("func ")
                || (line.contains("(")
                    && line.contains(")")
                    && line.contains("{")
                    && !line.trim().starts_with("if")
                    && !line.trim().starts_with("for")
                    && !line.trim().starts_with("while")
                    && !line.trim().starts_with("switch")
                    && !line.trim().starts_with("match"));
            if is_func_start {
                let start_line = i + 1;
                let mut brace_depth: isize = 0;
                let mut started = false;
                let mut end_line = i;
                for (j, l) in lines.iter().enumerate().skip(i) {
                    for c in l.chars() {
                        if c == '{' {
                            brace_depth += 1;
                            started = true;
                        } else if c == '}' {
                            brace_depth -= 1;
                        }
                    }
                    end_line = j;
                    if started && brace_depth <= 0 {
                        break;
                    }
                }
                let func_len = end_line - i + 1;
                if func_len > self.max_lines {
                    messages.push(LintMessage::new(
                        start_line,
                        1,
                        self.default_severity(),
                        format!(
                            "Function spans {} lines, exceeding maximum {}",
                            func_len, self.max_lines
                        ),
                        self.name().to_string(),
                        Some(
                            "Refactor: extract helper functions to reduce complexity.".to_string(),
                        ),
                    ));
                }
                i = end_line + 1;
            } else {
                i += 1;
            }
        }
        messages
    }
}

#[derive(Debug, Clone)]
pub struct SortImportsRule;

impl Rule for SortImportsRule {
    fn name(&self) -> &str {
        "sort-imports"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Import/use statements that are not in alphabetical order."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            if Self::is_import_line(lines[i]) {
                // Find consecutive import block
                let start = i;
                while i < lines.len() && Self::is_import_line(lines[i]) {
                    i += 1;
                }
                let end = i;
                // Check if block has 2+ lines and is sorted
                if end - start >= 2 {
                    let mut prev_key = String::new();
                    for (j, line) in lines.iter().enumerate().take(end).skip(start) {
                        let key = Self::extract_sort_key(line);
                        if !prev_key.is_empty() && key < prev_key {
                            messages.push(LintMessage::new(
                                j + 1,
                                1,
                                self.default_severity(),
                                format!(
                                    "Import '{}' is out of alphabetical order (should come before '{}')",
                                    key, prev_key
                                ),
                                self.name().to_string(),
                                Some("Reorder imports alphabetically for consistency.".to_string()),
                            ));
                        }
                        prev_key = key;
                    }
                }
            } else {
                i += 1;
            }
        }

        // If there are violations, attach a full-file fix to the first message
        if !messages.is_empty() {
            let fixed = Self::sort_all_imports(content);
            messages[0].fix = Some(crate::output::Fix {
                line: 0,
                replacement: fixed,
                is_safe: true,
            });
        }

        messages
    }
}

impl SortImportsRule {
    fn is_import_line(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("#include ")
            || trimmed.contains("require(")
    }

    fn extract_sort_key(line: &str) -> String {
        let trimmed = line.trim();
        if let Some(after) = trimmed.strip_prefix("from ") {
            // Python: from os import path → key: "os"
            if let Some(pos) = after.find(" import") {
                return after[..pos].trim().to_lowercase();
            }
        }
        if let Some(after) = trimmed.strip_prefix("use ") {
            // Rust: use std::fs; → key: "std::fs"
            return after.trim_end_matches(';').trim().to_lowercase();
        }
        if let Some(after) = trimmed.strip_prefix("import ") {
            // JS: import { foo } from 'bar'; → key: "bar" (from clause)
            if let Some(from_pos) = trimmed.rfind(" from ") {
                let after_from = &trimmed[from_pos + 6..];
                let cleaned = after_from.trim().trim_end_matches(';');
                return cleaned
                    .trim_matches(|c| c == '\'' || c == '"')
                    .to_lowercase();
            }
            // Python: import os, sys → key: "os"
            return after
                .split(',')
                .next()
                .unwrap_or(after)
                .trim()
                .to_lowercase();
        }
        if let Some(after) = trimmed.strip_prefix("#include ") {
            return after.trim().to_lowercase();
        }
        if let Some(pos) = trimmed.find("require(") {
            let after = &trimmed[pos + 8..];
            if let Some(end) = after.find(')') {
                let inner = &after[..end];
                return inner.trim_matches(|c| c == '\'' || c == '"').to_lowercase();
            }
        }
        trimmed.to_lowercase()
    }

    /// Returns a copy of `content` with every import block sorted alphabetically.
    fn sort_all_imports(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            if Self::is_import_line(lines[i]) {
                let start = i;
                while i < lines.len() && Self::is_import_line(lines[i]) {
                    i += 1;
                }
                let end = i;
                if end - start >= 2 {
                    let mut block: Vec<(&str, String)> = lines[start..end]
                        .iter()
                        .map(|&l| (l, Self::extract_sort_key(l)))
                        .collect();
                    block.sort_by(|a, b| a.1.cmp(&b.1));
                    for (line, _) in block {
                        result.push(line);
                    }
                } else {
                    result.extend(&lines[start..end]);
                }
            } else {
                result.push(lines[i]);
                i += 1;
            }
        }
        let mut output = result.join("\n");
        if content.ends_with('\n') || content.ends_with("\r\n") {
            output.push('\n');
        }
        output
    }
}

#[derive(Debug, Clone)]
pub struct HardcodedSecretRule;

static SECRET_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r##"(?i)(password|passwd|secret|api_key|apikey|token|auth)\s*[=:]\s*['"][^'"]+['"]"##,
    )
    .unwrap()
});

impl Rule for HardcodedSecretRule {
    fn name(&self) -> &str {
        "hardcoded-secret"
    }

    fn category(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects hardcoded credentials such as passwords, API keys, and tokens in source code."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SECRET_PATTERN;
        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                let col = line
                    .to_lowercase()
                    .find("password")
                    .or_else(|| line.to_lowercase().find("secret"))
                    .or_else(|| line.to_lowercase().find("api_key"))
                    .or_else(|| line.to_lowercase().find("apikey"))
                    .or_else(|| line.to_lowercase().find("token"))
                    .or_else(|| line.to_lowercase().find("auth"))
                    .unwrap_or(0);
                messages.push(LintMessage::new(
                    line_num + 1,
                    col,
                    self.default_severity(),
                    "Hardcoded secret detected".to_string(),
                    self.name().to_string(),
                    Some("Use environment variables or a secrets manager instead of hardcoding credentials".to_string()),
                ));
            }
        }
        messages
    }
}

#[derive(Debug, Clone)]
pub struct UnsafeEvalRule;

static EVAL_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\beval\s*\(").unwrap());

impl Rule for UnsafeEvalRule {
    fn name(&self) -> &str {
        "unsafe-eval"
    }

    fn category(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects unsafe eval() calls which can lead to code injection vulnerabilities."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "js" | "ts" | "jsx" | "tsx") {
            return messages;
        }
        let pattern = &*EVAL_PATTERN;
        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("eval").unwrap_or(0),
                    self.default_severity(),
                    "Unsafe eval() call detected".to_string(),
                    self.name().to_string(),
                    Some("Avoid eval(). Use JSON.parse for JSON data, or structured parsing for other formats".to_string()),
                ));
            }
        }
        messages
    }
}

#[derive(Debug, Clone)]
pub struct SqlInjectionRiskRule;

static SQL_INJECTION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r##"(?i)(SELECT|INSERT|UPDATE|DELETE).*\+.*\$|['"].*\$\{|['"].*\+.*\+"##).unwrap()
});

impl Rule for SqlInjectionRiskRule {
    fn name(&self) -> &str {
        "sql-injection-risk"
    }

    fn category(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects potential SQL injection risks from string concatenation or interpolation in queries."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SQL_INJECTION_PATTERN;
        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line)
                && !line.trim_start().starts_with("//")
                && !line.trim_start().starts_with("#")
                && !line.trim_start().starts_with("--")
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.to_lowercase().find("select").or_else(|| line.to_lowercase().find("insert")).or_else(|| line.to_lowercase().find("update")).or_else(|| line.to_lowercase().find("delete")).unwrap_or(0),
                    self.default_severity(),
                    "Potential SQL injection risk from string concatenation".to_string(),
                    self.name().to_string(),
                    Some("Use parameterized queries or prepared statements instead of string concatenation".to_string()),
                ));
            }
        }
        messages
    }
}

pub struct RuleSet {
    pub rules: Vec<Box<dyn Rule>>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(mut self, rule: Box<dyn Rule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn get_rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the list of rule names provided by a built-in plugin.
pub fn plugin_rules(plugin: &str) -> Vec<String> {
    match plugin {
        "security" => vec![
            "hardcoded-secret".to_string(),
            "unsafe-eval".to_string(),
            "sql-injection-risk".to_string(),
        ],
        "javascript" => vec![
            "no-console-log".to_string(),
            "no-var".to_string(),
            "missing-semicolon".to_string(),
        ],
        "python" => vec!["no-print".to_string(), "python-style".to_string()],
        "rust" => vec![
            "no-unwrap".to_string(),
            "no-expect".to_string(),
            "missing-semicolon".to_string(),
        ],
        "html" => vec![
            "html-no-inline-style".to_string(),
            "html-img-alt".to_string(),
        ],
        "css" => vec!["css-avoid-important".to_string()],
        _ => Vec::new(),
    }
}

/// Returns all rule names belonging to a built-in category.
pub fn category_rules(category: &str) -> Vec<String> {
    match category {
        "style" => vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
            "final-newline".to_string(),
            "no-mixed-line-endings".to_string(),
            "no-tabs".to_string(),
            "no-consecutive-empty-lines".to_string(),
            "python-style".to_string(),
            "go-style".to_string(),
            "java-style".to_string(),
            "kotlin-style".to_string(),
            "ruby-style".to_string(),
            "csharp-style".to_string(),
            "missing-semicolon".to_string(),
            "html-no-inline-style".to_string(),
            "css-avoid-important".to_string(),
        ],
        "correctness" => vec![
            "no-todo".to_string(),
            "no-empty-file".to_string(),
            "no-console-log".to_string(),
            "no-var".to_string(),
            "no-print".to_string(),
            "no-unwrap".to_string(),
            "no-expect".to_string(),
            "no-puts".to_string(),
            "no-echo".to_string(),
            "no-swift-print".to_string(),
            "no-dart-print".to_string(),
            "no-csharp-console".to_string(),
            "no-lua-print".to_string(),
            "no-scala-println".to_string(),
            "no-r-print".to_string(),
            "no-zig-debug-print".to_string(),
            "sql-no-select-star".to_string(),
            "html-img-alt".to_string(),
            "shell-echo-quote".to_string(),
        ],
        "security" => vec![
            "hardcoded-secret".to_string(),
            "unsafe-eval".to_string(),
            "sql-injection-risk".to_string(),
        ],
        _ => Vec::new(),
    }
}

/// Returns true if `name` is a known rule category.
pub fn is_category(name: &str) -> bool {
    matches!(name, "style" | "correctness" | "security")
}

/// Returns all built-in rule names (generic + language).
pub fn known_rules() -> Vec<String> {
    vec![
        // Generic
        "line-length".to_string(),
        "trailing-whitespace".to_string(),
        "no-todo".to_string(),
        "no-empty-file".to_string(),
        "no-consecutive-empty-lines".to_string(),
        "no-tabs".to_string(),
        "final-newline".to_string(),
        "no-mixed-line-endings".to_string(),
        "hardcoded-secret".to_string(),
        "unsafe-eval".to_string(),
        "sql-injection-risk".to_string(),
        "max-nesting-depth".to_string(),
        "max-function-lines".to_string(),
        "sort-imports".to_string(),
        // Language
        "no-console-log".to_string(),
        "no-var".to_string(),
        "no-print".to_string(),
        "python-style".to_string(),
        "go-style".to_string(),
        "java-style".to_string(),
        "no-unwrap".to_string(),
        "no-expect".to_string(),
        "missing-semicolon".to_string(),
        "no-puts".to_string(),
        "ruby-style".to_string(),
        "no-echo".to_string(),
        "no-swift-print".to_string(),
        "kotlin-style".to_string(),
        "no-dart-print".to_string(),
        "no-csharp-console".to_string(),
        "csharp-style".to_string(),
        "shell-echo-quote".to_string(),
        "sql-no-select-star".to_string(),
        "no-lua-print".to_string(),
        "no-scala-println".to_string(),
        "no-r-print".to_string(),
        "no-zig-debug-print".to_string(),
        "html-no-inline-style".to_string(),
        "html-img-alt".to_string(),
        "css-avoid-important".to_string(),
    ]
}

/// Returns all built-in plugin names.
pub fn known_plugins() -> Vec<String> {
    vec![
        "security".to_string(),
        "javascript".to_string(),
        "python".to_string(),
        "rust".to_string(),
        "html".to_string(),
        "css".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_rule_categories() {
        let line_length = LineLengthRule { max_length: 100 };
        let trailing_ws = TrailingWhitespaceRule;
        let no_todo = NoTodoRule::default();
        assert_eq!(line_length.category(), "style");
        assert_eq!(trailing_ws.category(), "style");
        assert_eq!(no_todo.category(), "correctness");
    }

    #[test]
    fn test_line_length_rule_short_line() {
        let rule = LineLengthRule { max_length: 100 };
        let content = "let x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_line_length_rule_long_line() {
        let rule = LineLengthRule { max_length: 10 };
        let content = "let very_long_variable_name = 42;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].message.contains("exceeds"));
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_line_length_rule_multiple_lines() {
        let rule = LineLengthRule { max_length: 10 };
        let content = "short\nthis is very long\nshort";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 2);
    }

    #[test]
    fn test_trailing_whitespace_rule_clean() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_trailing_whitespace_rule_spaces() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;   \nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_trailing_whitespace_rule_tabs() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;\t\t\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
    }

    #[test]
    fn test_trailing_whitespace_rule_multiple_lines() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;   \nlet y = 10;\t\nlet z = 15;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_no_todo_rule_clean() {
        let rule = NoTodoRule::default();
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_todo_rule_todo() {
        let rule = NoTodoRule::default();
        let content = "// TODO: implement this\nlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Info);
    }

    #[test]
    fn test_no_todo_rule_fixme() {
        let rule = NoTodoRule::default();
        let content = "// FIXME: fix this bug";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("FIXME"));
    }

    #[test]
    fn test_no_todo_rule_case_insensitive() {
        let rule = NoTodoRule::default();
        let content = "// todo: implement this\n// TODO: implement this\n// FIXME: fix this";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_rule_set_add_rule() {
        let rule_set = RuleSet::new()
            .add_rule(Box::new(LineLengthRule { max_length: 100 }))
            .add_rule(Box::new(TrailingWhitespaceRule));
        assert_eq!(rule_set.rules.len(), 2);
    }

    #[test]
    fn test_rule_set_default() {
        let rule_set = RuleSet::default();
        assert_eq!(rule_set.rules.len(), 0);
    }

    #[test]
    fn test_custom_rule_matches() {
        let def = CustomRuleDefinition {
            name: "no-debugger".to_string(),
            pattern: r"\bdebugger\b".to_string(),
            message: "Debugger statement found".to_string(),
            severity: "Warning".to_string(),
            suggestion: Some("Remove debugger".to_string()),
            extensions: None,
        };
        let rule = CustomRule::from_definition(def).unwrap();
        let messages = rule.check("function foo() { debugger; }", Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert_eq!(messages[0].message, "Debugger statement found");
    }

    #[test]
    fn test_custom_rule_respects_extensions() {
        let def = CustomRuleDefinition {
            name: "no-eval".to_string(),
            pattern: r"\beval\(".to_string(),
            message: "eval found".to_string(),
            severity: "Error".to_string(),
            suggestion: None,
            extensions: Some(vec!["js".to_string()]),
        };
        let rule = CustomRule::from_definition(def).unwrap();
        assert!(rule.check("eval(x)", Path::new("test.py")).is_empty());
        assert!(!rule.check("eval(x)", Path::new("test.js")).is_empty());
    }

    #[test]
    fn test_custom_rule_invalid_regex() {
        let def = CustomRuleDefinition {
            name: "bad".to_string(),
            pattern: "[".to_string(),
            message: "msg".to_string(),
            severity: "Info".to_string(),
            suggestion: None,
            extensions: None,
        };
        assert!(CustomRule::from_definition(def).is_err());
    }

    #[test]
    fn test_property_line_length_never_flags_short_lines() {
        let rule = LineLengthRule { max_length: 100 };
        for len in 1..=100 {
            let line = "x".repeat(len);
            let messages = rule.check(&line, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Line of length {} should not be flagged",
                len
            );
        }
    }

    #[test]
    fn test_property_trailing_whitespace_never_flags_clean_lines() {
        let rule = TrailingWhitespaceRule;
        let clean_lines = [
            "let x = 5;",
            "fn main() {}",
            "    println!(\"hello\");",
            "",
            "// comment",
        ];
        for line in clean_lines {
            let messages = rule.check(line, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Clean line should not be flagged: {}",
                line
            );
        }
    }

    #[test]
    fn test_property_no_todo_never_flags_clean_content() {
        let rule = NoTodoRule::default();
        let clean_contents = [
            "let x = 5;\nlet y = 10;",
            "// This is a normal comment\nfn main() {}",
            "/* multi-line\ncomment */",
            "",
        ];
        for content in clean_contents {
            let messages = rule.check(content, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Clean content should not be flagged: {}",
                content
            );
        }
    }

    #[test]
    fn test_property_fix_never_increases_file_size() {
        let content = "line one   \nline two\t\nline three\n";
        let mut result =
            crate::output::LintResult::new(PathBuf::from("test.rs"), content.to_string());
        let rule = TrailingWhitespaceRule;
        for msg in rule.check(content, Path::new("test.rs")) {
            result.add_message(msg);
        }
        let original_len = result.file_content.len();
        result.apply_fixes();
        assert!(
            result.file_content.len() <= original_len,
            "Fix should not increase file size"
        );
    }

    #[test]
    fn test_no_empty_file_rule_empty() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].message.contains("Empty file"));
    }

    #[test]
    fn test_no_empty_file_rule_whitespace_only() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("   \n\n  ", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Empty file"));
    }

    #[test]
    fn test_no_empty_file_rule_non_empty() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_clean() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\nline two\nline three";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_violation() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\n\nline two";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 3);
        assert!(messages[0].message.contains("Consecutive empty lines"));
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_multiple_violations() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "a\n\n\n\nb\n\n\nc";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_no_tabs_rule_clean() {
        let rule = NoTabsRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_tabs_rule_violation() {
        let rule = NoTabsRule;
        let messages = rule.check("\tlet x = 5;", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].column, 1);
        assert!(messages[0].fix.is_some());
        assert_eq!(
            messages[0].fix.as_ref().unwrap().replacement,
            "    let x = 5;"
        );
    }

    #[test]
    fn test_no_tabs_rule_multiple_tabs() {
        let rule = NoTabsRule;
        let content = "\t\tlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].fix.as_ref().unwrap().replacement,
            "        let x = 5;"
        );
    }

    #[test]
    fn test_final_newline_rule_missing() {
        let rule = FinalNewlineRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("does not end with a newline"));
        assert!(messages[0].fix.is_some());
    }

    #[test]
    fn test_final_newline_rule_present() {
        let rule = FinalNewlineRule;
        let messages = rule.check("let x = 5;\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_final_newline_rule_empty() {
        let rule = FinalNewlineRule;
        let messages = rule.check("", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_clean() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\nline two\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_crlf_only() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\r\nline two\r\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_mixed() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\nline two\r\n", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Mixed line endings"));
    }

    #[test]
    fn test_no_mixed_line_endings_fix() {
        let rule = NoMixedLineEndingsRule;
        let content = "line one\nline two\r\nline three\r\n";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        assert_eq!(fix.replacement, "line one\nline two\nline three\n");
    }

    #[test]
    fn test_no_consecutive_empty_lines_fix() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\n\nline two\n\n\n\nline three\n";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        assert_eq!(fix.replacement, "line one\n\nline two\n\nline three\n");
    }

    #[test]
    fn test_apply_fixes_full_content() {
        let mut result = crate::output::LintResult::new(
            PathBuf::from("test.rs"),
            "line one\n\n\nline two\n".to_string(),
        );
        let rule = NoConsecutiveEmptyLinesRule;
        for msg in rule.check("line one\n\n\nline two\n", Path::new("test.rs")) {
            result.add_message(msg);
        }
        assert!(result.apply_fixes());
        assert_eq!(result.file_content, "line one\n\nline two\n");
    }

    #[test]
    fn test_hardcoded_secret_rule_detects_password() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("password = 'secret123'", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Hardcoded secret"));
        assert_eq!(messages[0].severity, Severity::Error);
    }

    #[test]
    fn test_hardcoded_secret_rule_detects_api_key() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("api_key = \"abc123\"", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_hardcoded_secret_rule_clean() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_unsafe_eval_rule_detects_eval() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("eval(userInput);", Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("eval"));
    }

    #[test]
    fn test_unsafe_eval_rule_ignores_non_js() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("eval(userInput);", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_unsafe_eval_rule_commented() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("// eval(x);", Path::new("test.js"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sql_injection_risk_rule_detects_concat() {
        let rule = SqlInjectionRiskRule;
        let messages = rule.check(
            "const q = \"SELECT * FROM users WHERE id = '\" + id + \"'\";",
            Path::new("test.js"),
        );
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("SQL injection"));
    }

    #[test]
    fn test_sql_injection_risk_rule_clean() {
        let rule = SqlInjectionRiskRule;
        let messages = rule.check("const query = 'SELECT * FROM users';", Path::new("test.js"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_default_severity_values() {
        assert_eq!(
            LineLengthRule { max_length: 80 }.default_severity(),
            Severity::Warning
        );
        assert_eq!(TrailingWhitespaceRule.default_severity(), Severity::Warning);
        assert_eq!(NoTodoRule::default().default_severity(), Severity::Info);
        assert_eq!(NoEmptyFileRule.default_severity(), Severity::Warning);
        assert_eq!(HardcodedSecretRule.default_severity(), Severity::Error);
        assert_eq!(UnsafeEvalRule.default_severity(), Severity::Error);
        assert_eq!(SqlInjectionRiskRule.default_severity(), Severity::Error);
        assert_eq!(SortImportsRule.default_severity(), Severity::Info);
    }

    #[test]
    fn test_plugin_rules_security() {
        let rules = plugin_rules("security");
        assert!(rules.contains(&"hardcoded-secret".to_string()));
        assert!(rules.contains(&"unsafe-eval".to_string()));
        assert!(rules.contains(&"sql-injection-risk".to_string()));
    }

    #[test]
    fn test_plugin_rules_javascript() {
        let rules = plugin_rules("javascript");
        assert!(rules.contains(&"no-console-log".to_string()));
        assert!(rules.contains(&"no-var".to_string()));
    }

    #[test]
    fn test_plugin_rules_unknown_returns_empty() {
        let rules = plugin_rules("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_category_rules_style() {
        let rules = category_rules("style");
        assert!(rules.contains(&"line-length".to_string()));
        assert!(rules.contains(&"trailing-whitespace".to_string()));
        assert!(!rules.contains(&"no-todo".to_string()));
    }

    #[test]
    fn test_category_rules_security() {
        let rules = category_rules("security");
        assert!(rules.contains(&"hardcoded-secret".to_string()));
        assert!(rules.contains(&"unsafe-eval".to_string()));
    }

    #[test]
    fn test_category_rules_unknown_returns_empty() {
        let rules = category_rules("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_is_category() {
        assert!(is_category("style"));
        assert!(is_category("correctness"));
        assert!(is_category("security"));
        assert!(!is_category("line-length"));
        assert!(!is_category("no-todo"));
    }

    #[test]
    fn test_max_nesting_depth_rule() {
        let rule = MaxNestingDepthRule { max_depth: 3 };
        let content = r#"fn main() {
    if true {
        if true {
            if true {
                deeply_nested();
            }
        }
    }
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        // Lines at depth 4 trigger: the 4th nested block start and the body line
        assert!(!messages.is_empty());
        assert_eq!(messages[0].rule, "max-nesting-depth");
        assert!(messages[0].message.contains("depth of 4"));
    }

    #[test]
    fn test_max_nesting_depth_rule_within_limit() {
        let rule = MaxNestingDepthRule { max_depth: 4 };
        let content = r#"fn main() {
    if true {
        if true {
            if true {
                ok();
            }
        }
    }
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_max_function_lines_rule() {
        let rule = MaxFunctionLinesRule { max_lines: 5 };
        let content = r#"fn long_function() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].rule, "max-function-lines");
        assert!(messages[0].message.contains("8 lines"));
    }

    #[test]
    fn test_max_function_lines_rule_within_limit() {
        let rule = MaxFunctionLinesRule { max_lines: 10 };
        let content = r#"fn short() {
    let a = 1;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_max_function_lines_multiple_functions() {
        let rule = MaxFunctionLinesRule { max_lines: 3 };
        let content = r#"fn a() {
    x();
}
fn b() {
    let x = 1;
    let y = 2;
    let z = 3;
    let w = 4;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        // Only fn b() exceeds 3 lines
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Function spans"));
    }

    #[test]
    fn test_sort_imports_rust_unsorted() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;
use std::collections::HashMap;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        assert_eq!(messages[0].rule, "sort-imports");
        assert!(messages[0].message.contains("std::fs"));
    }

    #[test]
    fn test_sort_imports_rust_sorted() {
        let rule = SortImportsRule;
        let content = r#"use std::collections::HashMap;
use std::fs;
use std::io;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_imports_python_unsorted() {
        let rule = SortImportsRule;
        let content = r#"import os
import json
from pathlib import Path
from collections import OrderedDict
"#;
        let messages = rule.check(content, Path::new("test.py"));
        // json < os, so line 2 should be flagged
        assert_eq!(messages.len(), 2);
        assert!(messages[0].message.contains("json"));
    }

    #[test]
    fn test_sort_imports_js_from_unsorted() {
        let rule = SortImportsRule;
        let content = r#"import { foo } from 'lodash';
import { bar } from 'express';
import React from 'react';
"#;
        let messages = rule.check(content, Path::new("test.js"));
        // express < lodash
        assert!(!messages.is_empty());
        assert!(messages[0].message.contains("express"));
    }

    #[test]
    fn test_sort_imports_single_line_noop() {
        let rule = SortImportsRule;
        let content = r#"import os
"#;
        let messages = rule.check(content, Path::new("test.py"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_imports_fix_reorders_block() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;
use std::collections::HashMap;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        assert!(messages[0].fix.is_some());
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        let expected = r#"use std::collections::HashMap;
use std::fs;
use std::io;
"#;
        assert_eq!(fix.replacement, expected);
    }

    #[test]
    fn test_sort_imports_fix_preserves_non_import_lines() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;

fn main() {}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        let fix = messages[0].fix.as_ref().unwrap();
        let expected = r#"use std::fs;
use std::io;

fn main() {}
"#;
        assert_eq!(fix.replacement, expected);
    }

    #[test]
    fn test_fixable_rules_report_has_fix() {
        assert!(TrailingWhitespaceRule.has_fix());
        assert!(NoTabsRule.has_fix());
        assert!(FinalNewlineRule.has_fix());
        assert!(NoConsecutiveEmptyLinesRule.has_fix());
        assert!(NoMixedLineEndingsRule.has_fix());
        assert!(SortImportsRule.has_fix());
    }

    #[test]
    fn test_non_fixable_rules_report_no_fix() {
        assert!(!LineLengthRule { max_length: 100 }.has_fix());
        assert!(!NoEmptyFileRule.has_fix());
        assert!(!NoTodoRule::default().has_fix());
        assert!(!HardcodedSecretRule.has_fix());
        assert!(!UnsafeEvalRule.has_fix());
        assert!(!SqlInjectionRiskRule.has_fix());
        assert!(!MaxNestingDepthRule { max_depth: 4 }.has_fix());
        assert!(!MaxFunctionLinesRule { max_lines: 50 }.has_fix());
    }

    #[test]
    fn test_code_from_name_maps_known_rules() {
        assert_eq!(code_from_name("line-length"), "W001");
        assert_eq!(code_from_name("trailing-whitespace"), "W002");
        assert_eq!(code_from_name("no-empty-file"), "E001");
        assert_eq!(code_from_name("hardcoded-secret"), "S001");
        assert_eq!(code_from_name("max-nesting-depth"), "E002");
        assert_eq!(code_from_name("sort-imports"), "W008");
        assert_eq!(code_from_name("no-console-log"), "L001");
        assert_eq!(code_from_name("css-avoid-important"), "L026");
    }

    #[test]
    fn test_name_from_code_roundtrips() {
        assert_eq!(name_from_code("W001"), "line-length");
        assert_eq!(name_from_code("E001"), "no-empty-file");
        assert_eq!(name_from_code("S001"), "hardcoded-secret");
        assert_eq!(name_from_code("L001"), "no-console-log");
    }

    #[test]
    fn test_unknown_code_returns_itself() {
        assert_eq!(code_from_name("unknown-rule"), "unknown-rule");
        assert_eq!(name_from_code("ZZZ999"), "ZZZ999");
    }

    #[test]
    fn test_rule_code_method_returns_expected() {
        assert_eq!(LineLengthRule { max_length: 100 }.code(), "W001");
        assert_eq!(SortImportsRule.code(), "W008");
        assert_eq!(HardcodedSecretRule.code(), "S001");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Severity;
    use crate::rules::builtin::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_rule_categories() {
        let line_length = LineLengthRule { max_length: 100 };
        let trailing_ws = TrailingWhitespaceRule;
        let no_todo = NoTodoRule::default();
        assert_eq!(line_length.category(), "style");
        assert_eq!(trailing_ws.category(), "style");
        assert_eq!(no_todo.category(), "correctness");
    }

    #[test]
    fn test_line_length_rule_short_line() {
        let rule = LineLengthRule { max_length: 100 };
        let content = "let x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_line_length_rule_long_line() {
        let rule = LineLengthRule { max_length: 10 };
        let content = "let very_long_variable_name = 42;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].message.contains("exceeds"));
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_line_length_rule_multiple_lines() {
        let rule = LineLengthRule { max_length: 10 };
        let content = "short\nthis is very long\nshort";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 2);
    }

    #[test]
    fn test_trailing_whitespace_rule_clean() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_trailing_whitespace_rule_spaces() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;   \nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_trailing_whitespace_rule_tabs() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;\t\t\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
    }

    #[test]
    fn test_trailing_whitespace_rule_multiple_lines() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;   \nlet y = 10;\t\nlet z = 15;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_no_todo_rule_clean() {
        let rule = NoTodoRule::default();
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_todo_rule_todo() {
        let rule = NoTodoRule::default();
        let content = "// TODO: implement this\nlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Info);
    }

    #[test]
    fn test_no_todo_rule_fixme() {
        let rule = NoTodoRule::default();
        let content = "// FIXME: fix this bug";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("FIXME"));
    }

    #[test]
    fn test_no_todo_rule_case_insensitive() {
        let rule = NoTodoRule::default();
        let content = "// todo: implement this\n// TODO: implement this\n// FIXME: fix this";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_rule_set_add_rule() {
        let rule_set = RuleSet::new()
            .add_rule(Box::new(LineLengthRule { max_length: 100 }))
            .add_rule(Box::new(TrailingWhitespaceRule));
        assert_eq!(rule_set.rules.len(), 2);
    }

    #[test]
    fn test_rule_set_default() {
        let rule_set = RuleSet::default();
        assert_eq!(rule_set.rules.len(), 0);
    }

    #[test]
    fn test_custom_rule_matches() {
        let def = CustomRuleDefinition {
            name: "no-debugger".to_string(),
            pattern: r"\bdebugger\b".to_string(),
            message: "Debugger statement found".to_string(),
            severity: "Warning".to_string(),
            suggestion: Some("Remove debugger".to_string()),
            extensions: None,
        };
        let rule = CustomRule::from_definition(def).unwrap();
        let messages = rule.check("function foo() { debugger; }", Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert_eq!(messages[0].message, "Debugger statement found");
    }

    #[test]
    fn test_custom_rule_respects_extensions() {
        let def = CustomRuleDefinition {
            name: "no-eval".to_string(),
            pattern: r"\beval\(".to_string(),
            message: "eval found".to_string(),
            severity: "Error".to_string(),
            suggestion: None,
            extensions: Some(vec!["js".to_string()]),
        };
        let rule = CustomRule::from_definition(def).unwrap();
        assert!(rule.check("eval(x)", Path::new("test.py")).is_empty());
        assert!(!rule.check("eval(x)", Path::new("test.js")).is_empty());
    }

    #[test]
    fn test_custom_rule_invalid_regex() {
        let def = CustomRuleDefinition {
            name: "bad".to_string(),
            pattern: "[".to_string(),
            message: "msg".to_string(),
            severity: "Info".to_string(),
            suggestion: None,
            extensions: None,
        };
        assert!(CustomRule::from_definition(def).is_err());
    }

    #[test]
    fn test_property_line_length_never_flags_short_lines() {
        let rule = LineLengthRule { max_length: 100 };
        for len in 1..=100 {
            let line = "x".repeat(len);
            let messages = rule.check(&line, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Line of length {} should not be flagged",
                len
            );
        }
    }

    #[test]
    fn test_property_trailing_whitespace_never_flags_clean_lines() {
        let rule = TrailingWhitespaceRule;
        let clean_lines = [
            "let x = 5;",
            "fn main() {}",
            "    println!(\"hello\");",
            "",
            "// comment",
        ];
        for line in clean_lines {
            let messages = rule.check(line, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Clean line should not be flagged: {}",
                line
            );
        }
    }

    #[test]
    fn test_property_no_todo_never_flags_clean_content() {
        let rule = NoTodoRule::default();
        let clean_contents = [
            "let x = 5;\nlet y = 10;",
            "// This is a normal comment\nfn main() {}",
            "/* multi-line\ncomment */",
            "",
        ];
        for content in clean_contents {
            let messages = rule.check(content, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Clean content should not be flagged: {}",
                content
            );
        }
    }

    #[test]
    fn test_property_fix_never_increases_file_size() {
        let content = "line one   \nline two\t\nline three\n";
        let mut result =
            crate::output::LintResult::new(PathBuf::from("test.rs"), content.to_string());
        let rule = TrailingWhitespaceRule;
        for msg in rule.check(content, Path::new("test.rs")) {
            result.add_message(msg);
        }
        let original_len = result.file_content.len();
        result.apply_fixes();
        assert!(
            result.file_content.len() <= original_len,
            "Fix should not increase file size"
        );
    }

    #[test]
    fn test_no_empty_file_rule_empty() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].message.contains("Empty file"));
    }

    #[test]
    fn test_no_empty_file_rule_whitespace_only() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("   \n\n  ", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Empty file"));
    }

    #[test]
    fn test_no_empty_file_rule_non_empty() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_clean() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\nline two\nline three";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_violation() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\n\nline two";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 3);
        assert!(messages[0].message.contains("Consecutive empty lines"));
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_multiple_violations() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "a\n\n\n\nb\n\n\nc";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_no_tabs_rule_clean() {
        let rule = NoTabsRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_tabs_rule_violation() {
        let rule = NoTabsRule;
        let messages = rule.check("\tlet x = 5;", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].column, 1);
        assert!(messages[0].fix.is_some());
        assert_eq!(
            messages[0].fix.as_ref().unwrap().replacement,
            "    let x = 5;"
        );
    }

    #[test]
    fn test_no_tabs_rule_multiple_tabs() {
        let rule = NoTabsRule;
        let content = "\t\tlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].fix.as_ref().unwrap().replacement,
            "        let x = 5;"
        );
    }

    #[test]
    fn test_final_newline_rule_missing() {
        let rule = FinalNewlineRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("does not end with a newline"));
        assert!(messages[0].fix.is_some());
    }

    #[test]
    fn test_final_newline_rule_present() {
        let rule = FinalNewlineRule;
        let messages = rule.check("let x = 5;\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_final_newline_rule_empty() {
        let rule = FinalNewlineRule;
        let messages = rule.check("", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_clean() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\nline two\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_crlf_only() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\r\nline two\r\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_mixed() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\nline two\r\n", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Mixed line endings"));
    }

    #[test]
    fn test_no_mixed_line_endings_fix() {
        let rule = NoMixedLineEndingsRule;
        let content = "line one\nline two\r\nline three\r\n";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        assert_eq!(fix.replacement, "line one\nline two\nline three\n");
    }

    #[test]
    fn test_no_consecutive_empty_lines_fix() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\n\nline two\n\n\n\nline three\n";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        assert_eq!(fix.replacement, "line one\n\nline two\n\nline three\n");
    }

    #[test]
    fn test_apply_fixes_full_content() {
        let mut result = crate::output::LintResult::new(
            PathBuf::from("test.rs"),
            "line one\n\n\nline two\n".to_string(),
        );
        let rule = NoConsecutiveEmptyLinesRule;
        for msg in rule.check("line one\n\n\nline two\n", Path::new("test.rs")) {
            result.add_message(msg);
        }
        assert!(result.apply_fixes());
        assert_eq!(result.file_content, "line one\n\nline two\n");
    }

    #[test]
    fn test_hardcoded_secret_rule_detects_password() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("password = 'secret123'", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Hardcoded secret"));
        assert_eq!(messages[0].severity, Severity::Error);
    }

    #[test]
    fn test_hardcoded_secret_rule_detects_api_key() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("api_key = \"abc123\"", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_hardcoded_secret_rule_clean() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_unsafe_eval_rule_detects_eval() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("eval(userInput);", Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("eval"));
    }

    #[test]
    fn test_unsafe_eval_rule_ignores_non_js() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("eval(userInput);", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_unsafe_eval_rule_commented() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("// eval(x);", Path::new("test.js"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sql_injection_risk_rule_detects_concat() {
        let rule = SqlInjectionRiskRule;
        let messages = rule.check(
            "const q = \"SELECT * FROM users WHERE id = '\" + id + \"'\";",
            Path::new("test.js"),
        );
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("SQL injection"));
    }

    #[test]
    fn test_sql_injection_risk_rule_clean() {
        let rule = SqlInjectionRiskRule;
        let messages = rule.check("const query = 'SELECT * FROM users';", Path::new("test.js"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_default_severity_values() {
        assert_eq!(
            LineLengthRule { max_length: 80 }.default_severity(),
            Severity::Warning
        );
        assert_eq!(TrailingWhitespaceRule.default_severity(), Severity::Warning);
        assert_eq!(NoTodoRule::default().default_severity(), Severity::Info);
        assert_eq!(NoEmptyFileRule.default_severity(), Severity::Warning);
        assert_eq!(HardcodedSecretRule.default_severity(), Severity::Error);
        assert_eq!(UnsafeEvalRule.default_severity(), Severity::Error);
        assert_eq!(SqlInjectionRiskRule.default_severity(), Severity::Error);
        assert_eq!(SortImportsRule.default_severity(), Severity::Info);
    }

    #[test]
    fn test_plugin_rules_security() {
        let rules = plugin_rules("security");
        assert!(rules.contains(&"hardcoded-secret".to_string()));
        assert!(rules.contains(&"unsafe-eval".to_string()));
        assert!(rules.contains(&"sql-injection-risk".to_string()));
    }

    #[test]
    fn test_plugin_rules_javascript() {
        let rules = plugin_rules("javascript");
        assert!(rules.contains(&"no-console-log".to_string()));
        assert!(rules.contains(&"no-var".to_string()));
    }

    #[test]
    fn test_plugin_rules_unknown_returns_empty() {
        let rules = plugin_rules("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_category_rules_style() {
        let rules = category_rules("style");
        assert!(rules.contains(&"line-length".to_string()));
        assert!(rules.contains(&"trailing-whitespace".to_string()));
        assert!(!rules.contains(&"no-todo".to_string()));
    }

    #[test]
    fn test_category_rules_security() {
        let rules = category_rules("security");
        assert!(rules.contains(&"hardcoded-secret".to_string()));
        assert!(rules.contains(&"unsafe-eval".to_string()));
    }

    #[test]
    fn test_category_rules_unknown_returns_empty() {
        let rules = category_rules("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_is_category() {
        assert!(is_category("style"));
        assert!(is_category("correctness"));
        assert!(is_category("security"));
        assert!(!is_category("line-length"));
        assert!(!is_category("no-todo"));
    }

    #[test]
    fn test_max_nesting_depth_rule() {
        let rule = MaxNestingDepthRule { max_depth: 3 };
        let content = r#"fn main() {
    if true {
        if true {
            if true {
                deeply_nested();
            }
        }
    }
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        // Lines at depth 4 trigger: the 4th nested block start and the body line
        assert!(!messages.is_empty());
        assert_eq!(messages[0].rule, "max-nesting-depth");
        assert!(messages[0].message.contains("depth of 4"));
    }

    #[test]
    fn test_max_nesting_depth_rule_within_limit() {
        let rule = MaxNestingDepthRule { max_depth: 4 };
        let content = r#"fn main() {
    if true {
        if true {
            if true {
                ok();
            }
        }
    }
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_max_function_lines_rule() {
        let rule = MaxFunctionLinesRule { max_lines: 5 };
        let content = r#"fn long_function() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].rule, "max-function-lines");
        assert!(messages[0].message.contains("8 lines"));
    }

    #[test]
    fn test_max_function_lines_rule_within_limit() {
        let rule = MaxFunctionLinesRule { max_lines: 10 };
        let content = r#"fn short() {
    let a = 1;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_max_function_lines_multiple_functions() {
        let rule = MaxFunctionLinesRule { max_lines: 3 };
        let content = r#"fn a() {
    x();
}
fn b() {
    let x = 1;
    let y = 2;
    let z = 3;
    let w = 4;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        // Only fn b() exceeds 3 lines
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Function spans"));
    }

    #[test]
    fn test_sort_imports_rust_unsorted() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;
use std::collections::HashMap;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        assert_eq!(messages[0].rule, "sort-imports");
        assert!(messages[0].message.contains("std::fs"));
    }

    #[test]
    fn test_sort_imports_rust_sorted() {
        let rule = SortImportsRule;
        let content = r#"use std::collections::HashMap;
use std::fs;
use std::io;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_imports_python_unsorted() {
        let rule = SortImportsRule;
        let content = r#"import os
import json
from pathlib import Path
from collections import OrderedDict
"#;
        let messages = rule.check(content, Path::new("test.py"));
        // json < os, so line 2 should be flagged
        assert_eq!(messages.len(), 2);
        assert!(messages[0].message.contains("json"));
    }

    #[test]
    fn test_sort_imports_js_from_unsorted() {
        let rule = SortImportsRule;
        let content = r#"import { foo } from 'lodash';
import { bar } from 'express';
import React from 'react';
"#;
        let messages = rule.check(content, Path::new("test.js"));
        // express < lodash
        assert!(!messages.is_empty());
        assert!(messages[0].message.contains("express"));
    }

    #[test]
    fn test_sort_imports_single_line_noop() {
        let rule = SortImportsRule;
        let content = r#"import os
"#;
        let messages = rule.check(content, Path::new("test.py"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_imports_fix_reorders_block() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;
use std::collections::HashMap;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        assert!(messages[0].fix.is_some());
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        let expected = r#"use std::collections::HashMap;
use std::fs;
use std::io;
"#;
        assert_eq!(fix.replacement, expected);
    }

    #[test]
    fn test_sort_imports_fix_preserves_non_import_lines() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;

fn main() {}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        let fix = messages[0].fix.as_ref().unwrap();
        let expected = r#"use std::fs;
use std::io;

fn main() {}
"#;
        assert_eq!(fix.replacement, expected);
    }

    #[test]
    fn test_fixable_rules_report_has_fix() {
        assert!(TrailingWhitespaceRule.has_fix());
        assert!(NoTabsRule.has_fix());
        assert!(FinalNewlineRule.has_fix());
        assert!(NoConsecutiveEmptyLinesRule.has_fix());
        assert!(NoMixedLineEndingsRule.has_fix());
        assert!(SortImportsRule.has_fix());
    }

    #[test]
    fn test_non_fixable_rules_report_no_fix() {
        assert!(!LineLengthRule { max_length: 100 }.has_fix());
        assert!(!NoEmptyFileRule.has_fix());
        assert!(!NoTodoRule::default().has_fix());
        assert!(!HardcodedSecretRule.has_fix());
        assert!(!UnsafeEvalRule.has_fix());
        assert!(!SqlInjectionRiskRule.has_fix());
        assert!(!MaxNestingDepthRule { max_depth: 4 }.has_fix());
        assert!(!MaxFunctionLinesRule { max_lines: 50 }.has_fix());
    }

    #[test]
    fn test_code_from_name_maps_known_rules() {
        assert_eq!(code_from_name("line-length"), "W001");
        assert_eq!(code_from_name("trailing-whitespace"), "W002");
        assert_eq!(code_from_name("no-empty-file"), "E001");
        assert_eq!(code_from_name("hardcoded-secret"), "S001");
        assert_eq!(code_from_name("max-nesting-depth"), "E002");
        assert_eq!(code_from_name("sort-imports"), "W008");
        assert_eq!(code_from_name("no-console-log"), "L001");
        assert_eq!(code_from_name("css-avoid-important"), "L026");
    }

    #[test]
    fn test_name_from_code_roundtrips() {
        assert_eq!(name_from_code("W001"), "line-length");
        assert_eq!(name_from_code("E001"), "no-empty-file");
        assert_eq!(name_from_code("S001"), "hardcoded-secret");
        assert_eq!(name_from_code("L001"), "no-console-log");
    }

    #[test]
    fn test_unknown_code_returns_itself() {
        assert_eq!(code_from_name("unknown-rule"), "unknown-rule");
        assert_eq!(name_from_code("ZZZ999"), "ZZZ999");
    }

    #[test]
    fn test_rule_code_method_returns_expected() {
        assert_eq!(LineLengthRule { max_length: 100 }.code(), "W001");
        assert_eq!(SortImportsRule.code(), "W008");
        assert_eq!(HardcodedSecretRule.code(), "S001");
    }
}
