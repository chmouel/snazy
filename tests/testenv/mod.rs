use std::{env, path::PathBuf, process};

// create macro that tests arguments and output
#[macro_export]
macro_rules! snazytest {
    ($fun: ident, $args:tt,  $input:literal, $expected_output:literal, $substring:expr) => {
        #[test]
        fn $fun() {
            // create a temporary file and write the input to it
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            // unset SNAZY_LEVEL_SYMBOLS env variable to avoid test failures
            let _ = std::env::remove_var("SNAZY_LEVEL_SYMBOLS");
            let _ = std::env::remove_var("SNAZY_KAIL_PREFIX_FORMAT");
            tmpfile.write_all($input.as_bytes()).unwrap();
            tmpfile.flush().unwrap();
            let filepath = tmpfile.path().to_str().unwrap().clone();
            let env = testenv::TestEnv::new();
            let mut args = Vec::new();
            // need to figure out how to make macros  works with empty args
            // so the type system recognizes the args when empty
            if !$args.is_empty() && $args.get(0).unwrap() != &"" {
                args.extend($args);
            }
            args.push(filepath);
            if $substring {
                env.assert_command_with_substr_output(&args, $expected_output);
            } else {
                env.assert_command_with_output(&args, $expected_output);
            }
            tmpfile.close().unwrap();
        }
    };
}

/// Find the *snazy* executable
pub fn find_snazy() -> PathBuf {
    let root = env::current_exe()
        .expect("tests executable")
        .parent()
        .expect("tests executable directory")
        .parent()
        .expect("snazy executable directory")
        .to_path_buf();
    root.join("snazy")
}

/// Format an error message for when *snazy* did not exit successfully.
pub fn format_exit_error(args: &[&str], output: &process::Output) -> String {
    format!(
        "`snazy {}` did not exit successfully.\nstdout:\n---\n{}---\nstderr:\n---\n{}---",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub struct TestEnv {
    pub snazy_exe: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        Self {
            snazy_exe: find_snazy(),
        }
    }

    pub fn assert_success_and_get_output(&self, args: &[&str]) -> process::Output {
        let mut cmd = process::Command::new(&self.snazy_exe);
        cmd.args(args);
        // Run *snazy*.
        let output = cmd.output().expect("snazy output");

        // Check for exit status.
        if !output.status.success() {
            panic!("{}", format_exit_error(args, &output));
        }

        output
    }

    pub fn assert_command_with_output(&self, args: &[&str], expected: &str) {
        let output = self.assert_success_and_get_output(args);
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }

    pub fn assert_command_with_substr_output(&self, args: &[&str], expectedsub: &str) {
        let output = self.assert_success_and_get_output(args);
        assert!(String::from_utf8_lossy(&output.stdout).contains(expectedsub));
    }
}
