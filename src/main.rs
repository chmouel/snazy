mod cli;
mod config;
mod parse;
mod utils;

use crate::config::Config;
use atty::Stream;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use yansi::{Color, Paint};

fn construct_config(matches: clap::ArgMatches) -> Config {
    let interactive_terminal = atty::is(Stream::Stdout);
    let colored_output = match matches.value_of("color") {
        Some("always") => true,
        Some("never") => false,
        _ => env::var_os("NO_COLOR").is_none() && interactive_terminal,
    };
    if colored_output {
        Paint::enable();
    } else {
        Paint::disable();
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
            regexp_colours.insert(regexp.to_string(), colours[i % colours.len()]);
        }
    }

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
    let mut level_symbols = false;
    // if the env SNAZY_LEVEL_SYMBOLS is set, we use it to set the level symbols
    if let Ok(level_symbols_env) = env::var("SNAZY_LEVEL_SYMBOLS") {
        level_symbols = level_symbols_env.parse::<bool>().unwrap();
    }
    if matches.is_present("level-symbols") {
        level_symbols = true;
    }

    Config {
        kail_no_prefix: matches.is_present("kail-no-prefix"),
        filter_levels: matches
            .values_of("filter-levels")
            .map(|v| v.map(String::from).collect())
            .unwrap_or_else(Vec::new),
        time_format: matches.value_of("time_format").unwrap().to_string(),
        regexp_colours,
        colored_output,
        level_symbols,
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
    }
}

fn main() {
    let matches = cli::build_cli().get_matches_from(env::args_os());
    let config = construct_config(matches);
    parse::read_from_stdin(Arc::new(config))
}
