use crate::language_rules::LanguageRule;
use crate::output::LintMessage;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ZigDebugPrintRule;

static ZIG_DEBUG_PRINT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"std\.debug\.print").unwrap());

impl LanguageRule for ZigDebugPrintRule {
    fn name(&self) -> &str {
        "no-zig-debug-print"
    }

    fn description(&self) -> &str {
        "Detects `std.debug.print` statements that should not be in production Zig code."
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = &*ZIG_DEBUG_PRINT_PATTERN;

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("std.debug.print").unwrap_or(0),
                    self.default_severity(),
                    "std.debug.print detected".to_string(),
                    self.name().to_string(),
                    Some(
                        "Remove debug prints before release or use std.log for structured logging"
                            .to_string(),
                    ),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "zig"
    }
}