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

    if !config.kail_no_prefix && !kail_msg_prefix.is_empty() && msg.contains_key("msg") {
        *msg.get_mut("msg").unwrap() = format!("{} {}", kail_msg_prefix.blue(), msg["msg"]);
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
        if Command::new("sh")
            .arg("-c")
            .arg(action_command)
            .spawn()
            .is_ok()
        {
            println!("Spawned command: for action: {}", Paint::cyan(regexpmatch));
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

    if config
        .skip_line_regexp
        .iter()
        .any(|s| Regex::new(s).unwrap().is_match(&msg["msg"]))
    {
        return None;
    }

    if !config.filter_levels.is_empty()
        && !config
            .filter_levels
            .contains(config::level_from_str(&msg["level"].to_lowercase()))
    {
        return None;
    }

    let mut level = crate::utils::color_by_level(msg.get("level").unwrap());
    if config.level_symbols {
        level = crate::utils::level_symbols(msg.get("level").unwrap());
    }
    let ts = if let Some(ts) = msg.get("ts") {
        ts.fixed(13).to_string()
    } else {
        String::new()
    };
    let other = if let Some(o) = msg.get("others") {
        format!(" {}", Paint::cyan(o).italic())
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
                        let formatted_line = format_stack_line(stack_line);
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
    let file = File::open(filename).map_err(|e| {
        eprintln!("file {filename}, {e}");
        std::process::exit(1);
    });
    let buf_reader = BufReader::new(file.unwrap());
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
                        let formatted_line = format_stack_line(stack_line);
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
fn format_stack_line(line: &str) -> String {
    // Check if this is a file path line with line numbers (contains a colon with numbers after it)
    if line.contains(".go:")
        || line.contains(".rs:")
        || line.contains(".js:")
        || line.contains(".py:")
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
                    path.fixed(15),                 // Path in dim gray
                    filename.yellow().bold(),       // Filename in bright yellow
                    format!(":{line_num}").green()  // Line number in green
                );
            }

            // If no colon found, just color the filename
            return format!("{}{}", path.fixed(15), rest.yellow().bold());
        }

        // If no slash found, check for a colon to split filename and line number
        if let Some(colon_pos) = line.find(':') {
            let filename = &line[0..colon_pos];
            let line_num = &line[colon_pos + 1..];

            return format!(
                "{}{}",
                filename.yellow().bold(),
                format!(":{line_num}").green()
            );
        }
    }

    // Format function name lines
    if let Some(dot_pos) = line.rfind('.') {
        let package_path = &line[0..=dot_pos];
        let func_name = &line[dot_pos + 1..];

        return format!(
            "{}{}",
            package_path.fixed(15),  // Package path in dim gray
            func_name.cyan().bold()  // Function name in cyan
        );
    }

    // Default formatting if no patterns matched
    line.fixed(15).to_string()
}
