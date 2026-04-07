#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![forbid(unsafe_code)]

use std::sync::Arc;

mod app;
mod cli;
mod config;
mod input;
mod model;
mod parser;
mod pipeline;
mod render;
mod utils;

fn main() {
    let config = cli::build_cli_config();
    if config.files.is_some() {
        input::read_from_files(&Arc::new(config));
    } else {
        input::read_from_stdin(&Arc::new(config));
    }
}
