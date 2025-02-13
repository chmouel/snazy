use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
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
        "DEBUG" => format!("{:<19}", "DEBUG".fixed(14).to_string()),
        "WARNING" => format!("{:<19}", "WARN".yellow().to_string()),
        "ERROR" => format!("{:<18}", "ERROR".red().to_string()),
        "INFO" => format!("{:<19}", "INFO".green().to_string()),
        _ => format!("{:<19}", level.fixed(4).to_string()),
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

pub fn convert_str_to_ts(s: &str, time_format: &str, timezone: Option<&str>) -> String {
    if let Ok(ts) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ") {
        let utc_dt = Utc.from_utc_datetime(&ts);
        if let Some(tz) = timezone {
            if let Ok(tz) = tz.parse::<Tz>() {
                return utc_dt.with_timezone(&tz).format(time_format).to_string();
            }
        }
        return utc_dt.format(time_format).to_string();
    }
    s.to_string()
}

fn convert_unix_ts(value: i64, time_format: &str, timezone: Option<&str>) -> String {
    if let Some(ts) = DateTime::from_timestamp(value, 0) {
        if let Some(tz) = timezone {
            if let Ok(tz) = tz.parse::<Tz>() {
                return ts.with_timezone(&tz).format(time_format).to_string();
            }
        }
        return ts.format(time_format).to_string();
    }
    value.to_string()
}

pub fn convert_ts_float_or_str(value: &Value, time_format: &str, timezone: Option<&str>) -> String {
    match value {
        Value::String(s) => convert_str_to_ts(s.as_str(), time_format, timezone),
        Value::Number(n) => convert_unix_ts(n.as_f64().unwrap() as i64, time_format, timezone),
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
                "%Y-%m-%d %H:%M:%S",
                None
            ),
            "2020-01-01 00:00:00"
        );
    }

    #[test]
    fn test_convert_ts_float_or_str_with_timezone() {
        assert_eq!(
            convert_ts_float_or_str(
                &Value::String("2020-01-01T00:00:00.000Z".to_string()),
                "%Y-%m-%d %H:%M:%S",
                Some("Europe/Paris")
            ),
            "2020-01-01 01:00:00" // Paris is UTC+1
        );
    }
}
