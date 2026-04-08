use crate::config::Config;
use crate::model::{KubectlEvent, ParsedLine, StructuredLog};
use crate::parser::{self, ParseState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputEvent {
    pub collapse_key: Option<String>,
    pub rendered_lines: Vec<String>,
}

pub fn process_raw_line(
    config: &Config,
    line: &str,
    state: &mut ParseState,
) -> Option<OutputEvent> {
    let parsed = parser::parse_line(config, line, state)?;

    if matches!(
        parsed,
        crate::model::ParsedLine::Structured(_) | crate::model::ParsedLine::Raw(_)
    ) {
        crate::pipeline::maybe_run_action(config, line);
    }

    let processed = crate::pipeline::process_line(config, parsed)?;

    Some(OutputEvent {
        collapse_key: collapse_key(&processed),
        rendered_lines: crate::render::render_parsed_line(config, &processed, state),
    })
}

fn collapse_key(parsed: &ParsedLine) -> Option<String> {
    match parsed {
        ParsedLine::Structured(log) => Some(structured_log_key(log)),
        ParsedLine::Raw(line) => Some(format!("raw\0{line}")),
        ParsedLine::KubectlHeader => None,
        ParsedLine::KubectlEvent(event) => Some(kubectl_event_key(event)),
    }
}

fn structured_log_key(log: &StructuredLog) -> String {
    format!(
        "structured\0{}\0{}\0{}\0{}\0{:?}\0{}",
        log.level,
        log.kail_prefix.as_deref().unwrap_or_default(),
        log.others.as_deref().unwrap_or_default(),
        log.message,
        log.extra_fields,
        log.stacktrace.as_deref().unwrap_or_default()
    )
}

fn kubectl_event_key(event: &KubectlEvent) -> String {
    format!(
        "kubectl\0{}\0{}\0{}\0{}\0{}",
        event.last_seen, event.type_, event.reason, event.object, event.message
    )
}
