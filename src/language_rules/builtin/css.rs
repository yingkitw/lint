use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct CssImportantRule;

static CSS_IMPORTANT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!important").unwrap());

impl LanguageRule for CssImportantRule {
    fn name(&self) -> &str {
        "css-avoid-important"
    }

    fn description(&self) -> &str {
        "Warns about `!important` usage in CSS that can break maintainability."
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*CSS_IMPORTANT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("/*") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("!important").unwrap_or(0),
                    self.default_severity(),
                    "!important usage detected".to_string(),
                    self.name().to_string(),
                    Some("Increase selector specificity instead: use .parent .child or BEM naming instead of !important".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "css" | "scss" | "sass")
    }
}