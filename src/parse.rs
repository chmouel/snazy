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
    }

    if let Ok(p) = serde_json::from_str::<Knative>(line.as_str()) {
        msg.insert("msg".to_string(), p.msg.trim().to_string());
        msg.insert("level".to_string(), p.level.to_uppercase());
        if let Some(ts) = p.other.get("ts") {
            msg.insert(
                String::from("ts"),
                crate::utils::convert_ts_float_or_str(ts, time_format, timezone),
            );
        };
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
    Some(Info {
        level,
        timestamp: ts,
        others: other,
        msg: themsg,
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
