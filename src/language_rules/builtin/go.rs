use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GoStyleRule;

impl LanguageRule for GoStyleRule {
    fn name(&self) -> &str {
        "go-style"
    }

    fn description(&self) -> &str {
        "Enforces Go naming conventions and style guidelines."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("func ")
                && let Some(open_paren) = line.find('(')
            {
                let func_name_part = &line[4..open_paren].trim();
                if func_name_part
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    messages.push(LintMessage::new(
                        line_num + 1,
                        line.find("func").unwrap_or(0),
                        self.default_severity(),
                        "Exported function detected".to_string(),
                        self.name().to_string(),
                        Some("Ensure exported functions have documentation comments".to_string()),
                    ));
                }
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "go"
    }
}