#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::{thread, vec};

    use regex::Regex;
    use yansi::{Color, Paint, Style};

    use crate::config::Config;
    use crate::parse::{do_line, extract_info};

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
        map.insert(String::from("red"), Style::new().fg(Color::Red));
        map.insert(regexp.to_string(), Style::new().fg(Color::Blue));
        let ret = crate::utils::apply_regexps(&map, line);
        assert_eq!(ret, format!("{} {} normal", "red".red(), "blue".blue()));
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
        let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        file.close().expect("Failed to close temp file");

        let config = Config {
            action_regexp: Some(String::from(r"HELLO\s\w+")),
            action_command: Some(
                String::from("echo \"you said {}\" > ")
                    + file_path
                        .to_str()
                        .expect("Failed to convert file path to str"),
            ),
            ..Config::default()
        };
        let line = r"un HELLO MOTO nono el petiot roboto";
        crate::parse::action_on_regexp(&config, line);
        // Wait for the file to be created, up to 500ms
        let mut waited = 0;
        while !file_path.exists() && waited < 500 {
            thread::sleep(core::time::Duration::from_millis(10));
            waited += 10;
        }
        assert!(
            file_path.exists(),
            "File was not created by action_on_regexp"
        );
        let mut file = std::fs::File::open(file_path).expect("Failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file contents");
        assert_eq!(contents, "you said HELLO MOTO\n");
    }

    #[test]
    fn test_read_from_file() {
        let mut file: tempfile::NamedTempFile =
            tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        let line = r#"{"level":"INFO","ts":"2022-04-25T14:20:32.505637358Z", "msg":"hello world"}
{"level":"DEBUG","ts":"2022-04-25T14:20:32.505637358Z", "msg":"debug"}"#;

        Write::write_all(&mut file, line.as_bytes()).expect("Failed to write to temp file");

        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            ..Config::default()
        };
        let writeto = &mut Vec::new();
        crate::parse::read_a_file(&config, file_path.to_str().unwrap(), writeto);
        file.close().unwrap();
        let output = std::str::from_utf8(writeto).expect("Failed to convert output to utf8");
        // Use regex to validate output contains expected log lines
        let re_info = regex::Regex::new(r"INFO.*14:20:32.*hello world").unwrap();
        let re_debug = regex::Regex::new(r"DEBUG.*14:20:32.*debug").unwrap();
        assert!(
            re_info.is_match(output),
            "INFO log line not found or formatted incorrectly: {}",
            output
        );
        assert!(
            re_debug.is_match(output),
            "DEBUG log line not found or formatted incorrectly: {}",
            output
        );
    }

    #[test]
    fn test_hide_stacktrace() {
        // Create a temporary file for testing
        let mut file: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_path = file.path().to_path_buf();

        // Create a log message with a stacktrace
        // Using a format that exactly matches Golang stacktrace format
        let stacktrace = r"goroutine 1 [running]:
github.com/example/app.Function1(0x123456)
        /home/user/app.go:42 +0x2a
github.com/example/app.Function2(0x123456)
        /home/user/main.go:15 +0x1b";

        let line = format!(
            r#"{{"level":"ERROR","ts":"2022-04-25T14:20:32.505637358Z", "msg":"Error occurred", "stacktrace":{stacktrace:?}}}"#
        );

        Write::write_all(&mut file, line.as_bytes()).unwrap();

        // Test with hide_stacktrace = false (default)
        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            hide_stacktrace: false,
            action_regexp: None,
            action_command: None,
            ..Config::default()
        };
        let writeto_visible = &mut Vec::new();
        crate::parse::read_a_file(&config, file_path.to_str().unwrap(), writeto_visible);

        // Stacktrace should be visible
        let output_with_stacktrace = std::str::from_utf8(writeto_visible).unwrap();

        // Check for presence of stacktrace header
        assert!(output_with_stacktrace.contains("Stacktrace"));

        // Check for actual stacktrace content
        // Look for "app.go" which should be present regardless of formatting
        assert!(output_with_stacktrace.contains("app.go"));

        // Test with hide_stacktrace = true
        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            hide_stacktrace: true,
            action_regexp: None,
            action_command: None,
            ..Config::default()
        };
        let writeto_hidden = &mut Vec::new();
        crate::parse::read_a_file(&config, file_path.to_str().unwrap(), writeto_hidden);
        file.close().unwrap();

        // Stacktrace should not be visible
        let output_without_stacktrace = std::str::from_utf8(writeto_hidden).unwrap();

        // Check basic stacktrace content is not there
        assert!(!output_without_stacktrace.contains("Stacktrace"));
        assert!(!output_without_stacktrace.contains("app.go"));

        // The error message should still be visible
        assert!(output_without_stacktrace.contains("Error occurred"));
    }

    #[test]
    fn test_read_from_missing_file() {
        let missing_path = "/tmp/this_file_should_not_exist_snazy_test";
        let config = Config {
            files: Some(vec![missing_path.to_string()]),
            ..Config::default()
        };
        let result = std::panic::catch_unwind(|| {
            let mut writeto = Vec::new();
            crate::parse::read_a_file(&config, missing_path, &mut writeto);
            writeto
        });
        assert!(result.is_ok(), "read_a_file panicked on missing file");
        let output = result.unwrap();
        assert_eq!(output.len(), 0, "Output should be empty for missing file");
    }

    #[test]
    fn test_read_from_malformed_json() {
        let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        let line = "{this is not valid json}";
        std::io::Write::write_all(&mut file.as_file(), line.as_bytes())
            .expect("Failed to write to temp file");
        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            ..Config::default()
        };
        let result = std::panic::catch_unwind(|| {
            let mut writeto = Vec::new();
            crate::parse::read_a_file(&config, file_path.to_str().unwrap(), &mut writeto);
            writeto
        });
        file.close().unwrap();
        assert!(result.is_ok(), "read_a_file panicked on malformed JSON");
        let output = result.unwrap();
        assert!(
            output.is_empty() || std::str::from_utf8(&output).unwrap().contains("error"),
            "Malformed JSON should not produce valid output"
        );
    }

    #[test]
    fn test_read_from_empty_file() {
        let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            ..Config::default()
        };
        let result = std::panic::catch_unwind(|| {
            let mut writeto = Vec::new();
            crate::parse::read_a_file(&config, file_path.to_str().unwrap(), &mut writeto);
            writeto
        });
        file.close().unwrap();
        assert!(result.is_ok(), "read_a_file panicked on empty file");
        let output = result.unwrap();
        assert_eq!(output.len(), 0, "Output should be empty for empty file");
    }
}
