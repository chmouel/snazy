use chrono::NaiveDateTime;
use serde_json::Value;
use yansi::Paint;

/// replace info level DEBUG, WARNING, ERROR, INFO, FATAL by pretty characters
pub fn level_symbols(level: &str) -> String {
    match level {
        "DEBUG" => "🐛".to_string(),
        "WARNING" => "⚠️".to_string(),
        "ERROR" => "🚨".to_string(),
        "INFO" => "💡".to_string(),
        "FATAL" => "💀".to_string(),
        _ => "∙".to_string(),
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
        "github" => "",
        "gitlab" => "",
        "bitbucket-cloud" => "",
        "bitbucket-server" => "S",
        "incoming" => "",
        _ => provider,
    }
}

pub fn convert_str_to_ts(s: &str, time_format: &str) -> String {
    // TODO: don't unwrap blindly, try to some more parsing
    let ts = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
    ts.format(time_format).to_string()
}

fn convert_unix_ts(value: i64, time_format: &str) -> String {
    let ts = NaiveDateTime::from_timestamp_opt(value, 0).unwrap();
    ts.format(time_format).to_string()
}

pub fn convert_ts_float_or_str(value: &Value, time_format: &str) -> String {
    match value {
        Value::String(s) => convert_str_to_ts(s.as_str(), time_format),
        Value::Number(n) => convert_unix_ts(n.as_f64().unwrap() as i64, time_format),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_symbols() {
        // auto generated :D
        assert_eq!(level_symbols("DEBUG"), "🐛");
        assert_eq!(level_symbols("WARNING"), "⚠️");
        assert_eq!(level_symbols("ERROR"), "🚨");
        assert_eq!(level_symbols("INFO"), "💡");
        assert_eq!(level_symbols("FATAL"), "💀");
        assert_eq!(level_symbols("UNKNOWN"), "∙");
    }
    #[test]
    fn test_providers() {
        // auto generated :D
        assert_eq!(convert_pac_provider_to_fa_icon("github"), "");
        assert_eq!(convert_pac_provider_to_fa_icon("gitlab"), "");
        assert_eq!(convert_pac_provider_to_fa_icon("bitbucket-cloud"), "");
        assert_eq!(convert_pac_provider_to_fa_icon("incoming"), "\u{f08b}");
        assert_eq!(convert_pac_provider_to_fa_icon("bitbucket-server"), "S");
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
