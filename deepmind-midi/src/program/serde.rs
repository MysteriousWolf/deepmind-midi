//! `Serialize` and `Deserialize` for [`Program`], written by hand because the
//! derive stops at arrays of 32.
//!
//! A program serializes as a struct of two fields: `version`, the comms
//! protocol version, and `data`, the bytes of the unpacked dump as a byte
//! string. In JSON that is an array of numbers; in a binary format it is the
//! bytes. Reading one back goes through [`Program::from_bytes`], so the length
//! rule holds for a program that came out of a file.

use core::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Program;
use crate::ids::ProtocolVersion;

const NAME: &str = "Program";
const FIELDS: &[&str] = &["version", "data"];

/// The dump bytes, serialized as a byte string rather than a sequence.
struct Bytes<'a>(&'a [u8]);

impl Serialize for Bytes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

impl Serialize for Program {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut program = serializer.serialize_struct(NAME, FIELDS.len())?;
        program.serialize_field("version", &self.version())?;
        program.serialize_field("data", &Bytes(self.as_bytes()))?;
        program.end()
    }
}

/// The dump bytes, read from a byte string or a sequence into a fixed buffer.
struct Data {
    bytes: [u8; Program::MAX_LEN],
    len: usize,
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DataVisitor;

        impl<'de> Visitor<'de> for DataVisitor {
            type Value = Data;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "at most {} bytes of program data", Program::MAX_LEN)
            }

            fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Data, E> {
                let mut data = Data {
                    bytes: [0; Program::MAX_LEN],
                    len: bytes.len(),
                };
                if bytes.len() > Program::MAX_LEN {
                    return Err(E::invalid_length(bytes.len(), &self));
                }
                for (slot, byte) in data.bytes.iter_mut().zip(bytes) {
                    *slot = *byte;
                }
                Ok(data)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Data, A::Error> {
                let mut data = Data {
                    bytes: [0; Program::MAX_LEN],
                    len: 0,
                };
                while let Some(byte) = seq.next_element::<u8>()? {
                    let Some(slot) = data.bytes.get_mut(data.len) else {
                        return Err(de::Error::invalid_length(data.len + 1, &self));
                    };
                    *slot = byte;
                    data.len += 1;
                }
                Ok(data)
            }
        }

        deserializer.deserialize_bytes(DataVisitor)
    }
}

/// Which of the two fields a key names.
enum Field {
    Version,
    Data,
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldVisitor;

        impl Visitor<'_> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("`version` or `data`")
            }

            fn visit_str<E: de::Error>(self, key: &str) -> Result<Field, E> {
                match key {
                    "version" => Ok(Field::Version),
                    "data" => Ok(Field::Data),
                    other => Err(de::Error::unknown_field(other, FIELDS)),
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)
    }
}

impl<'de> Deserialize<'de> for Program {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ProgramVisitor;

        impl<'de> Visitor<'de> for ProgramVisitor {
            type Value = Program;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a program: its comms protocol version and its bytes")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Program, A::Error> {
                let version: ProtocolVersion = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let data: Data = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                build(version, &data)
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Program, A::Error> {
                let mut version = None;
                let mut data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                    }
                }
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let data: Data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                build(version, &data)
            }
        }

        deserializer.deserialize_struct(NAME, FIELDS, ProgramVisitor)
    }
}

/// Builds the program, turning a wrong length into the format's own error.
fn build<E: de::Error>(version: ProtocolVersion, data: &Data) -> Result<Program, E> {
    let bytes = data.bytes.get(..data.len).unwrap_or_default();
    Program::from_bytes(version, bytes).map_err(E::custom)
}
