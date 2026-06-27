use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct KotlinStyleRule;

impl LanguageRule for KotlinStyleRule {
    fn name(&self) -> &str {
        "kotlin-style"
    }

    fn description(&self) -> &str {
        "Enforces Kotlin naming conventions and style guidelines."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
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
                    "Class name should use PascalCase".to_string(),
                    self.name().to_string(),
                    Some("Rename class to start with uppercase letter (e.g., MyClass)".to_string()),
                ));
            }

            if line.contains("println(") && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("println(").unwrap_or(0),
                    self.default_severity(),
                    "println() call found".to_string(),
                    self.name().to_string(),
                    Some("Use slf4j: 'LoggerFactory.getLogger(MyClass::class.java).info(\"message\")'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "kt"
    }
}