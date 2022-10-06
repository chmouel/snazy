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
use yansi::{Color, Paint, Style};

use crate::config;
use crate::config::Config;
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
    let mut msg = HashMap::new();
    let mut kail_msg_prefix = String::new();
    let mut line = rawline.to_string();

    if let Some(prefix) = parse_kail_lines(config, rawline) {
        let replacer = Regex::new(KAIL_RE).unwrap();
        line = replacer.replace_all(rawline, "$line").to_string();
        kail_msg_prefix = prefix;
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
            crate::utils::convert_str_to_ts(p.timestamp.as_str(), config.time_format.as_str()),
        );
        let mut others = String::new();
        if p.other.contains_key("provider") {
            // append provider icon to others
            others.push_str(crate::utils::convert_pac_provider_to_fa_icon(
                p.other["provider"].as_str().unwrap(),
            ));
            msg.insert("others".to_string(), format!("{} ", others));
        }
    }

    if let Ok(p) = serde_json::from_str::<Knative>(line.as_str()) {
        msg.insert("msg".to_string(), p.msg.trim().to_string());
        msg.insert("level".to_string(), p.level.to_uppercase());
        if let Some(ts) = p.other.get("ts") {
            msg.insert(
                String::from("ts"),
                crate::utils::convert_ts_float_or_str(ts, time_format),
            );
        };
    }

    if !config.kail_no_prefix && !kail_msg_prefix.is_empty() && msg.contains_key("msg") {
        *msg.get_mut("msg").unwrap() = format!("{} {}", Paint::blue(kail_msg_prefix), msg["msg"]);
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

fn custom_json_match(
    config: &Config,
    time_format: &str,
    kali_msg_prefix: &str,
    line: &str,
) -> HashMap<String, String> {
    let mut dico = HashMap::new();
    if let Ok(p) = serde_json::from_str::<Value>(line) {
        for (key, value) in &config.json_keys {
            if p.pointer(key).is_some() {
                // if value  equal ts or timestamp or date then parse as timestamp
                if value == "ts" || value == "timestamp" || value == "date" {
                    // make a serde json Value
                    let v = p.pointer(key).unwrap();
                    let ts = crate::utils::convert_ts_float_or_str(v, time_format);
                    dico.insert(value.to_string(), ts);
                } else {
                    let mut v = p.pointer(key).unwrap().to_string();
                    if v.contains('"') {
                        v = v.replace('"', "");
                    }

                    dico.insert(value.to_string(), v);
                }
            }
        }
    }
    if !config.kail_no_prefix && !kali_msg_prefix.is_empty() && dico.contains_key("msg") {
        *dico.get_mut("msg").unwrap() = format!("{} {}", Paint::blue(kali_msg_prefix), dico["msg"]);
    }
    dico
}

pub fn action_on_regexp(config: &Config, line: &str) {
    let reg = Regex::new(config.action_regexp.as_ref().unwrap()).unwrap();
    if let Some(reg) = reg.captures(line) {
        let regexpmatch = reg.get(0).unwrap().as_str();
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
    let unwrapped = serde_json::to_string(&msg).unwrap();
    //check if unwrapped is not an empty hashmap
    if unwrapped == "{}" {
        println!(
            "{}",
            apply_regexps(&config.regexp_colours, line.to_string())
        );
        return None;
    }

    if config
        .skip_line_regexp
        .iter()
        .map(|s| Regex::new(s).unwrap())
        .filter(|r| r.is_match(msg["msg"].as_str()))
        .count()
        > 0
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
    let mut ts = String::new();
    if msg.contains_key("ts") {
        ts = Paint::fixed(13, msg.get("ts").unwrap()).to_string();
    }
    let other = if msg.contains_key("others") {
        format!(" {}", Paint::cyan(msg.get("others").unwrap()).italic())
    } else {
        "".to_string()
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

pub fn apply_regexps(regexps: &HashMap<String, Color>, msg: String) -> String {
    let mut ret = msg;
    for (key, value) in regexps.iter() {
        let re = Regex::new(format!(r"(?P<r>{})", key.as_str()).as_str()).unwrap();
        let style = Style::new(*value);
        ret = re
            .replace_all(&ret, style.paint("$r").to_string())
            .to_string();
    }
    ret
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
    let file = File::open(&filename).map_err(|e| {
        eprintln!("file {}, {}", filename, e);
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