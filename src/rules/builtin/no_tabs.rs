use crate::output::LintMessage;
use crate::rules::Rule;
use std::path::Path;

pub struct NoTabsRule;

impl Rule for NoTabsRule {
    fn name(&self) -> &str {
        "no-tabs"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Tab characters used for indentation. Use spaces instead."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(col) = line.find('\t') {
                let fixed = line.replace('\t', "    ");
                messages.push(
                    LintMessage::new(
                        line_num + 1,
                        col + 1,
                        self.default_severity(),
                        "Tab character detected; use spaces for indentation".to_string(),
                        self.name().to_string(),
                        Some("Replace tabs with spaces. Most editors support 'Insert spaces instead of tabs' in settings.".to_string()),
                    )
                    .with_fix(fixed),
                );
            }
        }

        messages
    }
}
