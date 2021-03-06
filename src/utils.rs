use chrono::NaiveDateTime;
use serde_json::Value;
use yansi::Paint;

/// replace info level DEBUG, WARNING, ERROR, INFO, FATAL by pretty characters
pub fn level_symbols(level: &str) -> String {
    match level {
        "DEBUG" => "ð".to_string(),
        "WARNING" => "â ïļ".to_string(),
        "ERROR" => "ðĻ".to_string(),
        "INFO" => "ðĄ".to_string(),
        "FATAL" => "ð".to_string(),
        _ => "â".to_string(),
    }
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

pub fn convert_pac_provider_to_fa_icon(provider: &str) -> &str {
    match provider {
        "github" => "ï",
        "gitlab" => "ï",
        "bitbucket-cloud" => "ïą",
        "bitbucket-server" => "ïąS",
        "incoming" => "ï",
        _ => provider,
    }
}

pub fn convert_str_to_ts(s: &str, time_format: &str) -> String {
    // TODO: don't unwrap blindly, try to some more parsing
    let ts = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
    ts.format(time_format).to_string()
}

fn convert_unix_ts(value: i64, time_format: &str) -> String {
    let ts = NaiveDateTime::from_timestamp(value, 0);
    ts.format(time_format).to_string()
}

pub fn convert_ts_float_or_str(value: &Value, time_format: &str) -> String {
    match value {
        Value::String(s) => convert_str_to_ts(s.as_str(), time_format),
        Value::Number(n) => convert_unix_ts(n.as_f64().unwrap() as i64, time_format),
        _ => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_symbols() {
        // auto generated :D
        assert_eq!(level_symbols("DEBUG"), "ð");
        assert_eq!(level_symbols("WARNING"), "â ïļ");
        assert_eq!(level_symbols("ERROR"), "ðĻ");
        assert_eq!(level_symbols("INFO"), "ðĄ");
        assert_eq!(level_symbols("FATAL"), "ð");
        assert_eq!(level_symbols("UNKNOWN"), "â");
    }
    #[test]
    fn test_providers() {
        // auto generated :D
        assert_eq!(convert_pac_provider_to_fa_icon("github"), "ï");
        assert_eq!(convert_pac_provider_to_fa_icon("gitlab"), "ï");
        assert_eq!(convert_pac_provider_to_fa_icon("bitbucket-cloud"), "ïą");
        assert_eq!(convert_pac_provider_to_fa_icon("incoming"), "\u{f08b}");
        assert_eq!(convert_pac_provider_to_fa_icon("bitbucket-server"), "ïąS");
        assert_eq!(convert_pac_provider_to_fa_icon("UNKNOWN"), "UNKNOWN");
    }

    #[test]
    fn test_convert_ts_float_or_str() {
        // auto generated :D
        assert_eq!(
            convert_ts_float_or_str(
                &Value::String("2020-01-01T00:00:00.000Z".to_string()),
                "%Y-%m-%d %H:%M:%S"
            ),
            "2020-01-01 00:00:00"
        );
    }
}
