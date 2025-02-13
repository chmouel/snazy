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

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum ColorWhen {
    /// show colors if the output goes to an interactive console (default)
    Auto,
    /// always use colorized output
    Always,
    /// do not use colorized output
    Never,
}

#[derive(Debug)]
pub struct Config {
    pub action_command: Option<String>,
    pub action_regexp: Option<String>,
    pub files: Option<Vec<String>>,
    pub filter_levels: Vec<LogLevel>,
    pub json_keys: HashMap<String, String>,
    pub kail_no_prefix: bool,
    pub kail_prefix_format: String,
    pub level_symbols: bool,
    pub regexp_colours: HashMap<String, Style>,
    pub skip_line_regexp: Vec<String>,
    pub time_format: String,
    pub timezone: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            files: Some(vec![]),
            kail_no_prefix: false,
            kail_prefix_format: String::from("{namespace}/{pod}[{container}]"),
            time_format: String::from("%H:%M:%S"),
            timezone: None,
            filter_levels: <Vec<LogLevel>>::new(),
            regexp_colours: HashMap::new(),
            json_keys: HashMap::new(),
            level_symbols: bool::default(),
            action_regexp: Some(String::new()),
            action_command: Some(String::new()),
            skip_line_regexp: Vec::new(),
        }
    }
}
