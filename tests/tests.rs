use std::io::Write;

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
    kali_log_and_regexp,
    ["-rHello", "-rMoto", "--color", "always"],
    r#"ns/pod[container]: {"level":"INFO","msg":"Hello Moto"}"#,
    "\u{1b}[38;5;10mINFO\u{1b}[0m   \u{1b}[34mns/pod[container]\u{1b}[0m \u{1b}[33mHello\u{1b}[0m \u{1b}[35mMoto\u{1b}[0m\n",
    false
);
