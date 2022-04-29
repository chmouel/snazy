use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
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

struct Info {
    level: String,
    msg: String,
    timestamp: String,
    others: String,
}

pub fn extract_info(rawline: &str, config: &Config) -> Option<HashMap<String, String>> {
    let time_format = config.time_format.as_str();
    let mut msg = HashMap::new();

    let mut line = String::from(rawline);
    let kali_re =
        Regex::new(r"^(?P<namespace>[^/]*)/(?P<pod>[^\[]*)\[(?P<container>[^]]*)]: (?P<line>.*)")
            .unwrap();
    let mut kali_msg_prefix = String::new();
    if kali_re.is_match(line.as_str()) {
        line = kali_re.replace_all(rawline, "$line").to_string();
        kali_msg_prefix = kali_re
            .replace_all(rawline, "$namespace/$pod[$container]")
            .to_string();
    }

    if !config.json_keys.is_empty() {
        if let Ok(p) = serde_json::from_str::<Generic>(line.as_str()) {
            for (key, value) in p.other {
                if config.json_keys.contains_key(key.as_str()) {
                    if config.json_keys[key.as_str()].as_str() == "ts" {
                        msg.insert(
                            String::from("ts"),
                            crate::utils::conver_ts_float_or_str(&value, time_format),
                        );
                    } else {
                        msg.insert(
                            config.json_keys[key.as_str()].clone(),
                            value.as_str().unwrap().to_string(),
                        );
                    }
                }
            }
            if !config.kail_no_prefix && !kali_msg_prefix.is_empty() && msg.contains_key("msg") {
                *msg.get_mut("msg").unwrap() =
                    format!("{} {}", Paint::blue(kali_msg_prefix), msg["msg"])
            }
            return Some(msg);
        }
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
        *msg.get_mut("msg").unwrap() = format!("{} {}", Paint::blue(kali_msg_prefix), msg["msg"])
    }
    Some(msg)
}

fn parse_line(config: Arc<Config>, line: &str) -> Option<Info> {
    // exclude lines with only space or empty
    if line.trim().is_empty() {
        return None;
    }

    if let Some(msg) = extract_info(line, &config) {
        let unwrapped = serde_json::to_string(&msg).unwrap();
        //check if unwrapped is not an empty hashmap
        if unwrapped == "{}" {
            println!("{}", line);
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
            themsg = apply_regexps(&config.regexp_colours, themsg)
        }
        Some(Info {
            level,
            timestamp: ts,
            others: other,
            msg: themsg,
        })
    } else {
        None
    }
}

fn apply_regexps(regexps: &HashMap<String, Color>, msg: String) -> String {
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

pub fn read_from_stdin(config: Arc<Config>) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let parseline = &line.unwrap();

        if let Some(info) = parse_line(config.clone(), parseline) {
            println!(
                "{} {} {}{}",
                info.level, info.timestamp, info.others, info.msg
            );
        }
    }
}

pub fn read_from_files(config: Arc<Config>) {
    for f in &config.files {
        // open file and parse each lines
        let file = File::open(f).map_err(|e| {
            eprintln!("file {}, {}", f, e);
            std::process::exit(1);
        });
        let buf_reader = BufReader::new(file.unwrap());
        for line in buf_reader.lines() {
            let parseline = &line.unwrap();

            if let Some(info) = parse_line(config.clone(), parseline) {
                println!(
                    "{} {} {}{}",
                    info.level, info.timestamp, info.others, info.msg
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use regex::Regex;
    use std::collections::HashMap;
    use yansi::Color;

    #[test]
    fn test_get_line() {
        let line = r#"{"severity":"INFO","timestamp":"2022-04-25T10:24:30.155404234Z","logger":"pipelinesascode","caller":"kubeinteraction/secrets.go:114","message":"hello moto"}"#;
        let msg = super::extract_info(
            line,
            &Config {
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(msg["msg"], "hello moto");
    }

    #[test]
    fn test_kail_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":"updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = super::extract_info(
            line,
            &Config {
                kail_no_prefix: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(msg["msg"].contains("ns/pod[container]"));
        assert!(msg["msg"].contains("updated"));
    }

    #[test]
    fn test_kail_no_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = super::extract_info(
            line,
            &Config {
                kail_no_prefix: true,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(msg["msg"], "updated");
    }

    #[test]
    fn test_pac_provider_icon() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" github","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = super::extract_info(
            line,
            &Config {
                kail_no_prefix: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(msg.contains_key("others"));
        assert!(msg["others"].contains(" "));
    }
    #[test]
    fn test_apply_regexps() {
        let line = String::from("red blue normal");
        // define a regexp
        let regexp = Regex::new(r"\b(b.ue)\b").unwrap();
        let mut map = HashMap::new();
        map.insert(String::from("red"), Color::Red);
        map.insert(regexp.to_string(), Color::Blue);
        let ret = super::apply_regexps(&map, line);
        assert_eq!(
            ret,
            format!(
                "{} {} normal",
                Color::Red.paint("red"),
                Color::Blue.paint("blue")
            )
        )
    }
}
