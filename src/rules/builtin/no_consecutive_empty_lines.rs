use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use std::path::Path;

pub struct NoConsecutiveEmptyLinesRule;

impl Rule for NoConsecutiveEmptyLinesRule {
    fn name(&self) -> &str {
        "no-consecutive-empty-lines"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "More than one consecutive empty line."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut prev_empty = false;
        let mut had_consecutive = false;

        for (i, line) in lines.iter().enumerate() {
            let is_empty = line.trim().is_empty();
            if is_empty && prev_empty {
                messages.push(LintMessage::new(
                    i + 1,
                    1,
                    self.default_severity(),
                    "Consecutive empty lines detected".to_string(),
                    self.name().to_string(),
                    Some("Remove extra blank lines; files should not contain more than one consecutive empty line.".to_string()),
                ));
                had_consecutive = true;
            }
            prev_empty = is_empty;
        }

        if had_consecutive {
            let fixed = collapse_empty_lines(content);
            if let Some(first) = messages.first_mut() {
                first.fix = Some(crate::output::Fix {
                    line: 0,
                    replacement: fixed,
                    is_safe: true,
                });
            }
        }

        messages
    }
}
