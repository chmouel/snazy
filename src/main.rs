mod app;
mod parse;
mod utils;

use std::env;

fn main() {
    let matches = crate::app::build_app().get_matches_from(env::args_os());
    crate::parse::read_from_stdin(&matches)
}
