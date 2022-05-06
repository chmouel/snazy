use std::collections::HashMap;

use yansi::Color;

/// Configuration options for *snazy*.
#[derive(Debug)]
pub struct Config {
    pub files: Vec<String>,
    pub kail_no_prefix: bool,
    pub time_format: String,
    pub colored_output: bool,
    pub filter_level: Vec<String>,
    pub regexp_colours: HashMap<String, Color>,
    pub json_keys: HashMap<String, String>,
    pub level_symbols: bool,
    pub kail_prefix_format: String,
    pub action_regexp: String,
    pub action_command: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            files: vec![],
            kail_no_prefix: false,
            kail_prefix_format: String::from("{namespace}/{pod}[{container}]"),
            time_format: String::from("%H:%M:%S"),
            colored_output: false,
            filter_level: Vec::new(),
            regexp_colours: HashMap::new(),
            json_keys: HashMap::new(),
            level_symbols: bool::default(),
            action_regexp: String::new(),
            action_command: String::new(),
        }
    }
}
