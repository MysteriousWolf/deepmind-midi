//! Generates `docs/midi-spec.md` and `docs/diagrams/*.mmd` from `spec/`.
//!
//! ```text
//! cargo xtask docs            regenerate
//! cargo xtask docs --check    verify, without writing
//! ```
//!
//! `spec/` is the single source of truth for the protocol. `docs --check` and
//! the `generated_documentation_is_current` test both fail when the checked-in
//! document does not match it.
//!
//! Everything else this repository needs is a shell one-liner and lives where it
//! runs: the version guard in `.github/workflows/ci.yml`, the release in
//! `.github/workflows/release.yml`.

mod diagrams;
mod docs;
mod spec;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, flags) = args
        .split_first()
        .map_or(("help", &[][..]), |(head, tail)| (head.as_str(), tail));

    let result = match command {
        "help" | "--help" | "-h" => {
            usage();
            return ExitCode::SUCCESS;
        }
        "docs" => docs_command(flags.iter().any(|f| f == "--check")),
        other => Err(format!("unknown command: {other}")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    println!("usage: cargo xtask <command>");
    println!();
    println!("commands:");
    println!("  docs [--check]  regenerate docs/midi-spec.md and docs/diagrams/*.mmd");
    println!("                  from spec/, or report whether they are current without");
    println!("                  writing");
    println!("  help            show this message");
}

/// Returns the repository root, which is the parent of this crate's directory.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn docs_command(check: bool) -> Result<(), String> {
    match docs::run(&root(), check)? {
        docs::Outcome::Current => {
            println!("{} is up to date", docs::DOC_PATH);
            Ok(())
        }
        docs::Outcome::Stale if check => Err(format!(
            "{} is out of date with spec/. Run `cargo xtask docs` and commit the result.",
            docs::DOC_PATH
        )),
        docs::Outcome::Stale => {
            println!("regenerated {}", docs::DOC_PATH);
            Ok(())
        }
    }
}
