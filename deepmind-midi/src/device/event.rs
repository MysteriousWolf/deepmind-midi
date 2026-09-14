//! What the state machine reports back.

use core::fmt;

use crate::error::Error;
use crate::ids::Slot;
use crate::param::ParamId;
use crate::program::Program;
use crate::sysex::{Command, Identity};
use crate::wire::{Channel, ChannelMessage};

use super::Request;

/// Something that happened, waiting to be polled.
///
/// Events are queued as bytes are fed in and drained by
/// [`poll_event`](super::Device::poll_event), so a host sees them in the order
/// they happened rather than in the middle of its own call to
/// [`feed`](super::Device::feed).
///
/// A program-bearing event carries the program rather than pointing at the
/// device's copy of it. A bank transfer overwrites that copy 128 times, and an
/// event that meant "look at the current one" would be worth nothing by the
/// time anyone looked.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    /// The edit buffer arrived, and is now what
    /// [`program`](super::Device::program) confirms.
    EditBuffer(Program),
    /// A stored program arrived, and where the synthesizer said it lives.
    ///
    /// A stored program is not the sound the synthesizer is making, so it is
    /// reported and not tracked: [`program`](super::Device::program) keeps
    /// describing the edit buffer.
    Program {
        /// Bank and program number the dump names.
        slot: Slot,
        /// The program itself.
        program: Program,
    },
    /// A device inquiry was answered, which is the only thing that reports
    /// firmware.
    Identity(Identity),
    /// One parameter changed at the synthesizer, by NRPN or by its controller.
    ///
    /// Whether the tracked program still counts as confirmed after it is what
    /// [`program`](super::Device::program) says: an NRPN carries the whole
    /// value and a control change carries seven bits of it.
    Parameter {
        /// The parameter the synthesizer named.
        parameter: ParamId,
        /// The value it now holds, as this library reads the message.
        value: u16,
    },
    /// A channel message that is not a parameter edit: notes, bend, aftertouch,
    /// and the controllers that reach something other than a parameter.
    Channel {
        /// Channel it arrived on.
        channel: Channel,
        /// The message itself.
        message: ChannelMessage,
    },
    /// A request went unanswered for as long as the timeout allows.
    ///
    /// The request is dropped when this is raised. Nothing is retried: only the
    /// host knows whether asking again is the right thing to do.
    Timeout(Request),
    /// A `DeepMind` message this layer does not track.
    ///
    /// The globals, the patterns and the chord memories are payloads whose
    /// offsets the manual never publishes, so there is nothing to decode them
    /// into. A bank of program names is decodable and is two kilobytes, which is
    /// more than an event queue should carry. The command is reported so a host
    /// knows the traffic arrived.
    Unhandled(Command),
    /// A frame that could not be read, and why.
    Failed(Error),
    /// A `SysEx` frame from another manufacturer or another model.
    Foreign {
        /// The three manufacturer ID bytes the frame carried.
        manufacturer: [u8; 3],
        /// The model ID byte the frame carried.
        model: u8,
    },
    /// Events were dropped because the queue was full, and how many.
    ///
    /// Raised once the queue has room again, so the count is what was missed
    /// between the last event polled and the next one. A host seeing this is
    /// feeding more between drains than its queue holds.
    Lost(u16),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EditBuffer(program) => write!(f, "edit buffer: {}", program.name()),
            Self::Program { slot, program } => write!(f, "{slot}: {}", program.name()),
            Self::Identity(identity) => write!(f, "firmware {}", identity.firmware),
            Self::Parameter { parameter, value } => write!(f, "{parameter} = {value}"),
            Self::Channel { channel, message } => write!(f, "channel {channel}: {message:?}"),
            Self::Timeout(request) => write!(f, "no answer to the {request}"),
            Self::Unhandled(command) => write!(f, "untracked {command}"),
            Self::Failed(error) => write!(f, "unreadable frame: {error}"),
            Self::Foreign { manufacturer, .. } => {
                write!(f, "frame from manufacturer {manufacturer:02X?}")
            }
            Self::Lost(count) => write!(f, "{count} events dropped"),
        }
    }
}
