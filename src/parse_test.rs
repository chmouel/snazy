#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use regex::Regex;
    use yansi::Color;

    use crate::config::Config;
    use crate::parse::extract_info;

    #[test]
    fn test_get_line() {
        let line = r#"{"severity":"INFO","timestamp":"2022-04-25T10:24:30.155404234Z","logger":"pipelinesascode","caller":"kubeinteraction/secrets.go:114","message":"hello moto"}"#;
        let msg = extract_info(
            line,
            &Config {
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            msg["msg"],
            "\u{1b}[34m{namespace}/{pod}[{container}]\u{1b}[0m hello moto"
        );
    }

    #[test]
    fn test_kail_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":"updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = extract_info(
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
        let msg = extract_info(
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
        let msg = extract_info(
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
        let ret = crate::parse::apply_regexps(&map, line);
        assert_eq!(
            ret,
            format!(
                "{} {} normal",
                Color::Red.paint("red"),
                Color::Blue.paint("blue")
            )
        )
    }

    #[test]
    fn test_config_json_keys() {
        let mut keys = HashMap::new();
        keys.insert(String::from("foo"), String::from("msg"));
        keys.insert(String::from("bar"), String::from("level"));

        let config = Config {
            json_keys: keys,
            ..Default::default()
        };
        let line = r#"{"foo": "Bar", "bar": "info"}"#;
        let info = extract_info(line, &config).unwrap();
        assert_eq!(
            info.get("msg").unwrap(),
            "\u{1b}[34m{namespace}/{pod}[{container}]\u{1b}[0m Bar"
        );
        assert_eq!(info.get("level").unwrap(), "info")
    }

    #[test]
    fn test_config_json_timestamp_float() {
        let mut keys = HashMap::new();
        keys.insert(String::from("bar"), String::from("ts"));

        let config = Config {
            json_keys: keys,
            ..Default::default()
        };
        let line = r#"{"bar": 1650602040.6289625}"#;
        let info = extract_info(line, &config).unwrap();
        assert_eq!(info.get("ts").unwrap(), "04:34:00")
    }
}
