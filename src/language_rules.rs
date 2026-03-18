use crate::output::{LintMessage, Severity};
use regex::Regex;
use std::path::Path;

pub trait LanguageRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, content: &str, file_path: &Path) -> Vec<LintMessage>;
    fn supports_extension(&self, extension: &str) -> bool;
}

#[derive(Debug, Clone)]
pub struct ConsoleLogRule;

impl LanguageRule for ConsoleLogRule {
    fn name(&self) -> &str {
        "no-console-log"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"console\.(log|warn|error|info|debug)\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("console").unwrap_or(0),
                    Severity::Warning,
                    "Console statement found".to_string(),
                    self.name().to_string(),
                    Some("Remove before release or use a logger: `import { logger } from 'logger'; logger.debug('message')`".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "ts" | "jsx" | "tsx")
    }
}

#[derive(Debug, Clone)]
pub struct VarUsageRule;

impl LanguageRule for VarUsageRule {
    fn name(&self) -> &str {
        "no-var"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bvar\s+\w+").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("var").unwrap_or(0),
                    Severity::Warning,
                    "var usage detected".to_string(),
                    self.name().to_string(),
                    Some("Replace with let or const: 'var x = 1' → 'const x = 1' (or 'let x = 1' if reassigned)".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "ts" | "jsx" | "tsx")
    }
}

#[derive(Debug, Clone)]
pub struct PythonPrintRule;

impl LanguageRule for PythonPrintRule {
    fn name(&self) -> &str {
        "no-print"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bprint\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with('#') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    Severity::Warning,
                    "Print statement found".to_string(),
                    self.name().to_string(),
                    Some("Use logging: 'import logging; logging.info(\"message\")' or remove for production".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "py"
    }
}

#[derive(Debug, Clone)]
pub struct PythonStyleRule;

impl LanguageRule for PythonStyleRule {
    fn name(&self) -> &str {
        "python-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("class ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("class").unwrap_or(0),
                    Severity::Warning,
                    "Class name should use CapWords (PascalCase)".to_string(),
                    self.name().to_string(),
                    Some("Rename class to use PascalCase (e.g., MyClass)".to_string()),
                ));
            }

            if line.trim().starts_with("def ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("def").unwrap_or(0),
                    Severity::Info,
                    "Function name should use snake_case".to_string(),
                    self.name().to_string(),
                    Some("Rename function to use snake_case (e.g., my_function)".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "py"
    }
}

#[derive(Debug, Clone)]
pub struct GoStyleRule;

impl LanguageRule for GoStyleRule {
    fn name(&self) -> &str {
        "go-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("func ")
                && let Some(open_paren) = line.find('(')
            {
                let func_name_part = &line[4..open_paren].trim();
                if func_name_part
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    messages.push(LintMessage::new(
                        line_num + 1,
                        line.find("func").unwrap_or(0),
                        Severity::Warning,
                        "Exported function detected".to_string(),
                        self.name().to_string(),
                        Some(
                            "Ensure exported functions have documentation comments".to_string(),
                        ),
                    ));
                }
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "go"
    }
}

#[derive(Debug, Clone)]
pub struct JavaStyleRule;

impl LanguageRule for JavaStyleRule {
    fn name(&self) -> &str {
        "java-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("class ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("class").unwrap_or(0),
                    Severity::Error,
                    "Class name should use PascalCase".to_string(),
                    self.name().to_string(),
                    Some("Rename class to start with uppercase letter (e.g., MyClass)".to_string()),
                ));
            }

            if line.contains("System.out.print") && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("System.out.print").unwrap_or(0),
                    Severity::Warning,
                    "System.out.print statement found".to_string(),
                    self.name().to_string(),
                    Some("Use SLF4J: 'private static final Logger log = LoggerFactory.getLogger(MyClass.class); log.info(\"message\");'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "java"
    }
}

#[derive(Debug, Clone)]
pub struct RustUnwrapRule;

impl LanguageRule for RustUnwrapRule {
    fn name(&self) -> &str {
        "no-unwrap"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\.unwrap\(\)").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find(".unwrap()").unwrap_or(0),
                    Severity::Warning,
                    "unwrap() usage detected".to_string(),
                    self.name().to_string(),
                    Some("Use 'let x = opt?;' to propagate, or 'match opt { Some(v) => v, None => return Err(...) }' for explicit handling".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rs"
    }
}

#[derive(Debug, Clone)]
pub struct RustExpectRule;

impl LanguageRule for RustExpectRule {
    fn name(&self) -> &str {
        "no-expect"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\.expect\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find(".expect(").unwrap_or(0),
                    Severity::Warning,
                    "expect() usage detected".to_string(),
                    self.name().to_string(),
                    Some("Use 'let x = opt?;' to propagate, or 'match opt { Some(v) => v, None => return Err(...) }' for explicit handling".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rs"
    }
}

#[derive(Debug, Clone)]
pub struct SemicolonRule;

impl LanguageRule for SemicolonRule {
    fn name(&self) -> &str {
        "missing-semicolon"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with("*")
                && !trimmed.ends_with(';')
                && !trimmed.ends_with('{')
                && !trimmed.ends_with('}')
                && !trimmed.ends_with(',')
                && !trimmed.ends_with('(')
                && !trimmed.ends_with(':')
                && !trimmed.contains("return")
                && !trimmed.contains("if ")
                && !trimmed.contains("for ")
                && !trimmed.contains("while ")
                && !trimmed.contains("function")
                && !trimmed.contains("=>")
                && !trimmed.contains("import")
                && !trimmed.contains("export")
                && (trimmed.contains("=") || trimmed.contains("print") || trimmed.contains("println"))
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.len(),
                    Severity::Warning,
                    "Missing semicolon".to_string(),
                    self.name().to_string(),
                    Some("Add semicolon at the end of the statement".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "ts" | "java" | "c" | "cpp" | "rs")
    }
}

#[derive(Debug, Clone)]
pub struct RubyPutsRule;

impl LanguageRule for RubyPutsRule {
    fn name(&self) -> &str {
        "no-puts"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bputs\s").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with('#') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("puts").unwrap_or(0),
                    Severity::Warning,
                    "puts statement found".to_string(),
                    self.name().to_string(),
                    Some("Use Logger: 'require \"logger\"; Logger.new($stdout).info(\"message\")' or remove for production".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rb"
    }
}

#[derive(Debug, Clone)]
pub struct RubyStyleRule;

impl LanguageRule for RubyStyleRule {
    fn name(&self) -> &str {
        "ruby-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("class ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("class").unwrap_or(0),
                    Severity::Warning,
                    "Class name should use CamelCase".to_string(),
                    self.name().to_string(),
                    Some("Rename class to use CamelCase (e.g., MyClass)".to_string()),
                ));
            }

            if line.trim().starts_with("def ") && line.contains('=') {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("def").unwrap_or(0),
                    Severity::Info,
                    "Setter method detected".to_string(),
                    self.name().to_string(),
                    Some("Consider using attr_writer or attr_accessor".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "rb"
    }
}

#[derive(Debug, Clone)]
pub struct PhpEchoRule;

impl LanguageRule for PhpEchoRule {
    fn name(&self) -> &str {
        "no-echo"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\becho\s").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("echo").unwrap_or(0),
                    Severity::Warning,
                    "echo statement found".to_string(),
                    self.name().to_string(),
                    Some("Use error_log() for debugging or return JSON for API responses".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "php"
    }
}

#[derive(Debug, Clone)]
pub struct SwiftPrintRule;

impl LanguageRule for SwiftPrintRule {
    fn name(&self) -> &str {
        "no-swift-print"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bprint\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    Severity::Warning,
                    "print() call found".to_string(),
                    self.name().to_string(),
                    Some("Use OSLog: 'import os.log; os_log(\"message\", log: .default, type: .info)'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "swift"
    }
}

#[derive(Debug, Clone)]
pub struct KotlinStyleRule;

impl LanguageRule for KotlinStyleRule {
    fn name(&self) -> &str {
        "kotlin-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("class ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("class").unwrap_or(0),
                    Severity::Error,
                    "Class name should use PascalCase".to_string(),
                    self.name().to_string(),
                    Some("Rename class to start with uppercase letter (e.g., MyClass)".to_string()),
                ));
            }

            if line.contains("println(") && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("println(").unwrap_or(0),
                    Severity::Warning,
                    "println() call found".to_string(),
                    self.name().to_string(),
                    Some("Use slf4j: 'LoggerFactory.getLogger(MyClass::class.java).info(\"message\")'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "kt"
    }
}

#[derive(Debug, Clone)]
pub struct DartPrintRule;

impl LanguageRule for DartPrintRule {
    fn name(&self) -> &str {
        "no-dart-print"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bprint\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    Severity::Warning,
                    "print() call found".to_string(),
                    self.name().to_string(),
                    Some("Replace with debugPrint() or use the logging package: import 'dart:developer' as developer; developer.log('message')".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "dart"
    }
}

#[derive(Debug, Clone)]
pub struct CSharpConsoleRule;

impl LanguageRule for CSharpConsoleRule {
    fn name(&self) -> &str {
        "no-csharp-console"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"Console\.(WriteLine?|Write)\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                let col = line.find("Console").unwrap_or(0);
                messages.push(LintMessage::new(
                    line_num + 1,
                    col,
                    Severity::Warning,
                    "Console output detected".to_string(),
                    self.name().to_string(),
                    Some("Use ILogger from Microsoft.Extensions.Logging: inject ILogger<T> and call _logger.LogInformation(\"message\")".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "cs"
    }
}

#[derive(Debug, Clone)]
pub struct CSharpStyleRule;

impl LanguageRule for CSharpStyleRule {
    fn name(&self) -> &str {
        "csharp-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("class ")
                && line
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("class").unwrap_or(0),
                    Severity::Error,
                    "Class name should use PascalCase".to_string(),
                    self.name().to_string(),
                    Some("Rename class: 'class myClass' → 'class MyClass'".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "cs"
    }
}

#[derive(Debug, Clone)]
pub struct ShellEchoRule;

impl LanguageRule for ShellEchoRule {
    fn name(&self) -> &str {
        "shell-echo-quote"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("echo ") && !trimmed.starts_with("echo \"") && !trimmed.starts_with("echo '")
            {
                if trimmed.contains('$') && !trimmed.contains('"') {
                    messages.push(LintMessage::new(
                        line_num + 1,
                        line.find("echo").unwrap_or(0),
                        Severity::Warning,
                        "Unquoted variable in echo".to_string(),
                        self.name().to_string(),
                        Some("Quote variables: 'echo $VAR' → 'echo \"$VAR\"' to prevent word splitting".to_string()),
                    ));
                }
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "sh" | "bash")
    }
}

#[derive(Debug, Clone)]
pub struct SqlSelectStarRule;

impl LanguageRule for SqlSelectStarRule {
    fn name(&self) -> &str {
        "sql-no-select-star"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"(?i)SELECT\s+\*").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("--") && !line.trim_start().starts_with("/*") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("SELECT").or(line.find("select")).unwrap_or(0),
                    Severity::Info,
                    "SELECT * usage detected".to_string(),
                    self.name().to_string(),
                    Some("List explicit columns: 'SELECT *' → 'SELECT id, name, created_at' for clarity and performance".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "sql"
    }
}

#[derive(Debug, Clone)]
pub struct LuaPrintRule;

impl LanguageRule for LuaPrintRule {
    fn name(&self) -> &str {
        "no-lua-print"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bprint\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("--") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    Severity::Warning,
                    "print() call found".to_string(),
                    self.name().to_string(),
                    Some("Use a logging library or remove before production: require('log') or similar".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "lua"
    }
}

#[derive(Debug, Clone)]
pub struct ScalaPrintRule;

impl LanguageRule for ScalaPrintRule {
    fn name(&self) -> &str {
        "no-scala-println"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"println\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("println").unwrap_or(0),
                    Severity::Warning,
                    "println() call found".to_string(),
                    self.name().to_string(),
                    Some("Use slf4j: LoggerFactory.getLogger(getClass).info(\"message\")".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "scala"
    }
}

#[derive(Debug, Clone)]
pub struct RPrintRule;

impl LanguageRule for RPrintRule {
    fn name(&self) -> &str {
        "no-r-print"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"\bprint\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("#") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("print").unwrap_or(0),
                    Severity::Info,
                    "print() call found".to_string(),
                    self.name().to_string(),
                    Some("Use message() for user output or cat() with appropriate formatting".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "r"
    }
}

#[derive(Debug, Clone)]
pub struct ZigDebugPrintRule;

impl LanguageRule for ZigDebugPrintRule {
    fn name(&self) -> &str {
        "no-zig-debug-print"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"std\.debug\.print").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("//") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("std.debug.print").unwrap_or(0),
                    Severity::Warning,
                    "std.debug.print detected".to_string(),
                    self.name().to_string(),
                    Some("Remove debug prints before release or use std.log for structured logging".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        extension == "zig"
    }
}

#[derive(Debug, Clone)]
pub struct HtmlInlineStyleRule;

impl LanguageRule for HtmlInlineStyleRule {
    fn name(&self) -> &str {
        "html-no-inline-style"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r#"style\s*=\s*["']"#).unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("style=").unwrap_or(0),
                    Severity::Info,
                    "Inline style detected".to_string(),
                    self.name().to_string(),
                    Some("Move styles to CSS: use class=\"my-class\" and define in <style> or external .css file".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "html" | "htm")
    }
}

#[derive(Debug, Clone)]
pub struct HtmlMissingAltRule;

impl LanguageRule for HtmlMissingAltRule {
    fn name(&self) -> &str {
        "html-img-alt"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let img_pattern = Regex::new(r"<img\s+[^>]*>").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if img_pattern.is_match(line) && !line.to_lowercase().contains("alt=") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("<img").unwrap_or(0),
                    Severity::Warning,
                    "img tag missing alt attribute".to_string(),
                    self.name().to_string(),
                    Some("Add alt text for accessibility: <img src=\"x.png\" alt=\"Description of image\">".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "html" | "htm")
    }
}

#[derive(Debug, Clone)]
pub struct CssImportantRule;

impl LanguageRule for CssImportantRule {
    fn name(&self) -> &str {
        "css-avoid-important"
    }

    fn check(&self, content: &str, _file_path: &Path) -> Vec<LintMessage> {
        let mut messages = Vec::new();
        let pattern = Regex::new(r"!important").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if pattern.is_match(line) && !line.trim_start().starts_with("/*") {
                messages.push(LintMessage::new(
                    line_num + 1,
                    line.find("!important").unwrap_or(0),
                    Severity::Info,
                    "!important usage detected".to_string(),
                    self.name().to_string(),
                    Some("Increase selector specificity instead: use .parent .child or BEM naming instead of !important".to_string()),
                ));
            }
        }

        messages
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "css" | "scss" | "sass")
    }
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
        assert!(messages.len() >= 1);
    }

    #[test]
    fn test_rule_set_html() {
        let rule_set = LanguageRuleSet::new();
        let content = r#"<img src="x.png"><div style="x">"#;
        let messages = rule_set.check(content, Path::new("test.html"));
        assert_eq!(messages.len(), 2);
    }
}
