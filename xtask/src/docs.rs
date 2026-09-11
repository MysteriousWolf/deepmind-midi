//! Generates the reference tables in `docs/midi-spec.md` from `spec/`.
//!
//! Prose in that file is written by hand. Tables live between marker comments
//! and are owned by this module:
//!
//! ```text
//! <!-- generated:parameters -->
//! ...replaced on every run...
//! <!-- /generated:parameters -->
//! ```
//!
//! `cargo xtask docs` rewrites the marked regions; `cargo xtask docs --check`
//! reports staleness without touching the file. The `generated_documentation_is_current`
//! test runs the same comparison, so a stale checkout fails `cargo test`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::spec::Spec;

/// Path of the document this module owns, relative to the repository root.
pub const DOC_PATH: &str = "docs/midi-spec.md";

/// Outcome of a generation run.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The file on disk already matched.
    Current,
    /// The file differed. It was rewritten unless the run was a check.
    Stale,
}

/// Renders the marked regions and compares them against the file on disk.
///
/// Writes the result unless `check` is set.
///
/// # Errors
///
/// Returns a message when the spec cannot be loaded, the document is missing,
/// a marker pair is missing or malformed, or the file cannot be written.
pub fn run(root: &Path, check: bool) -> Result<Outcome, String> {
    let spec = Spec::load(root)?;
    let path: PathBuf = root.join(DOC_PATH);
    let current = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;

    let mut updated = current.clone();
    for (marker, body) in [
        ("messages", render_messages(&spec)),
        ("parameters", render_parameters(&spec)),
        ("value-tables", render_value_tables(&spec)),
        ("globals", render_globals(&spec)),
        ("corrections", render_corrections(&spec)),
    ] {
        updated = splice(&updated, marker, &body)?;
    }

    if updated == current {
        return Ok(Outcome::Current);
    }
    if !check {
        std::fs::write(&path, updated).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(Outcome::Stale)
}

/// Replaces the text between `<!-- generated:NAME -->` and `<!-- /generated:NAME -->`.
fn splice(document: &str, marker: &str, body: &str) -> Result<String, String> {
    let open = format!("<!-- generated:{marker} -->");
    let close = format!("<!-- /generated:{marker} -->");
    let start = document
        .find(&open)
        .ok_or_else(|| format!("{DOC_PATH}: missing marker {open}"))?
        + open.len();
    let end = document
        .find(&close)
        .ok_or_else(|| format!("{DOC_PATH}: missing marker {close}"))?;
    if end < start {
        return Err(format!("{DOC_PATH}: marker {close} precedes {open}"));
    }
    Ok(format!(
        "{}\n\n{}\n\n{}",
        &document[..start],
        body.trim(),
        &document[end..]
    ))
}

/// Escapes the characters that would otherwise break out of a table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

fn render_messages(spec: &Spec) -> String {
    let mut out = String::from("| Cmd | Message | Direction | Payload | Raw | Packed |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    for message in &spec.messages {
        let direction = if message.direction == "to_device" {
            "host to synth"
        } else {
            "synth to host"
        };
        let raw = message.raw_len.map_or_else(String::new, |n| n.to_string());
        let packed = message
            .packed_len
            .map_or_else(String::new, |n| n.to_string());
        let _ = writeln!(
            out,
            "| `{:02X}` | {} | {direction} | {} | {raw} | {packed} |",
            message.command,
            cell(&message.name),
            cell(message.payload.as_deref().unwrap_or("none")),
        );
    }

    let notes: Vec<&crate::spec::Message> =
        spec.messages.iter().filter(|m| m.note.is_some()).collect();
    if !notes.is_empty() {
        out.push('\n');
        for message in notes {
            let note = message.note.as_deref().unwrap_or_default().trim();
            let _ = writeln!(out, "**{}.** {note}\n", message.name);
        }
    }
    out
}

fn render_parameters(spec: &Spec) -> String {
    let mut out = String::new();
    let mut group = "";
    for parameter in &spec.parameters {
        if parameter.group != group {
            group = &parameter.group;
            let _ = write!(
                out,
                "\n### {group}\n\n| Offset | Parameter | Range | Values |\n|---|---|---|---|\n"
            );
        }
        let values = match (&parameter.kind, &parameter.value_table, &parameter.note) {
            (Some(kind), _, _) if kind == "switch" => "Off (0), On (1)".to_owned(),
            (_, Some(id), _) => {
                let name = spec.table(id).map_or(id.as_str(), |t| t.name.as_str());
                format!("[{name}](#{id})")
            }
            (_, _, Some(note)) => cell(note),
            _ => String::new(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {}-{} | {values} |",
            parameter.offset,
            cell(&parameter.name),
            parameter.min,
            parameter.max
        );
    }
    out
}

fn render_value_tables(spec: &Spec) -> String {
    let mut out = String::new();
    for table in &spec.tables {
        let _ = write!(
            out,
            "\n<a id=\"{}\"></a>\n\n#### {}\n\n",
            table.id, table.name
        );
        if let Some(note) = &table.note {
            let _ = write!(out, "{note}\n\n");
        }
        if !table.confirmed {
            out.push_str(
                "> Unconfirmed. This mapping is inferred and needs checking against hardware.\n\n",
            );
        }
        out.push_str("| Value | Name | Notes |\n|---|---|---|\n");
        for entry in &table.entries {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                entry.value,
                cell(&entry.name),
                cell(entry.description.as_deref().unwrap_or(""))
            );
        }
    }
    out
}

fn render_globals(spec: &Spec) -> String {
    let mut out = String::from("| Setting | Range | Notes |\n|---|---|---|\n");
    for global in &spec.globals {
        let _ = writeln!(
            out,
            "| {} | {}-{} | {} |",
            cell(&global.name),
            global.min,
            global.max,
            cell(global.note.as_deref().unwrap_or(""))
        );
    }
    out
}

fn render_corrections(spec: &Spec) -> String {
    // Several corrections apply identically to every slot of a repeated group,
    // such as the eight modulation matrix busses, so rows are grouped by text.
    let mut groups: Vec<(&str, Vec<&crate::spec::Parameter>)> = Vec::new();
    for parameter in &spec.parameters {
        let Some(correction) = parameter.correction.as_deref() else {
            continue;
        };
        match groups.iter_mut().find(|(text, _)| *text == correction) {
            Some((_, members)) => members.push(parameter),
            None => groups.push((correction, vec![parameter])),
        }
    }
    if groups.is_empty() {
        return "None recorded.".to_owned();
    }

    let mut out = String::from("| Offsets | Parameter | Correction |\n|---|---|---|\n");
    for (correction, members) in groups {
        let offsets = if members.len() <= 8 {
            members
                .iter()
                .map(|p| p.offset.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            format!(
                "{} values from {} to {}",
                members.len(),
                members[0].offset,
                members[members.len() - 1].offset
            )
        };
        let name = match members.as_slice() {
            [only] => cell(&only.name),
            _ => format!("{} and {} more", cell(&members[0].name), members.len() - 1),
        };
        let _ = writeln!(out, "| {offsets} | {name} | {} |", cell(correction));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails when `docs/midi-spec.md` does not match the spec files.
    ///
    /// This is the safety net that keeps generated documentation honest: editing
    /// a spec file without regenerating turns `cargo test` red.
    #[test]
    fn generated_documentation_is_current() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(Path::new("."));
        match run(root, true) {
            Ok(Outcome::Current) => {}
            Ok(Outcome::Stale) => panic!(
                "{DOC_PATH} is out of date with spec/. Run `cargo xtask docs` and commit the result."
            ),
            Err(message) => panic!("{message}"),
        }
    }
}
