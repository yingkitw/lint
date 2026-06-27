use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct SwiftPrintRule;

static SWIFT_PRINT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bprint\s*\(").unwrap());

impl LanguageRule for SwiftPrintRule {
    fn name(&self) -> &str {
        "no-swift-print"
    }

    fn description(&self) -> &str {
        "Detects `print()` statements that should not be in production Swift code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SWIFT_PRINT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    self.default_severity(),
                    "print() call found".to_string(),
                    self.name().to_string(),
                    Some("Use OSLog: 'import os.log; os_log(\"message\", log: .default, type: .info)'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "swift"
    }
}