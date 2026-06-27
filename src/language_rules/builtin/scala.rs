use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ScalaPrintRule;

static SCALA_PRINT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"println\s*\(").unwrap());

impl LanguageRule for ScalaPrintRule {
    fn name(&self) -> &str {
        "no-scala-println"
    }

    fn description(&self) -> &str {
        "Detects `println()` statements that should not be in production Scala code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*SCALA_PRINT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("println").unwrap_or(0),
                    self.default_severity(),
                    "println() call found".to_string(),
                    self.name().to_string(),
                    Some(
                        "Use slf4j: LoggerFactory.getLogger(getClass).info(\"message\")"
                            .to_string(),
                    ),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "scala"
    }
}