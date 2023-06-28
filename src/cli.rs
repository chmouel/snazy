use crate::config::{ColorWhen, Config, LogLevel};
use atty::Stream;
use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};
use std::collections::HashMap;
use std::{env, io};
use yansi::{Color, Paint};

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
        default_value_t = ColorWhen::Auto,
        value_name = "when",
        hide_possible_values = true,
        verbatim_doc_comment
    )]
    /// When to use colors
    ///
    /// 'auto':      show colors if the output goes to an interactive console (default)
    /// 'never':     do not use colorized output
    /// 'always':    always use colorized output
    pub color: ColorWhen,

    #[arg(long, default_value = "%H:%M:%S", env = "SNAZY_TIME_FORMAT")]
    /// Filter by log level
    ///
    /// A timeformat as documented by the strftime(3) manpage.
    pub time_format: String,

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

    #[arg(value_hint = ValueHint::FilePath)]
    files: Option<Vec<String>>,
}

fn regexp_colorize(regexps: &[String]) -> HashMap<String, Color> {
    let mut regexp_colours = HashMap::new();
    let colours = vec![
        Color::Cyan,
        Color::Yellow,
        Color::Red,
        Color::Magenta,
        Color::Blue,
    ];
    for (i, regexp) in regexps.iter().enumerate() {
        let defchosen = colours[i % colours.len()];
        let mut chosen = defchosen;
        let mut reg = regexp.to_string();
        if let Some(colour) = regexp.split(':').next() {
            // match colour in colours
            chosen = match colour {
                "yellow" => Color::Yellow,
                "cyan" => Color::Cyan,
                "red" => Color::Red,
                "magenta" => Color::Magenta,
                "blue" => Color::Blue,
                "green" => Color::Green,
                "white" => Color::White,
                "black" => Color::Black,
                _ => Color::Default,
            };
            if chosen == Color::Default {
                chosen = defchosen;
            } else {
                reg = regexp.replace(format!("{colour}:").as_str(), "");
            }
        }
        regexp_colours.insert(reg, chosen);
    }
    regexp_colours
}

fn colouring(color: ColorWhen) -> bool {
    let interactive_terminal = atty::is(Stream::Stdout);
    match color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => env::var_os("NO_COLOR").is_none() && interactive_terminal,
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
    let colouring = colouring(args.color);
    if !colouring {
        Paint::disable();
    }
    let json_keys = make_json_keys(&args.json_keys);

    Config {
        level_symbols: args.level_symbols,
        kail_prefix_format: args.kail_prefix_format,
        kail_no_prefix: args.kail_no_prefix,
        time_format: args.time_format,
        skip_line_regexp: args.skip_line_regexp,
        filter_levels: args.filter_levels,
        action_command: args.action_command,
        action_regexp: args.action_regexp,
        files: args.files,
        regexp_colours,
        colouring,
        json_keys,
    }
}
