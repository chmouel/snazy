use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;
use crate::model::{KubectlEvent, ParsedLine, StructuredLog};

pub const KAIL_RE: &str =
    r"^(?P<namespace>[^/]*)/(?P<pod>[^\[]*)\[(?P<container>[^]]*)]: (?P<line>.*)";

#[derive(Serialize, Deserialize, Debug)]
struct Pac {
    severity: String,
    timestamp: String,
    caller: String,
    message: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Knative {
    level: String,
    msg: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CaddyAccess {
    level: String,
    logger: Option<String>,
    #[serde(default)]
    msg: String,
    request: CaddyRequest,
    #[serde(default)]
    status: u16,
    #[serde(default)]
    duration: f64,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CaddyRequest {
    method: String,
    uri: String,
    #[serde(flatten)]
    _other: BTreeMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ParseState {
    pub kubectl_events_mode: bool,
    pub kubectl_events_cols: Option<(usize, usize, usize, usize, usize)>,
    pub previous_structured_timestamp: Option<DateTime<Utc>>,
}

struct StructuredFields<'a> {
    level: &'a str,
    message: &'a str,
    timestamp: Option<String>,
    parsed_timestamp: Option<DateTime<Utc>>,
    others: Option<String>,
    stacktrace: Option<String>,
    consumed_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedLine {
    line: String,
    kail_prefix: Option<String>,
}

pub fn parse_line(config: &Config, rawline: &str, state: &mut ParseState) -> Option<ParsedLine> {
    if rawline.trim().is_empty() {
        return None;
    }

    if is_kubectl_events_header(rawline, state) {
        state.kubectl_events_mode = true;
        return Some(ParsedLine::KubectlHeader);
    }

    if state.kubectl_events_mode {
        if let Some(event) = parse_kubectl_event_line(rawline, state) {
            return Some(ParsedLine::KubectlEvent(event));
        }
    }

    let prepared = prepare_line(config, rawline);

    parse_structured_log(config, &prepared).map_or_else(
        || Some(ParsedLine::Raw(rawline.to_string())),
        |log| Some(ParsedLine::Structured(log)),
    )
}

pub fn prepare_line(config: &Config, rawline: &str) -> PreparedLine {
    let kail_prefix = parse_kail_lines(config, rawline);
    let line = if kail_prefix.is_some() {
        Regex::new(KAIL_RE).map_or_else(
            |_| rawline.to_string(),
            |re| re.replace_all(rawline, "$line").to_string(),
        )
    } else {
        rawline.to_string()
    };

    PreparedLine { line, kail_prefix }
}

pub fn parse_structured_log(config: &Config, prepared: &PreparedLine) -> Option<StructuredLog> {
    let time_format = config.time_format.as_str();
    let timezone = config.timezone.as_deref();
    let raw_json = serde_json::from_str::<Value>(&prepared.line).ok();

    if !config.json_keys.is_empty() {
        if let Some(log) = parse_custom_json(config, prepared, raw_json.clone(), time_format) {
            return Some(log);
        }
    }

    parse_pac(prepared, raw_json.clone(), time_format, timezone)
        .or_else(|| parse_caddy(prepared, raw_json.as_ref(), time_format, timezone))
        .or_else(|| parse_logrus(prepared, raw_json.as_ref(), time_format, timezone))
        .or_else(|| parse_zerolog(prepared, raw_json.as_ref(), time_format, timezone))
        .or_else(|| parse_knative(prepared, raw_json.as_ref(), time_format, timezone))
        .or_else(|| parse_ecs(prepared, raw_json.as_ref(), time_format, timezone))
        .or_else(|| parse_cloud_logging(prepared, raw_json.as_ref(), time_format, timezone))
}

fn parse_custom_json(
    config: &Config,
    prepared: &PreparedLine,
    raw_json: Option<Value>,
    time_format: &str,
) -> Option<StructuredLog> {
    let raw_json = raw_json?;
    let mut message = None;
    let mut level = None;
    let mut timestamp = None;
    let mut parsed_timestamp = None;

    for (key, path) in &config.json_keys {
        let extracted = if path.starts_with('/') {
            raw_json.pointer(path)
        } else {
            raw_json.get(path)
        };
        let Some(extracted) = extracted else {
            continue;
        };

        let normalized = if key == "ts" || key == "timestamp" || key == "date" {
            crate::utils::convert_ts_float_or_str(
                extracted,
                time_format,
                config.timezone.as_deref(),
            )
        } else {
            extracted.to_string().replace('"', "")
        };

        if key == "ts" || key == "timestamp" || key == "date" {
            parsed_timestamp = crate::utils::parse_timestamp_value(extracted);
        }

        match key.as_str() {
            "msg" => message = Some(normalized),
            "level" => level = Some(normalized),
            "ts" | "timestamp" | "date" => timestamp = Some(normalized),
            _ => {}
        }
    }

    Some(StructuredLog {
        level: level?,
        message: message?,
        timestamp,
        parsed_timestamp,
        others: None,
        consumed_fields: Vec::new(),
        extra_fields: Vec::new(),
        stacktrace: raw_json
            .get("stacktrace")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        raw_json: Some(raw_json),
        kail_prefix: prepared.kail_prefix.clone(),
    })
}

fn parse_pac(
    prepared: &PreparedLine,
    raw_json: Option<Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let pac = serde_json::from_str::<Pac>(&prepared.line).ok()?;
    let mut others = String::new();
    let parsed_timestamp = crate::utils::parse_timestamp_str(pac.timestamp.as_str());

    if let Some(provider) = pac.other.get("provider").and_then(|value| value.as_str()) {
        others.push_str(crate::utils::convert_pac_provider_to_fa_icon(provider));
        others.push(' ');
    }

    Some(StructuredLog {
        level: pac.severity.to_uppercase(),
        message: pac.message.trim().to_string(),
        timestamp: Some(crate::utils::convert_str_to_ts(
            pac.timestamp.as_str(),
            time_format,
            timezone,
        )),
        parsed_timestamp,
        others: (!others.is_empty()).then_some(others),
        consumed_fields: Vec::new(),
        extra_fields: Vec::new(),
        stacktrace: pac
            .other
            .get("stacktrace")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        raw_json,
        kail_prefix: prepared.kail_prefix.clone(),
    })
}

fn parse_caddy(
    prepared: &PreparedLine,
    raw_json: Option<&Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let caddy = serde_json::from_str::<CaddyAccess>(&prepared.line).ok()?;
    if caddy.logger.as_deref() != Some("http.log.access") && caddy.msg.trim() != "handled request" {
        return None;
    }
    let (timestamp, parsed_timestamp) = caddy.other.get("ts").map_or((None, None), |value| {
        let (display, parsed) = timestamp_details(value, time_format, timezone);
        (Some(display), parsed)
    });

    Some(StructuredLog {
        level: caddy.level.to_uppercase(),
        message: format!(
            "{} {} -> {} ({}ms)",
            caddy.request.method,
            caddy.request.uri,
            caddy.status,
            (caddy.duration * 1000.0).round() as i64
        ),
        timestamp,
        parsed_timestamp,
        others: None,
        consumed_fields: vec![
            "/level".to_string(),
            "/ts".to_string(),
            "/stacktrace".to_string(),
            "/request/method".to_string(),
            "/request/uri".to_string(),
            "/status".to_string(),
            "/duration".to_string(),
        ],
        extra_fields: Vec::new(),
        stacktrace: caddy
            .other
            .get("stacktrace")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        raw_json: raw_json.cloned(),
        kail_prefix: prepared.kail_prefix.clone(),
    })
}

fn parse_knative(
    prepared: &PreparedLine,
    raw_json: Option<&Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let knative = serde_json::from_str::<Knative>(&prepared.line).ok()?;
    let (timestamp, parsed_timestamp) = knative.other.get("ts").map_or((None, None), |value| {
        let (display, parsed) = timestamp_details(value, time_format, timezone);
        (Some(display), parsed)
    });

    Some(StructuredLog {
        level: knative.level.to_uppercase(),
        message: knative.msg.trim().to_string(),
        timestamp,
        parsed_timestamp,
        others: None,
        consumed_fields: vec![
            "/level".to_string(),
            "/msg".to_string(),
            "/ts".to_string(),
            "/stacktrace".to_string(),
        ],
        extra_fields: Vec::new(),
        stacktrace: knative
            .other
            .get("stacktrace")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        raw_json: raw_json.cloned(),
        kail_prefix: prepared.kail_prefix.clone(),
    })
}

fn parse_logrus(
    prepared: &PreparedLine,
    raw_json: Option<&Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let raw_json = raw_json?;
    let message = json_string(raw_json, &["/msg"])?;
    let level = json_string(raw_json, &["/level"])?;
    let (timestamp, parsed_timestamp) =
        json_timestamp(raw_json, &["/time"], time_format, timezone)?;
    let stacktrace = json_string(raw_json, &["/stack", "/stacktrace"]).map(ToOwned::to_owned);

    Some(build_structured_log(
        prepared,
        raw_json.clone(),
        StructuredFields {
            level,
            message,
            timestamp: Some(timestamp),
            parsed_timestamp,
            others: None,
            stacktrace,
            consumed_fields: vec![
                "/level".to_string(),
                "/msg".to_string(),
                "/time".to_string(),
                "/stack".to_string(),
                "/stacktrace".to_string(),
            ],
        },
    ))
}

fn parse_zerolog(
    prepared: &PreparedLine,
    raw_json: Option<&Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let raw_json = raw_json?;
    let message = json_string(raw_json, &["/message"])?;
    let level = json_string(raw_json, &["/level"])?;
    let (timestamp, parsed_timestamp) =
        json_timestamp(raw_json, &["/time"], time_format, timezone)?;
    let stacktrace = json_string(raw_json, &["/stack"]).map(ToOwned::to_owned);

    Some(build_structured_log(
        prepared,
        raw_json.clone(),
        StructuredFields {
            level,
            message,
            timestamp: Some(timestamp),
            parsed_timestamp,
            others: None,
            stacktrace,
            consumed_fields: vec![
                "/level".to_string(),
                "/message".to_string(),
                "/time".to_string(),
                "/stack".to_string(),
            ],
        },
    ))
}

fn parse_ecs(
    prepared: &PreparedLine,
    raw_json: Option<&Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let raw_json = raw_json?;
    let message = json_string(raw_json, &["/message"])?;
    let level = json_string(raw_json, &["/log/level"])?;
    let (timestamp, parsed_timestamp) =
        json_timestamp(raw_json, &["/@timestamp"], time_format, timezone)?;
    let stacktrace = json_string(raw_json, &["/error/stack_trace"]).map(ToOwned::to_owned);

    Some(build_structured_log(
        prepared,
        raw_json.clone(),
        StructuredFields {
            level,
            message,
            timestamp: Some(timestamp),
            parsed_timestamp,
            others: None,
            stacktrace,
            consumed_fields: vec![
                "/message".to_string(),
                "/@timestamp".to_string(),
                "/log/level".to_string(),
                "/error/stack_trace".to_string(),
            ],
        },
    ))
}

fn parse_cloud_logging(
    prepared: &PreparedLine,
    raw_json: Option<&Value>,
    time_format: &str,
    timezone: Option<&str>,
) -> Option<StructuredLog> {
    let raw_json = raw_json?;
    let level = normalize_cloud_logging_level(json_string(raw_json, &["/severity"])?);
    let message = json_string(
        raw_json,
        &["/message", "/jsonPayload/message", "/textPayload"],
    )?;
    let timestamp = json_timestamp(
        raw_json,
        &["/timestamp", "/time", "/receiveTimestamp"],
        time_format,
        timezone,
    );
    let stacktrace = json_string(
        raw_json,
        &[
            "/jsonPayload/stacktrace",
            "/jsonPayload/stack_trace",
            "/stacktrace",
        ],
    )
    .map(ToOwned::to_owned);

    Some(build_structured_log(
        prepared,
        raw_json.clone(),
        StructuredFields {
            level,
            message,
            timestamp: timestamp.as_ref().map(|(display, _)| display.clone()),
            parsed_timestamp: timestamp.and_then(|(_, parsed)| parsed),
            others: None,
            stacktrace,
            consumed_fields: vec![
                "/severity".to_string(),
                "/message".to_string(),
                "/jsonPayload/message".to_string(),
                "/textPayload".to_string(),
                "/timestamp".to_string(),
                "/time".to_string(),
                "/receiveTimestamp".to_string(),
                "/jsonPayload/stacktrace".to_string(),
                "/jsonPayload/stack_trace".to_string(),
                "/stacktrace".to_string(),
            ],
        },
    ))
}

fn build_structured_log(
    prepared: &PreparedLine,
    raw_json: Value,
    fields: StructuredFields<'_>,
) -> StructuredLog {
    StructuredLog {
        level: fields.level.to_uppercase(),
        message: fields.message.trim().to_string(),
        timestamp: fields.timestamp,
        parsed_timestamp: fields.parsed_timestamp,
        others: fields.others,
        consumed_fields: fields.consumed_fields,
        extra_fields: Vec::new(),
        stacktrace: fields.stacktrace,
        raw_json: Some(raw_json),
        kail_prefix: prepared.kail_prefix.clone(),
    }
}

fn json_string<'a>(value: &'a Value, pointers: &[&str]) -> Option<&'a str> {
    pointers.iter().find_map(|pointer| {
        value
            .pointer(pointer)
            .and_then(|candidate| match candidate {
                Value::String(inner) if !inner.is_empty() => Some(inner.as_str()),
                _ => None,
            })
    })
}

fn json_timestamp(
    value: &Value,
    pointers: &[&str],
    time_format: &str,
    timezone: Option<&str>,
) -> Option<(String, Option<DateTime<Utc>>)> {
    pointers.iter().find_map(|pointer| {
        value
            .pointer(pointer)
            .map(|candidate| timestamp_details(candidate, time_format, timezone))
    })
}

fn timestamp_details(
    value: &Value,
    time_format: &str,
    timezone: Option<&str>,
) -> (String, Option<DateTime<Utc>>) {
    (
        crate::utils::convert_ts_float_or_str(value, time_format, timezone),
        crate::utils::parse_timestamp_value(value),
    )
}

fn normalize_cloud_logging_level(level: &str) -> &str {
    match level.to_uppercase().as_str() {
        "DEBUG" => "DEBUG",
        "WARNING" | "WARN" => "WARNING",
        "ERROR" | "CRITICAL" | "ALERT" | "EMERGENCY" => "ERROR",
        _ => "INFO",
    }
}

pub fn parse_kail_lines(config: &Config, rawline: &str) -> Option<String> {
    let reg = Regex::new(KAIL_RE).ok()?;
    if !reg.is_match(rawline) {
        return None;
    }

    let mut kail_msg_prefix = config.kail_prefix_format.clone();
    let capture = reg.captures(rawline)?;
    let namespace = capture.name("namespace")?.as_str();
    let pod = capture.name("pod")?.as_str();
    let container = capture.name("container")?.as_str();
    kail_msg_prefix = kail_msg_prefix
        .replace("{namespace}", namespace)
        .replace("{pod}", pod)
        .replace("{container}", container)
        .replace("\\n", "\n");
    Some(kail_msg_prefix)
}

pub fn is_kubectl_events_header(line: &str, state: &mut ParseState) -> bool {
    let header = line.trim_start();
    if header.starts_with("LAST SEEN") && header.contains("TYPE") && header.contains("REASON") {
        let last_seen_idx = header.find("LAST SEEN").unwrap_or(0);
        let type_idx = header.find("TYPE").unwrap_or(0);
        let reason_idx = header.find("REASON").unwrap_or(0);
        let object_idx = header.find("OBJECT").unwrap_or(0);
        let message_idx = header.find("MESSAGE").unwrap_or(0);
        state.kubectl_events_cols =
            Some((last_seen_idx, type_idx, reason_idx, object_idx, message_idx));
        return true;
    }
    false
}

pub fn parse_kubectl_event_line(line: &str, state: &ParseState) -> Option<KubectlEvent> {
    let (last_seen_idx, type_idx, reason_idx, object_idx, message_idx) =
        state.kubectl_events_cols?;

    Some(KubectlEvent {
        last_seen: line.get(last_seen_idx..type_idx)?.trim().to_string(),
        type_: line.get(type_idx..reason_idx)?.trim().to_string(),
        reason: line.get(reason_idx..object_idx)?.trim().to_string(),
        object: line.get(object_idx..message_idx)?.trim().to_string(),
        message: line.get(message_idx..)?.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::{Config, KailPrefix};

    use super::*;

    #[test]
    fn parses_pac_logs() {
        let line = "{\"severity\":\"INFO\",\"timestamp\":\"2022-04-25T10:24:30.155404234Z\",\"caller\":\"foo.rs:1\",\"message\":\"hello moto\"}";
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "hello moto");
        assert_eq!(log.level, "INFO");
        assert!(log.timestamp.is_some());
    }

    #[test]
    fn parses_knative_logs() {
        let line = "{\"level\":\"DEBUG\",\"msg\":\"knative log\",\"ts\":1650602040.0}";
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "knative log");
        assert_eq!(log.level, "DEBUG");
        assert!(log.timestamp.is_some());
    }

    #[test]
    fn parses_caddy_access_logs() {
        let line = r#"{"level":"info","ts":1588610091.0,"logger":"http.log.access","msg":"handled request","request":{"method":"GET","uri":"/api/users"},"status":200,"duration":0.001234}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "GET /api/users -> 200 (1ms)");
        assert_eq!(log.level, "INFO");
        assert!(log.timestamp.is_some());
    }

    #[test]
    fn does_not_treat_generic_request_logs_as_caddy() {
        let line = r#"{"level":"info","message":"request completed","time":"2022-04-25T14:20:32.505637358Z","request":{"method":"GET","uri":"/api/users"},"status":200,"duration":0.001234}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "request completed");
        assert_eq!(log.level, "INFO");
        assert_eq!(log.timestamp.as_deref(), Some("14:20:32"));
    }

    #[test]
    fn parses_logrus_logs() {
        let line =
            r#"{"level":"warning","msg":"logrus log","time":"2022-04-25T14:20:32.505637358Z"}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "logrus log");
        assert_eq!(log.level, "WARNING");
        assert_eq!(log.timestamp.as_deref(), Some("14:20:32"));
    }

    #[test]
    fn parses_zerolog_logs() {
        let line = r#"{"level":"error","message":"zerolog log","time":"2022-04-25T14:20:32.505637358Z","stack":"trace"}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "zerolog log");
        assert_eq!(log.level, "ERROR");
        assert_eq!(log.timestamp.as_deref(), Some("14:20:32"));
        assert_eq!(log.stacktrace.as_deref(), Some("trace"));
    }

    #[test]
    fn parses_ecs_logs() {
        let line = r#"{"@timestamp":"2022-04-25T14:20:32.505637358Z","message":"ecs log","log":{"level":"info"},"error":{"stack_trace":"trace"}}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "ecs log");
        assert_eq!(log.level, "INFO");
        assert_eq!(log.timestamp.as_deref(), Some("14:20:32"));
        assert_eq!(log.stacktrace.as_deref(), Some("trace"));
    }

    #[test]
    fn parses_cloud_logging_logs() {
        let line = r#"{"severity":"ERROR","textPayload":"cloud log","timestamp":"2022-04-25T14:20:32.505637358Z","jsonPayload":{"stacktrace":"trace"}}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.message, "cloud log");
        assert_eq!(log.level, "ERROR");
        assert_eq!(log.timestamp.as_deref(), Some("14:20:32"));
        assert_eq!(log.stacktrace.as_deref(), Some("trace"));
    }

    #[test]
    fn normalizes_cloud_logging_critical_to_error() {
        let line = r#"{"severity":"CRITICAL","textPayload":"cloud log","timestamp":"2022-04-25T14:20:32.505637358Z"}"#;
        let prepared = prepare_line(&Config::default(), line);
        let log = parse_structured_log(&Config::default(), &prepared).unwrap();
        assert_eq!(log.level, "ERROR");
    }

    #[test]
    fn parses_custom_json_keys() {
        let mut keys = HashMap::new();
        keys.insert("msg".to_string(), "foo".to_string());
        keys.insert("level".to_string(), "bar".to_string());
        keys.insert("ts".to_string(), "ts".to_string());
        let config = Config {
            json_keys: keys,
            ..Config::default()
        };
        let prepared = prepare_line(
            &config,
            r#"{"foo":"hello","bar":"info","ts":"2022-04-22T04:34:00.628550164Z"}"#,
        );
        let log = parse_structured_log(&config, &prepared).unwrap();
        assert_eq!(log.message, "hello");
        assert_eq!(log.level, "info");
        assert_eq!(log.timestamp.as_deref(), Some("04:34:00"));
    }

    #[test]
    fn parses_custom_json_float_timestamp() {
        let mut keys = HashMap::new();
        keys.insert("msg".to_string(), "foo".to_string());
        keys.insert("level".to_string(), "level".to_string());
        keys.insert("ts".to_string(), "bar".to_string());
        let config = Config {
            json_keys: keys,
            ..Config::default()
        };
        let prepared = prepare_line(
            &config,
            r#"{"foo":"hello","level":"info","bar":1650602040.6289625}"#,
        );
        let log = parse_structured_log(&config, &prepared).unwrap();
        assert_eq!(log.message, "hello");
        assert_eq!(log.level, "info");
        assert_eq!(log.timestamp.as_deref(), Some("04:34:00"));
    }

    #[test]
    fn skips_missing_custom_json_keys() {
        let mut keys = HashMap::new();
        keys.insert("msg".to_string(), "foo".to_string());
        keys.insert("level".to_string(), "level".to_string());
        keys.insert("ts".to_string(), "missing_timestamp".to_string());
        let config = Config {
            json_keys: keys,
            ..Config::default()
        };
        let prepared = prepare_line(&config, r#"{"foo":"hello","level":"info"}"#);
        let log = parse_structured_log(&config, &prepared).unwrap();
        assert_eq!(log.message, "hello");
        assert_eq!(log.level, "info");
        assert_eq!(log.timestamp, None);
    }

    #[test]
    fn applies_kail_prefix_template() {
        let config = Config {
            kail_prefix: KailPrefix::Show,
            kail_prefix_format: "{container}\n".to_string(),
            ..Config::default()
        };
        let prepared = prepare_line(
            &config,
            r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","caller":"pipelineascode/status.go:59","message":"updated"}"#,
        );
        let log = parse_structured_log(&config, &prepared).unwrap();
        assert_eq!(log.kail_prefix.as_deref(), Some("container\n"));
        assert_eq!(log.message, "updated");
    }

    #[test]
    fn kubectl_events_are_detected() {
        let header =
            "LAST SEEN   TYPE      REASON              OBJECT                                               MESSAGE";
        let event_line = "119m        Warning   Unhealthy           pod/pipelines-as-code-controller-76d86f74bb-vxjtd    Readiness probe failed";
        let mut state = ParseState::default();
        assert!(is_kubectl_events_header(header, &mut state));
        let parsed = parse_kubectl_event_line(event_line, &state).unwrap();
        assert_eq!(parsed.last_seen, "119m");
        assert_eq!(parsed.type_, "Warning");
        assert_eq!(parsed.reason, "Unhealthy");
        assert_eq!(
            parsed.object,
            "pod/pipelines-as-code-controller-76d86f74bb-vxjtd"
        );
    }
}
