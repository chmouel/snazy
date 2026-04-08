use chrono::{DateTime, LocalResult, TimeDelta, TimeZone, Utc};
use chrono_tz::Tz;
use regex::Regex;
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::time::Duration;
use yansi::Paint;
use yansi::Style;

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

/// Converts a string timestamp to formatted string, optionally applying a timezone.
/// Returns the original string if parsing fails.
pub fn convert_str_to_ts(s: &str, time_format: &str, timezone: Option<&str>) -> String {
    if let Some(timestamp) = parse_timestamp_str(s) {
        return format_timestamp(&timestamp, time_format, timezone);
    }
    s.to_string()
}

pub fn format_timestamp(
    timestamp: &DateTime<Utc>,
    time_format: &str,
    timezone: Option<&str>,
) -> String {
    if let Some(tz) = timezone {
        if let Ok(tz) = tz.parse::<Tz>() {
            return timestamp.with_timezone(&tz).format(time_format).to_string();
        }
    }
    timestamp.format(time_format).to_string()
}

pub fn parse_timestamp_str(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn parse_unix_ts(value: &Number) -> Option<DateTime<Utc>> {
    let raw = value.to_string();
    if raw.contains(['e', 'E']) {
        return None;
    }

    let (negative, unsigned) = raw
        .strip_prefix('-')
        .map_or((false, raw.as_str()), |rest| (true, rest));
    let (whole, fractional) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    let whole_seconds: i64 = whole.parse().ok()?;
    let (nanos, carry_second) = parse_fractional_nanos(fractional)?;

    let (seconds, nanos) = if negative {
        if nanos == 0 {
            (-whole_seconds, 0)
        } else {
            let whole_with_carry = whole_seconds.checked_add(i64::from(carry_second))?;
            let next_second = whole_with_carry.checked_add(1)?;
            (-next_second, 1_000_000_000_u32.saturating_sub(nanos))
        }
    } else {
        (whole_seconds.checked_add(i64::from(carry_second))?, nanos)
    };

    match Utc.timestamp_opt(seconds, nanos) {
        LocalResult::Single(timestamp) => Some(timestamp),
        _ => None,
    }
}

fn parse_fractional_nanos(fractional: &str) -> Option<(u32, bool)> {
    if fractional.is_empty() {
        return Some((0, false));
    }
    if !fractional.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let mut digits = fractional.chars();
    let nanos_digits: String = digits.by_ref().take(9).collect();
    let round_up = digits.next().is_some_and(|digit| digit >= '5');
    let nanos = format!("{nanos_digits:0<9}").parse::<u32>().ok()?;

    if round_up {
        if nanos == 999_999_999 {
            return Some((0, true));
        }
        return Some((nanos + 1, false));
    }

    Some((nanos, false))
}

pub fn parse_timestamp_value(value: &Value) -> Option<DateTime<Utc>> {
    match value {
        Value::String(s) => parse_timestamp_str(s),
        Value::Number(n) => {
            if let Some(value) = n.as_i64() {
                return match Utc.timestamp_opt(value, 0) {
                    LocalResult::Single(timestamp) => Some(timestamp),
                    _ => None,
                };
            }

            if let Some(value) = n.as_u64() {
                if let Ok(value) = i64::try_from(value) {
                    return match Utc.timestamp_opt(value, 0) {
                        LocalResult::Single(timestamp) => Some(timestamp),
                        _ => None,
                    };
                }
                return None;
            }

            parse_unix_ts(n)
        }
        _ => None,
    }
}

/// Converts a JSON value (string or number) to a formatted timestamp string.
/// Returns empty string for unsupported types or conversion errors.
pub fn convert_ts_float_or_str(value: &Value, time_format: &str, timezone: Option<&str>) -> String {
    match value {
        Value::String(s) => convert_str_to_ts(s.as_str(), time_format, timezone),
        Value::Number(n) => parse_timestamp_value(value).map_or_else(
            || n.to_string(),
            |timestamp| format_timestamp(&timestamp, time_format, timezone),
        ),
        _ => String::new(),
    }
}

pub fn format_time_delta(delta: TimeDelta) -> String {
    let negative = delta < TimeDelta::zero();
    let prefix = if negative { "-" } else { "+" };
    let abs_delta = if negative { -delta } else { delta };
    let total_milliseconds = abs_delta.num_milliseconds();

    if total_milliseconds < 1_000 {
        return format!("{prefix}{total_milliseconds}ms");
    }

    let total_seconds = abs_delta.num_seconds();
    if total_seconds < 60 {
        let tenths = ((total_milliseconds + 50) / 100).min(599);
        if tenths % 10 == 0 {
            return format!("{prefix}{}s", tenths / 10);
        }
        return format!("{prefix}{}.{:01}s", tenths / 10, tenths % 10);
    }

    if total_seconds < 3_600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        return format!("{prefix}{minutes}m{seconds:02}s");
    }

    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    format!("{prefix}{hours}h{minutes:02}m")
}

pub fn format_duration_compact(duration: Duration) -> String {
    let total_milliseconds = duration.as_millis();
    if total_milliseconds < 1_000 {
        return format!("{total_milliseconds}ms");
    }

    let total_seconds = duration.as_secs();
    if total_seconds < 60 {
        return format!("{total_seconds}s");
    }

    if total_seconds < 3_600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        return format!("{minutes}m{seconds:02}s");
    }

    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    format!("{hours}h{minutes:02}m")
}

/// Applies regex-based styles to a message string using the provided map.
/// Skips invalid regexes and returns the original string if no match.
pub fn apply_regexps(regexps: &HashMap<String, Style>, msg: String) -> String {
    let mut ret = msg;
    for (key, value) in regexps {
        let Ok(re) = Regex::new(format!(r"(?P<r>{})", key.as_str()).as_str()) else {
            continue;
        };
        if let Some(matched) = re.find(&ret) {
            let replace = matched.as_str().paint(*value).to_string();
            ret = re.replace_all(&ret, replace).to_string();
        }
    }
    ret
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

    #[test]
    fn test_convert_ts_float_or_str_non_float() {
        // Should return string representation for non-float number
        assert_eq!(
            convert_ts_float_or_str(
                &serde_json::json!("not_a_number"),
                "%Y-%m-%d %H:%M:%S",
                None
            ),
            "not_a_number"
        );
        assert_eq!(
            convert_ts_float_or_str(
                &serde_json::json!(9_223_372_036_854_775_807_u64),
                "%Y-%m-%d %H:%M:%S",
                None
            ),
            "9223372036854775807"
        );
    }

    #[test]
    fn test_convert_ts_float_or_str_keeps_subseconds() {
        assert_eq!(
            convert_ts_float_or_str(
                &serde_json::json!(1_650_602_040.628_962_5),
                "%H:%M:%S%.3f",
                None
            ),
            "04:34:00.628"
        );
    }

    #[test]
    fn test_apply_regexps() {
        let line = String::from("red blue normal");
        let mut regexps = HashMap::new();
        regexps.insert(String::from("red"), Style::new().fg(yansi::Color::Red));
        regexps.insert(
            String::from(r"\b(b.ue)\b"),
            Style::new().fg(yansi::Color::Blue),
        );
        let ret = apply_regexps(&regexps, line);
        assert_eq!(ret, format!("{} {} normal", "red".red(), "blue".blue()));
    }

    #[test]
    fn test_apply_regexps_invalid_regex() {
        let mut regexps = HashMap::new();
        regexps.insert(String::from("[invalid"), Style::new().fg(yansi::Color::Red));
        let msg = String::from("test [invalid regex");
        // Should not panic, should return original string
        assert_eq!(apply_regexps(&regexps, msg.clone()), msg);
    }

    #[test]
    fn test_format_time_delta_compact_units() {
        assert_eq!(format_time_delta(TimeDelta::milliseconds(12)), "+12ms");
        assert_eq!(format_time_delta(TimeDelta::milliseconds(999)), "+999ms");
        assert_eq!(format_time_delta(TimeDelta::seconds(1)), "+1s");
        assert_eq!(format_time_delta(TimeDelta::milliseconds(1_400)), "+1.4s");
        assert_eq!(format_time_delta(TimeDelta::milliseconds(59_900)), "+59.9s");
        assert_eq!(format_time_delta(TimeDelta::milliseconds(59_950)), "+59.9s");
        assert_eq!(format_time_delta(TimeDelta::seconds(60)), "+1m00s");
        assert_eq!(format_time_delta(TimeDelta::seconds(2)), "+2s");
        assert_eq!(format_time_delta(TimeDelta::seconds(123)), "+2m03s");
        assert_eq!(format_time_delta(TimeDelta::seconds(3_720)), "+1h02m");
        assert_eq!(format_time_delta(TimeDelta::milliseconds(-250)), "-250ms");
    }
}
