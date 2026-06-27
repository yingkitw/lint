use crate::language_rules::LanguageRule;
use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct SqlSelectStarRule;

static SQL_SELECT_STAR_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)SELECT\s+\*").unwrap());

impl LanguageRule for SqlSelectStarRule {
    fn name(&self) -> &str {
        "sql-no-select-star"
    }

    fn description(&self) -> &str {
        "Warns about `SELECT *` queries that can break when table schemas change."
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SQL_SELECT_STAR_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line)
                && !line.trim_start().starts_with("--")
                && !line.trim_start().starts_with("/*")
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("SELECT").or(line.find("select")).unwrap_or(0),
                    self.default_severity(),
                    "SELECT * usage detected".to_string(),
                    self.name().to_string(),
                    Some("List explicit columns: 'SELECT *' → 'SELECT id, name, created_at' for clarity and performance".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "sql"
    }
}