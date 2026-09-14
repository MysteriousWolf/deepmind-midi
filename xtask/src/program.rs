//! Generates the program layer's value types and typed accessors from `spec/`.
//!
//! Two things come out of the same tables the parameter layer is built from.
//! One Rust type per value table, so a field holds `LfoShape::Triangle` rather
//! than a three; and one pair of accessors per parameter, so a host reaches a
//! byte by the name the synthesizer's display gives it rather than by offset.
//!
//! # Which tables become a type
//!
//! A table becomes a Rust enum when its entries are a closed set of names. Two
//! kinds are left as raw bytes, with the label as the way to read them:
//!
//! - **Numbers.** The three clock divider tables list `1/2`, `3/8`, `1/16`. Those
//!   are divisions of a bar, not names, and `Div1Over2` would be a worse way of
//!   writing `1/2` than `1/2` is.
//! - **Runs.** `SPREAD-1` stands for `SPREAD-1` through `SPREAD-254` and
//!   `Preset-1` for a bank of patterns. The specification marks those tables
//!   partial and lists only the first of each run, so an enum of what is listed
//!   would answer `None` for almost every value the synthesizer sends.
//!
//! Whether a table is one of those is read off the table rather than declared:
//! `spec/` describes the synthesizer, and how it lands in Rust is this
//! program's business.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::codegen::{doc, pascal, snake_identifiers, table_identifiers, version_parts};
use crate::spec::{Spec, ValueTable};

/// Path of the generated program types, relative to the repository root.
pub const CODE_PATH: &str = "deepmind-midi/src/program/generated.rs";

/// Prefix of the parameters that spell out the program's name.
///
/// They are one contiguous run of bytes holding a string, not seventeen
/// parameters a host would ever set one at a time, so they get
/// [`Program::name`](../../deepmind-midi/src/program/mod.rs) instead of
/// accessors of their own.
const NAME_PREFIX: &str = "Program Name Char";

/// Renders the whole file, unformatted.
///
/// # Errors
///
/// Returns a message when a name in the specification does not make a usable
/// Rust identifier, or when the program name is not one contiguous run of
/// parameters.
pub fn render(spec: &Spec) -> Result<String, String> {
    let mut out = String::from(HEADER);
    render_name_field(spec, &mut out)?;
    render_values(spec, &mut out)?;
    render_accessors(spec, &mut out)?;
    Ok(out)
}

const HEADER: &str = "\
//! The program layer's value types and typed accessors, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. [`Program`] itself,
//! and everything that is not one parameter or one value table, is in the parent
//! module.

// A value table lands here as one match arm per value, and the largest has 133
// of them. Chunking those to satisfy a length lint would only hide what they
// are, and two values a firmware does not have are two arms answering `None`.
#![expect(
    clippy::too_many_lines,
    clippy::match_same_arms,
    reason = \"generated tables, not logic\"
)]

use core::fmt;

use super::Program;
use crate::param::{DEFAULT_FIRMWARE, ParamId, TableId};
use crate::sysex::inquiry::Version;

";

/// Renders where the program's name sits, which the parameter table states by
/// naming seventeen consecutive bytes after it.
fn render_name_field(spec: &Spec, out: &mut String) -> Result<(), String> {
    let mut offsets: Vec<u16> = spec
        .parameters
        .iter()
        .filter(|parameter| parameter.name.starts_with(NAME_PREFIX))
        .map(|parameter| parameter.offset)
        .collect();
    offsets.sort_unstable();
    let (Some(&first), Some(&last)) = (offsets.first(), offsets.last()) else {
        return Err(format!("parameters.toml: no {NAME_PREFIX} parameters"));
    };
    if usize::from(last - first) + 1 != offsets.len() {
        return Err(format!(
            "parameters.toml: {NAME_PREFIX} parameters are not one contiguous run"
        ));
    }

    let _ = writeln!(
        out,
        "\
/// Offset of the first byte of the program's name.
pub const NAME_OFFSET: u8 = {first};

/// Bytes the name occupies. It holds sixteen characters and a terminator.
pub const NAME_LEN: usize = {};
",
        offsets.len()
    );
    Ok(())
}

/// Renders one Rust enum per value table whose entries are a closed set of
/// names.
fn render_values(spec: &Spec, out: &mut String) -> Result<(), String> {
    let table_idents = table_identifiers(spec)?;
    for id in typed_tables(spec) {
        let mut versions: Vec<&ValueTable> = spec.tables.iter().filter(|t| t.id == id).collect();
        versions.sort_by_key(|table| {
            let (major, minor) = version_parts(table.firmware.as_deref().unwrap_or("0.0"));
            std::cmp::Reverse((major, minor))
        });
        let (newest, older) = versions
            .split_first()
            .ok_or_else(|| format!("enums.toml: no table for {id}"))?;
        let ident = pascal(id);
        let table = table_idents
            .get(id)
            .ok_or_else(|| format!("enums.toml: no identifier for {id}"))?;
        let variants = variant_identifiers(newest, id)?;

        render_value_enum(spec, newest, &ident, &variants, out);
        render_value_impl(&ident, table, newest, older, &variants, out)?;
    }
    Ok(())
}

/// Renders the enum itself: one variant per entry of the newest firmware's
/// table.
fn render_value_enum(
    spec: &Spec,
    newest: &ValueTable,
    ident: &str,
    variants: &[String],
    out: &mut String,
) {
    let renumbered = spec.tables.iter().filter(|t| t.id == newest.id).count() > 1;
    let _ = writeln!(out, "/// {}.\n///", doc(&newest.name));
    if renumbered {
        let _ = writeln!(
            out,
            "\
/// Firmware renumbered this table rather than extending it, so a variant is what
/// the value means and not the byte it travels as. [`{ident}::raw_for`] is that
/// byte on a given firmware, and [`{ident}::raw`] is it on the newest.
///"
        );
    }
    if !newest.confirmed {
        let _ = writeln!(
            out,
            "\
/// The manual contradicts itself about this table and hardware has not settled
/// it. The entries are as printed.
///"
        );
    }
    let _ = writeln!(
        out,
        "\
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`{ident}::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
pub enum {ident} {{"
    );
    for (entry, variant) in newest.entries.iter().zip(variants) {
        let _ = writeln!(out, "    /// {}.\n    {variant},", doc(&entry.name));
    }
    out.push_str("}\n\n");
}

/// Renders everything the enum can do: the two directions of the raw byte, on
/// the newest firmware and on any other.
fn render_value_impl(
    ident: &str,
    table: &str,
    newest: &ValueTable,
    older: &[&ValueTable],
    variants: &[String],
    out: &mut String,
) -> Result<(), String> {
    render_value_header(ident, table, variants, out);
    render_from_raw(newest, older, variants, out)?;
    render_raw(newest, older, variants, out)?;
    render_value_names(ident, out);
    Ok(())
}

/// Renders the table's identity and the list of its values.
fn render_value_header(ident: &str, table: &str, variants: &[String], out: &mut String) {
    let _ = writeln!(out, "impl {ident} {{");
    let _ = writeln!(
        out,
        "\
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::{table};

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: [Self; {}] = [",
        variants.len()
    );
    for variant in variants {
        let _ = writeln!(out, "        Self::{variant},");
    }
    out.push_str(
        "\
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

",
    );
}

/// Renders the byte-to-value direction, over every firmware that numbers the
/// table differently.
fn render_from_raw(
    newest: &ValueTable,
    older: &[&ValueTable],
    variants: &[String],
    out: &mut String,
) -> Result<(), String> {
    if older.is_empty() {
        out.push_str(
            "\
    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
",
        );
        render_from_raw_arms(newest, variants, out)?;
        out.push_str("        }\n    }\n\n");
    } else {
        out.push_str(
            "\
    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Worth reaching for whenever the byte came from a stored program: nothing
    /// in a dump says which firmware wrote it, and this table was renumbered.
    #[must_use]
    pub const fn from_raw_for(raw: u8, firmware: Version) -> Option<Self> {
",
        );
        render_firmware_chain(newest, older, out, |table, out| {
            out.push_str("match raw {\n");
            render_from_raw_arms(table, variants, out)?;
            out.push_str("}\n");
            Ok(())
        })?;
        out.push_str("    }\n\n");
    }
    Ok(())
}

/// Renders the value-to-byte direction, over every firmware that numbers the
/// table differently.
fn render_raw(
    newest: &ValueTable,
    older: &[&ValueTable],
    variants: &[String],
    out: &mut String,
) -> Result<(), String> {
    out.push_str(
        "\
    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
",
    );
    for (entry, variant) in newest.entries.iter().zip(variants) {
        let _ = writeln!(out, "            Self::{variant} => {},", entry.value);
    }
    out.push_str("        }\n    }\n\n");

    if older.is_empty() {
        out.push_str(
            "\
    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

",
        );
    } else {
        out.push_str(
            "\
    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// `None` for a value that firmware does not have.
    #[must_use]
    pub const fn raw_for(self, firmware: Version) -> Option<u8> {
",
        );
        render_firmware_chain(newest, older, out, |table, out| {
            out.push_str("match self {\n");
            render_raw_for_arms(table, newest, variants, out)?;
            out.push_str("}\n");
            Ok(())
        })?;
        out.push_str("    }\n\n");
    }
    Ok(())
}

/// Renders the two ways to the displayed name, which come out of the value
/// table the parameter layer already holds.
fn render_value_names(ident: &str, out: &mut String) {
    out.push_str(
        "\
    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

",
    );
    let _ = writeln!(
        out,
        "\
impl fmt::Display for {ident} {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        f.write_str(self.name())
    }}
}}
"
    );
}

/// Renders the arms of a `from_raw` match over one firmware's table.
fn render_from_raw_arms(
    table: &ValueTable,
    variants: &[String],
    out: &mut String,
) -> Result<(), String> {
    for entry in &table.entries {
        let variant = variant_for(entry.name.as_str(), table, variants)?;
        let _ = writeln!(out, "            {} => Some(Self::{variant}),", entry.value);
    }
    out.push_str("            _ => None,\n");
    Ok(())
}

/// Renders the arms of a `raw_for` match over one firmware's table, answering
/// `None` for the values that firmware does not have.
fn render_raw_for_arms(
    table: &ValueTable,
    newest: &ValueTable,
    variants: &[String],
    out: &mut String,
) -> Result<(), String> {
    let mut values: BTreeMap<&str, u16> = BTreeMap::new();
    for entry in &table.entries {
        values.insert(
            variant_for(entry.name.as_str(), table, variants)?,
            entry.value,
        );
    }
    for (entry, variant) in newest.entries.iter().zip(variants) {
        let _ = entry;
        match values.get(variant.as_str()) {
            Some(value) => {
                let _ = writeln!(out, "            Self::{variant} => Some({value}),");
            }
            None => {
                let _ = writeln!(out, "            Self::{variant} => None,");
            }
        }
    }
    Ok(())
}

/// Renders one `if firmware.at_least(..) { .. } else { .. }` chain, newest
/// first, with `body` rendering each arm.
fn render_firmware_chain(
    newest: &ValueTable,
    older: &[&ValueTable],
    out: &mut String,
    body: impl Fn(&ValueTable, &mut String) -> Result<(), String>,
) -> Result<(), String> {
    let range = newest
        .firmware
        .as_deref()
        .ok_or_else(|| format!("enums.toml: table {} has an unversioned copy", newest.id))?;
    let (major, minor) = version_parts(range);
    let test = if range.ends_with('+') {
        format!("firmware.at_least({major}, {minor})")
    } else {
        format!("firmware.is({major}, {minor})")
    };
    let _ = writeln!(out, "        if {test} {{");
    body(newest, out)?;
    out.push_str("        } else {\n");
    let (last, rest) = older
        .split_last()
        .ok_or_else(|| format!("enums.toml: table {} has one version", newest.id))?;
    for table in rest {
        let range = table
            .firmware
            .as_deref()
            .ok_or_else(|| format!("enums.toml: table {} has an unversioned copy", table.id))?;
        let (major, minor) = version_parts(range);
        let _ = writeln!(out, "        if firmware.at_least({major}, {minor}) {{");
        body(table, out)?;
        out.push_str("        } else {\n");
    }
    body(last, out)?;
    for _ in 0..=rest.len() {
        out.push_str("        }\n");
    }
    Ok(())
}

/// Renders the typed accessors, one pair per parameter that is not part of the
/// program's name.
fn render_accessors(spec: &Spec, out: &mut String) -> Result<(), String> {
    let typed = typed_tables(spec);
    let named: Vec<&crate::spec::Parameter> = spec
        .parameters
        .iter()
        .filter(|parameter| !parameter.name.starts_with(NAME_PREFIX))
        .collect();
    let methods = snake_identifiers(named.iter().map(|p| p.name.as_str()), "parameter")?;
    let idents = crate::codegen::identifiers(named.iter().map(|p| p.name.as_str()), "parameter")?;

    out.push_str(
        "\
/// The parameters, one pair of accessors each.
///
/// A getter reads the byte the program holds and a setter writes it. A setter
/// that takes a raw byte clamps it to what the parameter accepts, since a `u8`
/// reaches values a parameter does not; [`Program::set`] is the checked way in.
///
/// The parameters that spell the program's name are not here. They are one
/// string, and [`Program::name`] is how a string is read.
impl Program {
",
    );
    for ((parameter, method), ident) in named.iter().zip(&methods).zip(&idents) {
        let table = parameter
            .value_table
            .as_deref()
            .filter(|id| typed.contains(id));
        let switch = parameter.kind.as_deref() == Some("switch");
        let name = doc(&parameter.name);
        let offset = parameter.offset;
        let (min, max) = (parameter.min, parameter.max);

        match (table, switch) {
            (Some(id), _) => {
                let value = pascal(id);
                let _ = writeln!(
                    out,
                    "\
    /// {name}.
    ///
    /// Offset {offset}. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn {method}(&self) -> Option<{value}> {{
        {value}::from_raw(self.get(ParamId::{ident}))
    }}

    /// Sets {name}.
    pub fn set_{method}(&mut self, value: {value}) {{
        self.set_clamped(ParamId::{ident}, value.raw());
    }}
"
                );
            }
            (None, true) => {
                let _ = writeln!(
                    out,
                    "\
    /// {name}.
    ///
    /// Offset {offset}. Off is zero and on is one.
    #[must_use]
    pub fn {method}(&self) -> bool {{
        self.get(ParamId::{ident}) != 0
    }}

    /// Sets {name}.
    pub fn set_{method}(&mut self, value: bool) {{
        self.set_clamped(ParamId::{ident}, u8::from(value));
    }}
"
                );
            }
            (None, false) => {
                let _ = writeln!(
                    out,
                    "\
    /// {name}.
    ///
    /// Offset {offset}, {min} to {max}.
    #[must_use]
    pub fn {method}(&self) -> u8 {{
        self.get(ParamId::{ident})
    }}

    /// Sets {name}, clamped to {min} to {max}.
    pub fn set_{method}(&mut self, value: u8) {{
        self.set_clamped(ParamId::{ident}, value);
    }}
"
                );
            }
        }
    }
    out.push_str("}\n");
    Ok(())
}

/// Returns the identifiers of the tables that become a Rust enum, in the order
/// the specification lists them.
fn typed_tables(spec: &Spec) -> Vec<&str> {
    let mut ids: Vec<&str> = spec
        .tables
        .iter()
        .filter(|table| !table.partial && !numeric(table))
        .map(|table| table.id.as_str())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Returns whether a table's entries are numbers rather than names.
///
/// `1/2` and `3/16` are divisions of a bar; `16'` is a length, and the mark that
/// says so is a letter as far as this is concerned.
fn numeric(table: &ValueTable) -> bool {
    !table.entries.iter().any(|entry| {
        entry
            .name
            .contains(|c: char| c.is_ascii_alphabetic() || c == '\'')
    })
}

/// Returns one variant identifier per entry of a table.
fn variant_identifiers(table: &ValueTable, id: &str) -> Result<Vec<String>, String> {
    crate::codegen::identifiers(
        table.entries.iter().map(|entry| entry.name.as_str()),
        &format!("{id} value"),
    )
}

/// Returns the variant one entry of one firmware's table belongs to.
///
/// The join is the name, since the value is what firmware changed. Spelling
/// changed with it (`NoteOff Vel` became `Note Off Vel`), so the join is the
/// name as an identifier, which both of those spell the same way.
fn variant_for<'a>(
    name: &str,
    table: &ValueTable,
    variants: &'a [String],
) -> Result<&'a str, String> {
    let wanted = pascal(name);
    variants
        .iter()
        .find(|variant| **variant == wanted)
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "enums.toml: {} has {name:?}, which the newest firmware's table does not",
                table.id
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(entries: &[&str]) -> ValueTable {
        ValueTable {
            id: String::from("test"),
            name: String::from("Test"),
            firmware: None,
            note: None,
            confirmed: true,
            partial: false,
            entries: entries
                .iter()
                .enumerate()
                .map(|(value, name)| crate::spec::EnumEntry {
                    value: u16::try_from(value).unwrap_or(0),
                    name: (*name).to_owned(),
                    description: None,
                })
                .collect(),
        }
    }

    #[test]
    fn a_table_of_divisions_is_not_a_set_of_names() {
        assert!(numeric(&table(&["1/2", "3/8", "1/16"])));
        assert!(!numeric(&table(&["16'", "8'", "4'"])));
        assert!(!numeric(&table(&["Sine", "Triangle"])));
    }

    #[test]
    fn the_method_name_is_the_parameter_name() {
        let names = ["LFO 1 Shape", "FX 1 Output Gain"];
        assert_eq!(
            snake_identifiers(names.into_iter(), "parameter"),
            Ok(vec![
                String::from("lfo1_shape"),
                String::from("fx1_output_gain")
            ])
        );
    }
}
