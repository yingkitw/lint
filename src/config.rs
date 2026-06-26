use crate::output::Severity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub paths: Vec<PathBuf>,
    pub ignore_patterns: Vec<String>,
    pub max_line_length: Option<usize>,
    pub rule_set: RuleSetConfig,
    pub output_format: OutputFormat,
    pub per_file_ignores: HashMap<String, Vec<String>>,
    pub severity_overrides: HashMap<String, Severity>,
    pub extends: Option<String>,
    pub ignore_suppressions: bool,
    pub cache_strategy: CacheStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheStrategy {
    Metadata,
    Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetConfig {
    pub enabled_rules: Vec<String>,
    pub custom_rules_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    Github,
    Sarif,
    Junit,
    Concise,
    Gitlab,
}

pub struct ConfigBuilder {
    paths: Vec<PathBuf>,
    ignore_patterns: Vec<String>,
    max_line_length: Option<usize>,
    rule_set: RuleSetConfig,
    output_format: OutputFormat,
    per_file_ignores: HashMap<String, Vec<String>>,
    severity_overrides: HashMap<String, Severity>,
    extends: Option<String>,
    ignore_suppressions: bool,
    cache_strategy: CacheStrategy,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            ignore_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
            max_line_length: Some(100),
            rule_set: RuleSetConfig {
                enabled_rules: vec![
                    "line-length".to_string(),
                    "trailing-whitespace".to_string(),
                    "final-newline".to_string(),
                    "no-mixed-line-endings".to_string(),
                ],
                custom_rules_path: None,
            },
            output_format: OutputFormat::Text,
            per_file_ignores: HashMap::new(),
            severity_overrides: HashMap::new(),
            extends: None,
            ignore_suppressions: false,
            cache_strategy: CacheStrategy::Metadata,
        }
    }

    pub fn paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.paths = paths;
        self
    }

    pub fn ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    pub fn max_line_length(mut self, length: Option<usize>) -> Self {
        self.max_line_length = length;
        self
    }

    pub fn enabled_rules(mut self, rules: Vec<String>) -> Self {
        self.rule_set.enabled_rules = rules;
        self
    }

    pub fn custom_rules(mut self, path: Option<PathBuf>) -> Self {
        self.rule_set.custom_rules_path = path;
        self
    }

    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn per_file_ignores(mut self, ignores: HashMap<String, Vec<String>>) -> Self {
        self.per_file_ignores = ignores;
        self
    }

    pub fn severity_overrides(mut self, overrides: HashMap<String, Severity>) -> Self {
        self.severity_overrides = overrides;
        self
    }

    pub fn extends(mut self, path: Option<String>) -> Self {
        self.extends = path;
        self
    }

    pub fn ignore_suppressions(mut self, value: bool) -> Self {
        self.ignore_suppressions = value;
        self
    }

    pub fn cache_strategy(mut self, strategy: CacheStrategy) -> Self {
        self.cache_strategy = strategy;
        self
    }

    pub fn build(self) -> Config {
        Config {
            paths: self.paths,
            ignore_patterns: self.ignore_patterns,
            max_line_length: self.max_line_length,
            rule_set: self.rule_set,
            output_format: self.output_format,
            per_file_ignores: self.per_file_ignores,
            severity_overrides: self.severity_overrides,
            extends: self.extends,
            ignore_suppressions: self.ignore_suppressions,
            cache_strategy: self.cache_strategy,
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_new() {
        let builder = ConfigBuilder::new();
        assert!(builder.paths.is_empty());
        assert_eq!(builder.max_line_length, Some(100));
        assert_eq!(builder.ignore_patterns.len(), 3);
    }

    #[test]
    fn test_config_builder_paths() {
        let builder = ConfigBuilder::new().paths(vec![
            std::path::PathBuf::from("src"),
            std::path::PathBuf::from("tests"),
        ]);
        assert_eq!(builder.paths.len(), 2);
    }

    #[test]
    fn test_config_builder_max_line_length() {
        let builder = ConfigBuilder::new().max_line_length(Some(120));
        assert_eq!(builder.max_line_length, Some(120));
    }

    #[test]
    fn test_config_builder_max_line_length_none() {
        let builder = ConfigBuilder::new().max_line_length(None);
        assert!(builder.max_line_length.is_none());
    }

    #[test]
    fn test_config_builder_enabled_rules() {
        let builder = ConfigBuilder::new().enabled_rules(vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
        ]);
        assert_eq!(builder.rule_set.enabled_rules.len(), 2);
    }

    #[test]
    fn test_config_builder_output_format() {
        let builder = ConfigBuilder::new().output_format(OutputFormat::Json);
        assert!(matches!(builder.output_format, OutputFormat::Json));
    }

    #[test]
    fn test_config_builder_build() {
        let builder = ConfigBuilder::new();
        let config = builder.build();
        assert!(config.paths.is_empty());
        assert_eq!(config.max_line_length, Some(100));
    }

    #[test]
    fn test_config_builder_default() {
        let builder = ConfigBuilder::default();
        assert_eq!(builder.max_line_length, Some(100));
    }

    #[test]
    fn test_config_serialization() {
        let mut per_file_ignores = HashMap::new();
        per_file_ignores.insert("tests/**/*.rs".to_string(), vec!["line-length".to_string()]);
        let mut severity_overrides = HashMap::new();
        severity_overrides.insert("line-length".to_string(), Severity::Error);
        let config = Config {
            paths: vec![std::path::PathBuf::from("src")],
            ignore_patterns: vec!["node_modules".to_string()],
            max_line_length: Some(100),
            rule_set: RuleSetConfig {
                enabled_rules: vec!["line-length".to_string()],
                custom_rules_path: None,
            },
            output_format: OutputFormat::Text,
            per_file_ignores,
            severity_overrides,
            extends: None,
            ignore_suppressions: false,
            cache_strategy: CacheStrategy::Metadata,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.paths.len(), 1);
    }
}
