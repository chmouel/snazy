use chrono::NaiveDateTime;
use yansi::Paint;

pub fn convert_str_to_ts(s: &str, time_format: &str) -> String {
    let ts = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
    ts.format(time_format).to_string()
}

pub fn color_by_level(level: &str) -> String {
    match level {
        "DEBUG" => format!("{:<19}", Paint::fixed(14, "DEBUG").to_string()),
        "WARNING" => format!("{:<19}", Paint::fixed(11, "WARN").to_string()),
        "ERROR" => format!("{:<18}", Paint::fixed(9, "ERROR").to_string()),
        "INFO" => format!("{:<19}", Paint::fixed(10, "INFO").to_string()),
        _ => format!("{:<19}", Paint::fixed(10, level).to_string()),
    }
}

pub fn convert_unix_ts(value: i64, time_format: &str) -> String {
    let ts = NaiveDateTime::from_timestamp(value, 0);
    ts.format(time_format).to_string()
}
