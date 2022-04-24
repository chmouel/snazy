mod cli;
mod parse;
mod utils;

use std::env;

fn main() {
    let matches = crate::cli::build_app().get_matches_from(env::args_os());
    crate::parse::read_from_stdin(&matches)
}
