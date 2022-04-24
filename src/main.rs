mod cli;
mod parse;
mod utils;

fn main() {
    let cli = crate::cli::parse();
    crate::parse::read_from_stdin(cli)
}
