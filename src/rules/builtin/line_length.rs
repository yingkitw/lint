use crate::output::LintMessage;
use crate::rules::Rule;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LineLengthRule {
    pub max_length: usize,
}

impl Rule for LineLengthRule {
    fn name(&self) -> &str {
        "line-length"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Lines exceeding the configured maximum length."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.len() > self.max_length {
                messages.push(LintMessage::new(
                    line_num + 1,
                    self.max_length + 1,
                    self.default_severity(),
                    format!("Line exceeds maximum length of {} characters", self.max_length),
                    self.name().to_string(),
                    Some(format!(
                        "Break the line: extract to a variable, use line continuation (\\), or split at natural boundaries (e.g., after commas, operators). Max {} chars.",
                        self.max_length
                    )),
                ));
            }
        }

        messages
    }
}
