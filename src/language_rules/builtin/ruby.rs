use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct RubyPutsRule;

static PUTS_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bputs\s").unwrap());

impl LanguageRule for RubyPutsRule {
    fn name(&self) -> &str {
        "no-puts"
    }

    fn description(&self) -> &str {
        "Detects `puts` statements that should not be in production Ruby code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*PUTS_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with('#') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("puts").unwrap_or(0),
                    self.default_severity(),
                    "puts statement found".to_string(),
                    self.name().to_string(),
                    Some("Use Logger: 'require \"logger\"; Logger.new($stdout).info(\"message\")' or remove for production".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rb"
    }
}

#[derive(Debug, Clone)]
pub struct RubyStyleRule;

impl LanguageRule for RubyStyleRule {
    fn name(&self) -> &str {
        "ruby-style"
    }

    fn description(&self) -> &str {
        "Enforces Ruby naming conventions and style guidelines."
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
                    "Class name should use CamelCase".to_string(),
                    self.name().to_string(),
                    Some("Rename class to use CamelCase (e.g., MyClass)".to_string()),
                ));
            }

            if line.trim().starts_with("def ") && line.contains('=') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("def").unwrap_or(0),
                    self.default_severity(),
                    "Setter method detected".to_string(),
                    self.name().to_string(),
                    Some("Consider using attr_writer or attr_accessor".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rb"
    }
}