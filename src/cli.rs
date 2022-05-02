use clap::{crate_version, Arg, ColorChoice, Command};

pub fn build() -> Command<'static> {
    let clap_color_choice = if std::env::var_os("NO_COLOR").is_none() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    Command::new("snazy")
        .version(crate_version!())
        .color(clap_color_choice)
        .after_help(
            "You just need to pipe to snazy some logs formatted as json to humm (sorry) snazzy them 💄,eg:\n\
            kubectl logs -f controller-pod|snazy\n\
            Note: `snazy -h` prints a short and concise overview while `snazy --help` gives all details.",
        )
        .arg(
            Arg::new("regexp")
                .long("regexp")
                .short('r')
                .help("highlight word in a message with a regexp")
                .takes_value(true)
                .min_values(1)
                .multiple_occurrences(true)
                .long_help(
                    "Specify one or multiple regexps to highligh in message.\n\
                    each regexp match will be colored with a different color"
                ),
        )
        .arg(
            Arg::new("filter-levels")
            .help("filter by levels")
            .long_help("You can have multiple -f if you need to filter by multiple levels")
            .takes_value(true)
            .min_values(1)
            .multiple_occurrences(true)
            .short('f')
            .possible_values(&["info", "debug", "warning", "error", "info"])
            .long("filter-levels"),
        )
        .arg(
            Arg::new("time_format")
                .long("time-format")
                .help("Time format")
                .default_value("%H:%M:%S")
                .takes_value(true)
                .long_help(
                    "Specify a timeformat as documented in the strftime(3) manpage.\n\
                     You can set the environement variable `SNAZY_TIME_FORMAT` to have it set permanently."
                ),
        )
        .arg(
            Arg::new("kail-prefix-format")
                .long("kail-prefix-format")
                .help("Kail prefix format")
                .default_value("{namespace}/{pod}[{container}]")
                .takes_value(true)
                .long_help(
                    "Set the format on how to print the kail prefix.\n\
                     The {namespace}, {pod} and {container} tags will be replaced by their values \n\
                     You can set the enviroment variable `SNAZY_KAIL_PREFIX_FORMAT` to have it set permanently."
                ),
        )
        .arg(
            Arg::new("kail-no-prefix")
                .long("kail-no-prefix")
                .help("Hide container prefix when showing the log with kail"),
        )
        .arg(
            Arg::new("level-symbols")
                .long("level-symbols")
                .long_help("This will replace the level with a pretty emoji instead of the label.\n\
                You can set the enviroment variable `SNAZY_LEVEL_SYMBOLS` to always have it.\n")
                .help("Replace log level with pretty symbols")
        )
        .arg(
            Arg::new("json-keys")
                .long("json-keys")
                .short('k')
                .help("key to use for json parsing")
                .takes_value(true)
                .multiple_occurrences(true)
                .long_help(
                    "Specify multiple keys for json parsing.\n\
                    keys needed are: msg (message), level (logging level), ts (timestamp).\n\
                    all keys are needed to be present, eg: \n\n\
                    `snazy -k msg=message -k level=level -k ts=ts`\n \n\
                    will parse the json and use the message, level and timestamp keys\n\
                    from the (`message`, `level`, `ts`) json keys."
                ),
        )
        .arg(
            Arg::new("action-regexp")
                .long("action-regexp")
                .help("A regexp to match for action")
                .takes_value(true)
                .multiple_occurrences(true)
                .long_help("The regexp to match for action")
        )
        .arg(
            Arg::new("action-command")
                .long("action-command")
                .help("An action command to launch when action-regexp match")
                .takes_value(true)
                .multiple_occurrences(true)
                .long_help("The comment to run after matching an action-regexp.\n\
                            The string {} will be expanded to the match.\n\
                            The command will be run with a `sh -c` ")
        )
        .arg(
            Arg::new("files")
                .multiple_occurrences(true)
                .help("files to read, if not specified, stdin is used")
                .takes_value(true)
                .long_help(
                    "Specify one or multiple files to read.\n\
                    If no files are specified, stdin is used.\n\
                    If no files are specified and stdin is not a tty, stdin is used.\n\
                    If no files are specified and stdin is a tty, stdin is used."
                )
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
        )
}
