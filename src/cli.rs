use crate::config::{Config, LogLevel};
use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};
use std::collections::HashMap;
use std::io;
use yansi::{Color, Style};

// `cstr!` converts tags to ANSI codes
const AFTER_HELP: &str = color_print::cstr!(
    r#"<bold>Snazzy</bold> let you watch logs nicely.

It tries to be smart with Json logs by showing the levels,
the message and the date in a nice and visual way.

There is many more options to filter or highlight part of the logs or even launch some
actions when a match is found.

Try to stream some logs or specify a log file and let snazy, <red>snazzy them</red>!"#
);

/// Snazzy is a snazy log viewer
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    author,
    version,
    about,
    after_help = AFTER_HELP)]
struct Args {
    #[arg(short = 'r', long, verbatim_doc_comment)]
    /// regexp highlight
    ///
    /// Highlight a pattern in a message with a regexp
    pub regexp: Vec<String>,

    #[arg(short = 'S', long)]
    /// Skip a line matching a Regexp.
    ///
    /// Any lines matching the regexp, will be skipped to be printed.
    pub skip_line_regexp: Vec<String>,

    /// If provided, outputs the completion file for given shell
    #[arg(long, value_enum)]
    shell_completion: Option<Shell>,

    #[arg(short = 'f', long, verbatim_doc_comment)]
    /// Filter by log level
    ///
    /// Filter the json logs by log level. You can have multiple log levels.
    pub filter_levels: Vec<LogLevel>,

    #[clap(
        long,
        short = 'c',
        value_enum,
        default_value_t = crate::config::Coloring::Auto,
        value_name = "when",
        hide_possible_values = true,
        verbatim_doc_comment
    )]
    /// When to use colors
    ///
    /// 'auto':      show colors if the output goes to an interactive console (default)
    /// 'never':     do not use colorized output
    /// 'always':    always use colorized output
    pub color: crate::config::Coloring,

    #[arg(long, default_value = "%H:%M:%S", env = "SNAZY_TIME_FORMAT")]
    /// Filter by log level
    ///
    /// A timeformat as documented by the strftime(3) manpage.
    pub time_format: String,

    #[arg(long, env = "SNAZY_TIMEZONE")]
    /// Convert timestamps to specified timezone (e.g. `Europe/Paris`, `America/New_York`)
    pub timezone: Option<String>,

    #[arg(
        long,
        verbatim_doc_comment,
        default_value = "{namespace}/{pod}[{container}]",
        env = "SNAZY_KAIL_PREFIX_FORMAT"
    )]
    /// Set the format on how to print the kail prefix.
    ///
    /// The {namespace}, {pod} and {container} tags will be replaced by their
    /// values."
    pub kail_prefix_format: String,

    #[arg(long, action(clap::ArgAction::SetTrue))]
    /// Hide container prefix when showing the log with kail
    pub kail_no_prefix: bool,

    /// Pretty emojis instead of boring text level
    #[arg(long, action(clap::ArgAction::SetTrue), env = "SNAZY_LEVEL_SYMBOLS")]
    pub level_symbols: bool,

    #[arg(short = 'k', long, verbatim_doc_comment)]
    /// Keys / Values for JSON Parsing
    ///
    /// The keys needed to be passed are: msg (message), level (logging level),
    /// ts (timestamp).
    ///
    /// For example:
    ///
    /// `snazy -k msg=message -k level=level -k ts=ts`
    ///
    /// will parse the JSON log file and use the (`message`, `level`, `ts`) keys
    /// from the json as (`msg`), (`level`) and (`ts`) for snazy.
    pub json_keys: Vec<String>,

    #[arg(long, verbatim_doc_comment)]
    ///  A regexp to match an action on.
    ///
    ///  You can have an action matching a Regexp.
    ///  A good example is when  you have to have a notification on your desktop
    ///  when there is a match in a log.
    pub action_regexp: Option<String>,

    #[arg(long, verbatim_doc_comment)]
    ///  The command to run when a regexp match the --action-match
    pub action_command: Option<String>,

    #[arg(long, action(clap::ArgAction::SetTrue), env = "SNAZY_HIDE_STACKTRACE")]
    /// Hide stacktraces in the log output
    pub hide_stacktrace: bool,

    #[arg(long, action(clap::ArgAction::SetTrue), env = "SNAZY_EXTRA_FIELDS")]
    /// Include all available fields from JSON logs
    pub extra_fields: bool,

    #[arg(long, value_delimiter = ',', env = "SNAZY_INCLUDE_FIELDS")]
    /// Include specific fields from JSON logs (comma-separated)
    pub include_fields: Vec<String>,

    #[arg(value_hint = ValueHint::FilePath)]
    files: Option<Vec<String>>,
}

fn regexp_colorize(regexps: &[String]) -> HashMap<String, Style> {
    let mut regexp_colours = HashMap::new();
    let colours = [
        Color::Cyan,
        Color::Yellow,
        Color::Red,
        Color::Magenta,
        Color::Blue,
    ];
    for (i, regexp) in regexps.iter().enumerate() {
        let defchosen = colours[i % colours.len()];
        let mut foreground = defchosen;
        let mut background = None;
        let mut reg = regexp.to_string();
        if let Some(colour) = regexp.split(':').next() {
            if colour.contains("bg=") && colour.contains("fg=") && colour.split(',').count() == 2 {
                let parts: Vec<&str> = colour.split(',').collect();
                for part in parts {
                    if let Some(colorsss) = part.strip_prefix("bg=") {
                        background = Some(parse_color(colorsss));
                    } else if let Some(colorsss) = part.strip_prefix("fg=") {
                        foreground = parse_color(colorsss);
                    }
                }
            } else if colour.split(',').count() == 3 {
                let mut parts = colour.split(',');
                let r = parts.next().unwrap().parse::<u8>().unwrap();
                let g = parts.next().unwrap().parse::<u8>().unwrap();
                let b = parts.next().unwrap().parse::<u8>().unwrap();
                foreground = Color::Rgb(r, g, b);
            } else if let Ok(col) = colour.parse::<u8>() {
                foreground = Color::Fixed(col);
            } else {
                foreground = match_color(colour, defchosen);
            }
            reg = regexp.replace(format!("{colour}:").as_str(), "");
        }
        let mut style = Style::new().fg(foreground);
        if let Some(bg) = background {
            style = style.bg(bg);
        }
        regexp_colours.insert(reg, style);
    }
    regexp_colours
}

fn parse_color(color: &str) -> Color {
    if color.split(',').count() == 3 {
        let mut parts = color.split(',');
        let r = parts.next().unwrap().parse::<u8>().unwrap();
        let g = parts.next().unwrap().parse::<u8>().unwrap();
        let b = parts.next().unwrap().parse::<u8>().unwrap();
        Color::Rgb(r, g, b)
    } else if let Ok(col) = color.parse::<u8>() {
        Color::Fixed(col)
    } else {
        match_color(color, Color::Primary)
    }
}

fn match_color(color: &str, default: Color) -> Color {
    match color.to_lowercase().as_str() {
        "yellow" => Color::Yellow,
        "cyan" => Color::Cyan,
        "red" => Color::Red,
        "magenta" => Color::Magenta,
        "blue" => Color::Blue,
        "green" => Color::Green,
        "white" => Color::White,
        "black" => Color::Black,
        "grey" => Color::Rgb(128, 128, 128),
        _ => default,
    }
}

/// Return a `HashMap` of a vector of splited by = string
fn make_json_keys(json_keys: &[String]) -> HashMap<String, String> {
    let ret: HashMap<String, String> = json_keys
        .iter()
        .map(|s| {
            let mut parts = s.splitn(2, '=');
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap().to_string();
            (key, value)
        })
        .collect();
    ret
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn build_cli_config() -> Config {
    let args = Args::parse();

    if let Some(generator) = args.shell_completion {
        let mut cmd = Args::command();
        print_completions(generator, &mut cmd);
        std::process::exit(0)
    }

    if !args.json_keys.is_empty() && args.json_keys.len() != 3 {
        eprintln!("you should have multiple json-keys containning a match for the keys 'level', 'msg' and 'ts'");
        std::process::exit(1);
    }

    let regexp_colours = regexp_colorize(&args.regexp);
    let coloring = args.color;
    if coloring == crate::config::Coloring::Never {
        yansi::disable();
    }
    let json_keys = make_json_keys(&args.json_keys);

    Config {
        level_symbols: if args.level_symbols {
            crate::config::LevelSymbols::Emoji
        } else {
            crate::config::LevelSymbols::Text
        },
        kail_prefix_format: args.kail_prefix_format,
        kail_prefix: if args.kail_no_prefix {
            crate::config::KailPrefix::Hide
        } else {
            crate::config::KailPrefix::Show
        },
        time_format: args.time_format,
        timezone: args.timezone,
        skip_line_regexp: args.skip_line_regexp,
        filter_levels: args.filter_levels,
        action_command: args.action_command,
        action_regexp: args.action_regexp,
        files: args.files,
        regexp_colours,
        json_keys,
        hide_stacktrace: args.hide_stacktrace,
        coloring,
        extra_fields: args.extra_fields,
        include_fields: args.include_fields,
    }
}
