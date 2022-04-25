use std::fs;

use clap_complete::{generate_to, Shell};
use Shell::*;

include!("src/app.rs");

fn main() {
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("misc/")
        .join("completions/");
    fs::create_dir_all(&outdir).unwrap();

    let mut app = build_app();
    for shell in [Bash, Zsh, PowerShell, Fish, Elvish] {
        generate_to(shell, &mut app, "snazy", &outdir).unwrap();
    }
}
