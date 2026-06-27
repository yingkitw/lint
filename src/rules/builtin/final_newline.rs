use crate::output::LintMessage;
use crate::rules::Rule;
use std::path::Path;

pub struct FinalNewlineRule;

impl Rule for FinalNewlineRule {
    fn name(&self) -> &str {
        "final-newline"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Files that do not end with a newline character."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        if content.is_empty() {
            return Vec::new();
        }
        if !content.ends_with('\n') {
            vec![
                LintMessage::new(
                    content.lines().count().max(1),
                    1,
                    self.default_severity(),
                    "File does not end with a newline".to_string(),
                    self.name().to_string(),
                    Some("Add a final newline at the end of the file.".to_string()),
                )
                .with_fix("\n".to_string()),
            ]
        } else {
            Vec::new()
        }
    }
}
