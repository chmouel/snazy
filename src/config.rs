use std::collections::HashMap;

use yansi::Color;

/// Configuration options for *snazy*.
#[derive(Debug)]
pub struct Config {
    pub kail_no_prefix: bool,
    pub time_format: String,
    pub colored_output: bool,
    pub filter_levels: Vec<String>,
    pub regexp_colours: HashMap<String, Color>,
    pub json_keys: HashMap<String, String>,
    pub level_symbols: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            kail_no_prefix: false,
            time_format: String::from("%H:%M:%S"),
            colored_output: false,
            filter_levels: Vec::new(),
            regexp_colours: HashMap::new(),
            json_keys: HashMap::new(),
            level_symbols: bool::default(),
        }
    }
}
