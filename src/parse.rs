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

    // Try custom JSON key extraction first
    if !config.json_keys.is_empty() {
        msg = custom_json_match(config, time_format, &kail_msg_prefix, line.as_str());
        eprintln!(
            "extract_info: after custom_json_match, keys: {:?}",
            msg.keys().collect::<Vec<_>>()
        );
    }

    // Fallback to Pac/Knative parsing if custom extraction failed
    if msg.is_empty() {
        // Try to parse as a generic JSON to extract stacktrace field
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
        eprintln!(
            "extract_info: after fallback, keys: {:?}",
            msg.keys().collect::<Vec<_>>()
        );
    }

    // Kail prefix logic (apply for both custom and fallback)
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
    eprintln!(
        "extract_info: final keys: {:?}",
        msg.keys().collect::<Vec<_>>()
    );
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

// Struct to track kubectl events parsing mode and column positions
#[derive(Default)]
pub struct ParseState {
    pub kubectl_events_mode: bool,
    pub kubectl_events_cols: Option<(usize, usize, usize, usize, usize)>,
}

// Helper to detect kubectl events header and record column positions in state
fn is_kubectl_events_header(line: &str, state: &mut ParseState) -> bool {
    let header = line.trim_start();
    if header.starts_with("LAST SEEN") && header.contains("TYPE") && header.contains("REASON") {
        // Find column start indices
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

// Helper to parse kubectl events line using header-based column positions from state
fn parse_kubectl_event_line(
    line: &str,
    state: &ParseState,
) -> Option<(String, String, String, String, String)> {
    if let Some((last_seen_idx, type_idx, reason_idx, object_idx, message_idx)) =
        state.kubectl_events_cols
    {
        let last_seen = line.get(last_seen_idx..type_idx)?.trim().to_string();
        let type_ = line.get(type_idx..reason_idx)?.trim().to_string();
        let reason = line.get(reason_idx..object_idx)?.trim().to_string();
        let object = line.get(object_idx..message_idx)?.trim().to_string();
        let message = line.get(message_idx..)?.trim().to_string();
        return Some((last_seen, type_, reason, object, message));
    }
    None
}

// Helper to colorize object type prefix
fn colorize_object_type(object: &str) -> String {
    use yansi::Color;
    let (prefix, rest) = if let Some(idx) = object.find('/') {
        (&object[..idx], &object[idx..])
    } else {
        (object, "")
    };
    let colored_prefix = match prefix {
        "pod" => Paint::magenta(prefix).bold(),
        "replicaset" => Paint::blue(prefix).bold(),
        "deployment" => Paint::green(prefix).bold(),
        "service" => Paint::yellow(prefix).bold(),
        "job" => Paint::cyan(prefix).bold(),
        "daemonset" => Paint::red(prefix).bold(),
        "statefulset" => Paint::new(prefix).fg(Color::Fixed(93)).bold(), // purple
        "configmap" => Paint::new(prefix).fg(Color::Fixed(208)).bold(),  // orange
        "secret" => Paint::new(prefix).fg(Color::Fixed(244)).bold(),     // gray
        _ => Paint::white(prefix).bold(),
    };
    format!("{colored_prefix}{rest}")
}

// Update do_line to use new header/line parsing
pub fn do_line(config: &Config, line: &str, state: &mut ParseState) -> Option<Info> {
    // exclude lines with only space or empty
    if line.trim().is_empty() {
        return None;
    }

    // Kubectl events mode detection
    if is_kubectl_events_header(line, state) {
        state.kubectl_events_mode = true;
        // Print header with color
        println!(
            "{} {} {} {} {}",
            "LAST SEEN".bold(),
            "TYPE".bold(),
            "REASON".bold(),
            "OBJECT".bold(),
            "MESSAGE".bold()
        );
        return None;
    }
    if state.kubectl_events_mode {
        if let Some((last_seen, type_, reason, object, message)) =
            parse_kubectl_event_line(line, state)
        {
            // Define fixed widths for each column
            const LAST_SEEN_WIDTH: usize = 8;
            const TYPE_WIDTH: usize = 8;
            const REASON_WIDTH: usize = 18;
            const OBJECT_WIDTH: usize = 52;

            // Pad each column to its width
            let last_seen_padded = format!("{last_seen:LAST_SEEN_WIDTH$}");
            let type_padded = format!("{type_:TYPE_WIDTH$}");
            let reason_padded = format!("{reason:REASON_WIDTH$}");
            let object_padded = format!("{object:OBJECT_WIDTH$}");

            // Colorize after padding (convert to &str)
            let type_colored = match type_.as_str() {
                "Warning" => Paint::red(&type_padded).bold(),
                "Normal" => Paint::green(&type_padded).bold(),
                _ => Paint::new(&type_padded).bold(),
            };
            let reason_colored = Paint::yellow(&reason_padded).bold();
            let object_colored = colorize_object_type(&object_padded);
            let last_seen_colored = Paint::new(&last_seen_padded).bold();
            // Colorize MESSAGE (allow regex coloring)
            let msg_colored = if config.regexp_colours.is_empty() {
                message.clone()
            } else {
                apply_regexps(&config.regexp_colours, message.clone())
            };
            // Use a literal format string for Clippy compliance
            println!(
                concat!("{} {} {} {} {}"),
                last_seen_colored, type_colored, reason_colored, object_colored, msg_colored
            );
            return None;
        }
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
    let mut themsg = msg.get("msg").cloned().unwrap_or_default();

    if !config.regexp_colours.is_empty() {
        themsg = apply_regexps(&config.regexp_colours, themsg);
    }

    // Get stacktrace if available
    let stacktrace = msg.get("stacktrace").cloned();

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
    let mut state = ParseState::default();
    for line in stdin.lock().lines() {
        let parseline = &line.unwrap();

        if let Some(info) = do_line(config, parseline, &mut state) {
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
    let mut state = ParseState::default();
    for line in buf_reader.lines() {
        let parseline = &line.unwrap();

        if let Some(info) = do_line(config, parseline, &mut state) {
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
        assert_eq!(info.get("msg").cloned().unwrap_or_default(), "hello moto");
        assert_eq!(info.get("level").cloned().unwrap_or_default(), "INFO");
        assert!(info.get("ts").is_some_and(|v| !v.is_empty()));
    }

    #[test]
    fn test_extract_info_knative() {
        let line = "{\"level\":\"DEBUG\",\"msg\":\"knative log\",\"ts\":1650602040.0}";
        let config = Config::default();
        let info = extract_info(line, &config);
        assert_eq!(info.get("msg").cloned().unwrap_or_default(), "knative log");
        assert_eq!(info.get("level").cloned().unwrap_or_default(), "DEBUG");
        assert!(info.get("ts").is_some_and(|v| !v.is_empty()));
    }

    #[test]
    fn test_do_line_level_symbols() {
        let config = Config {
            level_symbols: crate::config::LevelSymbols::Emoji,
            ..Config::default()
        };
        let line = "{\"level\":\"INFO\",\"msg\":\"symbol test\",\"timestamp\":\"2022-04-25T10:24:30.155404234Z\"}";
        let mut state = ParseState::default();
        let result = do_line(&config, line, &mut state);
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

    #[test]
    fn test_kubectl_events_parsing_and_formatting() {
        let header = "LAST SEEN   TYPE      REASON              OBJECT                                               MESSAGE";
        let event_line = "119m        Warning   Unhealthy           pod/pipelines-as-code-controller-76d86f74bb-vxjtd    Readiness probe failed: Get \"http://10.128.0.97:8082/live\": dial tcp 10.128.0.97:8082: connect: connection refused";
        let mut state = ParseState::default();
        // Simulate header detection
        assert!(crate::parse::is_kubectl_events_header(header, &mut state));
        // Parse event line
        let parsed = crate::parse::parse_kubectl_event_line(event_line, &state);
        assert!(parsed.is_some());
        let (last_seen, type_, reason, object, message) = parsed.unwrap();
        assert_eq!(last_seen, "119m");
        assert_eq!(type_, "Warning");
        assert_eq!(reason, "Unhealthy");
        assert_eq!(object, "pod/pipelines-as-code-controller-76d86f74bb-vxjtd");
        assert!(message.contains("Readiness probe failed:"));
    }

    #[test]
    fn test_kubectl_events_object_type_coloring() {
        use yansi::Paint;
        // Each type should have a distinct color
        let pod = "pod/foo";
        let replicaset = "replicaset/bar";
        let deployment = "deployment/baz";
        let service = "service/qux";
        let job = "job/quux";
        let daemonset = "daemonset/quuz";
        let statefulset = "statefulset/corge";
        let configmap = "configmap/grault";
        let secret = "secret/garply";
        let unknown = "foobar/xyz";

        let pod_colored = colorize_object_type(pod);
        let replicaset_colored = colorize_object_type(replicaset);
        let deployment_colored = colorize_object_type(deployment);
        let service_colored = colorize_object_type(service);
        let job_colored = colorize_object_type(job);
        let daemonset_colored = colorize_object_type(daemonset);
        let statefulset_colored = colorize_object_type(statefulset);
        let configmap_colored = colorize_object_type(configmap);
        let secret_colored = colorize_object_type(secret);
        let unknown_colored = colorize_object_type(unknown);

        // All should contain ANSI color codes
        for colored in [
            &pod_colored,
            &replicaset_colored,
            &deployment_colored,
            &service_colored,
            &job_colored,
            &daemonset_colored,
            &statefulset_colored,
            &configmap_colored,
            &secret_colored,
            &unknown_colored,
        ] {
            assert!(
                colored.contains("\x1b["),
                "Expected ANSI color codes in object type coloring: {colored}"
            );
        }
        // Pod and replicaset should have different colors
        assert_ne!(pod_colored, replicaset_colored);
        // Pod and pod again should have the same color prefix
        let pod2_colored = colorize_object_type("pod/bar");
        assert_eq!(
            pod_colored.split('/').next().unwrap(),
            pod2_colored.split('/').next().unwrap()
        );
    }
}
