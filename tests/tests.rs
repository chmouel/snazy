use std::io::Write;
use std::process;

mod testenv;

snazytest!(
    show_help,
    ["--help"],
    "",
    "You just need to pipe to snazy some logs formatted as json",
    true
);

snazytest!(
    simple_parsing,
    [""],
    r#"{"level":"info","msg":"foo"}"#,
    "INFO                 foo\n",
    false
);

snazytest!(
    simple_date,
    [""],
    r#"{"level":"info", "ts": "2022-04-25T14:20:32.505637358Z", "msg":"foo"}"#,
    "INFO                14:20:32 foo\n",
    false
);

snazytest!(
    floated_date,
    [""],
    r#"{"level":"info", "ts": 1650602040.6289625, "msg":"foo"}"#,
    "INFO                04:34:00 foo\n",
    false
);

snazytest!(raw_non_json, [""], "Hello Moto", "Hello Moto\n", false);

snazytest!(
    regexp_raw_json,
    ["-rHello", "--color", "always"],
    "Hello Moto",
    "\x1b[33mHello\x1b[0m Moto\n",
    false
);

snazytest!(
    multiple_regexp_raw_json,
    ["-rHello", "-rMoto", "--color", "always"],
    "Hello Moto",
    "\x1b[33mHello\x1b[0m \x1b[35mMoto\x1b[0m\n",
    false
);

snazytest!(
    kail_log_and_regexp,
    ["-rHello", "-rMoto", "--color", "always"],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "\u{1b}[38;5;10mINFO\u{1b}[0m   \u{1b}[34mns/pod[container]\u{1b}[0m \u{1b}[33mHello\u{1b}[0m \u{1b}[35mMoto\u{1b}[0m\n",
    false
);

snazytest!(
    kail_custom_format,
    ["--kail-prefix-format", "{namespace}::{pod}|{container}"],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "ns::pod|container",
    true
);

snazytest!(
    kail_no_prefix,
    ["--kail-no-prefix"],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "INFO                 Hello Moto\n",
    false
);

snazytest!(
    pac_output_github,
    [""],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" github","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f09b} github\n",
    false
);

snazytest!(
    pac_output_gitlab,
    [""],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" gitlab","provider":"gitlab","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f296} gitlab\n",
    false
);

snazytest!(
    pac_output_bitbucket_cloud,
    [""],
    r#"{"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" bitbucket-cloud","provider":"bitbucket-cloud","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#,
    "INFO                14:20:32  \u{f171} bitbucket-cloud\n",
    false
);

snazytest!(
    pac_output_bitbucket_server,
    [""],
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
    "????  INFO\n???  warn\n????  error\n????  fatal\n",
    false
);

snazytest!(
    filter_level,
    ["--filter-level=info", "--filter-level=warning"],
    r#"{"level":"info","msg":"INFO"}
    {"level":"warning","msg":"warn"}
    {"level":"fatal","msg":"fatal"}
    "#,
    "INFO                 INFO\nWARN                 warn\n",
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
    custom_level,
    [
        "-k",
        "msg=/the/msg/is",
        "-k",
        "level=/the/level/is",
        "-k",
        "ts=/the/ts/is"
    ],
    r#"{"the": {"msg": {"is": "message"}, "level": {"is": "INFO"}, "ts": {"is": "2022-04-25T14:20:32.505637358Z"}}}
{"the": {"msg": {"is": "anotherone"}, "level": {"is": "DEBUG"}, "ts": {"is": 1650602040.6289625}}}
"#,
    "INFO                14:20:32 message\nDEBUG               04:34:00 anotherone\n",
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
