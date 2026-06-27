use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use std::path::Path;

pub struct NoMixedLineEndingsRule;

impl Rule for NoMixedLineEndingsRule {
    fn name(&self) -> &str {
        "no-mixed-line-endings"
    }

    fn category(&self) -> &str {
        "style"
    }

    fn description(&self) -> &str {
        "Mixed CRLF and LF line endings in the same file."
    }

    fn has_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut has_crlf = false;
        let mut has_lf = false;
        // Scan raw bytes for line endings
        let bytes = content.as_bytes();
        for i in 0..bytes.len() {
            if bytes[i] == b'\r' {
                has_crlf = true;
            } else if bytes[i] == b'\n' && (i == 0 || bytes[i - 1] != b'\r') {
                has_lf = true;
            }
        }
        if has_crlf && has_lf {
            let fixed = content.replace("\r\n", "\n");
            vec![{
                let mut msg = LintMessage::new(
                    1,
                    1,
                    self.default_severity(),
                    "Mixed line endings detected (both CRLF and LF)".to_string(),
                    self.name().to_string(),
                    Some("Use consistent line endings. Prefer LF (\n) for cross-platform compatibility.".to_string()),
                );
                msg.fix = Some(crate::output::Fix {
                    line: 0,
                    replacement: fixed,
                    is_safe: true,
                });
                msg
            }]
        } else {
            Vec::new()
        }
    }
}
