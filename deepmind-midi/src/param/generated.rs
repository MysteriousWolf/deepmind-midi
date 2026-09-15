//! The parameter, value table and controller data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

// The parameter table is one match with an arm per parameter. Splitting it into
// chunks to satisfy a length lint would only hide what it is.
#![expect(clippy::too_many_lines, reason = "a generated table, not logic")]

use super::{Controller, ControllerKind, Kind, Parameter, Shape, ValueEntry, ValueTable};
use crate::sysex::inquiry::Version;

/// Number of program parameters. Offsets run `0..PARAMETER_COUNT`.
pub const PARAMETER_COUNT: usize = 242;

/// Number of continuous controllers the synthesizer answers.
pub const CONTROLLER_COUNT: usize = 112;

/// Number of value tables, counting a renumbered one once per firmware version.
pub const TABLE_COUNT: usize = 30;

/// Firmware assumed by every lookup that is not given one: the newest the
/// specification describes.
pub const DEFAULT_FIRMWARE: Version = Version { major: 1, minor: 1 };

/// Section of the instrument a parameter belongs to.
///
/// The groups the manual's own NRPN table is divided into, which is also how the
/// front panel is divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Group {
    /// Arpeggiator.
    Arpeggiator,
    /// Control Sequencer.
    ControlSequencer,
    /// Effects.
    Effects,
    /// LFO 1.
    Lfo1,
    /// LFO 2.
    Lfo2,
    /// Mod Envelope.
    ModEnvelope,
    /// Mod Matrix.
    ModMatrix,
    /// Oscillators.
    Oscillators,
    /// Program.
    Program,
    /// VCA.
    Vca,
    /// VCA Envelope.
    VcaEnvelope,
    /// VCF.
    Vcf,
    /// VCF Envelope.
    VcfEnvelope,
    /// Voicing.
    Voicing,
}

impl Group {
    /// Every group, in alphabetical order.
    pub const ALL: &'static [Self] = &[
        Self::Arpeggiator,
        Self::ControlSequencer,
        Self::Effects,
        Self::Lfo1,
        Self::Lfo2,
        Self::ModEnvelope,
        Self::ModMatrix,
        Self::Oscillators,
        Self::Program,
        Self::Vca,
        Self::VcaEnvelope,
        Self::Vcf,
        Self::VcfEnvelope,
        Self::Voicing,
    ];

    /// Every group, in the order the instrument lays them out.
    ///
    /// Not [`ALL`](Self::ALL), which is alphabetical and puts the effects third
    /// and the oscillators eighth. A parameter's offset is its NRPN number and
    /// its place in a dump, so the order the groups first appear in the table is
    /// the order the panel is in: the LFOs, the oscillators, the filter, the
    /// envelopes and the VCA, voicing, modulation, sequencing, the arpeggiator,
    /// the effects, and the program's own settings last.
    ///
    /// Read off the parameter table rather than written down, so a group a later
    /// specification adds arrives in its right place.
    pub const ORDER: &'static [Self] = &[
        Self::Lfo1,
        Self::Lfo2,
        Self::Oscillators,
        Self::Vcf,
        Self::VcaEnvelope,
        Self::VcfEnvelope,
        Self::ModEnvelope,
        Self::Vca,
        Self::Voicing,
        Self::ModMatrix,
        Self::ControlSequencer,
        Self::Arpeggiator,
        Self::Effects,
        Self::Program,
    ];

    /// Returns the group's name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Arpeggiator => "Arpeggiator",
            Self::ControlSequencer => "Control Sequencer",
            Self::Effects => "Effects",
            Self::Lfo1 => "LFO 1",
            Self::Lfo2 => "LFO 2",
            Self::ModEnvelope => "Mod Envelope",
            Self::ModMatrix => "Mod Matrix",
            Self::Oscillators => "Oscillators",
            Self::Program => "Program",
            Self::Vca => "VCA",
            Self::VcaEnvelope => "VCA Envelope",
            Self::Vcf => "VCF",
            Self::VcfEnvelope => "VCF Envelope",
            Self::Voicing => "Voicing",
        }
    }
}

/// One of the 242 program parameters.
///
/// The discriminant is the parameter's NRPN number, which is also its byte
/// offset in an unpacked program dump; [`ParamId::offset`] is that number and
/// [`ParamId::from_offset`] the way back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum ParamId {
    /// LFO 1 Rate.
    Lfo1Rate = 0,
    /// LFO 1 Delay / Fade.
    Lfo1DelayFade = 1,
    /// LFO 1 Shape.
    Lfo1Shape = 2,
    /// LFO 1 Key Sync.
    Lfo1KeySync = 3,
    /// LFO 1 Arp Sync.
    Lfo1ArpSync = 4,
    /// LFO 1 Mono Mode.
    Lfo1MonoMode = 5,
    /// LFO 1 Slew Rate.
    Lfo1SlewRate = 6,
    /// LFO 2 Rate.
    Lfo2Rate = 7,
    /// LFO 2 Delay / Fade.
    Lfo2DelayFade = 8,
    /// LFO 2 Shape.
    Lfo2Shape = 9,
    /// LFO 2 Key Sync.
    Lfo2KeySync = 10,
    /// LFO 2 Arp Sync.
    Lfo2ArpSync = 11,
    /// LFO 2 Mono Mode.
    Lfo2MonoMode = 12,
    /// LFO 2 Slew Rate.
    Lfo2SlewRate = 13,
    /// OSC 1 Range.
    Osc1Range = 14,
    /// OSC 2 Range.
    Osc2Range = 15,
    /// OSC 1 PWM Source.
    Osc1PwmSource = 16,
    /// OSC 2 Tone Mod Source.
    Osc2ToneModSource = 17,
    /// OSC 1 Pulse Enable.
    Osc1PulseEnable = 18,
    /// OSC 1 Saw Enable.
    Osc1SawEnable = 19,
    /// OSC Sync Enable.
    OscSyncEnable = 20,
    /// OSC 1 Pitch Mod Depth.
    Osc1PitchModDepth = 21,
    /// OSC 1 Pitch Mod Select.
    Osc1PitchModSelect = 22,
    /// OSC 1 Aftertouch > Pitch Mod Depth.
    Osc1AftertouchToPitchModDepth = 23,
    /// OSC 1 Mod Wheel > Pitch Mod Depth.
    Osc1ModWheelToPitchModDepth = 24,
    /// OSC 1 PWM Depth.
    Osc1PwmDepth = 25,
    /// OSC 2 Level.
    Osc2Level = 26,
    /// OSC 2 Pitch.
    Osc2Pitch = 27,
    /// OSC 2 Tone Mod Depth.
    Osc2ToneModDepth = 28,
    /// OSC 2 Pitch Mod Depth.
    Osc2PitchModDepth = 29,
    /// OSC 2 Aftertouch > Pitch Mod Depth.
    Osc2AftertouchToPitchModDepth = 30,
    /// OSC 2 Mod Wheel > Pitch Mod Depth.
    Osc2ModWheelToPitchModDepth = 31,
    /// OSC 2 Pitch Mod Select.
    Osc2PitchModSelect = 32,
    /// Noise Level.
    NoiseLevel = 33,
    /// Portamento time.
    PortamentoTime = 34,
    /// Portamento mode.
    PortamentoMode = 35,
    /// Pitch Bend Up Depth.
    PitchBendUpDepth = 36,
    /// Pitch Bend Down Depth.
    PitchBendDownDepth = 37,
    /// OSC 1 Pitch Mod Mode.
    Osc1PitchModMode = 38,
    /// VCF Frequency.
    VcfFrequency = 39,
    /// VCF `HighPass` Frequency.
    VcfHighPassFrequency = 40,
    /// VCF Resonance.
    VcfResonance = 41,
    /// VCF Envelope Depth.
    VcfEnvelopeDepth = 42,
    /// VCF Envelope Velocity Sensitivity.
    VcfEnvelopeVelocitySensitivity = 43,
    /// VCF Pitch Bend to Freq Depth.
    VcfPitchBendToFreqDepth = 44,
    /// VCF LFO Depth.
    VcfLfoDepth = 45,
    /// VCF LFO Select.
    VcfLfoSelect = 46,
    /// VCF Aftertouch > LFO Depth.
    VcfAftertouchToLfoDepth = 47,
    /// VCF Mod Wheel > LFO Depth.
    VcfModWheelToLfoDepth = 48,
    /// VCF Keyboard Tracking.
    VcfKeyboardTracking = 49,
    /// VCF Envelope Polarity.
    VcfEnvelopePolarity = 50,
    /// VCF 2 Pole Mode.
    Vcf2PoleMode = 51,
    /// VCF Bass Boost.
    VcfBassBoost = 52,
    /// VCA Envelope Attack Time.
    VcaEnvelopeAttackTime = 53,
    /// VCA Envelope Decay Time.
    VcaEnvelopeDecayTime = 54,
    /// VCA Envelope Sustain Level.
    VcaEnvelopeSustainLevel = 55,
    /// VCA Envelope Release Time.
    VcaEnvelopeReleaseTime = 56,
    /// VCA Envelope Trigger Mode.
    VcaEnvelopeTriggerMode = 57,
    /// VCA Envelope Attack Curve.
    VcaEnvelopeAttackCurve = 58,
    /// VCA Envelope Decay Curve.
    VcaEnvelopeDecayCurve = 59,
    /// VCA Envelope Sustain Curve.
    VcaEnvelopeSustainCurve = 60,
    /// VCA Envelope Release Curve.
    VcaEnvelopeReleaseCurve = 61,
    /// VCF Envelope Attack Time.
    VcfEnvelopeAttackTime = 62,
    /// VCF Envelope Decay Time.
    VcfEnvelopeDecayTime = 63,
    /// VCF Envelope Sustain Level.
    VcfEnvelopeSustainLevel = 64,
    /// VCF Envelope Release Time.
    VcfEnvelopeReleaseTime = 65,
    /// VCF Envelope Trigger Mode.
    VcfEnvelopeTriggerMode = 66,
    /// VCF Envelope Attack Curve.
    VcfEnvelopeAttackCurve = 67,
    /// VCF Envelope Decay Curve.
    VcfEnvelopeDecayCurve = 68,
    /// VCF Envelope Sustain Curve.
    VcfEnvelopeSustainCurve = 69,
    /// VCF Envelope Release Curve.
    VcfEnvelopeReleaseCurve = 70,
    /// Mod Envelope Attack Time.
    ModEnvelopeAttackTime = 71,
    /// Mod Envelope Decay Time.
    ModEnvelopeDecayTime = 72,
    /// Mod Envelope Sustain Level.
    ModEnvelopeSustainLevel = 73,
    /// Mod Envelope Release Time.
    ModEnvelopeReleaseTime = 74,
    /// Mod Envelope Trigger Mode.
    ModEnvelopeTriggerMode = 75,
    /// Mod Envelope Attack Curve.
    ModEnvelopeAttackCurve = 76,
    /// Mod Envelope Decay Curve.
    ModEnvelopeDecayCurve = 77,
    /// Mod Envelope Sustain Curve.
    ModEnvelopeSustainCurve = 78,
    /// Mod Envelope Release Curve.
    ModEnvelopeReleaseCurve = 79,
    /// VCA Level.
    VcaLevel = 80,
    /// VCA Envelope Depth.
    VcaEnvelopeDepth = 81,
    /// VCA Envelope Velocity Sensitivity.
    VcaEnvelopeVelocitySensitivity = 82,
    /// VCA Pan Spread.
    VcaPanSpread = 83,
    /// Voice Priority Mode.
    VoicePriorityMode = 84,
    /// Polyphony Mode.
    PolyphonyMode = 85,
    /// Envelope Trigger Mode.
    EnvelopeTriggerMode = 86,
    /// Unison Detune.
    UnisonDetune = 87,
    /// Voice Drift.
    VoiceDrift = 88,
    /// Parameter Drift.
    ParameterDrift = 89,
    /// Drift Rate.
    DriftRate = 90,
    /// OSC Portamento Balance.
    OscPortamentoBalance = 91,
    /// OSC Key Down Reset.
    OscKeyDownReset = 92,
    /// Mod 1 Source.
    Mod1Source = 93,
    /// Mod 1 Destination.
    Mod1Destination = 94,
    /// Mod 1 Depth.
    Mod1Depth = 95,
    /// Mod 2 Source.
    Mod2Source = 96,
    /// Mod 2 Destination.
    Mod2Destination = 97,
    /// Mod 2 Depth.
    Mod2Depth = 98,
    /// Mod 3 Source.
    Mod3Source = 99,
    /// Mod 3 Destination.
    Mod3Destination = 100,
    /// Mod 3 Depth.
    Mod3Depth = 101,
    /// Mod 4 Source.
    Mod4Source = 102,
    /// Mod 4 Destination.
    Mod4Destination = 103,
    /// Mod 4 Depth.
    Mod4Depth = 104,
    /// Mod 5 Source.
    Mod5Source = 105,
    /// Mod 5 Destination.
    Mod5Destination = 106,
    /// Mod 5 Depth.
    Mod5Depth = 107,
    /// Mod 6 Source.
    Mod6Source = 108,
    /// Mod 6 Destination.
    Mod6Destination = 109,
    /// Mod 6 Depth.
    Mod6Depth = 110,
    /// Mod 7 Source.
    Mod7Source = 111,
    /// Mod 7 Destination.
    Mod7Destination = 112,
    /// Mod 7 Depth.
    Mod7Depth = 113,
    /// Mod 8 Source.
    Mod8Source = 114,
    /// Mod 8 Destination.
    Mod8Destination = 115,
    /// Mod 8 Depth.
    Mod8Depth = 116,
    /// Ctrl Sequencer Enable.
    CtrlSequencerEnable = 117,
    /// Ctrl Sequencer Clock Divider.
    CtrlSequencerClockDivider = 118,
    /// Sequence Length.
    SequenceLength = 119,
    /// Sequencer Swing Timing.
    SequencerSwingTiming = 120,
    /// Key Sync & Loop.
    KeySyncAndLoop = 121,
    /// Slew Rate.
    SlewRate = 122,
    /// Seq Step Value 1.
    SeqStepValue1 = 123,
    /// Seq Step Value 2.
    SeqStepValue2 = 124,
    /// Seq Step Value 3.
    SeqStepValue3 = 125,
    /// Seq Step Value 4.
    SeqStepValue4 = 126,
    /// Seq Step Value 5.
    SeqStepValue5 = 127,
    /// Seq Step Value 6.
    SeqStepValue6 = 128,
    /// Seq Step Value 7.
    SeqStepValue7 = 129,
    /// Seq Step Value 8.
    SeqStepValue8 = 130,
    /// Seq Step Value 9.
    SeqStepValue9 = 131,
    /// Seq Step Value 10.
    SeqStepValue10 = 132,
    /// Seq Step Value 11.
    SeqStepValue11 = 133,
    /// Seq Step Value 12.
    SeqStepValue12 = 134,
    /// Seq Step Value 13.
    SeqStepValue13 = 135,
    /// Seq Step Value 14.
    SeqStepValue14 = 136,
    /// Seq Step Value 15.
    SeqStepValue15 = 137,
    /// Seq Step Value 16.
    SeqStepValue16 = 138,
    /// Seq Step Value 17.
    SeqStepValue17 = 139,
    /// Seq Step Value 18.
    SeqStepValue18 = 140,
    /// Seq Step Value 19.
    SeqStepValue19 = 141,
    /// Seq Step Value 20.
    SeqStepValue20 = 142,
    /// Seq Step Value 21.
    SeqStepValue21 = 143,
    /// Seq Step Value 22.
    SeqStepValue22 = 144,
    /// Seq Step Value 23.
    SeqStepValue23 = 145,
    /// Seq Step Value 24.
    SeqStepValue24 = 146,
    /// Seq Step Value 25.
    SeqStepValue25 = 147,
    /// Seq Step Value 26.
    SeqStepValue26 = 148,
    /// Seq Step Value 27.
    SeqStepValue27 = 149,
    /// Seq Step Value 28.
    SeqStepValue28 = 150,
    /// Seq Step Value 29.
    SeqStepValue29 = 151,
    /// Seq Step Value 30.
    SeqStepValue30 = 152,
    /// Seq Step Value 31.
    SeqStepValue31 = 153,
    /// Seq Step Value 32.
    SeqStepValue32 = 154,
    /// Arp `On/Off`.
    ArpOnOff = 155,
    /// Arp Mode.
    ArpMode = 156,
    /// Arp Rate (tempo).
    ArpRateTempo = 157,
    /// Arp Clock.
    ArpClock = 158,
    /// Arp Key Sync.
    ArpKeySync = 159,
    /// Arp Gate Time.
    ArpGateTime = 160,
    /// Arp Hold.
    ArpHold = 161,
    /// Arp Pattern.
    ArpPattern = 162,
    /// Arp Swing.
    ArpSwing = 163,
    /// Arp Octaves.
    ArpOctaves = 164,
    /// FX Routing.
    FxRouting = 165,
    /// FX 1 Type.
    Fx1Type = 166,
    /// FX 1 Param 1.
    Fx1Param1 = 167,
    /// FX 1 Param 2.
    Fx1Param2 = 168,
    /// FX 1 Param 3.
    Fx1Param3 = 169,
    /// FX 1 Param 4.
    Fx1Param4 = 170,
    /// FX 1 Param 5.
    Fx1Param5 = 171,
    /// FX 1 Param 6.
    Fx1Param6 = 172,
    /// FX 1 Param 7.
    Fx1Param7 = 173,
    /// FX 1 Param 8.
    Fx1Param8 = 174,
    /// FX 1 Param 9.
    Fx1Param9 = 175,
    /// FX 1 Param 10.
    Fx1Param10 = 176,
    /// FX 1 Param 11.
    Fx1Param11 = 177,
    /// FX 1 Param 12.
    Fx1Param12 = 178,
    /// FX 2 Type.
    Fx2Type = 179,
    /// FX 2 Param 1.
    Fx2Param1 = 180,
    /// FX 2 Param 2.
    Fx2Param2 = 181,
    /// FX 2 Param 3.
    Fx2Param3 = 182,
    /// FX 2 Param 4.
    Fx2Param4 = 183,
    /// FX 2 Param 5.
    Fx2Param5 = 184,
    /// FX 2 Param 6.
    Fx2Param6 = 185,
    /// FX 2 Param 7.
    Fx2Param7 = 186,
    /// FX 2 Param 8.
    Fx2Param8 = 187,
    /// FX 2 Param 9.
    Fx2Param9 = 188,
    /// FX 2 Param 10.
    Fx2Param10 = 189,
    /// FX 2 Param 11.
    Fx2Param11 = 190,
    /// FX 2 Param 12.
    Fx2Param12 = 191,
    /// FX 3 Type.
    Fx3Type = 192,
    /// FX 3 Param 1.
    Fx3Param1 = 193,
    /// FX 3 Param 2.
    Fx3Param2 = 194,
    /// FX 3 Param 3.
    Fx3Param3 = 195,
    /// FX 3 Param 4.
    Fx3Param4 = 196,
    /// FX 3 Param 5.
    Fx3Param5 = 197,
    /// FX 3 Param 6.
    Fx3Param6 = 198,
    /// FX 3 Param 7.
    Fx3Param7 = 199,
    /// FX 3 Param 8.
    Fx3Param8 = 200,
    /// FX 3 Param 9.
    Fx3Param9 = 201,
    /// FX 3 Param 10.
    Fx3Param10 = 202,
    /// FX 3 Param 11.
    Fx3Param11 = 203,
    /// FX 3 Param 12.
    Fx3Param12 = 204,
    /// FX 4 Type.
    Fx4Type = 205,
    /// FX 4 Param 1.
    Fx4Param1 = 206,
    /// FX 4 Param 2.
    Fx4Param2 = 207,
    /// FX 4 Param 3.
    Fx4Param3 = 208,
    /// FX 4 Param 4.
    Fx4Param4 = 209,
    /// FX 4 Param 5.
    Fx4Param5 = 210,
    /// FX 4 Param 6.
    Fx4Param6 = 211,
    /// FX 4 Param 7.
    Fx4Param7 = 212,
    /// FX 4 Param 8.
    Fx4Param8 = 213,
    /// FX 4 Param 9.
    Fx4Param9 = 214,
    /// FX 4 Param 10.
    Fx4Param10 = 215,
    /// FX 4 Param 11.
    Fx4Param11 = 216,
    /// FX 4 Param 12.
    Fx4Param12 = 217,
    /// FX 1 Output Gain.
    Fx1OutputGain = 218,
    /// FX 2 Output Gain.
    Fx2OutputGain = 219,
    /// FX 3 Output Gain.
    Fx3OutputGain = 220,
    /// FX 4 Output Gain.
    Fx4OutputGain = 221,
    /// FX Mode.
    FxMode = 222,
    /// Program Name Char 1.
    ProgramNameChar1 = 223,
    /// Program Name Char 2.
    ProgramNameChar2 = 224,
    /// Program Name Char 3.
    ProgramNameChar3 = 225,
    /// Program Name Char 4.
    ProgramNameChar4 = 226,
    /// Program Name Char 5.
    ProgramNameChar5 = 227,
    /// Program Name Char 6.
    ProgramNameChar6 = 228,
    /// Program Name Char 7.
    ProgramNameChar7 = 229,
    /// Program Name Char 8.
    ProgramNameChar8 = 230,
    /// Program Name Char 9.
    ProgramNameChar9 = 231,
    /// Program Name Char 10.
    ProgramNameChar10 = 232,
    /// Program Name Char 11.
    ProgramNameChar11 = 233,
    /// Program Name Char 12.
    ProgramNameChar12 = 234,
    /// Program Name Char 13.
    ProgramNameChar13 = 235,
    /// Program Name Char 14.
    ProgramNameChar14 = 236,
    /// Program Name Char 15.
    ProgramNameChar15 = 237,
    /// Program Name Char 16.
    ProgramNameChar16 = 238,
    /// Program Name Char 17.
    ProgramNameChar17 = 239,
    /// Program Category.
    ProgramCategory = 240,
    /// Program Transpose.
    ProgramTranspose = 241,
}

impl ParamId {
    /// Every parameter, in offset order: [`PARAMETER_COUNT`] of them.
    pub const ALL: &'static [Self] = &[
        Self::Lfo1Rate,
        Self::Lfo1DelayFade,
        Self::Lfo1Shape,
        Self::Lfo1KeySync,
        Self::Lfo1ArpSync,
        Self::Lfo1MonoMode,
        Self::Lfo1SlewRate,
        Self::Lfo2Rate,
        Self::Lfo2DelayFade,
        Self::Lfo2Shape,
        Self::Lfo2KeySync,
        Self::Lfo2ArpSync,
        Self::Lfo2MonoMode,
        Self::Lfo2SlewRate,
        Self::Osc1Range,
        Self::Osc2Range,
        Self::Osc1PwmSource,
        Self::Osc2ToneModSource,
        Self::Osc1PulseEnable,
        Self::Osc1SawEnable,
        Self::OscSyncEnable,
        Self::Osc1PitchModDepth,
        Self::Osc1PitchModSelect,
        Self::Osc1AftertouchToPitchModDepth,
        Self::Osc1ModWheelToPitchModDepth,
        Self::Osc1PwmDepth,
        Self::Osc2Level,
        Self::Osc2Pitch,
        Self::Osc2ToneModDepth,
        Self::Osc2PitchModDepth,
        Self::Osc2AftertouchToPitchModDepth,
        Self::Osc2ModWheelToPitchModDepth,
        Self::Osc2PitchModSelect,
        Self::NoiseLevel,
        Self::PortamentoTime,
        Self::PortamentoMode,
        Self::PitchBendUpDepth,
        Self::PitchBendDownDepth,
        Self::Osc1PitchModMode,
        Self::VcfFrequency,
        Self::VcfHighPassFrequency,
        Self::VcfResonance,
        Self::VcfEnvelopeDepth,
        Self::VcfEnvelopeVelocitySensitivity,
        Self::VcfPitchBendToFreqDepth,
        Self::VcfLfoDepth,
        Self::VcfLfoSelect,
        Self::VcfAftertouchToLfoDepth,
        Self::VcfModWheelToLfoDepth,
        Self::VcfKeyboardTracking,
        Self::VcfEnvelopePolarity,
        Self::Vcf2PoleMode,
        Self::VcfBassBoost,
        Self::VcaEnvelopeAttackTime,
        Self::VcaEnvelopeDecayTime,
        Self::VcaEnvelopeSustainLevel,
        Self::VcaEnvelopeReleaseTime,
        Self::VcaEnvelopeTriggerMode,
        Self::VcaEnvelopeAttackCurve,
        Self::VcaEnvelopeDecayCurve,
        Self::VcaEnvelopeSustainCurve,
        Self::VcaEnvelopeReleaseCurve,
        Self::VcfEnvelopeAttackTime,
        Self::VcfEnvelopeDecayTime,
        Self::VcfEnvelopeSustainLevel,
        Self::VcfEnvelopeReleaseTime,
        Self::VcfEnvelopeTriggerMode,
        Self::VcfEnvelopeAttackCurve,
        Self::VcfEnvelopeDecayCurve,
        Self::VcfEnvelopeSustainCurve,
        Self::VcfEnvelopeReleaseCurve,
        Self::ModEnvelopeAttackTime,
        Self::ModEnvelopeDecayTime,
        Self::ModEnvelopeSustainLevel,
        Self::ModEnvelopeReleaseTime,
        Self::ModEnvelopeTriggerMode,
        Self::ModEnvelopeAttackCurve,
        Self::ModEnvelopeDecayCurve,
        Self::ModEnvelopeSustainCurve,
        Self::ModEnvelopeReleaseCurve,
        Self::VcaLevel,
        Self::VcaEnvelopeDepth,
        Self::VcaEnvelopeVelocitySensitivity,
        Self::VcaPanSpread,
        Self::VoicePriorityMode,
        Self::PolyphonyMode,
        Self::EnvelopeTriggerMode,
        Self::UnisonDetune,
        Self::VoiceDrift,
        Self::ParameterDrift,
        Self::DriftRate,
        Self::OscPortamentoBalance,
        Self::OscKeyDownReset,
        Self::Mod1Source,
        Self::Mod1Destination,
        Self::Mod1Depth,
        Self::Mod2Source,
        Self::Mod2Destination,
        Self::Mod2Depth,
        Self::Mod3Source,
        Self::Mod3Destination,
        Self::Mod3Depth,
        Self::Mod4Source,
        Self::Mod4Destination,
        Self::Mod4Depth,
        Self::Mod5Source,
        Self::Mod5Destination,
        Self::Mod5Depth,
        Self::Mod6Source,
        Self::Mod6Destination,
        Self::Mod6Depth,
        Self::Mod7Source,
        Self::Mod7Destination,
        Self::Mod7Depth,
        Self::Mod8Source,
        Self::Mod8Destination,
        Self::Mod8Depth,
        Self::CtrlSequencerEnable,
        Self::CtrlSequencerClockDivider,
        Self::SequenceLength,
        Self::SequencerSwingTiming,
        Self::KeySyncAndLoop,
        Self::SlewRate,
        Self::SeqStepValue1,
        Self::SeqStepValue2,
        Self::SeqStepValue3,
        Self::SeqStepValue4,
        Self::SeqStepValue5,
        Self::SeqStepValue6,
        Self::SeqStepValue7,
        Self::SeqStepValue8,
        Self::SeqStepValue9,
        Self::SeqStepValue10,
        Self::SeqStepValue11,
        Self::SeqStepValue12,
        Self::SeqStepValue13,
        Self::SeqStepValue14,
        Self::SeqStepValue15,
        Self::SeqStepValue16,
        Self::SeqStepValue17,
        Self::SeqStepValue18,
        Self::SeqStepValue19,
        Self::SeqStepValue20,
        Self::SeqStepValue21,
        Self::SeqStepValue22,
        Self::SeqStepValue23,
        Self::SeqStepValue24,
        Self::SeqStepValue25,
        Self::SeqStepValue26,
        Self::SeqStepValue27,
        Self::SeqStepValue28,
        Self::SeqStepValue29,
        Self::SeqStepValue30,
        Self::SeqStepValue31,
        Self::SeqStepValue32,
        Self::ArpOnOff,
        Self::ArpMode,
        Self::ArpRateTempo,
        Self::ArpClock,
        Self::ArpKeySync,
        Self::ArpGateTime,
        Self::ArpHold,
        Self::ArpPattern,
        Self::ArpSwing,
        Self::ArpOctaves,
        Self::FxRouting,
        Self::Fx1Type,
        Self::Fx1Param1,
        Self::Fx1Param2,
        Self::Fx1Param3,
        Self::Fx1Param4,
        Self::Fx1Param5,
        Self::Fx1Param6,
        Self::Fx1Param7,
        Self::Fx1Param8,
        Self::Fx1Param9,
        Self::Fx1Param10,
        Self::Fx1Param11,
        Self::Fx1Param12,
        Self::Fx2Type,
        Self::Fx2Param1,
        Self::Fx2Param2,
        Self::Fx2Param3,
        Self::Fx2Param4,
        Self::Fx2Param5,
        Self::Fx2Param6,
        Self::Fx2Param7,
        Self::Fx2Param8,
        Self::Fx2Param9,
        Self::Fx2Param10,
        Self::Fx2Param11,
        Self::Fx2Param12,
        Self::Fx3Type,
        Self::Fx3Param1,
        Self::Fx3Param2,
        Self::Fx3Param3,
        Self::Fx3Param4,
        Self::Fx3Param5,
        Self::Fx3Param6,
        Self::Fx3Param7,
        Self::Fx3Param8,
        Self::Fx3Param9,
        Self::Fx3Param10,
        Self::Fx3Param11,
        Self::Fx3Param12,
        Self::Fx4Type,
        Self::Fx4Param1,
        Self::Fx4Param2,
        Self::Fx4Param3,
        Self::Fx4Param4,
        Self::Fx4Param5,
        Self::Fx4Param6,
        Self::Fx4Param7,
        Self::Fx4Param8,
        Self::Fx4Param9,
        Self::Fx4Param10,
        Self::Fx4Param11,
        Self::Fx4Param12,
        Self::Fx1OutputGain,
        Self::Fx2OutputGain,
        Self::Fx3OutputGain,
        Self::Fx4OutputGain,
        Self::FxMode,
        Self::ProgramNameChar1,
        Self::ProgramNameChar2,
        Self::ProgramNameChar3,
        Self::ProgramNameChar4,
        Self::ProgramNameChar5,
        Self::ProgramNameChar6,
        Self::ProgramNameChar7,
        Self::ProgramNameChar8,
        Self::ProgramNameChar9,
        Self::ProgramNameChar10,
        Self::ProgramNameChar11,
        Self::ProgramNameChar12,
        Self::ProgramNameChar13,
        Self::ProgramNameChar14,
        Self::ProgramNameChar15,
        Self::ProgramNameChar16,
        Self::ProgramNameChar17,
        Self::ProgramCategory,
        Self::ProgramTranspose,
    ];

    /// Returns everything the specification says about this parameter.
    #[must_use]
    pub const fn info(self) -> Parameter {
        match self {
            Self::Lfo1Rate => Parameter {
                name: "LFO 1 Rate",
                group: Group::Lfo1,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Lfo1DelayFade => Parameter {
                name: "LFO 1 Delay / Fade",
                group: Group::Lfo1,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Lfo1Shape => Parameter {
                name: "LFO 1 Shape",
                group: Group::Lfo1,
                min: 0,
                max: 6,
                kind: Kind::Enumerated(TableId::LfoShape),
            },
            Self::Lfo1KeySync => Parameter {
                name: "LFO 1 Key Sync",
                group: Group::Lfo1,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Lfo1ArpSync => Parameter {
                name: "LFO 1 Arp Sync",
                group: Group::Lfo1,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Lfo1MonoMode => Parameter {
                name: "LFO 1 Mono Mode",
                group: Group::Lfo1,
                min: 0,
                max: 255,
                kind: Kind::Enumerated(TableId::LfoMonoMode),
            },
            Self::Lfo1SlewRate => Parameter {
                name: "LFO 1 Slew Rate",
                group: Group::Lfo1,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Lfo2Rate => Parameter {
                name: "LFO 2 Rate",
                group: Group::Lfo2,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Lfo2DelayFade => Parameter {
                name: "LFO 2 Delay / Fade",
                group: Group::Lfo2,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Lfo2Shape => Parameter {
                name: "LFO 2 Shape",
                group: Group::Lfo2,
                min: 0,
                max: 6,
                kind: Kind::Enumerated(TableId::LfoShape),
            },
            Self::Lfo2KeySync => Parameter {
                name: "LFO 2 Key Sync",
                group: Group::Lfo2,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Lfo2ArpSync => Parameter {
                name: "LFO 2 Arp Sync",
                group: Group::Lfo2,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Lfo2MonoMode => Parameter {
                name: "LFO 2 Mono Mode",
                group: Group::Lfo2,
                min: 0,
                max: 255,
                kind: Kind::Enumerated(TableId::LfoMonoMode),
            },
            Self::Lfo2SlewRate => Parameter {
                name: "LFO 2 Slew Rate",
                group: Group::Lfo2,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc1Range => Parameter {
                name: "OSC 1 Range",
                group: Group::Oscillators,
                min: 0,
                max: 2,
                kind: Kind::Enumerated(TableId::OscRange),
            },
            Self::Osc2Range => Parameter {
                name: "OSC 2 Range",
                group: Group::Oscillators,
                min: 0,
                max: 2,
                kind: Kind::Enumerated(TableId::OscRange),
            },
            Self::Osc1PwmSource => Parameter {
                name: "OSC 1 PWM Source",
                group: Group::Oscillators,
                min: 0,
                max: 5,
                kind: Kind::Enumerated(TableId::PwmSource),
            },
            Self::Osc2ToneModSource => Parameter {
                name: "OSC 2 Tone Mod Source",
                group: Group::Oscillators,
                min: 0,
                max: 5,
                kind: Kind::Enumerated(TableId::ToneModSource),
            },
            Self::Osc1PulseEnable => Parameter {
                name: "OSC 1 Pulse Enable",
                group: Group::Oscillators,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Osc1SawEnable => Parameter {
                name: "OSC 1 Saw Enable",
                group: Group::Oscillators,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::OscSyncEnable => Parameter {
                name: "OSC Sync Enable",
                group: Group::Oscillators,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Osc1PitchModDepth => Parameter {
                name: "OSC 1 Pitch Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc1PitchModSelect => Parameter {
                name: "OSC 1 Pitch Mod Select",
                group: Group::Oscillators,
                min: 0,
                max: 6,
                kind: Kind::Enumerated(TableId::PitchModSource),
            },
            Self::Osc1AftertouchToPitchModDepth => Parameter {
                name: "OSC 1 Aftertouch > Pitch Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc1ModWheelToPitchModDepth => Parameter {
                name: "OSC 1 Mod Wheel > Pitch Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc1PwmDepth => Parameter {
                name: "OSC 1 PWM Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2Level => Parameter {
                name: "OSC 2 Level",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2Pitch => Parameter {
                name: "OSC 2 Pitch",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2ToneModDepth => Parameter {
                name: "OSC 2 Tone Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2PitchModDepth => Parameter {
                name: "OSC 2 Pitch Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2AftertouchToPitchModDepth => Parameter {
                name: "OSC 2 Aftertouch > Pitch Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2ModWheelToPitchModDepth => Parameter {
                name: "OSC 2 Mod Wheel > Pitch Mod Depth",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Osc2PitchModSelect => Parameter {
                name: "OSC 2 Pitch Mod Select",
                group: Group::Oscillators,
                min: 0,
                max: 6,
                kind: Kind::Enumerated(TableId::PitchModSource),
            },
            Self::NoiseLevel => Parameter {
                name: "Noise Level",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::PortamentoTime => Parameter {
                name: "Portamento time",
                group: Group::Oscillators,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::PortamentoMode => Parameter {
                name: "Portamento mode",
                group: Group::Oscillators,
                min: 0,
                max: 13,
                kind: Kind::Enumerated(TableId::PortamentoMode),
            },
            Self::PitchBendUpDepth => Parameter {
                name: "Pitch Bend Up Depth",
                group: Group::Oscillators,
                min: 0,
                max: 48,
                kind: Kind::Continuous,
            },
            Self::PitchBendDownDepth => Parameter {
                name: "Pitch Bend Down Depth",
                group: Group::Oscillators,
                min: 0,
                max: 48,
                kind: Kind::Continuous,
            },
            Self::Osc1PitchModMode => Parameter {
                name: "OSC 1 Pitch Mod Mode",
                group: Group::Oscillators,
                min: 0,
                max: 1,
                kind: Kind::Enumerated(TableId::Osc1PitchModMode),
            },
            Self::VcfFrequency => Parameter {
                name: "VCF Frequency",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfHighPassFrequency => Parameter {
                name: "VCF HighPass Frequency",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfResonance => Parameter {
                name: "VCF Resonance",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeDepth => Parameter {
                name: "VCF Envelope Depth",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeVelocitySensitivity => Parameter {
                name: "VCF Envelope Velocity Sensitivity",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfPitchBendToFreqDepth => Parameter {
                name: "VCF Pitch Bend to Freq Depth",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfLfoDepth => Parameter {
                name: "VCF LFO Depth",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfLfoSelect => Parameter {
                name: "VCF LFO Select",
                group: Group::Vcf,
                min: 0,
                max: 1,
                kind: Kind::Enumerated(TableId::VcfLfoSelect),
            },
            Self::VcfAftertouchToLfoDepth => Parameter {
                name: "VCF Aftertouch > LFO Depth",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfModWheelToLfoDepth => Parameter {
                name: "VCF Mod Wheel > LFO Depth",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfKeyboardTracking => Parameter {
                name: "VCF Keyboard Tracking",
                group: Group::Vcf,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopePolarity => Parameter {
                name: "VCF Envelope Polarity",
                group: Group::Vcf,
                min: 0,
                max: 1,
                kind: Kind::Enumerated(TableId::VcfEnvelopePolarity),
            },
            Self::Vcf2PoleMode => Parameter {
                name: "VCF 2 Pole Mode",
                group: Group::Vcf,
                min: 0,
                max: 1,
                kind: Kind::Enumerated(TableId::VcfPoleMode),
            },
            Self::VcfBassBoost => Parameter {
                name: "VCF Bass Boost",
                group: Group::Vcf,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::VcaEnvelopeAttackTime => Parameter {
                name: "VCA Envelope Attack Time",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeDecayTime => Parameter {
                name: "VCA Envelope Decay Time",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeSustainLevel => Parameter {
                name: "VCA Envelope Sustain Level",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeReleaseTime => Parameter {
                name: "VCA Envelope Release Time",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeTriggerMode => Parameter {
                name: "VCA Envelope Trigger Mode",
                group: Group::VcaEnvelope,
                min: 0,
                max: 4,
                kind: Kind::Enumerated(TableId::EnvelopeTrigger),
            },
            Self::VcaEnvelopeAttackCurve => Parameter {
                name: "VCA Envelope Attack Curve",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeDecayCurve => Parameter {
                name: "VCA Envelope Decay Curve",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeSustainCurve => Parameter {
                name: "VCA Envelope Sustain Curve",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeReleaseCurve => Parameter {
                name: "VCA Envelope Release Curve",
                group: Group::VcaEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeAttackTime => Parameter {
                name: "VCF Envelope Attack Time",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeDecayTime => Parameter {
                name: "VCF Envelope Decay Time",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeSustainLevel => Parameter {
                name: "VCF Envelope Sustain Level",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeReleaseTime => Parameter {
                name: "VCF Envelope Release Time",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeTriggerMode => Parameter {
                name: "VCF Envelope Trigger Mode",
                group: Group::VcfEnvelope,
                min: 0,
                max: 4,
                kind: Kind::Enumerated(TableId::EnvelopeTrigger),
            },
            Self::VcfEnvelopeAttackCurve => Parameter {
                name: "VCF Envelope Attack Curve",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeDecayCurve => Parameter {
                name: "VCF Envelope Decay Curve",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeSustainCurve => Parameter {
                name: "VCF Envelope Sustain Curve",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcfEnvelopeReleaseCurve => Parameter {
                name: "VCF Envelope Release Curve",
                group: Group::VcfEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeAttackTime => Parameter {
                name: "Mod Envelope Attack Time",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeDecayTime => Parameter {
                name: "Mod Envelope Decay Time",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeSustainLevel => Parameter {
                name: "Mod Envelope Sustain Level",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeReleaseTime => Parameter {
                name: "Mod Envelope Release Time",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeTriggerMode => Parameter {
                name: "Mod Envelope Trigger Mode",
                group: Group::ModEnvelope,
                min: 0,
                max: 4,
                kind: Kind::Enumerated(TableId::EnvelopeTrigger),
            },
            Self::ModEnvelopeAttackCurve => Parameter {
                name: "Mod Envelope Attack Curve",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeDecayCurve => Parameter {
                name: "Mod Envelope Decay Curve",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeSustainCurve => Parameter {
                name: "Mod Envelope Sustain Curve",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ModEnvelopeReleaseCurve => Parameter {
                name: "Mod Envelope Release Curve",
                group: Group::ModEnvelope,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaLevel => Parameter {
                name: "VCA Level",
                group: Group::Vca,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeDepth => Parameter {
                name: "VCA Envelope Depth",
                group: Group::Vca,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaEnvelopeVelocitySensitivity => Parameter {
                name: "VCA Envelope Velocity Sensitivity",
                group: Group::Vca,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VcaPanSpread => Parameter {
                name: "VCA Pan Spread",
                group: Group::Vca,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VoicePriorityMode => Parameter {
                name: "Voice Priority Mode",
                group: Group::Voicing,
                min: 0,
                max: 2,
                kind: Kind::Enumerated(TableId::VoicePriority),
            },
            Self::PolyphonyMode => Parameter {
                name: "Polyphony Mode",
                group: Group::Voicing,
                min: 0,
                max: 12,
                kind: Kind::Enumerated(TableId::PolyphonyMode),
            },
            Self::EnvelopeTriggerMode => Parameter {
                name: "Envelope Trigger Mode",
                group: Group::Voicing,
                min: 0,
                max: 3,
                kind: Kind::Enumerated(TableId::KeyAssignMode),
            },
            Self::UnisonDetune => Parameter {
                name: "Unison Detune",
                group: Group::Voicing,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::VoiceDrift => Parameter {
                name: "Voice Drift",
                group: Group::Voicing,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ParameterDrift => Parameter {
                name: "Parameter Drift",
                group: Group::Voicing,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::DriftRate => Parameter {
                name: "Drift Rate",
                group: Group::Voicing,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::OscPortamentoBalance => Parameter {
                name: "OSC Portamento Balance",
                group: Group::Voicing,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::OscKeyDownReset => Parameter {
                name: "OSC Key Down Reset",
                group: Group::Voicing,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::Mod1Source => Parameter {
                name: "Mod 1 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod1Destination => Parameter {
                name: "Mod 1 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod1Depth => Parameter {
                name: "Mod 1 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod2Source => Parameter {
                name: "Mod 2 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod2Destination => Parameter {
                name: "Mod 2 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod2Depth => Parameter {
                name: "Mod 2 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod3Source => Parameter {
                name: "Mod 3 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod3Destination => Parameter {
                name: "Mod 3 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod3Depth => Parameter {
                name: "Mod 3 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod4Source => Parameter {
                name: "Mod 4 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod4Destination => Parameter {
                name: "Mod 4 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod4Depth => Parameter {
                name: "Mod 4 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod5Source => Parameter {
                name: "Mod 5 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod5Destination => Parameter {
                name: "Mod 5 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod5Depth => Parameter {
                name: "Mod 5 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod6Source => Parameter {
                name: "Mod 6 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod6Destination => Parameter {
                name: "Mod 6 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod6Depth => Parameter {
                name: "Mod 6 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod7Source => Parameter {
                name: "Mod 7 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod7Destination => Parameter {
                name: "Mod 7 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod7Depth => Parameter {
                name: "Mod 7 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Mod8Source => Parameter {
                name: "Mod 8 Source",
                group: Group::ModMatrix,
                min: 0,
                max: 24,
                kind: Kind::Enumerated(TableId::ModSource),
            },
            Self::Mod8Destination => Parameter {
                name: "Mod 8 Destination",
                group: Group::ModMatrix,
                min: 0,
                max: 132,
                kind: Kind::Enumerated(TableId::ModDestination),
            },
            Self::Mod8Depth => Parameter {
                name: "Mod 8 Depth",
                group: Group::ModMatrix,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::CtrlSequencerEnable => Parameter {
                name: "Ctrl Sequencer Enable",
                group: Group::ControlSequencer,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::CtrlSequencerClockDivider => Parameter {
                name: "Ctrl Sequencer Clock Divider",
                group: Group::ControlSequencer,
                min: 0,
                max: 15,
                kind: Kind::Enumerated(TableId::SequencerClock),
            },
            Self::SequenceLength => Parameter {
                name: "Sequence Length",
                group: Group::ControlSequencer,
                min: 0,
                max: 31,
                kind: Kind::Continuous,
            },
            Self::SequencerSwingTiming => Parameter {
                name: "Sequencer Swing Timing",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::KeySyncAndLoop => Parameter {
                name: "Key Sync & Loop",
                group: Group::ControlSequencer,
                min: 0,
                max: 2,
                kind: Kind::Enumerated(TableId::SequencerSync),
            },
            Self::SlewRate => Parameter {
                name: "Slew Rate",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue1 => Parameter {
                name: "Seq Step Value 1",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue2 => Parameter {
                name: "Seq Step Value 2",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue3 => Parameter {
                name: "Seq Step Value 3",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue4 => Parameter {
                name: "Seq Step Value 4",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue5 => Parameter {
                name: "Seq Step Value 5",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue6 => Parameter {
                name: "Seq Step Value 6",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue7 => Parameter {
                name: "Seq Step Value 7",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue8 => Parameter {
                name: "Seq Step Value 8",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue9 => Parameter {
                name: "Seq Step Value 9",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue10 => Parameter {
                name: "Seq Step Value 10",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue11 => Parameter {
                name: "Seq Step Value 11",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue12 => Parameter {
                name: "Seq Step Value 12",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue13 => Parameter {
                name: "Seq Step Value 13",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue14 => Parameter {
                name: "Seq Step Value 14",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue15 => Parameter {
                name: "Seq Step Value 15",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue16 => Parameter {
                name: "Seq Step Value 16",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue17 => Parameter {
                name: "Seq Step Value 17",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue18 => Parameter {
                name: "Seq Step Value 18",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue19 => Parameter {
                name: "Seq Step Value 19",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue20 => Parameter {
                name: "Seq Step Value 20",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue21 => Parameter {
                name: "Seq Step Value 21",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue22 => Parameter {
                name: "Seq Step Value 22",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue23 => Parameter {
                name: "Seq Step Value 23",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue24 => Parameter {
                name: "Seq Step Value 24",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue25 => Parameter {
                name: "Seq Step Value 25",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue26 => Parameter {
                name: "Seq Step Value 26",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue27 => Parameter {
                name: "Seq Step Value 27",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue28 => Parameter {
                name: "Seq Step Value 28",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue29 => Parameter {
                name: "Seq Step Value 29",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue30 => Parameter {
                name: "Seq Step Value 30",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue31 => Parameter {
                name: "Seq Step Value 31",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::SeqStepValue32 => Parameter {
                name: "Seq Step Value 32",
                group: Group::ControlSequencer,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ArpOnOff => Parameter {
                name: "Arp On/Off",
                group: Group::Arpeggiator,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::ArpMode => Parameter {
                name: "Arp Mode",
                group: Group::Arpeggiator,
                min: 0,
                max: 10,
                kind: Kind::Enumerated(TableId::ArpMode),
            },
            Self::ArpRateTempo => Parameter {
                name: "Arp Rate (tempo)",
                group: Group::Arpeggiator,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ArpClock => Parameter {
                name: "Arp Clock",
                group: Group::Arpeggiator,
                min: 0,
                max: 12,
                kind: Kind::Enumerated(TableId::ArpClock),
            },
            Self::ArpKeySync => Parameter {
                name: "Arp Key Sync",
                group: Group::Arpeggiator,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::ArpGateTime => Parameter {
                name: "Arp Gate Time",
                group: Group::Arpeggiator,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ArpHold => Parameter {
                name: "Arp Hold",
                group: Group::Arpeggiator,
                min: 0,
                max: 1,
                kind: Kind::Switch,
            },
            Self::ArpPattern => Parameter {
                name: "Arp Pattern",
                group: Group::Arpeggiator,
                min: 0,
                max: 64,
                kind: Kind::Enumerated(TableId::ArpPattern),
            },
            Self::ArpSwing => Parameter {
                name: "Arp Swing",
                group: Group::Arpeggiator,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::ArpOctaves => Parameter {
                name: "Arp Octaves",
                group: Group::Arpeggiator,
                min: 0,
                max: 5,
                kind: Kind::Continuous,
            },
            Self::FxRouting => Parameter {
                name: "FX Routing",
                group: Group::Effects,
                min: 0,
                max: 9,
                kind: Kind::Enumerated(TableId::FxRouting),
            },
            Self::Fx1Type => Parameter {
                name: "FX 1 Type",
                group: Group::Effects,
                min: 0,
                max: 34,
                kind: Kind::Enumerated(TableId::FxType),
            },
            Self::Fx1Param1 => Parameter {
                name: "FX 1 Param 1",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param2 => Parameter {
                name: "FX 1 Param 2",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param3 => Parameter {
                name: "FX 1 Param 3",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param4 => Parameter {
                name: "FX 1 Param 4",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param5 => Parameter {
                name: "FX 1 Param 5",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param6 => Parameter {
                name: "FX 1 Param 6",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param7 => Parameter {
                name: "FX 1 Param 7",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param8 => Parameter {
                name: "FX 1 Param 8",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param9 => Parameter {
                name: "FX 1 Param 9",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param10 => Parameter {
                name: "FX 1 Param 10",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param11 => Parameter {
                name: "FX 1 Param 11",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1Param12 => Parameter {
                name: "FX 1 Param 12",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Type => Parameter {
                name: "FX 2 Type",
                group: Group::Effects,
                min: 0,
                max: 34,
                kind: Kind::Enumerated(TableId::FxType),
            },
            Self::Fx2Param1 => Parameter {
                name: "FX 2 Param 1",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param2 => Parameter {
                name: "FX 2 Param 2",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param3 => Parameter {
                name: "FX 2 Param 3",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param4 => Parameter {
                name: "FX 2 Param 4",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param5 => Parameter {
                name: "FX 2 Param 5",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param6 => Parameter {
                name: "FX 2 Param 6",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param7 => Parameter {
                name: "FX 2 Param 7",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param8 => Parameter {
                name: "FX 2 Param 8",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param9 => Parameter {
                name: "FX 2 Param 9",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param10 => Parameter {
                name: "FX 2 Param 10",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param11 => Parameter {
                name: "FX 2 Param 11",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx2Param12 => Parameter {
                name: "FX 2 Param 12",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Type => Parameter {
                name: "FX 3 Type",
                group: Group::Effects,
                min: 0,
                max: 34,
                kind: Kind::Enumerated(TableId::FxType),
            },
            Self::Fx3Param1 => Parameter {
                name: "FX 3 Param 1",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param2 => Parameter {
                name: "FX 3 Param 2",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param3 => Parameter {
                name: "FX 3 Param 3",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param4 => Parameter {
                name: "FX 3 Param 4",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param5 => Parameter {
                name: "FX 3 Param 5",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param6 => Parameter {
                name: "FX 3 Param 6",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param7 => Parameter {
                name: "FX 3 Param 7",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param8 => Parameter {
                name: "FX 3 Param 8",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param9 => Parameter {
                name: "FX 3 Param 9",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param10 => Parameter {
                name: "FX 3 Param 10",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param11 => Parameter {
                name: "FX 3 Param 11",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx3Param12 => Parameter {
                name: "FX 3 Param 12",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Type => Parameter {
                name: "FX 4 Type",
                group: Group::Effects,
                min: 0,
                max: 34,
                kind: Kind::Enumerated(TableId::FxType),
            },
            Self::Fx4Param1 => Parameter {
                name: "FX 4 Param 1",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param2 => Parameter {
                name: "FX 4 Param 2",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param3 => Parameter {
                name: "FX 4 Param 3",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param4 => Parameter {
                name: "FX 4 Param 4",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param5 => Parameter {
                name: "FX 4 Param 5",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param6 => Parameter {
                name: "FX 4 Param 6",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param7 => Parameter {
                name: "FX 4 Param 7",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param8 => Parameter {
                name: "FX 4 Param 8",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param9 => Parameter {
                name: "FX 4 Param 9",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param10 => Parameter {
                name: "FX 4 Param 10",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param11 => Parameter {
                name: "FX 4 Param 11",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx4Param12 => Parameter {
                name: "FX 4 Param 12",
                group: Group::Effects,
                min: 0,
                max: 255,
                kind: Kind::Continuous,
            },
            Self::Fx1OutputGain => Parameter {
                name: "FX 1 Output Gain",
                group: Group::Effects,
                min: 0,
                max: 150,
                kind: Kind::Continuous,
            },
            Self::Fx2OutputGain => Parameter {
                name: "FX 2 Output Gain",
                group: Group::Effects,
                min: 0,
                max: 150,
                kind: Kind::Continuous,
            },
            Self::Fx3OutputGain => Parameter {
                name: "FX 3 Output Gain",
                group: Group::Effects,
                min: 0,
                max: 150,
                kind: Kind::Continuous,
            },
            Self::Fx4OutputGain => Parameter {
                name: "FX 4 Output Gain",
                group: Group::Effects,
                min: 0,
                max: 150,
                kind: Kind::Continuous,
            },
            Self::FxMode => Parameter {
                name: "FX Mode",
                group: Group::Effects,
                min: 0,
                max: 2,
                kind: Kind::Enumerated(TableId::FxMode),
            },
            Self::ProgramNameChar1 => Parameter {
                name: "Program Name Char 1",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar2 => Parameter {
                name: "Program Name Char 2",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar3 => Parameter {
                name: "Program Name Char 3",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar4 => Parameter {
                name: "Program Name Char 4",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar5 => Parameter {
                name: "Program Name Char 5",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar6 => Parameter {
                name: "Program Name Char 6",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar7 => Parameter {
                name: "Program Name Char 7",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar8 => Parameter {
                name: "Program Name Char 8",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar9 => Parameter {
                name: "Program Name Char 9",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar10 => Parameter {
                name: "Program Name Char 10",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar11 => Parameter {
                name: "Program Name Char 11",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar12 => Parameter {
                name: "Program Name Char 12",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar13 => Parameter {
                name: "Program Name Char 13",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar14 => Parameter {
                name: "Program Name Char 14",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar15 => Parameter {
                name: "Program Name Char 15",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar16 => Parameter {
                name: "Program Name Char 16",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramNameChar17 => Parameter {
                name: "Program Name Char 17",
                group: Group::Program,
                min: 0,
                max: 127,
                kind: Kind::Continuous,
            },
            Self::ProgramCategory => Parameter {
                name: "Program Category",
                group: Group::Program,
                min: 0,
                max: 16,
                kind: Kind::Enumerated(TableId::ProgramCategory),
            },
            Self::ProgramTranspose => Parameter {
                name: "Program Transpose",
                group: Group::Program,
                min: 80,
                max: 176,
                kind: Kind::Continuous,
            },
        }
    }
}

impl ParamId {
    /// Returns what this parameter's raw value means beyond its range.
    ///
    /// [`Shape::Unipolar`] for all but 45 of them, which keeps a host's
    /// "where does this control sit" code one path rather than an [`Option`]
    /// every caller unwraps the same way.
    #[must_use]
    pub const fn shape(self) -> Shape {
        match self {
            Self::PitchBendUpDepth | Self::PitchBendDownDepth => Shape::Bipolar { centre: 24 },
            Self::VcaPanSpread
            | Self::OscPortamentoBalance
            | Self::Mod1Depth
            | Self::Mod2Depth
            | Self::Mod3Depth
            | Self::Mod4Depth
            | Self::Mod5Depth
            | Self::Mod6Depth
            | Self::Mod7Depth
            | Self::Mod8Depth
            | Self::SeqStepValue1
            | Self::SeqStepValue2
            | Self::SeqStepValue3
            | Self::SeqStepValue4
            | Self::SeqStepValue5
            | Self::SeqStepValue6
            | Self::SeqStepValue7
            | Self::SeqStepValue8
            | Self::SeqStepValue9
            | Self::SeqStepValue10
            | Self::SeqStepValue11
            | Self::SeqStepValue12
            | Self::SeqStepValue13
            | Self::SeqStepValue14
            | Self::SeqStepValue15
            | Self::SeqStepValue16
            | Self::SeqStepValue17
            | Self::SeqStepValue18
            | Self::SeqStepValue19
            | Self::SeqStepValue20
            | Self::SeqStepValue21
            | Self::SeqStepValue22
            | Self::SeqStepValue23
            | Self::SeqStepValue24
            | Self::SeqStepValue25
            | Self::SeqStepValue26
            | Self::SeqStepValue27
            | Self::SeqStepValue28
            | Self::SeqStepValue29
            | Self::SeqStepValue30
            | Self::SeqStepValue31
            | Self::SeqStepValue32
            | Self::ProgramTranspose => Shape::Bipolar { centre: 128 },
            _ => Shape::Unipolar,
        }
    }

    /// Returns the value that means "not set" rather than a position in the
    /// range, where this parameter has one.
    ///
    /// `Some(0)` for a sequencer step, where zero is "skip this step" and not
    /// the smallest modulation it can apply: a strip that drew it as the
    /// smallest would state the wrong musical fact.
    #[must_use]
    pub const fn inactive(self) -> Option<u16> {
        match self {
            Self::SeqStepValue1
            | Self::SeqStepValue2
            | Self::SeqStepValue3
            | Self::SeqStepValue4
            | Self::SeqStepValue5
            | Self::SeqStepValue6
            | Self::SeqStepValue7
            | Self::SeqStepValue8
            | Self::SeqStepValue9
            | Self::SeqStepValue10
            | Self::SeqStepValue11
            | Self::SeqStepValue12
            | Self::SeqStepValue13
            | Self::SeqStepValue14
            | Self::SeqStepValue15
            | Self::SeqStepValue16
            | Self::SeqStepValue17
            | Self::SeqStepValue18
            | Self::SeqStepValue19
            | Self::SeqStepValue20
            | Self::SeqStepValue21
            | Self::SeqStepValue22
            | Self::SeqStepValue23
            | Self::SeqStepValue24
            | Self::SeqStepValue25
            | Self::SeqStepValue26
            | Self::SeqStepValue27
            | Self::SeqStepValue28
            | Self::SeqStepValue29
            | Self::SeqStepValue30
            | Self::SeqStepValue31
            | Self::SeqStepValue32 => Some(0),
            _ => None,
        }
    }

    /// Returns the parameter that says how many of a run are used, where one
    /// does.
    ///
    /// [`ParamId::SequenceLength`] for each of the 32 sequencer steps, so that
    /// a host drawing the run can dim what is not played instead of implying
    /// all of it is.
    #[must_use]
    pub const fn bounded_by(self) -> Option<Self> {
        match self {
            Self::SeqStepValue1
            | Self::SeqStepValue2
            | Self::SeqStepValue3
            | Self::SeqStepValue4
            | Self::SeqStepValue5
            | Self::SeqStepValue6
            | Self::SeqStepValue7
            | Self::SeqStepValue8
            | Self::SeqStepValue9
            | Self::SeqStepValue10
            | Self::SeqStepValue11
            | Self::SeqStepValue12
            | Self::SeqStepValue13
            | Self::SeqStepValue14
            | Self::SeqStepValue15
            | Self::SeqStepValue16
            | Self::SeqStepValue17
            | Self::SeqStepValue18
            | Self::SeqStepValue19
            | Self::SeqStepValue20
            | Self::SeqStepValue21
            | Self::SeqStepValue22
            | Self::SeqStepValue23
            | Self::SeqStepValue24
            | Self::SeqStepValue25
            | Self::SeqStepValue26
            | Self::SeqStepValue27
            | Self::SeqStepValue28
            | Self::SeqStepValue29
            | Self::SeqStepValue30
            | Self::SeqStepValue31
            | Self::SeqStepValue32 => Some(Self::SequenceLength),
            _ => None,
        }
    }
}

impl ParamId {
    /// Returns what the specification records about this parameter beyond its
    /// table row.
    ///
    /// The manual's own words, kept as prose because that is what they are: a
    /// sentence about when a rate becomes a clock division, or about a value
    /// that skips a step rather than sounding it. What a control has to act on
    /// is typed — see [`ParamId::shape`], [`ParamId::inactive`] and
    /// [`ParamId::bounded_by`] — and this is the rest of it, for a host with
    /// somewhere to print it.
    ///
    /// `None` for the 113 parameters the manual says nothing more about.
    #[must_use]
    pub const fn note(self) -> Option<&'static str> {
        match self {
            Self::Lfo1Rate => Some(
                "When LFO 1 Arp Sync is on, this selects a division of the master BPM from the LFO clock divider table instead of setting a free-running rate.",
            ),
            Self::Lfo1KeySync
            | Self::Lfo1ArpSync
            | Self::Lfo2KeySync
            | Self::Lfo2ArpSync
            | Self::Osc1PulseEnable
            | Self::Osc1SawEnable
            | Self::OscSyncEnable
            | Self::VcfBassBoost
            | Self::OscKeyDownReset
            | Self::CtrlSequencerEnable
            | Self::ArpOnOff
            | Self::ArpKeySync
            | Self::ArpHold => Some("Off (0), On (1)"),
            Self::Lfo2Rate => Some(
                "When LFO 2 Arp Sync is on, this selects a division of the master BPM from the LFO clock divider table instead of setting a free-running rate.",
            ),
            Self::VcaPanSpread
            | Self::OscPortamentoBalance
            | Self::Mod1Depth
            | Self::Mod2Depth
            | Self::Mod3Depth
            | Self::Mod4Depth
            | Self::Mod5Depth
            | Self::Mod6Depth
            | Self::Mod7Depth
            | Self::Mod8Depth => Some("-128 (0) to +127 (255)"),
            Self::UnisonDetune => Some("Sets the amount phatness!"),
            Self::SequenceLength => Some("1 (0) to 32 (31) steps"),
            Self::SequencerSwingTiming | Self::ArpSwing => {
                Some("0 is 50%, no swing. 255 is 75%, full swing. 66% is a triplet feel.")
            }
            Self::SeqStepValue1
            | Self::SeqStepValue2
            | Self::SeqStepValue3
            | Self::SeqStepValue4
            | Self::SeqStepValue5
            | Self::SeqStepValue6
            | Self::SeqStepValue7
            | Self::SeqStepValue8
            | Self::SeqStepValue9
            | Self::SeqStepValue10
            | Self::SeqStepValue11
            | Self::SeqStepValue12
            | Self::SeqStepValue13
            | Self::SeqStepValue14
            | Self::SeqStepValue15
            | Self::SeqStepValue16
            | Self::SeqStepValue17
            | Self::SeqStepValue18
            | Self::SeqStepValue19
            | Self::SeqStepValue20
            | Self::SeqStepValue21
            | Self::SeqStepValue22
            | Self::SeqStepValue23
            | Self::SeqStepValue24
            | Self::SeqStepValue25
            | Self::SeqStepValue26
            | Self::SeqStepValue27
            | Self::SeqStepValue28
            | Self::SeqStepValue29
            | Self::SeqStepValue30
            | Self::SeqStepValue31
            | Self::SeqStepValue32 => {
                Some("Bipolar step value -127 (1) to +127 (255). A value of 0 means \"skip step\".")
            }
            Self::ArpRateTempo => Some("20 bpm (0) to 275 bpm (255)"),
            Self::ArpOctaves => Some("1 to 6 octaves"),
            Self::Fx1Param1
            | Self::Fx1Param2
            | Self::Fx1Param3
            | Self::Fx1Param4
            | Self::Fx1Param5
            | Self::Fx1Param6
            | Self::Fx1Param7
            | Self::Fx1Param8
            | Self::Fx1Param9
            | Self::Fx1Param10
            | Self::Fx1Param11
            | Self::Fx1Param12 => Some("Meaning depends on FX 1 Type"),
            Self::Fx2Param1
            | Self::Fx2Param2
            | Self::Fx2Param3
            | Self::Fx2Param4
            | Self::Fx2Param5
            | Self::Fx2Param6
            | Self::Fx2Param7
            | Self::Fx2Param8
            | Self::Fx2Param9
            | Self::Fx2Param10
            | Self::Fx2Param11
            | Self::Fx2Param12 => Some("Meaning depends on FX 2 Type"),
            Self::Fx3Param1
            | Self::Fx3Param2
            | Self::Fx3Param3
            | Self::Fx3Param4
            | Self::Fx3Param5
            | Self::Fx3Param6
            | Self::Fx3Param7
            | Self::Fx3Param8
            | Self::Fx3Param9
            | Self::Fx3Param10
            | Self::Fx3Param11
            | Self::Fx3Param12 => Some("Meaning depends on FX 3 Type"),
            Self::Fx4Param1
            | Self::Fx4Param2
            | Self::Fx4Param3
            | Self::Fx4Param4
            | Self::Fx4Param5
            | Self::Fx4Param6
            | Self::Fx4Param7
            | Self::Fx4Param8
            | Self::Fx4Param9
            | Self::Fx4Param10
            | Self::Fx4Param11
            | Self::Fx4Param12 => Some("Meaning depends on FX 4 Type"),
            Self::ProgramNameChar1
            | Self::ProgramNameChar2
            | Self::ProgramNameChar3
            | Self::ProgramNameChar4
            | Self::ProgramNameChar5
            | Self::ProgramNameChar6
            | Self::ProgramNameChar7
            | Self::ProgramNameChar8
            | Self::ProgramNameChar9
            | Self::ProgramNameChar10
            | Self::ProgramNameChar11
            | Self::ProgramNameChar12
            | Self::ProgramNameChar13
            | Self::ProgramNameChar14
            | Self::ProgramNameChar15
            | Self::ProgramNameChar16
            | Self::ProgramNameChar17 => Some("Null-terminated 16 char ASCII string"),
            Self::ProgramTranspose => Some("-48 (80) ... 0 (128) ... +48 (176)"),
            _ => None,
        }
    }

    /// Returns the range the synthesizer's own display shows for this
    /// parameter, as the manual prints it.
    ///
    /// The smallest useful thing a panel can say about a byte whose curve
    /// nobody has measured: the reading stays raw, and this is what the two
    /// ends of it mean. The same answer [`FxSlot::min`](crate::effect::FxSlot::min)
    /// and [`FxSlot::max`](crate::effect::FxSlot::max) give for an effect slot,
    /// in one string because these are not all ranges — one of them has a
    /// discrete value before a range in it, and another is a sentence about two
    /// different behaviours.
    ///
    /// Not a conversion, and no promise that the curve between the ends is a
    /// straight line. `None` where the manual gives none, which is 216 of them.
    ///
    /// ```
    /// use deepmind_midi::param::ParamId;
    ///
    /// assert_eq!(
    ///     ParamId::VcfFrequency.display(),
    ///     Some("50.0 Hz to 20000.0 Hz"),
    /// );
    /// assert_eq!(ParamId::Lfo1SlewRate.display(), None);
    /// ```
    #[must_use]
    pub const fn display(self) -> Option<&'static str> {
        match self {
            Self::Lfo1Rate | Self::Lfo2Rate => {
                Some("0.041 Hz to 65.4 Hz, or up to 1280 Hz when driven from the modulation matrix")
            }
            Self::Lfo1DelayFade | Self::Lfo2DelayFade => Some("0.00 s to 6.59 s"),
            Self::Osc1PitchModDepth | Self::Osc2PitchModDepth => {
                Some("0.00 cents to 36.0 semitones, on a non-linear fader response")
            }
            Self::Osc1PwmDepth => Some(
                "50.0% to 99.0% pulse width when the source is Manual, otherwise 0 to plus or minus 49% modulation",
            ),
            Self::Osc2Level => Some("Off, then -48.0 dB to 0.0 dB"),
            Self::Osc2Pitch => Some("-12.0 to +12.0 semitones"),
            Self::Osc2ToneModDepth => Some(
                "50% to 100% tone modulation when the source is Manual, otherwise 0 to plus or minus 49%",
            ),
            Self::NoiseLevel => Some("Off, then -48.1 dB to 0.0 dB"),
            Self::PortamentoTime => Some("0.00 s to 10.00 s"),
            Self::PitchBendUpDepth => Some(
                "-24 to +24 semitones, where 24 is no bend. A negative depth inverts the wheel, so pushing up bends down.",
            ),
            Self::PitchBendDownDepth => Some(
                "-24 to +24 semitones, where 24 is no bend. A negative depth inverts the wheel, so pulling down bends up.",
            ),
            Self::VcfFrequency => Some("50.0 Hz to 20000.0 Hz"),
            Self::VcfHighPassFrequency => Some("20.0 Hz to 2000.0 Hz"),
            Self::VcfResonance
            | Self::VcfEnvelopeDepth
            | Self::VcfLfoDepth
            | Self::VcfKeyboardTracking => Some("0.0% to 100.0%"),
            Self::VcaLevel => Some("-12.0 dB to +6.0 dB"),
            Self::UnisonDetune => Some("plus or minus 0.0 to 50.0 cents"),
            Self::DriftRate => Some(
                "Each drift step lasts a random time between 25-50 ms at 0 and 2.5-5.0 s at 255",
            ),
            Self::SequencerSwingTiming | Self::ArpSwing => Some("50% to 75%"),
            Self::ArpRateTempo => Some("20.0 to 275.0 BPM"),
            _ => None,
        }
    }

    /// Returns why this parameter's row departs from what the manual prints,
    /// where it does.
    ///
    /// 37 rows do. The manual contradicts itself about a range, or runs two
    /// numbers together, and the specification records both the reading it took
    /// and the reason. Worth showing to somebody convinced the editor is wrong
    /// about a range, and worth reading beside
    /// [`ParamId::confirmed`](Self::confirmed).
    #[must_use]
    pub const fn correction(self) -> Option<&'static str> {
        match self {
            Self::Lfo1MonoMode => Some(
                "The manual prints 0-1, but its own note describes Poly (0), Mono (1) and SPREAD-1 (2) through SPREAD-254 (255), matching LFO 2 Mono Mode at offset 12.",
            ),
            Self::Osc1Range => {
                Some("The manual prints \"0-216' (0), 8' (1), 4' (2)\". The range is 0-2.")
            }
            Self::Osc2Range => Some("Same run-together as offset 14. The range is 0-2."),
            Self::PitchBendUpDepth => Some(
                "The NRPN table gives a range of 0-24, but section 8.4.4 states both pitch bend depths run from -24 to +24, which is 49 values. Encoded as 0-48 with 24 as zero, matching how the manual encodes Global Transpose (0-96 for -48 to +48). Needs confirming against hardware.",
            ),
            Self::PitchBendDownDepth => Some("Same as offset 36."),
            Self::Osc1PitchModMode => {
                Some("The manual prints \"0-10 (OSC1+2), 1 (OSC 1 Only)\". The range is 0-1.")
            }
            Self::Vcf2PoleMode => {
                Some("The manual prints \"0-14 Pole (0), 2 Pole (1)\". The range is 0-1.")
            }
            Self::VcaEnvelopeReleaseCurve => Some(
                "The manual repeats \"Attack Curve\" here. Offsets 58-61 are the attack, decay, sustain and release curves, matching the Env1 AtCur / DcyCur / SuSCur / RelCur modulation destinations.",
            ),
            Self::VcfEnvelopeReleaseCurve | Self::ModEnvelopeReleaseCurve => {
                Some("Same repeated-name error as offset 61.")
            }
            Self::Mod1Source
            | Self::Mod2Source
            | Self::Mod3Source
            | Self::Mod4Source
            | Self::Mod5Source
            | Self::Mod6Source
            | Self::Mod7Source
            | Self::Mod8Source => Some(
                "Firmware 1.1 added two modulation sources, raising the range from 0-22 to 0-24. The manual's NRPN table still prints the firmware 1.0 range.",
            ),
            Self::Mod1Destination
            | Self::Mod2Destination
            | Self::Mod3Destination
            | Self::Mod4Destination
            | Self::Mod5Destination
            | Self::Mod6Destination
            | Self::Mod7Destination
            | Self::Mod8Destination => Some(
                "Firmware 1.1 added three modulation destinations, raising the range from 0-129 to 0-132. The manual's NRPN table still prints the firmware 1.0 range.",
            ),
            Self::CtrlSequencerClockDivider => Some(
                "Section 8.1.8 lists twenty clock divisions against this range of 0-15. The range is left as printed and the value table is marked unconfirmed.",
            ),
            Self::SequenceLength => Some(
                "The manual runs the range and the first note value together as \"0-311 (0) to 32 (31) steps\". The range is 0-31.",
            ),
            Self::SequencerSwingTiming => Some(
                "The NRPN note reads \"0% (0) to 75% (25)\". Sections 8.1.7 and 8.1.8 give the swing range as 50% to 75%, so 0 is 50% and 255 is 75%.",
            ),
            Self::SeqStepValue9 | Self::SeqStepValue11 => Some(
                "The manual's table carries kind switch on this step alone, which its own range of 0-255 and its own note contradict. Read as the bipolar sweep every other step is.",
            ),
            Self::ArpSwing => Some("Same as offset 120."),
            Self::ArpOctaves => Some("The manual prints \"0-51 to 6 Octaves\". The range is 0-5."),
            Self::Fx1Type | Self::Fx2Type | Self::Fx3Type | Self::Fx4Type => Some(
                "Firmware 1.1 added the Vintage Pitch algorithm, raising the range from 0-33 to 0-34. The manual's NRPN table still prints the firmware 1.0 range.",
            ),
            _ => None,
        }
    }

    /// Returns whether the specification's reading of this parameter has been
    /// confirmed.
    ///
    /// `false` where a range or an ordering is inferred from the manual's prose
    /// because its own table contradicts itself, and no hardware has settled it;
    /// [`ParamId::correction`](Self::correction) is the reason in each case. A
    /// host with room to say so can mark the control rather than presenting a
    /// guess as a fact.
    #[must_use]
    pub const fn confirmed(self) -> bool {
        !matches!(self, Self::PitchBendUpDepth | Self::PitchBendDownDepth)
    }
}

impl ParamId {
    /// Returns what this parameter does, in a sentence.
    ///
    /// The answer to the question somebody points at a control to ask, which
    /// the name and the range on their own do not give: [`ParamId::name`] says
    /// `VCF Keyboard Tracking` and this says what happens when it is turned up.
    ///
    /// # Behind a feature
    ///
    /// `None` for every parameter unless the `descriptions` feature is on.
    /// With it on, every parameter has one.
    /// The signature does not change with the feature, so a host writes one
    /// code path: a caller given `None` draws the name and the range it already
    /// has.
    ///
    /// The feature is off by default because these 242 sentences are 31 kB
    /// of prose: free on a desktop host, real money on the microcontrollers
    /// this crate is also meant for, and so a cost that should land on whoever
    /// asked for it.
    ///
    /// # Where these come from
    ///
    /// Written for this specification against what the rest of it records, and
    /// *not* transcribed from the manual — unlike
    /// [`FxSlot::description`](crate::effect::FxSlot::description), which is the
    /// manual's own words. A parameter whose behaviour this specification does
    /// not establish has no sentence rather than a guessed one. See the
    /// `descriptions` key in `spec/parameters.toml`.
    ///
    /// ```
    /// use deepmind_midi::param::ParamId;
    ///
    /// let described = ParamId::VcfFrequency.description().is_some();
    /// assert_eq!(described, cfg!(feature = "descriptions"));
    /// ```
    #[must_use]
    pub const fn description(self) -> Option<&'static str> {
        #[cfg(feature = "descriptions")]
        {
            match self {
                Self::Lfo1Rate => Some(
                    "Sets how fast LFO 1 runs, which is what the modulation depths elsewhere in the program are a depth of.",
                ),
                Self::Lfo1DelayFade => Some(
                    "Sets how long LFO 1 takes to reach full depth after a note starts, so that vibrato can arrive rather than being there from the attack.",
                ),
                Self::Lfo1Shape => Some(
                    "Chooses the waveform LFO 1 runs: one of the four periodic shapes, or one of the two that pick a fresh random value each cycle.",
                ),
                Self::Lfo1KeySync => Some(
                    "Restarts LFO 1 at the start of its cycle on each new note, so every note is modulated the same way instead of catching the LFO wherever it had got to.",
                ),
                Self::Lfo1ArpSync => Some(
                    "Locks LFO 1 to the master tempo, which is what turns its rate into a choice of clock division rather than a speed.",
                ),
                Self::Lfo1MonoMode => Some(
                    "Chooses whether each voice gets its own copy of LFO 1, whether all voices share one, or whether the voices' copies are spread apart in phase.",
                ),
                Self::Lfo1SlewRate => Some(
                    "Rounds the corners off LFO 1's waveform, which turns a square into something that ramps between its two levels rather than jumping.",
                ),
                Self::Lfo2Rate => Some(
                    "Sets how fast LFO 2 runs, which is what the modulation depths elsewhere in the program are a depth of.",
                ),
                Self::Lfo2DelayFade => Some(
                    "Sets how long LFO 2 takes to reach full depth after a note starts, so that vibrato can arrive rather than being there from the attack.",
                ),
                Self::Lfo2Shape => Some(
                    "Chooses the waveform LFO 2 runs: one of the four periodic shapes, or one of the two that pick a fresh random value each cycle.",
                ),
                Self::Lfo2KeySync => Some(
                    "Restarts LFO 2 at the start of its cycle on each new note, so every note is modulated the same way instead of catching the LFO wherever it had got to.",
                ),
                Self::Lfo2ArpSync => Some(
                    "Locks LFO 2 to the master tempo, which is what turns its rate into a choice of clock division rather than a speed.",
                ),
                Self::Lfo2MonoMode => Some(
                    "Chooses whether each voice gets its own copy of LFO 2, whether all voices share one, or whether the voices' copies are spread apart in phase.",
                ),
                Self::Lfo2SlewRate => Some(
                    "Rounds the corners off LFO 2's waveform, which turns a square into something that ramps between its two levels rather than jumping.",
                ),
                Self::Osc1Range => Some(
                    "Sets the octave OSC 1 sounds at, in the organ footages the display prints: 16' is the lowest and 4' the highest.",
                ),
                Self::Osc2Range => Some(
                    "Sets the octave OSC 2 sounds at, in the organ footages the display prints: 16' is the lowest and 4' the highest.",
                ),
                Self::Osc1PwmSource => Some(
                    "Chooses what moves OSC 1's pulse width: a fixed setting, either LFO, or one of the three envelopes.",
                ),
                Self::Osc2ToneModSource => Some(
                    "Chooses what moves OSC 2's tone modulation: a fixed setting, either LFO, or one of the three envelopes.",
                ),
                Self::Osc1PulseEnable => Some(
                    "Adds OSC 1's pulse wave to the mix. It and the saw are independent, so either, both or neither can sound.",
                ),
                Self::Osc1SawEnable => Some(
                    "Adds OSC 1's sawtooth wave to the mix. It and the pulse are independent, so either, both or neither can sound.",
                ),
                Self::OscSyncEnable => Some(
                    "Hard-syncs the oscillators, so that one restarts each time the other completes a cycle and its own pitch becomes a timbre control rather than a note.",
                ),
                Self::Osc1PitchModDepth => Some(
                    "Sets how far the source chosen by OSC 1 Pitch Mod Select moves OSC 1's pitch.",
                ),
                Self::Osc1PitchModSelect => Some(
                    "Chooses what moves OSC 1's pitch: either LFO, one of the three envelopes, or an LFO taken unipolar so that it only bends one way.",
                ),
                Self::Osc1AftertouchToPitchModDepth => Some(
                    "Sets how much aftertouch adds to OSC 1's pitch modulation depth, so that leaning on a held key deepens the vibrato.",
                ),
                Self::Osc1ModWheelToPitchModDepth => Some(
                    "Sets how much the modulation wheel adds to OSC 1's pitch modulation depth.",
                ),
                Self::Osc1PwmDepth => Some(
                    "Sets how far OSC 1's pulse width moves. With a fixed source this is the width itself; with an LFO or an envelope it is how far that source sweeps it.",
                ),
                Self::Osc2Level => Some("Sets how much of OSC 2 reaches the filter."),
                Self::Osc2Pitch => Some(
                    "Tunes OSC 2 away from OSC 1, up to an octave either way, which is what a detune or an interval between the two is set with.",
                ),
                Self::Osc2ToneModDepth => Some(
                    "Sets how far OSC 2's tone modulation moves. With a fixed source this is the setting itself; with an LFO or an envelope it is how far that source sweeps it.",
                ),
                Self::Osc2PitchModDepth => Some(
                    "Sets how far the source chosen by OSC 2 Pitch Mod Select moves OSC 2's pitch.",
                ),
                Self::Osc2AftertouchToPitchModDepth => Some(
                    "Sets how much aftertouch adds to OSC 2's pitch modulation depth, so that leaning on a held key deepens the vibrato.",
                ),
                Self::Osc2ModWheelToPitchModDepth => Some(
                    "Sets how much the modulation wheel adds to OSC 2's pitch modulation depth.",
                ),
                Self::Osc2PitchModSelect => Some(
                    "Chooses what moves OSC 2's pitch: either LFO, one of the three envelopes, or an LFO taken unipolar so that it only bends one way.",
                ),
                Self::NoiseLevel => Some(
                    "Sets how much of the noise generator is mixed in with the oscillators ahead of the filter.",
                ),
                Self::PortamentoTime => Some(
                    "Sets how long a new note takes to slide to its pitch from the note before it.",
                ),
                Self::PortamentoMode => Some(
                    "Chooses how the slide behaves: whether it happens on every note or only where two overlap, whether its time or its rate is what stays fixed, and whether it is a fixed interval away rather than a slide at all.",
                ),
                Self::PitchBendUpDepth => {
                    Some("Sets how far the pitch bender bends when it is pushed up.")
                }
                Self::PitchBendDownDepth => {
                    Some("Sets how far the pitch bender bends when it is pulled down.")
                }
                Self::Osc1PitchModMode => Some(
                    "Chooses whether OSC 1's pitch modulation moves both oscillators together or OSC 1 alone.",
                ),
                Self::VcfFrequency => Some(
                    "Sets the cutoff frequency of the low pass filter, the point above which the oscillators' harmonics are removed. Turning it down darkens the sound.",
                ),
                Self::VcfHighPassFrequency => Some(
                    "Sets the cutoff of the high pass filter, which removes what is below it. The manual's block diagram places this after the VCA, so it acts on the mixed voices rather than on one.",
                ),
                Self::VcfResonance => Some(
                    "Emphasises the frequencies around the low pass cutoff, which sharpens the filter's peak and thins out what sits below it.",
                ),
                Self::VcfEnvelopeDepth => Some(
                    "Sets how far the VCF envelope moves the low pass cutoff, and so how much of the filter's sweep is played by the envelope rather than set by hand.",
                ),
                Self::VcfEnvelopeVelocitySensitivity => Some(
                    "Sets how much playing harder deepens the VCF envelope's effect on the cutoff.",
                ),
                Self::VcfPitchBendToFreqDepth => Some(
                    "Sets how much the pitch bender moves the low pass cutoff along with the pitch.",
                ),
                Self::VcfLfoDepth => {
                    Some("Sets how far the LFO chosen by VCF LFO Select moves the low pass cutoff.")
                }
                Self::VcfLfoSelect => {
                    Some("Chooses which of the two LFOs moves the low pass cutoff.")
                }
                Self::VcfAftertouchToLfoDepth => Some(
                    "Sets how much aftertouch adds to the LFO's effect on the cutoff, so that leaning on a held key opens up the filter's wobble.",
                ),
                Self::VcfModWheelToLfoDepth => Some(
                    "Sets how much the modulation wheel adds to the LFO's effect on the cutoff.",
                ),
                Self::VcfKeyboardTracking => Some(
                    "Sets how far the low pass cutoff follows the note played, so that high notes keep the brightness low ones have instead of being filtered away.",
                ),
                Self::VcfEnvelopePolarity => {
                    Some("Chooses whether the VCF envelope opens the filter or closes it.")
                }
                Self::Vcf2PoleMode => Some(
                    "Chooses the low pass filter's slope: four poles for the steeper, darker response, two for the gentler one.",
                ),
                Self::VcfBassBoost => Some(
                    "Lifts the low end after the filter, putting back the weight a resonant low pass takes out.",
                ),
                Self::VcaEnvelopeAttackTime
                | Self::VcfEnvelopeAttackTime
                | Self::ModEnvelopeAttackTime => Some(
                    "Sets how long the envelope takes to rise to full level once it is triggered.",
                ),
                Self::VcaEnvelopeDecayTime
                | Self::VcfEnvelopeDecayTime
                | Self::ModEnvelopeDecayTime => Some(
                    "Sets how long the envelope takes to fall from full level to its sustain level.",
                ),
                Self::VcaEnvelopeSustainLevel
                | Self::VcfEnvelopeSustainLevel
                | Self::ModEnvelopeSustainLevel => {
                    Some("Sets the level the envelope holds at for as long as the note is held.")
                }
                Self::VcaEnvelopeReleaseTime
                | Self::VcfEnvelopeReleaseTime
                | Self::ModEnvelopeReleaseTime => Some(
                    "Sets how long the envelope takes to fall back to nothing once the key is released.",
                ),
                Self::VcaEnvelopeTriggerMode
                | Self::VcfEnvelopeTriggerMode
                | Self::ModEnvelopeTriggerMode => Some(
                    "Chooses what triggers the envelope: a key, either LFO, a free-running loop, or a step of the control sequencer.",
                ),
                Self::VcaEnvelopeAttackCurve
                | Self::VcfEnvelopeAttackCurve
                | Self::ModEnvelopeAttackCurve => Some(
                    "Bends the attack segment away from a straight line, towards an exponential in either direction.",
                ),
                Self::VcaEnvelopeDecayCurve
                | Self::VcfEnvelopeDecayCurve
                | Self::ModEnvelopeDecayCurve => Some(
                    "Bends the decay segment away from a straight line, towards an exponential in either direction.",
                ),
                Self::VcaEnvelopeSustainCurve
                | Self::VcfEnvelopeSustainCurve
                | Self::ModEnvelopeSustainCurve => Some(
                    "Bends the sustain segment away from a straight line, towards an exponential in either direction.",
                ),
                Self::VcaEnvelopeReleaseCurve
                | Self::VcfEnvelopeReleaseCurve
                | Self::ModEnvelopeReleaseCurve => Some(
                    "Bends the release segment away from a straight line, towards an exponential in either direction.",
                ),
                Self::VcaLevel => Some(
                    "Sets the level the voice leaves the amplifier at, ahead of the high pass and the effects.",
                ),
                Self::VcaEnvelopeDepth => Some(
                    "Sets how far the VCA envelope moves the voice's level, and so how much of the loudness is played by the envelope rather than held flat.",
                ),
                Self::VcaEnvelopeVelocitySensitivity => {
                    Some("Sets how much playing harder raises the voice's level.")
                }
                Self::VcaPanSpread => Some(
                    "Spreads the voices across the stereo field, so that a chord is placed across it rather than stacked in the middle.",
                ),
                Self::VoicePriorityMode => Some(
                    "Chooses which note keeps a voice when more are held than there are voices: the lowest, the highest, or the most recently played.",
                ),
                Self::PolyphonyMode => Some(
                    "Chooses how the voices are handed out: one to a note, several stacked on each note in unison, a limited number of them at a time, or the whole instrument reduced to one voice.",
                ),
                Self::EnvelopeTriggerMode => Some(
                    "Chooses whether the envelopes restart on each new note or run on from where they are when notes overlap, and whether they run once through however long the key is held.",
                ),
                Self::UnisonDetune => Some(
                    "Sets how far the stacked voices of a unison mode are tuned apart from one another, which is what thickens the sound.",
                ),
                Self::VoiceDrift => Some(
                    "Sets how much drift is applied per voice, which is what keeps two voices playing the same note from being identical.",
                ),
                Self::ParameterDrift => Some(
                    "Sets how much drift is applied to parameter values, which is what keeps a setting from sounding exactly where it was left.",
                ),
                Self::DriftRate => {
                    Some("Sets how quickly drift moves from one random value to the next.")
                }
                Self::OscPortamentoBalance => Some(
                    "Sets how the portamento time is split between the two oscillators, so that one can arrive at the new note ahead of the other.",
                ),
                Self::OscKeyDownReset => Some(
                    "Restarts the oscillators' waveforms on each new note, so that every note begins from the same point in the cycle.",
                ),
                Self::Mod1Source
                | Self::Mod2Source
                | Self::Mod3Source
                | Self::Mod4Source
                | Self::Mod5Source
                | Self::Mod6Source
                | Self::Mod7Source
                | Self::Mod8Source => Some(
                    "Chooses what drives this modulation bus. Zero is off; the rest are the instrument's own controls, its envelopes, its LFOs and the control sequencer.",
                ),
                Self::Mod1Destination
                | Self::Mod2Destination
                | Self::Mod3Destination
                | Self::Mod4Destination
                | Self::Mod5Destination
                | Self::Mod6Destination
                | Self::Mod7Destination
                | Self::Mod8Destination => Some(
                    "Chooses what this modulation bus moves. Zero is off, and a destination names an abbreviation the display prints rather than a single parameter, so one of them can move several parameters together.",
                ),
                Self::Mod1Depth
                | Self::Mod2Depth
                | Self::Mod3Depth
                | Self::Mod4Depth
                | Self::Mod5Depth
                | Self::Mod6Depth
                | Self::Mod7Depth
                | Self::Mod8Depth => Some(
                    "Sets how far this bus moves its destination, and which way round: the value is signed about its centre, and below the centre it inverts what the source does.",
                ),
                Self::CtrlSequencerEnable => Some(
                    "Runs the control sequencer, the stepped modulation source the matrix can draw on.",
                ),
                Self::CtrlSequencerClockDivider => {
                    Some("Sets how fast the sequencer steps, as a division of the master tempo.")
                }
                Self::SequenceLength => Some(
                    "Sets how many of the 32 steps are played before the sequence returns to the first.",
                ),
                Self::SequencerSwingTiming => Some(
                    "Holds every second step back, which is what turns an even run of steps into a swung one.",
                ),
                Self::KeySyncAndLoop => Some(
                    "Chooses whether the sequence restarts on a new note, whether it repeats when it reaches its end, or both.",
                ),
                Self::SlewRate => Some(
                    "Smooths the jump from one step's value to the next, which turns a staircase into a slope.",
                ),
                Self::SeqStepValue1
                | Self::SeqStepValue2
                | Self::SeqStepValue3
                | Self::SeqStepValue4
                | Self::SeqStepValue5
                | Self::SeqStepValue6
                | Self::SeqStepValue7
                | Self::SeqStepValue8
                | Self::SeqStepValue9
                | Self::SeqStepValue10
                | Self::SeqStepValue11
                | Self::SeqStepValue12
                | Self::SeqStepValue13
                | Self::SeqStepValue14
                | Self::SeqStepValue15
                | Self::SeqStepValue16
                | Self::SeqStepValue17
                | Self::SeqStepValue18
                | Self::SeqStepValue19
                | Self::SeqStepValue20
                | Self::SeqStepValue21
                | Self::SeqStepValue22
                | Self::SeqStepValue23
                | Self::SeqStepValue24
                | Self::SeqStepValue25
                | Self::SeqStepValue26
                | Self::SeqStepValue27
                | Self::SeqStepValue28
                | Self::SeqStepValue29
                | Self::SeqStepValue30
                | Self::SeqStepValue31
                | Self::SeqStepValue32 => Some(
                    "Sets how far this step moves whatever the matrix points at it. The value is signed about its centre, so a step can modulate either way from nothing.",
                ),
                Self::ArpOnOff => Some("Runs the arpeggiator over the notes being held."),
                Self::ArpMode => Some(
                    "Chooses the order the held notes are played in: up, down, alternating, as they were played, at random, or all at once.",
                ),
                Self::ArpRateTempo => Some(
                    "Sets the master tempo, in beats per minute. The arpeggiator, the control sequencer and a tempo-locked LFO all divide it.",
                ),
                Self::ArpClock => {
                    Some("Sets how fast the arpeggiator steps, as a division of the master tempo.")
                }
                Self::ArpKeySync => Some(
                    "Restarts the arpeggiated pattern on each new note rather than letting it run on.",
                ),
                Self::ArpGateTime => Some(
                    "Sets how much of each step the note actually sounds for, from a short stab to a run that joins up.",
                ),
                Self::ArpHold => Some("Keeps the arpeggio running after the keys are let go."),
                Self::ArpPattern => Some(
                    "Chooses the rhythm the arpeggiator plays: which of its steps sound and which are rests.",
                ),
                Self::ArpSwing => Some(
                    "Holds every second step of the arpeggio back, which is what turns an even run into a swung one.",
                ),
                Self::ArpOctaves => Some(
                    "Sets how many octaves the arpeggio climbs through before it returns to the note it started on.",
                ),
                Self::FxRouting => Some(
                    "Chooses how the four effect engines are wired to each other: in a chain, side by side, or a mixture of the two, with two of the ten routings feeding a later engine back into an earlier one.",
                ),
                Self::Fx1Type | Self::Fx2Type | Self::Fx3Type | Self::Fx4Type => Some(
                    "Chooses which algorithm this engine runs, which is what decides what its twelve parameter bytes mean.",
                ),
                Self::Fx1Param1
                | Self::Fx1Param2
                | Self::Fx1Param3
                | Self::Fx1Param4
                | Self::Fx1Param5
                | Self::Fx1Param6
                | Self::Fx1Param7
                | Self::Fx1Param8
                | Self::Fx1Param9
                | Self::Fx1Param10
                | Self::Fx1Param11
                | Self::Fx1Param12
                | Self::Fx2Param1
                | Self::Fx2Param2
                | Self::Fx2Param3
                | Self::Fx2Param4
                | Self::Fx2Param5
                | Self::Fx2Param6
                | Self::Fx2Param7
                | Self::Fx2Param8
                | Self::Fx2Param9
                | Self::Fx2Param10
                | Self::Fx2Param11
                | Self::Fx2Param12
                | Self::Fx3Param1
                | Self::Fx3Param2
                | Self::Fx3Param3
                | Self::Fx3Param4
                | Self::Fx3Param5
                | Self::Fx3Param6
                | Self::Fx3Param7
                | Self::Fx3Param8
                | Self::Fx3Param9
                | Self::Fx3Param10
                | Self::Fx3Param11
                | Self::Fx3Param12
                | Self::Fx4Param1
                | Self::Fx4Param2
                | Self::Fx4Param3
                | Self::Fx4Param4
                | Self::Fx4Param5
                | Self::Fx4Param6
                | Self::Fx4Param7
                | Self::Fx4Param8
                | Self::Fx4Param9
                | Self::Fx4Param10
                | Self::Fx4Param11
                | Self::Fx4Param12 => Some(
                    "One of the engine's twelve parameter bytes. What it controls depends on the algorithm the engine is running, so on its own it has no meaning to show.",
                ),
                Self::Fx1OutputGain
                | Self::Fx2OutputGain
                | Self::Fx3OutputGain
                | Self::Fx4OutputGain => Some(
                    "Sets how much of this engine's output carries on, which for some routings is into the next engine and for others is to the instrument's output.",
                ),
                Self::FxMode => Some(
                    "Chooses what the effects do to the two signal paths: sit in the chain, be fed from a send alongside it, or be bypassed.",
                ),
                Self::ProgramNameChar1
                | Self::ProgramNameChar2
                | Self::ProgramNameChar3
                | Self::ProgramNameChar4
                | Self::ProgramNameChar5
                | Self::ProgramNameChar6
                | Self::ProgramNameChar7
                | Self::ProgramNameChar8
                | Self::ProgramNameChar9
                | Self::ProgramNameChar10
                | Self::ProgramNameChar11
                | Self::ProgramNameChar12
                | Self::ProgramNameChar13
                | Self::ProgramNameChar14
                | Self::ProgramNameChar15
                | Self::ProgramNameChar16
                | Self::ProgramNameChar17 => Some(
                    "One byte of the program's name, as an ASCII code: sixteen printable characters at most, and a zero where the name ends.",
                ),
                Self::ProgramCategory => Some(
                    "Tags the program with the kind of sound it is, which is what the instrument's own browser sorts and filters on.",
                ),
                Self::ProgramTranspose => {
                    Some("Shifts the whole program in semitones, either side of its centre.")
                }
            }
        }
        #[cfg(not(feature = "descriptions"))]
        {
            let _ = self;
            None
        }
    }
}

/// A named set of parameter values.
///
/// Three of these were renumbered by firmware 1.1 rather than extended, so an
/// identifier names one table per firmware version rather than one table;
/// [`TableId::table_for`] is what picks between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum TableId {
    /// Arpeggiator Clock Divider.
    ArpClock,
    /// Arpeggiator Mode.
    ArpMode,
    /// Arpeggiator Pattern.
    ArpPattern,
    /// Envelope Trigger Source.
    EnvelopeTrigger,
    /// FX Mode.
    FxMode,
    /// FX Connection Mode.
    FxRouting,
    /// FX Type.
    FxType,
    /// Envelope Trigger Mode.
    KeyAssignMode,
    /// LFO Clock Divider.
    LfoClock,
    /// LFO Mono Mode.
    LfoMonoMode,
    /// LFO Shape.
    LfoShape,
    /// Modulation Matrix Destination.
    ModDestination,
    /// Modulation Matrix Source.
    ModSource,
    /// OSC 1 Pitch Mod Mode.
    Osc1PitchModMode,
    /// Oscillator Range.
    OscRange,
    /// Oscillator Pitch Mod Source.
    PitchModSource,
    /// Polyphony Mode.
    PolyphonyMode,
    /// Portamento Mode.
    PortamentoMode,
    /// Program Category.
    ProgramCategory,
    /// OSC 1 PWM Source.
    PwmSource,
    /// Control Sequencer Clock Divider.
    SequencerClock,
    /// Control Sequencer Key Sync and Loop.
    SequencerSync,
    /// OSC 2 Tone Mod Source.
    ToneModSource,
    /// VCF Envelope Polarity.
    VcfEnvelopePolarity,
    /// VCF LFO Select.
    VcfLfoSelect,
    /// VCF Pole Mode.
    VcfPoleMode,
    /// Voice Priority Mode.
    VoicePriority,
}

impl TableId {
    /// Every value table identifier, in alphabetical order.
    pub const ALL: &'static [Self] = &[
        Self::ArpClock,
        Self::ArpMode,
        Self::ArpPattern,
        Self::EnvelopeTrigger,
        Self::FxMode,
        Self::FxRouting,
        Self::FxType,
        Self::KeyAssignMode,
        Self::LfoClock,
        Self::LfoMonoMode,
        Self::LfoShape,
        Self::ModDestination,
        Self::ModSource,
        Self::Osc1PitchModMode,
        Self::OscRange,
        Self::PitchModSource,
        Self::PolyphonyMode,
        Self::PortamentoMode,
        Self::ProgramCategory,
        Self::PwmSource,
        Self::SequencerClock,
        Self::SequencerSync,
        Self::ToneModSource,
        Self::VcfEnvelopePolarity,
        Self::VcfLfoSelect,
        Self::VcfPoleMode,
        Self::VoicePriority,
    ];

    /// Returns this table as the given firmware numbers it.
    #[must_use]
    pub const fn table_for(self, firmware: Version) -> &'static ValueTable {
        match self {
            Self::ArpClock => &ARP_CLOCK,
            Self::ArpMode => &ARP_MODE,
            Self::ArpPattern => &ARP_PATTERN,
            Self::EnvelopeTrigger => &ENVELOPE_TRIGGER,
            Self::FxMode => &FX_MODE,
            Self::FxRouting => &FX_ROUTING,
            Self::FxType => {
                if firmware.at_least(1, 1) {
                    &FX_TYPE_FW_1_1
                } else {
                    &FX_TYPE_FW_1_0
                }
            }
            Self::KeyAssignMode => &KEY_ASSIGN_MODE,
            Self::LfoClock => &LFO_CLOCK,
            Self::LfoMonoMode => &LFO_MONO_MODE,
            Self::LfoShape => &LFO_SHAPE,
            Self::ModDestination => {
                if firmware.at_least(1, 1) {
                    &MOD_DESTINATION_FW_1_1
                } else {
                    &MOD_DESTINATION_FW_1_0
                }
            }
            Self::ModSource => {
                if firmware.at_least(1, 1) {
                    &MOD_SOURCE_FW_1_1
                } else {
                    &MOD_SOURCE_FW_1_0
                }
            }
            Self::Osc1PitchModMode => &OSC1_PITCH_MOD_MODE,
            Self::OscRange => &OSC_RANGE,
            Self::PitchModSource => &PITCH_MOD_SOURCE,
            Self::PolyphonyMode => &POLYPHONY_MODE,
            Self::PortamentoMode => &PORTAMENTO_MODE,
            Self::ProgramCategory => &PROGRAM_CATEGORY,
            Self::PwmSource => &PWM_SOURCE,
            Self::SequencerClock => &SEQUENCER_CLOCK,
            Self::SequencerSync => &SEQUENCER_SYNC,
            Self::ToneModSource => &TONE_MOD_SOURCE,
            Self::VcfEnvelopePolarity => &VCF_ENVELOPE_POLARITY,
            Self::VcfLfoSelect => &VCF_LFO_SELECT,
            Self::VcfPoleMode => &VCF_POLE_MODE,
            Self::VoicePriority => &VOICE_PRIORITY,
        }
    }
}

static LFO_SHAPE: ValueTable = ValueTable {
    id: TableId::LfoShape,
    name: "LFO Shape",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Sine",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Triangle",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Square",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Ramp Up",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Ramp Down",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Sample & Hold",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Sample & Glide",
            parameters: &[],
        },
    ],
};

static OSC_RANGE: ValueTable = ValueTable {
    id: TableId::OscRange,
    name: "Oscillator Range",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "16'",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "8'",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "4'",
            parameters: &[],
        },
    ],
};

static PWM_SOURCE: ValueTable = ValueTable {
    id: TableId::PwmSource,
    name: "OSC 1 PWM Source",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Manual",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO 1",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "LFO 2",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "VCA Env",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "VCF Env",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Mod Env",
            parameters: &[],
        },
    ],
};

static TONE_MOD_SOURCE: ValueTable = ValueTable {
    id: TableId::ToneModSource,
    name: "OSC 2 Tone Mod Source",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Manual",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO 1",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "LFO 2",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "VCA Env",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "VCF Env",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Mod Env",
            parameters: &[],
        },
    ],
};

static PITCH_MOD_SOURCE: ValueTable = ValueTable {
    id: TableId::PitchModSource,
    name: "Oscillator Pitch Mod Source",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "LFO 1",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO 2",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "VCA Env",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "VCF Env",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Mod Env",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "LFO 1 Unipolar",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "LFO 2 Unipolar",
            parameters: &[],
        },
    ],
};

static PORTAMENTO_MODE: ValueTable = ValueTable {
    id: TableId::PortamentoMode,
    name: "Portamento Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Normal",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Fingered",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Fixed Rate",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Fixed Rate Fingered",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Exponential",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Exponential Fingered",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Fixed +2",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "Fixed -2",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "Fixed +5",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Fixed -5",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "Fixed +12",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "Fixed -12",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "Fixed +24",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "Fixed -24",
            parameters: &[],
        },
    ],
};

static OSC1_PITCH_MOD_MODE: ValueTable = ValueTable {
    id: TableId::Osc1PitchModMode,
    name: "OSC 1 Pitch Mod Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "OSC 1 + 2",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "OSC 1 only",
            parameters: &[],
        },
    ],
};

static VCF_LFO_SELECT: ValueTable = ValueTable {
    id: TableId::VcfLfoSelect,
    name: "VCF LFO Select",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "LFO 1",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO 2",
            parameters: &[],
        },
    ],
};

static VCF_ENVELOPE_POLARITY: ValueTable = ValueTable {
    id: TableId::VcfEnvelopePolarity,
    name: "VCF Envelope Polarity",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Negative",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Positive",
            parameters: &[],
        },
    ],
};

static VCF_POLE_MODE: ValueTable = ValueTable {
    id: TableId::VcfPoleMode,
    name: "VCF Pole Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "4 Pole",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "2 Pole",
            parameters: &[],
        },
    ],
};

static ENVELOPE_TRIGGER: ValueTable = ValueTable {
    id: TableId::EnvelopeTrigger,
    name: "Envelope Trigger Source",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Key",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO 1",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "LFO 2",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Loop",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Control Sequencer Step",
            parameters: &[],
        },
    ],
};

static VOICE_PRIORITY: ValueTable = ValueTable {
    id: TableId::VoicePriority,
    name: "Voice Priority Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Lowest",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Highest",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Last",
            parameters: &[],
        },
    ],
};

static POLYPHONY_MODE: ValueTable = ValueTable {
    id: TableId::PolyphonyMode,
    name: "Polyphony Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Poly",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Unison 2",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Unison 3",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Unison 4",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Unison 6",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Unison 12",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Mono",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "Mono 2",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "Mono 3",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Mono 4",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "Mono 6",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "Poly 6",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "Poly 8",
            parameters: &[],
        },
    ],
};

static KEY_ASSIGN_MODE: ValueTable = ValueTable {
    id: TableId::KeyAssignMode,
    name: "Envelope Trigger Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Mono",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Re-Trigger",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Legato",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "One-Shot",
            parameters: &[],
        },
    ],
};

static SEQUENCER_SYNC: ValueTable = ValueTable {
    id: TableId::SequencerSync,
    name: "Control Sequencer Key Sync and Loop",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Loop on",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Key sync on",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Loop and key sync on",
            parameters: &[],
        },
    ],
};

static ARP_MODE: ValueTable = ValueTable {
    id: TableId::ArpMode,
    name: "Arpeggiator Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Up",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Down",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Up & Down",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Up Inv",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Down Inv",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Up & Down Inv",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Up Alt",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "Down Alt",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "Random",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "As Played",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "Chord",
            parameters: &[],
        },
    ],
};

static FX_ROUTING: ValueTable = ValueTable {
    id: TableId::FxRouting,
    name: "FX Connection Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Serial 1-2-3-4",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Parallel 1/2, serial 3-4",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Parallel 1/2, parallel 3/4",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Parallel 1/2/3/4",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Parallel 1/2/3, serial 4",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Serial 1-2, parallel 3/4",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Serial 1, parallel 2/3/4",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "Parallel (serial 1-2-3)/4",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "Serial 3-4 feedback 4(1-2)",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Serial 4 feedback 4(1-2-3)",
            parameters: &[],
        },
    ],
};

static FX_MODE: ValueTable = ValueTable {
    id: TableId::FxMode,
    name: "FX Mode",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Insert",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Send",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Bypass",
            parameters: &[],
        },
    ],
};

static PROGRAM_CATEGORY: ValueTable = ValueTable {
    id: TableId::ProgramCategory,
    name: "Program Category",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "None",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Bass",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Pad",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Lead",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "Mono",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Poly",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Stab",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "SFX",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "Arp",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Seq",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "Perc",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "Ambient",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "Modular",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "User-1",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "User-2",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "User-3",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "User-4",
            parameters: &[],
        },
    ],
};

static LFO_MONO_MODE: ValueTable = ValueTable {
    id: TableId::LfoMonoMode,
    name: "LFO Mono Mode",
    confirmed: true,
    partial: true,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Poly",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Mono",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "SPREAD-1",
            parameters: &[],
        },
    ],
};

static ARP_PATTERN: ValueTable = ValueTable {
    id: TableId::ArpPattern,
    name: "Arpeggiator Pattern",
    confirmed: true,
    partial: true,
    entries: &[
        ValueEntry {
            value: 0,
            name: "None",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Preset-1",
            parameters: &[],
        },
        ValueEntry {
            value: 33,
            name: "User-1",
            parameters: &[],
        },
    ],
};

static ARP_CLOCK: ValueTable = ValueTable {
    id: TableId::ArpClock,
    name: "Arpeggiator Clock Divider",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "1/2",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "3/8",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "1/3",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "1/4",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "3/16",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "1/6",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "1/8",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "3/32",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "1/12",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "1/16",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "1/24",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "1/32",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "1/48",
            parameters: &[],
        },
    ],
};

static SEQUENCER_CLOCK: ValueTable = ValueTable {
    id: TableId::SequencerClock,
    name: "Control Sequencer Clock Divider",
    confirmed: false,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "4",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "3",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "2",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "1",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "1/2",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "3/8",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "1/3",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "1/4",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "3/16",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "1/6",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "1/8",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "3/32",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "1/12",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "1/16",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "3/64",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "1/24",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "1/32",
            parameters: &[],
        },
        ValueEntry {
            value: 17,
            name: "3/128",
            parameters: &[],
        },
        ValueEntry {
            value: 18,
            name: "1/48",
            parameters: &[],
        },
        ValueEntry {
            value: 19,
            name: "1/64",
            parameters: &[],
        },
    ],
};

static LFO_CLOCK: ValueTable = ValueTable {
    id: TableId::LfoClock,
    name: "LFO Clock Divider",
    confirmed: false,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "4",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "3",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "2",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "1",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "1/2",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "3/8",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "1/3",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "1/4",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "3/16",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "1/6",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "1/8",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "3/32",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "1/12",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "1/16",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "3/64",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "1/24",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "1/32",
            parameters: &[],
        },
        ValueEntry {
            value: 17,
            name: "3/128",
            parameters: &[],
        },
        ValueEntry {
            value: 18,
            name: "1/48",
            parameters: &[],
        },
        ValueEntry {
            value: 19,
            name: "1/64",
            parameters: &[],
        },
    ],
};

static MOD_SOURCE_FW_1_1: ValueTable = ValueTable {
    id: TableId::ModSource,
    name: "Modulation Matrix Source",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Off",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Pitch Bend",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Mod Wheel",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Foot Ctrl",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "BreathCtrl",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Pressure",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "Expression",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "LFO1",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "LFO2",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Env 1",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "Env 2",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "Env 3",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "Note Num",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "Note Vel",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "Note Off Vel",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "Ctrl Seq",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "LFO1 (Uni)",
            parameters: &[],
        },
        ValueEntry {
            value: 17,
            name: "LFO2 (Uni)",
            parameters: &[],
        },
        ValueEntry {
            value: 18,
            name: "LFO1 (Fade)",
            parameters: &[],
        },
        ValueEntry {
            value: 19,
            name: "LFO2 (Fade)",
            parameters: &[],
        },
        ValueEntry {
            value: 20,
            name: "Voice Num",
            parameters: &[],
        },
        ValueEntry {
            value: 21,
            name: "Uni Voice",
            parameters: &[],
        },
        ValueEntry {
            value: 22,
            name: "CC X (115)",
            parameters: &[],
        },
        ValueEntry {
            value: 23,
            name: "CC Y (116)",
            parameters: &[],
        },
        ValueEntry {
            value: 24,
            name: "CC Z (117)",
            parameters: &[],
        },
    ],
};

static MOD_SOURCE_FW_1_0: ValueTable = ValueTable {
    id: TableId::ModSource,
    name: "Modulation Matrix Source (firmware 1.0)",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Off",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "Pitch Bend",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "Mod Wheel",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "Foot Ctrl",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "BreathCtrl",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "Pressure",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "LFO1",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "LFO2",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "Env 1",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Env 2",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "Env 3",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "Note Num",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "Note Vel",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "Ctrl Seq",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "LFO1 (Uni)",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "LFO2 (Uni)",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "LFO1 (Fade)",
            parameters: &[],
        },
        ValueEntry {
            value: 17,
            name: "LFO2 (Fade)",
            parameters: &[],
        },
        ValueEntry {
            value: 18,
            name: "NoteOff Vel",
            parameters: &[],
        },
        ValueEntry {
            value: 19,
            name: "Voice Num",
            parameters: &[],
        },
        ValueEntry {
            value: 20,
            name: "CC X (114)",
            parameters: &[],
        },
        ValueEntry {
            value: 21,
            name: "CC Y (115)",
            parameters: &[],
        },
        ValueEntry {
            value: 22,
            name: "CC Z (116)",
            parameters: &[],
        },
    ],
};

static MOD_DESTINATION_FW_1_1: ValueTable = ValueTable {
    id: TableId::ModDestination,
    name: "Modulation Matrix Destination",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Off",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO1 Rate",
            parameters: &[ParamId::Lfo1Rate],
        },
        ValueEntry {
            value: 2,
            name: "LFO1 Delay",
            parameters: &[ParamId::Lfo1DelayFade],
        },
        ValueEntry {
            value: 3,
            name: "LFO1 Slew",
            parameters: &[ParamId::Lfo1SlewRate],
        },
        ValueEntry {
            value: 4,
            name: "LFO1 Shape",
            parameters: &[ParamId::Lfo1Shape],
        },
        ValueEntry {
            value: 5,
            name: "LFO2 Rate",
            parameters: &[ParamId::Lfo2Rate],
        },
        ValueEntry {
            value: 6,
            name: "LFO2 Delay",
            parameters: &[ParamId::Lfo2DelayFade],
        },
        ValueEntry {
            value: 7,
            name: "LFO2 Slew",
            parameters: &[ParamId::Lfo2SlewRate],
        },
        ValueEntry {
            value: 8,
            name: "LFO2 Shape",
            parameters: &[ParamId::Lfo2Shape],
        },
        ValueEntry {
            value: 9,
            name: "OSC1+2 Pit",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "OSC1+2 Fine",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "OSC1 Pitch",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "OSC1 Fine",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "OSC2 Pitch",
            parameters: &[ParamId::Osc2Pitch],
        },
        ValueEntry {
            value: 14,
            name: "OSC2 Fine",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "OSC1 PM Dep",
            parameters: &[ParamId::Osc1PitchModDepth],
        },
        ValueEntry {
            value: 16,
            name: "PWM Depth",
            parameters: &[ParamId::Osc1PwmDepth],
        },
        ValueEntry {
            value: 17,
            name: "TMod Depth",
            parameters: &[ParamId::Osc2ToneModDepth],
        },
        ValueEntry {
            value: 18,
            name: "OSC2 PM Dep",
            parameters: &[ParamId::Osc2PitchModDepth],
        },
        ValueEntry {
            value: 19,
            name: "Porta Time",
            parameters: &[ParamId::PortamentoTime],
        },
        ValueEntry {
            value: 20,
            name: "VCF Freq",
            parameters: &[ParamId::VcfFrequency],
        },
        ValueEntry {
            value: 21,
            name: "VCF Res",
            parameters: &[ParamId::VcfResonance],
        },
        ValueEntry {
            value: 22,
            name: "VCF Env",
            parameters: &[ParamId::VcfEnvelopeDepth],
        },
        ValueEntry {
            value: 23,
            name: "VCF LFO",
            parameters: &[ParamId::VcfLfoDepth],
        },
        ValueEntry {
            value: 24,
            name: "Env Rates",
            parameters: &[
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcaEnvelopeReleaseTime,
                ParamId::VcfEnvelopeAttackTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::VcfEnvelopeReleaseTime,
                ParamId::ModEnvelopeAttackTime,
                ParamId::ModEnvelopeDecayTime,
                ParamId::ModEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 25,
            name: "All Attack",
            parameters: &[
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcfEnvelopeAttackTime,
                ParamId::ModEnvelopeAttackTime,
            ],
        },
        ValueEntry {
            value: 26,
            name: "All Decay",
            parameters: &[
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::ModEnvelopeDecayTime,
            ],
        },
        ValueEntry {
            value: 27,
            name: "All Sus",
            parameters: &[
                ParamId::VcaEnvelopeSustainLevel,
                ParamId::VcfEnvelopeSustainLevel,
                ParamId::ModEnvelopeSustainLevel,
            ],
        },
        ValueEntry {
            value: 28,
            name: "All Rel",
            parameters: &[
                ParamId::VcaEnvelopeReleaseTime,
                ParamId::VcfEnvelopeReleaseTime,
                ParamId::ModEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 29,
            name: "Env1 Rates",
            parameters: &[
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcaEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 30,
            name: "Env2 Rates",
            parameters: &[
                ParamId::VcfEnvelopeAttackTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::VcfEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 31,
            name: "Env3 Rates",
            parameters: &[
                ParamId::ModEnvelopeAttackTime,
                ParamId::ModEnvelopeDecayTime,
                ParamId::ModEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 32,
            name: "Env1CurveS",
            parameters: &[
                ParamId::VcaEnvelopeAttackCurve,
                ParamId::VcaEnvelopeDecayCurve,
                ParamId::VcaEnvelopeSustainCurve,
                ParamId::VcaEnvelopeReleaseCurve,
            ],
        },
        ValueEntry {
            value: 33,
            name: "Env2CurveS",
            parameters: &[
                ParamId::VcfEnvelopeAttackCurve,
                ParamId::VcfEnvelopeDecayCurve,
                ParamId::VcfEnvelopeSustainCurve,
                ParamId::VcfEnvelopeReleaseCurve,
            ],
        },
        ValueEntry {
            value: 34,
            name: "Env3CurveS",
            parameters: &[
                ParamId::ModEnvelopeAttackCurve,
                ParamId::ModEnvelopeDecayCurve,
                ParamId::ModEnvelopeSustainCurve,
                ParamId::ModEnvelopeReleaseCurve,
            ],
        },
        ValueEntry {
            value: 35,
            name: "Env1 Attack",
            parameters: &[ParamId::VcaEnvelopeAttackTime],
        },
        ValueEntry {
            value: 36,
            name: "Env1 Decay",
            parameters: &[ParamId::VcaEnvelopeDecayTime],
        },
        ValueEntry {
            value: 37,
            name: "Env1 Sus",
            parameters: &[ParamId::VcaEnvelopeSustainLevel],
        },
        ValueEntry {
            value: 38,
            name: "Env1 Rel",
            parameters: &[ParamId::VcaEnvelopeReleaseTime],
        },
        ValueEntry {
            value: 39,
            name: "Env1 AtCur",
            parameters: &[ParamId::VcaEnvelopeAttackCurve],
        },
        ValueEntry {
            value: 40,
            name: "Env1 DcyCur",
            parameters: &[ParamId::VcaEnvelopeDecayCurve],
        },
        ValueEntry {
            value: 41,
            name: "Env1 SuSCur",
            parameters: &[ParamId::VcaEnvelopeSustainCurve],
        },
        ValueEntry {
            value: 42,
            name: "Env1 RelCur",
            parameters: &[ParamId::VcaEnvelopeReleaseCurve],
        },
        ValueEntry {
            value: 43,
            name: "Env2 Attack",
            parameters: &[ParamId::VcfEnvelopeAttackTime],
        },
        ValueEntry {
            value: 44,
            name: "Env2 Decay",
            parameters: &[ParamId::VcfEnvelopeDecayTime],
        },
        ValueEntry {
            value: 45,
            name: "Env2 Sus",
            parameters: &[ParamId::VcfEnvelopeSustainLevel],
        },
        ValueEntry {
            value: 46,
            name: "Env2 Rel",
            parameters: &[ParamId::VcfEnvelopeReleaseTime],
        },
        ValueEntry {
            value: 47,
            name: "Env2 AtCur",
            parameters: &[ParamId::VcfEnvelopeAttackCurve],
        },
        ValueEntry {
            value: 48,
            name: "Env2 DcyCur",
            parameters: &[ParamId::VcfEnvelopeDecayCurve],
        },
        ValueEntry {
            value: 49,
            name: "Env2 SuSCur",
            parameters: &[ParamId::VcfEnvelopeSustainCurve],
        },
        ValueEntry {
            value: 50,
            name: "Env2 RelCur",
            parameters: &[ParamId::VcfEnvelopeReleaseCurve],
        },
        ValueEntry {
            value: 51,
            name: "Env3 Attack",
            parameters: &[ParamId::ModEnvelopeAttackTime],
        },
        ValueEntry {
            value: 52,
            name: "Env3 Decay",
            parameters: &[ParamId::ModEnvelopeDecayTime],
        },
        ValueEntry {
            value: 53,
            name: "Env3 Sus",
            parameters: &[ParamId::ModEnvelopeSustainLevel],
        },
        ValueEntry {
            value: 54,
            name: "Env3 Rel",
            parameters: &[ParamId::ModEnvelopeReleaseTime],
        },
        ValueEntry {
            value: 55,
            name: "Env3 AtCur",
            parameters: &[ParamId::ModEnvelopeAttackCurve],
        },
        ValueEntry {
            value: 56,
            name: "Env3 DcyCur",
            parameters: &[ParamId::ModEnvelopeDecayCurve],
        },
        ValueEntry {
            value: 57,
            name: "Env3 SuSCur",
            parameters: &[ParamId::ModEnvelopeSustainCurve],
        },
        ValueEntry {
            value: 58,
            name: "Env3 RelCur",
            parameters: &[ParamId::ModEnvelopeReleaseCurve],
        },
        ValueEntry {
            value: 59,
            name: "VCA All",
            parameters: &[],
        },
        ValueEntry {
            value: 60,
            name: "VCA Active",
            parameters: &[],
        },
        ValueEntry {
            value: 61,
            name: "VCA EnvDep",
            parameters: &[ParamId::VcaEnvelopeDepth],
        },
        ValueEntry {
            value: 62,
            name: "Pan Spread",
            parameters: &[ParamId::VcaPanSpread],
        },
        ValueEntry {
            value: 63,
            name: "VCA Pan",
            parameters: &[],
        },
        ValueEntry {
            value: 64,
            name: "OSC2 Lvl",
            parameters: &[ParamId::Osc2Level],
        },
        ValueEntry {
            value: 65,
            name: "Noise Lvl",
            parameters: &[ParamId::NoiseLevel],
        },
        ValueEntry {
            value: 66,
            name: "HP Freq",
            parameters: &[ParamId::VcfHighPassFrequency],
        },
        ValueEntry {
            value: 67,
            name: "Uni Detune",
            parameters: &[ParamId::UnisonDetune],
        },
        ValueEntry {
            value: 68,
            name: "OSC Drift",
            parameters: &[ParamId::VoiceDrift],
        },
        ValueEntry {
            value: 69,
            name: "Param Drift",
            parameters: &[ParamId::ParameterDrift],
        },
        ValueEntry {
            value: 70,
            name: "Drift Rate",
            parameters: &[ParamId::DriftRate],
        },
        ValueEntry {
            value: 71,
            name: "Arp Gate",
            parameters: &[ParamId::ArpGateTime],
        },
        ValueEntry {
            value: 72,
            name: "Seq Slew",
            parameters: &[ParamId::SlewRate],
        },
        ValueEntry {
            value: 73,
            name: "Mod 1 Dep",
            parameters: &[ParamId::Mod1Depth],
        },
        ValueEntry {
            value: 74,
            name: "Mod 2 Dep",
            parameters: &[ParamId::Mod2Depth],
        },
        ValueEntry {
            value: 75,
            name: "Mod 3 Dep",
            parameters: &[ParamId::Mod3Depth],
        },
        ValueEntry {
            value: 76,
            name: "Mod 4 Dep",
            parameters: &[ParamId::Mod4Depth],
        },
        ValueEntry {
            value: 77,
            name: "Mod 5 Dep",
            parameters: &[ParamId::Mod5Depth],
        },
        ValueEntry {
            value: 78,
            name: "Mod 6 Dep",
            parameters: &[ParamId::Mod6Depth],
        },
        ValueEntry {
            value: 79,
            name: "Mod 7 Dep",
            parameters: &[ParamId::Mod7Depth],
        },
        ValueEntry {
            value: 80,
            name: "Mod 8 Dep",
            parameters: &[ParamId::Mod8Depth],
        },
        ValueEntry {
            value: 81,
            name: "Fx 1 Param 1",
            parameters: &[ParamId::Fx1Param1],
        },
        ValueEntry {
            value: 82,
            name: "Fx 1 Param 2",
            parameters: &[ParamId::Fx1Param2],
        },
        ValueEntry {
            value: 83,
            name: "Fx 1 Param 3",
            parameters: &[ParamId::Fx1Param3],
        },
        ValueEntry {
            value: 84,
            name: "Fx 1 Param 4",
            parameters: &[ParamId::Fx1Param4],
        },
        ValueEntry {
            value: 85,
            name: "Fx 1 Param 5",
            parameters: &[ParamId::Fx1Param5],
        },
        ValueEntry {
            value: 86,
            name: "Fx 1 Param 6",
            parameters: &[ParamId::Fx1Param6],
        },
        ValueEntry {
            value: 87,
            name: "Fx 1 Param 7",
            parameters: &[ParamId::Fx1Param7],
        },
        ValueEntry {
            value: 88,
            name: "Fx 1 Param 8",
            parameters: &[ParamId::Fx1Param8],
        },
        ValueEntry {
            value: 89,
            name: "Fx 1 Param 9",
            parameters: &[ParamId::Fx1Param9],
        },
        ValueEntry {
            value: 90,
            name: "Fx 1 Param 10",
            parameters: &[ParamId::Fx1Param10],
        },
        ValueEntry {
            value: 91,
            name: "Fx 1 Param 11",
            parameters: &[ParamId::Fx1Param11],
        },
        ValueEntry {
            value: 92,
            name: "Fx 1 Param 12",
            parameters: &[ParamId::Fx1Param12],
        },
        ValueEntry {
            value: 93,
            name: "Fx 2 Param 1",
            parameters: &[ParamId::Fx2Param1],
        },
        ValueEntry {
            value: 94,
            name: "Fx 2 Param 2",
            parameters: &[ParamId::Fx2Param2],
        },
        ValueEntry {
            value: 95,
            name: "Fx 2 Param 3",
            parameters: &[ParamId::Fx2Param3],
        },
        ValueEntry {
            value: 96,
            name: "Fx 2 Param 4",
            parameters: &[ParamId::Fx2Param4],
        },
        ValueEntry {
            value: 97,
            name: "Fx 2 Param 5",
            parameters: &[ParamId::Fx2Param5],
        },
        ValueEntry {
            value: 98,
            name: "Fx 2 Param 6",
            parameters: &[ParamId::Fx2Param6],
        },
        ValueEntry {
            value: 99,
            name: "Fx 2 Param 7",
            parameters: &[ParamId::Fx2Param7],
        },
        ValueEntry {
            value: 100,
            name: "Fx 2 Param 8",
            parameters: &[ParamId::Fx2Param8],
        },
        ValueEntry {
            value: 101,
            name: "Fx 2 Param 9",
            parameters: &[ParamId::Fx2Param9],
        },
        ValueEntry {
            value: 102,
            name: "Fx 2 Param 10",
            parameters: &[ParamId::Fx2Param10],
        },
        ValueEntry {
            value: 103,
            name: "Fx 2 Param 11",
            parameters: &[ParamId::Fx2Param11],
        },
        ValueEntry {
            value: 104,
            name: "Fx 2 Param 12",
            parameters: &[ParamId::Fx2Param12],
        },
        ValueEntry {
            value: 105,
            name: "Fx 3 Param 1",
            parameters: &[ParamId::Fx3Param1],
        },
        ValueEntry {
            value: 106,
            name: "Fx 3 Param 2",
            parameters: &[ParamId::Fx3Param2],
        },
        ValueEntry {
            value: 107,
            name: "Fx 3 Param 3",
            parameters: &[ParamId::Fx3Param3],
        },
        ValueEntry {
            value: 108,
            name: "Fx 3 Param 4",
            parameters: &[ParamId::Fx3Param4],
        },
        ValueEntry {
            value: 109,
            name: "Fx 3 Param 5",
            parameters: &[ParamId::Fx3Param5],
        },
        ValueEntry {
            value: 110,
            name: "Fx 3 Param 6",
            parameters: &[ParamId::Fx3Param6],
        },
        ValueEntry {
            value: 111,
            name: "Fx 3 Param 7",
            parameters: &[ParamId::Fx3Param7],
        },
        ValueEntry {
            value: 112,
            name: "Fx 3 Param 8",
            parameters: &[ParamId::Fx3Param8],
        },
        ValueEntry {
            value: 113,
            name: "Fx 3 Param 9",
            parameters: &[ParamId::Fx3Param9],
        },
        ValueEntry {
            value: 114,
            name: "Fx 3 Param 10",
            parameters: &[ParamId::Fx3Param10],
        },
        ValueEntry {
            value: 115,
            name: "Fx 3 Param 11",
            parameters: &[ParamId::Fx3Param11],
        },
        ValueEntry {
            value: 116,
            name: "Fx 3 Param 12",
            parameters: &[ParamId::Fx3Param12],
        },
        ValueEntry {
            value: 117,
            name: "Fx 4 Param 1",
            parameters: &[ParamId::Fx4Param1],
        },
        ValueEntry {
            value: 118,
            name: "Fx 4 Param 2",
            parameters: &[ParamId::Fx4Param2],
        },
        ValueEntry {
            value: 119,
            name: "Fx 4 Param 3",
            parameters: &[ParamId::Fx4Param3],
        },
        ValueEntry {
            value: 120,
            name: "Fx 4 Param 4",
            parameters: &[ParamId::Fx4Param4],
        },
        ValueEntry {
            value: 121,
            name: "Fx 4 Param 5",
            parameters: &[ParamId::Fx4Param5],
        },
        ValueEntry {
            value: 122,
            name: "Fx 4 Param 6",
            parameters: &[ParamId::Fx4Param6],
        },
        ValueEntry {
            value: 123,
            name: "Fx 4 Param 7",
            parameters: &[ParamId::Fx4Param7],
        },
        ValueEntry {
            value: 124,
            name: "Fx 4 Param 8",
            parameters: &[ParamId::Fx4Param8],
        },
        ValueEntry {
            value: 125,
            name: "Fx 4 Param 9",
            parameters: &[ParamId::Fx4Param9],
        },
        ValueEntry {
            value: 126,
            name: "Fx 4 Param 10",
            parameters: &[ParamId::Fx4Param10],
        },
        ValueEntry {
            value: 127,
            name: "Fx 4 Param 11",
            parameters: &[ParamId::Fx4Param11],
        },
        ValueEntry {
            value: 128,
            name: "Fx 4 Param 12",
            parameters: &[ParamId::Fx4Param12],
        },
        ValueEntry {
            value: 129,
            name: "Fx 1 Level",
            parameters: &[ParamId::Fx1OutputGain],
        },
        ValueEntry {
            value: 130,
            name: "Fx 2 Level",
            parameters: &[ParamId::Fx2OutputGain],
        },
        ValueEntry {
            value: 131,
            name: "Fx 3 Level",
            parameters: &[ParamId::Fx3OutputGain],
        },
        ValueEntry {
            value: 132,
            name: "Fx 4 Level",
            parameters: &[ParamId::Fx4OutputGain],
        },
    ],
};

static FX_TYPE_FW_1_1: ValueTable = ValueTable {
    id: TableId::FxType,
    name: "FX Type",
    confirmed: true,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "TC-DeepVRB",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "AmbVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "RoomRev",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "VintageRev",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "HallRev",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "ChamberRev",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "PlateRev",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "RichPltRev",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "GatedRev",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Reverse",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "ChorusVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "DelayVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "FlangVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "MidasEQ",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "Enhancer",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "FairComp",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "MulBndDist",
            parameters: &[],
        },
        ValueEntry {
            value: 17,
            name: "RackAmp",
            parameters: &[],
        },
        ValueEntry {
            value: 18,
            name: "EdisonEX1",
            parameters: &[],
        },
        ValueEntry {
            value: 19,
            name: "Auto Pan",
            parameters: &[],
        },
        ValueEntry {
            value: 20,
            name: "NoiseGate",
            parameters: &[],
        },
        ValueEntry {
            value: 21,
            name: "Delay",
            parameters: &[],
        },
        ValueEntry {
            value: 22,
            name: "3TapDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 23,
            name: "4TapDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 24,
            name: "T-RayDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 25,
            name: "DecimDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 26,
            name: "ModDlyRev",
            parameters: &[],
        },
        ValueEntry {
            value: 27,
            name: "Chorus",
            parameters: &[],
        },
        ValueEntry {
            value: 28,
            name: "Chorus-D",
            parameters: &[],
        },
        ValueEntry {
            value: 29,
            name: "Flanger",
            parameters: &[],
        },
        ValueEntry {
            value: 30,
            name: "Phaser",
            parameters: &[],
        },
        ValueEntry {
            value: 31,
            name: "MoodFilter",
            parameters: &[],
        },
        ValueEntry {
            value: 32,
            name: "DualPitch",
            parameters: &[],
        },
        ValueEntry {
            value: 33,
            name: "Vintage Pitch",
            parameters: &[],
        },
        ValueEntry {
            value: 34,
            name: "RotarySpkr",
            parameters: &[],
        },
    ],
};

static MOD_DESTINATION_FW_1_0: ValueTable = ValueTable {
    id: TableId::ModDestination,
    name: "Modulation Matrix Destination (firmware 1.0)",
    confirmed: false,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "Off",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "LFO1 Rate",
            parameters: &[ParamId::Lfo1Rate],
        },
        ValueEntry {
            value: 2,
            name: "LFO1 Delay",
            parameters: &[ParamId::Lfo1DelayFade],
        },
        ValueEntry {
            value: 3,
            name: "LFO1 Slew",
            parameters: &[ParamId::Lfo1SlewRate],
        },
        ValueEntry {
            value: 4,
            name: "LFO1 Shape",
            parameters: &[ParamId::Lfo1Shape],
        },
        ValueEntry {
            value: 5,
            name: "LFO2 Rate",
            parameters: &[ParamId::Lfo2Rate],
        },
        ValueEntry {
            value: 6,
            name: "LFO2 Delay",
            parameters: &[ParamId::Lfo2DelayFade],
        },
        ValueEntry {
            value: 7,
            name: "LFO2 Slew",
            parameters: &[ParamId::Lfo2SlewRate],
        },
        ValueEntry {
            value: 8,
            name: "LFO2 Shape",
            parameters: &[ParamId::Lfo2Shape],
        },
        ValueEntry {
            value: 9,
            name: "OSC1+2 Pit",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "OSC1 Pitch",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "OSC2 Pitch",
            parameters: &[ParamId::Osc2Pitch],
        },
        ValueEntry {
            value: 12,
            name: "OSC1 PM Dep",
            parameters: &[ParamId::Osc1PitchModDepth],
        },
        ValueEntry {
            value: 13,
            name: "PWM Depth",
            parameters: &[ParamId::Osc1PwmDepth],
        },
        ValueEntry {
            value: 14,
            name: "TMod Depth",
            parameters: &[ParamId::Osc2ToneModDepth],
        },
        ValueEntry {
            value: 15,
            name: "OSC2 PM Dep",
            parameters: &[ParamId::Osc2PitchModDepth],
        },
        ValueEntry {
            value: 16,
            name: "Porta Time",
            parameters: &[ParamId::PortamentoTime],
        },
        ValueEntry {
            value: 17,
            name: "VCF Freq",
            parameters: &[ParamId::VcfFrequency],
        },
        ValueEntry {
            value: 18,
            name: "VCF Res",
            parameters: &[ParamId::VcfResonance],
        },
        ValueEntry {
            value: 19,
            name: "VCF Env",
            parameters: &[ParamId::VcfEnvelopeDepth],
        },
        ValueEntry {
            value: 20,
            name: "VCF LFO",
            parameters: &[ParamId::VcfLfoDepth],
        },
        ValueEntry {
            value: 21,
            name: "Env Rates",
            parameters: &[
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcaEnvelopeReleaseTime,
                ParamId::VcfEnvelopeAttackTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::VcfEnvelopeReleaseTime,
                ParamId::ModEnvelopeAttackTime,
                ParamId::ModEnvelopeDecayTime,
                ParamId::ModEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 22,
            name: "All Attack",
            parameters: &[
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcfEnvelopeAttackTime,
                ParamId::ModEnvelopeAttackTime,
            ],
        },
        ValueEntry {
            value: 23,
            name: "All Decay",
            parameters: &[
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::ModEnvelopeDecayTime,
            ],
        },
        ValueEntry {
            value: 24,
            name: "All Sus",
            parameters: &[
                ParamId::VcaEnvelopeSustainLevel,
                ParamId::VcfEnvelopeSustainLevel,
                ParamId::ModEnvelopeSustainLevel,
            ],
        },
        ValueEntry {
            value: 25,
            name: "All Rel",
            parameters: &[
                ParamId::VcaEnvelopeReleaseTime,
                ParamId::VcfEnvelopeReleaseTime,
                ParamId::ModEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 26,
            name: "Env1 Rates",
            parameters: &[
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcaEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 27,
            name: "Env2 Rates",
            parameters: &[
                ParamId::VcfEnvelopeAttackTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::VcfEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 28,
            name: "Env3 Rates",
            parameters: &[
                ParamId::ModEnvelopeAttackTime,
                ParamId::ModEnvelopeDecayTime,
                ParamId::ModEnvelopeReleaseTime,
            ],
        },
        ValueEntry {
            value: 29,
            name: "Env1CurveS",
            parameters: &[
                ParamId::VcaEnvelopeAttackCurve,
                ParamId::VcaEnvelopeDecayCurve,
                ParamId::VcaEnvelopeSustainCurve,
                ParamId::VcaEnvelopeReleaseCurve,
            ],
        },
        ValueEntry {
            value: 30,
            name: "Env2CurveS",
            parameters: &[
                ParamId::VcfEnvelopeAttackCurve,
                ParamId::VcfEnvelopeDecayCurve,
                ParamId::VcfEnvelopeSustainCurve,
                ParamId::VcfEnvelopeReleaseCurve,
            ],
        },
        ValueEntry {
            value: 31,
            name: "Env3CurveS",
            parameters: &[
                ParamId::ModEnvelopeAttackCurve,
                ParamId::ModEnvelopeDecayCurve,
                ParamId::ModEnvelopeSustainCurve,
                ParamId::ModEnvelopeReleaseCurve,
            ],
        },
        ValueEntry {
            value: 32,
            name: "Env1 Attack",
            parameters: &[ParamId::VcaEnvelopeAttackTime],
        },
        ValueEntry {
            value: 33,
            name: "Env1 Decay",
            parameters: &[ParamId::VcaEnvelopeDecayTime],
        },
        ValueEntry {
            value: 34,
            name: "Env1 Sus",
            parameters: &[ParamId::VcaEnvelopeSustainLevel],
        },
        ValueEntry {
            value: 35,
            name: "Env1 Rel",
            parameters: &[ParamId::VcaEnvelopeReleaseTime],
        },
        ValueEntry {
            value: 36,
            name: "Env1 AtCur",
            parameters: &[ParamId::VcaEnvelopeAttackCurve],
        },
        ValueEntry {
            value: 37,
            name: "Env1 DcyCur",
            parameters: &[ParamId::VcaEnvelopeDecayCurve],
        },
        ValueEntry {
            value: 38,
            name: "Env1 SuSCur",
            parameters: &[ParamId::VcaEnvelopeSustainCurve],
        },
        ValueEntry {
            value: 39,
            name: "Env1 RelCur",
            parameters: &[ParamId::VcaEnvelopeReleaseCurve],
        },
        ValueEntry {
            value: 40,
            name: "Env2 Attack",
            parameters: &[ParamId::VcfEnvelopeAttackTime],
        },
        ValueEntry {
            value: 41,
            name: "Env2 Decay",
            parameters: &[ParamId::VcfEnvelopeDecayTime],
        },
        ValueEntry {
            value: 42,
            name: "Env2 Sus",
            parameters: &[ParamId::VcfEnvelopeSustainLevel],
        },
        ValueEntry {
            value: 43,
            name: "Env2 Rel",
            parameters: &[ParamId::VcfEnvelopeReleaseTime],
        },
        ValueEntry {
            value: 44,
            name: "Env2 AtCur",
            parameters: &[ParamId::VcfEnvelopeAttackCurve],
        },
        ValueEntry {
            value: 45,
            name: "Env2 DcyCur",
            parameters: &[ParamId::VcfEnvelopeDecayCurve],
        },
        ValueEntry {
            value: 46,
            name: "Env2 SuSCur",
            parameters: &[ParamId::VcfEnvelopeSustainCurve],
        },
        ValueEntry {
            value: 47,
            name: "Env2 RelCur",
            parameters: &[ParamId::VcfEnvelopeReleaseCurve],
        },
        ValueEntry {
            value: 48,
            name: "Env3 Attack",
            parameters: &[ParamId::ModEnvelopeAttackTime],
        },
        ValueEntry {
            value: 49,
            name: "Env3 Decay",
            parameters: &[ParamId::ModEnvelopeDecayTime],
        },
        ValueEntry {
            value: 50,
            name: "Env3 Sus",
            parameters: &[ParamId::ModEnvelopeSustainLevel],
        },
        ValueEntry {
            value: 51,
            name: "Env3 Rel",
            parameters: &[ParamId::ModEnvelopeReleaseTime],
        },
        ValueEntry {
            value: 52,
            name: "Env3 AtCur",
            parameters: &[ParamId::ModEnvelopeAttackCurve],
        },
        ValueEntry {
            value: 53,
            name: "Env3 DcyCur",
            parameters: &[ParamId::ModEnvelopeDecayCurve],
        },
        ValueEntry {
            value: 54,
            name: "Env3 SuSCur",
            parameters: &[ParamId::ModEnvelopeSustainCurve],
        },
        ValueEntry {
            value: 55,
            name: "Env3 RelCur",
            parameters: &[ParamId::ModEnvelopeReleaseCurve],
        },
        ValueEntry {
            value: 56,
            name: "VCA All",
            parameters: &[],
        },
        ValueEntry {
            value: 57,
            name: "VCA Active",
            parameters: &[],
        },
        ValueEntry {
            value: 58,
            name: "VCA EnvDep",
            parameters: &[ParamId::VcaEnvelopeDepth],
        },
        ValueEntry {
            value: 59,
            name: "Pan Spread",
            parameters: &[ParamId::VcaPanSpread],
        },
        ValueEntry {
            value: 60,
            name: "VCA Pan",
            parameters: &[],
        },
        ValueEntry {
            value: 61,
            name: "OSC2 Lvl",
            parameters: &[ParamId::Osc2Level],
        },
        ValueEntry {
            value: 62,
            name: "Noise Lvl",
            parameters: &[ParamId::NoiseLevel],
        },
        ValueEntry {
            value: 63,
            name: "HP Freq",
            parameters: &[ParamId::VcfHighPassFrequency],
        },
        ValueEntry {
            value: 64,
            name: "Uni Detune",
            parameters: &[ParamId::UnisonDetune],
        },
        ValueEntry {
            value: 65,
            name: "OSC Drift",
            parameters: &[ParamId::VoiceDrift],
        },
        ValueEntry {
            value: 66,
            name: "Param Drift",
            parameters: &[ParamId::ParameterDrift],
        },
        ValueEntry {
            value: 67,
            name: "Drift Rate",
            parameters: &[ParamId::DriftRate],
        },
        ValueEntry {
            value: 68,
            name: "Arp Gate",
            parameters: &[ParamId::ArpGateTime],
        },
        ValueEntry {
            value: 69,
            name: "Seq Slew",
            parameters: &[ParamId::SlewRate],
        },
        ValueEntry {
            value: 70,
            name: "Mod 1 Dep",
            parameters: &[ParamId::Mod1Depth],
        },
        ValueEntry {
            value: 71,
            name: "Mod 2 Dep",
            parameters: &[ParamId::Mod2Depth],
        },
        ValueEntry {
            value: 72,
            name: "Mod 3 Dep",
            parameters: &[ParamId::Mod3Depth],
        },
        ValueEntry {
            value: 73,
            name: "Mod 4 Dep",
            parameters: &[ParamId::Mod4Depth],
        },
        ValueEntry {
            value: 74,
            name: "Mod 5 Dep",
            parameters: &[ParamId::Mod5Depth],
        },
        ValueEntry {
            value: 75,
            name: "Mod 6 Dep",
            parameters: &[ParamId::Mod6Depth],
        },
        ValueEntry {
            value: 76,
            name: "Mod 7 Dep",
            parameters: &[ParamId::Mod7Depth],
        },
        ValueEntry {
            value: 77,
            name: "Mod 8 Dep",
            parameters: &[ParamId::Mod8Depth],
        },
        ValueEntry {
            value: 78,
            name: "Fx 1 Param 1",
            parameters: &[ParamId::Fx1Param1],
        },
        ValueEntry {
            value: 79,
            name: "Fx 1 Param 2",
            parameters: &[ParamId::Fx1Param2],
        },
        ValueEntry {
            value: 80,
            name: "Fx 1 Param 3",
            parameters: &[ParamId::Fx1Param3],
        },
        ValueEntry {
            value: 81,
            name: "Fx 1 Param 4",
            parameters: &[ParamId::Fx1Param4],
        },
        ValueEntry {
            value: 82,
            name: "Fx 1 Param 5",
            parameters: &[ParamId::Fx1Param5],
        },
        ValueEntry {
            value: 83,
            name: "Fx 1 Param 6",
            parameters: &[ParamId::Fx1Param6],
        },
        ValueEntry {
            value: 84,
            name: "Fx 1 Param 7",
            parameters: &[ParamId::Fx1Param7],
        },
        ValueEntry {
            value: 85,
            name: "Fx 1 Param 8",
            parameters: &[ParamId::Fx1Param8],
        },
        ValueEntry {
            value: 86,
            name: "Fx 1 Param 9",
            parameters: &[ParamId::Fx1Param9],
        },
        ValueEntry {
            value: 87,
            name: "Fx 1 Param 10",
            parameters: &[ParamId::Fx1Param10],
        },
        ValueEntry {
            value: 88,
            name: "Fx 1 Param 11",
            parameters: &[ParamId::Fx1Param11],
        },
        ValueEntry {
            value: 89,
            name: "Fx 1 Param 12",
            parameters: &[ParamId::Fx1Param12],
        },
        ValueEntry {
            value: 90,
            name: "Fx 2 Param 1",
            parameters: &[ParamId::Fx2Param1],
        },
        ValueEntry {
            value: 91,
            name: "Fx 2 Param 2",
            parameters: &[ParamId::Fx2Param2],
        },
        ValueEntry {
            value: 92,
            name: "Fx 2 Param 3",
            parameters: &[ParamId::Fx2Param3],
        },
        ValueEntry {
            value: 93,
            name: "Fx 2 Param 4",
            parameters: &[ParamId::Fx2Param4],
        },
        ValueEntry {
            value: 94,
            name: "Fx 2 Param 5",
            parameters: &[ParamId::Fx2Param5],
        },
        ValueEntry {
            value: 95,
            name: "Fx 2 Param 6",
            parameters: &[ParamId::Fx2Param6],
        },
        ValueEntry {
            value: 96,
            name: "Fx 2 Param 7",
            parameters: &[ParamId::Fx2Param7],
        },
        ValueEntry {
            value: 97,
            name: "Fx 2 Param 8",
            parameters: &[ParamId::Fx2Param8],
        },
        ValueEntry {
            value: 98,
            name: "Fx 2 Param 9",
            parameters: &[ParamId::Fx2Param9],
        },
        ValueEntry {
            value: 99,
            name: "Fx 2 Param 10",
            parameters: &[ParamId::Fx2Param10],
        },
        ValueEntry {
            value: 100,
            name: "Fx 2 Param 11",
            parameters: &[ParamId::Fx2Param11],
        },
        ValueEntry {
            value: 101,
            name: "Fx 2 Param 12",
            parameters: &[ParamId::Fx2Param12],
        },
        ValueEntry {
            value: 102,
            name: "Fx 3 Param 1",
            parameters: &[ParamId::Fx3Param1],
        },
        ValueEntry {
            value: 103,
            name: "Fx 3 Param 2",
            parameters: &[ParamId::Fx3Param2],
        },
        ValueEntry {
            value: 104,
            name: "Fx 3 Param 3",
            parameters: &[ParamId::Fx3Param3],
        },
        ValueEntry {
            value: 105,
            name: "Fx 3 Param 4",
            parameters: &[ParamId::Fx3Param4],
        },
        ValueEntry {
            value: 106,
            name: "Fx 3 Param 5",
            parameters: &[ParamId::Fx3Param5],
        },
        ValueEntry {
            value: 107,
            name: "Fx 3 Param 6",
            parameters: &[ParamId::Fx3Param6],
        },
        ValueEntry {
            value: 108,
            name: "Fx 3 Param 7",
            parameters: &[ParamId::Fx3Param7],
        },
        ValueEntry {
            value: 109,
            name: "Fx 3 Param 8",
            parameters: &[ParamId::Fx3Param8],
        },
        ValueEntry {
            value: 110,
            name: "Fx 3 Param 9",
            parameters: &[ParamId::Fx3Param9],
        },
        ValueEntry {
            value: 111,
            name: "Fx 3 Param 10",
            parameters: &[ParamId::Fx3Param10],
        },
        ValueEntry {
            value: 112,
            name: "Fx 3 Param 11",
            parameters: &[ParamId::Fx3Param11],
        },
        ValueEntry {
            value: 113,
            name: "Fx 3 Param 12",
            parameters: &[ParamId::Fx3Param12],
        },
        ValueEntry {
            value: 114,
            name: "Fx 4 Param 1",
            parameters: &[ParamId::Fx4Param1],
        },
        ValueEntry {
            value: 115,
            name: "Fx 4 Param 2",
            parameters: &[ParamId::Fx4Param2],
        },
        ValueEntry {
            value: 116,
            name: "Fx 4 Param 3",
            parameters: &[ParamId::Fx4Param3],
        },
        ValueEntry {
            value: 117,
            name: "Fx 4 Param 4",
            parameters: &[ParamId::Fx4Param4],
        },
        ValueEntry {
            value: 118,
            name: "Fx 4 Param 5",
            parameters: &[ParamId::Fx4Param5],
        },
        ValueEntry {
            value: 119,
            name: "Fx 4 Param 6",
            parameters: &[ParamId::Fx4Param6],
        },
        ValueEntry {
            value: 120,
            name: "Fx 4 Param 7",
            parameters: &[ParamId::Fx4Param7],
        },
        ValueEntry {
            value: 121,
            name: "Fx 4 Param 8",
            parameters: &[ParamId::Fx4Param8],
        },
        ValueEntry {
            value: 122,
            name: "Fx 4 Param 9",
            parameters: &[ParamId::Fx4Param9],
        },
        ValueEntry {
            value: 123,
            name: "Fx 4 Param 10",
            parameters: &[ParamId::Fx4Param10],
        },
        ValueEntry {
            value: 124,
            name: "Fx 4 Param 11",
            parameters: &[ParamId::Fx4Param11],
        },
        ValueEntry {
            value: 125,
            name: "Fx 4 Param 12",
            parameters: &[ParamId::Fx4Param12],
        },
        ValueEntry {
            value: 126,
            name: "Fx 1 Level",
            parameters: &[ParamId::Fx1OutputGain],
        },
        ValueEntry {
            value: 127,
            name: "Fx 2 Level",
            parameters: &[ParamId::Fx2OutputGain],
        },
        ValueEntry {
            value: 128,
            name: "Fx 3 Level",
            parameters: &[ParamId::Fx3OutputGain],
        },
        ValueEntry {
            value: 129,
            name: "Fx 4 Level",
            parameters: &[ParamId::Fx4OutputGain],
        },
    ],
};

static FX_TYPE_FW_1_0: ValueTable = ValueTable {
    id: TableId::FxType,
    name: "FX Type (firmware 1.0)",
    confirmed: false,
    partial: false,
    entries: &[
        ValueEntry {
            value: 0,
            name: "TC-DeepVRB",
            parameters: &[],
        },
        ValueEntry {
            value: 1,
            name: "AmbVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 2,
            name: "RoomRev",
            parameters: &[],
        },
        ValueEntry {
            value: 3,
            name: "VintageRev",
            parameters: &[],
        },
        ValueEntry {
            value: 4,
            name: "HallRev",
            parameters: &[],
        },
        ValueEntry {
            value: 5,
            name: "ChamberRev",
            parameters: &[],
        },
        ValueEntry {
            value: 6,
            name: "PlateRev",
            parameters: &[],
        },
        ValueEntry {
            value: 7,
            name: "RichPltRev",
            parameters: &[],
        },
        ValueEntry {
            value: 8,
            name: "GatedRev",
            parameters: &[],
        },
        ValueEntry {
            value: 9,
            name: "Reverse",
            parameters: &[],
        },
        ValueEntry {
            value: 10,
            name: "ChorusVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 11,
            name: "DelayVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 12,
            name: "FlangVerb",
            parameters: &[],
        },
        ValueEntry {
            value: 13,
            name: "MidasEQ",
            parameters: &[],
        },
        ValueEntry {
            value: 14,
            name: "Enhancer",
            parameters: &[],
        },
        ValueEntry {
            value: 15,
            name: "FairComp",
            parameters: &[],
        },
        ValueEntry {
            value: 16,
            name: "MulBndDist",
            parameters: &[],
        },
        ValueEntry {
            value: 17,
            name: "RackAmp",
            parameters: &[],
        },
        ValueEntry {
            value: 18,
            name: "EdisonEX1",
            parameters: &[],
        },
        ValueEntry {
            value: 19,
            name: "Auto Pan",
            parameters: &[],
        },
        ValueEntry {
            value: 20,
            name: "NoiseGate",
            parameters: &[],
        },
        ValueEntry {
            value: 21,
            name: "Delay",
            parameters: &[],
        },
        ValueEntry {
            value: 22,
            name: "3TapDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 23,
            name: "4TapDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 24,
            name: "T-RayDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 25,
            name: "DecimDelay",
            parameters: &[],
        },
        ValueEntry {
            value: 26,
            name: "ModDlyRev",
            parameters: &[],
        },
        ValueEntry {
            value: 27,
            name: "Chorus",
            parameters: &[],
        },
        ValueEntry {
            value: 28,
            name: "Chorus-D",
            parameters: &[],
        },
        ValueEntry {
            value: 29,
            name: "Flanger",
            parameters: &[],
        },
        ValueEntry {
            value: 30,
            name: "Phaser",
            parameters: &[],
        },
        ValueEntry {
            value: 31,
            name: "MoodFilter",
            parameters: &[],
        },
        ValueEntry {
            value: 32,
            name: "DualPitch",
            parameters: &[],
        },
        ValueEntry {
            value: 33,
            name: "RotarySpkr",
            parameters: &[],
        },
    ],
};

/// Every controller the synthesizer answers, in controller number order.
///
/// 90 of them drive a program parameter; the rest are the controllers the MIDI
/// specification defines, used as it defines them. [`Controller::for_cc`] and
/// [`Controller::for_parameter`] are the two ways in.
pub const CONTROLLERS: [Controller; CONTROLLER_COUNT] = [
    Controller {
        cc: 1,
        name: "Modulation Wheel",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 2,
        name: "Breath Controller",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 4,
        name: "Foot Controller",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 5,
        name: "Portamento time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::PortamentoTime),
    },
    Controller {
        cc: 6,
        name: "Data Entry MSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 7,
        name: "Channel Volume",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 8,
        name: "Balance",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 10,
        name: "Pan",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 11,
        name: "Expression",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 12,
        name: "Arp Rate (tempo)",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ArpRateTempo),
    },
    Controller {
        cc: 13,
        name: "Arp Gate Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ArpGateTime),
    },
    Controller {
        cc: 16,
        name: "LFO 1 Rate",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Lfo1Rate),
    },
    Controller {
        cc: 17,
        name: "LFO 1 Delay / Fade",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Lfo1DelayFade),
    },
    Controller {
        cc: 18,
        name: "LFO 2 Rate",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Lfo2Rate),
    },
    Controller {
        cc: 19,
        name: "LFO 2 Delay / Fade",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Lfo2DelayFade),
    },
    Controller {
        cc: 20,
        name: "OSC 1 Pitch Mod Depth",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Osc1PitchModDepth),
    },
    Controller {
        cc: 21,
        name: "OSC 1 PWM Depth",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Osc1PwmDepth),
    },
    Controller {
        cc: 23,
        name: "OSC 2 Pitch Mod Depth",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Osc2PitchModDepth),
    },
    Controller {
        cc: 24,
        name: "OSC 2 Tone Mod Depth",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Osc2ToneModDepth),
    },
    Controller {
        cc: 25,
        name: "OSC 2 Pitch",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Osc2Pitch),
    },
    Controller {
        cc: 26,
        name: "OSC 2 Level",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Osc2Level),
    },
    Controller {
        cc: 27,
        name: "Noise Level",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::NoiseLevel),
    },
    Controller {
        cc: 28,
        name: "Unison Detune",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::UnisonDetune),
    },
    Controller {
        cc: 29,
        name: "VCF Frequency",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfFrequency),
    },
    Controller {
        cc: 30,
        name: "VCF Resonance",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfResonance),
    },
    Controller {
        cc: 31,
        name: "VCF Mod",
        kind: ControllerKind::Other,
        parameter: None,
    },
    Controller {
        cc: 32,
        name: "Bank Select LSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 33,
        name: "VCF LFO Depth",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfLfoDepth),
    },
    Controller {
        cc: 34,
        name: "VCF Keyboard Tracking",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfKeyboardTracking),
    },
    Controller {
        cc: 35,
        name: "VCF HighPass Frequency",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfHighPassFrequency),
    },
    Controller {
        cc: 36,
        name: "VCA Level",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaLevel),
    },
    Controller {
        cc: 37,
        name: "VCA Envelope Attack Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeAttackTime),
    },
    Controller {
        cc: 38,
        name: "Data Entry LSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 39,
        name: "VCA Envelope Decay Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeDecayTime),
    },
    Controller {
        cc: 40,
        name: "VCA Envelope Sustain Level",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeSustainLevel),
    },
    Controller {
        cc: 41,
        name: "VCA Envelope Release Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeReleaseTime),
    },
    Controller {
        cc: 42,
        name: "VCF Envelope Attack Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeAttackTime),
    },
    Controller {
        cc: 43,
        name: "VCF Envelope Decay Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeDecayTime),
    },
    Controller {
        cc: 44,
        name: "VCF Envelope Sustain Level",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeSustainLevel),
    },
    Controller {
        cc: 45,
        name: "VCF Envelope Release Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeReleaseTime),
    },
    Controller {
        cc: 46,
        name: "Mod Envelope Attack Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeAttackTime),
    },
    Controller {
        cc: 47,
        name: "Mod Envelope Decay Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeDecayTime),
    },
    Controller {
        cc: 48,
        name: "Mod Envelope Sustain Level",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeSustainLevel),
    },
    Controller {
        cc: 49,
        name: "Mod Envelope Release Time",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeReleaseTime),
    },
    Controller {
        cc: 50,
        name: "VCA Envelope Attack Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeAttackCurve),
    },
    Controller {
        cc: 51,
        name: "VCA Envelope Decay Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeDecayCurve),
    },
    Controller {
        cc: 52,
        name: "VCA Envelope Sustain Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeSustainCurve),
    },
    Controller {
        cc: 53,
        name: "VCA Envelope Release Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcaEnvelopeReleaseCurve),
    },
    Controller {
        cc: 54,
        name: "VCF Envelope Attack Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeAttackCurve),
    },
    Controller {
        cc: 55,
        name: "VCF Envelope Decay Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeDecayCurve),
    },
    Controller {
        cc: 56,
        name: "VCF Envelope Sustain Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeSustainCurve),
    },
    Controller {
        cc: 57,
        name: "VCF Envelope Release Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::VcfEnvelopeReleaseCurve),
    },
    Controller {
        cc: 58,
        name: "Mod Envelope Attack Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeAttackCurve),
    },
    Controller {
        cc: 59,
        name: "Mod Envelope Decay Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeDecayCurve),
    },
    Controller {
        cc: 60,
        name: "Mod Envelope Sustain Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeSustainCurve),
    },
    Controller {
        cc: 61,
        name: "Mod Envelope Release Curve",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::ModEnvelopeReleaseCurve),
    },
    Controller {
        cc: 62,
        name: "FX 1 Param 1",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param1),
    },
    Controller {
        cc: 63,
        name: "FX 1 Param 2",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param2),
    },
    Controller {
        cc: 64,
        name: "Sustain Pedal",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 65,
        name: "FX 1 Param 3",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param3),
    },
    Controller {
        cc: 66,
        name: "FX 1 Param 4",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param4),
    },
    Controller {
        cc: 67,
        name: "FX 1 Param 5",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param5),
    },
    Controller {
        cc: 68,
        name: "FX 1 Param 6",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param6),
    },
    Controller {
        cc: 69,
        name: "FX 1 Param 7",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param7),
    },
    Controller {
        cc: 70,
        name: "FX 1 Param 8",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param8),
    },
    Controller {
        cc: 71,
        name: "FX 1 Param 9",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param9),
    },
    Controller {
        cc: 72,
        name: "FX 1 Param 10",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param10),
    },
    Controller {
        cc: 73,
        name: "FX 1 Param 11",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param11),
    },
    Controller {
        cc: 74,
        name: "FX 1 Param 12",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Param12),
    },
    Controller {
        cc: 75,
        name: "FX 2 Param 1",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param1),
    },
    Controller {
        cc: 76,
        name: "FX 2 Param 2",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param2),
    },
    Controller {
        cc: 77,
        name: "FX 2 Param 3",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param3),
    },
    Controller {
        cc: 78,
        name: "FX 2 Param 4",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param4),
    },
    Controller {
        cc: 79,
        name: "FX 2 Param 5",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param5),
    },
    Controller {
        cc: 80,
        name: "FX 2 Param 6",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param6),
    },
    Controller {
        cc: 81,
        name: "FX 2 Param 7",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param7),
    },
    Controller {
        cc: 82,
        name: "FX 2 Param 8",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param8),
    },
    Controller {
        cc: 83,
        name: "FX 2 Param 9",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param9),
    },
    Controller {
        cc: 84,
        name: "FX 2 Param 10",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param10),
    },
    Controller {
        cc: 85,
        name: "FX 2 Param 11",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param11),
    },
    Controller {
        cc: 86,
        name: "FX 2 Param 12",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Param12),
    },
    Controller {
        cc: 87,
        name: "FX 3 Param 1",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param1),
    },
    Controller {
        cc: 88,
        name: "FX 3 Param 2",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param2),
    },
    Controller {
        cc: 89,
        name: "FX 3 Param 3",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param3),
    },
    Controller {
        cc: 90,
        name: "FX 3 Param 4",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param4),
    },
    Controller {
        cc: 91,
        name: "FX 3 Param 5",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param5),
    },
    Controller {
        cc: 92,
        name: "FX 3 Param 6",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param6),
    },
    Controller {
        cc: 93,
        name: "FX 3 Param 7",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param7),
    },
    Controller {
        cc: 94,
        name: "FX 3 Param 8",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param8),
    },
    Controller {
        cc: 95,
        name: "FX 3 Param 9",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param9),
    },
    Controller {
        cc: 96,
        name: "Data Increment",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 97,
        name: "Data Decrement",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 98,
        name: "NRPN LSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 99,
        name: "NRPN MSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 100,
        name: "RPN LSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 101,
        name: "RPN MSB",
        kind: ControllerKind::Standard,
        parameter: None,
    },
    Controller {
        cc: 102,
        name: "FX 3 Param 10",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param10),
    },
    Controller {
        cc: 103,
        name: "FX 3 Param 11",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param11),
    },
    Controller {
        cc: 104,
        name: "FX 3 Param 12",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Param12),
    },
    Controller {
        cc: 105,
        name: "FX 1 Type",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1Type),
    },
    Controller {
        cc: 106,
        name: "FX 2 Type",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2Type),
    },
    Controller {
        cc: 107,
        name: "FX 3 Type",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3Type),
    },
    Controller {
        cc: 108,
        name: "FX 4 Type",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx4Type),
    },
    Controller {
        cc: 109,
        name: "FX 1 Output Gain",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx1OutputGain),
    },
    Controller {
        cc: 110,
        name: "FX 2 Output Gain",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx2OutputGain),
    },
    Controller {
        cc: 111,
        name: "FX 3 Output Gain",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx3OutputGain),
    },
    Controller {
        cc: 112,
        name: "FX 4 Output Gain",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::Fx4OutputGain),
    },
    Controller {
        cc: 113,
        name: "Analog Thru",
        kind: ControllerKind::Other,
        parameter: None,
    },
    Controller {
        cc: 114,
        name: "FX Mode",
        kind: ControllerKind::Parameter,
        parameter: Some(ParamId::FxMode),
    },
    Controller {
        cc: 115,
        name: "3D X axis",
        kind: ControllerKind::Other,
        parameter: None,
    },
    Controller {
        cc: 116,
        name: "3D Y axis",
        kind: ControllerKind::Other,
        parameter: None,
    },
    Controller {
        cc: 117,
        name: "3D Z axis",
        kind: ControllerKind::Other,
        parameter: None,
    },
];

/// Stands for a parameter no controller drives, in `CONTROLLER_OF_PARAMETER`.
///
/// The table holds indices into `CONTROLLERS`, which is shorter than a byte, so
/// the largest byte is free to mean "none" and the table stays one byte wide.
pub(super) const NO_CONTROLLER: u8 = u8::MAX;

/// The controller that drives each parameter, by the parameter's own offset.
pub(super) static CONTROLLER_OF_PARAMETER: [u8; PARAMETER_COUNT] = [
    // 90 of the 242 parameters; NRPN is the only way to the rest.
    11,
    12,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    13,
    14,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    15,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    16,
    20,
    19,
    18,
    17,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    21,
    3,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    23,
    29,
    24,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    27,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    28,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    31,
    33,
    34,
    35,
    NO_CONTROLLER,
    44,
    45,
    46,
    47,
    36,
    37,
    38,
    39,
    NO_CONTROLLER,
    48,
    49,
    50,
    51,
    40,
    41,
    42,
    43,
    NO_CONTROLLER,
    52,
    53,
    54,
    55,
    30,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    22,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    9,
    NO_CONTROLLER,
    NO_CONTROLLER,
    10,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    99,
    56,
    57,
    59,
    60,
    61,
    62,
    63,
    64,
    65,
    66,
    67,
    68,
    100,
    69,
    70,
    71,
    72,
    73,
    74,
    75,
    76,
    77,
    78,
    79,
    80,
    101,
    81,
    82,
    83,
    84,
    85,
    86,
    87,
    88,
    89,
    96,
    97,
    98,
    102,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    103,
    104,
    105,
    106,
    108,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
    NO_CONTROLLER,
];
