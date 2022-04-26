use chrono::NaiveDateTime;
use serde_json::Value;
use yansi::Paint;

pub fn color_by_level(level: &str) -> String {
    match level {
        "DEBUG" => format!("{:<19}", Paint::fixed(14, "DEBUG").to_string()),
        "WARNING" => format!("{:<19}", Paint::fixed(11, "WARN").to_string()),
        "ERROR" => format!("{:<18}", Paint::fixed(9, "ERROR").to_string()),
        "INFO" => format!("{:<19}", Paint::fixed(10, "INFO").to_string()),
        _ => format!("{:<19}", Paint::fixed(10, level).to_string()),
    }
}

pub fn convert_str_to_ts(s: &str, time_format: &str) -> String {
    let ts = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
    ts.format(time_format).to_string()
}

fn convert_unix_ts(value: i64, time_format: &str) -> String {
    let ts = NaiveDateTime::from_timestamp(value, 0);
    ts.format(time_format).to_string()
}

pub fn conver_ts_float_or_str(value: &Value, time_format: &str) -> String {
    match value {
        Value::String(s) => convert_str_to_ts(s.as_str(), time_format),
        Value::Number(n) => convert_unix_ts(n.as_f64().unwrap() as i64, time_format),
        _ => "".to_string(),
    }
}
