use colored::Colorize;
use lint::OutputFormat;

pub fn severity_to_github_command(severity: lint::Severity) -> &'static str {
    match severity {
        lint::Severity::Error => "error",
        lint::Severity::Warning => "warning",
        lint::Severity::Info => "notice",
    }
}

pub fn escape_github_command(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

pub fn render_github(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    for result in results {
        let file = result.file_path.display().to_string();
        let file = escape_github_command(&file);
        for msg in &result.messages {
            let level = severity_to_github_command(msg.severity);
            let code = lint::rules::code_from_name(&msg.rule);
            let title = if code != msg.rule {
                format!("{} ({})", code, msg.rule)
            } else {
                msg.rule.clone()
            };
            out.push_str(&format!(
                "::{level} file={file},line={line},col={col},title={title}::{message}\n",
                level = level,
                file = file,
                line = msg.line,
                col = msg.column,
                title = escape_github_command(&title),
                message = escape_github_command(&msg.message),
            ));
        }
    }
    out
}

pub fn render_sarif(results: &[lint::LintResult]) -> String {
    use serde_json::json;

    let mut rules_seen = std::collections::HashSet::new();
    let mut sarif_results = Vec::new();

    for result in results {
        for msg in &result.messages {
            let level = match msg.severity {
                lint::Severity::Error => "error",
                lint::Severity::Warning => "warning",
                lint::Severity::Info => "note",
            };

            let code = lint::rules::code_from_name(&msg.rule);
            rules_seen.insert(code.to_string());

            sarif_results.push(json!({
                "ruleId": code,
                "level": level,
                "message": {
                    "text": msg.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": result.file_path.to_string_lossy()
                        },
                        "region": {
                            "startLine": msg.line,
                            "startColumn": msg.column
                        }
                    }
                }]
            }));
        }
    }

    let rules: Vec<_> = rules_seen
        .into_iter()
        .map(|code| {
            let name = lint::rules::name_from_code(&code);
            let short_desc = if name != code {
                format!("{} ({})", code, name)
            } else {
                code.clone()
            };
            json!({
                "id": code,
                "shortDescription": {
                    "text": short_desc
                },
                "defaultConfiguration": {
                    "level": "warning"
                }
            })
        })
        .collect();

    let doc = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "lint",
                    "informationUri": "https://github.com/yingkitw/lint",
                    "rules": rules
                }
            },
            "results": sarif_results
        }]
    });

    serde_json::to_string_pretty(&doc).unwrap()
}

pub fn render_junit(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<testsuites>\n");

    let total_tests = results.len();
    let total_failures: usize = results.iter().map(|r| r.messages.len()).sum();

    out.push_str(&format!(
        "  <testsuite name=\"lint\" tests=\"{}\" failures=\"{}\" errors=\"0\">\n",
        total_tests, total_failures
    ));

    for result in results {
        let file = result.file_path.to_string_lossy();
        out.push_str(&format!(
            "    <testcase name=\"{}\" classname=\"{}\" />\n",
            escape_xml(&file),
            escape_xml(&file)
        ));
        for msg in &result.messages {
            let code = lint::rules::code_from_name(&msg.rule);
            out.push_str(&format!(
                "    <testcase name=\"{}\" classname=\"{}\">\n",
                escape_xml(&msg.rule),
                escape_xml(&file)
            ));
            out.push_str(&format!(
                "      <failure message=\"{}\" type=\"{}\">\n",
                escape_xml(&msg.message),
                escape_xml(code)
            ));
            out.push_str(&format!(
                "        {}:{}:{} - {} [{}]\n",
                escape_xml(&file),
                msg.line,
                msg.column,
                escape_xml(&msg.message),
                escape_xml(code)
            ));
            out.push_str("      </failure>\n");
            out.push_str("    </testcase>\n");
        }
    }

    out.push_str("  </testsuite>\n");
    out.push_str("</testsuites>\n");
    out
}

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn render_gitlab(results: &[lint::LintResult]) -> String {
    use serde_json::json;

    let mut issues = Vec::new();
    for result in results {
        for msg in &result.messages {
            let severity = match msg.severity {
                lint::Severity::Error => "blocker",
                lint::Severity::Warning => "major",
                lint::Severity::Info => "info",
            };
            let code = lint::rules::code_from_name(&msg.rule);
            issues.push(json!({
                "description": msg.message,
                "check_name": code,
                "fingerprint": format!("{}:{}:{}", result.file_path.display(), msg.line, msg.column),
                "severity": severity,
                "location": {
                    "path": result.file_path.to_string_lossy(),
                    "lines": {
                        "begin": msg.line,
                        "end": msg.line
                    }
                }
            }));
        }
    }
    serde_json::to_string_pretty(&issues).unwrap()
}

pub fn render_concise(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    for result in results {
        for msg in &result.messages {
            out.push_str(&format!(
                "{}:{}:{} [{}] {} ({})\n",
                result.file_path.display(),
                msg.line,
                msg.column,
                msg.severity.as_str(),
                msg.message,
                msg.rule,
            ));
        }
    }
    out
}

pub fn render_grouped(results: &[lint::LintResult]) -> String {
    let mut out = String::new();
    for result in results {
        if result.messages.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "{}\n",
            format!("{}", result.file_path.display()).bold()
        ));
        for msg in &result.messages {
            let (prefix, color) = match msg.severity {
                lint::Severity::Error => ("error", colored::Color::Red),
                lint::Severity::Warning => ("warning", colored::Color::Yellow),
                lint::Severity::Info => ("info", colored::Color::Blue),
            };
            out.push_str(&format!(
                "  {}:{}:{} {} {}\n",
                msg.line,
                msg.column,
                prefix.color(color),
                msg.message,
                format!("({})", msg.rule).dimmed(),
            ));
        }
        out.push('\n');
    }
    out
}

pub fn render_results(
    results: &[lint::LintResult],
    format: &OutputFormat,
    quiet: bool,
    show_source: bool,
) -> String {
    match format {
        OutputFormat::Json => {
            if quiet {
                let filtered: Vec<_> = results
                    .iter()
                    .map(|r| {
                        let mut cr = r.clone();
                        cr.messages.retain(|m| m.severity == lint::Severity::Error);
                        cr
                    })
                    .collect();
                serde_json::to_string_pretty(&filtered).unwrap()
            } else {
                serde_json::to_string_pretty(results).unwrap()
            }
        }
        OutputFormat::Markdown => {
            let mut out = String::new();
            for result in results {
                let relevant: Vec<_> = if quiet {
                    result
                        .messages
                        .iter()
                        .filter(|m| m.severity == lint::Severity::Error)
                        .collect()
                } else {
                    result.messages.iter().collect()
                };
                if !relevant.is_empty() {
                    out.push_str(&format!("# {}\n", result.file_path.display()));
                    for msg in relevant {
                        out.push_str(&format!(
                            "- **Line {}**: [{}] {} (`{}`)\n",
                            msg.line,
                            msg.severity.as_str(),
                            msg.message,
                            msg.rule
                        ));
                        if let Some(suggestion) = &msg.suggestion {
                            out.push_str(&format!("  - **Fix**: {}\n", suggestion));
                        }
                    }
                    out.push('\n');
                }
            }
            out
        }
        OutputFormat::Github => {
            if quiet {
                let filtered: Vec<_> = results
                    .iter()
                    .map(|r| {
                        let mut cr = r.clone();
                        cr.messages.retain(|m| m.severity == lint::Severity::Error);
                        cr
                    })
                    .collect();
                render_github(&filtered)
            } else {
                render_github(results)
            }
        }
        OutputFormat::Sarif => render_sarif(results),
        OutputFormat::Junit => render_junit(results),
        OutputFormat::Concise => render_concise(results),
        OutputFormat::Gitlab => render_gitlab(results),
        OutputFormat::Grouped => render_grouped(results),
        OutputFormat::Text => {
            let mut out = String::new();
            let mut total_errors = 0;
            let mut total_warnings = 0;
            let mut total_infos = 0;

            for result in results {
                let relevant: Vec<_> = if quiet {
                    result
                        .messages
                        .iter()
                        .filter(|m| m.severity == lint::Severity::Error)
                        .collect()
                } else {
                    result.messages.iter().collect()
                };

                if relevant.is_empty() {
                    if !quiet && result.messages.is_empty() {
                        out.push_str(&format!(
                            "{}: No issues found\n",
                            result.file_path.display()
                        ));
                    }
                } else {
                    out.push_str(&format!(
                        "{}\n",
                        format!("{}", result.file_path.display()).bold()
                    ));
                    let lines: Vec<&str> = result.file_content.lines().collect();
                    for msg in relevant {
                        let (prefix, color) = match msg.severity {
                            lint::Severity::Error => {
                                total_errors += 1;
                                ("error", colored::Color::Red)
                            }
                            lint::Severity::Warning => {
                                total_warnings += 1;
                                ("warning", colored::Color::Yellow)
                            }
                            lint::Severity::Info => {
                                total_infos += 1;
                                ("info", colored::Color::Cyan)
                            }
                        };

                        let fix_indicator = if msg.fix.is_some() { "[*] " } else { "" };
                        let code = lint::rules::code_from_name(&msg.rule);
                        out.push_str(&format!(
                            "  {}:{} {}{}[{}] {}: {}\n",
                            msg.line,
                            msg.column,
                            fix_indicator.dimmed(),
                            if fix_indicator.is_empty() { "" } else { " " },
                            code.dimmed(),
                            prefix.color(color),
                            msg.message
                        ));

                        #[allow(clippy::collapsible_if)]
                        if show_source {
                            let line_idx = msg.line.saturating_sub(1);
                            if line_idx > 0 {
                                if let Some(prev) = lines.get(line_idx - 1) {
                                    out.push_str(&format!("    | {}\n", prev.dimmed()));
                                }
                            }
                            if let Some(source_line) = lines.get(line_idx) {
                                out.push_str(&format!("    | {}\n", source_line));
                                let caret =
                                    format!("    |{}^", " ".repeat(msg.column.saturating_sub(1)));
                                out.push_str(&format!("{}\n", caret.dimmed()));
                            }
                            if let Some(next) = lines.get(line_idx + 1) {
                                out.push_str(&format!("    | {}\n", next.dimmed()));
                            }
                        }

                        if let Some(suggestion) = &msg.suggestion {
                            out.push_str(&format!(
                                "    {} help: {}\n",
                                "→".dimmed(),
                                suggestion.dimmed()
                            ));
                        }
                    }
                    out.push('\n');
                }
            }

            if !quiet {
                let file_count = results.len();
                let summary = format!(
                    "Summary: {} errors, {} warnings, {} infos in {} files",
                    total_errors, total_warnings, total_infos, file_count
                );
                out.push_str(&format!("{}\n", summary.bold()));
            }
            out
        }
    }
}

pub fn print_results(
    results: &[lint::LintResult],
    format: &OutputFormat,
    quiet: bool,
    show_source: bool,
) {
    print!("{}", render_results(results, format, quiet, show_source));
}
