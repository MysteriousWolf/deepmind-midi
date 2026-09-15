//! Generates the library's parameter tables from `spec/`.
//!
//! The library runs on targets with no filesystem and no allocator, so it cannot
//! read TOML at runtime; the specification is rendered into Rust and checked
//! in. `cargo xtask codegen` writes it; `cargo xtask codegen --check` and the
//! `generated_code_is_current` test fail when the checked-in file has fallen
//! behind `spec/`, the same safety net the documentation has.
//!
//! Only data is generated. The types that data fills, and everything that acts
//! on it, are hand-written in `deepmind-midi/src/param/mod.rs` and
//! `deepmind-midi/src/program/mod.rs`, so reviewing a specification change means
//! reading a table rather than logic.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use crate::output::Output;
use crate::spec::{Spec, ValueTable, version_parts};

/// Path of the generated parameter tables, relative to the repository root.
pub const CODE_PATH: &str = "deepmind-midi/src/param/generated.rs";

/// Every generated source file, in the order they are written.
pub const CODE_PATHS: [&str; 2] = [CODE_PATH, crate::program::CODE_PATH];

/// Loads the spec, then renders the generated source files as [`generate`].
///
/// # Errors
///
/// As [`generate`], or a message when the spec cannot be loaded.
pub fn run(root: &Path, check: bool) -> Result<Vec<String>, String> {
    generate(&Spec::load(root)?, root, check)
}

/// Renders the generated source files and compares them against the files on
/// disk.
///
/// Writes each one that differs, unless `check` is set. Returns the paths,
/// relative to the root, of the files that differed.
///
/// # Errors
///
/// Returns a message when a name in the spec does not make a usable Rust
/// identifier, `rustfmt` cannot be run, or a file cannot be written.
pub fn generate(spec: &Spec, root: &Path, check: bool) -> Result<Vec<String>, String> {
    let idents = Identifiers::new(spec)?;
    let files = [
        (CODE_PATH, render(spec, &idents)?),
        (
            crate::program::CODE_PATH,
            crate::program::render(spec, &idents)?,
        ),
    ];

    let mut output = Output::new(root, check);
    for (path, source) in files {
        output.write(path, &format(root, path, &source)?)?;
    }
    Ok(output.stale())
}

/// Runs the rendered source through `rustfmt`, so that `cargo fmt --check` has
/// nothing to say about a file nobody edits by hand.
///
/// `rustfmt` reads a file, so the source goes through one under `target/`,
/// named after the file it is destined for and rewritten only when the render
/// changed.
fn format(root: &Path, path: &str, source: &str) -> Result<String, String> {
    let scratch = root.join("target").join("xtask");
    std::fs::create_dir_all(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let stem = Path::new(path)
        .parent()
        .and_then(Path::file_name)
        .map_or_else(
            || "generated".to_owned(),
            |dir| dir.to_string_lossy().into_owned(),
        );
    let file = scratch.join(format!("{stem}-generated.rs"));
    if !std::fs::read_to_string(&file).is_ok_and(|found| found == source) {
        std::fs::write(&file, source).map_err(|e| format!("{}: {e}", file.display()))?;
    }

    let output = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg("--config-path")
        .arg(root.join("rustfmt.toml"))
        .arg("--emit")
        .arg("stdout")
        .arg(&file)
        .output()
        .map_err(|e| format!("rustfmt: {e}. Install it with `rustup component add rustfmt`"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt rejected the generated source: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    // Emitting to stdout prefixes the source with the file's name and a blank line.
    let formatted = String::from_utf8_lossy(&output.stdout);
    let header = format!("{}:\n\n", file.display());
    Ok(formatted
        .strip_prefix(&header)
        .unwrap_or(&formatted)
        .to_owned())
}

/// The Rust identifiers the specification's names become, built once per run.
pub struct Identifiers {
    /// One per parameter, in offset order.
    pub parameters: Vec<String>,
    /// By group name.
    pub groups: BTreeMap<String, String>,
    /// By value table identifier.
    pub tables: BTreeMap<String, String>,
}

impl Identifiers {
    /// Turns every parameter, group and value table name into an identifier.
    ///
    /// # Errors
    ///
    /// As [`identifiers`].
    pub fn new(spec: &Spec) -> Result<Self, String> {
        let owned = |map: BTreeMap<&str, String>| {
            map.into_iter()
                .map(|(name, ident)| (name.to_owned(), ident))
                .collect()
        };
        Ok(Self {
            parameters: identifiers(spec.parameters.iter().map(|p| p.name.as_str()), "parameter")?,
            groups: owned(group_identifiers(spec)?),
            tables: owned(table_identifiers(spec)?),
        })
    }
}

/// Renders the whole file, unformatted.
fn render(spec: &Spec, idents: &Identifiers) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(HEADER);
    render_counts(spec, &mut out);
    render_groups(idents, &mut out);
    render_parameters(spec, idents, &mut out)?;
    render_tables(spec, idents, &mut out)?;
    render_controllers(spec, idents, &mut out)?;
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

fn render_groups(idents: &Identifiers, out: &mut String) {
    let groups: Vec<&str> = idents.groups.keys().map(String::as_str).collect();
    let idents: Vec<&str> = idents.groups.values().map(String::as_str).collect();

    out.push_str(
        "\
/// Section of the instrument a parameter belongs to.
///
/// The groups the manual's own NRPN table is divided into, which is also how the
/// front panel is divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Group {
",
    );
    for (name, ident) in groups.iter().zip(&idents) {
        let _ = writeln!(out, "    /// {}.\n    {ident},", doc(name));
    }
    out.push_str("}\n\nimpl Group {\n");
    let _ = writeln!(
        out,
        "    /// Every group, in alphabetical order.\n    pub const ALL: &'static [Self] = &[",
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
}

fn render_parameters(spec: &Spec, idents: &Identifiers, out: &mut String) -> Result<(), String> {
    let Identifiers {
        parameters: idents,
        groups,
        tables,
    } = idents;

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
#[non_exhaustive]
pub enum ParamId {
",
    );
    for (parameter, ident) in spec.parameters.iter().zip(idents) {
        let _ = writeln!(
            out,
            "    /// {}.\n    {ident} = {},",
            doc(&parameter.name),
            parameter.offset
        );
    }
    out.push_str("}\n\nimpl ParamId {\n");
    out.push_str(
        "    /// Every parameter, in offset order: [`PARAMETER_COUNT`] of them.\n    pub const ALL: &'static [Self] = &[\n",
    );
    for ident in idents {
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
    for (parameter, ident) in spec.parameters.iter().zip(idents) {
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

fn render_tables(spec: &Spec, idents: &Identifiers, out: &mut String) -> Result<(), String> {
    // A value table entry names the parameters it moves under the names
    // `parameters.toml` gives, which is what keeps the file readable by hand.
    let by_name: BTreeMap<&str, &str> = spec
        .parameters
        .iter()
        .zip(idents.parameters.iter())
        .map(|(parameter, ident)| (parameter.name.as_str(), ident.as_str()))
        .collect();
    let idents = &idents.tables;
    let ids: Vec<&str> = idents.keys().map(String::as_str).collect();

    out.push_str(
        "\
/// A named set of parameter values.
///
/// Three of these were renumbered by firmware 1.1 rather than extended, so an
/// identifier names one table per firmware version rather than one table;
/// [`TableId::table_for`] is what picks between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
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
        "    /// Every value table identifier, in alphabetical order.\n    pub const ALL: &'static [Self] = &[",
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
                "    ValueEntry {{ value: {}, name: {:?}, parameters: {} }},",
                entry.value,
                entry.name,
                parameter_slice(&by_name, &entry.parameters)?
            );
        }
        out.push_str("] };\n\n");
    }
    Ok(())
}

/// Renders the parameters a value table entry names, as a `ParamId` slice.
///
/// The spec writes them under the names `parameters.toml` gives, which is what
/// keeps the table readable; the loader has already checked that every one of
/// them resolves.
fn parameter_slice(by_name: &BTreeMap<&str, &str>, names: &[String]) -> Result<String, String> {
    if names.is_empty() {
        return Ok("&[]".to_owned());
    }
    let mut rendered = String::from("&[");
    for name in names {
        let ident = by_name
            .get(name.as_str())
            .ok_or_else(|| format!("enums.toml: no parameter named {name:?}"))?;
        let _ = write!(rendered, "ParamId::{ident}, ");
    }
    rendered.push(']');
    Ok(rendered)
}

/// Renders the body of one `table_for` arm: a static, or a chain choosing
/// between the versions of a renumbered table, newest first.
fn table_arm(spec: &Spec, id: &str) -> Result<String, String> {
    let versions = spec.versions_of(id);
    let (last, rest) = versions
        .split_last()
        .ok_or_else(|| format!("enums.toml: no table for {id}"))?;

    let mut arm = String::new();
    for table in rest {
        let _ = write!(
            arm,
            "if {} {{ &{} }} else ",
            firmware_test(table)?,
            static_name(spec, table)
        );
    }
    let _ = write!(arm, "{{ &{} }}", static_name(spec, last));
    Ok(arm)
}

/// Renders the test that picks one version of a renumbered table: `firmware`
/// is at least the version a `1.1+` range starts at, or is exactly a `1.0`.
///
/// # Errors
///
/// Returns a message when the table names no firmware, since a table without
/// a range has nothing to test.
pub fn firmware_test(table: &ValueTable) -> Result<String, String> {
    let range = table.firmware.as_deref().ok_or_else(|| {
        format!(
            "enums.toml: table {} has an unversioned duplicate",
            table.id
        )
    })?;
    let (major, minor) = version_parts(range);
    Ok(if range.ends_with('+') {
        format!("firmware.at_least({major}, {minor})")
    } else {
        format!("firmware.is({major}, {minor})")
    })
}

fn render_controllers(spec: &Spec, idents: &Identifiers, out: &mut String) -> Result<(), String> {
    let idents = &idents.parameters;

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
pub fn group_identifiers(spec: &Spec) -> Result<BTreeMap<&str, String>, String> {
    let mut groups: Vec<&str> = spec.parameters.iter().map(|p| p.group.as_str()).collect();
    groups.sort_unstable();
    groups.dedup();
    let idents = identifiers(groups.iter().copied(), "group")?;
    Ok(groups.into_iter().zip(idents).collect())
}

/// Maps each value table identifier to its Rust identifier.
pub fn table_identifiers(spec: &Spec) -> Result<BTreeMap<&str, String>, String> {
    let mut ids: Vec<&str> = spec.tables.iter().map(|t| t.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    let idents = identifiers(ids.iter().copied(), "value table")?;
    Ok(ids.into_iter().zip(idents).collect())
}

/// Turns each name into a Rust identifier, rejecting anything unusable.
///
/// # Errors
///
/// Returns a message when a name makes no identifier, or when two names make
/// the same one, which would compile into one variant.
pub fn identifiers<'a>(
    names: impl Iterator<Item = &'a str>,
    what: &str,
) -> Result<Vec<String>, String> {
    cased(names, what, pascal)
}

/// Turns each name into a Rust identifier in the case Rust writes values and
/// functions in, rejecting anything unusable.
///
/// # Errors
///
/// As [`identifiers`].
pub fn snake_identifiers<'a>(
    names: impl Iterator<Item = &'a str>,
    what: &str,
) -> Result<Vec<String>, String> {
    cased(names, what, snake)
}

fn cased<'a>(
    names: impl Iterator<Item = &'a str>,
    what: &str,
    case: impl Fn(&str) -> String,
) -> Result<Vec<String>, String> {
    let idents: Vec<String> = names.map(case).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (index, ident) in idents.iter().enumerate() {
        if ident.is_empty() || ident.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(format!("{what} {index} makes no identifier: {ident:?}"));
        }
        if !seen.insert(ident) {
            return Err(format!("two {what} names both make the identifier {ident}"));
        }
    }
    Ok(idents)
}

/// Renders a name as one Rust identifier, in the case Rust writes type names in.
///
/// A leading number is spelled out, since an identifier cannot start with a
/// digit: `4 Pole` is `FourPole`. A number the table below does not spell is an
/// error rather than a mangled name.
pub fn pascal(name: &str) -> String {
    let mut out = String::new();
    for word in words(name) {
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(characters.map(|c| c.to_ascii_lowercase()));
        }
    }
    spell_leading_number(&out)
}

/// Renders a name as one Rust identifier, in the case Rust writes values in.
///
/// A word that is only digits joins the word before it, so `LFO 1 Rate` is
/// `lfo1_rate` rather than `lfo_1_rate`.
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    for word in words(name) {
        if !out.is_empty() && !word.chars().all(|c| c.is_ascii_digit()) {
            out.push('_');
        }
        out.extend(word.chars().map(|c| c.to_ascii_lowercase()));
    }
    spell_leading_number(&out)
}

/// Splits a name into the words an identifier is built from.
///
/// Punctuation that carries meaning becomes a word rather than being dropped
/// with the rest: `>` and `&` are a route and a pair in a parameter name
/// (`Mod Wheel > Pitch Mod Depth`, `Key Sync & Loop`), `'` is the unit in an
/// oscillator range (`16'`), and a sign against a number is its sign
/// (`Fixed -12`). A parenthesised number is what a controller is numbered on one
/// firmware, and firmware renumbered it, so it is not part of the name: `CC X
/// (115)` and `CC X (114)` are one value under two firmware versions.
///
/// Words already run together in the manual's own spelling are split where the
/// case changes, so `DeepVRB` is `Deep` and `VRB` rather than one long word.
fn words(name: &str) -> Vec<String> {
    let mut spelled = String::new();
    let mut rest = name;
    while let Some(open) = rest.find('(') {
        let Some(close) = rest[open..].find(')').map(|end| open + end) else {
            break;
        };
        let inner = rest.get(open + 1..close).unwrap_or_default();
        spelled.push_str(rest.get(..open).unwrap_or_default());
        if !inner.chars().all(|c| c.is_ascii_digit()) || inner.is_empty() {
            spelled.push_str(inner);
        }
        rest = rest.get(close + 1..).unwrap_or_default();
    }
    spelled.push_str(rest);

    let spelled = spelled
        .replace('>', " to ")
        .replace('&', " and ")
        .replace('\'', " foot ");

    let mut signed = String::new();
    let mut previous = ' ';
    let mut characters = spelled.chars().peekable();
    while let Some(character) = characters.next() {
        let sign = matches!(character, '+' | '-')
            && !previous.is_ascii_alphanumeric()
            && characters.peek().is_some_and(char::is_ascii_digit);
        match (sign, character) {
            (true, '+') => signed.push_str(" plus "),
            (true, _) => signed.push_str(" minus "),
            _ => signed.push(character),
        }
        previous = character;
    }

    signed
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .flat_map(humps)
        .collect()
}

/// Splits one run of characters where the manual runs two words together: where
/// a lower-case letter meets an upper-case one, and where a number meets a
/// letter.
fn humps(word: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut previous = ' ';
    for character in word.chars() {
        let hump = character.is_ascii_uppercase() && previous.is_ascii_lowercase();
        let counted = character.is_ascii_alphabetic() && previous.is_ascii_digit();
        if hump || counted {
            out.push(String::new());
        }
        match out.last_mut() {
            Some(current) => current.push(character),
            None => out.push(String::from(character)),
        }
        previous = character;
    }
    out
}

/// Spells out a leading number, which an identifier cannot start with.
///
/// Returns the identifier unchanged when it starts with a number this does not
/// spell, which [`cased`] then rejects by name rather than shipping something
/// that does not compile.
fn spell_leading_number(ident: &str) -> String {
    const SPELLED: [(&str, &str); 10] = [
        ("16", "Sixteen"),
        ("12", "Twelve"),
        ("10", "Ten"),
        ("8", "Eight"),
        ("6", "Six"),
        ("5", "Five"),
        ("4", "Four"),
        ("3", "Three"),
        ("2", "Two"),
        ("1", "One"),
    ];
    let digits: String = ident.chars().take_while(char::is_ascii_digit).collect();
    let Some((_, word)) = SPELLED.iter().find(|(number, _)| *number == digits) else {
        return ident.to_owned();
    };
    let rest = ident.get(digits.len()..).unwrap_or_default();
    // The case of what follows says which case the identifier is in: `4Pole` is
    // a type name and `4_pole` a method name.
    let word = if rest.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
        word.to_ascii_lowercase()
    } else {
        (*word).to_owned()
    };
    format!("{word}{rest}")
}

/// Quotes any word rustdoc would otherwise read as an unlinked item name.
///
/// `DeepMind` in a doc comment is a clippy warning; in a generated file it would
/// be a warning nobody can fix by hand.
pub fn doc(text: &str) -> String {
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
    let mut characters = word.chars();
    let Some(_) = characters.next() else {
        return false;
    };
    let rest: String = characters.collect();
    rest.contains(|c: char| c.is_ascii_uppercase())
        && rest.contains(|c: char| c.is_ascii_lowercase())
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
        match generate(crate::spec::shared(), &crate::root(), true) {
            Ok(stale) if stale.is_empty() => {}
            Ok(stale) => panic!(
                "{} out of date with spec/. Run `cargo xtask codegen` and commit the result.",
                stale.join(" and ")
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
    fn manual_spellings_split_into_the_words_they_run_together() {
        assert_eq!(pascal("TC-DeepVRB"), "TcDeepVrb");
        assert_eq!(pascal("NoteOff Vel"), "NoteOffVel");
        assert_eq!(pascal("EdisonEX1"), "EdisonEx1");
    }

    /// A parenthesised number is a controller number, and firmware 1.1 moved
    /// the three that have one, so it cannot be part of the identifier.
    #[test]
    fn a_parenthesised_number_is_not_part_of_the_name() {
        assert_eq!(pascal("CC X (115)"), "CcX");
        assert_eq!(pascal("CC X (114)"), "CcX");
        assert_eq!(pascal("LFO1 (Uni)"), "Lfo1Uni");
    }

    #[test]
    fn signs_and_units_become_words_and_leading_numbers_are_spelled() {
        assert_eq!(pascal("Fixed +12"), "FixedPlus12");
        assert_eq!(pascal("Fixed -12"), "FixedMinus12");
        assert_eq!(pascal("16'"), "SixteenFoot");
        assert_eq!(pascal("4 Pole"), "FourPole");
        assert_eq!(pascal("3TapDelay"), "ThreeTapDelay");
        assert_eq!(pascal("4 Pole"), "FourPole");
    }

    #[test]
    fn method_names_read_the_way_rust_writes_them() {
        assert_eq!(snake("LFO 1 Rate"), "lfo1_rate");
        assert_eq!(snake("Arp On/Off"), "arp_on_off");
        assert_eq!(snake("VCF HighPass Frequency"), "vcf_high_pass_frequency");
        assert_eq!(
            snake("Mod Wheel > Pitch Mod Depth"),
            "mod_wheel_to_pitch_mod_depth"
        );
        assert_eq!(snake("4 Pole"), "four_pole");
    }

    #[test]
    fn colliding_names_are_an_error_not_a_lost_parameter() {
        let names = ["Arp On/Off", "Arp On Off"];
        assert!(identifiers(names.into_iter(), "parameter").is_err());
    }
}
