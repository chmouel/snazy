use std::fmt::Write as _;

use crate::config::Config;
use crate::model::{KubectlEvent, ParsedLine, RenderedLog, StructuredLog};
use crate::utils::apply_regexps;
use yansi::Paint;

pub fn render_parsed_line(config: &Config, parsed: &ParsedLine) -> Vec<String> {
    match parsed {
        ParsedLine::Structured(log) => render_structured_lines(config, log),
        ParsedLine::Raw(line) => vec![apply_regexps(&config.regexp_colours, line.clone())],
        ParsedLine::KubectlHeader => vec![format!(
            "{} {} {} {} {}",
            "LAST SEEN".bold(),
            "TYPE".bold(),
            "REASON".bold(),
            "OBJECT".bold(),
            "MESSAGE".bold()
        )],
        ParsedLine::KubectlEvent(event) => vec![render_kubectl_event(config, event)],
    }
}

pub fn render_structured_log(config: &Config, log: &StructuredLog) -> RenderedLog {
    let level = match config.level_symbols {
        crate::config::LevelSymbols::Emoji => crate::utils::level_symbols(&log.level),
        crate::config::LevelSymbols::Text => crate::utils::color_by_level(&log.level),
    };
    let timestamp = log
        .timestamp
        .as_ref()
        .map_or_else(String::new, |ts| ts.fixed(13).to_string());
    let others = log
        .others
        .as_ref()
        .map_or_else(String::new, |others| match config.coloring {
            crate::config::Coloring::Never => format!(" {others}"),
            _ => format!(" {}", Paint::cyan(others).italic()),
        });

    let mut message = prefixed_message(config, log);
    if !config.regexp_colours.is_empty() {
        message = apply_regexps(&config.regexp_colours, message);
    }

    let extras = render_extra_fields(config, &log.extra_fields);
    if !extras.is_empty() {
        message.push_str(&extras);
    }

    RenderedLog {
        level,
        timestamp,
        others,
        message,
        stacktrace: log.stacktrace.clone(),
    }
}

pub fn render_structured_lines(config: &Config, log: &StructuredLog) -> Vec<String> {
    let rendered = render_structured_log(config, log);
    let mut lines = vec![format!(
        "{} {} {}{}",
        rendered.level, rendered.timestamp, rendered.others, rendered.message
    )];

    if !config.hide_stacktrace {
        if let Some(stacktrace) = rendered.stacktrace.as_ref() {
            lines.extend(render_stacktrace_block(
                stacktrace,
                config.coloring == crate::config::Coloring::Never,
            ));
        }
    }

    lines
}

pub fn render_stacktrace_block(stacktrace: &str, disable_coloring: bool) -> Vec<String> {
    let mut lines = vec![
        format!("\n{}", "─".repeat(80).fixed(8)),
        " Stacktrace:".red().bold().to_string(),
    ];

    for stack_line in stacktrace.lines() {
        lines.push(format!(
            "   {}",
            format_stack_line(stack_line, disable_coloring)
        ));
    }

    lines.push(format!("{}\n", "─".repeat(80).fixed(8)));
    lines
}

pub fn render_kubectl_event(config: &Config, event: &KubectlEvent) -> String {
    const LAST_SEEN_WIDTH: usize = 8;
    const TYPE_WIDTH: usize = 8;
    const REASON_WIDTH: usize = 18;

    let last_seen_padded = format!("{:LAST_SEEN_WIDTH$}", event.last_seen);
    let type_padded = format!("{:TYPE_WIDTH$}", event.type_);
    let reason_padded = format!("{:REASON_WIDTH$}", event.reason);
    let object_padded = format!("{:52}", event.object);

    let type_colored = match event.type_.as_str() {
        "Warning" => Paint::red(&type_padded).bold(),
        "Normal" => Paint::green(&type_padded).bold(),
        _ => Paint::new(&type_padded).bold(),
    };
    let reason_colored = Paint::yellow(&reason_padded).bold();
    let object_colored = colorize_object_type(&object_padded);
    let last_seen_colored = Paint::new(&last_seen_padded).bold();
    let message = if config.regexp_colours.is_empty() {
        event.message.clone()
    } else {
        apply_regexps(&config.regexp_colours, event.message.clone())
    };

    format!("{last_seen_colored} {type_colored} {reason_colored} {object_colored} {message}")
}

pub fn colorize_object_type(object: &str) -> String {
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
        "statefulset" => Paint::new(prefix).fg(Color::Fixed(93)).bold(),
        "configmap" => Paint::new(prefix).fg(Color::Fixed(208)).bold(),
        "secret" => Paint::new(prefix).fg(Color::Fixed(244)).bold(),
        _ => Paint::white(prefix).bold(),
    };
    format!("{colored_prefix}{rest}")
}

pub fn format_stack_line(line: &str, disable_coloring: bool) -> String {
    if line.contains(".go:")
        || line.contains(".rs:")
        || line.contains(".js:")
        || line.contains(".py:")
        || line.contains(".cpp:")
        || line.contains(".ts:")
        || line.contains(".rb:")
    {
        if let Some(last_slash_pos) = line.rfind('/') {
            let path = &line[0..=last_slash_pos];
            let rest = &line[last_slash_pos + 1..];

            if let Some(colon_pos) = rest.find(':') {
                let filename = &rest[0..colon_pos];
                let line_num = &rest[colon_pos + 1..];

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

    if disable_coloring {
        line.to_string()
    } else {
        line.fixed(15).to_string()
    }
}

fn prefixed_message(config: &Config, log: &StructuredLog) -> String {
    if log.kail_prefix.is_none() || config.kail_prefix == crate::config::KailPrefix::Hide {
        return log.message.clone();
    }

    let prefix = log.kail_prefix.as_ref().unwrap();
    match config.coloring {
        crate::config::Coloring::Never => format!("{prefix} {}", log.message),
        _ => format!("{} {}", Paint::blue(prefix), log.message),
    }
}

fn render_extra_fields(config: &Config, extra_fields: &[(String, String)]) -> String {
    let mut rendered = String::new();
    for (key, value) in extra_fields {
        let key_str = match config.coloring {
            crate::config::Coloring::Never => key.clone(),
            _ => Paint::new(key).bold().to_string(),
        };
        let _ = write!(rendered, " {key_str}={value}");
    }
    rendered
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::model::StructuredLog;

    #[test]
    fn render_structured_log_appends_extra_fields() {
        let rendered = super::render_structured_log(
            &Config {
                coloring: crate::config::Coloring::Never,
                ..Config::default()
            },
            &StructuredLog {
                level: "INFO".to_string(),
                message: "hello".to_string(),
                timestamp: Some("10:00:00".to_string()),
                others: None,
                consumed_fields: Vec::new(),
                extra_fields: vec![("status".to_string(), "200".to_string())],
                stacktrace: None,
                raw_json: None,
                kail_prefix: None,
            },
        );

        assert_eq!(rendered.message, "hello status=200");
    }

    #[test]
    fn format_stack_line_respects_coloring_toggle() {
        let line = "/foo/bar.rs:42";
        assert!(super::format_stack_line(line, false).contains("\x1b["));
        assert_eq!(super::format_stack_line(line, true), "/foo/bar.rs:42");
    }

    #[test]
    fn colorize_object_type_assigns_color_codes() {
        let pod = super::colorize_object_type("pod/foo");
        let deployment = super::colorize_object_type("deployment/bar");
        assert!(pod.contains("\x1b["));
        assert!(deployment.contains("\x1b["));
        assert_ne!(pod, deployment);
    }
}
