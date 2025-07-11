use std::io::Write;
use std::process;

mod testenv;

snazytest!(
    show_help,
    ["--help"],
    "",
    "snazy is a snazzy json log viewer",
    true
);

snazytest!(
    simple_parsing,
    ["--color", "never"],
    r#"{"level":"info","msg":"foo"}"#,
    "INFO                 foo\n",
    false
);

snazytest!(
    simple_date,
    ["--color", "never"],
    r#"{"level":"info", "ts": "2022-04-25T14:20:32.505637358Z", "msg":"foo"}"#,
    "INFO                14:20:32 foo\n",
    false
);

snazytest!(
    floated_date,
    ["--color", "never"],
    r#"{"level":"info", "ts": 1650602040.6289625, "msg":"foo"}"#,
    "INFO                04:34:00 foo\n",
    false
);

snazytest!(
    raw_non_json,
    ["--color", "never"],
    "Hello Moto",
    "Hello Moto\n",
    false
);

snazytest!(
    regexp_raw_json,
    ["-rHello", "--color", "always"],
    "Hello Moto",
    "\x1b[36mHello\x1b[0m Moto\n",
    false
);

snazytest!(
    regexp_color_fg_bg,
    [
        "-r",
        "fg=yellow,bg=black:YellowOnBlack",
        "--color",
        "always"
    ],
    "YellowOnBlack",
    "\u{1b}[40;33mYellowOnBlack\u{1b}[0m\n",
    false
);
snazytest!(
    regexp_rgb_colored,
    ["-ryellow:Hello", "-r88,48,235:Moto", "--color", "always"],
    "Hello Moto",
    "\u{1b}[33mHello\u{1b}[0m \u{1b}[38;2;88;48;235mMoto\u{1b}[0m\n",
    false
);

snazytest!(
    multiple_regexp_raw_json,
    ["-rHello", "-rMoto", "--color", "always"],
    "Hello Moto",
    "\u{1b}[36mHello\u{1b}[0m \u{1b}[33mMoto\u{1b}[0m\n",
    false
);

snazytest!(
    kail_log_and_regexp,
    ["-rHello", "-rMoto", "--color", "always"],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "\u{1b}[32mINFO\u{1b}[0m        \u{1b}[34mns/pod[container]\u{1b}[0m \u{1b}[36mHello\u{1b}[0m \u{1b}[33mMoto\u{1b}[0m\n",
    false
);

snazytest!(
    kail_custom_format,
    [
        "--kail-prefix-format",
        "{namespace}::{pod}|{container}",
        "--color",
        "never"
    ],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "ns::pod|container",
    true
);

snazytest!(
    kail_no_prefix,
    ["--kail-no-prefix", "--color", "never"],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "INFO                 Hello Moto\n",
    false
);

snazytest!(
    pac_output_github,
    ["--color", "never"],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" github","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f09b} github\n",
    false
);

snazytest!(
    pac_output_gitlab,
    ["--color", "never"],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" gitlab","provider":"gitlab","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f296} gitlab\n",
    false
);

snazytest!(
    pac_output_bitbucket_cloud,
    ["--color", "never"],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" bitbucket-cloud","provider":"bitbucket-cloud","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f171} bitbucket-cloud\n",
    false
);

snazytest!(
    pac_output_fallback_ts,
    ["--color", "never"],
    r#"{"severity":"INFO","timestamp":"2022-04-25:FOO","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" bitbucket-cloud","provider":"bitbucket-cloud","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                2022-04-25:FOO  \u{f171} bitbucket-cloud\n",
    false
);

snazytest!(
    pac_output_bitbucket_server,
    ["--color", "never"],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" bitbucket-server","provider":"bitbucket-server","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f171}S bitbucket-server\n",
    false
);

snazytest!(
    level_symbols,
    ["--level-symbols"],
    r#"{"level":"info","msg":"INFO"}
    {"level":"warn","msg":"warn"}
    {"level":"error","msg":"error"}
    {"level":"fatal","msg":"fatal"}
    "#,
    "💡  INFO\n∙  warn\n🚨  error\n💀  fatal\n",
    false
);

snazytest!(
    filter_level_info,
    ["--color", "never", "--filter-levels", "info"],
    r#"{"level":"info", "msg":"INFO"}"#,
    "INFO                 INFO\n",
    false
);

snazytest!(
    filter_level_warning,
    ["--color", "never", "--filter-levels", "warning"],
    r#"{"level":"warning", "msg":"warn"}"#,
    "WARN                 warn\n",
    false
);

snazytest!(
    skip_lines,
    ["-S", "yolo"],
    r#"{"level":"info","msg":"yolo"}"#,
    "",
    false
);

snazytest!(
    custom_level_info,
    ["--color", "never", "--level-symbols", "emoji"],
    r#"{"level":"info", "ts": "2022-04-25T14:20:32.505637358Z", "msg":"message"}"#,
    "💡 14:20:32 message\n",
    false
);

snazytest!(
    custom_level_debug,
    ["--color", "never", "--level-symbols", "emoji"],
    r#"{"level":"debug", "ts": 1650602040.0, "msg":"anotherone"}"#,
    "🐛 04:34:00 anotherone\n",
    false
);

snazytest!(
    timezone_parsing,
    ["--color", "never"],
    r#"{"level":"info", "ts": "2022-04-25T06:56:22.505637358Z", "msg":"timezone test"}"#,
    "INFO                06:56:22 timezone test\n",
    false
);

snazytest!(
    stacktrace_default_display,
    [""],
    r#"{"level":"error","ts":"2022-04-25T14:20:32.505637358Z","msg":"Something went wrong","stacktrace":"github.com/example/app.Function\n\tat app.go:42\n\tat main.go:15"}"#,
    "Stacktrace",
    true
);

snazytest!(
    stacktrace_hidden,
    ["--color", "never", "--hide-stacktrace"],
    r#"{"level":"error", "ts": "2022-04-25T14:20:32.505637358Z", "msg":"Something went wrong", "stacktrace": "hidden"}"#,
    "ERROR              14:20:32 Something went wrong\n",
    false
);

#[test]
#[should_panic]
fn all_json_keys_need_tobe_specified() {
    let tenv = testenv::TestEnv::new();
    let mut cmd = process::Command::new(tenv.snazy_exe);
    let args = &["-k", "msg=/foo"];
    cmd.args(args);
    // Run *snazy*.
    let output = cmd.output().expect("snazy output");
    if !output.status.success() {
        panic!("{}", testenv::format_exit_error(args, &output));
    }
}
