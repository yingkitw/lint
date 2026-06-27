use crate::output::LintMessage;
use crate::rules::Rule;
use std::path::Path;

pub struct MaxNestingDepthRule {
    pub max_depth: usize,
}

impl Rule for MaxNestingDepthRule {
    fn name(&self) -> &str {
        "max-nesting-depth"
    }

    fn category(&self) -> &str {
        "correctness"
    }

    fn description(&self) -> &str {
        "Code blocks nested deeper than the configured maximum depth."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let mut depth: isize = 0;
        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            // Count brace closings before measuring depth for this line
            let closings = line.chars().filter(|c| *c == '}').count() as isize;
            depth -= closings;
            if depth < 0 {
                depth = 0;
            }
            if depth > self.max_depth as isize {
                messages.push(LintMessage::new(
                    line_num,
                    1,
                    self.default_severity(),
                    format!(
                        "Nesting depth of {} exceeds maximum {}",
                        depth, self.max_depth
                    ),
                    self.name().to_string(),
                    Some(
                        "Refactor to reduce nesting: extract into functions or use early returns."
                            .to_string(),
                    ),
                ));
            }
            // Count brace openings after measuring depth for this line
            let openings = line.chars().filter(|c| *c == '{').count() as isize;
            depth += openings;
        }
        messages
    }
}
