use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

pub struct SqlInjectionRiskRule;

static SQL_INJECTION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r##"(?i)(SELECT|INSERT|UPDATE|DELETE).*\+.*\$|['"].*\$\{|['"].*\+.*\+"##).unwrap()
});

impl Rule for SqlInjectionRiskRule {
    fn name(&self) -> &str {
        "sql-injection-risk"
    }

    fn category(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects potential SQL injection risks from string concatenation or interpolation in queries."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SQL_INJECTION_PATTERN;
        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line)
                && !line.trim_start().starts_with("//")
                && !line.trim_start().starts_with("#")
                && !line.trim_start().starts_with("--")
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.to_lowercase().find("select").or_else(|| line.to_lowercase().find("insert")).or_else(|| line.to_lowercase().find("update")).or_else(|| line.to_lowercase().find("delete")).unwrap_or(0),
                    self.default_severity(),
                    "Potential SQL injection risk from string concatenation".to_string(),
                    self.name().to_string(),
                    Some("Use parameterized queries or prepared statements instead of string concatenation".to_string()),
                ));
            }
        }
        messages
    }
}
