use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead};
use std::process::Command;
use std::sync::Arc;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use yansi::Paint;

use crate::config;
use crate::config::Config;
use crate::utils::{apply_regexps, custom_json_match};

const KAIL_RE: &str = r"^(?P<namespace>[^/]*)/(?P<pod>[^\[]*)\[(?P<container>[^]]*)]: (?P<line>.*)";

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

#[derive(Debug)]
pub struct Info {
    level: String,
    msg: String,
    timestamp: String,
    others: String,
    stacktrace: Option<String>,
}

// Add to Config struct (in config.rs):
// pub disable_coloring: bool,

pub fn extract_info(rawline: &str, config: &Config) -> HashMap<String, String> {
    let time_format = config.time_format.as_str();
    let timezone = config.timezone.as_deref();
    let mut msg = HashMap::new();
    let mut kail_msg_prefix = String::new();
    let mut line = rawline.to_string();

    if let Some(prefix) = parse_kail_lines(config, rawline) {
        if let Ok(replacer) = Regex::new(KAIL_RE) {
            line = replacer.replace_all(rawline, "$line").to_string();
            kail_msg_prefix = prefix;
        }
    }

    if !config.json_keys.is_empty() {
        msg = custom_json_match(config, time_format, &kail_msg_prefix, line.as_str());
    }

    // Try to parse as a generic JSON to extract stacktrace field
    // regardless of the specific log format
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&line) {
        if let Some(stacktrace) = json_value.get("stacktrace").and_then(|s| s.as_str()) {
            msg.insert("stacktrace".to_string(), stacktrace.to_string());
        }
    }

    if let Ok(p) = serde_json::from_str::<Pac>(line.as_str()) {
        msg.insert("msg".to_string(), p.message.trim().to_string());
        msg.insert("level".to_string(), p.severity.to_uppercase());
        // parse timestamp to a unix timestamp
        msg.insert(
            "ts".to_string(),
            crate::utils::convert_str_to_ts(
                p.timestamp.as_str(),
                config.time_format.as_str(),
                config.timezone.as_deref(),
            ),
        );
        let mut others = String::new();
        if p.other.contains_key("provider") {
            if let Some(provider) = p.other["provider"].as_str() {
                others.push_str(crate::utils::convert_pac_provider_to_fa_icon(provider));
                msg.insert("others".to_string(), format!("{others} "));
            }
        }

        // Extract stacktrace from Pac struct if available
        if p.other.contains_key("stacktrace") {
            if let Some(stacktrace) = p.other["stacktrace"].as_str() {
                msg.insert("stacktrace".to_string(), stacktrace.to_string());
            }
        }
    }

    if let Ok(p) = serde_json::from_str::<Knative>(line.as_str()) {
        msg.insert("msg".to_string(), p.msg.trim().to_string());
        msg.insert("level".to_string(), p.level.to_uppercase());
        if let Some(ts) = p.other.get("ts") {
            msg.insert(
                String::from("ts"),
                crate::utils::convert_ts_float_or_str(ts, time_format, timezone),
            );
        }

        // Extract stacktrace from Knative struct if available
        if p.other.contains_key("stacktrace") {
            if let Some(stacktrace) = p.other["stacktrace"].as_str() {
                msg.insert("stacktrace".to_string(), stacktrace.to_string());
            }
        }
    }

    // Kail prefix logic
    if config.kail_prefix == crate::config::KailPrefix::Hide
        || kail_msg_prefix.is_empty()
        || !msg.contains_key("msg")
    {
        // Do nothing, keep the message as is
    } else {
        let prefix = match config.coloring {
            crate::config::Coloring::Never => kail_msg_prefix.clone(),
            _ => kail_msg_prefix.blue().to_string(),
        };
        *msg.get_mut("msg").unwrap() = format!("{} {}", prefix, msg["msg"]);
    }
    msg
}

fn parse_kail_lines(config: &Config, rawline: &str) -> Option<String> {
    let reg = Regex::new(KAIL_RE).unwrap();
    if !reg.is_match(rawline) {
        return None;
    }
    let mut kail_msg_prefix = config.kail_prefix_format.clone();
    let capture = reg.captures(rawline).unwrap();
    let namespace = capture.name("namespace").unwrap().as_str();
    let pod = capture.name("pod").unwrap().as_str();
    let container = capture.name("container").unwrap().as_str();
    kail_msg_prefix = kail_msg_prefix
        .replace("{namespace}", namespace)
        .replace("{pod}", pod)
        .replace("{container}", container)
        .replace("\\n", "\n");
    Some(kail_msg_prefix)
}

pub fn action_on_regexp(config: &Config, line: &str) {
    let action_re =
        Regex::new(config.action_regexp.as_ref().unwrap()).expect("Invalid action_regexp");
    if let Some(captures) = action_re.captures(line) {
        let regexpmatch = captures.get(0).unwrap().as_str();
        // replace {} by the actual match
        let action_command = config
            .action_command
            .as_ref()
            .unwrap()
            .replace("{}", regexpmatch);
        let action_command_str = action_command.clone();
        if Command::new("sh")
            .arg("-c")
            .arg(action_command)
            .spawn()
            .is_ok()
        {
            println!("Spawned command: for action: {}", Paint::cyan(regexpmatch));
        } else {
            eprintln!("Failed to spawn action command: {action_command_str}");
        }
    }
}

pub fn do_line(config: &Config, line: &str) -> Option<Info> {
    // exclude lines with only space or empty
    if line.trim().is_empty() {
        return None;
    }

    if config.action_regexp.is_some() {
        action_on_regexp(config, line);
    }

    let msg = extract_info(line, config);
    // use an empty map check instead of serializing to string
    if msg.is_empty() {
        println!(
            "{}",
            apply_regexps(&config.regexp_colours, line.to_string())
        );
        return None;
    }

    if config.skip_line_regexp.iter().any(|s| match Regex::new(s) {
        Ok(re) => msg.get("msg").is_some_and(|m| re.is_match(m)),
        Err(e) => {
            eprintln!("Invalid skip_line_regexp pattern '{s}': {e}");
            false
        }
    }) {
        return None;
    }

    if !config.filter_levels.is_empty()
        && !config
            .filter_levels
            .contains(config::level_from_str(&msg["level"].to_lowercase()))
    {
        return None;
    }

    // Level symbols logic
    let level = match config.level_symbols {
        crate::config::LevelSymbols::Emoji => {
            crate::utils::level_symbols(msg.get("level").unwrap())
        }
        crate::config::LevelSymbols::Text => {
            crate::utils::color_by_level(msg.get("level").unwrap())
        }
    };
    let ts = if let Some(ts) = msg.get("ts") {
        ts.fixed(13).to_string()
    } else {
        String::new()
    };
    // Coloring logic for others
    let other = if let Some(o) = msg.get("others") {
        match config.coloring {
            crate::config::Coloring::Never => format!(" {o}"),
            _ => format!(" {}", Paint::cyan(o).italic()),
        }
    } else {
        String::new()
    };
    let mut themsg = msg.get("msg").unwrap().to_string();

    if !config.regexp_colours.is_empty() {
        themsg = apply_regexps(&config.regexp_colours, themsg);
    }

    // Get stacktrace if available
    let stacktrace = msg.get("stacktrace").map(std::string::ToString::to_string);

    Some(Info {
        level,
        timestamp: ts,
        others: other,
        msg: themsg,
        stacktrace,
    })
}

pub fn read_from_stdin(config: &Arc<Config>) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let parseline = &line.unwrap();

        if let Some(info) = do_line(config, parseline) {
            println!(
                "{} {} {}{}",
                info.level, info.timestamp, info.others, info.msg
            );

            // Display stacktrace if available with prettier formatting
            if let Some(stack) = &info.stacktrace {
                // Only display stacktrace if hide_stacktrace is false
                if !config.hide_stacktrace {
                    println!("\n{}", "─".repeat(80).fixed(8));
                    println!("{}", " Stacktrace:".red().bold());

                    // Format each line of the stacktrace with color highlighting
                    for stack_line in stack.lines() {
                        // Format the line with colored components
                        let formatted_line = format_stack_line(
                            stack_line,
                            config.coloring == crate::config::Coloring::Never,
                        );
                        println!("   {formatted_line}");
                    }

                    println!("{}\n", "─".repeat(80).fixed(8));
                }
            }
        }
    }
}

// read from file and output to the writer. This makes it easy to unittest
pub fn read_a_file(config: &Config, filename: &str, writeto: &mut dyn io::Write) {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("file {filename}, {e}");
            return; // Gracefully return instead of exiting the process
        }
    };
    let buf_reader = BufReader::new(file);
    for line in buf_reader.lines() {
        let parseline = &line.unwrap();

        if let Some(info) = do_line(config, parseline) {
            writeln!(
                writeto,
                "{} {} {}{}",
                info.level, info.timestamp, info.others, info.msg
            )
            .unwrap();

            // Display stacktrace if available with prettier formatting
            if let Some(stack) = &info.stacktrace {
                // Only display stacktrace if hide_stacktrace is false
                if !config.hide_stacktrace {
                    writeln!(writeto, "\n{}", "─".repeat(80).fixed(8)).unwrap();
                    writeln!(writeto, "{}", " Stacktrace:".red().bold()).unwrap();

                    // Format each line of the stacktrace with color highlighting
                    for stack_line in stack.lines() {
                        // Format the line with colored components
                        let formatted_line = format_stack_line(
                            stack_line,
                            config.coloring == crate::config::Coloring::Never,
                        );
                        writeln!(writeto, "   {formatted_line}").unwrap();
                    }

                    writeln!(writeto, "{}\n", "─".repeat(80).fixed(8)).unwrap();
                }
            }
        }
    }
}

// read from a bunch files and pass read_from_stdin to stdout
pub fn read_from_files(config: &Arc<Config>) {
    for filename in config.files.as_ref().unwrap() {
        // write to a BufWriter that output to stdout
        let stdout = io::stdout();
        let stdout = stdout.lock();
        let mut stdout = io::BufWriter::new(stdout);
        read_a_file(config, filename, &mut stdout);
    }
}

// Format a line of stacktrace with colored components
fn format_stack_line(line: &str, disable_coloring: bool) -> String {
    // Check if this is a file path line with line numbers (contains a colon with numbers after it)
    if line.contains(".go:")
        || line.contains(".rs:")
        || line.contains(".js:")
        || line.contains(".py:")
        || line.contains(".cpp:")
        || line.contains(".ts:")
        || line.contains(".rb:")
    {
        // Format lines with file paths and line numbers
        if let Some(last_slash_pos) = line.rfind('/') {
            // Extract the filename and path
            let path = &line[0..=last_slash_pos];
            let rest = &line[last_slash_pos + 1..];

            // Try to split on colon to separate filename from line number
            if let Some(colon_pos) = rest.find(':') {
                let filename = &rest[0..colon_pos];
                let line_num = &rest[colon_pos + 1..];

                // Return colored parts
                return format!(
                    "{}{}{}",
                    if disable_coloring {
                        path.to_string()
                    } else {
                        path.fixed(15).to_string()
                    },
                    if disable_coloring {
                        filename.to_string()
                    } else {
                        filename.yellow().bold().to_string()
                    },
                    if disable_coloring {
                        format!(":{line_num}")
                    } else {
                        format!(":{line_num}").green().to_string()
                    }
                );
            }

            // If no colon found, just color the filename
            return format!(
                "{}{}",
                if disable_coloring {
                    path.to_string()
                } else {
                    path.fixed(15).to_string()
                },
                if disable_coloring {
                    rest.to_string()
                } else {
                    rest.yellow().bold().to_string()
                }
            );
        }

        // If no slash found, check for a colon to split filename and line number
        if let Some(colon_pos) = line.find(':') {
            let filename = &line[0..colon_pos];
            let line_num = &line[colon_pos + 1..];

            return format!(
                "{}{}",
                if disable_coloring {
                    filename.to_string()
                } else {
                    filename.yellow().bold().to_string()
                },
                if disable_coloring {
                    format!(":{line_num}")
                } else {
                    format!(":{line_num}").green().to_string()
                }
            );
        }
    }

    // Format function name lines
    if let Some(dot_pos) = line.rfind('.') {
        let package_path = &line[0..=dot_pos];
        let func_name = &line[dot_pos + 1..];

        return format!(
            "{}{}",
            if disable_coloring {
                package_path.to_string()
            } else {
                package_path.fixed(15).to_string()
            },
            if disable_coloring {
                func_name.to_string()
            } else {
                func_name.cyan().bold().to_string()
            }
        );
    }

    // Default formatting if no patterns matched
    if disable_coloring {
        line.to_string()
    } else {
        line.fixed(15).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_extract_info_pac() {
        let line = "{\"severity\":\"INFO\",\"timestamp\":\"2022-04-25T10:24:30.155404234Z\",\"caller\":\"foo.rs:1\",\"message\":\"hello moto\"}";
        let config = Config::default();
        let info = extract_info(line, &config);
        assert_eq!(
            info.get("msg").map_or(String::new(), |v| v.clone()),
            "hello moto"
        );
        assert_eq!(
            info.get("level").map_or(String::new(), |v| v.clone()),
            "INFO"
        );
        assert!(info.get("ts").map_or(false, |v| !v.is_empty()));
    }

    #[test]
    fn test_extract_info_knative() {
        let line = "{\"level\":\"DEBUG\",\"msg\":\"knative log\",\"ts\":1650602040.0}";
        let config = Config::default();
        let info = extract_info(line, &config);
        assert_eq!(
            info.get("msg").map_or(String::new(), |v| v.clone()),
            "knative log"
        );
        assert_eq!(
            info.get("level").map_or(String::new(), |v| v.clone()),
            "DEBUG"
        );
        assert!(info.get("ts").map_or(false, |v| !v.is_empty()));
    }

    #[test]
    fn test_do_line_level_symbols() {
        let config = Config {
            level_symbols: crate::config::LevelSymbols::Emoji,
            ..Config::default()
        };
        let line = "{\"level\":\"INFO\",\"msg\":\"symbol test\",\"timestamp\":\"2022-04-25T10:24:30.155404234Z\"}";
        let result = do_line(&config, line);
        if result.is_none() {
            println!("DEBUG: do_line returned None for input: {line}");
        }
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, "💡");
    }

    #[test]
    fn test_format_stack_line_coloring_toggle() {
        let line = "/foo/bar.rs:42";
        // With coloring
        let colored = super::format_stack_line(line, false);
        assert!(
            colored.contains("\x1b["),
            "Expected ANSI color codes in output"
        );
        // Without coloring
        let plain = super::format_stack_line(line, true);
        assert!(
            !plain.contains("\x1b["),
            "Expected no ANSI color codes in output"
        );
        assert!(plain.contains("/foo/bar.rs:42"));
    }

    #[test]
    fn test_kail_prefix_coloring_toggle() {
        let config = Config {
            kail_prefix_format: "{namespace}/{pod}[{container}]".to_string(),
            ..Config::default()
        };
        // Use a valid Kail line with a proper JSON log message
        let line = "ns/pod[container]: {\"msg\":\"foo\",\"level\":\"INFO\"}";
        // With coloring
        let mut config_colored = config.clone();
        config_colored.coloring = crate::config::Coloring::Always;
        let info_colored = super::extract_info(line, &config_colored);
        assert!(
            info_colored.get("msg").unwrap().contains("\x1b["),
            "Expected ANSI color codes in prefix"
        );
        // Without coloring
        let mut config_plain = config.clone();
        config_plain.coloring = crate::config::Coloring::Never;
        let info_plain = super::extract_info(line, &config_plain);
        assert!(
            !info_plain.get("msg").unwrap().contains("\x1b["),
            "Expected no ANSI color codes in prefix"
        );
        assert!(info_plain
            .get("msg")
            .unwrap()
            .contains("ns/pod[container] foo"));
    }
}
