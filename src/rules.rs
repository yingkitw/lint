use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;

pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage>;
}

#[derive(Debug, Clone)]
pub struct LineLengthRule {
    pub max_length: usize,
}

impl Rule for LineLengthRule {
    fn name(&self) -> &str {
        "line-length"
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

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.len(),
                    Severity::Warning,
                    "Trailing whitespace detected".to_string(),
                    self.name().to_string(),
                    Some("Delete spaces/tabs at end of line. In most editors: place cursor at line end and press Backspace until clean.".to_string()),
                ));
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct NoTodoRule;

impl Rule for NoTodoRule {
    fn name(&self) -> &str {
        "no-todo"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let todo_pattern = Regex::new(r"(?i)\b(TODO|FIXME|HACK)\b").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if todo_pattern.is_match(line) {
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
    use std::path::Path;

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
        let rule = NoTodoRule;
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_todo_rule_todo() {
        let rule = NoTodoRule;
        let content = "// TODO: implement this\nlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Info);
    }

    #[test]
    fn test_no_todo_rule_fixme() {
        let rule = NoTodoRule;
        let content = "// FIXME: fix this bug";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("FIXME"));
    }

    #[test]
    fn test_no_todo_rule_case_insensitive() {
        let rule = NoTodoRule;
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
}
