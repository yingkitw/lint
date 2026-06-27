use crate::output::{LintMessage, Severity};
use std::path::Path;

pub mod builtin;
pub mod codes;
pub mod custom;

pub use builtin::{
    FinalNewlineRule, HardcodedSecretRule, LineLengthRule, MaxFunctionLinesRule,
    MaxNestingDepthRule, NoConsecutiveEmptyLinesRule, NoEmptyFileRule, NoMixedLineEndingsRule,
    NoTabsRule, NoTodoRule, SortImportsRule, SqlInjectionRiskRule, TrailingWhitespaceRule,
    UnsafeEvalRule,
};
pub use codes::{
    category_rules, code_from_name, is_category, known_plugins, known_rules, name_from_code,
    plugin_rules,
};
pub use custom::{CustomRule, CustomRuleDefinition};

pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn code(&self) -> &str {
        code_from_name(self.name())
    }
    fn category(&self) -> &str {
        "style"
    }
    fn description(&self) -> &str {
        ""
    }
    fn has_fix(&self) -> bool {
        false
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn url(&self) -> &str {
        ""
    }
    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage>;
}

pub struct RuleSet {
    pub rules: Vec<Box<dyn Rule>>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(mut self, rule: Box<dyn Rule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn get_rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Severity;
    use crate::rules::builtin::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_rule_categories() {
        let line_length = LineLengthRule { max_length: 100 };
        let trailing_ws = TrailingWhitespaceRule;
        let no_todo = NoTodoRule::default();
        assert_eq!(line_length.category(), "style");
        assert_eq!(trailing_ws.category(), "style");
        assert_eq!(no_todo.category(), "correctness");
    }

    #[test]
    fn test_line_length_rule_short_line() {
        let rule = LineLengthRule { max_length: 100 };
        let content = "let x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_line_length_rule_long_line() {
        let rule = LineLengthRule { max_length: 10 };
        let content = "let very_long_variable_name = 42;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].message.contains("exceeds"));
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_line_length_rule_multiple_lines() {
        let rule = LineLengthRule { max_length: 10 };
        let content = "short\nthis is very long\nshort";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 2);
    }

    #[test]
    fn test_trailing_whitespace_rule_clean() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_trailing_whitespace_rule_spaces() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;   \nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_trailing_whitespace_rule_tabs() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;\t\t\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
    }

    #[test]
    fn test_trailing_whitespace_rule_multiple_lines() {
        let rule = TrailingWhitespaceRule;
        let content = "let x = 5;   \nlet y = 10;\t\nlet z = 15;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_no_todo_rule_clean() {
        let rule = NoTodoRule::default();
        let content = "let x = 5;\nlet y = 10;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_todo_rule_todo() {
        let rule = NoTodoRule::default();
        let content = "// TODO: implement this\nlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].severity, Severity::Info);
    }

    #[test]
    fn test_no_todo_rule_fixme() {
        let rule = NoTodoRule::default();
        let content = "// FIXME: fix this bug";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("FIXME"));
    }

    #[test]
    fn test_no_todo_rule_case_insensitive() {
        let rule = NoTodoRule::default();
        let content = "// todo: implement this\n// TODO: implement this\n// FIXME: fix this";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_rule_set_add_rule() {
        let rule_set = RuleSet::new()
            .add_rule(Box::new(LineLengthRule { max_length: 100 }))
            .add_rule(Box::new(TrailingWhitespaceRule));
        assert_eq!(rule_set.rules.len(), 2);
    }

    #[test]
    fn test_rule_set_default() {
        let rule_set = RuleSet::default();
        assert_eq!(rule_set.rules.len(), 0);
    }

    #[test]
    fn test_custom_rule_matches() {
        let def = CustomRuleDefinition {
            name: "no-debugger".to_string(),
            pattern: r"\bdebugger\b".to_string(),
            message: "Debugger statement found".to_string(),
            severity: "Warning".to_string(),
            suggestion: Some("Remove debugger".to_string()),
            extensions: None,
        };
        let rule = CustomRule::from_definition(def).unwrap();
        let messages = rule.check("function foo() { debugger; }", Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert_eq!(messages[0].message, "Debugger statement found");
    }

    #[test]
    fn test_custom_rule_respects_extensions() {
        let def = CustomRuleDefinition {
            name: "no-eval".to_string(),
            pattern: r"\beval\(".to_string(),
            message: "eval found".to_string(),
            severity: "Error".to_string(),
            suggestion: None,
            extensions: Some(vec!["js".to_string()]),
        };
        let rule = CustomRule::from_definition(def).unwrap();
        assert!(rule.check("eval(x)", Path::new("test.py")).is_empty());
        assert!(!rule.check("eval(x)", Path::new("test.js")).is_empty());
    }

    #[test]
    fn test_custom_rule_invalid_regex() {
        let def = CustomRuleDefinition {
            name: "bad".to_string(),
            pattern: "[".to_string(),
            message: "msg".to_string(),
            severity: "Info".to_string(),
            suggestion: None,
            extensions: None,
        };
        assert!(CustomRule::from_definition(def).is_err());
    }

    #[test]
    fn test_property_line_length_never_flags_short_lines() {
        let rule = LineLengthRule { max_length: 100 };
        for len in 1..=100 {
            let line = "x".repeat(len);
            let messages = rule.check(&line, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Line of length {} should not be flagged",
                len
            );
        }
    }

    #[test]
    fn test_property_trailing_whitespace_never_flags_clean_lines() {
        let rule = TrailingWhitespaceRule;
        let clean_lines = [
            "let x = 5;",
            "fn main() {}",
            "    println!(\"hello\");",
            "",
            "// comment",
        ];
        for line in clean_lines {
            let messages = rule.check(line, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Clean line should not be flagged: {}",
                line
            );
        }
    }

    #[test]
    fn test_property_no_todo_never_flags_clean_content() {
        let rule = NoTodoRule::default();
        let clean_contents = [
            "let x = 5;\nlet y = 10;",
            "// This is a normal comment\nfn main() {}",
            "/* multi-line\ncomment */",
            "",
        ];
        for content in clean_contents {
            let messages = rule.check(content, Path::new("test.rs"));
            assert!(
                messages.is_empty(),
                "Clean content should not be flagged: {}",
                content
            );
        }
    }

    #[test]
    fn test_property_fix_never_increases_file_size() {
        let content = "line one   \nline two\t\nline three\n";
        let mut result =
            crate::output::LintResult::new(PathBuf::from("test.rs"), content.to_string());
        let rule = TrailingWhitespaceRule;
        for msg in rule.check(content, Path::new("test.rs")) {
            result.add_message(msg);
        }
        let original_len = result.file_content.len();
        result.apply_fixes();
        assert!(
            result.file_content.len() <= original_len,
            "Fix should not increase file size"
        );
    }

    #[test]
    fn test_no_empty_file_rule_empty() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, Severity::Warning);
        assert!(messages[0].message.contains("Empty file"));
    }

    #[test]
    fn test_no_empty_file_rule_whitespace_only() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("   \n\n  ", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Empty file"));
    }

    #[test]
    fn test_no_empty_file_rule_non_empty() {
        let rule = NoEmptyFileRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_clean() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\nline two\nline three";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_violation() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\n\nline two";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 3);
        assert!(messages[0].message.contains("Consecutive empty lines"));
    }

    #[test]
    fn test_no_consecutive_empty_lines_rule_multiple_violations() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "a\n\n\n\nb\n\n\nc";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_no_tabs_rule_clean() {
        let rule = NoTabsRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_tabs_rule_violation() {
        let rule = NoTabsRule;
        let messages = rule.check("\tlet x = 5;", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].line, 1);
        assert_eq!(messages[0].column, 1);
        assert!(messages[0].fix.is_some());
        assert_eq!(
            messages[0].fix.as_ref().unwrap().replacement,
            "    let x = 5;"
        );
    }

    #[test]
    fn test_no_tabs_rule_multiple_tabs() {
        let rule = NoTabsRule;
        let content = "\t\tlet x = 5;";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].fix.as_ref().unwrap().replacement,
            "        let x = 5;"
        );
    }

    #[test]
    fn test_final_newline_rule_missing() {
        let rule = FinalNewlineRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("does not end with a newline"));
        assert!(messages[0].fix.is_some());
    }

    #[test]
    fn test_final_newline_rule_present() {
        let rule = FinalNewlineRule;
        let messages = rule.check("let x = 5;\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_final_newline_rule_empty() {
        let rule = FinalNewlineRule;
        let messages = rule.check("", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_clean() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\nline two\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_crlf_only() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\r\nline two\r\n", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_no_mixed_line_endings_mixed() {
        let rule = NoMixedLineEndingsRule;
        let messages = rule.check("line one\nline two\r\n", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Mixed line endings"));
    }

    #[test]
    fn test_no_mixed_line_endings_fix() {
        let rule = NoMixedLineEndingsRule;
        let content = "line one\nline two\r\nline three\r\n";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        assert_eq!(fix.replacement, "line one\nline two\nline three\n");
    }

    #[test]
    fn test_no_consecutive_empty_lines_fix() {
        let rule = NoConsecutiveEmptyLinesRule;
        let content = "line one\n\n\nline two\n\n\n\nline three\n";
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        assert_eq!(fix.replacement, "line one\n\nline two\n\nline three\n");
    }

    #[test]
    fn test_apply_fixes_full_content() {
        let mut result = crate::output::LintResult::new(
            PathBuf::from("test.rs"),
            "line one\n\n\nline two\n".to_string(),
        );
        let rule = NoConsecutiveEmptyLinesRule;
        for msg in rule.check("line one\n\n\nline two\n", Path::new("test.rs")) {
            result.add_message(msg);
        }
        assert!(result.apply_fixes());
        assert_eq!(result.file_content, "line one\n\nline two\n");
    }

    #[test]
    fn test_hardcoded_secret_rule_detects_password() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("password = 'secret123'", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Hardcoded secret"));
        assert_eq!(messages[0].severity, Severity::Error);
    }

    #[test]
    fn test_hardcoded_secret_rule_detects_api_key() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("api_key = \"abc123\"", Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_hardcoded_secret_rule_clean() {
        let rule = HardcodedSecretRule;
        let messages = rule.check("let x = 5;", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_unsafe_eval_rule_detects_eval() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("eval(userInput);", Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("eval"));
    }

    #[test]
    fn test_unsafe_eval_rule_ignores_non_js() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("eval(userInput);", Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_unsafe_eval_rule_commented() {
        let rule = UnsafeEvalRule;
        let messages = rule.check("// eval(x);", Path::new("test.js"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sql_injection_risk_rule_detects_concat() {
        let rule = SqlInjectionRiskRule;
        let messages = rule.check(
            "const q = \"SELECT * FROM users WHERE id = '\" + id + \"'\";",
            Path::new("test.js"),
        );
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("SQL injection"));
    }

    #[test]
    fn test_sql_injection_risk_rule_clean() {
        let rule = SqlInjectionRiskRule;
        let messages = rule.check("const query = 'SELECT * FROM users';", Path::new("test.js"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_default_severity_values() {
        assert_eq!(
            LineLengthRule { max_length: 80 }.default_severity(),
            Severity::Warning
        );
        assert_eq!(TrailingWhitespaceRule.default_severity(), Severity::Warning);
        assert_eq!(NoTodoRule::default().default_severity(), Severity::Info);
        assert_eq!(NoEmptyFileRule.default_severity(), Severity::Warning);
        assert_eq!(HardcodedSecretRule.default_severity(), Severity::Error);
        assert_eq!(UnsafeEvalRule.default_severity(), Severity::Error);
        assert_eq!(SqlInjectionRiskRule.default_severity(), Severity::Error);
        assert_eq!(SortImportsRule.default_severity(), Severity::Info);
    }

    #[test]
    fn test_plugin_rules_security() {
        let rules = plugin_rules("security");
        assert!(rules.contains(&"hardcoded-secret".to_string()));
        assert!(rules.contains(&"unsafe-eval".to_string()));
        assert!(rules.contains(&"sql-injection-risk".to_string()));
    }

    #[test]
    fn test_plugin_rules_javascript() {
        let rules = plugin_rules("javascript");
        assert!(rules.contains(&"no-console-log".to_string()));
        assert!(rules.contains(&"no-var".to_string()));
    }

    #[test]
    fn test_plugin_rules_unknown_returns_empty() {
        let rules = plugin_rules("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_category_rules_style() {
        let rules = category_rules("style");
        assert!(rules.contains(&"line-length".to_string()));
        assert!(rules.contains(&"trailing-whitespace".to_string()));
        assert!(!rules.contains(&"no-todo".to_string()));
    }

    #[test]
    fn test_category_rules_security() {
        let rules = category_rules("security");
        assert!(rules.contains(&"hardcoded-secret".to_string()));
        assert!(rules.contains(&"unsafe-eval".to_string()));
    }

    #[test]
    fn test_category_rules_unknown_returns_empty() {
        let rules = category_rules("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_is_category() {
        assert!(is_category("style"));
        assert!(is_category("correctness"));
        assert!(is_category("security"));
        assert!(!is_category("line-length"));
        assert!(!is_category("no-todo"));
    }

    #[test]
    fn test_max_nesting_depth_rule() {
        let rule = MaxNestingDepthRule { max_depth: 3 };
        let content = r#"fn main() {
    if true {
        if true {
            if true {
                deeply_nested();
            }
        }
    }
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        // Lines at depth 4 trigger: the 4th nested block start and the body line
        assert!(!messages.is_empty());
        assert_eq!(messages[0].rule, "max-nesting-depth");
        assert!(messages[0].message.contains("depth of 4"));
    }

    #[test]
    fn test_max_nesting_depth_rule_within_limit() {
        let rule = MaxNestingDepthRule { max_depth: 4 };
        let content = r#"fn main() {
    if true {
        if true {
            if true {
                ok();
            }
        }
    }
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_max_function_lines_rule() {
        let rule = MaxFunctionLinesRule { max_lines: 5 };
        let content = r#"fn long_function() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].rule, "max-function-lines");
        assert!(messages[0].message.contains("8 lines"));
    }

    #[test]
    fn test_max_function_lines_rule_within_limit() {
        let rule = MaxFunctionLinesRule { max_lines: 10 };
        let content = r#"fn short() {
    let a = 1;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_max_function_lines_multiple_functions() {
        let rule = MaxFunctionLinesRule { max_lines: 3 };
        let content = r#"fn a() {
    x();
}
fn b() {
    let x = 1;
    let y = 2;
    let z = 3;
    let w = 4;
}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        // Only fn b() exceeds 3 lines
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Function spans"));
    }

    #[test]
    fn test_sort_imports_rust_unsorted() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;
use std::collections::HashMap;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        assert_eq!(messages[0].rule, "sort-imports");
        assert!(messages[0].message.contains("std::fs"));
    }

    #[test]
    fn test_sort_imports_rust_sorted() {
        let rule = SortImportsRule;
        let content = r#"use std::collections::HashMap;
use std::fs;
use std::io;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_imports_python_unsorted() {
        let rule = SortImportsRule;
        let content = r#"import os
import json
from pathlib import Path
from collections import OrderedDict
"#;
        let messages = rule.check(content, Path::new("test.py"));
        // json < os, so line 2 should be flagged
        assert_eq!(messages.len(), 2);
        assert!(messages[0].message.contains("json"));
    }

    #[test]
    fn test_sort_imports_js_from_unsorted() {
        let rule = SortImportsRule;
        let content = r#"import { foo } from 'lodash';
import { bar } from 'express';
import React from 'react';
"#;
        let messages = rule.check(content, Path::new("test.js"));
        // express < lodash
        assert!(!messages.is_empty());
        assert!(messages[0].message.contains("express"));
    }

    #[test]
    fn test_sort_imports_single_line_noop() {
        let rule = SortImportsRule;
        let content = r#"import os
"#;
        let messages = rule.check(content, Path::new("test.py"));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_imports_fix_reorders_block() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;
use std::collections::HashMap;
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        assert!(messages[0].fix.is_some());
        let fix = messages[0].fix.as_ref().unwrap();
        assert_eq!(fix.line, 0);
        let expected = r#"use std::collections::HashMap;
use std::fs;
use std::io;
"#;
        assert_eq!(fix.replacement, expected);
    }

    #[test]
    fn test_sort_imports_fix_preserves_non_import_lines() {
        let rule = SortImportsRule;
        let content = r#"use std::io;
use std::fs;

fn main() {}
"#;
        let messages = rule.check(content, Path::new("test.rs"));
        assert!(!messages.is_empty());
        let fix = messages[0].fix.as_ref().unwrap();
        let expected = r#"use std::fs;
use std::io;

fn main() {}
"#;
        assert_eq!(fix.replacement, expected);
    }

    #[test]
    fn test_fixable_rules_report_has_fix() {
        assert!(TrailingWhitespaceRule.has_fix());
        assert!(NoTabsRule.has_fix());
        assert!(FinalNewlineRule.has_fix());
        assert!(NoConsecutiveEmptyLinesRule.has_fix());
        assert!(NoMixedLineEndingsRule.has_fix());
        assert!(SortImportsRule.has_fix());
    }

    #[test]
    fn test_non_fixable_rules_report_no_fix() {
        assert!(!LineLengthRule { max_length: 100 }.has_fix());
        assert!(!NoEmptyFileRule.has_fix());
        assert!(!NoTodoRule::default().has_fix());
        assert!(!HardcodedSecretRule.has_fix());
        assert!(!UnsafeEvalRule.has_fix());
        assert!(!SqlInjectionRiskRule.has_fix());
        assert!(!MaxNestingDepthRule { max_depth: 4 }.has_fix());
        assert!(!MaxFunctionLinesRule { max_lines: 50 }.has_fix());
    }

    #[test]
    fn test_code_from_name_maps_known_rules() {
        assert_eq!(code_from_name("line-length"), "W001");
        assert_eq!(code_from_name("trailing-whitespace"), "W002");
        assert_eq!(code_from_name("no-empty-file"), "E001");
        assert_eq!(code_from_name("hardcoded-secret"), "S001");
        assert_eq!(code_from_name("max-nesting-depth"), "E002");
        assert_eq!(code_from_name("sort-imports"), "W008");
        assert_eq!(code_from_name("no-console-log"), "L001");
        assert_eq!(code_from_name("css-avoid-important"), "L026");
    }

    #[test]
    fn test_name_from_code_roundtrips() {
        assert_eq!(name_from_code("W001"), "line-length");
        assert_eq!(name_from_code("E001"), "no-empty-file");
        assert_eq!(name_from_code("S001"), "hardcoded-secret");
        assert_eq!(name_from_code("L001"), "no-console-log");
    }

    #[test]
    fn test_unknown_code_returns_itself() {
        assert_eq!(code_from_name("unknown-rule"), "unknown-rule");
        assert_eq!(name_from_code("ZZZ999"), "ZZZ999");
    }

    #[test]
    fn test_rule_code_method_returns_expected() {
        assert_eq!(LineLengthRule { max_length: 100 }.code(), "W001");
        assert_eq!(SortImportsRule.code(), "W008");
        assert_eq!(HardcodedSecretRule.code(), "S001");
    }
}
