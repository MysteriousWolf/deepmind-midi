//! Typed view of the machine-readable specification in `spec/`.
//!
//! These files are the single source of truth for the protocol. Documentation is
//! generated from them, and the parameter tables the library will use are
//! generated from the same data, so a correction only ever has to be made once.

use std::path::Path;

use serde::Deserialize;

/// A program parameter.
///
/// `offset` is both the parameter's 14-bit NRPN number and its byte offset in an
/// unpacked program dump; on this synthesizer those are the same number.
#[derive(Debug, Deserialize)]
pub struct Parameter {
    /// NRPN number, and byte offset into unpacked program data.
    pub offset: u16,
    /// Human-readable parameter name.
    pub name: String,
    /// Section of the synthesizer this parameter belongs to.
    pub group: String,
    /// Lowest accepted value.
    pub min: u16,
    /// Highest accepted value.
    pub max: u16,
    /// `"switch"` for two-state parameters, absent otherwise.
    #[serde(default)]
    pub kind: Option<String>,
    /// Identifier of the value table in `enums.toml` that decodes this parameter.
    #[serde(default, rename = "enum")]
    pub value_table: Option<String>,
    /// Free-form note carried through from the manual.
    #[serde(default)]
    pub note: Option<String>,
    /// Why this row departs from what the manual prints.
    #[serde(default)]
    pub correction: Option<String>,
}

/// One value of an enumerated parameter.
#[derive(Debug, Deserialize)]
pub struct EnumEntry {
    /// The wire value.
    pub value: u16,
    /// Name as the synthesizer displays it.
    pub name: String,
    /// Longer explanation, where the manual gives one.
    #[serde(default)]
    pub description: Option<String>,
}

/// A named table of parameter values.
#[derive(Debug, Deserialize)]
pub struct ValueTable {
    /// Identifier referenced by [`Parameter::value_table`].
    pub id: String,
    /// Human-readable table name.
    pub name: String,
    /// Free-form note, typically firmware differences.
    #[serde(default)]
    pub note: Option<String>,
    /// `false` when the mapping is inferred and still needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
    /// `true` when the entries list only the start of a documented range, such
    /// as SPREAD-1 standing for SPREAD-1 through SPREAD-254.
    #[serde(default)]
    pub partial: bool,
    /// The values themselves.
    pub entries: Vec<EnumEntry>,
}

const fn yes() -> bool {
    true
}

/// A `SysEx` message.
#[derive(Debug, Deserialize)]
pub struct Message {
    /// Command byte, the one following the device ID.
    pub command: u8,
    /// Human-readable message name.
    pub name: String,
    /// `"to_device"` or `"from_device"`.
    pub direction: String,
    /// Summary of the bytes between the command byte and `F7`.
    #[serde(default)]
    pub payload: Option<String>,
    /// Unpacked payload length in bytes, where the message carries bulk data.
    #[serde(default)]
    pub raw_len: Option<u32>,
    /// Packed payload length in bytes, where the message carries bulk data.
    #[serde(default)]
    pub packed_len: Option<u32>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// A device-wide setting.
#[derive(Debug, Deserialize)]
pub struct Global {
    /// Human-readable setting name.
    pub name: String,
    /// Lowest accepted value.
    pub min: u16,
    /// Highest accepted value.
    pub max: u16,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Parameters {
    parameter: Vec<Parameter>,
}

#[derive(Debug, Deserialize)]
struct ValueTables {
    #[serde(rename = "enum")]
    tables: Vec<ValueTable>,
}

#[derive(Debug, Deserialize)]
struct Messages {
    message: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Globals {
    #[serde(rename = "global")]
    globals: Vec<Global>,
}

/// Everything in `spec/`, loaded and validated.
#[derive(Debug)]
pub struct Spec {
    /// Program parameters, ordered by offset.
    pub parameters: Vec<Parameter>,
    /// Value tables, ordered as written.
    pub tables: Vec<ValueTable>,
    /// `SysEx` messages, ordered by command byte.
    pub messages: Vec<Message>,
    /// Device-wide settings.
    pub globals: Vec<Global>,
}

impl Spec {
    /// Loads and validates every file under `spec/`.
    ///
    /// # Errors
    ///
    /// Returns a message naming the file and the problem when a file is missing,
    /// is not valid TOML, or fails one of the consistency checks: offsets must
    /// cover 0..=241 exactly, every referenced value table must exist, and every
    /// enumerated parameter's maximum must match its table.
    pub fn load(root: &Path) -> Result<Self, String> {
        let spec = root.join("spec");
        let parameters: Parameters = read(&spec.join("parameters.toml"))?;
        let tables: ValueTables = read(&spec.join("enums.toml"))?;
        let messages: Messages = read(&spec.join("messages.toml"))?;
        let globals: Globals = read(&spec.join("globals.toml"))?;

        let this = Self {
            parameters: parameters.parameter,
            tables: tables.tables,
            messages: messages.message,
            globals: globals.globals,
        };
        this.validate()?;
        Ok(this)
    }

    /// Returns the value table with this identifier, if it exists.
    #[must_use]
    pub fn table(&self, id: &str) -> Option<&ValueTable> {
        self.tables.iter().find(|table| table.id == id)
    }

    fn validate(&self) -> Result<(), String> {
        let offsets: Vec<u16> = self.parameters.iter().map(|p| p.offset).collect();
        let expected: Vec<u16> = (0..242).collect();
        if offsets != expected {
            return Err(format!(
                "parameters.toml: offsets must be 0..=241 in order, found {} entries starting {:?}",
                offsets.len(),
                &offsets[..offsets.len().min(5)]
            ));
        }

        for parameter in &self.parameters {
            let Some(id) = &parameter.value_table else {
                continue;
            };
            let table = self.table(id).ok_or_else(|| {
                format!(
                    "parameter {} references unknown table {id}",
                    parameter.offset
                )
            })?;
            let highest = table.entries.iter().map(|e| e.value).max().unwrap_or(0);
            // Tables whose tail is a documented range (SPREAD-n, User-n) list only
            // the first entry of that range, so only exact tables are checked.
            let contiguous = !table.partial
                && table
                    .entries
                    .iter()
                    .enumerate()
                    .all(|(i, e)| u16::try_from(i) == Ok(e.value));
            if contiguous && highest != parameter.max {
                return Err(format!(
                    "parameter {} ({}) has max {} but table {id} tops out at {highest}",
                    parameter.offset, parameter.name, parameter.max
                ));
            }
        }

        let mut commands: Vec<u8> = self.messages.iter().map(|m| m.command).collect();
        commands.sort_unstable();
        if commands.windows(2).any(|w| w[0] == w[1]) {
            // 0x08 is deliberately shared by two pattern responses.
            let duplicates: Vec<u8> = commands
                .windows(2)
                .filter(|w| w[0] == w[1])
                .map(|w| w[0])
                .filter(|c| *c != 0x08)
                .collect();
            if !duplicates.is_empty() {
                return Err(format!(
                    "messages.toml: duplicate command bytes {duplicates:02X?}"
                ));
            }
        }

        Ok(())
    }
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}
