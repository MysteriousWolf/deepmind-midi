//! The NRPN registers, which both ends of the conversation keep.
//!
//! A parameter is selected by one pair of controllers and then written by
//! another, and the selection stays in force, so a knob sweep is one selection
//! and a run of values. Both the synthesizer and the host tracking it have to
//! read that the same way, which is why this is here rather than in either of
//! them.

use crate::param::{
    Controller, DATA_ENTRY_LSB, DATA_ENTRY_MSB, NRPN_NUMBER_LSB, NRPN_NUMBER_MSB, ParamId,
};

/// What one control change turned out to mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Change {
    /// The controller reaches something other than a parameter.
    Ignored,
    /// Part of the parameter protocol, with nothing completed yet: a selection,
    /// or the top half of a value still waiting for its bottom half.
    Pending,
    /// A parameter now holds a value.
    Parameter {
        /// The parameter named.
        parameter: ParamId,
        /// The value it now holds.
        value: u16,
        /// Whether the message carried the whole value. An NRPN does; a control
        /// change carries seven bits of it, which for most parameters is less
        /// than the value has.
        exact: bool,
    },
}

/// The three registers a parameter edit is assembled from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Nrpn {
    number_msb: Option<u8>,
    number_lsb: Option<u8>,
    data_msb: Option<u8>,
}

impl Nrpn {
    /// Registers with nothing selected and nothing held.
    pub(crate) const fn new() -> Self {
        Self {
            number_msb: None,
            number_lsb: None,
            data_msb: None,
        }
    }

    /// Returns the parameter currently selected, where one is and it names one.
    fn parameter(self) -> Option<ParamId> {
        let (msb, lsb) = (self.number_msb?, self.number_lsb?);
        let number = (u16::from(msb) << 7) | u16::from(lsb);
        ParamId::from_offset(u8::try_from(number).ok()?).ok()
    }

    /// Takes one control change and says what it meant.
    pub(crate) fn control_change(&mut self, controller: u8, value: u8) -> Change {
        match controller {
            NRPN_NUMBER_MSB => {
                self.number_msb = Some(value);
                self.data_msb = None;
                Change::Pending
            }
            NRPN_NUMBER_LSB => {
                self.number_lsb = Some(value);
                self.data_msb = None;
                Change::Pending
            }
            // Held until its other half arrives: the value is fourteen bits and
            // this is the top seven of them.
            DATA_ENTRY_MSB => {
                self.data_msb = Some(value);
                Change::Pending
            }
            DATA_ENTRY_LSB => self.data_entry(value),
            _ => match Controller::for_cc(controller).and_then(|control| control.parameter) {
                Some(parameter) => Change::Parameter {
                    parameter,
                    value: parameter.from_cc_value(value),
                    exact: false,
                },
                None => Change::Ignored,
            },
        }
    }

    /// Completes an edit with the bottom seven bits of its value.
    fn data_entry(&mut self, lsb: u8) -> Change {
        let Some(parameter) = self.parameter() else {
            return Change::Pending;
        };
        // Both the selection and the top half stay in force, as MIDI data entry
        // has them: a sweep is one selection, one top half and a run of bottom
        // halves. Reselecting is what clears the held half.
        let msb = self.data_msb.unwrap_or(0);
        Change::Parameter {
            parameter,
            value: (u16::from(msb) << 7) | u16::from(lsb),
            exact: true,
        }
    }
}

#[cfg(test)]
#[expect(clippy::panic, reason = "a failed expectation is the test failure")]
mod tests {
    use super::{Change, Nrpn};
    use crate::param::{DATA_ENTRY_LSB, DATA_ENTRY_MSB, NRPN_NUMBER_LSB, NRPN_NUMBER_MSB, ParamId};

    /// Selects `parameter` and writes `value`, returning what each step meant.
    fn edit(nrpn: &mut Nrpn, parameter: ParamId, value: u16) -> [Change; 4] {
        let offset = u16::from(parameter.offset());
        [
            nrpn.control_change(NRPN_NUMBER_MSB, u8::try_from(offset >> 7).unwrap_or(0)),
            nrpn.control_change(NRPN_NUMBER_LSB, u8::try_from(offset & 0x7F).unwrap_or(0)),
            nrpn.control_change(
                DATA_ENTRY_MSB,
                u8::try_from((value >> 7) & 0x7F).unwrap_or(0),
            ),
            nrpn.control_change(DATA_ENTRY_LSB, u8::try_from(value & 0x7F).unwrap_or(0)),
        ]
    }

    #[test]
    fn a_selection_and_a_value_make_one_edit() {
        let mut nrpn = Nrpn::default();
        let steps = edit(&mut nrpn, ParamId::Lfo1Rate, 200);

        assert_eq!(steps[0], Change::Pending);
        assert_eq!(steps[1], Change::Pending);
        assert_eq!(steps[2], Change::Pending);
        assert_eq!(
            steps[3],
            Change::Parameter {
                parameter: ParamId::Lfo1Rate,
                value: 200,
                exact: true,
            }
        );
    }

    #[test]
    fn the_selection_stays_in_force_for_a_sweep() {
        let mut nrpn = Nrpn::default();
        edit(&mut nrpn, ParamId::Lfo1Rate, 0);

        // A sweep sends values without reselecting.
        for value in 1..=5 {
            assert_eq!(nrpn.control_change(DATA_ENTRY_MSB, 0), Change::Pending);
            assert_eq!(
                nrpn.control_change(DATA_ENTRY_LSB, value),
                Change::Parameter {
                    parameter: ParamId::Lfo1Rate,
                    value: u16::from(value),
                    exact: true,
                }
            );
        }
    }

    #[test]
    fn a_data_entry_msb_stays_in_force_for_a_run_of_lsbs() {
        let mut nrpn = Nrpn::default();
        edit(&mut nrpn, ParamId::Lfo1Rate, 0);

        assert_eq!(nrpn.control_change(DATA_ENTRY_MSB, 1), Change::Pending);
        for lsb in [0, 5, 10] {
            assert_eq!(
                nrpn.control_change(DATA_ENTRY_LSB, lsb),
                Change::Parameter {
                    parameter: ParamId::Lfo1Rate,
                    value: 128 + u16::from(lsb),
                    exact: true,
                }
            );
        }
    }

    #[test]
    fn a_value_with_no_selection_completes_nothing() {
        let mut nrpn = Nrpn::default();
        assert_eq!(nrpn.control_change(DATA_ENTRY_LSB, 64), Change::Pending);
    }

    #[test]
    fn a_controller_that_reaches_no_parameter_is_ignored() {
        let mut nrpn = Nrpn::default();
        // Bank select reaches the bank, not a parameter.
        assert_eq!(nrpn.control_change(0, 1), Change::Ignored);
    }

    #[test]
    fn a_controller_carries_seven_bits_of_its_parameter() {
        let Some(controller) = ParamId::Lfo1Rate.controller() else {
            panic!("LFO 1 rate has a controller");
        };
        let mut nrpn = Nrpn::default();
        assert_eq!(
            nrpn.control_change(controller.cc, 127),
            Change::Parameter {
                parameter: ParamId::Lfo1Rate,
                value: ParamId::Lfo1Rate.from_cc_value(127),
                exact: false,
            }
        );
    }

    /// Either half of a reselection has to clear the held value, or the next
    /// edit is assembled out of the top of one and the bottom of another, which
    /// is a parameter nobody set.
    ///
    /// Sent one selector at a time on purpose: a reselection that sends both
    /// overwrites the held half on the second byte and hides a missing clear on
    /// the first.
    #[test]
    fn either_half_of_a_reselection_drops_a_half_written_value() {
        let offset = ParamId::Lfo1Rate.offset();
        for (selector, half) in [
            (NRPN_NUMBER_MSB, (offset >> 7) & 0x7F),
            (NRPN_NUMBER_LSB, offset & 0x7F),
        ] {
            let mut nrpn = Nrpn::default();
            edit(&mut nrpn, ParamId::Lfo1Rate, 0);
            // The top seven bits of a value that is never finished.
            assert_eq!(nrpn.control_change(DATA_ENTRY_MSB, 1), Change::Pending);

            assert_eq!(nrpn.control_change(selector, half), Change::Pending);

            assert_eq!(
                nrpn.control_change(DATA_ENTRY_LSB, 5),
                Change::Parameter {
                    parameter: ParamId::Lfo1Rate,
                    value: 5,
                    exact: true,
                },
                "controller {selector} left a half-written value behind"
            );
        }
    }
}
