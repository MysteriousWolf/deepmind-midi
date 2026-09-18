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

use super::{Lamp, PanelControl, PanelPress, PanelShape, Section, Sends};
use crate::effect::Colour;
use crate::param::{Group, ParamId};
use crate::pixels::Glyph;

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
    render_banners(spec, &mut out)?;
    render_sections(spec, idents, &statics, &mut out)?;
    render_controls(spec, idents, &statics, &mut out)?;
    render_presses(spec, &statics, &mut out)?;
    Ok(out)
}

/// Renders the colours a section's name is printed on.
///
/// An enum rather than a colour on each section: the instrument has three, a
/// plate is one of them, and a host that wants to know which two plates are not
/// red asks the variant rather than comparing hex.
fn render_banners(spec: &Spec, out: &mut String) -> Result<(), String> {
    out.push_str(
        "\
/// A colour the front panel prints a section's name on.
///
/// The largest colour on the instrument, and a fact about the front rather than
/// about a byte: a photograph of a `DeepMind` is a dark panel with a row of red
/// stripes across it. Three of them, measured off the product photographs, and
/// reached from [`Section::banner`](super::Section::banner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Banner {
",
    );
    for banner in &spec.banners {
        let _ = writeln!(
            out,
            "    /// {}\n    {},",
            doc(&banner.description),
            pascal(&banner.name)?
        );
    }
    out.push_str("}\n\n");

    let _ = writeln!(
        out,
        "\
/// Number of colours a section's name is printed on.
pub const BANNER_COUNT: usize = {};
",
        spec.banners.len()
    );

    out.push_str(
        "\
/// The strip and the ink of each banner, in the order the enum names them.
pub(super) static BANNERS: [(Colour, Colour); BANNER_COUNT] = [
",
    );
    for banner in &spec.banners {
        let _ = writeln!(
            out,
            "    ({}, {}),",
            crate::effect::colour_of("front.toml", &banner.plate)?,
            crate::effect::colour_of("front.toml", &banner.ink)?,
        );
    }
    out.push_str("];\n\n");
    Ok(())
}

/// Returns the variant a banner's name became: `red` is `Red`.
fn pascal(name: &str) -> Result<String, String> {
    let mut chars = name.chars();
    let first = chars
        .next()
        .ok_or_else(|| "front.toml: a banner has no name".to_owned())?;
    Ok(first.to_ascii_uppercase().to_string() + chars.as_str())
}

fn render_counts(spec: &Spec, out: &mut String) {
    let controls: usize = spec
        .sections
        .iter()
        .map(|section| section.controls.len())
        .sum();
    let presses: usize = spec
        .sections
        .iter()
        .map(|section| section.presses.len())
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

/// Number of presses the front panel carries that are not a parameter change.
///
/// The two chord presses in the arpeggiator's row of buttons. Everything else a
/// hand can reach on the front either moves a parameter or changes what the
/// instrument's own display is showing.
pub const PANEL_PRESS_COUNT: usize = {presses};
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
        let clusters = section
            .controls
            .iter()
            .map(|control| control.cluster)
            .chain(section.presses.iter().map(|press| press.cluster))
            .max()
            .map_or(1, |last| last + 1);
        let banner = pascal(section.banner.as_deref().unwrap_or("red"))?;
        let presses = if section.presses.is_empty() {
            "&[]".to_owned()
        } else {
            format!("&{name}_PRESSES")
        };
        let _ = writeln!(
            out,
            "    Section {{ name: {:?}, row: {}, group: Group::{group}, banner: Banner::{banner}, \
             clusters: {clusters}, note: {}, controls: &{name}, presses: {presses} }},",
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
            let drawing = match &control.drawing {
                Some(name) => format!("Some(Glyph::{})", crate::glyph::ident(idents, name)?),
                None => "None".to_owned(),
            };
            let _ = writeln!(
                out,
                "    PanelControl {{ parameter: ParamId::{parameter}, legend: {:?}, \
                 drawing: {drawing}, shape: {}, cluster: {}, lamp: {} }},",
                control.legend,
                shape(&control.shape)?,
                control.cluster,
                if control.shape == "button" {
                    format!("Some({})", lamp(control.lamp.as_deref())?)
                } else {
                    "None".to_owned()
                },
            );
        }
        out.push_str("];\n\n");
    }
    Ok(())
}

/// Renders the presses a plate carries that are not a parameter change.
///
/// A second table beside the controls rather than a row in them: a
/// `PanelControl` is keyed by the parameter it moves, and the whole point of
/// these is that there is no parameter to key them by.
fn render_presses(spec: &Spec, statics: &[String], out: &mut String) -> Result<(), String> {
    for (section, name) in spec.sections.iter().zip(statics) {
        if section.presses.is_empty() {
            continue;
        }
        let _ = writeln!(
            out,
            "/// The presses the {} plate carries that move no parameter.\nstatic {name}_PRESSES: [PanelPress; {}] = [",
            doc(&section.name),
            section.presses.len()
        );
        for press in &section.presses {
            let _ = writeln!(
                out,
                "    PanelPress {{ legend: {:?}, shape: {}, lamp: {}, sends: {}, cluster: {}, \
                 note: {} }},",
                press.legend,
                shape(&press.shape)?,
                lamp(press.lamp.as_deref())?,
                sends(spec, &press.sends)?,
                press.cluster,
                crate::effect::optional(press.note.as_deref()),
            );
        }
        out.push_str("];\n\n");
    }
    Ok(())
}

/// Renders the colour of the lamp behind a press.
///
/// White is the rule's own answer for everything it does not name, so a press
/// that declares no colour is white.
fn lamp(colour: Option<&str>) -> Result<String, String> {
    Ok(match colour.unwrap_or("white") {
        "amber" => "Lamp::Amber",
        "cyan" => "Lamp::Cyan",
        "white" => "Lamp::White",
        other => return Err(format!("front.toml: unknown lamp {other:?}")),
    }
    .to_owned())
}

/// Renders what pressing a press puts on the wire.
fn sends(spec: &Spec, what: &str) -> Result<String, String> {
    if what == "nothing" {
        return Ok("Sends::Nothing".to_owned());
    }
    let cc = spec
        .controllers
        .iter()
        .find(|controller| controller.name == what)
        .ok_or_else(|| format!("front.toml: {what:?} is neither nothing nor a controller"))?
        .cc;
    Ok(format!("Sends::Controller({cc})"))
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
