//! Generates the library's front panel table from `spec/front.toml`.
//!
//! Which parameters the instrument puts a physical control under, what is
//! silkscreened over each one, and which of the panel's two rows its plate is
//! in. `parameters.toml` says what exists and `controllers.toml` says what has a
//! CC; neither says what has a fader, and this is the file that does.
//!
//! Only data is generated. The types it fills, and everything that reads them,
//! are hand-written in `deepmind-midi/src/front/mod.rs`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::codegen::{Identifiers, doc, snake_identifiers};
use crate::spec::Spec;

/// Path of the generated front panel table, relative to the repository root.
pub const CODE_PATH: &str = "deepmind-midi/src/front/generated.rs";

const HEADER: &str = "\
//! The front panel data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

use super::{PanelControl, PanelShape, Section};
use crate::param::{Group, ParamId};

";

/// Renders the whole file, unformatted.
///
/// # Errors
///
/// Returns a message when a section's name makes no Rust identifier, when two
/// make the same one, or when a section names a parameter, a group or a control
/// shape the rest of the specification does not have.
pub fn render(spec: &Spec, idents: &Identifiers) -> Result<String, String> {
    let statics = snake_identifiers(spec.sections.iter().map(|s| s.name.as_str()), "section")?;
    let statics: Vec<String> = statics
        .into_iter()
        .map(|name| name.to_ascii_uppercase())
        .collect();

    let mut out = String::new();
    out.push_str(HEADER);
    render_counts(spec, &mut out);
    render_sections(spec, idents, &statics, &mut out)?;
    render_controls(spec, idents, &statics, &mut out)?;
    Ok(out)
}

fn render_counts(spec: &Spec, out: &mut String) {
    let controls: usize = spec
        .sections
        .iter()
        .map(|section| section.controls.len())
        .sum();
    let _ = writeln!(
        out,
        "\
/// Number of groups the front panel is divided into.
pub const SECTION_COUNT: usize = {};

/// Number of rows the front panel is printed in.
pub const PANEL_ROWS: u8 = {};

/// Number of parameters the front panel puts a control under.
///
/// A fraction of [`PARAMETER_COUNT`](crate::param::PARAMETER_COUNT): the panel
/// is the handful a player reaches for, and everything else is a press away on
/// the instrument's own display.
pub const PANEL_CONTROL_COUNT: usize = {controls};
",
        spec.sections.len(),
        spec.panel_rows,
    );
}

fn render_sections(
    spec: &Spec,
    idents: &Identifiers,
    statics: &[String],
    out: &mut String,
) -> Result<(), String> {
    out.push_str(
        "\
/// Every group of the front panel, in the order the instrument prints them.
pub(super) static SECTIONS: [Section; SECTION_COUNT] = [
",
    );
    for (section, name) in spec.sections.iter().zip(statics) {
        let group = idents
            .groups
            .get(section.group.as_str())
            .ok_or_else(|| format!("front.toml: {} opens unknown group", section.name))?;
        let _ = writeln!(
            out,
            "    Section {{ name: {:?}, row: {}, group: Group::{group}, note: {}, controls: &{name} }},",
            section.name,
            section.row,
            crate::effect::optional(section.note.as_deref()),
        );
    }
    out.push_str("];\n\n");
    Ok(())
}

fn render_controls(
    spec: &Spec,
    idents: &Identifiers,
    statics: &[String],
    out: &mut String,
) -> Result<(), String> {
    let by_name: BTreeMap<&str, &str> = spec
        .parameters
        .iter()
        .zip(idents.parameters.iter())
        .map(|(parameter, ident)| (parameter.name.as_str(), ident.as_str()))
        .collect();

    for (section, name) in spec.sections.iter().zip(statics) {
        let _ = writeln!(
            out,
            "/// The controls the {} plate carries.\nstatic {name}: [PanelControl; {}] = [",
            doc(&section.name),
            section.controls.len()
        );
        for control in &section.controls {
            let parameter = by_name
                .get(control.parameter.as_str())
                .ok_or_else(|| format!("front.toml: no parameter named {:?}", control.parameter))?;
            let _ = writeln!(
                out,
                "    PanelControl {{ parameter: ParamId::{parameter}, legend: {:?}, shape: {} }},",
                control.legend,
                shape(&control.shape)?
            );
        }
        out.push_str("];\n\n");
    }
    Ok(())
}

/// Renders what a hand touches on the panel.
fn shape(name: &str) -> Result<String, String> {
    Ok(match name {
        "fader" => "PanelShape::Fader",
        "button" => "PanelShape::Button",
        "lamps" => "PanelShape::Lamps",
        other => return Err(format!("front.toml: unknown shape {other:?}")),
    }
    .to_owned())
}
