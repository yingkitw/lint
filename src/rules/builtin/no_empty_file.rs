use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use std::path::Path;

pub struct NoEmptyFileRule;

impl Rule for NoEmptyFileRule {
    fn name(&self) -> &str {
        "no-empty-file"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Files that are completely empty or contain only whitespace."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            vec![LintMessage::new(
                1,
                1,
                self.default_severity(),
                "Empty file detected".to_string(),
                self.name().to_string(),
                Some("Add content or delete the file if it is unused.".to_string()),
            )]
        } else {
            Vec::new()
        }
    }
}

fn collapse_empty_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut prev_empty = false;

    for line in lines {
        let is_empty = line.trim().is_empty();
        if is_empty && prev_empty {
            continue;
        }
        result.push(line);
        prev_empty = is_empty;
    }

    let mut output = result.join("\n");
    if content.ends_with('\n') || content.ends_with("\r\n") {
        output.push('\n');
    }
    output
}
