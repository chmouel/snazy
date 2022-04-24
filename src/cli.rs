use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version=env!("CARGO_PKG_VERSION"), bin_name="snazy", long_about = None)]
pub struct Cli {
    // a regexp to match against the log line
    #[clap(
        short,
        long,
        help = "highlight word in a message with a regexp",
        value_name = "REGEXP",
        hide_default_value = true,
        default_value = ""
    )]
    pub regexp: String,

    #[clap(long, help = "Hide container prefix when showing kail")]
    pub kail_no_prefix: bool,

    #[clap(
        short,
        long,
        help = "Time format",
        value_name = "FORMAT",
        default_value = "%H:%M:%S"
    )]
    pub time_format: String,

    #[clap(
        short,
        long,
        multiple_values = true,
        value_delimiter = ',',
        help = "filter levels separated by commas, eg: info,debug"
    )]
    pub filter_levels: Vec<String>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
