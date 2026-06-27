use crate::output::{LintMessage, Severity};
use std::path::Path;

pub mod builtin;

pub use builtin::{
    CSharpConsoleRule, CSharpStyleRule, ConsoleLogRule, CssImportantRule, DartPrintRule,
    GoStyleRule, HtmlInlineStyleRule, HtmlMissingAltRule, JavaStyleRule, KotlinStyleRule,
    LuaPrintRule, PhpEchoRule, PythonPrintRule, PythonStyleRule, RPrintRule, RubyPutsRule,
    RubyStyleRule, RustExpectRule, RustUnwrapRule, ScalaPrintRule, SemicolonRule, ShellEchoRule,
    SqlSelectStarRule, SwiftPrintRule, VarUsageRule, ZigDebugPrintRule,
};

pub trait LanguageRule: Send + Sync {
    fn name(&self) -> &str;
    fn code(&self) -> &str {
        crate::rules::code_from_name(self.name())
    }
    fn category(&self) -> &str {
        "correctness"
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
    fn supports_extension(&self, extension: &str) -> bool;
}

pub struct LanguageRuleSet {
    rules: Vec<Box<dyn LanguageRule>>,
}

impl LanguageRuleSet {
    pub fn new() -> Self {
        let rules: Vec<Box<dyn LanguageRule>> = vec![
            Box::new(ConsoleLogRule),
            Box::new(VarUsageRule),
            Box::new(PythonPrintRule),
            Box::new(PythonStyleRule),
            Box::new(GoStyleRule),
            Box::new(JavaStyleRule),
            Box::new(RustUnwrapRule),
            Box::new(RustExpectRule),
            Box::new(SemicolonRule),
            Box::new(RubyPutsRule),
            Box::new(RubyStyleRule),
            Box::new(PhpEchoRule),
            Box::new(SwiftPrintRule),
            Box::new(KotlinStyleRule),
            Box::new(DartPrintRule),
            Box::new(CSharpConsoleRule),
            Box::new(CSharpStyleRule),
            Box::new(ShellEchoRule),
            Box::new(SqlSelectStarRule),
            Box::new(LuaPrintRule),
            Box::new(ScalaPrintRule),
            Box::new(RPrintRule),
            Box::new(ZigDebugPrintRule),
            Box::new(HtmlInlineStyleRule),
            Box::new(HtmlMissingAltRule),
            Box::new(CssImportantRule),
        ];
        Self { rules }
    }

    pub fn new_filtered(enabled: &std::collections::HashSet<String>) -> Self {
        let all = Self::new();
        let rules = all
            .rules
            .into_iter()
            .filter(|r| enabled.contains(r.name()))
            .collect();
        Self { rules }
    }

    pub fn get_rules(&self) -> &[Box<dyn LanguageRule>] {
        &self.rules
    }

    pub fn known_rules() -> Vec<String> {
        Self::new()
            .rules
            .iter()
            .map(|r| r.name().to_string())
            .collect()
    }

    pub fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        if let Some(extension) = file_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            for rule in &self.rules {
                if rule.supports_extension(&ext) {
                    messages.extend(rule.check(content, file_path));
                }
            }
        }

        messages
    }
}

impl Default for LanguageRuleSet {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_rule_categories() {
        assert_eq!(ConsoleLogRule.category(), "correctness");
        assert_eq!(VarUsageRule.category(), "correctness");
        assert_eq!(PythonPrintRule.category(), "correctness");
        assert_eq!(SemicolonRule.category(), "style");
    }

    #[test]
    fn test_console_log_rule_js() {
        let rule = ConsoleLogRule;
        let content = "console.log('hello');";
        let messages = rule.check(content, Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "Console statement found");
    }

    #[test]
    fn test_var_usage_rule() {
        let rule = VarUsageRule;
        let content = "var x = 5;";
        let messages = rule.check(content, Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "var usage detected");
    }

    #[test]
    fn test_python_print_rule() {
        let rule = PythonPrintRule;
        let content = "print('hello')";
        let messages = rule.check(content, Path::new("test.py"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_python_print_rule_commented() {
        let rule = PythonPrintRule;
        let content = "# print('hello')";
        let messages = rule.check(content, Path::new("test.py"));
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_rust_unwrap_rule() {
        let rule = RustUnwrapRule;
        let content = "let x = some_value.unwrap();";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_rust_expect_rule() {
        let rule = RustExpectRule;
        let content = "let x = some_value.expect('value');";
        let messages = rule.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_java_class_naming() {
        let rule = JavaStyleRule;
        let content = "class myClass {}";
        let messages = rule.check(content, Path::new("test.java"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("PascalCase"));
    }

    #[test]
    fn test_rule_set_js() {
        let rule_set = LanguageRuleSet::new();
        let content = "console.log('hello');\nvar x = 5;";
        let messages = rule_set.check(content, Path::new("test.js"));
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_rule_set_py() {
        let rule_set = LanguageRuleSet::new();
        let content = "print('hello')";
        let messages = rule_set.check(content, Path::new("test.py"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_rule_set_rs() {
        let rule_set = LanguageRuleSet::new();
        let content = "let x = some_value.unwrap();";
        let messages = rule_set.check(content, Path::new("test.rs"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_language_rule_supports_extension() {
        assert!(ConsoleLogRule.supports_extension("js"));
        assert!(ConsoleLogRule.supports_extension("ts"));
        assert!(!ConsoleLogRule.supports_extension("py"));

        assert!(PythonPrintRule.supports_extension("py"));
        assert!(!PythonPrintRule.supports_extension("js"));

        assert!(RustUnwrapRule.supports_extension("rs"));
        assert!(!RustUnwrapRule.supports_extension("js"));
    }

    #[test]
    fn test_csharp_console_rule() {
        let rule = CSharpConsoleRule;
        let content = "Console.WriteLine(\"hello\");";
        let messages = rule.check(content, Path::new("test.cs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Console"));
        assert!(messages[0].suggestion.is_some());
    }

    #[test]
    fn test_csharp_console_rule_commented() {
        let rule = CSharpConsoleRule;
        let content = "// Console.WriteLine(\"hello\");";
        let messages = rule.check(content, Path::new("test.cs"));
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_csharp_style_rule() {
        let rule = CSharpStyleRule;
        let content = "class myClass {}";
        let messages = rule.check(content, Path::new("test.cs"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("PascalCase"));
    }

    #[test]
    fn test_shell_echo_rule() {
        let rule = ShellEchoRule;
        let content = "echo $VAR";
        let messages = rule.check(content, Path::new("test.sh"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Unquoted"));
    }

    #[test]
    fn test_shell_echo_rule_quoted() {
        let rule = ShellEchoRule;
        let content = "echo \"$VAR\"";
        let messages = rule.check(content, Path::new("test.sh"));
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_sql_select_star_rule() {
        let rule = SqlSelectStarRule;
        let content = "SELECT * FROM users;";
        let messages = rule.check(content, Path::new("test.sql"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("SELECT *"));
    }

    #[test]
    fn test_sql_select_star_commented() {
        let rule = SqlSelectStarRule;
        let content = "-- SELECT * FROM users;";
        let messages = rule.check(content, Path::new("test.sql"));
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_lua_print_rule() {
        let rule = LuaPrintRule;
        let content = "print('hello')";
        let messages = rule.check(content, Path::new("test.lua"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_scala_println_rule() {
        let rule = ScalaPrintRule;
        let content = "println(\"hello\")";
        let messages = rule.check(content, Path::new("test.scala"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_r_print_rule() {
        let rule = RPrintRule;
        let content = "print(x)";
        let messages = rule.check(content, Path::new("test.r"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_zig_debug_print_rule() {
        let rule = ZigDebugPrintRule;
        let content = "std.debug.print(\"hello\", .{});";
        let messages = rule.check(content, Path::new("test.zig"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_html_inline_style_rule() {
        let rule = HtmlInlineStyleRule;
        let content = r#"<div style="color: red;">"#;
        let messages = rule.check(content, Path::new("test.html"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("Inline"));
    }

    #[test]
    fn test_html_missing_alt_rule() {
        let rule = HtmlMissingAltRule;
        let content = r#"<img src="x.png">"#;
        let messages = rule.check(content, Path::new("test.html"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("alt"));
    }

    #[test]
    fn test_html_img_with_alt() {
        let rule = HtmlMissingAltRule;
        let content = r#"<img src="x.png" alt="Description">"#;
        let messages = rule.check(content, Path::new("test.html"));
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_css_important_rule() {
        let rule = CssImportantRule;
        let content = "color: red !important;";
        let messages = rule.check(content, Path::new("test.css"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.contains("!important"));
    }

    #[test]
    fn test_default_severity_values() {
        assert_eq!(ConsoleLogRule.default_severity(), Severity::Warning);
        assert_eq!(PythonStyleRule.default_severity(), Severity::Info);
        assert_eq!(JavaStyleRule.default_severity(), Severity::Error);
        assert_eq!(KotlinStyleRule.default_severity(), Severity::Error);
        assert_eq!(CSharpStyleRule.default_severity(), Severity::Error);
        assert_eq!(RubyStyleRule.default_severity(), Severity::Info);
        assert_eq!(SqlSelectStarRule.default_severity(), Severity::Info);
        assert_eq!(RPrintRule.default_severity(), Severity::Info);
        assert_eq!(HtmlInlineStyleRule.default_severity(), Severity::Info);
        assert_eq!(CssImportantRule.default_severity(), Severity::Info);
    }

    #[test]
    fn test_fix_suggestions_present() {
        let rule = ConsoleLogRule;
        let content = "console.log('x');";
        let messages = rule.check(content, Path::new("test.js"));
        assert_eq!(messages.len(), 1);
        assert!(messages[0].suggestion.as_ref().unwrap().contains("logger"));
    }

    #[test]
    fn test_rule_set_csharp() {
        let rule_set = LanguageRuleSet::new();
        let content = "Console.WriteLine(\"hello\");\nclass badClass {}";
        let messages = rule_set.check(content, Path::new("test.cs"));
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_rule_set_html() {
        let rule_set = LanguageRuleSet::new();
        let content = r#"<img src="x.png"><div style="x">"#;
        let messages = rule_set.check(content, Path::new("test.html"));
        assert_eq!(messages.len(), 2);
    }
}
