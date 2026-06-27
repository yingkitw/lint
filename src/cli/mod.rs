use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod output;

#[derive(Parser)]
#[command(name = "lint")]
#[command(about = "A versatile linting tool with multiple interfaces", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    #[command(alias = "check")]
    Lint {
        #[arg(value_name = "PATHS")]
        paths: Vec<PathBuf>,

        #[arg(short, long, visible_alias = "output-format", value_name = "FORMAT")]
        output: Option<String>,

        #[arg(short, long, visible_alias = "line-length", value_name = "LENGTH")]
        max_line_length: Option<usize>,

        #[arg(short = 'r', long)]
        rules: Option<Vec<String>>,

        #[arg(long)]
        fix: bool,

        #[arg(long)]
        watch: bool,

        #[arg(long)]
        cache: bool,

        #[arg(long)]
        quiet: bool,

        #[arg(long, value_name = "N")]
        max_warnings: Option<usize>,

        #[arg(long, value_name = "WHEN")]
        color: Option<String>,

        #[arg(long, value_name = "RULES", value_delimiter = ',')]
        select: Option<Vec<String>>,

        #[arg(long, value_name = "RULES", value_delimiter = ',')]
        ignore: Option<Vec<String>>,

        #[arg(long)]
        print_files: bool,

        #[arg(long, value_name = "PATH")]
        output_file: Option<PathBuf>,

        #[arg(long)]
        stdin: bool,

        #[arg(long, visible_alias = "stdin-file-path", value_name = "FILENAME")]
        stdin_filename: Option<String>,

        #[arg(long)]
        exit_zero: bool,

        #[arg(long)]
        statistics: bool,

        #[arg(long)]
        show_fixes: bool,

        #[arg(long, value_name = "PATTERN")]
        exclude: Option<Vec<String>>,

        #[arg(long)]
        show_settings: bool,

        #[arg(long, value_name = "PATH")]
        cache_location: Option<PathBuf>,

        #[arg(long)]
        diff: bool,

        #[arg(long)]
        fix_only: bool,

        #[arg(long)]
        add_noqa: bool,

        #[arg(long)]
        ignore_suppressions: bool,

        #[arg(long)]
        select_all: bool,

        #[arg(long)]
        no_ignore: bool,

        #[arg(long, value_name = "STRATEGY")]
        cache_strategy: Option<String>,

        #[arg(long)]
        no_error_on_unmatched_pattern: bool,

        #[arg(long)]
        deny_warnings: bool,

        #[arg(long)]
        exit_non_zero_on_fix: bool,

        #[arg(long)]
        no_cache: bool,

        #[arg(long)]
        no_config: bool,

        #[arg(long)]
        check: bool,

        #[arg(long, value_name = "N")]
        max_diagnostics: Option<usize>,

        #[arg(long, value_name = "PATH")]
        baseline: Option<PathBuf>,

        #[arg(long)]
        force_exclude: bool,

        #[arg(long, value_name = "EXT", use_value_delimiter = true)]
        ext: Option<Vec<String>>,

        #[arg(long)]
        verbose: bool,

        #[arg(long)]
        unsafe_fixes: bool,

        #[arg(long, value_name = "RULES", use_value_delimiter = true)]
        fixable: Option<Vec<String>>,

        #[arg(long, value_name = "RULES", use_value_delimiter = true)]
        unfixable: Option<Vec<String>>,

        #[arg(long)]
        preview: bool,

        #[arg(long)]
        no_show_source: bool,

        #[arg(long)]
        no_gitignore: bool,

        #[arg(long)]
        changed: bool,

        #[arg(long)]
        staged: bool,

        #[arg(long, value_name = "PLUGIN", use_value_delimiter = true)]
        plugin: Option<Vec<String>>,

        #[arg(long)]
        progress: bool,

        #[arg(long)]
        validate_config: bool,

        #[arg(long, value_name = "DEPTH")]
        max_nesting_depth: Option<usize>,

        #[arg(long, value_name = "LINES")]
        max_function_lines: Option<usize>,

        #[arg(long)]
        profile: bool,
    },
    ListRules,
    Version,
    Explain {
        rule: String,
    },
    Init,
}

pub struct RunOptions<'a> {
    pub fix: bool,
    pub fix_only: bool,
    pub diff: bool,
    pub check: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub statistics: bool,
    pub show_fixes: bool,
    pub add_noqa: bool,
    pub exit_zero: bool,
    pub exit_non_zero_on_fix: bool,
    pub deny_warnings: bool,
    pub unsafe_fixes: bool,
    pub max_warnings: Option<usize>,
    pub max_diagnostics: Option<usize>,
    pub output_format: &'a lint::OutputFormat,
    pub output_file: Option<&'a PathBuf>,
    pub baseline: Option<&'a PathBuf>,
    pub fixable: Option<&'a [String]>,
    pub unfixable: Option<&'a [String]>,
    pub progress: bool,
    pub profile: bool,
    pub per_dir_configs: std::collections::HashMap<PathBuf, lint::Config>,
}

impl<'a> RunOptions<'a> {
    pub fn effective_fix(&self) -> bool {
        self.fix || self.fix_only
    }
}
