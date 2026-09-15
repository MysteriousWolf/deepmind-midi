//! The front of the synthesizer: which parameters have a physical control, what
//! is printed over them, and which of the panel's two rows they are in.
//!
//! [`param`](crate::param) says what exists, [`Controller`](crate::param::Controller)
//! says what has a CC, and [`effect`](crate::effect) says where an *effect's*
//! slots sit on its own editor panel. None of the three says that
//! [`ParamId::VcfFrequency`] has a fader on the instrument with `FREQ`
//! silkscreened over it, that [`ParamId::UnisonDetune`] is under `POLY`, or that
//! the arpeggiator's two faders are `RATE` and `GATE TIME`. This is that table:
//! the one fact about this synthesizer a person can see in a photograph.
//!
//! ```
//! use deepmind_midi::front::{Shape, sections};
//! use deepmind_midi::param::{Group, ParamId};
//!
//! let vcf = sections().iter().find(|s| s.name() == "VCF").expect("a filter");
//! assert_eq!(vcf.group(), Group::Vcf);
//! assert_eq!(vcf.row(), 1);
//!
//! let frequency = vcf.controls().first().expect("five faders and a button");
//! assert_eq!(frequency.parameter(), ParamId::VcfFrequency);
//! assert_eq!(frequency.legend(), "FREQ");
//! assert_eq!(frequency.shape(), Shape::Fader);
//! ```
//!
//! # Why it is here rather than in each host
//!
//! Three things, and each of them is something every host would otherwise
//! invent separately and differently.
//!
//! **The legend is not the parameter's name.** A silkscreen has room for `KYBD`
//! and `RES`; the parameter table says `VCF Keyboard Tracking` and `VCF
//! Resonance`, which is right for a rack slot and does not fit over a fader.
//!
//! **The shape is not derivable.** The instrument puts [`ParamId::ArpOnOff`] in
//! a row of buttons and [`ParamId::ArpRateTempo`] under a fader, and both are a
//! [`Kind`](crate::param::Kind) to the parameter table — which says how a byte
//! is read, not what a hand touches.
//!
//! **A renamed parameter is a compile error.** The table is [`ParamId`]s, so a
//! later reading of the manual that moves a parameter between groups fails the
//! specification's own checks rather than mislabelling somebody's fader.
//!
//! # What it does not carry
//!
//! No pixels. Which row, which order within a section, and what is printed over
//! each control are facts off the instrument; how wide a lane is and how long a
//! fader runs are the host's, exactly as [`effect::grid`](crate::effect::grid)
//! splits the grid from the drawing.
//!
//! Not the controls that are not parameters. The panel's `DATA ENTRY` fader
//! edits whatever the display is showing, the large encoder selects programs,
//! the row of twelve lamps over `POLY` counts the voices that are sounding, and
//! the `EDIT` buttons open a section on the display. None of them addresses a
//! program byte, and a table that carried them would be describing a workflow
//! rather than a sound.
//!
//! # One instrument
//!
//! Read off a `DeepMind` 12. The 6 has the same 242 parameters and its own
//! front, and a variant gets its own table when somebody has one in front of
//! them — the way a value table gets its own firmware range. One documented
//! panel is worth more than three inferred ones.

mod generated;

pub use generated::{PANEL_CONTROL_COUNT, PANEL_ROWS, SECTION_COUNT};

use generated::SECTIONS;

use core::fmt;

use crate::param::{Group, ParamId};

/// Returns every group of the front panel, in the order the instrument prints
/// them.
///
/// Row 0 is what a player reaches for between notes: the arpeggiator, the two
/// LFOs, and the voicing, with the display between them. Row 1 is the voice
/// itself, left to right in the order the signal takes it.
#[must_use]
pub fn sections() -> &'static [Section; SECTION_COUNT] {
    &SECTIONS
}

/// One group of controls as the front panel prints it: `VCF`, `ARP / SEQ`,
/// `ENVELOPES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Section {
    name: &'static str,
    row: u8,
    group: Group,
    note: Option<&'static str>,
    controls: &'static [Control],
}

impl Section {
    /// Returns the name across the top of the plate, as the silkscreen has it.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns which of the panel's rows this plate is in, counting from 0.
    ///
    /// [`PANEL_ROWS`] of them.
    #[must_use]
    pub const fn row(&self) -> u8 {
        self.row
    }

    /// Returns the group of the parameter table its controls belong to, which
    /// is where the rest of this section's parameters are.
    ///
    /// Not a key: the instrument gives the high-pass a plate of its own and the
    /// parameter table keeps its two parameters in the filter's group, so `VCF`
    /// and `HPF` both answer [`Group::Vcf`].
    #[must_use]
    pub const fn group(&self) -> Group {
        self.group
    }

    /// Returns the controls this plate carries, in the order the panel puts
    /// them.
    #[must_use]
    pub const fn controls(&self) -> &'static [Control] {
        self.controls
    }

    /// Returns what the specification records about this plate beyond its
    /// controls, where there is anything.
    ///
    /// Which fader of the instrument's is missing here and why, or which of
    /// three envelopes the shared faders address.
    #[must_use]
    pub const fn note(&self) -> Option<&'static str> {
        self.note
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// One control the front panel puts under a legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Control {
    parameter: ParamId,
    legend: &'static str,
    shape: Shape,
}

impl Control {
    /// Returns the parameter it moves.
    #[must_use]
    pub const fn parameter(&self) -> ParamId {
        self.parameter
    }

    /// Returns what the panel prints over it: `KYBD`, `PITCH MOD`, `A`.
    ///
    /// The instrument's own word, in caps, and not the parameter's name. Two
    /// words are printed on two lines, which is what the panel does with
    /// `PITCH MOD`.
    #[must_use]
    pub const fn legend(&self) -> &'static str {
        self.legend
    }

    /// Returns what a hand touches.
    #[must_use]
    pub const fn shape(&self) -> Shape {
        self.shape
    }
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.legend)
    }
}

/// What the front panel puts a parameter under.
///
/// Not [`Kind`](crate::param::Kind), which says how a byte is read. A switch is
/// a button here and a sweep is a fader, but the instrument also gives an LFO's
/// shape a column of lit legends rather than a list on its display, and nothing
/// about the byte says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Shape {
    /// A vertical fader.
    Fader,
    /// A lit button in the row under the faders.
    Button,
    /// A column of printed legends with the selected one lit.
    Lamps,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{PANEL_CONTROL_COUNT, PANEL_ROWS, Shape, sections};
    use crate::param::{Group, Kind, PARAMETER_COUNT, ParamId};

    #[test]
    fn a_parameter_has_at_most_one_control_on_the_panel() {
        let mut seen = [false; PARAMETER_COUNT];
        let mut counted = 0;
        for section in sections() {
            for control in section.controls() {
                let parameter = control.parameter();
                let slot = seen
                    .get_mut(usize::from(parameter.offset()))
                    .expect("a parameter is inside the table");
                assert!(!*slot, "{parameter} is on the panel twice");
                *slot = true;
                counted += 1;
            }
        }
        assert_eq!(counted, PANEL_CONTROL_COUNT);
    }

    /// A plate's controls belong to the section its name claims, so a parameter
    /// moved between groups by a later reading of the manual fails here rather
    /// than opening a panel it is not on.
    #[test]
    fn every_plate_holds_parameters_of_one_group() {
        for section in sections() {
            assert!(!section.controls().is_empty(), "{section} has no controls");
            assert!(section.row() < PANEL_ROWS, "{section} is in no row");
            for control in section.controls() {
                assert_eq!(
                    control.parameter().group(),
                    section.group(),
                    "{} is on the {section} plate and in another section",
                    control.parameter()
                );
            }
        }
    }

    /// The legend is a silkscreen: caps, and short enough to print over a lane.
    #[test]
    fn a_legend_is_printed_the_way_a_panel_prints_one() {
        for section in sections() {
            for control in section.controls() {
                let legend = control.legend();
                assert!(!legend.is_empty(), "{} has no legend", control.parameter());
                assert!(
                    legend.chars().all(|c| !c.is_ascii_lowercase()),
                    "{legend} is not printed in caps"
                );
                assert!(
                    legend.split(' ').all(|word| word.len() <= 6),
                    "{legend} is too long a word to print over a fader"
                );
            }
        }
    }

    /// A column of lit legends is only worth drawing where there are names to
    /// light, which is the value table the parameter decodes through.
    #[test]
    fn a_strip_of_lamps_has_names_to_light() {
        for section in sections() {
            for control in section.controls() {
                if control.shape() != Shape::Lamps {
                    continue;
                }
                assert!(
                    matches!(control.parameter().kind(), Kind::Enumerated(_)),
                    "{} is drawn as lamps and has no names",
                    control.parameter()
                );
                assert!(control.parameter().choices().is_some());
            }
        }
    }

    /// Not a second rack. A table that grew past a fraction of the instrument
    /// would have stopped describing a front panel.
    #[test]
    fn the_panel_is_the_handful_a_player_reaches_for() {
        let panelled = sections()
            .iter()
            .map(|section| section.controls().len())
            .sum::<usize>();
        assert_eq!(panelled, PANEL_CONTROL_COUNT);
        assert!(panelled * 4 < PARAMETER_COUNT, "a second rack, not a panel");
        assert!(panelled > 20, "a front panel with nothing on it");
    }

    /// The lower row is the signal path, left to right, which is the whole of
    /// why the row is worth publishing.
    #[test]
    fn the_rows_are_the_instruments_own() {
        let row = |wanted: u8| -> Vec<&'static str> {
            sections()
                .iter()
                .filter(|section| section.row() == wanted)
                .map(super::Section::name)
                .collect()
        };
        assert_eq!(row(0), ["ARP / SEQ", "LFO 1", "LFO 2", "POLY"]);
        assert_eq!(row(1), ["DCO 1 & 2", "VCF", "VCA", "HPF", "ENVELOPES"]);
    }

    /// The panel is drawn from the parameter table, so every control it carries
    /// has to be one.
    #[test]
    fn every_control_names_a_parameter_the_table_has() {
        for section in sections() {
            for control in section.controls() {
                let parameter = control.parameter();
                assert_eq!(ParamId::from_offset(parameter.offset()), Ok(parameter));
                assert!(parameter.max() > parameter.min());
            }
        }
        // The two the instrument gives a plate of its own and the table keeps
        // together: both `VCF` and `HPF` answer the filter's group.
        let filters: Vec<&str> = sections()
            .iter()
            .filter(|section| section.group() == Group::Vcf)
            .map(super::Section::name)
            .collect();
        assert_eq!(filters, ["VCF", "HPF"]);
    }
}
