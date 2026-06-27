use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct HtmlInlineStyleRule;

static HTML_STYLE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"style\s*=\s*["']"#).unwrap());

impl LanguageRule for HtmlInlineStyleRule {
    fn name(&self) -> &str {
        "html-no-inline-style"
    }

    fn description(&self) -> &str {
        "Warns about inline `style` attributes in HTML; prefer CSS classes."
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*HTML_STYLE_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("style=").unwrap_or(0),
                    self.default_severity(),
                    "Inline style detected".to_string(),
                    self.name().to_string(),
                    Some("Move styles to CSS: use class=\"my-class\" and define in <style> or external .css file".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "html" | "htm")
    }
}

#[derive(Debug, Clone)]
pub struct HtmlMissingAltRule;

static HTML_IMG_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<img\s+[^>]*>").unwrap());

impl LanguageRule for HtmlMissingAltRule {
    fn name(&self) -> &str {
        "html-img-alt"
    }

    fn description(&self) -> &str {
        "Warns about `<img>` tags missing `alt` attributes for accessibility."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let img_pattern = &*HTML_IMG_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if img_pattern.is_match(line) && !line.to_lowercase().contains("alt=") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("<img").unwrap_or(0),
                    self.default_severity(),
                    "img tag missing alt attribute".to_string(),
                    self.name().to_string(),
                    Some("Add alt text for accessibility: <img src=\"x.png\" alt=\"Description of image\">".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "html" | "htm")
    }
}