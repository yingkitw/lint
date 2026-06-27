use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

pub struct HardcodedSecretRule;

static SECRET_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r##"(?i)(password|passwd|secret|api_key|apikey|token|auth)\s*[=:]\s*['"][^'"]+['"]"##,
    )
    .unwrap()
});

impl Rule for HardcodedSecretRule {
    fn name(&self) -> &str {
        "hardcoded-secret"
    }

    fn category(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects hardcoded credentials such as passwords, API keys, and tokens in source code."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SECRET_PATTERN;
        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                let col = line
                    .to_lowercase()
                    .find("password")
                    .or_else(|| line.to_lowercase().find("secret"))
                    .or_else(|| line.to_lowercase().find("api_key"))
                    .or_else(|| line.to_lowercase().find("apikey"))
                    .or_else(|| line.to_lowercase().find("token"))
                    .or_else(|| line.to_lowercase().find("auth"))
                    .unwrap_or(0);
                messages.push(LintMessage::new(
                    line_num + 1,
                    col,
                    self.default_severity(),
                    "Hardcoded secret detected".to_string(),
                    self.name().to_string(),
                    Some("Use environment variables or a secrets manager instead of hardcoding credentials".to_string()),
                ));
            }
        }
        messages
    }
}
