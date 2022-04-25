use clap::{crate_version, Arg, ColorChoice, Command};

pub fn build_app() -> Command<'static> {
    let clap_color_choice = if std::env::var_os("NO_COLOR").is_none() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    let app = Command::new("snazy")
        .version(crate_version!())
        .color(clap_color_choice)
        .after_help(
            r#"You just need to pipe to snazy some logs formatted as json to humm (sorry) snazzy them 💄
eg: `kubectl logs -f controller-pod|snazy`"#
        )

        .arg(
            Arg::new("regexp")
                .long("regexp")
                .short('r')
                .help("highlight word in a message with a regexp")
                .takes_value(true)
                .min_values(1)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("time_format")
                .long("time-format")
                .help("Time format")
                .default_value("%H:%M:%S"),
        )
        .arg(
            Arg::new("filter-levels")
                .help("filter levels separated by commas, eg: info,debug")
                .short('f')
                .possible_values(&["info", "debug", "warning", "error", "info", "fatal", "panic", "dpanic"])
                .long("filter-levels"),
        )
        .arg(
            Arg::new("kail-no-prefix")
                .long("kail-no-prefix")
                .help("Hide container prefix when showing kail"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .short('c')
                .takes_value(true)
                .value_name("when")
                .possible_values(&["never", "auto", "always"])
                .hide_possible_values(true)
                .help("When to use colors: never, *auto*, always")
                .long_help(
                    "Declare when to use color for the pattern match output:\n  \
                       'auto':      show colors if the output goes to an interactive console (default)\n  \
                       'never':     do not use colorized output\n  \
                       'always':    always use colorized output",
                ),
        );
    app
}
