use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ShellEchoRule;

impl LanguageRule for ShellEchoRule {
    fn name(&self) -> &str {
        "shell-echo-quote"
    }

    fn description(&self) -> &str {
        "Warns about unquoted variables in shell echo statements to prevent word splitting."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("echo ")
                && !trimmed.starts_with("echo \"")
                && !trimmed.starts_with("echo '")
                && trimmed.contains('$')
                && !trimmed.contains('"')
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("echo").unwrap_or(0),
                    self.default_severity(),
                    "Unquoted variable in echo".to_string(),
                    self.name().to_string(),
                    Some(
                        "Quote variables: 'echo $VAR' → 'echo \"$VAR\"' to prevent word splitting"
                            .to_string(),
                    ),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "sh" | "bash")
    }
}