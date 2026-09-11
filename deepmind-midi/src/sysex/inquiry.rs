//! The universal device inquiry, and the firmware versions it answers with.
//!
//! ```text
//! request:  F0 7E <dev|7F> 06 01 F7
//! response: F0 7E <dev> 06 02 00 20 32 20 00 01 00 <jn> 00 <ii> <nn> F7
//! ```
//!
//! This is not a `DeepMind` message and does not use the `DeepMind` framing: it
//! is the MIDI non-realtime identity request, which is the only way to find a
//! unit whose device ID is unknown and the only documented way to read firmware
//! versions. The firmware version is not the comms protocol version and the two
//! move independently; see [`crate::ids::ProtocolVersion`].

use core::fmt;

use crate::error::{Error, Result};
use crate::ids::{DeviceId, MANUFACTURER_ID, Model};
use crate::wire::{SYSEX_END, SYSEX_START};

/// Universal non-realtime `SysEx` ID.
pub const NON_REALTIME: u8 = 0x7E;

/// General information sub-ID.
pub const GENERAL_INFORMATION: u8 = 0x06;

/// Identity request sub-ID.
pub const IDENTITY_REQUEST: u8 = 0x01;

/// Identity reply sub-ID.
pub const IDENTITY_REPLY: u8 = 0x02;

/// Device family the `DeepMind` reports, little end first.
pub const FAMILY: [u8; 2] = [0x20, 0x00];

/// Family member the `DeepMind` reports, little end first.
pub const MEMBER: [u8; 2] = [0x01, 0x00];

/// Length of an identity request.
pub const REQUEST_LEN: usize = 6;

/// Length of an identity reply.
pub const REPLY_LEN: usize = 17;

/// A software version as the identity reply reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
}

impl Version {
    /// Reads the main software revision out of its packed byte.
    ///
    /// The manual writes that byte as `jn`, with `j` the minor revision and `n`
    /// the major: firmware 1.1 is `0x11` and firmware 1.0 is `0x01`.
    #[must_use]
    pub const fn from_packed(byte: u8) -> Self {
        Self {
            major: byte & 0x0F,
            minor: byte >> 4,
        }
    }

    /// Returns whether this version is `major.minor` or later.
    ///
    /// Value tables in the parameter layer declare the firmware they describe as
    /// a version and later, so selecting one is this comparison.
    #[must_use]
    pub const fn at_least(self, major: u8, minor: u8) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }

    /// Returns whether this version is exactly `major.minor`.
    #[must_use]
    pub const fn is(self, major: u8, minor: u8) -> bool {
        self.major == major && self.minor == minor
    }

    /// Returns the packed byte for this version.
    #[must_use]
    pub const fn to_packed(self) -> u8 {
        ((self.minor & 0x07) << 4) | (self.major & 0x0F)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// What a unit answers a device inquiry with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    /// Device ID the unit answered on, which is also its global MIDI channel.
    pub device: DeviceId,
    /// Main software revision, the one `spec/firmware.toml` describes.
    pub firmware: Version,
    /// Voice software revision, which the manual reports separately.
    pub voice: Version,
}

/// Builds an identity request for `device`.
///
/// [`DeviceId::Broadcast`] asks every unit on the port, which is how a host
/// finds a synthesizer whose device ID it does not know.
///
/// # Errors
///
/// Returns [`Error::BufferTooSmall`] when `out` is shorter than
/// [`REQUEST_LEN`].
///
/// ```
/// use deepmind_midi::ids::DeviceId;
/// use deepmind_midi::sysex::inquiry;
///
/// let mut bytes = [0u8; inquiry::REQUEST_LEN];
/// inquiry::request_into(DeviceId::Broadcast, &mut bytes).expect("the buffer fits");
/// assert_eq!(bytes, [0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]);
/// ```
pub fn request_into(device: DeviceId, out: &mut [u8]) -> Result<usize> {
    let available = out.len();
    let out = out.get_mut(..REQUEST_LEN).ok_or(Error::BufferTooSmall {
        needed: REQUEST_LEN,
        available,
    })?;
    let bytes = [
        SYSEX_START,
        NON_REALTIME,
        device.to_byte(),
        GENERAL_INFORMATION,
        IDENTITY_REQUEST,
        SYSEX_END,
    ];
    for (slot, byte) in out.iter_mut().zip(bytes) {
        *slot = byte;
    }
    Ok(REQUEST_LEN)
}

/// Reads an identity reply.
///
/// # Errors
///
/// Returns [`Error::Unframed`] when the `F0` or the `F7` is missing, and
/// [`Error::NotAnInquiryResponse`] when the frame is some other universal
/// message, or an identity reply from something that is not a `DeepMind`.
pub fn parse_reply(bytes: &[u8]) -> Result<Identity> {
    let (first, rest) = bytes.split_first().ok_or(Error::Unframed)?;
    let (last, body) = rest.split_last().ok_or(Error::Unframed)?;
    if *first != SYSEX_START || *last != SYSEX_END {
        return Err(Error::Unframed);
    }
    let &[
        NON_REALTIME,
        device,
        GENERAL_INFORMATION,
        IDENTITY_REPLY,
        manufacturer_0,
        manufacturer_1,
        manufacturer_2,
        family_0,
        family_1,
        member_0,
        member_1,
        firmware,
        _reserved,
        voice_major,
        voice_minor,
    ] = body
    else {
        return Err(Error::NotAnInquiryResponse);
    };
    if [manufacturer_0, manufacturer_1, manufacturer_2] != MANUFACTURER_ID
        || [family_0, family_1] != FAMILY
        || [member_0, member_1] != MEMBER
    {
        return Err(Error::NotAnInquiryResponse);
    }
    Ok(Identity {
        device: DeviceId::from_byte(device)?,
        firmware: Version::from_packed(firmware),
        voice: Version {
            major: voice_major,
            minor: voice_minor,
        },
    })
}

/// Writes the identity reply a unit would send.
///
/// Here so a test harness or a fake synthesizer can answer an inquiry; a host
/// has no reason to send one.
///
/// # Errors
///
/// Returns [`Error::BufferTooSmall`] when `out` is shorter than [`REPLY_LEN`].
pub fn reply_into(identity: &Identity, out: &mut [u8]) -> Result<usize> {
    let available = out.len();
    let out = out.get_mut(..REPLY_LEN).ok_or(Error::BufferTooSmall {
        needed: REPLY_LEN,
        available,
    })?;
    let bytes = [
        SYSEX_START,
        NON_REALTIME,
        identity.device.to_byte(),
        GENERAL_INFORMATION,
        IDENTITY_REPLY,
        MANUFACTURER_ID[0],
        MANUFACTURER_ID[1],
        MANUFACTURER_ID[2],
        FAMILY[0],
        FAMILY[1],
        MEMBER[0],
        MEMBER[1],
        identity.firmware.to_packed(),
        0x00,
        identity.voice.major,
        identity.voice.minor,
        SYSEX_END,
    ];
    for (slot, byte) in out.iter_mut().zip(bytes) {
        *slot = byte;
    }
    Ok(REPLY_LEN)
}

/// Returns the model every `DeepMind` reports, which is every variant at once.
///
/// The identity reply carries the shared model ID and nothing that separates a
/// `DeepMind` 6 from a 12XD, so there is nothing to return but the list. A host
/// that needs to know asks the user.
#[must_use]
pub const fn models() -> [Model; 6] {
    Model::ALL
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        Identity {
            device: DeviceId::Unit(3),
            firmware: Version { major: 1, minor: 1 },
            voice: Version { major: 1, minor: 0 },
        }
    }

    #[test]
    fn a_reply_round_trips() {
        let mut bytes = [0u8; REPLY_LEN];
        let len = reply_into(&identity(), &mut bytes).expect("the buffer fits");
        assert_eq!(len, REPLY_LEN);
        assert_eq!(parse_reply(&bytes), Ok(identity()));
    }

    #[test]
    fn the_manuals_own_byte_layout_parses() {
        // F0 7E dev 06 02 00 20 32 20 00 01 00 jn 00 ii nn F7, with jn = 0x11.
        let bytes = [
            0xF0, 0x7E, 0x00, 0x06, 0x02, 0x00, 0x20, 0x32, 0x20, 0x00, 0x01, 0x00, 0x11, 0x00,
            0x01, 0x00, 0xF7,
        ];
        let identity = parse_reply(&bytes).expect("the manual's own bytes");
        assert_eq!(identity.device, DeviceId::Unit(0));
        assert_eq!(identity.firmware.major, 1);
        assert_eq!(identity.firmware.minor, 1);
    }

    #[test]
    fn firmware_versions_pack_the_minor_revision_above_the_major() {
        assert_eq!(Version::from_packed(0x01).to_string(), "1.0");
        assert_eq!(Version::from_packed(0x11).to_string(), "1.1");
        for minor in 0..=7 {
            for major in 0..=15 {
                let version = Version { major, minor };
                assert_eq!(Version::from_packed(version.to_packed()), version);
            }
        }
    }

    #[test]
    fn another_manufacturers_identity_reply_is_not_ours() {
        let mut bytes = [0u8; REPLY_LEN];
        reply_into(&identity(), &mut bytes).expect("the buffer fits");
        let mut foreign = bytes;
        if let Some(slot) = foreign.get_mut(5) {
            *slot = 0x41;
        }
        assert_eq!(parse_reply(&foreign), Err(Error::NotAnInquiryResponse));

        // A reply one byte short is not one either.
        assert_eq!(
            parse_reply(bytes.get(..REPLY_LEN - 1).expect("a shorter reply")),
            Err(Error::Unframed)
        );
    }

    #[test]
    fn a_request_addresses_one_unit_or_all_of_them() {
        let mut bytes = [0u8; REQUEST_LEN];
        request_into(DeviceId::Unit(9), &mut bytes).expect("the buffer fits");
        assert_eq!(bytes, [0xF0, 0x7E, 0x09, 0x06, 0x01, 0xF7]);
        let mut small = [0u8; 5];
        assert_eq!(
            request_into(DeviceId::Broadcast, &mut small),
            Err(Error::BufferTooSmall {
                needed: 6,
                available: 5
            })
        );
    }
}
