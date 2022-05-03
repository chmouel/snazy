use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::Command;
use std::sync::Arc;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use yansi::{Color, Paint, Style};

use crate::config::Config;

// 0.6.5

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

#[derive(Serialize, Deserialize, Debug)]
struct Generic {
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

    let mut line = String::from(rawline);
    let kali_re =
        Regex::new(r"^(?P<namespace>[^/]*)/(?P<pod>[^\[]*)\[(?P<container>[^]]*)]: (?P<line>.*)")
            .unwrap();
    let mut kali_msg_prefix = config.kail_prefix_format.clone();
    if kali_re.is_match(line.as_str()) {
        line = kali_re.replace_all(rawline, "$line").to_string();
        let capture = kali_re.captures(rawline).unwrap();
        let namespace = capture.name("namespace").unwrap().as_str();
        let pod = capture.name("pod").unwrap().as_str();
        let container = capture.name("container").unwrap().as_str();
        kali_msg_prefix = kali_msg_prefix
            .replace("{namespace}", namespace)
            .replace("{pod}", pod)
            .replace("{container}", container);
    } else {
        kali_msg_prefix = String::new();
    }

    if !config.json_keys.is_empty() {
        msg = custom_json_match(config, time_format, kali_msg_prefix.as_str(), line.as_str())
            .unwrap();
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
            others.push_str(
                &(match p.other["provider"].as_str() {
                    Some("github") => " ".to_string(),
                    Some("gitlab") => " ".to_string(),
                    Some("bitbucket-cloud") => " ".to_string(),
                    Some("bitbucket-server") => " Server".to_string(),
                    _ => p.other["provider"].to_string(),
                }),
            );
            msg.insert("others".to_string(), format!("{} ", others));
        }
    }

    if let Ok(p) = serde_json::from_str::<Knative>(line.as_str()) {
        msg.insert("msg".to_string(), p.msg.trim().to_string());
        msg.insert("level".to_string(), p.level.to_uppercase());
        if let Some(ts) = p.other.get("ts") {
            msg.insert(
                String::from("ts"),
                crate::utils::conver_ts_float_or_str(ts, time_format),
            );
        };
    }

    if !config.kail_no_prefix && !kali_msg_prefix.is_empty() && msg.contains_key("msg") {
        *msg.get_mut("msg").unwrap() = format!("{} {}", Paint::blue(kali_msg_prefix), msg["msg"]);
    }
    msg
}

fn custom_json_match(
    config: &Config,
    time_format: &str,
    kali_msg_prefix: &str,
    line: &str,
) -> Option<HashMap<String, String>> {
    let mut dico = HashMap::new();
    if let Ok(p) = serde_json::from_str::<Generic>(line) {
        for (key, value) in p.other {
            if config.json_keys.contains_key(key.as_str()) {
                match config.json_keys[key.as_str()].as_str() {
                    "ts" => {
                        dico.insert(
                            String::from("ts"),
                            crate::utils::conver_ts_float_or_str(&value, time_format),
                        );
                    }
                    _ => {
                        dico.insert(
                            config.json_keys[key.as_str()].clone(),
                            value.as_str().unwrap().to_string(),
                        );
                    }
                }
            }
        }
        if !config.kail_no_prefix && !kali_msg_prefix.is_empty() && dico.contains_key("msg") {
            *dico.get_mut("msg").unwrap() =
                format!("{} {}", Paint::blue(kali_msg_prefix), dico["msg"]);
        }
        return Some(dico);
    }
    None
}

pub fn action_on_regexp(config: &Config, line: &str) {
    if config.action_regexp.is_empty() || config.action_command.is_empty() {
        return;
    }
    let action_regexp = Regex::new(config.action_regexp.as_str()).unwrap();
    if let Some(reg) = action_regexp.captures(line) {
        let regexpmatch = reg.get(0).unwrap().as_str();
        // replace {} by the actual match
        let action_command = config.action_command.replace("{}", regexpmatch);
        if Command::new("sh")
            .arg("-c")
            .arg(action_command)
            .spawn()
            .is_ok()
        {
            println!(
                "Spawned command: {} for action: {}",
                Paint::yellow(&config.action_command),
                Paint::cyan(action_regexp)
            );
        }
    }
}

fn do_line(config: &Arc<Config>, line: &str) -> Option<Info> {
    // exclude lines with only space or empty
    if line.trim().is_empty() {
        return None;
    }

    action_on_regexp(config, line);

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

    if !config.filter_levels.is_empty()
        && !config.filter_levels.contains(&msg["level"].to_lowercase())
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

pub fn read_from_files(config: &Arc<Config>) {
    for f in &config.files {
        // open file and parse each lines
        let file = File::open(f).map_err(|e| {
            eprintln!("file {}, {}", f, e);
            std::process::exit(1);
        });
        let buf_reader = BufReader::new(file.unwrap());
        for line in buf_reader.lines() {
            let parseline = &line.unwrap();

            if let Some(info) = do_line(config, parseline) {
                println!(
                    "{} {} {}{}",
                    info.level, info.timestamp, info.others, info.msg
                );
            }
        }
    }
}
