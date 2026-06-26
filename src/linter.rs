use crate::cache::Cache;
use crate::config::Config;
use crate::language_rules::LanguageRuleSet;
use crate::output::{LintMessage, LintResult, Severity};
use crate::rules::{Rule, RuleSet};
use anyhow::{Context, Result};
use ignore::gitignore::GitignoreBuilder;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct SuppressionDirective {
    start_line: usize,
    end_line: usize,
    rule: Option<String>,
    is_block: bool,
}

pub struct Linter {
    config: Config,
    rule_set: RuleSet,
    language_rule_set: LanguageRuleSet,
    cache: Option<Arc<Mutex<Cache>>>,
    gitignore: Option<ignore::gitignore::Gitignore>,
    ignore_names: std::collections::HashSet<String>,
}

impl Linter {
    pub fn new(config: &Config) -> Self {
        Self::new_with_cache(config, None)
    }

    pub fn new_with_cache(config: &Config, cache: Option<Arc<Mutex<Cache>>>) -> Self {
        let gitignore = if config.no_gitignore {
            None
        } else {
            let mut builder = GitignoreBuilder::new(".");
            builder.add(".gitignore");
            builder.build().ok()
        };

        let ignore_names: std::collections::HashSet<String> = config.ignore_patterns.iter().cloned().collect();

        let mut rule_set = RuleSet::new();

        let rules = vec![
            Box::new(crate::rules::LineLengthRule {
                max_length: config.max_line_length.unwrap_or(100),
            }) as Box<dyn Rule>,
            Box::new(crate::rules::TrailingWhitespaceRule) as Box<dyn Rule>,
            Box::new(crate::rules::NoTodoRule::default()) as Box<dyn Rule>,
            Box::new(crate::rules::NoEmptyFileRule) as Box<dyn Rule>,
            Box::new(crate::rules::NoConsecutiveEmptyLinesRule) as Box<dyn Rule>,
            Box::new(crate::rules::NoTabsRule) as Box<dyn Rule>,
            Box::new(crate::rules::FinalNewlineRule) as Box<dyn Rule>,
            Box::new(crate::rules::NoMixedLineEndingsRule) as Box<dyn Rule>,
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

        if let Some(ref custom_path) = config.rule_set.custom_rules_path
            && let Ok(content) = fs::read_to_string(custom_path)
            && let Ok(definitions) =
                serde_json::from_str::<Vec<crate::rules::CustomRuleDefinition>>(&content)
        {
            for def in definitions {
                if let Ok(rule) = crate::rules::CustomRule::from_definition(def)
                    && config
                        .rule_set
                        .enabled_rules
                        .contains(&rule.name().to_string())
                {
                    rule_set = rule_set.add_rule(Box::new(rule));
                }
            }
        }

        Self {
            config: config.clone(),
            rule_set,
            language_rule_set: LanguageRuleSet::new(),
            cache,
            gitignore,
            ignore_names,
        }
    }

    pub fn list_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for path in &self.config.paths {
            if path.is_file() && self.should_lint_file(path) && !self.is_ignored(path) {
                files.push(path.to_path_buf());
            } else if path.is_dir() {
                let walker = WalkBuilder::new(path)
                    .git_ignore(!self.config.no_gitignore)
                    .build();
                for entry in walker.flatten() {
                    let entry_path = entry.path();
                    if self.is_ignored(entry_path) {
                        continue;
                    }
                    if entry_path.is_file() && self.should_lint_file(entry_path) {
                        files.push(entry_path.to_path_buf());
                    }
                }
            }
        }
        files
    }

    pub fn run(&self) -> Result<Vec<LintResult>> {
        let path_results: Vec<Result<Vec<LintResult>>> = self
            .config
            .paths
            .par_iter()
            .map(|path| self.lint_path(path))
            .collect();

        let mut results = Vec::new();
        for pr in path_results {
            results.extend(pr?);
        }

        Ok(results)
    }

    fn lint_path(&self, path: &Path) -> Result<Vec<LintResult>> {
        let mut results = Vec::new();

        if path.is_file() {
            if self.config.force_exclude && self.is_ignored(path) {
                return Ok(results);
            }
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

        let walker = WalkBuilder::new(dir)
            .git_ignore(!self.config.no_gitignore)
            .build();
        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if self.is_ignored(path) {
                        continue;
                    }
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

    fn is_ignored(&self, path: &Path) -> bool {
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str()
                && self.ignore_names.contains(name)
            {
                return true;
            }
        }
        if let Some(ref gitignore) = self.gitignore {
            let is_dir = path.is_dir();
            if gitignore.matched(path, is_dir).is_ignore() {
                return true;
            }
        }
        false
    }

    fn effective_path<'a>(&'a self, path: &'a Path) -> &'a Path {
        if let Some(ref stdin_path) = self.config.stdin_file_path
            && path.file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|f| f.starts_with("lint_stdin_"))
        {
            Path::new(stdin_path.as_str())
        } else {
            path
        }
    }

    fn lint_file(&self, path: &Path) -> Result<LintResult> {
        let use_content_hash = self.config.cache_strategy == crate::config::CacheStrategy::Content;

        let metadata = fs::metadata(path);
        let cache_key = metadata.as_ref().ok().and_then(|m| {
            let mtime = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
            let size = m.len();
            Some((mtime, size))
        });

        // Fast path: check metadata-based cache first for both strategies
        if let Some((mtime, size)) = cache_key
            && let Some(ref cache) = self.cache
        {
            let cache_guard = cache.lock().unwrap();
            if let Some(cached_messages) = cache_guard.get(path, mtime, size) {
                let mut result = LintResult::new(self.effective_path(path).to_path_buf(), String::new());
                result.messages = cached_messages.clone();
                return Ok(result);
            }
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // Content-based fallback: if metadata changed but content didn't
        if use_content_hash
            && let Some(ref cache) = self.cache
        {
            let hash = format!("{:x}", {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                hasher.finish()
            });
            let cache_guard = cache.lock().unwrap();
            if let Some(cached_messages) = cache_guard.get_by_hash(path, &hash) {
                let mut result = LintResult::new(self.effective_path(path).to_path_buf(), content);
                result.messages = cached_messages.clone();
                return Ok(result);
            }
        }

        let mut result = LintResult::new(self.effective_path(path).to_path_buf(), content);

        let file_level_ignored = Self::parse_file_level_ignore(&result.file_content);
        if file_level_ignored.as_ref().is_some_and(|rules| rules.is_empty()) {
            if !use_content_hash {
                if let Some((mtime, size)) = cache_key
                    && let Some(ref cache) = self.cache
                {
                    let mut cache_guard = cache.lock().unwrap();
                    cache_guard.insert(path.to_path_buf(), mtime, size, result.messages.clone());
                }
            } else if let Some(ref cache) = self.cache {
                let hash = format!("{:x}", {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    result.file_content.hash(&mut hasher);
                    hasher.finish()
                });
                let mut cache_guard = cache.lock().unwrap();
                cache_guard.insert_with_hash(path.to_path_buf(), hash, result.messages.clone());
            }
            return Ok(result);
        }

        for rule in self.rule_set.get_rules() {
            if !self.config.ignore_suppressions
                && file_level_ignored
                    .as_ref()
                    .is_some_and(|rules| rules.contains(&rule.name().to_string()))
            {
                continue;
            }
            let messages = rule.check(&result.file_content, path);
            for message in messages {
                result.add_message(message);
            }
        }

        let language_messages = self.language_rule_set.check(&result.file_content, path);
        for message in language_messages {
            if !self.config.ignore_suppressions
                && file_level_ignored
                    .as_ref()
                    .is_some_and(|rules| rules.contains(&message.rule))
            {
                continue;
            }
            result.add_message(message);
        }

        if !self.config.ignore_suppressions {
            let lines: Vec<&str> = result.file_content.lines().collect();
            let directives = Self::parse_suppression_directives(&lines);
            let disabled_by_line = Self::parse_block_suppressions(&lines);

            let check_unused = self.config.rule_set.enabled_rules.contains(&"unused-suppression".to_string());
            let mut unused_suppressions = Vec::new();

            if check_unused {
                for d in &directives {
                    let used = if d.is_block {
                        result.messages.iter().any(|msg| {
                            let line_idx = msg.line.saturating_sub(1);
                            line_idx >= d.start_line && line_idx < d.end_line
                                && (d.rule.is_none() || d.rule.as_ref().is_some_and(|r| r == &msg.rule))
                        })
                    } else {
                        result.messages.iter().any(|msg| {
                            msg.line.saturating_sub(1) == d.start_line
                                && (d.rule.is_none() || d.rule.as_ref().is_some_and(|r| r == &msg.rule))
                        })
                    };
                    if !used {
                        unused_suppressions.push(d);
                    }
                }
            }

            result.messages.retain(|msg| {
                if let Some(line) = lines.get(msg.line.saturating_sub(1)) {
                    let inline_suppressed = Self::is_line_suppressed(line, &msg.rule);
                    let block_suppressed = disabled_by_line
                        .get(&msg.line.saturating_sub(1))
                        .is_some_and(|disabled| disabled.is_empty() || disabled.contains(&msg.rule));
                    !(inline_suppressed || block_suppressed)
                } else {
                    true
                }
            });

            for d in unused_suppressions {
                let message = if let Some(ref rule) = d.rule {
                    format!("Unused suppression comment: `lint: ignore={}`", rule)
                } else {
                    "Unused suppression comment: `lint: ignore`".to_string()
                };
                result.add_message(LintMessage::new(
                    d.start_line + 1,
                    1,
                    Severity::Warning,
                    message,
                    "unused-suppression".to_string(),
                    Some("Remove this suppression comment or fix the underlying issue.".to_string()),
                ));
            }
        }

        if !self.config.per_file_ignores.is_empty() {
            let path_str = self.effective_path(path).to_string_lossy();
            let mut ignored_rules: Vec<&str> = Vec::new();
            for (pattern, rules) in &self.config.per_file_ignores {
                if let Ok(glob_pattern) = glob::Pattern::new(pattern)
                    && glob_pattern.matches(&path_str)
                {
                    ignored_rules.extend(rules.iter().map(|r| r.as_str()));
                }
            }
            if !ignored_rules.is_empty() {
                result.messages.retain(|msg| !ignored_rules.contains(&msg.rule.as_str()));
            }
        }

        if !self.config.severity_overrides.is_empty() {
            for msg in &mut result.messages {
                if let Some(severity) = self.config.severity_overrides.get(&msg.rule) {
                    msg.severity = *severity;
                }
            }
        }

        if !use_content_hash {
            if let Some((mtime, size)) = cache_key
                && let Some(ref cache) = self.cache
            {
                let mut cache_guard = cache.lock().unwrap();
                cache_guard.insert(path.to_path_buf(), mtime, size, result.messages.clone());
            }
        } else if let Some(ref cache) = self.cache {
            let hash = format!("{:x}", {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                result.file_content.hash(&mut hasher);
                hasher.finish()
            });
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert_with_hash(path.to_path_buf(), hash, result.messages.clone());
        }

        Ok(result)
    }

    fn is_line_suppressed(line: &str, rule_name: &str) -> bool {
        if let Some(pos) = line.find("lint: ignore") {
            let after = &line[pos + "lint: ignore".len()..];
            let after = after.trim_start();
            if after.is_empty() {
                return true;
            }
            if let Some(stripped) = after.strip_prefix('=') {
                let suppressed = stripped.trim();
                return suppressed == rule_name;
            }
        }
        false
    }

    fn parse_block_suppressions(lines: &[&str]) -> HashMap<usize, HashSet<String>> {
        let mut result = HashMap::new();
        let mut all_disabled = false;
        let mut disabled_rules: HashSet<String> = HashSet::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(pos) = line.find("lint: disable") {
                let after = &line[pos + "lint: disable".len()..];
                let after = after.trim_start();
                if after.is_empty() {
                    all_disabled = true;
                    disabled_rules.clear();
                } else if let Some(stripped) = after.strip_prefix('=') {
                    let rule = stripped.trim().to_string();
                    if !all_disabled {
                        disabled_rules.insert(rule);
                    }
                }
            } else if let Some(pos) = line.find("lint: enable") {
                let after = &line[pos + "lint: enable".len()..];
                let after = after.trim_start();
                if after.is_empty() {
                    all_disabled = false;
                    disabled_rules.clear();
                } else if let Some(stripped) = after.strip_prefix('=') {
                    let rule = stripped.trim().to_string();
                    if !all_disabled {
                        disabled_rules.remove(&rule);
                    }
                }
            }

            if all_disabled {
                let _ = result.insert(i, HashSet::new());
            } else if !disabled_rules.is_empty() {
                let _ = result.insert(i, disabled_rules.clone());
            }
        }

        result
    }

    fn parse_file_level_ignore(content: &str) -> Option<Vec<String>> {
        let first_line = content.lines().next()?;
        let pos = first_line.find("lint: ignore-file")?;
        let after = &first_line[pos + "lint: ignore-file".len()..];
        let after = after.trim_start();
        if after.is_empty() {
            return Some(Vec::new());
        }
        if let Some(stripped) = after.strip_prefix('=') {
            let rule = stripped.trim().to_string();
            return Some(vec![rule]);
        }
        None
    }

    fn parse_suppression_directives(lines: &[&str]) -> Vec<SuppressionDirective> {
        let mut directives = Vec::new();
        let mut block_stack: Vec<(usize, Option<String>)> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(pos) = line.find("lint: ignore") {
                let after = &line[pos + "lint: ignore".len()..];
                let after = after.trim_start();
                let rule = if after.is_empty() {
                    None
                } else if let Some(stripped) = after.strip_prefix('=') {
                    Some(stripped.trim().to_string())
                } else {
                    continue;
                };
                directives.push(SuppressionDirective {
                    start_line: i,
                    end_line: i + 1,
                    rule,
                    is_block: false,
                });
            } else if let Some(pos) = line.find("lint: disable") {
                let after = &line[pos + "lint: disable".len()..];
                let after = after.trim_start();
                let rule = if after.is_empty() {
                    None
                } else if let Some(stripped) = after.strip_prefix('=') {
                    Some(stripped.trim().to_string())
                } else {
                    continue;
                };
                block_stack.push((i, rule));
            } else if let Some(pos) = line.find("lint: enable") {
                let after = &line[pos + "lint: enable".len()..];
                let after = after.trim_start();
                if after.is_empty() {
                    while let Some((start, _)) = block_stack.pop() {
                        directives.push(SuppressionDirective {
                            start_line: start,
                            end_line: i,
                            rule: None,
                            is_block: true,
                        });
                    }
                } else if let Some(stripped) = after.strip_prefix('=') {
                    let rule_name = stripped.trim().to_string();
                    if let Some(pos) = block_stack.iter().rposition(|(_, r)| r.as_ref() == Some(&rule_name)) {
                        let (start, _) = block_stack.remove(pos);
                        directives.push(SuppressionDirective {
                            start_line: start,
                            end_line: i,
                            rule: Some(rule_name),
                            is_block: true,
                        });
                    }
                }
            }
        }

        while let Some((start, rule)) = block_stack.pop() {
            directives.push(SuppressionDirective {
                start_line: start,
                end_line: lines.len(),
                rule,
                is_block: true,
            });
        }

        directives
    }

    fn should_lint_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy();
            if let Some(ref configured_exts) = self.config.ext {
                configured_exts.iter().any(|ext| ext == ext_str.as_ref())
            } else {
                let supported_extensions = [
                    "rs", "js", "ts", "jsx", "tsx", "py", "java", "go", "c", "cpp", "h", "hpp", "rb",
                    "php", "swift", "kt", "dart", "cs", "sh", "bash", "sql", "lua", "scala", "r", "zig",
                    "html", "htm", "css", "scss", "sass",
                ];
                supported_extensions.iter().any(|ext| *ext == ext_str.as_ref())
            }
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
    fn test_is_ignored_exact_match() {
        let config = ConfigBuilder::new()
            .ignore_patterns(vec![
                "node_modules".to_string(),
                ".git".to_string(),
            ])
            .build();
        let linter = Linter::new(&config);

        assert!(linter.is_ignored(Path::new("node_modules/package.json")));
        assert!(linter.is_ignored(Path::new("src/node_modules/package.json")));
        assert!(linter.is_ignored(Path::new(".git/config")));
        assert!(linter.is_ignored(Path::new("project/.git/HEAD")));

        assert!(!linter.is_ignored(Path::new("src/main.rs")));
        assert!(!linter.is_ignored(Path::new("lib/git.rs")));
        assert!(!linter.is_ignored(Path::new("node_modules_info.txt")));
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

    #[test]
    fn test_suppression_specific_rule() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "let x = 5;   // lint: ignore=trailing-whitespace\n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_suppression_all_rules() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "let very_long_line_that_exceeds_the_default_maximum_length_of_one_hundred_characters = 42; // lint: ignore\n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .max_line_length(Some(50))
            .enabled_rules(vec!["line-length".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_suppression_only_affects_own_line() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "let x = 5;   // lint: ignore=trailing-whitespace\nlet y = 10;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].line, 2);

        Ok(())
    }

    #[test]
    fn test_is_line_suppressed_unit() {
        assert!(Linter::is_line_suppressed("let x = 5; // lint: ignore=line-length", "line-length"));
        assert!(!Linter::is_line_suppressed("let x = 5; // lint: ignore=trailing-whitespace", "line-length"));
        assert!(Linter::is_line_suppressed("let x = 5; // lint: ignore", "any-rule"));
        assert!(!Linter::is_line_suppressed("let x = 5;", "line-length"));
    }

    #[test]
    fn test_per_file_ignore_filters_rule() -> anyhow::Result<()> {
        use std::collections::HashMap;
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "let x = 5;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let mut per_file_ignores = HashMap::new();
        per_file_ignores.insert("*.rs".to_string(), vec!["trailing-whitespace".to_string()]);

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .per_file_ignores(per_file_ignores)
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_per_file_ignore_does_not_affect_other_files() -> anyhow::Result<()> {
        use std::collections::HashMap;
        let mut file = NamedTempFile::new()?;
        let content = "let x = 5;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let mut per_file_ignores = HashMap::new();
        per_file_ignores.insert("*.py".to_string(), vec!["trailing-whitespace".to_string()]);

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .per_file_ignores(per_file_ignores)
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(!result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_severity_override_changes_severity() -> anyhow::Result<()> {
        use std::collections::HashMap;
        use crate::output::Severity;
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "let x = 5;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let mut overrides = HashMap::new();
        overrides.insert("trailing-whitespace".to_string(), Severity::Error);

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .severity_overrides(overrides)
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].severity, Severity::Error);

        Ok(())
    }

    #[test]
    fn test_severity_override_only_affects_matching_rule() -> anyhow::Result<()> {
        use std::collections::HashMap;
        use crate::output::Severity;
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "let x = 5;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let mut overrides = HashMap::new();
        overrides.insert("line-length".to_string(), Severity::Error);

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .severity_overrides(overrides)
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].severity, Severity::Warning);

        Ok(())
    }

    #[test]
    fn test_block_suppression_disable_enable_rule() -> anyhow::Result<()> {
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "let x = 5;   \n// lint: disable=trailing-whitespace\nlet y = 10;   \nlet z = 20;   \n// lint: enable=trailing-whitespace\nlet w = 30;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        let lines: Vec<_> = result.messages.iter().map(|m| m.line).collect();
        assert!(lines.contains(&1));
        assert!(!lines.contains(&3));
        assert!(!lines.contains(&4));
        assert!(lines.contains(&6));

        Ok(())
    }

    #[test]
    fn test_block_suppression_disable_all_enable_all() -> anyhow::Result<()> {
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "let x = 5;   \n// lint: disable\nlet y = 10;   \nlet z = 20;   \n// lint: enable\nlet w = 30;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        let lines: Vec<_> = result.messages.iter().map(|m| m.line).collect();
        assert!(lines.contains(&1));
        assert!(!lines.contains(&3));
        assert!(!lines.contains(&4));
        assert!(lines.contains(&6));

        Ok(())
    }

    #[test]
    fn test_block_suppression_with_inline_ignore() -> anyhow::Result<()> {
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "// lint: disable=trailing-whitespace\nlet x = 5;   // lint: ignore=trailing-whitespace\nlet y = 10;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        let tw_messages: Vec<_> = result
            .messages
            .iter()
            .filter(|m| m.rule == "trailing-whitespace")
            .collect();
        assert!(tw_messages.is_empty(), "trailing-whitespace should be fully suppressed");

        Ok(())
    }

    #[test]
    fn test_file_level_ignore_all_rules() -> anyhow::Result<()> {
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "// lint: ignore-file\nlet x = 5;   \nlet y = 10;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_file_level_ignore_specific_rule() -> anyhow::Result<()> {
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "// lint: ignore-file=trailing-whitespace\nlet x = 5;   \nTODO: fix this\n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string(), "no-todo".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.iter().all(|m| m.rule != "trailing-whitespace"));
        assert!(result.messages.iter().any(|m| m.rule == "no-todo"));

        Ok(())
    }

    #[test]
    fn test_file_level_ignore_not_on_first_line_is_ignored() -> anyhow::Result<()> {
        let mut file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        let content = "let x = 5;   \n// lint: ignore-file\nlet y = 10;   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(!result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_unused_inline_suppression_detected() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "let x = 5; // lint: ignore=trailing-whitespace\n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string(), "unused-suppression".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].rule, "unused-suppression");
        assert!(result.messages[0].message.contains("trailing-whitespace"));

        Ok(())
    }

    #[test]
    fn test_used_inline_suppression_not_reported() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "let x = 5; // lint: ignore=trailing-whitespace   \n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string(), "unused-suppression".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_unused_block_suppression_detected() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "// lint: disable=trailing-whitespace\nlet x = 5;\n// lint: enable=trailing-whitespace\n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string(), "unused-suppression".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].rule, "unused-suppression");

        Ok(())
    }

    #[test]
    fn test_used_block_suppression_not_reported() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        let content = "// lint: disable=trailing-whitespace\nlet x = 5;   \n// lint: enable=trailing-whitespace\n";
        file.write_all(content.as_bytes())?;
        file.flush()?;

        let config = ConfigBuilder::new()
            .paths(vec![file.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string(), "unused-suppression".to_string()])
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(file.path())?;

        assert!(result.messages.is_empty());

        Ok(())
    }

    #[test]
    fn test_gitignore_respected() -> anyhow::Result<()> {
        use std::io::Write;
        let temp_dir = tempfile::tempdir()?;
        let gitignore_path = temp_dir.path().join(".gitignore");
        let mut gitignore = std::fs::File::create(&gitignore_path)?;
        gitignore.write_all(b"ignored.rs\n")?;

        let ignored_file = temp_dir.path().join("ignored.rs");
        std::fs::write(&ignored_file, "let x = 5;   \n")?;

        let kept_file = temp_dir.path().join("kept.rs");
        std::fs::write(&kept_file, "let x = 5;   \n")?;

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(temp_dir.path())?;

        let config = ConfigBuilder::new()
            .paths(vec![temp_dir.path().to_path_buf()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .build();

        let linter = Linter::new(&config);
        let files = linter.list_files();

        std::env::set_current_dir(original_dir)?;

        assert!(files.iter().any(|p| p.file_name() == Some(std::ffi::OsStr::new("kept.rs"))));
        assert!(!files.iter().any(|p| p.file_name() == Some(std::ffi::OsStr::new("ignored.rs"))));

        Ok(())
    }

    #[test]
    fn test_effective_path_uses_stdin_file_path() {
        let config = ConfigBuilder::new()
            .stdin_file_path(Some("src/main.rs".to_string()))
            .build();
        let linter = Linter::new(&config);
        let temp_path = Path::new("/tmp/lint_stdin_12345.rs");
        assert_eq!(linter.effective_path(temp_path), Path::new("src/main.rs"));

        let normal_path = Path::new("/tmp/other.rs");
        assert_eq!(linter.effective_path(normal_path), normal_path);
    }

    #[test]
    fn test_stdin_file_path_per_file_ignores() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let stdin_temp = temp_dir.path().join("lint_stdin_12345.rs");
        std::fs::write(&stdin_temp, "let x = 5;   \n")?;

        let mut per_file_ignores = HashMap::new();
        per_file_ignores.insert("src/**/*.rs".to_string(), vec!["trailing-whitespace".to_string()]);

        let config = ConfigBuilder::new()
            .paths(vec![stdin_temp.clone()])
            .enabled_rules(vec!["trailing-whitespace".to_string()])
            .per_file_ignores(per_file_ignores)
            .stdin_file_path(Some("src/main.rs".to_string()))
            .build();

        let linter = Linter::new(&config);
        let result = linter.lint_file(&stdin_temp)?;

        assert!(result.messages.is_empty(), "per-file ignores should match stdin_file_path");
        assert_eq!(result.file_path, PathBuf::from("src/main.rs"));

        Ok(())
    }
}
