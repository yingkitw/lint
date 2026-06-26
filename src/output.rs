use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub file_path: PathBuf,
    pub messages: Vec<LintMessage>,
    pub file_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub line: usize,
    pub replacement: String,
    #[serde(default = "default_true")]
    pub is_safe: bool,
}

pub fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintMessage {
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub message: String,
    pub rule: String,
    pub suggestion: Option<String>,
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

impl LintMessage {
    pub fn new(
        line: usize,
        column: usize,
        severity: Severity,
        message: String,
        rule: String,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            line,
            column,
            severity,
            message,
            rule,
            suggestion,
            fix: None,
        }
    }

    pub fn with_fix(mut self, replacement: String) -> Self {
        self.fix = Some(Fix {
            line: self.line,
            replacement,
            is_safe: true,
        });
        self
    }
}

impl LintResult {
    pub fn new(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            messages: Vec::new(),
            file_content,
        }
    }

    pub fn add_message(&mut self, message: LintMessage) {
        self.messages.push(message);
    }

    pub fn has_errors(&self) -> bool {
        self.messages.iter().any(|m| m.severity == Severity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.messages
            .iter()
            .any(|m| m.severity == Severity::Warning)
    }

    pub fn apply_fixes(&mut self) -> bool {
        let mut fixes: Vec<_> = self
            .messages
            .iter()
            .filter_map(|m| m.fix.as_ref())
            .cloned()
            .collect();

        if fixes.is_empty() {
            return false;
        }

        // Full-content fixes (line == 0) replace the entire file content
        let full_content_fixes: Vec<_> = fixes.iter().filter(|f| f.line == 0).cloned().collect();
        if !full_content_fixes.is_empty() {
            self.file_content = full_content_fixes[0].replacement.clone();
            return true;
        }

        fixes.sort_by_key(|f| f.line);
        fixes.reverse();

        let mut lines: Vec<String> = self.file_content.lines().map(String::from).collect();

        for fix in fixes {
            if fix.line > 0 && fix.line <= lines.len() {
                lines[fix.line - 1] = fix.replacement;
            }
        }

        self.file_content = lines.join("\n");
        if self.file_content.ends_with('\n') || self.file_content.is_empty() {
            // Preserve trailing newline if original had one
        } else if !self.file_content.is_empty() {
            self.file_content.push('\n');
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lint_message_new() {
        let message = LintMessage::new(
            1,
            10,
            Severity::Error,
            "Test error".to_string(),
            "test-rule".to_string(),
            Some("Fix it".to_string()),
        );

        assert_eq!(message.line, 1);
        assert_eq!(message.column, 10);
        assert_eq!(message.severity, Severity::Error);
        assert_eq!(message.message, "Test error");
        assert_eq!(message.rule, "test-rule");
        assert_eq!(message.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_lint_message_new_no_suggestion() {
        let message = LintMessage::new(
            1,
            10,
            Severity::Warning,
            "Test warning".to_string(),
            "test-rule".to_string(),
            None,
        );

        assert!(message.suggestion.is_none());
    }

    #[test]
    fn test_lint_result_new() {
        let path = PathBuf::from("test.rs");
        let content = "let x = 5;";
        let result = LintResult::new(path.clone(), content.to_string());

        assert_eq!(result.file_path, path);
        assert_eq!(result.file_content, content);
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_lint_result_add_message() {
        let mut result = LintResult::new(PathBuf::from("test.rs"), "let x = 5;".to_string());

        let message = LintMessage::new(
            1,
            10,
            Severity::Error,
            "Test error".to_string(),
            "test-rule".to_string(),
            None,
        );

        result.add_message(message);
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_lint_result_has_errors() {
        let mut result = LintResult::new(PathBuf::from("test.rs"), "let x = 5;".to_string());

        assert!(!result.has_errors());

        result.add_message(LintMessage::new(
            1,
            10,
            Severity::Error,
            "Test error".to_string(),
            "test-rule".to_string(),
            None,
        ));

        assert!(result.has_errors());
    }

    #[test]
    fn test_lint_result_has_warnings() {
        let mut result = LintResult::new(PathBuf::from("test.rs"), "let x = 5;".to_string());

        assert!(!result.has_warnings());

        result.add_message(LintMessage::new(
            1,
            10,
            Severity::Warning,
            "Test warning".to_string(),
            "test-rule".to_string(),
            None,
        ));

        assert!(result.has_warnings());
    }

    #[test]
    fn test_lint_result_has_errors_false_for_warnings() {
        let mut result = LintResult::new(PathBuf::from("test.rs"), "let x = 5;".to_string());

        result.add_message(LintMessage::new(
            1,
            10,
            Severity::Warning,
            "Test warning".to_string(),
            "test-rule".to_string(),
            None,
        ));

        assert!(!result.has_errors());
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Info.as_str(), "info");
    }

    #[test]
    fn test_lint_message_serialization() {
        let message = LintMessage::new(
            1,
            10,
            Severity::Error,
            "Test error".to_string(),
            "test-rule".to_string(),
            Some("Fix it".to_string()),
        );

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_lint_result_serialization() {
        let mut result = LintResult::new(PathBuf::from("test.rs"), "let x = 5;".to_string());

        result.add_message(LintMessage::new(
            1,
            10,
            Severity::Error,
            "Test error".to_string(),
            "test-rule".to_string(),
            None,
        ));

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test.rs"));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_apply_fixes_trailing_whitespace() {
        let mut result = LintResult::new(
            PathBuf::from("test.rs"),
            "let x = 5;   \nlet y = 10;\n".to_string(),
        );

        result.add_message(
            LintMessage::new(
                1,
                10,
                Severity::Warning,
                "Trailing whitespace".to_string(),
                "trailing-whitespace".to_string(),
                None,
            )
            .with_fix("let x = 5;".to_string()),
        );

        assert!(result.apply_fixes());
        assert_eq!(result.file_content, "let x = 5;\nlet y = 10;\n");
    }

    #[test]
    fn test_apply_fixes_no_fixes() {
        let mut result = LintResult::new(
            PathBuf::from("test.rs"),
            "let x = 5;\n".to_string(),
        );

        result.add_message(LintMessage::new(
            1,
            1,
            Severity::Warning,
            "Some warning".to_string(),
            "test-rule".to_string(),
            None,
        ));

        assert!(!result.apply_fixes());
        assert_eq!(result.file_content, "let x = 5;\n");
    }

    #[test]
    fn test_with_fix_chains() {
        let message = LintMessage::new(
            1,
            10,
            Severity::Warning,
            "Trailing whitespace".to_string(),
            "trailing-whitespace".to_string(),
            Some("Remove trailing spaces".to_string()),
        )
        .with_fix("fixed line".to_string());

        assert!(message.fix.is_some());
        assert_eq!(message.fix.unwrap().replacement, "fixed line");
    }
}
