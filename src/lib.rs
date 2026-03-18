pub mod config;
pub mod linter;
pub mod rules;
pub mod output;
pub mod mcp;
pub mod language_rules;

pub use config::{Config, ConfigBuilder, OutputFormat};
pub use linter::Linter;
pub use rules::{Rule, RuleSet};
pub use output::{LintResult, LintMessage, Format, Severity};
pub use mcp::McpServer;
pub use language_rules::{LanguageRule, LanguageRuleSet};

use anyhow::Result;

pub fn lint_files(config: &Config) -> Result<Vec<LintResult>> {
    let mut linter = Linter::new(config);
    linter.run()
}

pub fn create_default_config() -> Config {
    ConfigBuilder::default().build()
}
