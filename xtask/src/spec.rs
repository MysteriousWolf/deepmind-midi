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
    /// The physical range the synthesizer shows for the same raw value, where the
    /// manual states one. Hz, dB, seconds and so on.
    #[serde(default)]
    pub display: Option<String>,
    /// Why this row departs from what the manual prints.
    #[serde(default)]
    pub correction: Option<String>,
    /// `false` when the value is inferred and needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
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

/// A firmware version that changes the protocol.
#[derive(Debug, Deserialize)]
pub struct Firmware {
    /// Dotted version as a device inquiry reports it, such as `"1.1"`.
    pub version: String,
    /// `true` for the version assumed when a caller names none.
    #[serde(default)]
    pub default: bool,
    /// What this version changed.
    #[serde(default)]
    pub note: Option<String>,
}

impl Firmware {
    /// Parses a dotted version into comparable parts.
    fn parts(version: &str) -> (u32, u32) {
        let mut split = version.split('.');
        let major = split.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let minor = split.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        (major, minor)
    }

    /// Returns whether `range` covers `version`.
    ///
    /// A range is a bare version for exactly that one, a version with a
    /// trailing `+` for that one and later, or absent for every version.
    #[must_use]
    pub fn range_covers(range: Option<&str>, version: &str) -> bool {
        match range {
            None => true,
            Some(range) => match range.strip_suffix('+') {
                Some(from) => Self::parts(version) >= Self::parts(from),
                None => range == version,
            },
        }
    }
}

/// A named table of parameter values.
#[derive(Debug, Deserialize)]
pub struct ValueTable {
    /// Identifier referenced by [`Parameter::value_table`].
    pub id: String,
    /// Human-readable table name.
    pub name: String,
    /// Which firmware versions this table describes. See [`Firmware::range_covers`].
    #[serde(default)]
    pub firmware: Option<String>,
    /// Free-form note.
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

/// A MIDI continuous controller the synthesizer answers.
#[derive(Debug, Deserialize)]
pub struct Controller {
    /// Controller number, 0-127.
    pub cc: u8,
    /// What it controls.
    pub name: String,
    /// `"parameter"`, `"standard"` or `"other"`.
    pub kind: String,
    /// Offset of the program parameter it drives, when it drives one.
    #[serde(default)]
    pub parameter: Option<u16>,
    /// `false` when the assignment is inferred and needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// One parameter of one effect algorithm.
///
/// An engine holds twelve raw bytes whatever it is running; `slot` says which
/// of those twelve this is, and the meaning comes from the algorithm.
#[derive(Debug, Deserialize)]
pub struct EffectParameter {
    /// Position within the engine's twelve parameters, counting from 1.
    pub slot: u8,
    /// Short name as the synthesizer's display shows it.
    pub r#ref: String,
    /// Full parameter name.
    pub name: String,
    /// Unit of the displayed value, where there is one.
    #[serde(default)]
    pub unit: Option<String>,
    /// Lowest displayed value, for a parameter with a numeric range.
    #[serde(default)]
    pub min: Option<String>,
    /// Highest displayed value, for a parameter with a numeric range.
    #[serde(default)]
    pub max: Option<String>,
    /// Options this parameter selects between, for a parameter without a range.
    #[serde(default)]
    pub values: Option<String>,
    /// `true` when the manual marks the parameter as responding to modulation.
    ///
    /// Every slot is addressable from the modulation matrix regardless, as
    /// `Fx <engine> Param <slot>`; this says the engine acts on what arrives.
    #[serde(default)]
    pub mod_dest: bool,
    /// What the parameter does, from the manual. See NOTICE.
    #[serde(default)]
    pub description: Option<String>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// An effect algorithm and the twelve parameters it gives its engine.
#[derive(Debug, Deserialize)]
pub struct Effect {
    /// Value of the `FX Type` parameter that selects this algorithm.
    pub r#type: u16,
    /// Short name, matching the `fx_type` value table.
    pub name: String,
    /// Full name as the manual writes it.
    pub full_name: String,
    /// The parameters, ordered by slot.
    pub parameters: Vec<EffectParameter>,
}

/// How one effect slot presents itself to a host.
#[derive(Debug, Deserialize)]
pub struct PanelSlot {
    /// Position within the engine's twelve parameters, counting from 1.
    pub slot: u8,
    /// `continuous`, `switch` or `selector`.
    pub kind: String,
    /// Slots sharing a label belong together, such as one side of a dual engine.
    #[serde(default)]
    pub group: Option<String>,
    /// `true` when the engine acts on modulation reaching this slot.
    #[serde(default)]
    pub modulatable: bool,
}

/// The presentation of one effect algorithm's slots.
#[derive(Debug, Deserialize)]
pub struct Panel {
    /// Value of the `FX Type` parameter that selects this algorithm.
    pub r#type: u16,
    /// Short name, matching the effect and the `fx_type` value table.
    pub name: String,
    /// The slots, ordered.
    pub slots: Vec<PanelSlot>,
}

/// One field of a transport's byte pattern.
#[derive(Debug, Deserialize)]
pub struct MappingField {
    /// Name used by the `<placeholder>` in the pattern.
    pub name: String,
    /// Where the value comes from, such as `parameter.offset`.
    pub source: String,
    /// Identifier of the encoding applied, if any.
    #[serde(default)]
    pub encoding: Option<String>,
    /// Width in bits, where the field is a fixed-width number.
    #[serde(default)]
    pub bits: Option<u8>,
    /// `true` when the field may be left out.
    #[serde(default)]
    pub optional: bool,
    /// When the field may be left out.
    #[serde(default)]
    pub optional_when: Option<String>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// A way of carrying an address and a value over MIDI.
#[derive(Debug, Deserialize)]
pub struct Transport {
    /// Identifier, such as `nrpn`.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// One-line summary of what it reaches.
    pub summary: String,
    /// Byte pattern, with `<name>` standing for a field.
    pub pattern: String,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// The fields the pattern refers to.
    #[serde(default, rename = "field")]
    pub fields: Vec<MappingField>,
}

/// A named rule turning a value into wire bytes.
#[derive(Debug, Deserialize)]
pub struct Encoding {
    /// Identifier referenced by [`MappingField::encoding`].
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// The rule itself.
    pub rule: String,
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
    /// `false` when the range or ordering is inferred and needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
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
struct Controllers {
    controller: Vec<Controller>,
}

#[derive(Debug, Deserialize)]
struct Panels {
    panel: Vec<Panel>,
}

#[derive(Debug, Deserialize)]
struct Mapping {
    transport: Vec<Transport>,
    #[serde(rename = "encoding")]
    encodings: Vec<Encoding>,
}

#[derive(Debug, Deserialize)]
struct Firmwares {
    firmware: Vec<Firmware>,
}

#[derive(Debug, Deserialize)]
struct Effects {
    effect: Vec<Effect>,
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
    /// Continuous controllers, ordered by number.
    pub controllers: Vec<Controller>,
    /// Effect algorithms, ordered by `FX Type` value.
    pub effects: Vec<Effect>,
    /// Firmware versions that change the protocol, oldest first.
    pub firmwares: Vec<Firmware>,
    /// Ways of carrying an address and a value over MIDI.
    pub transports: Vec<Transport>,
    /// Rules turning a value into wire bytes.
    pub encodings: Vec<Encoding>,
    /// How each effect presents its slots, ordered by `FX Type` value.
    pub panels: Vec<Panel>,
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
        let controllers: Controllers = read(&spec.join("controllers.toml"))?;
        let effects: Effects = read(&spec.join("effects.toml"))?;
        let firmwares: Firmwares = read(&spec.join("firmware.toml"))?;
        let mapping: Mapping = read(&spec.join("mapping.toml"))?;
        let panels: Panels = read(&spec.join("panels.toml"))?;

        let this = Self {
            parameters: parameters.parameter,
            tables: tables.tables,
            messages: messages.message,
            globals: globals.globals,
            controllers: controllers.controller,
            effects: effects.effect,
            firmwares: firmwares.firmware,
            transports: mapping.transport,
            encodings: mapping.encodings,
            panels: panels.panel,
        };
        this.validate()?;
        Ok(this)
    }

    /// Returns the firmware version used when a caller names none.
    ///
    /// The one marked `default` in `firmware.toml`, or the last listed.
    #[must_use]
    pub fn default_firmware(&self) -> &str {
        self.firmwares
            .iter()
            .find(|f| f.default)
            .or_else(|| self.firmwares.last())
            .map_or("", |f| f.version.as_str())
    }

    /// Returns the value table with this identifier as of `firmware`.
    ///
    /// Several tables may share an identifier when firmware renumbered their
    /// entries; the one whose firmware range covers `firmware` is the right one.
    #[must_use]
    pub fn table_for(&self, id: &str, firmware: &str) -> Option<&ValueTable> {
        self.tables
            .iter()
            .find(|t| t.id == id && Firmware::range_covers(t.firmware.as_deref(), firmware))
    }

    /// Returns the value table with this identifier on the default firmware.
    #[must_use]
    pub fn table(&self, id: &str) -> Option<&ValueTable> {
        self.table_for(id, self.default_firmware())
    }

    fn validate(&self) -> Result<(), String> {
        self.validate_firmware()?;
        self.validate_parameters()?;
        self.validate_controllers()?;
        self.validate_effects()?;
        self.validate_mapping()?;
        self.validate_messages()
    }

    /// Checks that every table identifier resolves for every known firmware.
    ///
    /// A table that covers no version is unreachable; two that cover the same
    /// version make lookup depend on file order, which is how a renumbering
    /// silently goes wrong.
    fn validate_firmware(&self) -> Result<(), String> {
        if self.firmwares.is_empty() {
            return Err("firmware.toml: no versions listed".to_owned());
        }
        if self.firmwares.iter().filter(|f| f.default).count() > 1 {
            return Err("firmware.toml: more than one version marked default".to_owned());
        }

        let mut ids: Vec<&str> = self.tables.iter().map(|t| t.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        for id in ids {
            for firmware in &self.firmwares {
                let matches = self
                    .tables
                    .iter()
                    .filter(|t| {
                        t.id == id
                            && Firmware::range_covers(t.firmware.as_deref(), &firmware.version)
                    })
                    .count();
                if matches != 1 {
                    return Err(format!(
                        "enums.toml: table {id} has {matches} definitions for firmware {}, want 1",
                        firmware.version
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_parameters(&self) -> Result<(), String> {
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
            // A parameter's stated maximum is the one for the default firmware.
            // Older firmware reaches a lower maximum through its own table.
            let table = self.table(id).ok_or_else(|| {
                format!(
                    "parameter {} references unknown table {id}",
                    parameter.offset
                )
            })?;
            let highest = table.entries.iter().map(|e| e.value).max().unwrap_or(0);
            // Only exact, confirmed tables are checked. A table whose tail is a
            // documented range (SPREAD-n, User-n) lists just its first entry, and an
            // unconfirmed one is where the manual contradicts itself about the range.
            let contiguous = !table.partial
                && table.confirmed
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

        Ok(())
    }

    fn validate_controllers(&self) -> Result<(), String> {
        let mut seen_cc: Vec<u8> = Vec::new();
        for controller in &self.controllers {
            if seen_cc.contains(&controller.cc) {
                return Err(format!(
                    "controllers.toml: CC {} appears twice",
                    controller.cc
                ));
            }
            seen_cc.push(controller.cc);
            let Some(offset) = controller.parameter else {
                continue;
            };
            if !self.parameters.iter().any(|p| p.offset == offset) {
                return Err(format!(
                    "controllers.toml: CC {} maps to unknown parameter offset {offset}",
                    controller.cc
                ));
            }
            if self
                .controllers
                .iter()
                .filter(|c| c.parameter == Some(offset))
                .count()
                > 1
            {
                return Err(format!(
                    "controllers.toml: parameter offset {offset} is claimed by more than one CC"
                ));
            }
        }

        Ok(())
    }

    fn validate_effects(&self) -> Result<(), String> {
        let fx_types: Vec<u16> = self.effects.iter().map(|e| e.r#type).collect();
        let expected_types: Vec<u16> = (0..35).collect();
        if fx_types != expected_types {
            return Err(format!(
                "effects.toml: type values must be 0..=34 in order, found {} entries",
                fx_types.len()
            ));
        }
        let fx_table = self
            .table("fx_type")
            .ok_or("enums.toml: no fx_type table to check effects.toml against")?;
        for effect in &self.effects {
            let listed = fx_table
                .entries
                .iter()
                .find(|e| e.value == effect.r#type)
                .ok_or_else(|| format!("fx_type has no value {}", effect.r#type))?;
            if listed.name != effect.name {
                return Err(format!(
                    "effects.toml: type {} is {:?} but fx_type calls it {:?}",
                    effect.r#type, effect.name, listed.name
                ));
            }
            if effect.parameters.len() > 12 {
                return Err(format!(
                    "effects.toml: {} has {} parameters, an engine holds 12",
                    effect.name,
                    effect.parameters.len()
                ));
            }
            for (index, parameter) in effect.parameters.iter().enumerate() {
                if usize::from(parameter.slot) != index + 1 {
                    return Err(format!(
                        "effects.toml: {} slot {} is out of order",
                        effect.name, parameter.slot
                    ));
                }
                if parameter.values.is_none() && parameter.min.is_none() {
                    return Err(format!(
                        "effects.toml: {} slot {} has neither a range nor a value list",
                        effect.name, parameter.slot
                    ));
                }
            }
        }

        Ok(())
    }

    /// Checks that every pattern placeholder has a field and every field an
    /// encoding that exists.
    fn validate_mapping(&self) -> Result<(), String> {
        for transport in &self.transports {
            for field in &transport.fields {
                let placeholder = format!("<{}>", field.name);
                if !transport.pattern.contains(&placeholder) {
                    return Err(format!(
                        "mapping.toml: {} defines field {} which its pattern never uses",
                        transport.id, field.name
                    ));
                }
                if let Some(id) = &field.encoding
                    && !self.encodings.iter().any(|e| &e.id == id)
                {
                    return Err(format!(
                        "mapping.toml: {} field {} uses unknown encoding {id}",
                        transport.id, field.name
                    ));
                }
            }
            for placeholder in transport.pattern.split('<').skip(1) {
                let Some(name) = placeholder.split('>').next() else {
                    continue;
                };
                if !transport.fields.iter().any(|f| f.name == name) {
                    return Err(format!(
                        "mapping.toml: {} pattern uses <{name}> with no field to fill it",
                        transport.id
                    ));
                }
            }
        }
        self.validate_panels()
    }

    /// Checks that panels.toml lines up with effects.toml slot for slot.
    ///
    /// The two are generated together, so a mismatch means one was edited by
    /// hand and the other was not.
    fn validate_panels(&self) -> Result<(), String> {
        const KINDS: [&str; 3] = ["continuous", "switch", "selector"];
        if self.panels.len() != self.effects.len() {
            return Err(format!(
                "panels.toml has {} panels but effects.toml has {} effects",
                self.panels.len(),
                self.effects.len()
            ));
        }
        for (panel, effect) in self.panels.iter().zip(&self.effects) {
            if panel.r#type != effect.r#type || panel.name != effect.name {
                return Err(format!(
                    "panels.toml has {} at type {} where effects.toml has {} at type {}",
                    panel.name, panel.r#type, effect.name, effect.r#type
                ));
            }
            if panel.slots.len() != effect.parameters.len() {
                return Err(format!(
                    "panels.toml: {} has {} slots but effects.toml has {} parameters",
                    panel.name,
                    panel.slots.len(),
                    effect.parameters.len()
                ));
            }
            for (slot, parameter) in panel.slots.iter().zip(&effect.parameters) {
                if slot.slot != parameter.slot {
                    return Err(format!(
                        "panels.toml: {} slot {} does not line up with effects.toml",
                        panel.name, slot.slot
                    ));
                }
                if !KINDS.contains(&slot.kind.as_str()) {
                    return Err(format!(
                        "panels.toml: {} slot {} has unknown kind {:?}",
                        panel.name, slot.slot, slot.kind
                    ));
                }
                if slot.modulatable != parameter.mod_dest {
                    return Err(format!(
                        "panels.toml: {} slot {} disagrees with effects.toml about modulation",
                        panel.name, slot.slot
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_messages(&self) -> Result<(), String> {
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
