use std::collections::HashMap;

use clap::ValueEnum;
use yansi::Style;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Debug,
    Warning,
    Error,
    Fatal,
}

pub fn level_from_str(level: &str) -> &'static LogLevel {
    match level {
        "debug" => &LogLevel::Debug,
        "warn" | "warning" => &LogLevel::Warning,
        "err" | "error" => &LogLevel::Error,
        "fatal" => &LogLevel::Fatal,
        _ => &LogLevel::Info,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum KailPrefix {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LevelSymbols {
    Emoji,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Coloring {
    Always,
    Never,
    Auto,
}

#[derive(Debug, Clone)]
/// NOTE: Clippy warns about excessive bools in this struct. Refactored to enums for clarity.
pub struct Config {
    pub action_command: Option<String>,
    pub action_regexp: Option<String>,
    pub files: Option<Vec<String>>,
    pub filter_levels: Vec<LogLevel>,
    pub json_keys: HashMap<String, String>,
    pub kail_prefix_format: String,
    pub kail_prefix: KailPrefix,
    pub level_symbols: LevelSymbols,
    pub regexp_colours: HashMap<String, Style>,
    pub skip_line_regexp: Vec<String>,
    pub time_format: String,
    pub timezone: Option<String>,
    pub hide_stacktrace: bool,
    pub coloring: Coloring,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            files: Some(vec![]),
            kail_prefix_format: String::from("{namespace}/{pod}[{container}]"),
            kail_prefix: KailPrefix::Show,
            time_format: String::from("%H:%M:%S"),
            timezone: None,
            filter_levels: <Vec<LogLevel>>::new(),
            regexp_colours: HashMap::new(),
            json_keys: HashMap::new(),
            level_symbols: LevelSymbols::Text,
            action_regexp: Some(String::new()),
            action_command: Some(String::new()),
            skip_line_regexp: Vec::new(),
            hide_stacktrace: false,
            coloring: Coloring::Auto,
        }
    }
}
