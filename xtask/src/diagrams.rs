//! Mermaid diagram sources, generated from the specification.
//!
//! Every diagram is written twice: to `docs/diagrams/<id>.mmd` so the source
//! stands alone and can be rendered by any Mermaid tool, and inline into
//! `docs/midi-spec.md` so the document renders on its own anywhere Mermaid is
//! supported. Both come from the same function, so they cannot diverge.
//!
//! Figures that would otherwise be a pre-rendered image are Mermaid too. Nothing
//! here needs a build step to be readable.

use std::fmt::Write as _;

use crate::spec::Spec;

/// Directory holding the standalone diagram sources, relative to the repository root.
pub const DIR: &str = "docs/diagrams";

/// One diagram: its file stem, a human-readable title, and its Mermaid source.
#[derive(Debug)]
pub struct Diagram {
    /// File stem, used for `docs/diagrams/<id>.mmd`.
    pub id: &'static str,
    /// Title, used as the section heading in the document.
    pub title: &'static str,
    /// Mermaid source, without the surrounding fence.
    pub source: String,
}

/// Builds every diagram from the specification.
///
/// # Errors
///
/// Returns a message when a parameter group the diagrams describe is empty, or
/// when a value table they count is missing.
pub fn all(spec: &Spec) -> Result<Vec<Diagram>, String> {
    Ok(vec![
        Diagram {
            id: "signal-path",
            title: "Voice signal path",
            source: signal_path(spec)?,
        },
        Diagram {
            id: "modulation-matrix",
            title: "Modulation matrix",
            source: modulation_matrix(spec)?,
        },
        Diagram {
            id: "envelope",
            title: "Envelopes",
            source: envelope(),
        },
    ])
}

/// Returns the inclusive offset range of a parameter group, as `first-last`.
fn range(spec: &Spec, group: &str) -> Result<String, String> {
    let offsets: Vec<u16> = spec
        .parameters
        .iter()
        .filter(|p| p.group == group)
        .map(|p| p.offset)
        .collect();
    match (offsets.first(), offsets.last()) {
        (Some(first), Some(last)) => Ok(format!("{first}-{last}")),
        _ => Err(format!("no parameters in group {group}")),
    }
}

/// Returns how many real choices a value table offers, discounting its Off entry.
fn choices(spec: &Spec, id: &str) -> Result<usize, String> {
    Ok(spec
        .table(id)
        .ok_or_else(|| format!("missing value table {id}"))?
        .entries
        .len()
        - 1)
}

fn signal_path(spec: &Spec) -> Result<String, String> {
    let mut out = String::from("flowchart LR\n");
    for (id, label, group) in [
        ("OSC", "OSC 1 + OSC 2<br>noise", "Oscillators"),
        ("VCF", "VCF<br>low pass + high pass", "VCF"),
        ("VCA", "VCA", "VCA"),
        ("FX", "FX<br>4 slots", "Effects"),
    ] {
        let _ = writeln!(
            out,
            "    {id}[\"{label}<br><small>{}</small>\"]",
            range(spec, group)?
        );
    }
    out.push_str("    OUT([output])\n    OSC --> VCF --> VCA --> FX --> OUT\n\n");
    for (id, label, group) in [
        ("VCAENV", "VCA envelope", "VCA Envelope"),
        ("VCFENV", "VCF envelope", "VCF Envelope"),
        ("MODENV", "mod envelope", "Mod Envelope"),
        ("LFO1", "LFO 1", "LFO 1"),
        ("LFO2", "LFO 2", "LFO 2"),
        ("SEQ", "control sequencer", "Control Sequencer"),
    ] {
        let _ = writeln!(
            out,
            "    {id}(\"{label}<br><small>{}</small>\")",
            range(spec, group)?
        );
    }
    let _ = writeln!(
        out,
        "    MOD{{{{\"mod matrix<br>8 busses<br><small>{}</small>\"}}}}",
        range(spec, "Mod Matrix")?
    );
    out.push_str(
        "    VCAENV -.-> VCA\n    VCFENV -.-> VCF\n    MODENV -.-> MOD\n    \
         LFO1 -.-> MOD\n    LFO2 -.-> MOD\n    SEQ -.-> MOD\n    \
         MOD -.-> OSC\n    MOD -.-> VCF\n    MOD -.-> VCA\n    MOD -.-> FX",
    );
    Ok(out)
}

fn modulation_matrix(spec: &Spec) -> Result<String, String> {
    let mut out = String::from("flowchart LR\n");
    let _ = writeln!(
        out,
        "    SRC[\"source<br><small>0 = off, {} to choose from</small>\"]",
        choices(spec, "mod_source")?
    );
    out.push_str("    DEPTH[\"depth<br><small>-128 to +127</small>\"]\n");
    let _ = writeln!(
        out,
        "    DST[\"destination<br><small>0 = off, {} to choose from</small>\"]",
        choices(spec, "mod_destination")?
    );
    out.push_str("    SRC --> DEPTH --> DST");
    Ok(out)
}

/// The envelope shape, as a chart rather than a pre-rendered drawing.
///
/// Two series, one per curve setting, over the same four stages. Stage values are
/// on the parameters' own 0-255 scale.
fn envelope() -> String {
    "xychart-beta\n    \
     title \"One envelope, at two attack and decay curve settings\"\n    \
     x-axis \"time, key released after the sustain stage\" 0 --> 15\n    \
     y-axis \"level\" 0 --> 255\n    \
     line \"linear\" [0, 85, 170, 255, 215, 185, 160, 160, 160, 160, 160, 120, 80, 40, 0, 0]\n    \
     line \"exponential\" [0, 160, 215, 255, 190, 170, 160, 160, 160, 160, 160, 75, 35, 15, 0, 0]"
        .to_owned()
}
