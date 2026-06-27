use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct PythonPrintRule;

static PYTHON_PRINT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bprint\s*\(").unwrap());

impl LanguageRule for PythonPrintRule {
    fn name(&self) -> &str {
        "no-print"
    }

    fn description(&self) -> &str {
        "Detects print() statements that should not be in production Python code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*PYTHON_PRINT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with('#') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    self.default_severity(),
                    "Print statement found".to_string(),
                    self.name().to_string(),
                    Some("Use logging: 'import logging; logging.info(\"message\")' or remove for production".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "py"
    }
}

#[derive(Debug, Clone)]
pub struct PythonStyleRule;

impl LanguageRule for PythonStyleRule {
    fn name(&self) -> &str {
        "python-style"
    }

    fn description(&self) -> &str {
        "Enforces Python naming conventions and style guidelines."
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("class ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("class").unwrap_or(0),
                    self.default_severity(),
                    "Class name should use CapWords (PascalCase)".to_string(),
                    self.name().to_string(),
                    Some("Rename class to use PascalCase (e.g., MyClass)".to_string()),
                ));
            }

            if line.trim().starts_with("def ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("def").unwrap_or(0),
                    self.default_severity(),
                    "Function name should use snake_case".to_string(),
                    self.name().to_string(),
                    Some("Rename function to use snake_case (e.g., my_function)".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "py"
    }
}