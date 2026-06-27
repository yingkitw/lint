use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use regex::Regex;
use std::path::Path;

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
