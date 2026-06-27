use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct RPrintRule;

static R_PRINT_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bprint\s*\(").unwrap());

impl LanguageRule for RPrintRule {
    fn name(&self) -> &str {
        "no-r-print"
    }

    fn description(&self) -> &str {
        "Detects `print()` statements that should not be in production R code."
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*R_PRINT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("#") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    self.default_severity(),
                    "print() call found".to_string(),
                    self.name().to_string(),
                    Some(
                        "Use message() for user output or cat() with appropriate formatting"
                            .to_string(),
                    ),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "r"
    }
}