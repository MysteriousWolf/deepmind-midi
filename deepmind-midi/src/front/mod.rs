//! The front of the synthesizer: which parameters have a physical control, what
//! is printed over them, and which of the panel's two rows they are in.
//!
//! [`param`](crate::param) says what exists,
//! [`Controller`] says what has a CC, and
//! [`effect`](crate::effect) says where an effect's slots sit on its own editor
//! panel. None of them says that [`ParamId::VcfFrequency`] has a fader with
//! `FREQ` silkscreened over it, that [`ParamId::UnisonDetune`] is under `POLY`,
//! or that the arpeggiator's two faders are `RATE` and `GATE TIME`. This is that
//! table, and it is the one fact about this synthesizer you can read off a
//! photograph.
//!
//! ```
//! use deepmind_midi::front::{PanelShape, sections};
//! use deepmind_midi::param::{Group, ParamId};
//!
//! let vcf = sections().iter().find(|s| s.name() == "VCF").expect("a filter");
//! assert_eq!(vcf.group(), Group::Vcf);
//! assert_eq!(vcf.row(), 1);
//!
//! let frequency = vcf.controls().first().expect("five faders and a button");
//! assert_eq!(frequency.parameter(), ParamId::VcfFrequency);
//! assert_eq!(frequency.legend(), "FREQ");
//! assert_eq!(frequency.shape(), PanelShape::Fader);
//! ```
//!
//! # Why it is here rather than in each host
//!
//! **The legend is not the parameter's name.** A silkscreen has room for `KYBD`
//! and `RES`; the parameter table says `VCF Keyboard Tracking` and `VCF
//! Resonance`, which is right for a rack slot and too long for a fader.
//!
//! **The shape is not derivable.** The instrument puts [`ParamId::ArpOnOff`] in
//! a row of buttons and [`ParamId::ArpRateTempo`] under a fader. The parameter
//! table knows only their [`Kind`](crate::param::Kind), which says how a byte is
//! read rather than what a hand touches.
//!
//! **A renamed parameter is a compile error.** The table holds [`ParamId`]s, so
//! a later reading of the manual that moves a parameter between groups fails the
//! specification's own checks instead of mislabelling somebody's fader.
//!
//! # How the panel presents them
//!
//! Which control is on the front is one question; how the front of the
//! instrument *presents* it is another, and four answers to it are here.
//!
//! **A rule between clusters.** A wide plate is divided by a thin printed line:
//! `VCF` is ruled between `RES` and `ENV`, so the filter's own controls are
//! separated from the three that modulate it. [`PanelControl::cluster`] numbers
//! them from the left, so a rule falls wherever the number changes, and
//! [`Section::clusters`] says how many a plate is divided into.
//!
//! **A lamp behind a press.** The instrument has three lamp colours and spends
//! them on a rule, which [`Lamp`] states. [`PanelControl::lamp`] gives the one
//! behind each press.
//!
//! **A banner over a plate.** The strip across the top of a plate with the
//! section's name on it is the largest colour on the instrument.
//! [`Section::banner`] gives it, as one of three measured colours.
//!
//! **A drawing instead of a word.** Two presses are printed as a waveform and
//! not as a word. [`PanelControl::silkscreen`] says which of the two a panel
//! prints, and [`PanelControl::legend`] stays a word for a host with a row of
//! text and no room for a picture.
//!
//! # The presses that are not a parameter
//!
//! `CHORD` and `POLY CHORD` sit in the arpeggiator's own row of buttons and
//! latch what the keyboard is playing. Neither is a program parameter, so
//! neither can be a [`PanelControl`], and a window drawing the front would have
//! a gap where two buttons are. [`Section::presses`] is the other kind of entry:
//! a legend, a lamp, a cluster, and [`PanelPress::sends`], which says what
//! pressing it puts on the wire.
//!
//! For those two the answer is [`Sends::Nothing`] — no controller reaches them
//! and no message the manual gives presses a button. That is still an answer: a
//! host that knows `CHORD` cannot be pressed over MIDI draws it disabled with a
//! sentence, where one that knows nothing draws nothing.
//!
//! # What it does not carry
//!
//! No pixels. Which row, which order within a section, and what is printed over
//! each control are facts off the instrument; lane widths and fader lengths are
//! the host's, the same split [`effect::grid`](crate::effect::grid) makes.
//!
//! Not the presses whose whole function is the instrument's own display. `DATA
//! ENTRY` edits whatever it is showing, the large encoder selects programs, the
//! twelve lamps over `POLY` count the voices that are sounding, the `EDIT`
//! buttons open a section on it, and `WRITE` and `COMPARE` act on an edit
//! buffer a host is keeping its own copy of. A program drawing this panel is
//! already the display, and has no use for a press that changes it.
//!
//! # One instrument
//!
//! Read off a `DeepMind` 12. The 6 has the same 242 parameters and its own
//! front; a variant gets its own table once somebody has one in front of them,
//! the way a value table gets its own firmware range.

mod generated;

pub use generated::{
    BANNER_COUNT, Banner, PANEL_CONTROL_COUNT, PANEL_PRESS_COUNT, PANEL_ROWS, SECTION_COUNT,
};

use generated::{BANNERS, SECTIONS};

use core::fmt;

use crate::effect::Colour;
use crate::param::{Controller, Group, ParamId};
use crate::pixels::Glyph;

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
    banner: Banner,
    clusters: u8,
    note: Option<&'static str>,
    controls: &'static [PanelControl],
    presses: &'static [PanelPress],
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
    pub const fn controls(&self) -> &'static [PanelControl] {
        self.controls
    }

    /// Returns the presses this plate carries that are not a parameter change.
    ///
    /// Empty for every plate but the arpeggiator's, which is where `CHORD` and
    /// `POLY CHORD` are. They latch what the keyboard is playing rather than
    /// set a byte, so a [`PanelControl`] cannot name them however the table
    /// grows.
    ///
    /// ```
    /// use deepmind_midi::front::{Lamp, Sends, sections};
    ///
    /// let arp = sections().iter().find(|s| s.name() == "ARP / SEQ").expect("an arpeggiator");
    /// let chord = arp.presses().first().expect("CHORD and POLY CHORD");
    /// assert_eq!(chord.legend(), "CHORD");
    /// assert_eq!(chord.lamp(), Lamp::Cyan);
    ///
    /// // And the fact worth publishing: no cable can press it.
    /// assert_eq!(chord.sends(), Sends::Nothing);
    /// assert!(chord.note().is_some());
    /// ```
    #[must_use]
    pub const fn presses(&self) -> &'static [PanelPress] {
        self.presses
    }

    /// Returns the colour the plate's name is printed on.
    ///
    /// The largest colour on the instrument: a photograph of a `DeepMind` is a
    /// dark front with a row of red stripes across it. Not a fact about the
    /// [`Group`], which is why it is here: `VCF` and `HPF` are one group printed
    /// on two colours.
    ///
    /// ```
    /// use deepmind_midi::front::{Banner, sections};
    ///
    /// let plate = |wanted| sections().iter().find(|s| s.name() == wanted).expect("a plate");
    /// assert_eq!(plate("VCF").banner(), Banner::Red);
    /// assert_eq!(plate("HPF").banner(), Banner::Blue);
    /// assert_eq!(plate("ENVELOPES").banner().ink().to_rgb(), [0, 0, 0]);
    /// ```
    #[must_use]
    pub const fn banner(&self) -> Banner {
        self.banner
    }

    /// Returns how many clusters a printed rule divides this plate into.
    ///
    /// 1 where the panel prints no rule, which is most of them. A host walking
    /// [`controls`](Self::controls) draws a rule wherever
    /// [`PanelControl::cluster`] changes; this is what it would have counted.
    ///
    /// ```
    /// use deepmind_midi::front::sections;
    ///
    /// let plate = |wanted| sections().iter().find(|s| s.name() == wanted).expect("a plate");
    ///
    /// // Ruled between RES and ENV: the filter's own, then the three that move it.
    /// assert_eq!(plate("VCF").clusters(), 2);
    /// let ruled = plate("VCF").controls().windows(2).find(|p| p[0].cluster() != p[1].cluster());
    /// let ruled = ruled.expect("one rule");
    /// assert_eq!((ruled[0].legend(), ruled[1].legend()), ("POLES", "ENV"));
    ///
    /// assert_eq!(plate("VCA").clusters(), 1);
    /// ```
    #[must_use]
    pub const fn clusters(&self) -> u8 {
        self.clusters
    }

    /// Returns what the specification records about this plate beyond its
    /// controls, where there is anything.
    ///
    /// Which fader of the instrument's is missing here and why, which of three
    /// envelopes the shared faders address, or where the printed rule falls.
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
pub struct PanelControl {
    parameter: ParamId,
    legend: &'static str,
    drawing: Option<Glyph>,
    shape: PanelShape,
    cluster: u8,
    lamp: Option<Lamp>,
}

impl PanelControl {
    /// Returns the parameter it moves.
    #[must_use]
    pub const fn parameter(&self) -> ParamId {
        self.parameter
    }

    /// Returns a word to print over it: `KYBD`, `PITCH MOD`, `A`.
    ///
    /// The instrument's own word, in caps, and not the parameter's name. Two
    /// words are printed on two lines, which is what the panel does with
    /// `PITCH MOD`.
    ///
    /// Two presses the panel prints as a waveform and not as a word still
    /// answer here, with this project's word for that wave, because a host with
    /// a row of text and no room for a drawing has to print something.
    /// [`silkscreen`](Self::silkscreen) is the one that says which of the two
    /// the instrument does.
    #[must_use]
    pub const fn legend(&self) -> &'static str {
        self.legend
    }

    /// Returns what the silkscreen puts over it: a word, or a drawing.
    ///
    /// ```
    /// use deepmind_midi::front::{Silkscreen, sections};
    /// use deepmind_midi::pixels::Glyph;
    ///
    /// let dco = sections().iter().find(|s| s.name() == "DCO 1 & 2").expect("two oscillators");
    /// let saw = dco.controls().iter().find(|c| c.legend() == "SAW").expect("a saw press");
    ///
    /// // The panel prints the wave itself and no word beside it.
    /// assert_eq!(saw.silkscreen(), Silkscreen::Drawing(Glyph::Saw));
    ///
    /// let pwm = dco.controls().iter().find(|c| c.legend() == "PWM").expect("a PWM fader");
    /// assert_eq!(pwm.silkscreen(), Silkscreen::Word("PWM"));
    /// ```
    #[must_use]
    pub const fn silkscreen(&self) -> Silkscreen {
        match self.drawing {
            Some(glyph) => Silkscreen::Drawing(glyph),
            None => Silkscreen::Word(self.legend),
        }
    }

    /// Returns what a hand touches.
    #[must_use]
    pub const fn shape(&self) -> PanelShape {
        self.shape
    }

    /// Returns which cluster of its plate it is in, counting from the left.
    ///
    /// A thin printed rule falls wherever this changes, so a host draws one by
    /// walking the plate's controls and watching the number. 0 throughout a
    /// plate the panel prints no rule on; see [`Section::clusters`].
    #[must_use]
    pub const fn cluster(&self) -> u8 {
        self.cluster
    }

    /// Returns the colour of the lamp behind it, where the panel lights one.
    ///
    /// `None` for a fader, which is not lit, and for a column of lamps, whose
    /// lit legend says which value is selected rather than what kind of press
    /// this is. Every press answers, and the rule behind the answer is in
    /// [`Lamp`].
    ///
    /// ```
    /// use deepmind_midi::front::{Lamp, sections};
    /// use deepmind_midi::param::ParamId;
    ///
    /// let arp = sections().iter().find(|s| s.name() == "ARP / SEQ").expect("an arpeggiator");
    /// let control = |id| arp.controls().iter().find(|c| c.parameter() == id).expect("a control");
    ///
    /// // Hold changes what the keys mean, so it is one of the cyan ones.
    /// assert_eq!(control(ParamId::ArpHold).lamp(), Some(Lamp::Cyan));
    /// assert_eq!(control(ParamId::ArpOnOff).lamp(), Some(Lamp::White));
    /// assert_eq!(control(ParamId::ArpRateTempo).lamp(), None);
    /// ```
    #[must_use]
    pub const fn lamp(&self) -> Option<Lamp> {
        self.lamp
    }
}

impl fmt::Display for PanelControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.legend)
    }
}

/// One press the front panel carries that is not a parameter change.
///
/// `CHORD` and `POLY CHORD`, which latch what the keyboard is playing. The
/// parameter table has nothing for either — a sweep over all 242 turns up one
/// arpeggiator switch a cap could sit on, and it is `Arp Hold` — so they are
/// performance functions rather than bytes in the edit buffer, and a table
/// keyed by [`ParamId`] cannot mention them however it grows.
///
/// What a host wants from one is where it sits, what is printed on it, and
/// whether a cable can press it. [`sends`](Self::sends) is the last of those,
/// and for both of these the honest answer is [`Sends::Nothing`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PanelPress {
    legend: &'static str,
    shape: PanelShape,
    lamp: Lamp,
    sends: Sends,
    cluster: u8,
    note: Option<&'static str>,
}

impl PanelPress {
    /// Returns what the panel prints over it: `CHORD`, `POLY CHORD`.
    #[must_use]
    pub const fn legend(&self) -> &'static str {
        self.legend
    }

    /// Returns what a hand touches, which for a press is
    /// [`PanelShape::Button`].
    ///
    /// Here so that a host drawing a plate can put the presses through the same
    /// arm as the controls beside them.
    #[must_use]
    pub const fn shape(&self) -> PanelShape {
        self.shape
    }

    /// Returns the colour of the lamp behind it.
    ///
    /// Not optional, the way [`PanelControl::lamp`] is: a press has a cap and
    /// the cap is lit.
    #[must_use]
    pub const fn lamp(&self) -> Lamp {
        self.lamp
    }

    /// Returns what pressing it puts on the wire.
    #[must_use]
    pub const fn sends(&self) -> Sends {
        self.sends
    }

    /// Returns which cluster of its plate it is in, counting from the left.
    ///
    /// The same numbering [`PanelControl::cluster`] uses, so a press and the
    /// controls beside it fall between the same two rules.
    #[must_use]
    pub const fn cluster(&self) -> u8 {
        self.cluster
    }

    /// Returns what the specification records about the press.
    ///
    /// What it does to the keyboard, and — where it sends nothing — what a host
    /// can reach instead. Both chord presses play from a memory a `SysEx` dump
    /// returns, which is not the press but is the nearest thing to it.
    #[must_use]
    pub const fn note(&self) -> Option<&'static str> {
        self.note
    }
}

impl fmt::Display for PanelPress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.legend)
    }
}

/// What pressing a [`PanelPress`] puts on the wire.
///
/// The fact worth publishing about a button that is not a parameter. A host
/// that knows a press cannot be sent draws it disabled with a sentence, which
/// is honest; one that knows nothing draws nothing, which reads as an
/// oversight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum Sends {
    /// Nothing a cable carries: the press is local to the instrument.
    ///
    /// No controller number reaches it, and no message in the manual presses a
    /// button. Both of the chord presses are this, and both play from a memory
    /// a dump request returns, which is in the press's
    /// [`note`](PanelPress::note).
    Nothing,
    /// The controller number a host sends to press it.
    ///
    /// [`controller`](Self::controller) resolves it to the entry the controller
    /// table holds, which is where its name and its glyph are.
    Controller(u8),
}

impl Sends {
    /// Returns the controller a host sends to press it, where one reaches it.
    ///
    /// `None` for [`Sends::Nothing`], which is the whole of the table today.
    #[must_use]
    pub fn controller(self) -> Option<&'static Controller> {
        match self {
            Self::Nothing => None,
            Self::Controller(cc) => Controller::for_cc(cc),
        }
    }
}

/// The colour of the lamp behind a press.
///
/// The instrument has three and spends them on a rule, which is the library's
/// kind of fact rather than a host's taste:
///
/// - **Amber** on a press that changes what the display is showing: `EDIT`,
///   `PROG`, `FX`, `GLOBAL`, `COMPARE`, `WRITE`. None of those is in this table,
///   for the reason the module's own list of omissions gives.
/// - **Cyan** on a press that changes what the *other* controls mean: `CHORD`,
///   `POLY CHORD`, `TAP/HOLD`, `MOD`, `CURVES`. Of those, `Arp Hold` is the only
///   one that is also a program parameter; two more are [`PanelPress`]es here.
/// - **White** on everything else, which is every other press on the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Lamp {
    /// The display is about to show something else.
    Amber,
    /// The other controls are about to mean something else.
    Cyan,
    /// Neither: the press does what it says.
    White,
}

impl fmt::Display for Lamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Amber => "amber",
            Self::Cyan => "cyan",
            Self::White => "white",
        })
    }
}

/// What the silkscreen puts over a control: a word, or a drawing.
///
/// Almost every control on the front is printed with a word, and two are not:
/// the pair at the left-hand end of the oscillator block that add OSC 1's saw
/// and its pulse to the mix are printed as the waves themselves. A host asking
/// what the panel prints over one of those cannot be told the truth by a
/// string, which is what this is for — the same shape of answer
/// [`Algorithm::mark`](crate::effect::Algorithm::mark) gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum Silkscreen {
    /// The word the panel prints, in caps.
    Word(&'static str),
    /// The wave the panel draws, and no word beside it.
    ///
    /// A [`Glyph`], so it is the drawing a small display is already blitting
    /// for the parameter itself rather than a second set of pixels to carry.
    Drawing(Glyph),
}

impl Banner {
    /// Returns the colour of the strip itself.
    #[must_use]
    pub fn plate(self) -> Colour {
        self.colours().0
    }

    /// Returns the colour the section's name is printed in.
    ///
    /// White on the two dark banners and black on the light one, which is the
    /// whole of the choice.
    #[must_use]
    pub fn ink(self) -> Colour {
        self.colours().1
    }

    /// Returns both, which is how they are stored.
    fn colours(self) -> (Colour, Colour) {
        BANNERS
            .get(self as usize)
            .copied()
            .unwrap_or((Colour::new(0, 0, 0), Colour::new(0, 0, 0)))
    }
}

impl fmt::Display for Banner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.plate())
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
pub enum PanelShape {
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
    use super::{
        BANNER_COUNT, Banner, Lamp, PANEL_CONTROL_COUNT, PANEL_PRESS_COUNT, PANEL_ROWS, PanelShape,
        Sends, Silkscreen, sections,
    };
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
                if control.shape() != PanelShape::Lamps {
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
        let row = |wanted: u8| {
            sections()
                .iter()
                .filter(move |section| section.row() == wanted)
                .map(super::Section::name)
        };
        assert!(row(0).eq(["ARP / SEQ", "LFO 1", "LFO 2", "POLY"]));
        assert!(row(1).eq(["DCO 1 & 2", "VCF", "VCA", "HPF", "ENVELOPES"]));
    }

    /// A rule is a change of cluster, so the numbering has to be the runs a
    /// panel could print: from 0, never backwards, and no gap.
    #[test]
    fn a_clusters_numbering_is_the_runs_a_rule_divides() {
        for section in sections() {
            let mut current = 0;
            let clusters = section
                .controls()
                .iter()
                .map(super::PanelControl::cluster)
                .chain(section.presses().iter().map(super::PanelPress::cluster));
            for cluster in clusters {
                assert!(
                    cluster == current || cluster == current + 1,
                    "{section} rules a cluster {cluster} after {current}"
                );
                current = cluster;
            }
            assert_eq!(
                section.clusters(),
                current + 1,
                "{section} counts its clusters differently from its controls"
            );
        }
    }

    /// The rule the three lamp colours follow: a press is lit, and nothing else
    /// on the panel claims a colour it has no cap for.
    #[test]
    fn only_a_press_carries_a_lamp() {
        let mut lit = 0;
        for section in sections() {
            for control in section.controls() {
                match control.shape() {
                    PanelShape::Button => {
                        assert!(control.lamp().is_some(), "{} is unlit", control.parameter());
                        lit += 1;
                    }
                    PanelShape::Fader | PanelShape::Lamps => assert_eq!(
                        control.lamp(),
                        None,
                        "{} is not a press and carries a lamp",
                        control.parameter()
                    ),
                }
            }
        }
        assert!(lit > 0, "a front panel with nothing to press");
        // Cyan is spent on a press that changes what the other controls mean,
        // and `Arp Hold` is the only program parameter that does.
        let cyan = sections()
            .iter()
            .flat_map(super::Section::controls)
            .filter(|control| control.lamp() == Some(Lamp::Cyan))
            .map(super::PanelControl::parameter);
        assert!(cyan.eq([ParamId::ArpHold]));
    }

    /// A press that is not a parameter is in the table for what it does to the
    /// keyboard, so it has to say what a cable can do about it.
    #[test]
    fn a_press_says_what_it_sends() {
        let presses: usize = sections()
            .iter()
            .map(|section| section.presses().len())
            .sum();
        assert_eq!(presses, PANEL_PRESS_COUNT);
        for section in sections() {
            for press in section.presses() {
                assert_eq!(press.shape(), PanelShape::Button);
                assert!(press.cluster() < section.clusters());
                match press.sends() {
                    // Nothing over the wire is an answer, and it needs the
                    // sentence that makes it one rather than a silent gap.
                    Sends::Nothing => assert!(
                        press.note().is_some(),
                        "{press} sends nothing and says nothing about why"
                    ),
                    Sends::Controller(cc) => {
                        assert!(
                            press.sends().controller().is_some(),
                            "{press} sends CC {cc}"
                        );
                    }
                }
            }
        }
        // The two the arpeggiator has, and no plate has quietly grown a third.
        let legends = sections()
            .iter()
            .flat_map(super::Section::presses)
            .map(super::PanelPress::legend);
        assert!(legends.eq(["CHORD", "POLY CHORD"]));
    }

    /// A press printed as a wave still answers `legend`, because a host with a
    /// row of text has to print something; `silkscreen` is what says which.
    #[test]
    fn a_drawn_silkscreen_is_a_wave_and_a_word() {
        let mut drawn = 0;
        for section in sections() {
            for control in section.controls() {
                match control.silkscreen() {
                    Silkscreen::Word(word) => assert_eq!(word, control.legend()),
                    Silkscreen::Drawing(glyph) => {
                        drawn += 1;
                        assert!(!control.legend().is_empty());
                        // The panel draws the wave the parameter is pictured
                        // by, so a small display blits one drawing and not two.
                        assert_eq!(control.parameter().glyph(), Some(glyph));
                    }
                }
            }
        }
        assert_eq!(drawn, 2, "the two waveform presses on OSC 1");
    }

    /// Three colours, and every plate printed on one of them.
    #[test]
    fn every_plate_is_printed_on_a_measured_banner() {
        assert_eq!(BANNER_COUNT, 3);
        for section in sections() {
            let banner = section.banner();
            assert_ne!(
                banner.plate(),
                banner.ink(),
                "{section} prints its name in the colour of the strip"
            );
        }
        // Red is what a plate is unless the panel says otherwise.
        let red = sections()
            .iter()
            .filter(|section| section.banner() == Banner::Red)
            .count();
        assert!(
            red * 2 > sections().len(),
            "a panel without its red stripes"
        );
        assert_eq!(Banner::Red.plate().to_rgb(), [0xc8, 0x17, 0x2e]);
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
        assert!(
            sections()
                .iter()
                .filter(|section| section.group() == Group::Vcf)
                .map(super::Section::name)
                .eq(["VCF", "HPF"])
        );
    }
}
