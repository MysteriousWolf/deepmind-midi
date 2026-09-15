//! The front panel data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

use super::{PanelControl, PanelShape, Section};
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
static ARP_SEQ: [PanelControl; 4] = [
    PanelControl {
        parameter: ParamId::ArpRateTempo,
        legend: "RATE",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::ArpGateTime,
        legend: "GATE TIME",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::ArpOnOff,
        legend: "ON/OFF",
        shape: PanelShape::Button,
    },
    PanelControl {
        parameter: ParamId::ArpHold,
        legend: "HOLD",
        shape: PanelShape::Button,
    },
];

/// The controls the LFO 1 plate carries.
static LFO1: [PanelControl; 3] = [
    PanelControl {
        parameter: ParamId::Lfo1Rate,
        legend: "RATE",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Lfo1DelayFade,
        legend: "DELAY TIME",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Lfo1Shape,
        legend: "SHAPE",
        shape: PanelShape::Lamps,
    },
];

/// The controls the LFO 2 plate carries.
static LFO2: [PanelControl; 3] = [
    PanelControl {
        parameter: ParamId::Lfo2Rate,
        legend: "RATE",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Lfo2DelayFade,
        legend: "DELAY TIME",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Lfo2Shape,
        legend: "SHAPE",
        shape: PanelShape::Lamps,
    },
];

/// The controls the POLY plate carries.
static POLY: [PanelControl; 1] = [PanelControl {
    parameter: ParamId::UnisonDetune,
    legend: "UNISON DETUNE",
    shape: PanelShape::Fader,
}];

/// The controls the DCO 1 & 2 plate carries.
static DCO1_AND2: [PanelControl; 8] = [
    PanelControl {
        parameter: ParamId::Osc1PitchModDepth,
        legend: "PITCH MOD",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Osc1PwmDepth,
        legend: "PWM",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Osc2PitchModDepth,
        legend: "PITCH MOD",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Osc2ToneModDepth,
        legend: "TONE MOD",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Osc2Pitch,
        legend: "PITCH",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Osc2Level,
        legend: "LEVEL",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::NoiseLevel,
        legend: "NOISE",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::OscSyncEnable,
        legend: "SYNC",
        shape: PanelShape::Button,
    },
];

/// The controls the VCF plate carries.
static VCF: [PanelControl; 6] = [
    PanelControl {
        parameter: ParamId::VcfFrequency,
        legend: "FREQ",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcfResonance,
        legend: "RES",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcfEnvelopeDepth,
        legend: "ENV",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcfLfoDepth,
        legend: "LFO",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcfKeyboardTracking,
        legend: "KYBD",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::Vcf2PoleMode,
        legend: "POLES",
        shape: PanelShape::Button,
    },
];

/// The controls the VCA plate carries.
static VCA: [PanelControl; 1] = [PanelControl {
    parameter: ParamId::VcaLevel,
    legend: "LEVEL",
    shape: PanelShape::Fader,
}];

/// The controls the HPF plate carries.
static HPF: [PanelControl; 2] = [
    PanelControl {
        parameter: ParamId::VcfHighPassFrequency,
        legend: "FREQ",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcfBassBoost,
        legend: "BOOST",
        shape: PanelShape::Button,
    },
];

/// The controls the ENVELOPES plate carries.
static ENVELOPES: [PanelControl; 4] = [
    PanelControl {
        parameter: ParamId::VcaEnvelopeAttackTime,
        legend: "A",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcaEnvelopeDecayTime,
        legend: "D",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcaEnvelopeSustainLevel,
        legend: "S",
        shape: PanelShape::Fader,
    },
    PanelControl {
        parameter: ParamId::VcaEnvelopeReleaseTime,
        legend: "R",
        shape: PanelShape::Fader,
    },
];
