use clap::{Arg, Command};

pub fn build_app() -> Command<'static> {
    let app = Command::new("snazy")
        .arg(
            Arg::new("regexp")
                .long("regexp")
                .short('r')
                .help("highlight word in a message with a regexp"),
        )
        .arg(
            Arg::new("time_format")
                .long("time-format")
                .help("Time format")
                .default_value("%H:%M:%S"),
        )
        .arg(
            Arg::new("filter-levels")
                .min_values(1)
                .multiple_values(true)
                .help("filter levels separated by commas, eg: info,debug")
                .short('f')
                .long("filter-levels"),
        )
        .arg(
            Arg::new("kail-no-prefix")
                .long("kail-no-prefix")
                .help("Hide container prefix when showing kail"),
        );
    app
}
