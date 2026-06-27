use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct RustUnwrapRule;

static UNWRAP_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.unwrap\(\)").unwrap());

impl LanguageRule for RustUnwrapRule {
    fn name(&self) -> &str {
        "no-unwrap"
    }

    fn description(&self) -> &str {
        "Detects `.unwrap()` calls that can panic; prefer `?` or explicit error handling."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*UNWRAP_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find(".unwrap()").unwrap_or(0),
                    self.default_severity(),
                    "unwrap() usage detected".to_string(),
                    self.name().to_string(),
                    Some("Use 'let x = opt?;' to propagate, or 'match opt { Some(v) => v, None => return Err(...) }' for explicit handling".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rs"
    }
}

#[derive(Debug, Clone)]
pub struct RustExpectRule;

static EXPECT_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.expect\(").unwrap());

impl LanguageRule for RustExpectRule {
    fn name(&self) -> &str {
        "no-expect"
    }

    fn description(&self) -> &str {
        "Detects `.expect()` calls that can panic; prefer `?` or explicit error handling."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*EXPECT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find(".expect(").unwrap_or(0),
                    self.default_severity(),
                    "expect() usage detected".to_string(),
                    self.name().to_string(),
                    Some("Use 'let x = opt?;' to propagate, or 'match opt { Some(v) => v, None => return Err(...) }' for explicit handling".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rs"
    }
}