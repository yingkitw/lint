pub mod cache;
pub mod config;
pub mod linter;
pub mod rules;
pub mod output;
pub mod mcp;
pub mod language_rules;

pub use config::{Config, ConfigBuilder, OutputFormat};
pub use linter::Linter;
pub use rules::{Rule, RuleSet};
pub use output::{LintResult, LintMessage, Severity};
pub use mcp::McpServer;
pub use language_rules::{LanguageRule, LanguageRuleSet};

use anyhow::Result;
use std::sync::{Arc, Mutex};

pub fn lint_files(config: &Config) -> Result<Vec<LintResult>> {
    let linter = Linter::new(config);
    linter.run()
}

pub fn lint_files_with_cache(
    config: &Config,
    cache: Option<Arc<Mutex<crate::cache::Cache>>>,
) -> Result<Vec<LintResult>> {
    let linter = Linter::new_with_cache(config, cache);
    linter.run()
}

