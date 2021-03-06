use std::fs;

use clap_complete::{generate_to, Shell};
use Shell::*;

include!("src/cli.rs");

fn main() {
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("misc/")
        .join("completions/");
    fs::create_dir_all(&outdir).expect("cannot create directory");

    let mut app = build();
    for shell in [Bash, Zsh, PowerShell, Fish, Elvish] {
        generate_to(shell, &mut app, "snazy", &outdir).expect("cannot generate completions");
    }
}
