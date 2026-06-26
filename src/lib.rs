pub mod cache;
pub mod config;
pub mod language_rules;
pub mod linter;
pub mod lsp;
pub mod mcp;
pub mod output;
pub mod rules;

pub use config::{Config, ConfigBuilder, OutputFormat};
pub use language_rules::{LanguageRule, LanguageRuleSet};
pub use linter::Linter;
pub use lsp::LspServer;
pub use mcp::McpServer;
pub use output::{LintMessage, LintResult, Severity};
pub use rules::{Rule, RuleSet};

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
