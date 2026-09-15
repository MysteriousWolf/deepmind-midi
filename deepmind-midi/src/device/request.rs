//! What a host asked for, and the outbound items that carry the asking.

use core::fmt;

use crate::error::Result;
use crate::ids::{Bank, DeviceId, ProgramNumber, Slot};
use crate::param::NrpnEdit;
use crate::sysex::{Frame, Message, inquiry};
use crate::wire::Channel;

/// Bytes the longest outbound item encodes to.
///
/// The requests this layer sends carry at most three payload bytes, and an NRPN
/// edit is four control changes. Nothing here sends a bulk payload: a dump
/// travels through [`syx`](crate::syx) or a frame the host builds itself.
pub const MAX_OUTBOUND_LEN: usize = 16;

/// Something asked of the synthesizer that an answer is expected for.
///
/// A request is outstanding from the moment its bytes leave
/// [`drain_tx`](super::Device::drain_tx) until the answer arrives or the
/// timeout passes. Queuing one does not start the clock, because queued bytes
/// are not sent bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Request {
    /// A device inquiry, answered by an identity.
    Identity,
    /// An announcement to the synthesizer, answered by what the interface
    /// currently holds.
    ///
    /// The one request whose answer names the selected program. What a unit
    /// does with the announcement itself, and whether it answers at all, is
    /// open until one is on the bench.
    ControlApp,
    /// The edit buffer, answered by one dump.
    EditBuffer,
    /// One stored program, answered by one dump.
    Program(Slot),
    /// A run of stored programs, answered by one dump each.
    ///
    /// The run is outstanding until the dump for `last` arrives. Every dump
    /// inside it is progress, so the timeout measures the gap between dumps
    /// rather than the length of the whole transfer.
    Bank {
        /// Bank being read.
        bank: Bank,
        /// First program of the run.
        first: ProgramNumber,
        /// Last program of the run, whose dump ends the request.
        last: ProgramNumber,
    },
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity => f.write_str("device inquiry"),
            Self::ControlApp => f.write_str("control app notification"),
            Self::EditBuffer => f.write_str("edit buffer dump"),
            Self::Program(slot) => write!(f, "program dump {slot}"),
            Self::Bank { bank, first, last } => {
                write!(f, "program dumps {bank}{first} through {bank}{last}")
            }
        }
    }
}

/// One thing waiting to go out, in the order it was queued.
///
/// Small by construction. A whole-program edit is 242 of these, so the queue
/// pays for what it holds, and holding a packed dump here would cost every host
/// three hundred bytes for something only some of them send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outbound {
    /// A request, which becomes outstanding once it is sent.
    Ask(Request),
    /// One parameter edit, which nothing answers.
    Edit(NrpnEdit),
}

impl Outbound {
    /// Returns the request this item starts, for the items that start one.
    pub(super) const fn request(self) -> Option<Request> {
        match self {
            Self::Ask(request) => Some(request),
            Self::Edit(_) => None,
        }
    }

    /// Writes the item into `out` and returns how many bytes it used.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`](crate::Error::BufferTooSmall) for a
    /// buffer shorter than [`MAX_OUTBOUND_LEN`].
    pub(super) fn encode_into(
        self,
        device: DeviceId,
        channel: Channel,
        out: &mut [u8],
    ) -> Result<usize> {
        let message = match self {
            Self::Edit(edit) => return edit.encode_into(channel, out),
            Self::Ask(Request::Identity) => return inquiry::request_into(device, out),
            // The manual prints one reserved payload byte and does not say what
            // it is for, so a host announcing itself sends a zero.
            Self::Ask(Request::ControlApp) => Message::ControlAppNotifyRequest { reserved: 0 },
            Self::Ask(Request::EditBuffer) => Message::EditBufferDumpRequest,
            Self::Ask(Request::Program(slot)) => Message::ProgramDumpRequest {
                bank: slot.bank,
                program: slot.number,
            },
            Self::Ask(Request::Bank { bank, first, last }) => {
                Message::ProgramBankDumpRequest { bank, first, last }
            }
        };
        Frame::new(device, message).encode_into(out)
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_OUTBOUND_LEN, Outbound, Request};
    use crate::ids::{Bank, DeviceId, ProgramNumber, Slot};
    use crate::param::{NrpnEdit, ParamId};
    use crate::wire::Channel;

    fn encoded(item: Outbound) -> usize {
        let mut out = [0; MAX_OUTBOUND_LEN];
        item.encode_into(DeviceId::Unit(0), Channel::ONE, &mut out)
            .unwrap_or(0)
    }

    #[test]
    fn every_outbound_item_fits_the_scratch_buffer() {
        let slot = Slot::new(Bank::H, ProgramNumber::LAST);
        let items = [
            Outbound::Ask(Request::Identity),
            Outbound::Ask(Request::ControlApp),
            Outbound::Ask(Request::EditBuffer),
            Outbound::Ask(Request::Program(slot)),
            Outbound::Ask(Request::Bank {
                bank: Bank::H,
                first: ProgramNumber::FIRST,
                last: ProgramNumber::LAST,
            }),
            Outbound::Edit(NrpnEdit {
                parameter: ParamId::Lfo1Rate,
                value: 127,
            }),
        ];
        for item in items {
            let len = encoded(item);
            assert!(len > 0, "{item:?} did not encode");
            assert!(len <= MAX_OUTBOUND_LEN, "{item:?} needed {len} bytes");
        }
    }

    #[test]
    fn only_a_request_starts_one() {
        assert_eq!(
            Outbound::Ask(Request::EditBuffer).request(),
            Some(Request::EditBuffer)
        );
        assert_eq!(
            Outbound::Edit(NrpnEdit {
                parameter: ParamId::Lfo1Rate,
                value: 0
            })
            .request(),
            None
        );
    }

    #[test]
    fn an_edit_buffer_request_is_the_frame_the_manual_prints() {
        let mut out = [0; MAX_OUTBOUND_LEN];
        let len = Outbound::Ask(Request::EditBuffer)
            .encode_into(DeviceId::Unit(0), Channel::ONE, &mut out)
            .unwrap_or(0);
        assert_eq!(
            out.get(..len),
            Some(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7][..])
        );
    }
}
