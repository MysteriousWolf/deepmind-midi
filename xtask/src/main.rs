//! Repository automation.
//!
//! ```text
//! cargo xtask docs [--check]        regenerate docs/midi-spec.md, or verify it
//! cargo xtask hooks install        install a pre-commit hook that regenerates
//! cargo xtask version              print the current workspace version
//! cargo xtask release --bump KIND  bump the workspace version (release|patch)
//! ```
//!
//! Documentation tables are generated from `spec/`, which is the single source
//! of truth for the protocol. `docs --check` and the `docs_are_current` test both
//! fail when the checked-in document does not match the spec.

mod docs;
mod release;
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
        "hooks" => hooks_command(flags),
        "version" => version_command(),
        "release" => release_command(flags),
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
    println!("  docs [--check]         regenerate docs/midi-spec.md from spec/, or");
    println!("                         report whether it is current without writing");
    println!("  hooks install          install a pre-commit hook that regenerates the");
    println!("                         documentation and stages it before each commit");
    println!("  version                print the current workspace version");
    println!("  release --bump KIND    bump the workspace version, KIND is release or patch");
    println!("  help                   show this message");
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

/// Contents of the pre-commit hook. Keeping generation local means a commit can
/// never carry a stale document, and CI only ever has to verify.
const PRE_COMMIT: &str = "\
#!/bin/sh
# Installed by `cargo xtask hooks install`.
# Regenerates documentation from spec/ and stages the result.
set -e
cargo xtask docs
git add docs/midi-spec.md
";

fn hooks_command(flags: &[String]) -> Result<(), String> {
    match flags.first().map(String::as_str) {
        Some("install") => {}
        Some(other) => return Err(format!("unknown hooks action: {other}, expected install")),
        None => return Err("hooks needs an action: install".to_owned()),
    }

    let path = root().join(".git/hooks/pre-commit");
    let parent = path.parent().ok_or("hook path has no parent")?;
    if !parent.is_dir() {
        return Err(format!(
            "{} does not exist, is this a git checkout?",
            parent.display()
        ));
    }
    if path.exists() {
        return Err(format!(
            "{} already exists. Remove it first, or add `cargo xtask docs` to it by hand.",
            path.display()
        ));
    }

    std::fs::write(&path, PRE_COMMIT).map_err(|e| format!("{}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    println!("installed {}", path.display());
    Ok(())
}

fn version_command() -> Result<(), String> {
    println!("{}", release::current(&root())?);
    Ok(())
}

fn release_command(flags: &[String]) -> Result<(), String> {
    let kind = flags
        .iter()
        .position(|f| f == "--bump")
        .and_then(|i| flags.get(i + 1))
        .ok_or("release needs --bump release or --bump patch")?;
    let bump = match kind.as_str() {
        "release" => release::Bump::Release,
        "patch" => release::Bump::Patch,
        other => {
            return Err(format!(
                "unknown bump kind: {other}, expected release or patch"
            ));
        }
    };

    let root = root();
    let current = release::current(&root)?;
    let next = current.next(bump, release::current_year());
    release::write(&root, next)?;
    println!("{current} -> {next}");
    // Consumed by the release workflow to tag and name the release.
    if let Ok(output) = std::env::var("GITHUB_OUTPUT") {
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(output)
            .map_err(|e| format!("GITHUB_OUTPUT: {e}"))?;
        writeln!(file, "version={next}").map_err(|e| format!("GITHUB_OUTPUT: {e}"))?;
    }
    Ok(())
}
