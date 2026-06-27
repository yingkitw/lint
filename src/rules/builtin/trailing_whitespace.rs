use crate::output::LintMessage;
use crate::rules::Rule;
use std::path::Path;

pub struct TrailingWhitespaceRule;

impl Rule for TrailingWhitespaceRule {
    fn name(&self) -> &str {
        "trailing-whitespace"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Trailing whitespace at the end of lines."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed = line.trim_end();
                let message = LintMessage::new(
                    line_num + 1,
                    line.len(),
                    self.default_severity(),
                    "Trailing whitespace detected".to_string(),
                    self.name().to_string(),
                    Some("Delete spaces/tabs at end of line. In most editors: place cursor at line end and press Backspace until clean.".to_string()),
                )
                .with_fix(trimmed.to_string());
                messages.push(message);
            }
        }

        messages
    }
}
