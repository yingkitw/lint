use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

pub struct UnsafeEvalRule;

static EVAL_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\beval\s*\(").unwrap());

impl Rule for UnsafeEvalRule {
    fn name(&self) -> &str {
        "unsafe-eval"
    }

    fn category(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects unsafe eval() calls which can lead to code injection vulnerabilities."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "js" | "ts" | "jsx" | "tsx") {
            return messages;
        }
        let pattern = &*EVAL_PATTERN;
        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("eval").unwrap_or(0),
                    self.default_severity(),
                    "Unsafe eval() call detected".to_string(),
                    self.name().to_string(),
                    Some("Avoid eval(). Use JSON.parse for JSON data, or structured parsing for other formats".to_string()),
                ));
            }
        }
        messages
    }
}
