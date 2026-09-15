//! Generates the library's effect panel tables from `spec/`.
//!
//! An FX engine holds twelve raw parameter bytes and what they mean depends on
//! which of 35 algorithms it is running. `effects.toml` says what each slot is
//! called, what it is measured in and where its range ends; `panels.toml` says
//! what kind of control it is and which slots belong together; `layout.toml`
//! says which family the algorithm belongs to. All three are validated when the
//! spec loads, and this is what carries them into the crate.
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

use super::{Algorithm, EngineParameters, FxSlot};
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
    render_slots(spec, &statics, &mut out);
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
            "    Algorithm {{ name: {:?}, full_name: {:?}, category: {category:?}, slots: &{name} }},",
            effect.name, effect.full_name,
        );
    }
    out.push_str("];\n\n");
}

fn render_slots(spec: &Spec, statics: &BTreeMap<u16, String>, out: &mut String) {
    for effect in &spec.effects {
        let name = statics.get(&effect.r#type).map_or("", String::as_str);
        let _ = writeln!(
            out,
            "/// {}.\nstatic {name}: [FxSlot; {}] = [",
            doc(&effect.full_name),
            effect.parameters.len()
        );
        for parameter in &effect.parameters {
            let panel = spec
                .panels
                .iter()
                .find(|panel| panel.r#type == effect.r#type)
                .and_then(|panel| panel.slots.iter().find(|slot| slot.slot == parameter.slot));
            let title = panel.map_or(parameter.name.as_str(), |slot| slot.title.as_str());
            let group = panel.and_then(|slot| slot.group.as_deref());
            let switch = panel.is_some_and(|slot| slot.kind == "switch");
            let _ = writeln!(
                out,
                "    FxSlot {{ slot: {}, reference: {:?}, title: {title:?}, kind: {}, values: {}, unit: {}, min: {}, max: {}, group: {}, modulatable: {} }},",
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

fn optional(value: Option<&str>) -> String {
    value.map_or_else(|| "None".to_owned(), |value| format!("Some({value:?})"))
}
