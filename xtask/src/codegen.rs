//! Generates the library's parameter tables from `spec/`.
//!
//! The library cannot read TOML at runtime - it runs on targets with no
//! filesystem and no allocator - so the specification is rendered into Rust and
//! checked in. `cargo xtask codegen` writes it; `cargo xtask codegen --check`
//! and the `generated_code_is_current` test fail when the checked-in file has
//! fallen behind `spec/`, the same safety net the documentation has.
//!
//! Only data is generated. The shapes that data fills, and everything that acts
//! on it, are hand-written in `deepmind-midi/src/param/mod.rs`, so reviewing a
//! specification change means reading a table rather than reading logic.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::docs::Outcome;
use crate::spec::{Spec, ValueTable};

/// Path of the generated parameter tables, relative to the repository root.
pub const CODE_PATH: &str = "deepmind-midi/src/param/generated.rs";

/// Renders the parameter tables and compares them against the file on disk.
///
/// Writes the result unless `check` is set.
///
/// # Errors
///
/// Returns a message when the spec cannot be loaded, a name in it does not make
/// a usable Rust identifier, `rustfmt` cannot be run, or the file cannot be
/// written.
pub fn run(root: &Path, check: bool) -> Result<Outcome, String> {
    let spec = Spec::load(root)?;
    let rendered = format(root, &render(&spec)?)?;
    let path = root.join(CODE_PATH);
    if std::fs::read_to_string(&path).is_ok_and(|found| found == rendered) {
        return Ok(Outcome::Current);
    }
    if !check {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, rendered).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(Outcome::Stale)
}

/// Runs the rendered source through `rustfmt`, so that `cargo fmt --check` has
/// nothing to say about a file nobody edits by hand.
///
/// It goes through a file under `target/` rather than a pipe: a quarter of a
/// megabyte of source deadlocks a pipe that nothing is draining, and a scratch
/// file costs one write.
fn format(root: &Path, source: &str) -> Result<String, String> {
    let scratch = root.join("target").join("xtask");
    std::fs::create_dir_all(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let file: PathBuf = scratch.join("generated.rs");
    std::fs::write(&file, source).map_err(|e| format!("{}: {e}", file.display()))?;

    let output = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg("--config-path")
        .arg(root.join("rustfmt.toml"))
        .arg(&file)
        .output()
        .map_err(|e| format!("rustfmt: {e}. Install it with `rustup component add rustfmt`"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt rejected the generated source: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))
}

/// Renders the whole file, unformatted.
fn render(spec: &Spec) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(HEADER);
    render_counts(spec, &mut out);
    render_groups(spec, &mut out)?;
    render_parameters(spec, &mut out)?;
    render_tables(spec, &mut out)?;
    render_controllers(spec, &mut out)?;
    Ok(out)
}

const HEADER: &str = "\
//! The parameter, value table and controller data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

// The parameter table is one match with an arm per parameter. Splitting it into
// chunks to satisfy a length lint would only hide what it is.
#![expect(clippy::too_many_lines, reason = \"a generated table, not logic\")]

use super::{Controller, ControllerKind, Kind, Parameter, ValueEntry, ValueTable};
use crate::sysex::inquiry::Version;

";

fn render_counts(spec: &Spec, out: &mut String) {
    let firmware = spec.default_firmware();
    let (major, minor) = version_parts(firmware);
    let _ = writeln!(
        out,
        "\
/// Number of program parameters. Offsets run `0..PARAMETER_COUNT`.
pub const PARAMETER_COUNT: usize = {};

/// Number of continuous controllers the synthesizer answers.
pub const CONTROLLER_COUNT: usize = {};

/// Number of value tables, counting a renumbered one once per firmware version.
pub const TABLE_COUNT: usize = {};

/// Firmware assumed by every lookup that is not given one: the newest the
/// specification describes.
pub const DEFAULT_FIRMWARE: Version = Version {{
    major: {major},
    minor: {minor},
}};
",
        spec.parameters.len(),
        spec.controllers.len(),
        spec.tables.len(),
    );
}

fn render_groups(spec: &Spec, out: &mut String) -> Result<(), String> {
    let mut groups: Vec<&str> = spec.parameters.iter().map(|p| p.group.as_str()).collect();
    groups.sort_unstable();
    groups.dedup();
    let idents = identifiers(groups.iter().copied(), "group")?;

    out.push_str(
        "\
/// Section of the instrument a parameter belongs to.
///
/// The groups the manual's own NRPN table is divided into, which is also how the
/// front panel is divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
pub enum Group {
",
    );
    for (name, ident) in groups.iter().zip(&idents) {
        let _ = writeln!(out, "    /// {}.\n    {ident},", doc(name));
    }
    out.push_str("}\n\nimpl Group {\n");
    let _ = writeln!(
        out,
        "    /// Every group, in alphabetical order.\n    pub const ALL: [Self; {}] = [",
        groups.len()
    );
    for ident in &idents {
        let _ = writeln!(out, "        Self::{ident},");
    }
    out.push_str(
        "    ];\n\n    /// Returns the group's name.\n    #[must_use]\n    pub const fn name(self) -> &'static str {\n        match self {\n",
    );
    for (name, ident) in groups.iter().zip(&idents) {
        let _ = writeln!(out, "            Self::{ident} => {name:?},");
    }
    out.push_str("        }\n    }\n}\n\n");
    Ok(())
}

fn render_parameters(spec: &Spec, out: &mut String) -> Result<(), String> {
    let names: Vec<&str> = spec.parameters.iter().map(|p| p.name.as_str()).collect();
    let idents = identifiers(names.iter().copied(), "parameter")?;
    let groups = group_identifiers(spec)?;
    let tables = table_identifiers(spec)?;

    out.push_str(
        "\
/// One of the 242 program parameters.
///
/// The discriminant is the parameter's NRPN number, which is also its byte
/// offset in an unpacked program dump; [`ParamId::offset`] is that number and
/// [`ParamId::from_offset`] the way back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ParamId {
",
    );
    for (parameter, ident) in spec.parameters.iter().zip(&idents) {
        let _ = writeln!(
            out,
            "    /// {}.\n    {ident} = {},",
            doc(&parameter.name),
            parameter.offset
        );
    }
    out.push_str("}\n\nimpl ParamId {\n");
    out.push_str(
        "    /// Every parameter, in offset order.\n    pub const ALL: [Self; PARAMETER_COUNT] = [\n",
    );
    for ident in &idents {
        let _ = writeln!(out, "        Self::{ident},");
    }
    out.push_str(
        "\
    ];

    /// Returns everything the specification says about this parameter.
    #[must_use]
    pub const fn info(self) -> Parameter {
        match self {
",
    );
    for (parameter, ident) in spec.parameters.iter().zip(&idents) {
        let group = groups
            .get(parameter.group.as_str())
            .ok_or_else(|| format!("parameter {} has an unknown group", parameter.offset))?;
        let kind = match (&parameter.value_table, parameter.kind.as_deref()) {
            (Some(id), Some(kind)) => {
                return Err(format!(
                    "parameter {} is both a {kind} and the {id} table",
                    parameter.offset
                ));
            }
            (Some(id), None) => {
                let ident = tables
                    .get(id.as_str())
                    .ok_or_else(|| format!("parameter {} names table {id}", parameter.offset))?;
                format!("Kind::Enumerated(TableId::{ident})")
            }
            (None, Some("switch")) => "Kind::Switch".to_owned(),
            (None, Some(other)) => {
                return Err(format!(
                    "parameter {} has unknown kind {other:?}",
                    parameter.offset
                ));
            }
            (None, None) => "Kind::Continuous".to_owned(),
        };
        let _ = writeln!(
            out,
            "            Self::{ident} => Parameter {{ name: {:?}, group: Group::{group}, min: {}, max: {}, kind: {kind} }},",
            parameter.name, parameter.min, parameter.max
        );
    }
    out.push_str("        }\n    }\n}\n\n");
    Ok(())
}

fn render_tables(spec: &Spec, out: &mut String) -> Result<(), String> {
    let idents = table_identifiers(spec)?;
    let mut ids: Vec<&str> = spec.tables.iter().map(|t| t.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();

    out.push_str(
        "\
/// A named set of parameter values.
///
/// Three of these were renumbered by firmware 1.1 rather than extended, so an
/// identifier names one table per firmware version rather than one table;
/// [`TableId::table_for`] is what picks between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
pub enum TableId {
",
    );
    for &id in &ids {
        let ident = &idents[id];
        let name = spec
            .table(id)
            .map_or(id, |table| table.name.as_str())
            .trim_end_matches(" (firmware 1.0)");
        let _ = writeln!(out, "    /// {}.\n    {ident},", doc(name));
    }
    out.push_str("}\n\nimpl TableId {\n");
    let _ = writeln!(
        out,
        "    /// Every value table identifier, in alphabetical order.\n    pub const ALL: [Self; {}] = [",
        ids.len()
    );
    for &id in &ids {
        let _ = writeln!(out, "        Self::{},", idents[id]);
    }
    out.push_str(
        "\
    ];

    /// Returns this table as the given firmware numbers it.
    #[must_use]
    pub const fn table_for(self, firmware: Version) -> &'static ValueTable {
        match self {
",
    );
    for &id in &ids {
        let arm = table_arm(spec, id)?;
        let _ = writeln!(out, "            Self::{} => {arm},", idents[id]);
    }
    out.push_str("        }\n    }\n}\n\n");

    for table in &spec.tables {
        let _ = writeln!(
            out,
            "static {}: ValueTable = ValueTable {{ id: TableId::{}, name: {:?}, confirmed: {}, partial: {}, entries: &[",
            static_name(spec, table),
            idents[table.id.as_str()],
            table.name,
            table.confirmed,
            table.partial
        );
        for entry in &table.entries {
            let _ = writeln!(
                out,
                "    ValueEntry {{ value: {}, name: {:?} }},",
                entry.value, entry.name
            );
        }
        out.push_str("] };\n\n");
    }
    Ok(())
}

/// Renders the body of one `table_for` arm: a static, or a chain choosing
/// between the versions of a renumbered table, newest first.
fn table_arm(spec: &Spec, id: &str) -> Result<String, String> {
    let mut versions: Vec<&ValueTable> = spec.tables.iter().filter(|t| t.id == id).collect();
    versions.sort_by_key(|table| {
        let (major, minor) = version_parts(table.firmware.as_deref().unwrap_or("0.0"));
        core::cmp::Reverse((major, minor))
    });
    let (last, rest) = versions
        .split_last()
        .ok_or_else(|| format!("enums.toml: no table for {id}"))?;

    let mut arm = String::new();
    for table in rest {
        let range = table
            .firmware
            .as_deref()
            .ok_or_else(|| format!("enums.toml: table {id} has an unversioned duplicate"))?;
        let (major, minor) = version_parts(range);
        let test = if range.ends_with('+') {
            format!("firmware.at_least({major}, {minor})")
        } else {
            format!("firmware.is({major}, {minor})")
        };
        let _ = write!(arm, "if {test} {{ &{} }} else ", static_name(spec, table));
    }
    let _ = write!(arm, "{{ &{} }}", static_name(spec, last));
    Ok(arm)
}

fn render_controllers(spec: &Spec, out: &mut String) -> Result<(), String> {
    let names: Vec<&str> = spec.parameters.iter().map(|p| p.name.as_str()).collect();
    let idents = identifiers(names.iter().copied(), "parameter")?;

    out.push_str(
        "\
/// Every controller the synthesizer answers, in controller number order.
///
/// 90 of them drive a program parameter; the rest are the controllers the MIDI
/// specification defines, used as it defines them. [`Controller::for_cc`] and
/// [`Controller::for_parameter`] are the two ways in.
pub const CONTROLLERS: [Controller; CONTROLLER_COUNT] = [
",
    );
    for controller in &spec.controllers {
        let kind = match controller.kind.as_str() {
            "parameter" => "ControllerKind::Parameter",
            "standard" => "ControllerKind::Standard",
            "other" => "ControllerKind::Other",
            other => return Err(format!("controllers.toml: unknown kind {other:?}")),
        };
        let parameter = match controller.parameter {
            None => "None".to_owned(),
            Some(offset) => {
                let index = usize::from(offset);
                let ident = idents.get(index).ok_or_else(|| {
                    format!(
                        "controllers.toml: CC {} names offset {offset}",
                        controller.cc
                    )
                })?;
                format!("Some(ParamId::{ident})")
            }
        };
        let _ = writeln!(
            out,
            "    Controller {{ cc: {}, name: {:?}, kind: {kind}, parameter: {parameter} }},",
            controller.cc, controller.name
        );
    }
    out.push_str("];\n");
    Ok(())
}

/// Returns the static's name for one table: the identifier, with the firmware
/// version appended where more than one table shares it.
fn static_name(spec: &Spec, table: &ValueTable) -> String {
    let upper = table.id.to_ascii_uppercase();
    let versions = spec.tables.iter().filter(|t| t.id == table.id).count();
    match (versions, table.firmware.as_deref()) {
        (1, _) | (_, None) => upper,
        (_, Some(range)) => {
            let (major, minor) = version_parts(range);
            format!("{upper}_FW_{major}_{minor}")
        }
    }
}

/// Maps each group name to its Rust identifier.
fn group_identifiers(spec: &Spec) -> Result<BTreeMap<&str, String>, String> {
    let mut groups: Vec<&str> = spec.parameters.iter().map(|p| p.group.as_str()).collect();
    groups.sort_unstable();
    groups.dedup();
    let idents = identifiers(groups.iter().copied(), "group")?;
    Ok(groups.into_iter().zip(idents).collect())
}

/// Maps each value table identifier to its Rust identifier.
fn table_identifiers(spec: &Spec) -> Result<BTreeMap<&str, String>, String> {
    let mut ids: Vec<&str> = spec.tables.iter().map(|t| t.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    let idents = identifiers(ids.iter().copied(), "value table")?;
    Ok(ids.into_iter().zip(idents).collect())
}

/// Turns each name into a Rust identifier, rejecting anything unusable.
///
/// Two names that collide would compile into one variant and silently lose a
/// parameter, so a collision is an error rather than a suffix.
fn identifiers<'a>(
    names: impl Iterator<Item = &'a str>,
    what: &str,
) -> Result<Vec<String>, String> {
    let idents: Vec<String> = names.map(pascal).collect();
    for (index, ident) in idents.iter().enumerate() {
        if ident.is_empty() || ident.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(format!("{what} {index} makes no identifier: {ident:?}"));
        }
        if idents.iter().skip(index + 1).any(|other| other == ident) {
            return Err(format!("two {what} names both make the identifier {ident}"));
        }
    }
    Ok(idents)
}

/// Renders a name as one Rust identifier, in the case Rust writes type names in.
///
/// `>` and `&` carry meaning in a parameter name - `Mod Wheel > Pitch Mod Depth`
/// is a route, `Key Sync & Loop` a pair - so they become words rather than being
/// dropped with the rest of the punctuation.
fn pascal(name: &str) -> String {
    let spelled = name.replace('>', " to ").replace('&', " and ");
    let mut out = String::new();
    for word in spelled
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(characters.map(|c| c.to_ascii_lowercase()));
        }
    }
    out
}

/// Quotes any word rustdoc would otherwise read as an unlinked item name.
///
/// `DeepMind` in a doc comment is a clippy warning; in a generated file it would
/// be a warning nobody can fix by hand.
fn doc(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            if camel_case(word) {
                format!("`{word}`")
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn camel_case(word: &str) -> bool {
    let mut lower = false;
    word.chars().any(|c| {
        let hump = lower && c.is_ascii_uppercase();
        lower |= c.is_ascii_lowercase();
        hump
    })
}

/// Splits a version or version range, `"1.1"` or `"1.1+"`, into its two numbers.
fn version_parts(version: &str) -> (u32, u32) {
    let mut parts = version.trim_end_matches('+').split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails when `deepmind-midi/src/param/generated.rs` does not match `spec/`.
    ///
    /// The same net the documentation has: editing a spec file without
    /// regenerating turns `cargo test` red rather than shipping a library that
    /// disagrees with its own specification.
    #[test]
    fn generated_code_is_current() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(Path::new("."));
        match run(root, true) {
            Ok(Outcome::Current) => {}
            Ok(Outcome::Stale) => panic!(
                "{CODE_PATH} is out of date with spec/. Run `cargo xtask codegen` and commit the result."
            ),
            Err(message) => panic!("{message}"),
        }
    }

    #[test]
    fn names_become_identifiers_rust_would_have_chosen() {
        assert_eq!(pascal("LFO 1 Rate"), "Lfo1Rate");
        assert_eq!(pascal("OSC 1 PWM Source"), "Osc1PwmSource");
        assert_eq!(pascal("Arp On/Off"), "ArpOnOff");
        assert_eq!(pascal("Arp Rate (tempo)"), "ArpRateTempo");
        assert_eq!(pascal("Key Sync & Loop"), "KeySyncAndLoop");
        assert_eq!(pascal("VCF Mod Wheel > LFO Depth"), "VcfModWheelToLfoDepth");
    }

    #[test]
    fn colliding_names_are_an_error_not_a_lost_parameter() {
        let names = ["Arp On/Off", "Arp On Off"];
        assert!(identifiers(names.into_iter(), "parameter").is_err());
    }

    #[test]
    fn firmware_ranges_split_into_numbers() {
        assert_eq!(version_parts("1.1+"), (1, 1));
        assert_eq!(version_parts("1.0"), (1, 0));
    }
}
