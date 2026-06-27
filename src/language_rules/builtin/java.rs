use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct JavaStyleRule;

impl LanguageRule for JavaStyleRule {
    fn name(&self) -> &str {
        "java-style"
    }

    fn description(&self) -> &str {
        "Enforces Java naming conventions and style guidelines."
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

            if line.contains("System.out.print") && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("System.out.print").unwrap_or(0),
                    self.default_severity(),
                    "System.out.print statement found".to_string(),
                    self.name().to_string(),
                    Some("Use SLF4J: 'private static final Logger log = LoggerFactory.getLogger(MyClass.class); log.info(\"message\");'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "java"
    }
}

#[derive(Debug, Clone)]
pub struct SemicolonRule;

impl LanguageRule for SemicolonRule {
    fn name(&self) -> &str {
        "missing-semicolon"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Detects missing semicolons in JS/TS/Java/C-style code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with("*")
                && !trimmed.ends_with(';')
                && !trimmed.ends_with('{')
                && !trimmed.ends_with('}')
                && !trimmed.ends_with(',')
                && !trimmed.ends_with('(')
                && !trimmed.ends_with(':')
                && !trimmed.contains("return")
                && !trimmed.contains("if ")
                && !trimmed.contains("for ")
                && !trimmed.contains("while ")
                && !trimmed.contains("function")
                && !trimmed.contains("=>")
                && !trimmed.contains("import")
                && !trimmed.contains("export")
                && (trimmed.contains("=")
                    || trimmed.contains("print")
                    || trimmed.contains("println"))
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.len(),
                    self.default_severity(),
                    "Missing semicolon".to_string(),
                    self.name().to_string(),
                    Some("Add semicolon at the end of the statement".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "ts" | "java" | "c" | "cpp" | "rs")
    }
}