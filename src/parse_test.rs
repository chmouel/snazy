#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::{thread, vec};

    use regex::Regex;
    use yansi::{Color, Paint};

    use crate::config::Config;
    use crate::parse::{action_on_regexp, do_line, extract_info};

    #[test]
    fn test_get_line() {
        let line = r#"{"severity":"INFO","timestamp":"2022-04-25T10:24:30.155404234Z","logger":"pipelinesascode","caller":"kubeinteraction/secrets.go:114","message":"hello moto"}"#;
        let msg = extract_info(
            line,
            &Config {
                ..Config::default()
            },
        );
        assert_eq!(msg["msg"], "hello moto");
    }

    #[test]
    fn test_kail_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":"updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = extract_info(
            line,
            &Config {
                kail_no_prefix: false,
                ..Config::default()
            },
        );
        assert!(msg["msg"].contains("ns/pod[container]"));
        assert!(msg["msg"].contains("updated"));
    }

    #[test]
    fn test_kail_newline() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":"updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = extract_info(
            line,
            &Config {
                kail_no_prefix: false,
                kail_prefix_format: String::from("{container}\n"),
                ..Config::default()
            },
        );
        assert!(msg["msg"].contains("container\n"));
    }

    #[test]
    fn test_skip_lines() {
        let line = r#"{"level":"INFO","msg":"yolo"}"#;
        let msg = do_line(
            &Config {
                skip_line_regexp: vec![String::from("yolo")],
                ..Config::default()
            },
            line,
        );
        assert!(msg.is_none());
    }

    #[test]
    fn test_kail_no_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = extract_info(
            line,
            &Config {
                kail_no_prefix: true,
                ..Config::default()
            },
        );
        assert_eq!(msg["msg"], "updated");
    }

    #[test]
    fn test_pac_provider_icon() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" github","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = extract_info(
            line,
            &Config {
                kail_no_prefix: false,
                ..Config::default()
            },
        );
        assert!(msg.contains_key("others"));
        assert!(msg["others"].contains(" "));
    }

    #[test]
    fn test_apply_regexps() {
        let line = String::from("red blue normal");
        // define a regexp
        let regexp = Regex::new(r"\b(b.ue)\b").unwrap();
        let mut map = HashMap::new();
        map.insert(String::from("red"), Color::Red);
        map.insert(regexp.to_string(), Color::Blue);
        let ret = crate::parse::apply_regexps(&map, line);
        assert_eq!(
            ret,
            format!(
                "{} {} normal",
                "red".red().to_string(),
                "blue".blue().to_string()
            )
        );
    }

    #[test]
    fn test_config_json_keys() {
        let mut keys = HashMap::new();
        keys.insert(String::from("msg"), String::from("/foo"));
        keys.insert(String::from("level"), String::from("/bar"));

        let config = Config {
            json_keys: keys,
            ..Config::default()
        };
        let line = r#"{"foo": "Bar", "bar": "info"}"#;
        let info = extract_info(line, &config);
        assert_eq!(info.get("msg").unwrap(), "Bar");
        assert_eq!(info.get("level").unwrap(), "info");
    }

    #[test]
    fn test_config_json_timestamp_float() {
        let mut keys = HashMap::new();
        keys.insert(String::from("ts"), String::from("/bar"));

        let config = Config {
            json_keys: keys,
            ..Config::default()
        };
        let line = r#"{"bar": 1650602040.6289625}"#;
        let info = extract_info(line, &config);
        assert_eq!(info.get("ts").unwrap(), "04:34:00");
    }

    #[test]
    fn test_custom_json_match() {
        let mut keys = HashMap::new();
        keys.insert(String::from("ts"), String::from("/bar"));
        keys.insert(String::from("msg"), String::from("/foo"));
        keys.insert(String::from("level"), String::from("/level"));

        let config = Config {
            json_keys: keys,
            ..Config::default()
        };
        let line =
            r#"{"bar": "2022-04-22T04:34:00.628550164Z", "foo": "hello", "level": "lelevel"}"#;
        let info = extract_info(line, &config);
        assert_eq!(info.get("ts").unwrap(), "04:34:00");
        assert_eq!(info.get("msg").unwrap(), "hello");
        assert_eq!(info.get("level").unwrap(), "lelevel");

        let line = r#"{"bar": 1650992726.6289625, "foo": "hello", "level": "lelevel"}"#;
        let info = extract_info(line, &config);
        assert_eq!(info.get("ts").unwrap(), "17:05:26");
    }

    #[test]
    fn test_action_on_regexp() {
        // create a temporary file to delete at the end of the test
        let file = tempfile::NamedTempFile::new().unwrap();
        let file_path = file.path().to_path_buf();
        file.close().unwrap();

        let config = Config {
            action_regexp: Some(String::from(r"HELLO\s\w+")),
            action_command: Some(
                String::from("echo \"you said {}\" > ") + file_path.to_str().unwrap(),
            ),
            ..Config::default()
        };
        let line = r"un HELLO MOTO nono el petiot roboto";
        action_on_regexp(&config, line);
        // sleep for a bit to let the file be created
        thread::sleep(core::time::Duration::from_millis(50));
        let mut file = std::fs::File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "you said HELLO MOTO\n");
    }

    #[test]
    fn test_read_from_file() {
        let mut file: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_path = file.path().to_path_buf();
        let line = r#"{"level":"INFO","ts":"2022-04-25T14:20:32.505637358Z", "msg":"hello world"}
{"level":"DEBUG","ts":"2022-04-25T14:20:32.505637358Z", "msg":"debug"}"#;

        Write::write_all(&mut file, line.as_bytes()).unwrap();

        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            ..Config::default()
        };
        let writeto = &mut Vec::new();
        crate::parse::read_a_file(&config, file_path.to_str().unwrap(), writeto);
        file.close().unwrap();
        assert_eq!(
            "\u{1b}[32mINFO\u{1b}[0m       \u{1b}[38;5;13m14:20:32\u{1b}[0m hello world\n\u{1b}[38;5;14mDEBUG\u{1b}[0m \u{1b}[38;5;13m14:20:32\u{1b}[0m debug\n",
            std::str::from_utf8(writeto).unwrap()
        );
    }
}
