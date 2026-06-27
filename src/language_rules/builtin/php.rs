use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct PhpEchoRule;

static PHP_ECHO_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\becho\s").unwrap());

impl LanguageRule for PhpEchoRule {
    fn name(&self) -> &str {
        "no-echo"
    }

    fn description(&self) -> &str {
        "Detects `echo` statements that should not be in production PHP code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*PHP_ECHO_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("echo").unwrap_or(0),
                    self.default_severity(),
                    "echo statement found".to_string(),
                    self.name().to_string(),
                    Some(
                        "Use error_log() for debugging or return JSON for API responses"
                            .to_string(),
                    ),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "php"
    }
}