//! What the synthesizer answers a control application with.

use core::fmt;

use crate::ids::{Bank, ProgramNumber, Slot};
use crate::sysex::Interface;
use crate::wire::Channel;

/// What the synthesizer reports about the interface a host is talking to it on.
///
/// The only message that says **which program the synthesizer currently has
/// selected**, and it carries the two MIDI channels alongside it. Everything
/// else a host learns about the instrument's state it either asked for by slot
/// or inferred.
///
/// This is reported and not tracked. The manual does not settle whether the
/// reported program is the sound in the edit buffer or only the last one
/// recalled, nor whether a unit announces a later change. So
/// [`Device`](super::Device) hands the answer over as it arrived and leaves the
/// host to put it beside the edit buffer it read.
///
/// # Channels
///
/// The two channel bytes are counted differently, as the manual prints them,
/// and are kept that way rather than reconciled: the receive channel runs 0 to
/// 16 with zero meaning omni, and the transmit channel runs 0 to 15.
/// [`receive_channel`](Self::receive_channel) and
/// [`transmit_channel`](Self::transmit_channel) are the readings that account
/// for the difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ControlApp {
    /// Receive channel as the manual prints it: 0 for omni, 1 through 16
    /// otherwise.
    pub rx_channel: u8,
    /// Transmit channel as the manual prints it: 0 through 15, the wire value.
    pub tx_channel: u8,
    /// Interface this answer describes.
    pub interface: Interface,
    /// Bank the synthesizer names as selected.
    pub bank: Bank,
    /// Program the synthesizer names as selected.
    pub program: ProgramNumber,
}

impl ControlApp {
    /// Returns the slot the synthesizer names as selected.
    #[must_use]
    pub const fn slot(&self) -> Slot {
        Slot::new(self.bank, self.program)
    }

    /// Returns whether the synthesizer receives on every channel.
    ///
    /// Which is what a receive channel of zero means.
    #[must_use]
    pub const fn is_omni(&self) -> bool {
        self.rx_channel == 0
    }

    /// Returns the channel the synthesizer receives on.
    ///
    /// `None` when it answered omni, and when the byte is outside the 0 to 16
    /// the manual prints, which is a reading this library will not invent a
    /// channel for. [`is_omni`](Self::is_omni) tells the two apart.
    #[must_use]
    pub fn receive_channel(&self) -> Option<Channel> {
        self.rx_channel
            .checked_sub(1)
            .and_then(|index| Channel::new(index).ok())
    }

    /// Returns the channel the synthesizer transmits on.
    ///
    /// `None` for a byte outside the 0 to 15 the manual prints.
    #[must_use]
    pub fn transmit_channel(&self) -> Option<Channel> {
        Channel::new(self.tx_channel).ok()
    }
}

impl fmt::Display for ControlApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} on {}, receiving ", self.slot(), self.interface)?;
        match self.receive_channel() {
            Some(channel) => write!(f, "on {channel}")?,
            None if self.is_omni() => f.write_str("on every channel")?,
            None => write!(f, "on channel byte {}", self.rx_channel)?,
        }
        match self.transmit_channel() {
            Some(channel) => write!(f, " and transmitting on {channel}"),
            None => write!(f, " and transmitting on channel byte {}", self.tx_channel),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ControlApp;
    use crate::ids::{Bank, ProgramNumber};
    use crate::sysex::Interface;
    use crate::wire::Channel;

    /// Bank C, which the crate names by letter rather than by constant.
    const BANK_C: Bank = match Bank::new(2) {
        Ok(bank) => bank,
        Err(_) => Bank::A,
    };

    fn answer(rx_channel: u8, tx_channel: u8) -> ControlApp {
        ControlApp {
            rx_channel,
            tx_channel,
            interface: Interface::Usb,
            bank: BANK_C,
            program: ProgramNumber::FIRST,
        }
    }

    /// The two bytes are counted differently, which is the whole reason these
    /// readings exist: the same channel 1 is a 1 in one of them and a 0 in the
    /// other.
    #[test]
    fn the_two_channel_bytes_are_read_the_way_the_manual_prints_them() {
        let one = answer(1, 0);
        assert_eq!(one.receive_channel(), Some(Channel::ONE));
        assert_eq!(one.transmit_channel(), Some(Channel::ONE));
        assert!(!one.is_omni());

        let sixteen = answer(16, 15);
        assert_eq!(sixteen.receive_channel(), Some(Channel::SIXTEEN));
        assert_eq!(sixteen.transmit_channel(), Some(Channel::SIXTEEN));
    }

    #[test]
    fn omni_is_not_a_channel_and_says_so() {
        let omni = answer(0, 0);
        assert!(omni.is_omni());
        assert_eq!(omni.receive_channel(), None);
    }

    /// A byte past the printed range is refused rather than folded into a
    /// channel, which would be this layer inventing an answer.
    #[test]
    fn a_byte_past_the_printed_range_is_no_channel_at_all() {
        let nonsense = answer(17, 16);
        assert_eq!(nonsense.receive_channel(), None);
        assert!(!nonsense.is_omni());
        assert_eq!(nonsense.transmit_channel(), None);
    }

    #[test]
    fn the_slot_is_the_bank_and_program_together() {
        let answer = answer(1, 0);
        assert_eq!(answer.slot().bank, BANK_C);
        assert_eq!(answer.slot().number, ProgramNumber::FIRST);
    }
}
