#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![forbid(unsafe_code)]

use std::sync::Arc;

mod cli;
mod config;
mod parse;
mod utils;

#[cfg(test)]
mod parse_test;

fn main() {
    let config = cli::build_cli_config();
    if config.files.is_some() {
        parse::read_from_files(&Arc::new(config));
    } else {
        parse::read_from_stdin(&Arc::new(config));
    }
}
