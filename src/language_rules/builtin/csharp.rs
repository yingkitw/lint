use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct CSharpStyleRule;

impl LanguageRule for CSharpStyleRule {
    fn name(&self) -> &str {
        "csharp-style"
    }

    fn description(&self) -> &str {
        "Enforces C# naming conventions and style guidelines."
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
                    Some("Rename class: 'class myClass' → 'class MyClass'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "cs"
    }
}

#[derive(Debug, Clone)]
pub struct CSharpConsoleRule;

static CSHARP_CONSOLE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Console\.(WriteLine?|Write)\s*\(").unwrap());

impl LanguageRule for CSharpConsoleRule {
    fn name(&self) -> &str {
        "no-csharp-console"
    }

    fn description(&self) -> &str {
        "Detects `Console.WriteLine` and similar debug output in C# code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*CSHARP_CONSOLE_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                let col = line.find("Console").unwrap_or(0);
                messages.push(LintMessage::new(
                    line_num + 1,
                    col,
                    self.default_severity(),
                    "Console output detected".to_string(),
                    self.name().to_string(),
                    Some("Use ILogger from Microsoft.Extensions.Logging: inject ILogger<T> and call _logger.LogInformation(\"message\")".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "cs"
    }
}