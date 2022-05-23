#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use atty::Stream;
use yansi::{Color, Paint};

use crate::config::Config;

mod cli;
mod config;
mod parse;
mod utils;

#[cfg(test)]
mod parse_test;

fn construct_config(matches: &clap::ArgMatches) -> Config {
    let interactive_terminal = atty::is(Stream::Stdout);
    let colored_output = match matches.value_of("color") {
        Some("always") => true,
        Some("never") => false,
        _ => env::var_os("NO_COLOR").is_none() && interactive_terminal,
    };
    // enable colored output if set
    if colored_output {
        Paint::enable();
    } else {
        Paint::disable();
    }

    let mut kail_prefix_format = matches.value_of("kail-prefix-format").unwrap().to_string();
    if matches.occurrences_of("kail-prefix-format") > 0 {
        kail_prefix_format = matches.value_of("kail-prefix-format").unwrap().to_string();
    } else if env::var("SNAZY_KAIL_PREFIX_FORMAT").is_ok() {
        kail_prefix_format = env::var("SNAZY_KAIL_PREFIX_FORMAT").unwrap();
    }

    let mut regexp_colours = HashMap::new();
    let mut json_values = HashMap::new();
    if matches.occurrences_of("regexp") > 0 {
        let colours = vec![
            Color::Yellow,
            Color::Magenta,
            Color::Cyan,
            Color::Red,
            Color::Blue,
        ];
        let regexps: Vec<&str> = matches.values_of("regexp").unwrap().collect();
        // assign a colour to each regexp
        for (i, regexp) in regexps.iter().enumerate() {
            regexp_colours.insert((*regexp).to_string(), colours[i % colours.len()]);
        }
    }

    // collect match to a vec
    let skip_line_regexp: Vec<regex::Regex> = matches
        .values_of("skip-line-regexp")
        .unwrap_or_default()
        .map(|s| regex::Regex::new(s).unwrap())
        .collect();

    if matches.occurrences_of("json-keys") > 0 {
        // split all json-keys matches by =
        let json_keys: Vec<&str> = matches
            .values_of("json-keys")
            .unwrap()
            .flat_map(|s| s.split('='))
            .collect();
        if !json_keys.contains(&"level")
            || !json_keys.contains(&"msg")
            || !json_keys.contains(&"ts")
        {
            eprintln!("you should have multiple json-keys containning a match for the keys 'level', 'msg' and 'ts'");
            std::process::exit(1);
        }
    }
    Config {
        kail_prefix_format,
        files: matches
            .values_of("files")
            .map(|v| v.map(String::from).collect())
            .unwrap_or_else(Vec::new),
        kail_no_prefix: matches.is_present("kail-no-prefix"),
        filter_level: matches
            .values_of("filter-level")
            .map(|v| v.map(String::from).collect())
            .unwrap_or_else(Vec::new),
        time_format: matches
            .value_of("time_format")
            .map(String::from)
            .or_else(|| env::var("SNAZY_TIME_FORMAT").ok())
            .and_then(|t| t.parse().ok())
            .unwrap_or_default(),
        regexp_colours,
        colored_output,
        skip_line_regexp,
        level_symbols: matches.is_present("level-symbols")
            || env::var("SNAZY_LEVEL_SYMBOLS").is_ok(),
        // split json keys by '=' and store in a key, value hashmap
        json_keys: matches
            .values_of("json-keys")
            .map(|v| {
                for s in v {
                    let mut parts = s.splitn(2, '=');
                    let key = parts.next().unwrap().to_string();
                    let value = parts.next().unwrap().to_string();
                    json_values.insert(value, key);
                }
                json_values
            })
            .unwrap_or_else(HashMap::new),
        action_regexp: matches
            .value_of("action-regexp")
            .map(String::from)
            .unwrap_or_default(),
        action_command: matches
            .value_of("action-command")
            .map(String::from)
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_construct_config() {
        let args = vec![
            "snazy",
            "--regexp",
            "foo.*",
            "--time-format",
            "%S",
            "-f",
            "info",
            "--json-keys",
            "level=level",
            "--json-keys",
            "msg=msg",
            "--json-keys",
            "ts=ts",
            "test.log",
        ];

        let matches = cli::build().get_matches_from(args);
        let config = construct_config(&matches);
        assert_eq!(config.files, vec!["test.log".to_string()]);
        assert!(!config.kail_no_prefix);
        assert_eq!(config.filter_level, vec!["info".to_string()]);
        assert_eq!(config.time_format, "%S".to_string());
        assert_eq!(config.json_keys.len(), 3);
        assert_eq!(config.action_regexp, "".to_string());
        assert_eq!(config.action_command, "".to_string());
    }
}

fn main() {
    let matches = cli::build().get_matches_from(env::args_os());
    let config = construct_config(&matches);
    if config.files.is_empty() {
        parse::read_from_stdin(&Arc::new(config));
    } else {
        parse::read_from_files(&Arc::new(config));
    }
}
