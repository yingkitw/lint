use crate::config::Config;
use crate::language_rules::LanguageRuleSet;
use crate::output::LintResult;
use crate::rules::{Rule, RuleSet};
use anyhow::{Context, Result};
use ignore::Walk;
use std::fs;
use std::path::Path;

pub struct Linter {
    config: Config,
    rule_set: RuleSet,
    language_rule_set: LanguageRuleSet,
}

impl Linter {
    pub fn new(config: &Config) -> Self {
        let mut rule_set = RuleSet::new();

        let rules = vec![
            Box::new(crate::rules::LineLengthRule {
                max_length: config.max_line_length.unwrap_or(100),
            }) as Box<dyn Rule>,
            Box::new(crate::rules::TrailingWhitespaceRule) as Box<dyn Rule>,
            Box::new(crate::rules::NoTodoRule) as Box<dyn Rule>,
        ];

        for rule in rules {
            if config
                .rule_set
                .enabled_rules
                .contains(&rule.name().to_string())
            {
                rule_set = rule_set.add_rule(rule);
            }
        }

        Self {
            config: config.clone(),
            rule_set,
            language_rule_set: LanguageRuleSet::new(),
        }
    }

    pub fn run(&mut self) -> Result<Vec<LintResult>> {
        let mut results = Vec::new();

        for path in &self.config.paths {
            let path_results = self.lint_path(path)?;
            results.extend(path_results);
        }

        Ok(results)
    }

    fn lint_path(&self, path: &Path) -> Result<Vec<LintResult>> {
        let mut results = Vec::new();

        if path.is_file() {
            let result = self.lint_file(path)?;
            results.push(result);
        } else if path.is_dir() {
            let dir_results = self.lint_directory(path)?;
            results.extend(dir_results);
        }

        Ok(results)
    }

    fn lint_directory(&self, dir: &Path) -> Result<Vec<LintResult>> {
        let mut results = Vec::new();

        for entry in Walk::new(dir) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() && self.should_lint_file(path)
                        && let Ok(result) = self.lint_file(path)
                    {
                        results.push(result);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Could not read entry: {}", e);
                }
            }
        }

        Ok(results)
    }

    fn lint_file(&self, path: &Path) -> Result<LintResult> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let mut result = LintResult::new(path.to_path_buf(), content);

        for rule in self.rule_set.get_rules() {
            let messages = rule.check(&result.file_content, path);
            for message in messages {
                result.add_message(message);
            }
        }

        let language_messages = self.language_rule_set.check(&result.file_content, path);
        for message in language_messages {
            result.add_message(message);
        }

        Ok(result)
    }

    fn should_lint_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let supported_extensions = [
                "rs", "js", "ts", "jsx", "tsx", "py", "java", "go", "c", "cpp", "h", "hpp", "rb",
                "php", "swift", "kt", "dart", "cs", "sh", "bash", "sql", "lua", "scala", "r", "zig",
                "html", "htm", "css", "scss", "sass",
            ];
            supported_extensions
                .iter()
                .any(|ext| *ext == extension.to_string_lossy().as_ref())
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_linter_new() {
        let config = ConfigBuilder::new()
            .paths(vec![PathBuf::from("src")])
            .build();
        let linter = Linter::new(&config);
        assert_eq!(linter.config.paths.len(), 1);
    }

    #[test]
    fn test_linter_default_rules() {
        let config = ConfigBuilder::new()
            .paths(vec![PathBuf::from("src")])
            .build();
        let linter = Linter::new(&config);
        assert!(!linter.rule_set.get_rules().is_empty());
    }

    #[test]
    fn test_linter_enabled_rules() {
        let config = ConfigBuilder::new()
            .paths(vec![PathBuf::from("src")])
            .enabled_rules(vec!["line-length".to_string()])
            .build();
        let linter = Linter::new(&config);
        assert_eq!(linter.rule_set.get_rules().len(), 1);
    }

    #[test]
    fn test_linter_empty_config() {
        let config = ConfigBuilder::new().enabled_rules(vec![]).build();
        let linter = Linter::new(&config);
        assert_eq!(linter.rule_set.get_rules().len(), 0);
    }

    #[test]
    fn test_should_lint_file_supported() {
        let config = ConfigBuilder::new().build();
        let linter = Linter::new(&config);
        assert!(linter.should_lint_file(Path::new("test.rs")));
        assert!(linter.should_lint_file(Path::new("test.js")));
        assert!(linter.should_lint_file(Path::new("test.py")));
        assert!(linter.should_lint_file(Path::new("test.go")));
        assert!(linter.should_lint_file(Path::new("test.cs")));
        assert!(linter.should_lint_file(Path::new("test.sh")));
        assert!(linter.should_lint_file(Path::new("test.sql")));
        assert!(linter.should_lint_file(Path::new("test.lua")));
        assert!(linter.should_lint_file(Path::new("test.html")));
        assert!(linter.should_lint_file(Path::new("test.css")));
        assert!(linter.should_lint_file(Path::new("test.scss")));
        assert!(linter.should_lint_file(Path::new("test.zig")));
    }

    #[test]
    fn test_should_lint_file_unsupported() {
        let config = ConfigBuilder::new().build();
        let linter = Linter::new(&config);
        assert!(!linter.should_lint_file(Path::new("test.txt")));
        assert!(!linter.should_lint_file(Path::new("test.md")));
        assert!(!linter.should_lint_file(Path::new("test.json")));
    }

    #[test]
    fn test_should_lint_file_no_extension() {
        let config = ConfigBuilder::new().build();
        let linter = Linter::new(&config);
        assert!(!linter.should_lint_file(Path::new("Makefile")));
        assert!(!linter.should_lint_file(Path::new(".gitignore")));
    }

    #[test]
    fn test_lint_file_content() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "fn main() {\n    println!(\"Hello\");\n}";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec![])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.file_path, file.path());
        assert_eq!(result.file_content, content);
        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_lint_file_with_issues() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content =
            "fn main() {\n    let very_long_line_that_exceeds_default_maximum_length = 42;\n}";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .max_line_length(Some(50))
            .enabled_rules(vec!["line-length".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.file_path, file.path());
        assert!(!result.messages.is_empty());
        assert!(result.has_warnings());

        Ok(())
    }
}
