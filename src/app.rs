use crate::config::Config;
use crate::parser::{self, ParseState};

pub fn process_raw_line(config: &Config, line: &str, state: &mut ParseState) -> Vec<String> {
    let Some(parsed) = parser::parse_line(config, line, state) else {
        return Vec::new();
    };

    if matches!(
        parsed,
        crate::model::ParsedLine::Structured(_) | crate::model::ParsedLine::Raw(_)
    ) {
        crate::pipeline::maybe_run_action(config, line);
    }

    let Some(processed) = crate::pipeline::process_line(config, parsed) else {
        return Vec::new();
    };

    crate::render::render_parsed_line(config, &processed, state)
}
