use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;

use crate::config::Config;
use crate::parser::ParseState;

pub fn read_from_stdin(config: &Arc<Config>) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = io::BufWriter::new(stdout.lock());
    process_reader(config, stdin.lock(), &mut stdout);
}

pub fn read_from_files(config: &Arc<Config>) {
    for filename in config.files.as_ref().unwrap() {
        let stdout = io::stdout();
        let mut stdout = io::BufWriter::new(stdout.lock());
        read_a_file(config, filename, &mut stdout);
    }
}

pub fn read_a_file(config: &Config, filename: &str, writeto: &mut dyn Write) {
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("file {filename}, {error}");
            return;
        }
    };

    process_reader(config, BufReader::new(file), writeto);
}

pub fn process_reader(config: &Config, reader: impl BufRead, writeto: &mut dyn Write) {
    let mut state = ParseState::default();
    let flush_live_output = config.files.is_none();
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };

        for rendered in crate::app::process_raw_line(config, &line, &mut state) {
            writeln!(writeto, "{rendered}").unwrap();
            if flush_live_output {
                writeto.flush().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write as _};

    use crate::config::Config;

    #[test]
    fn read_a_file_formats_logs() {
        let mut file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        let line = r#"{"level":"INFO","ts":"2022-04-25T14:20:32.505637358Z", "msg":"hello world"}
{"level":"DEBUG","ts":"2022-04-25T14:20:32.505637358Z", "msg":"debug"}"#;
        file.write_all(line.as_bytes())
            .expect("Failed to write to temp file");

        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            ..Config::default()
        };
        let mut output = Vec::new();
        super::read_a_file(&config, file_path.to_str().unwrap(), &mut output);
        file.close().unwrap();

        let output = std::str::from_utf8(&output).expect("Failed to convert output to utf8");
        let re_info = regex::Regex::new(r"INFO.*14:20:32.*hello world").unwrap();
        let re_debug = regex::Regex::new(r"DEBUG.*14:20:32.*debug").unwrap();
        assert!(re_info.is_match(output));
        assert!(re_debug.is_match(output));
    }

    #[test]
    fn stacktrace_output_can_be_hidden() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let file_path = file.path().to_path_buf();
        let stacktrace = r"goroutine 1 [running]:
github.com/example/app.Function1(0x123456)
        /home/user/app.go:42 +0x2a";
        let line = format!(
            r#"{{"level":"ERROR","ts":"2022-04-25T14:20:32.505637358Z","msg":"Error occurred","stacktrace":{stacktrace:?}}}"#
        );
        file.write_all(line.as_bytes()).unwrap();

        let visible = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            hide_stacktrace: false,
            ..Config::default()
        };
        let mut visible_output = Vec::new();
        super::read_a_file(&visible, file_path.to_str().unwrap(), &mut visible_output);
        let visible_output = std::str::from_utf8(&visible_output).unwrap();
        assert!(visible_output.contains("Stacktrace"));
        assert!(visible_output.contains("app.go"));

        let hidden = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            hide_stacktrace: true,
            ..Config::default()
        };
        let mut hidden_output = Vec::new();
        super::read_a_file(&hidden, file_path.to_str().unwrap(), &mut hidden_output);
        file.close().unwrap();

        let hidden_output = std::str::from_utf8(&hidden_output).unwrap();
        assert!(!hidden_output.contains("Stacktrace"));
        assert!(!hidden_output.contains("app.go"));
        assert!(hidden_output.contains("Error occurred"));
    }

    #[test]
    fn read_a_file_handles_missing_and_empty_files() {
        let missing_path = "/tmp/this_file_should_not_exist_snazy_test";
        let config = Config {
            files: Some(vec![missing_path.to_string()]),
            ..Config::default()
        };
        let mut missing_output = Vec::new();
        super::read_a_file(&config, missing_path, &mut missing_output);
        assert!(missing_output.is_empty());

        let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        let config = Config {
            files: Some(vec![file_path.to_str().unwrap().to_string()]),
            ..Config::default()
        };
        let mut empty_output = Vec::new();
        super::read_a_file(&config, file_path.to_str().unwrap(), &mut empty_output);
        file.close().unwrap();
        assert!(empty_output.is_empty());
    }

    struct FlushTrackingWriter {
        output: Vec<u8>,
        flush_count: usize,
    }

    impl std::io::Write for FlushTrackingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.flush_count += 1;
            Ok(())
        }
    }

    #[test]
    fn process_reader_flushes_each_rendered_line_for_stdin() {
        let input = Cursor::new(
            br#"{"level":"INFO","ts":"2022-04-25T14:20:32.505637358Z","msg":"hello world"}"#,
        );
        let config = Config {
            files: None,
            ..Config::default()
        };
        let mut output = FlushTrackingWriter {
            output: Vec::new(),
            flush_count: 0,
        };

        super::process_reader(&config, input, &mut output);

        let rendered = std::str::from_utf8(&output.output).unwrap();
        assert!(rendered.contains("hello world"));
        assert_eq!(output.flush_count, 1);
    }
}
