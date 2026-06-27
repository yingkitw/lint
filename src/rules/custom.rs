use crate::output::{LintMessage, Severity};
use crate::rules::Rule;
use regex::Regex;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomRuleDefinition {
    pub name: String,
    pub pattern: String,
    pub message: String,
    pub severity: String,
    pub suggestion: Option<String>,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CustomRule {
    definition: CustomRuleDefinition,
    regex: Regex,
}

impl CustomRule {
    pub fn from_definition(def: CustomRuleDefinition) -> Result<Self, regex::Error> {
        let regex = Regex::new(&def.pattern)?;
        Ok(Self {
            definition: def,
            regex,
        })
    }
}

impl Rule for CustomRule {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn category(&self) -> &str {
        "custom"
    }

    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage> {
        if let Some(ref extensions) = self.definition.extensions {
            let ext_matches = file_path.extension().is_some_and(|ext| {
                let ext_str = ext.to_string_lossy();
                extensions.iter().any(|e| e == ext_str.as_ref())
            });
            if !ext_matches {
                return Vec::new();
            }
        }

        let mut messages = Vec::new();
        let severity = match self.definition.severity.as_str() {
            "Error" => Severity::Error,
            "Warning" => Severity::Warning,
            _ => Severity::Info,
        };

        for (line_num, line) in content.lines().enumerate() {
            if self.regex.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    0,
                    severity,
                    self.definition.message.clone(),
                    self.definition.name.clone(),
                    self.definition.suggestion.clone(),
                ));
            }
        }

        messages
    }
}
