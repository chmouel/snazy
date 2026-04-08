use std::process::Command;

use regex::Regex;

use crate::config::{self, Config};
use crate::model::{ParsedLine, StructuredLog};

pub fn maybe_run_action(config: &Config, line: &str) {
    let (Some(action_regexp), Some(action_command)) = (
        config.action_regexp.as_ref(),
        config.action_command.as_ref(),
    ) else {
        return;
    };

    let action_re = Regex::new(action_regexp).expect("Invalid action_regexp");
    if let Some(captures) = action_re.captures(line) {
        let regexpmatch = captures.get(0).unwrap().as_str();
        let action_command = action_command.replace("{}", regexpmatch);
        let action_command_str = action_command.clone();
        if Command::new("sh")
            .arg("-c")
            .arg(action_command)
            .spawn()
            .is_ok()
        {
            println!(
                "Spawned command: for action: {}",
                yansi::Paint::cyan(regexpmatch)
            );
        } else {
            eprintln!("Failed to spawn action command: {action_command_str}");
        }
    }
}

pub fn process_line(config: &Config, parsed: ParsedLine) -> Option<ParsedLine> {
    match parsed {
        ParsedLine::Structured(log) => {
            process_structured_log(config, log).map(ParsedLine::Structured)
        }
        ParsedLine::Raw(line) => Some(ParsedLine::Raw(line)),
        ParsedLine::KubectlHeader => Some(ParsedLine::KubectlHeader),
        ParsedLine::KubectlEvent(event) => Some(ParsedLine::KubectlEvent(event)),
    }
}

fn process_structured_log(config: &Config, mut log: StructuredLog) -> Option<StructuredLog> {
    if config
        .skip_line_regexp
        .iter()
        .any(|pattern| match Regex::new(pattern) {
            Ok(re) => re.is_match(&log.message),
            Err(error) => {
                eprintln!("Invalid skip_line_regexp pattern '{pattern}': {error}");
                false
            }
        })
    {
        return None;
    }

    if !config.filter_levels.is_empty()
        && !config
            .filter_levels
            .contains(config::level_from_str(&log.level.to_lowercase()))
    {
        return None;
    }

    if config.extra_fields || !config.include_fields.is_empty() {
        log.extra_fields =
            collect_extra_fields(config, log.raw_json.as_ref(), &log.consumed_fields);
    }

    Some(log)
}

fn collect_extra_fields(
    config: &Config,
    raw_json: Option<&serde_json::Value>,
    consumed_fields: &[String],
) -> Vec<(String, String)> {
    let Some(raw_json) = raw_json else {
        return Vec::new();
    };

    let Some(map) = raw_json.as_object() else {
        return Vec::new();
    };

    let main_fields = [
        "msg",
        "message",
        "level",
        "severity",
        "ts",
        "timestamp",
        "stacktrace",
    ];

    if config.include_fields.is_empty() {
        map.iter()
            .filter(|(key, value)| {
                !main_fields.contains(&key.as_str())
                    && !field_is_consumed(key, value, consumed_fields)
            })
            .map(|(key, value)| (key.clone(), json_value_to_string(value)))
            .collect()
    } else {
        config
            .include_fields
            .iter()
            .filter_map(|field| {
                get_nested_value(raw_json, field)
                    .map(|value| (field.clone(), json_value_to_string(value)))
            })
            .collect()
    }
}

fn field_is_consumed(key: &str, value: &serde_json::Value, consumed_fields: &[String]) -> bool {
    let pointer = format!("/{}", escape_json_pointer_segment(key));
    value_is_consumed(pointer.as_str(), value, consumed_fields)
}

fn value_is_consumed(pointer: &str, value: &serde_json::Value, consumed_fields: &[String]) -> bool {
    if consumed_fields.iter().any(|field| field == pointer) {
        return true;
    }

    let serde_json::Value::Object(map) = value else {
        return false;
    };

    !map.is_empty()
        && map.iter().all(|(key, value)| {
            let child_pointer = format!("{pointer}/{}", escape_json_pointer_segment(key));
            value_is_consumed(child_pointer.as_str(), value, consumed_fields)
        })
}

fn escape_json_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    if value.is_string() {
        value.as_str().unwrap().to_string()
    } else {
        value.to_string()
    }
}

fn get_nested_value<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for part in path.split('.') {
        match current {
            serde_json::Value::Object(map) => current = map.get(part)?,
            _ => return None,
        }
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::{thread, vec};

    use crate::config::Config;
    use crate::model::StructuredLog;

    use super::*;

    #[test]
    fn include_fields_support_nested_paths() {
        let config = Config {
            include_fields: vec!["meta.status.code".to_string()],
            ..Config::default()
        };
        let log = StructuredLog {
            level: "INFO".to_string(),
            message: "hello".to_string(),
            timestamp: None,
            others: None,
            consumed_fields: Vec::new(),
            extra_fields: Vec::new(),
            stacktrace: None,
            raw_json: Some(serde_json::json!({
                "meta": {
                    "status": {
                        "code": 200
                    }
                }
            })),
            kail_prefix: None,
        };

        let processed =
            super::process_line(&config, crate::model::ParsedLine::Structured(log)).unwrap();

        let crate::model::ParsedLine::Structured(log) = processed else {
            panic!("expected structured log");
        };

        assert_eq!(
            log.extra_fields,
            vec![("meta.status.code".to_string(), "200".to_string())]
        );
    }

    #[test]
    fn extra_fields_skip_consumed_top_level_fields() {
        let config = Config {
            extra_fields: true,
            ..Config::default()
        };
        let log = StructuredLog {
            level: "WARNING".to_string(),
            message: "hello".to_string(),
            timestamp: Some("10:00:00".to_string()),
            others: None,
            consumed_fields: vec![
                "/level".to_string(),
                "/msg".to_string(),
                "/time".to_string(),
                "/stack".to_string(),
            ],
            extra_fields: Vec::new(),
            stacktrace: Some("trace".to_string()),
            raw_json: Some(serde_json::json!({
                "level": "warning",
                "msg": "hello",
                "time": "2022-04-25T14:20:32.505637358Z",
                "stack": "trace",
                "request_id": "req-1"
            })),
            kail_prefix: None,
        };

        let processed =
            super::process_line(&config, crate::model::ParsedLine::Structured(log)).unwrap();

        let crate::model::ParsedLine::Structured(log) = processed else {
            panic!("expected structured log");
        };

        assert_eq!(
            log.extra_fields,
            vec![("request_id".to_string(), "req-1".to_string())]
        );
    }

    #[test]
    fn extra_fields_skip_consumed_nested_objects() {
        let config = Config {
            extra_fields: true,
            ..Config::default()
        };
        let log = StructuredLog {
            level: "INFO".to_string(),
            message: "ecs".to_string(),
            timestamp: Some("10:00:00".to_string()),
            others: None,
            consumed_fields: vec![
                "/@timestamp".to_string(),
                "/message".to_string(),
                "/log/level".to_string(),
                "/error/stack_trace".to_string(),
            ],
            extra_fields: Vec::new(),
            stacktrace: Some("trace".to_string()),
            raw_json: Some(serde_json::json!({
                "@timestamp": "2022-04-25T14:20:32.505637358Z",
                "message": "ecs",
                "log": { "level": "info" },
                "error": { "stack_trace": "trace" },
                "service": { "name": "api" }
            })),
            kail_prefix: None,
        };

        let processed =
            super::process_line(&config, crate::model::ParsedLine::Structured(log)).unwrap();

        let crate::model::ParsedLine::Structured(log) = processed else {
            panic!("expected structured log");
        };

        assert_eq!(
            log.extra_fields,
            vec![("service".to_string(), r#"{"name":"api"}"#.to_string())]
        );
    }

    #[test]
    fn action_command_is_triggered() {
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

        maybe_run_action(&config, "un HELLO MOTO nono");

        let mut waited = 0;
        while !file_path.exists() && waited < 500 {
            thread::sleep(core::time::Duration::from_millis(10));
            waited += 10;
        }

        assert!(file_path.exists());
        let mut file = std::fs::File::open(file_path).expect("Failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file contents");
        assert_eq!(contents, "you said HELLO MOTO\n");
    }
}
