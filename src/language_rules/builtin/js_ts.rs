use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ConsoleLogRule;

static CONSOLE_LOG_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"console\.(log|warn|error|info|debug)\(").unwrap());

impl LanguageRule for ConsoleLogRule {
    fn name(&self) -> &str {
        "no-console-log"
    }

    fn description(&self) -> &str {
        "Detects console.log and similar debugging statements in JS/TS code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*CONSOLE_LOG_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("console").unwrap_or(0),
                    self.default_severity(),
                    "Console statement found".to_string(),
                    self.name().to_string(),
                    Some("Remove before release or use a logger: `import { logger } from 'logger'; logger.debug('message')`".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "ts" | "jsx" | "tsx")
    }
}

#[derive(Debug, Clone)]
pub struct VarUsageRule;

static NO_VAR_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bvar\s+\w+").unwrap());

impl LanguageRule for VarUsageRule {
    fn name(&self) -> &str {
        "no-var"
    }

    fn description(&self) -> &str {
        "Detects usage of `var` instead of `let` or `const` in JavaScript."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*NO_VAR_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("var").unwrap_or(0),
                    self.default_severity(),
                    "var usage detected".to_string(),
                    self.name().to_string(),
                    Some("Replace with let or const: 'var x = 1' → 'const x = 1' (or 'let x = 1' if reassigned)".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "ts" | "jsx" | "tsx")
    }
}