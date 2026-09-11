//! Generates the documentation and the library's parameter tables from `spec/`.
//!
//! ```text
//! cargo xtask docs               regenerate the documentation
//! cargo xtask docs --check       verify it, without writing
//! cargo xtask codegen            regenerate the library's parameter tables
//! cargo xtask codegen --check    verify them, without writing
//! ```
//!
//! `spec/` is the single source of truth for the protocol. It renders two ways:
//! into documentation for a person, and into Rust tables for the library, which
//! cannot read TOML on the targets it runs on. Both renderings are checked in,
//! and both have a `--check` mode and a test that fails when the checked-in copy
//! has fallen behind.
//!
//! Everything else this repository needs is a shell one-liner and lives where it
//! runs: the version guard in `.github/workflows/ci.yml`, the release in
//! `.github/workflows/release.yml`.

mod codegen;
mod diagrams;
mod docs;
mod fx;
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
        "codegen" => codegen_command(flags.iter().any(|f| f == "--check")),
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
    println!("  docs [--check]  regenerate docs/midi-spec.md, docs/effects.md and");
    println!("                  docs/diagrams/ from spec/, or report whether they are");
    println!("                  current without writing");
    println!("  codegen [--check]  regenerate the library's parameter tables from spec/,");
    println!("                  or report whether they are current without writing");
    println!("  help            show this message");
}

/// Returns the repository root, which is the parent of this crate's directory.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn codegen_command(check: bool) -> Result<(), String> {
    match codegen::run(&root(), check)? {
        docs::Outcome::Current => {
            println!("{} is up to date", codegen::CODE_PATH);
            Ok(())
        }
        docs::Outcome::Stale if check => Err(format!(
            "{} is out of date with spec/. Run `cargo xtask codegen` and commit the result.",
            codegen::CODE_PATH
        )),
        docs::Outcome::Stale => {
            println!("regenerated {} from spec/", codegen::CODE_PATH);
            Ok(())
        }
    }
}

fn docs_command(check: bool) -> Result<(), String> {
    match docs::run(&root(), check)? {
        docs::Outcome::Current => {
            println!(
                "{} and {} are up to date",
                docs::DOC_PATH,
                docs::EFFECTS_PATH
            );
            Ok(())
        }
        docs::Outcome::Stale if check => Err(
            "docs/ is out of date with spec/. Run `cargo xtask docs` and commit the result."
                .to_owned(),
        ),
        docs::Outcome::Stale => {
            println!("regenerated docs/ from spec/");
            Ok(())
        }
    }
}
