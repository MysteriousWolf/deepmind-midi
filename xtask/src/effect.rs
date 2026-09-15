//! Generates the library's effect panel tables from `spec/`.
//!
//! An FX engine holds twelve raw parameter bytes and what they mean depends on
//! which of 35 algorithms it is running. `effects.toml` says what each slot is
//! called, what it is measured in and where its range ends; `panels.toml` says
//! what kind of control it is and which slots belong together; `layout.toml`
//! says which family the algorithm belongs to, where its slots sit on the grid
//! and what its own editor panel is made of; `routing.toml` says how the four
//! engines are wired to each other. All four are validated when the spec loads,
//! and this is what carries them into the crate.
//!
//! Only data is generated. The types it fills, and everything that reads them,
//! are hand-written in `deepmind-midi/src/effect/mod.rs`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::codegen::{Identifiers, doc, snake_identifiers};
use crate::spec::Spec;

/// Path of the generated effect tables, relative to the repository root.
pub const CODE_PATH: &str = "deepmind-midi/src/effect/generated.rs";

const HEADER: &str = "\
//! The effect algorithm and panel data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

use super::{
    Algorithm, Align, Colour, Control, Engine, EngineParameters, FxSlot, Grid, Mode, Panel, Routing,
    Row, Source,
};
use crate::param::{Kind, ParamId};

";

/// Renders the whole file, unformatted.
///
/// # Errors
///
/// Returns a message when an algorithm's name makes no Rust identifier, when two
/// make the same one, or when the parameter table holds no `FX n Param m` for a
/// slot the engines are supposed to address.
pub fn render(spec: &Spec, idents: &Identifiers) -> Result<String, String> {
    let statics = statics(spec)?;
    let mut out = String::new();
    out.push_str(HEADER);
    render_counts(spec, &mut out);
    render_engines(spec, idents, &mut out)?;
    render_algorithms(spec, &statics, &mut out);
    render_slots(spec, &statics, &mut out)?;
    render_grid(spec, &mut out);
    render_panels(spec, &statics, &mut out)?;
    render_routings(spec, &mut out)?;
    Ok(out)
}

/// The static's name for each algorithm, by `FX Type` value.
///
/// Uppercased from the same snake case the rest of the generator uses, so a
/// name that makes no identifier, or two that make the same one, is an error
/// here rather than a table with a slot missing from it.
fn statics(spec: &Spec) -> Result<BTreeMap<u16, String>, String> {
    let names = snake_identifiers(spec.effects.iter().map(|e| e.name.as_str()), "effect")?;
    Ok(spec
        .effects
        .iter()
        .map(|effect| effect.r#type)
        .zip(names.into_iter().map(|name| name.to_ascii_uppercase()))
        .collect())
}

fn render_counts(spec: &Spec, out: &mut String) {
    let _ = writeln!(
        out,
        "\
/// Number of effect algorithms the newest firmware offers.
///
/// Firmware decides how many exist, so this counts what the newest one has;
/// [`Algorithm::for_value`](super::Algorithm::for_value) is what reads an older
/// numbering.
pub const ALGORITHM_COUNT: usize = {};

/// Parameter slots one effect engine holds, whatever it is running.
///
/// An algorithm that uses fewer leaves the rest doing nothing, which is what a
/// short [`Algorithm::slots`](super::Algorithm::slots) means.
pub const SLOTS_PER_ENGINE: usize = {};

/// Number of effect engines.
pub const ENGINE_COUNT: usize = {};
",
        spec.effects.len(),
        spec.engines.slots_per_engine,
        spec.engines.engine_offsets.len(),
    );
}

/// Renders the parameters each engine addresses, as `ParamId`s.
///
/// Found by name in the parameter table rather than by adding a slot number to
/// a base offset, so a renamed or moved parameter is a build error instead of a
/// table that addresses the wrong byte.
fn render_engines(spec: &Spec, idents: &Identifiers, out: &mut String) -> Result<(), String> {
    let by_name: BTreeMap<&str, &str> = spec
        .parameters
        .iter()
        .zip(idents.parameters.iter())
        .map(|(parameter, ident)| (parameter.name.as_str(), ident.as_str()))
        .collect();
    let named = |name: String| -> Result<String, String> {
        by_name
            .get(name.as_str())
            .map(|ident| format!("ParamId::{ident}"))
            .ok_or_else(|| format!("parameters.toml has no {name:?} for the effect tables"))
    };

    out.push_str(
        "\
/// What each engine addresses, in engine order.
pub(super) static ENGINES: [EngineParameters; ENGINE_COUNT] = [
",
    );
    for engine in 1..=spec.engines.engine_offsets.len() {
        let _ = writeln!(
            out,
            "    EngineParameters {{\n        algorithm: {},\n        gain: {},\n        slots: [",
            named(format!("FX {engine} Type"))?,
            named(format!("FX {engine} Output Gain"))?,
        );
        for slot in 1..=spec.engines.slots_per_engine {
            let _ = writeln!(
                out,
                "            {},",
                named(format!("FX {engine} Param {slot}"))?
            );
        }
        out.push_str("        ],\n    },\n");
    }
    out.push_str("];\n\n");
    Ok(())
}

fn render_algorithms(spec: &Spec, statics: &BTreeMap<u16, String>, out: &mut String) {
    out.push_str(
        "\
/// Every algorithm, in the order the newest firmware numbers them.
pub(super) static ALGORITHMS: [Algorithm; ALGORITHM_COUNT] = [
",
    );
    for effect in &spec.effects {
        let category = spec
            .layouts
            .iter()
            .find(|layout| layout.r#type == effect.r#type)
            .map_or("", |layout| layout.category.as_str());
        let name = statics.get(&effect.r#type).map_or("", String::as_str);
        let _ = writeln!(
            out,
            "    Algorithm {{ index: {}, name: {:?}, full_name: {:?}, category: {category:?}, slots: &{name} }},",
            effect.r#type, effect.name, effect.full_name,
        );
    }
    out.push_str("];\n\n");
}

/// Renders each algorithm's slots, including where the FX page draws them.
///
/// The column and row come from `layout.toml` rather than from dividing the
/// slot number by the grid's width: the file measured which row each slot is
/// on, and a derivation would quietly stop agreeing with it the day an
/// algorithm is measured as filling its rows some other way.
///
/// # Errors
///
/// Returns a message when a slot is placed on no row, which the spec loader
/// has already checked and which would otherwise be a silent `0, 0`.
fn render_slots(
    spec: &Spec,
    statics: &BTreeMap<u16, String>,
    out: &mut String,
) -> Result<(), String> {
    for effect in &spec.effects {
        let name = statics.get(&effect.r#type).map_or("", String::as_str);
        let _ = writeln!(
            out,
            "/// {}.\nstatic {name}: [FxSlot; {}] = [",
            doc(&effect.full_name),
            effect.parameters.len()
        );
        let places = placements(spec, effect.r#type);
        for parameter in &effect.parameters {
            let &(column, row) = places.get(&parameter.slot).ok_or_else(|| {
                format!(
                    "layout.toml: {} puts slot {} on no row",
                    effect.name, parameter.slot
                )
            })?;
            let panel = spec
                .panels
                .iter()
                .find(|panel| panel.r#type == effect.r#type)
                .and_then(|panel| panel.slots.iter().find(|slot| slot.slot == parameter.slot));
            let title = panel.map_or(parameter.name.as_str(), |slot| slot.title.as_str());
            let group = panel.and_then(|slot| slot.group.as_deref());
            let switch = panel.is_some_and(|slot| slot.kind == "switch");
            // The description is a field behind a `cfg`, not a table beside
            // the slots: with the feature off it is not in the struct at all,
            // so a build without it carries neither the prose nor a pointer to
            // where the prose would have been.
            let description = format!(
                "#[cfg(feature = \"descriptions\")] description: {}",
                optional(parameter.description.as_deref()),
            );
            let _ = writeln!(
                out,
                "    FxSlot {{ slot: {}, reference: {:?}, title: {title:?}, kind: {}, values: {}, unit: {}, min: {}, max: {}, group: {}, modulatable: {}, column: {column}, row: {row}, {description} }},",
                parameter.slot,
                parameter.r#ref,
                if switch {
                    "Kind::Switch"
                } else {
                    "Kind::Continuous"
                },
                values(parameter.values.as_deref()),
                optional(parameter.unit.as_deref()),
                optional(parameter.min.as_deref()),
                optional(parameter.max.as_deref()),
                optional(group),
                parameter.mod_dest,
            );
        }
        out.push_str("];\n\n");
    }
    Ok(())
}

/// Returns the column and row `layout.toml` puts each of an effect's slots on.
fn placements(spec: &Spec, r#type: u16) -> BTreeMap<u8, (usize, usize)> {
    let mut places = BTreeMap::new();
    let Some(layout) = spec.layouts.iter().find(|layout| layout.r#type == r#type) else {
        return places;
    };
    for (row, line) in layout.rows.iter().enumerate() {
        for (column, slot) in line.slots.iter().enumerate() {
            places.insert(*slot, (column, row));
        }
    }
    places
}

/// Renders the names a slot picks between, in the order the manual prints them.
fn values(listed: Option<&str>) -> String {
    let Some(listed) = listed else {
        return "&[]".to_owned();
    };
    let mut out = String::from("&[");
    for value in listed.split(',').map(str::trim).filter(|v| !v.is_empty()) {
        let _ = write!(out, "{value:?}, ");
    }
    out.push(']');
    out
}

/// Renders an optional string as the `Option<&'static str>` a table holds.
pub fn optional(value: Option<&str>) -> String {
    value.map_or_else(|| "None".to_owned(), |value| format!("Some({value:?})"))
}

/// Renders the grid the synthesizer draws its own FX page on.
///
/// In the pixels of the 128x64 display it was measured on, with the display's
/// dimensions beside it, which is what makes it a proportion a host can scale
/// rather than a size it has to adopt.
fn render_grid(spec: &Spec, out: &mut String) {
    let grid = &spec.grid;
    let _ = writeln!(
        out,
        "\
/// The one grid every algorithm's slots are placed on.
pub(super) static GRID: Grid = Grid {{
    columns: {},
    rows: {},
    display_width: {:?},
    display_height: {:?},
    first_x: {:?},
    first_y: {:?},
    column_pitch: {:?},
    row_pitch: {:?},
    control_diameter: {:?},
}};
",
        grid.columns,
        grid.rows,
        grid.display_width,
        grid.display_height,
        grid.first_x,
        grid.first_y,
        grid.column_pitch,
        grid.row_pitch,
        grid.control_diameter,
    );
}

/// Renders what each algorithm's own editor panel is made of: its control
/// shape, its four measured colours, and the rows it fills.
///
/// A separate table from the algorithm table, reached by index, so that a host
/// that never draws a panel does not carry twelve bytes of colour per effect:
/// nothing in the algorithm table points at this one.
///
/// # Errors
///
/// Returns a message when an algorithm has no layout, or when a layout declares
/// a control or an alignment the library has no variant for.
fn render_panels(
    spec: &Spec,
    statics: &BTreeMap<u16, String>,
    out: &mut String,
) -> Result<(), String> {
    out.push_str(
        "\
/// What each algorithm's own editor panel is made of, in the same order as
/// `ALGORITHMS`.
pub(super) static PANELS: [Panel; ALGORITHM_COUNT] = [
",
    );
    for effect in &spec.effects {
        let layout = spec
            .layouts
            .iter()
            .find(|layout| layout.r#type == effect.r#type)
            .ok_or_else(|| format!("layout.toml: no layout for {}", effect.name))?;
        let name = statics.get(&effect.r#type).map_or("", String::as_str);
        let _ = writeln!(
            out,
            "    Panel {{ control: {}, chassis: {}, face: {}, cap: {}, accent: {}, rows: &{name}_ROWS }},",
            control(&layout.control)?,
            colour(&layout.chassis)?,
            colour(&layout.face)?,
            colour(&layout.cap)?,
            colour(&layout.accent)?,
        );
    }
    out.push_str("];\n\n");

    for effect in &spec.effects {
        let layout = spec
            .layouts
            .iter()
            .find(|layout| layout.r#type == effect.r#type)
            .ok_or_else(|| format!("layout.toml: no layout for {}", effect.name))?;
        let name = statics.get(&effect.r#type).map_or("", String::as_str);
        let _ = writeln!(
            out,
            "/// The rows {} fills.\nstatic {name}_ROWS: [Row; {}] = [",
            doc(&effect.full_name),
            layout.rows.len()
        );
        for row in &layout.rows {
            let slots: Vec<String> = row.slots.iter().map(u8::to_string).collect();
            let _ = writeln!(
                out,
                "    Row {{ slots: &[{}], align: {} }},",
                slots.join(", "),
                align(&row.align)?
            );
        }
        out.push_str("];\n\n");
    }
    Ok(())
}

/// Renders the ten topologies and the three FX modes.
///
/// # Errors
///
/// Returns a message when a routing names a slot outside the four engines,
/// which the spec loader has already checked.
fn render_routings(spec: &Spec, out: &mut String) -> Result<(), String> {
    let _ = writeln!(
        out,
        "\
/// Number of ways the four engines can be wired.
pub const ROUTING_COUNT: usize = {};

/// Number of settings the `FX Mode` byte has.
pub const MODE_COUNT: usize = {};
",
        spec.routings.len(),
        spec.fx_modes.len(),
    );

    out.push_str(
        "\
/// Every topology, in the order the `FX Routing` byte numbers them.
pub(super) static ROUTINGS: [Routing; ROUTING_COUNT] = [
",
    );
    for routing in &spec.routings {
        let value = u8::try_from(routing.value)
            .map_err(|_| format!("routing.toml: {} is past a byte", routing.label))?;
        let feeds: Vec<String> = (1..=spec.engines.engine_offsets.len())
            .map(|engine| {
                let slot = u8::try_from(engine).unwrap_or(u8::MAX);
                let sources = routing
                    .slots
                    .iter()
                    .find(|s| s.slot == slot)
                    .map(|s| s.from.as_slice())
                    .unwrap_or_default();
                sources
                    .iter()
                    .map(|&from| source(from))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|rendered| format!("&[{}]", rendered.join(", ")))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let output: Vec<String> = routing
            .output
            .iter()
            .map(|&slot| engine(slot))
            .collect::<Result<_, _>>()?;
        let _ = writeln!(
            out,
            "    Routing {{ value: {value}, label: {:?}, name: {:?}, feedback: {}, note: {}, feeds: [{}], output: &[{}] }},",
            routing.label,
            routing.name,
            routing.feedback,
            optional(routing.note.as_deref()),
            feeds.join(", "),
            output.join(", "),
        );
    }
    out.push_str("];\n\n");

    out.push_str(
        "\
/// Every `FX Mode` setting, in the order the byte numbers them.
pub(super) static MODES: [Mode; MODE_COUNT] = [
",
    );
    for mode in &spec.fx_modes {
        let value = u8::try_from(mode.value)
            .map_err(|_| format!("routing.toml: mode {} is past a byte", mode.name))?;
        let _ = writeln!(
            out,
            "    Mode {{ value: {value}, name: {:?}, analog_path: {}, digital_path: {} }},",
            mode.name, mode.analog_path, mode.digital_path,
        );
    }
    out.push_str("];\n");
    Ok(())
}

/// Renders what reaches an engine: the block's own input, or another engine.
fn source(from: u8) -> Result<String, String> {
    if from == 0 {
        return Ok("Source::Input".to_owned());
    }
    Ok(format!("Source::Engine({})", engine(from)?))
}

/// Renders one engine, counting from 1 as the manual numbers them.
fn engine(slot: u8) -> Result<String, String> {
    Ok(match slot {
        1 => "Engine::One",
        2 => "Engine::Two",
        3 => "Engine::Three",
        4 => "Engine::Four",
        other => return Err(format!("routing.toml: {other} is not an engine")),
    }
    .to_owned())
}

/// Renders a `#rrggbb` colour as its three components.
fn colour(hex: &str) -> Result<String, String> {
    let digits = hex
        .strip_prefix('#')
        .filter(|digits| digits.len() == 6)
        .ok_or_else(|| format!("layout.toml: {hex:?} is not an #rrggbb colour"))?;
    let mut parts = Vec::with_capacity(3);
    for index in 0..3 {
        let pair = digits
            .get(index * 2..index * 2 + 2)
            .ok_or_else(|| format!("layout.toml: {hex:?} is not an #rrggbb colour"))?;
        parts.push(
            u8::from_str_radix(pair, 16)
                .map_err(|e| format!("layout.toml: {hex:?}: {e}"))?
                .to_string(),
        );
    }
    Ok(format!("Colour::new({})", parts.join(", ")))
}

/// Renders what an effect's own editor panel draws for a sweeping parameter.
fn control(name: &str) -> Result<String, String> {
    Ok(match name {
        "knob" => "Control::Knob",
        "fader" => "Control::Fader",
        "display" => "Control::Display",
        other => return Err(format!("layout.toml: unknown control {other:?}")),
    }
    .to_owned())
}

/// Renders what a row that is not full does with the space left over.
fn align(name: &str) -> Result<String, String> {
    Ok(match name {
        "left" => "Align::Left",
        "right" => "Align::Right",
        "centre" => "Align::Centre",
        "spread" => "Align::Spread",
        other => return Err(format!("layout.toml: unknown alignment {other:?}")),
    }
    .to_owned())
}
