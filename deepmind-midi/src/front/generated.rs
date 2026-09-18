//! The front panel data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

use super::{Lamp, PanelControl, PanelPress, PanelShape, Section, Sends};
use crate::effect::Colour;
use crate::param::{Group, ParamId};
use crate::pixels::Glyph;

/// Number of groups the front panel is divided into.
pub const SECTION_COUNT: usize = 9;

/// Number of rows the front panel is printed in.
pub const PANEL_ROWS: u8 = 2;

/// Number of parameters the front panel puts a control under.
///
/// A fraction of [`PARAMETER_COUNT`](crate::param::PARAMETER_COUNT): the panel
/// is the handful a player reaches for, and everything else is a press away on
/// the instrument's own display.
pub const PANEL_CONTROL_COUNT: usize = 35;

/// Number of presses the front panel carries that are not a parameter change.
///
/// The two chord presses in the arpeggiator's row of buttons. Everything else a
/// hand can reach on the front either moves a parameter or changes what the
/// instrument's own display is showing.
pub const PANEL_PRESS_COUNT: usize = 2;

/// A colour the front panel prints a section's name on.
///
/// The largest colour on the instrument, and a fact about the front rather than
/// about a byte: a photograph of a `DeepMind` is a dark panel with a row of red
/// stripes across it. Three of them, measured off the product photographs, and
/// reached from [`Section::banner`](super::Section::banner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Banner {
    /// The panel's own colour, and what a plate is unless it is said otherwise: a photograph of a `DeepMind` is a dark front with a row of red stripes across it.
    Red,
    /// The two plates that are not a stage of the voice: the arpeggiator, which is what plays it, and the high-pass, which the whole instrument passes through after the voices are mixed.
    Blue,
    /// The envelopes, the one plate whose faders address a section the panel's own buttons choose rather than a section of their own.
    White,
}

/// Number of colours a section's name is printed on.
pub const BANNER_COUNT: usize = 3;

/// The strip and the ink of each banner, in the order the enum names them.
pub(super) static BANNERS: [(Colour, Colour); BANNER_COUNT] = [
    (Colour::new(200, 23, 46), Colour::new(255, 255, 255)),
    (Colour::new(1, 96, 155), Colour::new(255, 255, 255)),
    (Colour::new(242, 243, 240), Colour::new(0, 0, 0)),
];

/// Every group of the front panel, in the order the instrument prints them.
pub(super) static SECTIONS: [Section; SECTION_COUNT] = [
    Section {
        name: "ARP / SEQ",
        row: 0,
        group: Group::Arpeggiator,
        banner: Banner::Blue,
        clusters: 1,
        note: None,
        controls: &ARP_SEQ,
        presses: &ARP_SEQ_PRESSES,
    },
    Section {
        name: "LFO 1",
        row: 0,
        group: Group::Lfo1,
        banner: Banner::Red,
        clusters: 1,
        note: Some(
            "The panel rules one plate between its two LFOs rather than printing two. They are two sections here because they are two groups of the parameter table, and a section carries one group: a host drawing the front puts them side by side.",
        ),
        controls: &LFO1,
        presses: &[],
    },
    Section {
        name: "LFO 2",
        row: 0,
        group: Group::Lfo2,
        banner: Banner::Red,
        clusters: 1,
        note: None,
        controls: &LFO2,
        presses: &[],
    },
    Section {
        name: "POLY",
        row: 0,
        group: Group::Voicing,
        banner: Banner::Red,
        clusters: 1,
        note: Some(
            "One fader, where the instrument has two. The other is `DATA ENTRY`, which edits whatever the display is showing rather than a parameter of its own.",
        ),
        controls: &POLY,
        presses: &[],
    },
    Section {
        name: "DCO 1 & 2",
        row: 1,
        group: Group::Oscillators,
        banner: Banner::Red,
        clusters: 2,
        note: Some(
            "Ruled between its two oscillators. The two waveform presses are on OSC 1's side of the rule, which is what makes them OSC 1's and not the pair the parameter table also gives OSC 2.",
        ),
        controls: &DCO1_AND2,
        presses: &[],
    },
    Section {
        name: "VCF",
        row: 1,
        group: Group::Vcf,
        banner: Banner::Red,
        clusters: 2,
        note: Some(
            "Ruled between `RES` and `ENV`: the filter's own controls on one side, the three that modulate it on the other.",
        ),
        controls: &VCF,
        presses: &[],
    },
    Section {
        name: "VCA",
        row: 1,
        group: Group::Vca,
        banner: Banner::Red,
        clusters: 1,
        note: None,
        controls: &VCA,
        presses: &[],
    },
    Section {
        name: "HPF",
        row: 1,
        group: Group::Vcf,
        banner: Banner::Blue,
        clusters: 1,
        note: Some(
            "The high-pass has a plate of its own on the panel and its two parameters are in the VCF group of the table, which is why a section's group is not a key. It is also printed on a different colour from the filter it shares that group with.",
        ),
        controls: &HPF,
        presses: &[],
    },
    Section {
        name: "ENVELOPES",
        row: 1,
        group: Group::VcaEnvelope,
        banner: Banner::White,
        clusters: 1,
        note: Some(
            "Four faders shared by three envelopes: the panel's own `VCA`, `VCF` and `MOD` buttons choose which one they address, and the group here is the one they address when the instrument is switched on.",
        ),
        controls: &ENVELOPES,
        presses: &[],
    },
];

/// The controls the ARP / SEQ plate carries.
static ARP_SEQ: [PanelControl; 4] = [
    PanelControl {
        parameter: ParamId::ArpRateTempo,
        legend: "RATE",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::ArpGateTime,
        legend: "GATE TIME",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::ArpOnOff,
        legend: "ON/OFF",
        drawing: None,
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::White),
    },
    PanelControl {
        parameter: ParamId::ArpHold,
        legend: "HOLD",
        drawing: None,
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::Cyan),
    },
];

/// The controls the LFO 1 plate carries.
static LFO1: [PanelControl; 3] = [
    PanelControl {
        parameter: ParamId::Lfo1Rate,
        legend: "RATE",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Lfo1DelayFade,
        legend: "DELAY TIME",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Lfo1Shape,
        legend: "SHAPE",
        drawing: None,
        shape: PanelShape::Lamps,
        cluster: 0,
        lamp: None,
    },
];

/// The controls the LFO 2 plate carries.
static LFO2: [PanelControl; 3] = [
    PanelControl {
        parameter: ParamId::Lfo2Rate,
        legend: "RATE",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Lfo2DelayFade,
        legend: "DELAY TIME",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Lfo2Shape,
        legend: "SHAPE",
        drawing: None,
        shape: PanelShape::Lamps,
        cluster: 0,
        lamp: None,
    },
];

/// The controls the POLY plate carries.
static POLY: [PanelControl; 1] = [PanelControl {
    parameter: ParamId::UnisonDetune,
    legend: "UNISON DETUNE",
    drawing: None,
    shape: PanelShape::Fader,
    cluster: 0,
    lamp: None,
}];

/// The controls the DCO 1 & 2 plate carries.
static DCO1_AND2: [PanelControl; 10] = [
    PanelControl {
        parameter: ParamId::Osc1PitchModDepth,
        legend: "PITCH MOD",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Osc1PwmDepth,
        legend: "PWM",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Osc1SawEnable,
        legend: "SAW",
        drawing: Some(Glyph::Saw),
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::White),
    },
    PanelControl {
        parameter: ParamId::Osc1PulseEnable,
        legend: "PULSE",
        drawing: Some(Glyph::Square),
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::White),
    },
    PanelControl {
        parameter: ParamId::OscSyncEnable,
        legend: "SYNC",
        drawing: None,
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::White),
    },
    PanelControl {
        parameter: ParamId::Osc2PitchModDepth,
        legend: "PITCH MOD",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Osc2ToneModDepth,
        legend: "TONE MOD",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Osc2Pitch,
        legend: "PITCH",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Osc2Level,
        legend: "LEVEL",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::NoiseLevel,
        legend: "NOISE",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
];

/// The controls the VCF plate carries.
static VCF: [PanelControl; 7] = [
    PanelControl {
        parameter: ParamId::VcfFrequency,
        legend: "FREQ",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcfResonance,
        legend: "RES",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::Vcf2PoleMode,
        legend: "POLES",
        drawing: None,
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::White),
    },
    PanelControl {
        parameter: ParamId::VcfEnvelopeDepth,
        legend: "ENV",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcfLfoDepth,
        legend: "LFO",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcfKeyboardTracking,
        legend: "KYBD",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 1,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcfEnvelopePolarity,
        legend: "INVERT",
        drawing: None,
        shape: PanelShape::Button,
        cluster: 1,
        lamp: Some(Lamp::White),
    },
];

/// The controls the VCA plate carries.
static VCA: [PanelControl; 1] = [PanelControl {
    parameter: ParamId::VcaLevel,
    legend: "LEVEL",
    drawing: None,
    shape: PanelShape::Fader,
    cluster: 0,
    lamp: None,
}];

/// The controls the HPF plate carries.
static HPF: [PanelControl; 2] = [
    PanelControl {
        parameter: ParamId::VcfHighPassFrequency,
        legend: "FREQ",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcfBassBoost,
        legend: "BOOST",
        drawing: None,
        shape: PanelShape::Button,
        cluster: 0,
        lamp: Some(Lamp::White),
    },
];

/// The controls the ENVELOPES plate carries.
static ENVELOPES: [PanelControl; 4] = [
    PanelControl {
        parameter: ParamId::VcaEnvelopeAttackTime,
        legend: "A",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcaEnvelopeDecayTime,
        legend: "D",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcaEnvelopeSustainLevel,
        legend: "S",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
    PanelControl {
        parameter: ParamId::VcaEnvelopeReleaseTime,
        legend: "R",
        drawing: None,
        shape: PanelShape::Fader,
        cluster: 0,
        lamp: None,
    },
];

/// The presses the ARP / SEQ plate carries that move no parameter.
static ARP_SEQ_PRESSES: [PanelPress; 2] = [
    PanelPress {
        legend: "CHORD",
        shape: PanelShape::Button,
        lamp: Lamp::Cyan,
        sends: Sends::Nothing,
        cluster: 0,
        note: Some(
            "Latches the chord the keyboard is holding and plays it from one key. What it plays from is readable: `Chord Memory Dump Request` (`0x1B`) returns the memory. Pressing it is not - no controller number reaches it, and no `SysEx` message the manual gives presses a button.",
        ),
    },
    PanelPress {
        legend: "POLY CHORD",
        shape: PanelShape::Button,
        lamp: Lamp::Cyan,
        sends: Sends::Nothing,
        cluster: 0,
        note: Some(
            "The same, with a chord under every key rather than one chord transposed. `Poly Chord Memory Dump Request` (`0x1D`) returns that memory; the press itself is local to the instrument.",
        ),
    },
];
