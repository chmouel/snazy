use chrono::NaiveDateTime;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufRead};
use yansi::Paint;

#[derive(Parser, Debug)]
#[clap(about = "    a snazy json log viewer", long_about = None)]
struct Args {
    // a regexp to match against the log line
    #[clap(
        short,
        long,
        help = "highlight word in a message with a regexp",
        value_name = "REGEXP",
        hide_default_value = true,
        default_value = ""
    )]
    regexp: String,

    #[clap(long, help = "Hide container prefix when showing kail")]
    kail_no_prefix: bool,

    #[clap(
        short,
        long,
        help = "Time format",
        value_name = "FORMAT",
        default_value = "%H:%M:%S"
    )]
    time_format: String,

    #[clap(
        short,
        long,
        multiple_values = true,
        value_delimiter = ',',
        help = "filter levels separated by commas, eg: info,debug"
    )]
    filter_levels: Vec<String>,
}

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

fn convert_str_to_ts(s: &str, time_format: &str) -> String {
    let ts = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
    ts.format(time_format).to_string()
}

fn getinfo(
    rawline: &str,
    kail_no_prefix: bool,
    time_format: &str,
) -> Option<HashMap<String, String>> {
    let mut msg = HashMap::new();
    let mut sample = rawline.to_string();
    let kali_re =
        Regex::new(r"^(?P<namespace>[^/]*)/(?P<pod>[^\[]*)\[(?P<container>[^]]*)]: (?P<line>.*)")
            .unwrap();
    let mut kali_msg_prefix = String::new();
    if kali_re.is_match(rawline) {
        let _result = kali_re.replace_all(rawline, "$line").to_string();
        sample = _result;
        kali_msg_prefix = kali_re
            .replace_all(rawline, "$namespace/$pod[$container]")
            .to_string();
    }
    if let Ok(p) = serde_json::from_str::<Pac>(sample.as_str()) {
        msg.insert("msg".to_string(), p.message.trim().to_string());
        msg.insert("level".to_string(), p.severity.to_uppercase());
        // parse timestamp to a unix timestamp
        msg.insert(
            "ts".to_string(),
            convert_str_to_ts(p.timestamp.as_str(), time_format),
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

    if let Ok(p) = serde_json::from_str::<Knative>(sample.as_str()) {
        msg.insert("msg".to_string(), p.msg.trim().to_string());
        msg.insert("level".to_string(), p.level.to_uppercase());
        if let Some(ts) = p.other.get("ts") {
            if ts.is_f64() {
                let ts = NaiveDateTime::from_timestamp(ts.as_f64().unwrap() as i64, 0);
                msg.insert("ts".to_string(), ts.format("%H:%M:%S").to_string());
            } else if ts.as_str().is_some() {
                msg.insert(
                    "ts".to_string(),
                    convert_str_to_ts(ts.as_str().unwrap(), time_format),
                );
            }
        };
    }
    // TODO: no prefix
    if !kail_no_prefix && !kali_msg_prefix.is_empty() && msg.contains_key("msg") {
        *msg.get_mut("msg").unwrap() = format!("{} {}", Paint::blue(kali_msg_prefix), msg["msg"])
    }
    Some(msg)
}

fn color_by_level(level: &str) -> String {
    match level {
        "DEBUG" => format!("{:<19}", Paint::fixed(14, "DEBUG").to_string()),
        "WARNING" => format!("{:<19}", Paint::fixed(11, "WARN").to_string()),
        "ERROR" => format!("{:<18}", Paint::fixed(9, "ERROR").to_string()),
        "INFO" => format!("{:<19}", Paint::fixed(10, "INFO").to_string()),
        _ => format!("{:<19}", Paint::fixed(10, level).to_string()),
    }
}

fn main() {
    let args = Args::parse();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let parseline = &line.unwrap();
        // exclude lines with only space or empty
        if parseline.trim().is_empty() {
            continue;
        }

        if let Some(msg) = getinfo(parseline, args.kail_no_prefix, args.time_format.as_str()) {
            let unwrapped = serde_json::to_string(&msg).unwrap();
            //check if unwrapped is not an empty hashmap
            if unwrapped == "{}" {
                println!("{}", parseline);
                continue;
            }
            if !args.filter_levels.is_empty()
                && !args.filter_levels.contains(&msg["level"].to_lowercase())
            {
                continue;
            }

            let level = color_by_level(msg.get("level").unwrap());
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
            if !args.regexp.is_empty() {
                let re = Regex::new(format!(r"(?P<r>{})", args.regexp).as_str()).unwrap();
                let _result = re
                    .replace_all(&themsg, Paint::yellow("$r").to_string())
                    .to_string();
                themsg = _result;
            }

            println!("{} {} {}{}", Paint::wrapping(level), ts, other, themsg);
        }
    }
}
