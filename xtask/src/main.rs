//! Generates the documentation and the library's parameter tables from `spec/`.
//!
//! ```text
//! cargo xtask docs               regenerate the documentation
//! cargo xtask docs --check       verify it, without writing
//! cargo xtask codegen            regenerate the library's parameter tables
//! cargo xtask codegen --check    verify them, without writing
//! ```
//!
//! `spec/` is the source of truth for the protocol. It renders two ways: into
//! documentation for a person, and into Rust tables for the library, which
//! cannot read TOML on the targets it runs on. Both renderings are checked in,
//! and both have a `--check` mode and a test that fails when the checked-in copy
//! has fallen behind.
//!
//! Everything else the repository automates is a shell one-liner and lives
//! where it runs: the version guard in `.github/workflows/ci.yml`, the release
//! in `.github/workflows/release.yml`.

mod codegen;
mod diagrams;
mod docs;
mod effect;
mod front;
mod fx;
mod glyph;
mod output;
mod program;
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
        "docs" | "codegen" => check_flag(command, flags).and_then(|check| {
            let run = if command == "docs" {
                docs::run
            } else {
                codegen::run
            };
            report(command, &run(&root(), check)?, check)
        }),
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
    println!("usage: cargo xtask <command> [--check]");
    println!();
    println!("commands:");
    println!("  docs [--check]     regenerate docs/midi-spec.md, docs/effects.md and");
    println!("                     docs/diagrams/ from spec/, or report whether they");
    println!("                     are current without writing");
    println!("  codegen [--check]  regenerate the library's generated sources from");
    println!("                     spec/, or report whether they are current");
    println!("  help               show this message");
}

/// Returns whether `--check` was given, and rejects any other flag.
fn check_flag(command: &str, flags: &[String]) -> Result<bool, String> {
    let mut check = false;
    for flag in flags {
        if flag == "--check" {
            check = true;
        } else {
            return Err(format!(
                "unknown flag for {command}: {flag}. The only flag is --check"
            ));
        }
    }
    Ok(check)
}

/// Prints what a run found: nothing stale, or the files it rewrote or would.
///
/// A stale file in check mode is an error, so that CI fails on it.
fn report(command: &str, stale: &[String], check: bool) -> Result<(), String> {
    let what = match command {
        "docs" => format!(
            "{}, {} and docs/diagrams/",
            docs::DOC_PATH,
            docs::EFFECTS_PATH
        ),
        _ => codegen::CODE_PATHS.join(" and "),
    };
    if stale.is_empty() {
        println!("{what} are up to date");
        return Ok(());
    }
    let listed = stale.join("\n  ");
    if check {
        return Err(format!(
            "out of date with spec/:\n  {listed}\nRun `cargo xtask {command}` and commit the result."
        ));
    }
    println!("regenerated from spec/:\n  {listed}");
    Ok(())
}

/// Returns the repository root, which is the parent of this crate's directory.
pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}
