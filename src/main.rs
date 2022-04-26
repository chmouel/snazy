mod app;
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

    Config {
        kail_no_prefix: matches.is_present("kail-no-prefix"),
        filter_levels: matches
            .values_of("filter-levels")
            .map(|v| v.map(String::from).collect())
            .unwrap_or_else(Vec::new),
        time_format: matches.value_of("time_format").unwrap().to_string(),
        regexp_colours,
        colored_output,
    }
}

fn main() {
    let matches = crate::app::build_app().get_matches_from(env::args_os());
    let config = construct_config(matches);
    crate::parse::read_from_stdin(Arc::new(config))
}
