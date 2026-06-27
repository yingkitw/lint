/// Maps a built-in rule name to its short letter-number code.
/// Returns the name itself as a fallback for unknown/custom rules.
pub fn code_from_name(name: &str) -> &str {
    match name {
        "line-length" => "W001",
        "trailing-whitespace" => "W002",
        "no-todo" => "W003",
        "no-empty-file" => "E001",
        "no-consecutive-empty-lines" => "W004",
        "no-tabs" => "W005",
        "final-newline" => "W006",
        "no-mixed-line-endings" => "W007",
        "hardcoded-secret" => "S001",
        "unsafe-eval" => "S002",
        "sql-injection-risk" => "S003",
        "max-nesting-depth" => "E002",
        "max-function-lines" => "E003",
        "sort-imports" => "W008",
        // Language-specific
        "no-console-log" => "L001",
        "no-var" => "L002",
        "no-print" => "L003",
        "python-style" => "L004",
        "go-style" => "L005",
        "java-style" => "L006",
        "no-unwrap" => "L007",
        "no-expect" => "L008",
        "missing-semicolon" => "L009",
        "no-puts" => "L010",
        "ruby-style" => "L011",
        "no-echo" => "L012",
        "no-swift-print" => "L013",
        "kotlin-style" => "L014",
        "no-dart-print" => "L015",
        "no-csharp-console" => "L016",
        "csharp-style" => "L017",
        "shell-echo-quote" => "L018",
        "sql-no-select-star" => "L019",
        "no-lua-print" => "L020",
        "no-scala-println" => "L021",
        "no-r-print" => "L022",
        "no-zig-debug-print" => "L023",
        "html-no-inline-style" => "L024",
        "html-img-alt" => "L025",
        "css-avoid-important" => "L026",
        _ => name,
    }
}

/// Maps a short letter-number code back to the canonical rule name.
/// Returns the code itself as a fallback for unknown codes.
pub fn name_from_code(code: &str) -> &str {
    match code {
        "W001" => "line-length",
        "W002" => "trailing-whitespace",
        "W003" => "no-todo",
        "E001" => "no-empty-file",
        "W004" => "no-consecutive-empty-lines",
        "W005" => "no-tabs",
        "W006" => "final-newline",
        "W007" => "no-mixed-line-endings",
        "S001" => "hardcoded-secret",
        "S002" => "unsafe-eval",
        "S003" => "sql-injection-risk",
        "E002" => "max-nesting-depth",
        "E003" => "max-function-lines",
        "W008" => "sort-imports",
        "L001" => "no-console-log",
        "L002" => "no-var",
        "L003" => "no-print",
        "L004" => "python-style",
        "L005" => "go-style",
        "L006" => "java-style",
        "L007" => "no-unwrap",
        "L008" => "no-expect",
        "L009" => "missing-semicolon",
        "L010" => "no-puts",
        "L011" => "ruby-style",
        "L012" => "no-echo",
        "L013" => "no-swift-print",
        "L014" => "kotlin-style",
        "L015" => "no-dart-print",
        "L016" => "no-csharp-console",
        "L017" => "csharp-style",
        "L018" => "shell-echo-quote",
        "L019" => "sql-no-select-star",
        "L020" => "no-lua-print",
        "L021" => "no-scala-println",
        "L022" => "no-r-print",
        "L023" => "no-zig-debug-print",
        "L024" => "html-no-inline-style",
        "L025" => "html-img-alt",
        "L026" => "css-avoid-important",
        _ => code,
    }
}

pub fn plugin_rules(plugin: &str) -> Vec<String> {
    match plugin {
        "security" => vec![
            "hardcoded-secret".to_string(),
            "unsafe-eval".to_string(),
            "sql-injection-risk".to_string(),
        ],
        "javascript" => vec![
            "no-console-log".to_string(),
            "no-var".to_string(),
            "missing-semicolon".to_string(),
        ],
        "python" => vec!["no-print".to_string(), "python-style".to_string()],
        "rust" => vec![
            "no-unwrap".to_string(),
            "no-expect".to_string(),
            "missing-semicolon".to_string(),
        ],
        "html" => vec![
            "html-no-inline-style".to_string(),
            "html-img-alt".to_string(),
        ],
        "css" => vec!["css-avoid-important".to_string()],
        _ => Vec::new(),
    }
}

/// Returns all rule names belonging to a built-in category.
pub fn category_rules(category: &str) -> Vec<String> {
    match category {
        "style" => vec![
            "line-length".to_string(),
            "trailing-whitespace".to_string(),
            "final-newline".to_string(),
            "no-mixed-line-endings".to_string(),
            "no-tabs".to_string(),
            "no-consecutive-empty-lines".to_string(),
            "python-style".to_string(),
            "go-style".to_string(),
            "java-style".to_string(),
            "kotlin-style".to_string(),
            "ruby-style".to_string(),
            "csharp-style".to_string(),
            "missing-semicolon".to_string(),
            "html-no-inline-style".to_string(),
            "css-avoid-important".to_string(),
        ],
        "correctness" => vec![
            "no-todo".to_string(),
            "no-empty-file".to_string(),
            "no-console-log".to_string(),
            "no-var".to_string(),
            "no-print".to_string(),
            "no-unwrap".to_string(),
            "no-expect".to_string(),
            "no-puts".to_string(),
            "no-echo".to_string(),
            "no-swift-print".to_string(),
            "no-dart-print".to_string(),
            "no-csharp-console".to_string(),
            "no-lua-print".to_string(),
            "no-scala-println".to_string(),
            "no-r-print".to_string(),
            "no-zig-debug-print".to_string(),
            "sql-no-select-star".to_string(),
            "html-img-alt".to_string(),
            "shell-echo-quote".to_string(),
        ],
        "security" => vec![
            "hardcoded-secret".to_string(),
            "unsafe-eval".to_string(),
            "sql-injection-risk".to_string(),
        ],
        _ => Vec::new(),
    }
}

/// Returns true if `name` is a known rule category.
pub fn is_category(name: &str) -> bool {
    matches!(name, "style" | "correctness" | "security")
}

/// Returns all built-in rule names (generic + language).
pub fn known_rules() -> Vec<String> {
    vec![
        // Generic
        "line-length".to_string(),
        "trailing-whitespace".to_string(),
        "no-todo".to_string(),
        "no-empty-file".to_string(),
        "no-consecutive-empty-lines".to_string(),
        "no-tabs".to_string(),
        "final-newline".to_string(),
        "no-mixed-line-endings".to_string(),
        "hardcoded-secret".to_string(),
        "unsafe-eval".to_string(),
        "sql-injection-risk".to_string(),
        "max-nesting-depth".to_string(),
        "max-function-lines".to_string(),
        "sort-imports".to_string(),
        // Language
        "no-console-log".to_string(),
        "no-var".to_string(),
        "no-print".to_string(),
        "python-style".to_string(),
        "go-style".to_string(),
        "java-style".to_string(),
        "no-unwrap".to_string(),
        "no-expect".to_string(),
        "missing-semicolon".to_string(),
        "no-puts".to_string(),
        "ruby-style".to_string(),
        "no-echo".to_string(),
        "no-swift-print".to_string(),
        "kotlin-style".to_string(),
        "no-dart-print".to_string(),
        "no-csharp-console".to_string(),
        "csharp-style".to_string(),
        "shell-echo-quote".to_string(),
        "sql-no-select-star".to_string(),
        "no-lua-print".to_string(),
        "no-scala-println".to_string(),
        "no-r-print".to_string(),
        "no-zig-debug-print".to_string(),
        "html-no-inline-style".to_string(),
        "html-img-alt".to_string(),
        "css-avoid-important".to_string(),
    ]
}

/// Returns all built-in plugin names.
pub fn known_plugins() -> Vec<String> {
    vec![
        "security".to_string(),
        "javascript".to_string(),
        "python".to_string(),
        "rust".to_string(),
        "html".to_string(),
        "css".to_string(),
    ]
}
