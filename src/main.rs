mod app;
mod parse;
mod utils;

use atty::Stream;
use std::env;
use yansi::Paint;

fn main() {
    let matches = crate::app::build_app().get_matches_from(env::args_os());
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
    crate::parse::read_from_stdin(&matches)
}
