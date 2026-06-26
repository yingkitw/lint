use crate::output::{LintMessage, Severity};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;

pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> &str {
        "style"
    }
    fn description(&self) -> &str {
        ""
    }
    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage>;
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.len() > self.max_length {
                messages.push(LintMessage::new(
                    line_num + 1,
                    self.max_length + 1,
                    Severity::Warning,
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed = line.trim_end();
                let message = LintMessage::new(
                    line_num + 1,
                    line.len(),
                    Severity::Warning,
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if self.regex.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find(|c: char| c.is_ascii_alphanumeric()).unwrap_or(0),
                    Severity::Info,
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            vec![LintMessage::new(
                1,
                1,
                Severity::Warning,
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
                    Severity::Warning,
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(col) = line.find('\t') {
                let fixed = line.replace('\t', "    ");
                messages.push(
                    LintMessage::new(
                        line_num + 1,
                        col + 1,
                        Severity::Warning,
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        if content.is_empty() {
            return Vec::new();
        }
        if !content.ends_with('\n') {
            vec![LintMessage::new(
                content.lines().count().max(1),
                1,
                Severity::Warning,
                "File does not end with a newline".to_string(),
                self.name().to_string(),
                Some("Add a final newline at the end of the file.".to_string()),
            )
            .with_fix("\n".to_string())]
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
                    Severity::Warning,
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
            assert!(messages.is_empty(), "Line of length {} should not be flagged", len);
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
            assert!(messages.is_empty(), "Clean line should not be flagged: {}", line);
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
            assert!(messages.is_empty(), "Clean content should not be flagged: {}", content);
        }
    }

    #[test]
    fn test_property_fix_never_increases_file_size() {
        let content = "line one   \nline two\t\nline three\n";
        let mut result = crate::output::LintResult::new(
            PathBuf::from("test.rs"),
            content.to_string(),
        );
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
        assert_eq!(messages[0].fix.as_ref().unwrap().replacement, "    let x = 5;");
    }

    #[test]
    fn test_no_tabs_rule_multiple_tabs() {
        let rule = NoTabsRule;
        let content = "\t\tlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].fix.as_ref().unwrap().replacement, "        let x = 5;");
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
}
