//! Repository automation.
//!
//! Run with `cargo xtask <command>`.
//!
//! - `docs --check` verifies that generated documentation matches its source
//!   data. CI runs it on every pull request so generated files cannot drift.
//! - `docs` regenerates that documentation in place.
//!
//! The parameter table that drives generation lands with the `param` module, so
//! today both commands have nothing to generate and succeed trivially.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, flags) = args
        .split_first()
        .map_or(("help", &[][..]), |(head, tail)| (head.as_str(), tail));

    match command {
        "help" | "--help" | "-h" => {
            usage();
            ExitCode::SUCCESS
        }
        "docs" => docs(flags.iter().any(|flag| flag == "--check")),
        other => {
            eprintln!("unknown command: {other}\n");
            usage();
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    println!("usage: cargo xtask <command>");
    println!();
    println!("commands:");
    println!("  docs [--check]   regenerate generated documentation, or verify it is current");
    println!("  help             show this message");
}

/// Regenerates generated documentation, or verifies it is current.
///
/// Generation is driven by `params/deepmind.toml`, which lands with the `param`
/// module. Until then there is nothing to generate and nothing to check.
fn docs(_check: bool) -> ExitCode {
    println!("no generated documentation yet; it arrives with the parameter table");
    ExitCode::SUCCESS
}
