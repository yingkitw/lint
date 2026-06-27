use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use std::path::Path;

pub struct MaxFunctionLinesRule {
    pub max_lines: usize,
}

impl Rule for MaxFunctionLinesRule {
    fn name(&self) -> &str {
        "max-function-lines"
    }

    fn category(&self) -> &str {
        "correctness"
    }

    fn description(&self) -> &str {
        "Functions that exceed the configured maximum line count."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            // Detect function definitions across common languages
            // Pattern: function name(...), fn name(...), def name(...), etc.
            let is_func_start = line.trim().starts_with("fn ")
                || line.trim().starts_with("function ")
                || line.trim().starts_with("def ")
                || line.trim().starts_with("func ")
                || (line.contains("(")
                    && line.contains(")")
                    && line.contains("{")
                    && !line.trim().starts_with("if")
                    && !line.trim().starts_with("for")
                    && !line.trim().starts_with("while")
                    && !line.trim().starts_with("switch")
                    && !line.trim().starts_with("match"));
            if is_func_start {
                let start_line = i + 1;
                let mut brace_depth: isize = 0;
                let mut started = false;
                let mut end_line = i;
                for (j, l) in lines.iter().enumerate().skip(i) {
                    for c in l.chars() {
                        if c == '{' {
                            brace_depth += 1;
                            started = true;
                        } else if c == '}' {
                            brace_depth -= 1;
                        }
                    }
                    end_line = j;
                    if started && brace_depth <= 0 {
                        break;
                    }
                }
                let func_len = end_line - i + 1;
                if func_len > self.max_lines {
                    messages.push(LintMessage::new(
                        start_line,
                        1,
                        self.default_severity(),
                        format!(
                            "Function spans {} lines, exceeding maximum {}",
                            func_len, self.max_lines
                        ),
                        self.name().to_string(),
                        Some(
                            "Refactor: extract helper functions to reduce complexity.".to_string(),
                        ),
                    ));
                }
                i = end_line + 1;
            } else {
                i += 1;
            }
        }
        messages
    }
}
