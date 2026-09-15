//! The front panel data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

use super::{Control, Section, Shape};
use crate::param::{Group, ParamId};

/// Number of groups the front panel is divided into.
pub const SECTION_COUNT: usize = 9;

/// Number of rows the front panel is printed in.
pub const PANEL_ROWS: u8 = 2;

/// Number of parameters the front panel puts a control under.
///
/// A fraction of [`PARAMETER_COUNT`](crate::param::PARAMETER_COUNT): the panel
/// is the handful a player reaches for, and everything else is a press away on
/// the instrument's own display.
pub const PANEL_CONTROL_COUNT: usize = 32;

/// Every group of the front panel, in the order the instrument prints them.
pub(super) static SECTIONS: [Section; SECTION_COUNT] = [
    Section {
        name: "ARP / SEQ",
        row: 0,
        group: Group::Arpeggiator,
        note: None,
        controls: &ARP_SEQ,
    },
    Section {
        name: "LFO 1",
        row: 0,
        group: Group::Lfo1,
        note: None,
        controls: &LFO1,
    },
    Section {
        name: "LFO 2",
        row: 0,
        group: Group::Lfo2,
        note: None,
        controls: &LFO2,
    },
    Section {
        name: "POLY",
        row: 0,
        group: Group::Voicing,
        note: Some(
            "One fader, where the instrument has two. The other is `DATA ENTRY`, which edits whatever the display is showing rather than a parameter of its own.",
        ),
        controls: &POLY,
    },
    Section {
        name: "DCO 1 & 2",
        row: 1,
        group: Group::Oscillators,
        note: None,
        controls: &DCO1_AND2,
    },
    Section {
        name: "VCF",
        row: 1,
        group: Group::Vcf,
        note: None,
        controls: &VCF,
    },
    Section {
        name: "VCA",
        row: 1,
        group: Group::Vca,
        note: None,
        controls: &VCA,
    },
    Section {
        name: "HPF",
        row: 1,
        group: Group::Vcf,
        note: Some(
            "The high-pass has a plate of its own on the panel and its two parameters are in the VCF group of the table, which is why a section's group is not a key.",
        ),
        controls: &HPF,
    },
    Section {
        name: "ENVELOPES",
        row: 1,
        group: Group::VcaEnvelope,
        note: Some(
            "Four faders shared by three envelopes: the panel's own `VCA`, `VCF` and `MOD` buttons choose which one they address, and the group here is the one they address when the instrument is switched on.",
        ),
        controls: &ENVELOPES,
    },
];

/// The controls the ARP / SEQ plate carries.
static ARP_SEQ: [Control; 4] = [
    Control {
        parameter: ParamId::ArpRateTempo,
        legend: "RATE",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::ArpGateTime,
        legend: "GATE TIME",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::ArpOnOff,
        legend: "ON/OFF",
        shape: Shape::Button,
    },
    Control {
        parameter: ParamId::ArpHold,
        legend: "HOLD",
        shape: Shape::Button,
    },
];

/// The controls the LFO 1 plate carries.
static LFO1: [Control; 3] = [
    Control {
        parameter: ParamId::Lfo1Rate,
        legend: "RATE",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Lfo1DelayFade,
        legend: "DELAY TIME",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Lfo1Shape,
        legend: "SHAPE",
        shape: Shape::Lamps,
    },
];

/// The controls the LFO 2 plate carries.
static LFO2: [Control; 3] = [
    Control {
        parameter: ParamId::Lfo2Rate,
        legend: "RATE",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Lfo2DelayFade,
        legend: "DELAY TIME",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Lfo2Shape,
        legend: "SHAPE",
        shape: Shape::Lamps,
    },
];

/// The controls the POLY plate carries.
static POLY: [Control; 1] = [Control {
    parameter: ParamId::UnisonDetune,
    legend: "UNISON DETUNE",
    shape: Shape::Fader,
}];

/// The controls the DCO 1 & 2 plate carries.
static DCO1_AND2: [Control; 8] = [
    Control {
        parameter: ParamId::Osc1PitchModDepth,
        legend: "PITCH MOD",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Osc1PwmDepth,
        legend: "PWM",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Osc2PitchModDepth,
        legend: "PITCH MOD",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Osc2ToneModDepth,
        legend: "TONE MOD",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Osc2Pitch,
        legend: "PITCH",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Osc2Level,
        legend: "LEVEL",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::NoiseLevel,
        legend: "NOISE",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::OscSyncEnable,
        legend: "SYNC",
        shape: Shape::Button,
    },
];

/// The controls the VCF plate carries.
static VCF: [Control; 6] = [
    Control {
        parameter: ParamId::VcfFrequency,
        legend: "FREQ",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcfResonance,
        legend: "RES",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcfEnvelopeDepth,
        legend: "ENV",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcfLfoDepth,
        legend: "LFO",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcfKeyboardTracking,
        legend: "KYBD",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::Vcf2PoleMode,
        legend: "POLES",
        shape: Shape::Button,
    },
];

/// The controls the VCA plate carries.
static VCA: [Control; 1] = [Control {
    parameter: ParamId::VcaLevel,
    legend: "LEVEL",
    shape: Shape::Fader,
}];

/// The controls the HPF plate carries.
static HPF: [Control; 2] = [
    Control {
        parameter: ParamId::VcfHighPassFrequency,
        legend: "FREQ",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcfBassBoost,
        legend: "BOOST",
        shape: Shape::Button,
    },
];

/// The controls the ENVELOPES plate carries.
static ENVELOPES: [Control; 4] = [
    Control {
        parameter: ParamId::VcaEnvelopeAttackTime,
        legend: "A",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcaEnvelopeDecayTime,
        legend: "D",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcaEnvelopeSustainLevel,
        legend: "S",
        shape: Shape::Fader,
    },
    Control {
        parameter: ParamId::VcaEnvelopeReleaseTime,
        legend: "R",
        shape: Shape::Fader,
    },
];
