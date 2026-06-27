use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use std::path::Path;

pub struct SortImportsRule;

impl Rule for SortImportsRule {
    fn name(&self) -> &str {
        "sort-imports"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Import/use statements that are not in alphabetical order."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            if Self::is_import_line(lines[i]) {
                // Find consecutive import block
                let start = i;
                while i < lines.len() && Self::is_import_line(lines[i]) {
                    i += 1;
                }
                let end = i;
                // Check if block has 2+ lines and is sorted
                if end - start >= 2 {
                    let mut prev_key = String::new();
                    for (j, line) in lines.iter().enumerate().take(end).skip(start) {
                        let key = Self::extract_sort_key(line);
                        if !prev_key.is_empty() && key < prev_key {
                            messages.push(LintMessage::new(
                                j + 1,
                                1,
                                self.default_severity(),
                                format!(
                                    "Import '{}' is out of alphabetical order (should come before '{}')",
                                    key, prev_key
                                ),
                                self.name().to_string(),
                                Some("Reorder imports alphabetically for consistency.".to_string()),
                            ));
                        }
                        prev_key = key;
                    }
                }
            } else {
                i += 1;
            }
        }

        // If there are violations, attach a full-file fix to the first message
        if !messages.is_empty() {
            let fixed = Self::sort_all_imports(content);
            messages[0].fix = Some(crate::output::Fix {
                line: 0,
                replacement: fixed,
                is_safe: true,
            });
        }

        messages
    }
}

impl SortImportsRule {
    fn is_import_line(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("#include ")
            || trimmed.contains("require(")
    }

    fn extract_sort_key(line: &str) -> String {
        let trimmed = line.trim();
        if let Some(after) = trimmed.strip_prefix("from ") {
            // Python: from os import path → key: "os"
            if let Some(pos) = after.find(" import") {
                return after[..pos].trim().to_lowercase();
            }
        }
        if let Some(after) = trimmed.strip_prefix("use ") {
            // Rust: use std::fs; → key: "std::fs"
            return after.trim_end_matches(';').trim().to_lowercase();
        }
        if let Some(after) = trimmed.strip_prefix("import ") {
            // JS: import { foo } from 'bar'; → key: "bar" (from clause)
            if let Some(from_pos) = trimmed.rfind(" from ") {
                let after_from = &trimmed[from_pos + 6..];
                let cleaned = after_from.trim().trim_end_matches(';');
                return cleaned
                    .trim_matches(|c| c == '\'' || c == '"')
                    .to_lowercase();
            }
            // Python: import os, sys → key: "os"
            return after
                .split(',')
                .next()
                .unwrap_or(after)
                .trim()
                .to_lowercase();
        }
        if let Some(after) = trimmed.strip_prefix("#include ") {
            return after.trim().to_lowercase();
        }
        if let Some(pos) = trimmed.find("require(") {
            let after = &trimmed[pos + 8..];
            if let Some(end) = after.find(')') {
                let inner = &after[..end];
                return inner.trim_matches(|c| c == '\'' || c == '"').to_lowercase();
            }
        }
        trimmed.to_lowercase()
    }

    /// Returns a copy of `content` with every import block sorted alphabetically.
    fn sort_all_imports(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            if Self::is_import_line(lines[i]) {
                let start = i;
                while i < lines.len() && Self::is_import_line(lines[i]) {
                    i += 1;
                }
                let end = i;
                if end - start >= 2 {
                    let mut block: Vec<(&str, String)> = lines[start..end]
                        .iter()
                        .map(|&l| (l, Self::extract_sort_key(l)))
                        .collect();
                    block.sort_by(|a, b| a.1.cmp(&b.1));
                    for (line, _) in block {
                        result.push(line);
                    }
                } else {
                    result.extend(&lines[start..end]);
                }
            } else {
                result.push(lines[i]);
                i += 1;
            }
        }
        let mut output = result.join("\n");
        if content.ends_with('\n') || content.ends_with("\r\n") {
            output.push('\n');
        }
        output
    }
}
