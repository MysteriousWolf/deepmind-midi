//! The parameter, value table and controller data, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The types these tables
//! fill, and everything that reads them, are in the parent module.

// The parameter table is one match with an arm per parameter. Splitting it into
// chunks to satisfy a length lint would only hide what it is.
#![expect(clippy::too_many_lines, reason = "a generated table, not logic")]

use super::{Controller, ControllerKind, Kind, Parameter, ValueEntry, ValueTable};
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
                kind: Kind::Switch,
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
                kind: Kind::Switch,
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
        },
        ValueEntry {
            value: 1,
            name: "Triangle",
        },
        ValueEntry {
            value: 2,
            name: "Square",
        },
        ValueEntry {
            value: 3,
            name: "Ramp Up",
        },
        ValueEntry {
            value: 4,
            name: "Ramp Down",
        },
        ValueEntry {
            value: 5,
            name: "Sample & Hold",
        },
        ValueEntry {
            value: 6,
            name: "Sample & Glide",
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
        },
        ValueEntry {
            value: 1,
            name: "8'",
        },
        ValueEntry {
            value: 2,
            name: "4'",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO 1",
        },
        ValueEntry {
            value: 2,
            name: "LFO 2",
        },
        ValueEntry {
            value: 3,
            name: "VCA Env",
        },
        ValueEntry {
            value: 4,
            name: "VCF Env",
        },
        ValueEntry {
            value: 5,
            name: "Mod Env",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO 1",
        },
        ValueEntry {
            value: 2,
            name: "LFO 2",
        },
        ValueEntry {
            value: 3,
            name: "VCA Env",
        },
        ValueEntry {
            value: 4,
            name: "VCF Env",
        },
        ValueEntry {
            value: 5,
            name: "Mod Env",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO 2",
        },
        ValueEntry {
            value: 2,
            name: "VCA Env",
        },
        ValueEntry {
            value: 3,
            name: "VCF Env",
        },
        ValueEntry {
            value: 4,
            name: "Mod Env",
        },
        ValueEntry {
            value: 5,
            name: "LFO 1 Unipolar",
        },
        ValueEntry {
            value: 6,
            name: "LFO 2 Unipolar",
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
        },
        ValueEntry {
            value: 1,
            name: "Fingered",
        },
        ValueEntry {
            value: 2,
            name: "Fixed Rate",
        },
        ValueEntry {
            value: 3,
            name: "Fixed Rate Fingered",
        },
        ValueEntry {
            value: 4,
            name: "Exponential",
        },
        ValueEntry {
            value: 5,
            name: "Exponential Fingered",
        },
        ValueEntry {
            value: 6,
            name: "Fixed +2",
        },
        ValueEntry {
            value: 7,
            name: "Fixed -2",
        },
        ValueEntry {
            value: 8,
            name: "Fixed +5",
        },
        ValueEntry {
            value: 9,
            name: "Fixed -5",
        },
        ValueEntry {
            value: 10,
            name: "Fixed +12",
        },
        ValueEntry {
            value: 11,
            name: "Fixed -12",
        },
        ValueEntry {
            value: 12,
            name: "Fixed +24",
        },
        ValueEntry {
            value: 13,
            name: "Fixed -24",
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
        },
        ValueEntry {
            value: 1,
            name: "OSC 1 only",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO 2",
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
        },
        ValueEntry {
            value: 1,
            name: "Positive",
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
        },
        ValueEntry {
            value: 1,
            name: "2 Pole",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO 1",
        },
        ValueEntry {
            value: 2,
            name: "LFO 2",
        },
        ValueEntry {
            value: 3,
            name: "Loop",
        },
        ValueEntry {
            value: 4,
            name: "Control Sequencer Step",
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
        },
        ValueEntry {
            value: 1,
            name: "Highest",
        },
        ValueEntry {
            value: 2,
            name: "Last",
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
        },
        ValueEntry {
            value: 1,
            name: "Unison 2",
        },
        ValueEntry {
            value: 2,
            name: "Unison 3",
        },
        ValueEntry {
            value: 3,
            name: "Unison 4",
        },
        ValueEntry {
            value: 4,
            name: "Unison 6",
        },
        ValueEntry {
            value: 5,
            name: "Unison 12",
        },
        ValueEntry {
            value: 6,
            name: "Mono",
        },
        ValueEntry {
            value: 7,
            name: "Mono 2",
        },
        ValueEntry {
            value: 8,
            name: "Mono 3",
        },
        ValueEntry {
            value: 9,
            name: "Mono 4",
        },
        ValueEntry {
            value: 10,
            name: "Mono 6",
        },
        ValueEntry {
            value: 11,
            name: "Poly 6",
        },
        ValueEntry {
            value: 12,
            name: "Poly 8",
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
        },
        ValueEntry {
            value: 1,
            name: "Re-Trigger",
        },
        ValueEntry {
            value: 2,
            name: "Legato",
        },
        ValueEntry {
            value: 3,
            name: "One-Shot",
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
        },
        ValueEntry {
            value: 1,
            name: "Key sync on",
        },
        ValueEntry {
            value: 2,
            name: "Loop and key sync on",
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
        },
        ValueEntry {
            value: 1,
            name: "Down",
        },
        ValueEntry {
            value: 2,
            name: "Up & Down",
        },
        ValueEntry {
            value: 3,
            name: "Up Inv",
        },
        ValueEntry {
            value: 4,
            name: "Down Inv",
        },
        ValueEntry {
            value: 5,
            name: "Up & Down Inv",
        },
        ValueEntry {
            value: 6,
            name: "Up Alt",
        },
        ValueEntry {
            value: 7,
            name: "Down Alt",
        },
        ValueEntry {
            value: 8,
            name: "Random",
        },
        ValueEntry {
            value: 9,
            name: "As Played",
        },
        ValueEntry {
            value: 10,
            name: "Chord",
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
        },
        ValueEntry {
            value: 1,
            name: "Parallel 1/2, serial 3-4",
        },
        ValueEntry {
            value: 2,
            name: "Parallel 1/2, parallel 3/4",
        },
        ValueEntry {
            value: 3,
            name: "Parallel 1/2/3/4",
        },
        ValueEntry {
            value: 4,
            name: "Parallel 1/2/3, serial 4",
        },
        ValueEntry {
            value: 5,
            name: "Serial 1-2, parallel 3/4",
        },
        ValueEntry {
            value: 6,
            name: "Serial 1, parallel 2/3/4",
        },
        ValueEntry {
            value: 7,
            name: "Parallel (serial 1-2-3)/4",
        },
        ValueEntry {
            value: 8,
            name: "Serial 3-4 feedback 4(1-2)",
        },
        ValueEntry {
            value: 9,
            name: "Serial 4 feedback 4(1-2-3)",
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
        },
        ValueEntry {
            value: 1,
            name: "Send",
        },
        ValueEntry {
            value: 2,
            name: "Bypass",
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
        },
        ValueEntry {
            value: 1,
            name: "Bass",
        },
        ValueEntry {
            value: 2,
            name: "Pad",
        },
        ValueEntry {
            value: 3,
            name: "Lead",
        },
        ValueEntry {
            value: 4,
            name: "Mono",
        },
        ValueEntry {
            value: 5,
            name: "Poly",
        },
        ValueEntry {
            value: 6,
            name: "Stab",
        },
        ValueEntry {
            value: 7,
            name: "SFX",
        },
        ValueEntry {
            value: 8,
            name: "Arp",
        },
        ValueEntry {
            value: 9,
            name: "Seq",
        },
        ValueEntry {
            value: 10,
            name: "Perc",
        },
        ValueEntry {
            value: 11,
            name: "Ambient",
        },
        ValueEntry {
            value: 12,
            name: "Modular",
        },
        ValueEntry {
            value: 13,
            name: "User-1",
        },
        ValueEntry {
            value: 14,
            name: "User-2",
        },
        ValueEntry {
            value: 15,
            name: "User-3",
        },
        ValueEntry {
            value: 16,
            name: "User-4",
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
        },
        ValueEntry {
            value: 1,
            name: "Mono",
        },
        ValueEntry {
            value: 2,
            name: "SPREAD-1",
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
        },
        ValueEntry {
            value: 1,
            name: "Preset-1",
        },
        ValueEntry {
            value: 33,
            name: "User-1",
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
        },
        ValueEntry {
            value: 1,
            name: "3/8",
        },
        ValueEntry {
            value: 2,
            name: "1/3",
        },
        ValueEntry {
            value: 3,
            name: "1/4",
        },
        ValueEntry {
            value: 4,
            name: "3/16",
        },
        ValueEntry {
            value: 5,
            name: "1/6",
        },
        ValueEntry {
            value: 6,
            name: "1/8",
        },
        ValueEntry {
            value: 7,
            name: "3/32",
        },
        ValueEntry {
            value: 8,
            name: "1/12",
        },
        ValueEntry {
            value: 9,
            name: "1/16",
        },
        ValueEntry {
            value: 10,
            name: "1/24",
        },
        ValueEntry {
            value: 11,
            name: "1/32",
        },
        ValueEntry {
            value: 12,
            name: "1/48",
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
        },
        ValueEntry {
            value: 1,
            name: "3",
        },
        ValueEntry {
            value: 2,
            name: "2",
        },
        ValueEntry {
            value: 3,
            name: "1",
        },
        ValueEntry {
            value: 4,
            name: "1/2",
        },
        ValueEntry {
            value: 5,
            name: "3/8",
        },
        ValueEntry {
            value: 6,
            name: "1/3",
        },
        ValueEntry {
            value: 7,
            name: "1/4",
        },
        ValueEntry {
            value: 8,
            name: "3/16",
        },
        ValueEntry {
            value: 9,
            name: "1/6",
        },
        ValueEntry {
            value: 10,
            name: "1/8",
        },
        ValueEntry {
            value: 11,
            name: "3/32",
        },
        ValueEntry {
            value: 12,
            name: "1/12",
        },
        ValueEntry {
            value: 13,
            name: "1/16",
        },
        ValueEntry {
            value: 14,
            name: "3/64",
        },
        ValueEntry {
            value: 15,
            name: "1/24",
        },
        ValueEntry {
            value: 16,
            name: "1/32",
        },
        ValueEntry {
            value: 17,
            name: "3/128",
        },
        ValueEntry {
            value: 18,
            name: "1/48",
        },
        ValueEntry {
            value: 19,
            name: "1/64",
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
        },
        ValueEntry {
            value: 1,
            name: "3",
        },
        ValueEntry {
            value: 2,
            name: "2",
        },
        ValueEntry {
            value: 3,
            name: "1",
        },
        ValueEntry {
            value: 4,
            name: "1/2",
        },
        ValueEntry {
            value: 5,
            name: "3/8",
        },
        ValueEntry {
            value: 6,
            name: "1/3",
        },
        ValueEntry {
            value: 7,
            name: "1/4",
        },
        ValueEntry {
            value: 8,
            name: "3/16",
        },
        ValueEntry {
            value: 9,
            name: "1/6",
        },
        ValueEntry {
            value: 10,
            name: "1/8",
        },
        ValueEntry {
            value: 11,
            name: "3/32",
        },
        ValueEntry {
            value: 12,
            name: "1/12",
        },
        ValueEntry {
            value: 13,
            name: "1/16",
        },
        ValueEntry {
            value: 14,
            name: "3/64",
        },
        ValueEntry {
            value: 15,
            name: "1/24",
        },
        ValueEntry {
            value: 16,
            name: "1/32",
        },
        ValueEntry {
            value: 17,
            name: "3/128",
        },
        ValueEntry {
            value: 18,
            name: "1/48",
        },
        ValueEntry {
            value: 19,
            name: "1/64",
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
        },
        ValueEntry {
            value: 1,
            name: "Pitch Bend",
        },
        ValueEntry {
            value: 2,
            name: "Mod Wheel",
        },
        ValueEntry {
            value: 3,
            name: "Foot Ctrl",
        },
        ValueEntry {
            value: 4,
            name: "BreathCtrl",
        },
        ValueEntry {
            value: 5,
            name: "Pressure",
        },
        ValueEntry {
            value: 6,
            name: "Expression",
        },
        ValueEntry {
            value: 7,
            name: "LFO1",
        },
        ValueEntry {
            value: 8,
            name: "LFO2",
        },
        ValueEntry {
            value: 9,
            name: "Env 1",
        },
        ValueEntry {
            value: 10,
            name: "Env 2",
        },
        ValueEntry {
            value: 11,
            name: "Env 3",
        },
        ValueEntry {
            value: 12,
            name: "Note Num",
        },
        ValueEntry {
            value: 13,
            name: "Note Vel",
        },
        ValueEntry {
            value: 14,
            name: "Note Off Vel",
        },
        ValueEntry {
            value: 15,
            name: "Ctrl Seq",
        },
        ValueEntry {
            value: 16,
            name: "LFO1 (Uni)",
        },
        ValueEntry {
            value: 17,
            name: "LFO2 (Uni)",
        },
        ValueEntry {
            value: 18,
            name: "LFO1 (Fade)",
        },
        ValueEntry {
            value: 19,
            name: "LFO2 (Fade)",
        },
        ValueEntry {
            value: 20,
            name: "Voice Num",
        },
        ValueEntry {
            value: 21,
            name: "Uni Voice",
        },
        ValueEntry {
            value: 22,
            name: "CC X (115)",
        },
        ValueEntry {
            value: 23,
            name: "CC Y (116)",
        },
        ValueEntry {
            value: 24,
            name: "CC Z (117)",
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
        },
        ValueEntry {
            value: 1,
            name: "Pitch Bend",
        },
        ValueEntry {
            value: 2,
            name: "Mod Wheel",
        },
        ValueEntry {
            value: 3,
            name: "Foot Ctrl",
        },
        ValueEntry {
            value: 4,
            name: "BreathCtrl",
        },
        ValueEntry {
            value: 5,
            name: "Pressure",
        },
        ValueEntry {
            value: 6,
            name: "LFO1",
        },
        ValueEntry {
            value: 7,
            name: "LFO2",
        },
        ValueEntry {
            value: 8,
            name: "Env 1",
        },
        ValueEntry {
            value: 9,
            name: "Env 2",
        },
        ValueEntry {
            value: 10,
            name: "Env 3",
        },
        ValueEntry {
            value: 11,
            name: "Note Num",
        },
        ValueEntry {
            value: 12,
            name: "Note Vel",
        },
        ValueEntry {
            value: 13,
            name: "Ctrl Seq",
        },
        ValueEntry {
            value: 14,
            name: "LFO1 (Uni)",
        },
        ValueEntry {
            value: 15,
            name: "LFO2 (Uni)",
        },
        ValueEntry {
            value: 16,
            name: "LFO1 (Fade)",
        },
        ValueEntry {
            value: 17,
            name: "LFO2 (Fade)",
        },
        ValueEntry {
            value: 18,
            name: "NoteOff Vel",
        },
        ValueEntry {
            value: 19,
            name: "Voice Num",
        },
        ValueEntry {
            value: 20,
            name: "CC X (114)",
        },
        ValueEntry {
            value: 21,
            name: "CC Y (115)",
        },
        ValueEntry {
            value: 22,
            name: "CC Z (116)",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO1 Rate",
        },
        ValueEntry {
            value: 2,
            name: "LFO1 Delay",
        },
        ValueEntry {
            value: 3,
            name: "LFO1 Slew",
        },
        ValueEntry {
            value: 4,
            name: "LFO1 Shape",
        },
        ValueEntry {
            value: 5,
            name: "LFO2 Rate",
        },
        ValueEntry {
            value: 6,
            name: "LFO2 Delay",
        },
        ValueEntry {
            value: 7,
            name: "LFO2 Slew",
        },
        ValueEntry {
            value: 8,
            name: "LFO2 Shape",
        },
        ValueEntry {
            value: 9,
            name: "OSC1+2 Pit",
        },
        ValueEntry {
            value: 10,
            name: "OSC1+2 Fine",
        },
        ValueEntry {
            value: 11,
            name: "OSC1 Pitch",
        },
        ValueEntry {
            value: 12,
            name: "OSC1 Fine",
        },
        ValueEntry {
            value: 13,
            name: "OSC2 Pitch",
        },
        ValueEntry {
            value: 14,
            name: "OSC2 Fine",
        },
        ValueEntry {
            value: 15,
            name: "OSC1 PM Dep",
        },
        ValueEntry {
            value: 16,
            name: "PWM Depth",
        },
        ValueEntry {
            value: 17,
            name: "TMod Depth",
        },
        ValueEntry {
            value: 18,
            name: "OSC2 PM Dep",
        },
        ValueEntry {
            value: 19,
            name: "Porta Time",
        },
        ValueEntry {
            value: 20,
            name: "VCF Freq",
        },
        ValueEntry {
            value: 21,
            name: "VCF Res",
        },
        ValueEntry {
            value: 22,
            name: "VCF Env",
        },
        ValueEntry {
            value: 23,
            name: "VCF LFO",
        },
        ValueEntry {
            value: 24,
            name: "Env Rates",
        },
        ValueEntry {
            value: 25,
            name: "All Attack",
        },
        ValueEntry {
            value: 26,
            name: "All Decay",
        },
        ValueEntry {
            value: 27,
            name: "All Sus",
        },
        ValueEntry {
            value: 28,
            name: "All Rel",
        },
        ValueEntry {
            value: 29,
            name: "Env1 Rates",
        },
        ValueEntry {
            value: 30,
            name: "Env2 Rates",
        },
        ValueEntry {
            value: 31,
            name: "Env3 Rates",
        },
        ValueEntry {
            value: 32,
            name: "Env1CurveS",
        },
        ValueEntry {
            value: 33,
            name: "Env2CurveS",
        },
        ValueEntry {
            value: 34,
            name: "Env3CurveS",
        },
        ValueEntry {
            value: 35,
            name: "Env1 Attack",
        },
        ValueEntry {
            value: 36,
            name: "Env1 Decay",
        },
        ValueEntry {
            value: 37,
            name: "Env1 Sus",
        },
        ValueEntry {
            value: 38,
            name: "Env1 Rel",
        },
        ValueEntry {
            value: 39,
            name: "Env1 AtCur",
        },
        ValueEntry {
            value: 40,
            name: "Env1 DcyCur",
        },
        ValueEntry {
            value: 41,
            name: "Env1 SuSCur",
        },
        ValueEntry {
            value: 42,
            name: "Env1 RelCur",
        },
        ValueEntry {
            value: 43,
            name: "Env2 Attack",
        },
        ValueEntry {
            value: 44,
            name: "Env2 Decay",
        },
        ValueEntry {
            value: 45,
            name: "Env2 Sus",
        },
        ValueEntry {
            value: 46,
            name: "Env2 Rel",
        },
        ValueEntry {
            value: 47,
            name: "Env2 AtCur",
        },
        ValueEntry {
            value: 48,
            name: "Env2 DcyCur",
        },
        ValueEntry {
            value: 49,
            name: "Env2 SuSCur",
        },
        ValueEntry {
            value: 50,
            name: "Env2 RelCur",
        },
        ValueEntry {
            value: 51,
            name: "Env3 Attack",
        },
        ValueEntry {
            value: 52,
            name: "Env3 Decay",
        },
        ValueEntry {
            value: 53,
            name: "Env3 Sus",
        },
        ValueEntry {
            value: 54,
            name: "Env3 Rel",
        },
        ValueEntry {
            value: 55,
            name: "Env3 AtCur",
        },
        ValueEntry {
            value: 56,
            name: "Env3 DcyCur",
        },
        ValueEntry {
            value: 57,
            name: "Env3 SuSCur",
        },
        ValueEntry {
            value: 58,
            name: "Env3 RelCur",
        },
        ValueEntry {
            value: 59,
            name: "VCA All",
        },
        ValueEntry {
            value: 60,
            name: "VCA Active",
        },
        ValueEntry {
            value: 61,
            name: "VCA EnvDep",
        },
        ValueEntry {
            value: 62,
            name: "Pan Spread",
        },
        ValueEntry {
            value: 63,
            name: "VCA Pan",
        },
        ValueEntry {
            value: 64,
            name: "OSC2 Lvl",
        },
        ValueEntry {
            value: 65,
            name: "Noise Lvl",
        },
        ValueEntry {
            value: 66,
            name: "HP Freq",
        },
        ValueEntry {
            value: 67,
            name: "Uni Detune",
        },
        ValueEntry {
            value: 68,
            name: "OSC Drift",
        },
        ValueEntry {
            value: 69,
            name: "Param Drift",
        },
        ValueEntry {
            value: 70,
            name: "Drift Rate",
        },
        ValueEntry {
            value: 71,
            name: "Arp Gate",
        },
        ValueEntry {
            value: 72,
            name: "Seq Slew",
        },
        ValueEntry {
            value: 73,
            name: "Mod 1 Dep",
        },
        ValueEntry {
            value: 74,
            name: "Mod 2 Dep",
        },
        ValueEntry {
            value: 75,
            name: "Mod 3 Dep",
        },
        ValueEntry {
            value: 76,
            name: "Mod 4 Dep",
        },
        ValueEntry {
            value: 77,
            name: "Mod 5 Dep",
        },
        ValueEntry {
            value: 78,
            name: "Mod 6 Dep",
        },
        ValueEntry {
            value: 79,
            name: "Mod 7 Dep",
        },
        ValueEntry {
            value: 80,
            name: "Mod 8 Dep",
        },
        ValueEntry {
            value: 81,
            name: "Fx 1 Param 1",
        },
        ValueEntry {
            value: 82,
            name: "Fx 1 Param 2",
        },
        ValueEntry {
            value: 83,
            name: "Fx 1 Param 3",
        },
        ValueEntry {
            value: 84,
            name: "Fx 1 Param 4",
        },
        ValueEntry {
            value: 85,
            name: "Fx 1 Param 5",
        },
        ValueEntry {
            value: 86,
            name: "Fx 1 Param 6",
        },
        ValueEntry {
            value: 87,
            name: "Fx 1 Param 7",
        },
        ValueEntry {
            value: 88,
            name: "Fx 1 Param 8",
        },
        ValueEntry {
            value: 89,
            name: "Fx 1 Param 9",
        },
        ValueEntry {
            value: 90,
            name: "Fx 1 Param 10",
        },
        ValueEntry {
            value: 91,
            name: "Fx 1 Param 11",
        },
        ValueEntry {
            value: 92,
            name: "Fx 1 Param 12",
        },
        ValueEntry {
            value: 93,
            name: "Fx 2 Param 1",
        },
        ValueEntry {
            value: 94,
            name: "Fx 2 Param 2",
        },
        ValueEntry {
            value: 95,
            name: "Fx 2 Param 3",
        },
        ValueEntry {
            value: 96,
            name: "Fx 2 Param 4",
        },
        ValueEntry {
            value: 97,
            name: "Fx 2 Param 5",
        },
        ValueEntry {
            value: 98,
            name: "Fx 2 Param 6",
        },
        ValueEntry {
            value: 99,
            name: "Fx 2 Param 7",
        },
        ValueEntry {
            value: 100,
            name: "Fx 2 Param 8",
        },
        ValueEntry {
            value: 101,
            name: "Fx 2 Param 9",
        },
        ValueEntry {
            value: 102,
            name: "Fx 2 Param 10",
        },
        ValueEntry {
            value: 103,
            name: "Fx 2 Param 11",
        },
        ValueEntry {
            value: 104,
            name: "Fx 2 Param 12",
        },
        ValueEntry {
            value: 105,
            name: "Fx 3 Param 1",
        },
        ValueEntry {
            value: 106,
            name: "Fx 3 Param 2",
        },
        ValueEntry {
            value: 107,
            name: "Fx 3 Param 3",
        },
        ValueEntry {
            value: 108,
            name: "Fx 3 Param 4",
        },
        ValueEntry {
            value: 109,
            name: "Fx 3 Param 5",
        },
        ValueEntry {
            value: 110,
            name: "Fx 3 Param 6",
        },
        ValueEntry {
            value: 111,
            name: "Fx 3 Param 7",
        },
        ValueEntry {
            value: 112,
            name: "Fx 3 Param 8",
        },
        ValueEntry {
            value: 113,
            name: "Fx 3 Param 9",
        },
        ValueEntry {
            value: 114,
            name: "Fx 3 Param 10",
        },
        ValueEntry {
            value: 115,
            name: "Fx 3 Param 11",
        },
        ValueEntry {
            value: 116,
            name: "Fx 3 Param 12",
        },
        ValueEntry {
            value: 117,
            name: "Fx 4 Param 1",
        },
        ValueEntry {
            value: 118,
            name: "Fx 4 Param 2",
        },
        ValueEntry {
            value: 119,
            name: "Fx 4 Param 3",
        },
        ValueEntry {
            value: 120,
            name: "Fx 4 Param 4",
        },
        ValueEntry {
            value: 121,
            name: "Fx 4 Param 5",
        },
        ValueEntry {
            value: 122,
            name: "Fx 4 Param 6",
        },
        ValueEntry {
            value: 123,
            name: "Fx 4 Param 7",
        },
        ValueEntry {
            value: 124,
            name: "Fx 4 Param 8",
        },
        ValueEntry {
            value: 125,
            name: "Fx 4 Param 9",
        },
        ValueEntry {
            value: 126,
            name: "Fx 4 Param 10",
        },
        ValueEntry {
            value: 127,
            name: "Fx 4 Param 11",
        },
        ValueEntry {
            value: 128,
            name: "Fx 4 Param 12",
        },
        ValueEntry {
            value: 129,
            name: "Fx 1 Level",
        },
        ValueEntry {
            value: 130,
            name: "Fx 2 Level",
        },
        ValueEntry {
            value: 131,
            name: "Fx 3 Level",
        },
        ValueEntry {
            value: 132,
            name: "Fx 4 Level",
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
        },
        ValueEntry {
            value: 1,
            name: "AmbVerb",
        },
        ValueEntry {
            value: 2,
            name: "RoomRev",
        },
        ValueEntry {
            value: 3,
            name: "VintageRev",
        },
        ValueEntry {
            value: 4,
            name: "HallRev",
        },
        ValueEntry {
            value: 5,
            name: "ChamberRev",
        },
        ValueEntry {
            value: 6,
            name: "PlateRev",
        },
        ValueEntry {
            value: 7,
            name: "RichPltRev",
        },
        ValueEntry {
            value: 8,
            name: "GatedRev",
        },
        ValueEntry {
            value: 9,
            name: "Reverse",
        },
        ValueEntry {
            value: 10,
            name: "ChorusVerb",
        },
        ValueEntry {
            value: 11,
            name: "DelayVerb",
        },
        ValueEntry {
            value: 12,
            name: "FlangVerb",
        },
        ValueEntry {
            value: 13,
            name: "MidasEQ",
        },
        ValueEntry {
            value: 14,
            name: "Enhancer",
        },
        ValueEntry {
            value: 15,
            name: "FairComp",
        },
        ValueEntry {
            value: 16,
            name: "MulBndDist",
        },
        ValueEntry {
            value: 17,
            name: "RackAmp",
        },
        ValueEntry {
            value: 18,
            name: "EdisonEX1",
        },
        ValueEntry {
            value: 19,
            name: "Auto Pan",
        },
        ValueEntry {
            value: 20,
            name: "NoiseGate",
        },
        ValueEntry {
            value: 21,
            name: "Delay",
        },
        ValueEntry {
            value: 22,
            name: "3TapDelay",
        },
        ValueEntry {
            value: 23,
            name: "4TapDelay",
        },
        ValueEntry {
            value: 24,
            name: "T-RayDelay",
        },
        ValueEntry {
            value: 25,
            name: "DecimDelay",
        },
        ValueEntry {
            value: 26,
            name: "ModDlyRev",
        },
        ValueEntry {
            value: 27,
            name: "Chorus",
        },
        ValueEntry {
            value: 28,
            name: "Chorus-D",
        },
        ValueEntry {
            value: 29,
            name: "Flanger",
        },
        ValueEntry {
            value: 30,
            name: "Phaser",
        },
        ValueEntry {
            value: 31,
            name: "MoodFilter",
        },
        ValueEntry {
            value: 32,
            name: "DualPitch",
        },
        ValueEntry {
            value: 33,
            name: "Vintage Pitch",
        },
        ValueEntry {
            value: 34,
            name: "RotarySpkr",
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
        },
        ValueEntry {
            value: 1,
            name: "LFO1 Rate",
        },
        ValueEntry {
            value: 2,
            name: "LFO1 Delay",
        },
        ValueEntry {
            value: 3,
            name: "LFO1 Slew",
        },
        ValueEntry {
            value: 4,
            name: "LFO1 Shape",
        },
        ValueEntry {
            value: 5,
            name: "LFO2 Rate",
        },
        ValueEntry {
            value: 6,
            name: "LFO2 Delay",
        },
        ValueEntry {
            value: 7,
            name: "LFO2 Slew",
        },
        ValueEntry {
            value: 8,
            name: "LFO2 Shape",
        },
        ValueEntry {
            value: 9,
            name: "OSC1+2 Pit",
        },
        ValueEntry {
            value: 10,
            name: "OSC1 Pitch",
        },
        ValueEntry {
            value: 11,
            name: "OSC2 Pitch",
        },
        ValueEntry {
            value: 12,
            name: "OSC1 PM Dep",
        },
        ValueEntry {
            value: 13,
            name: "PWM Depth",
        },
        ValueEntry {
            value: 14,
            name: "TMod Depth",
        },
        ValueEntry {
            value: 15,
            name: "OSC2 PM Dep",
        },
        ValueEntry {
            value: 16,
            name: "Porta Time",
        },
        ValueEntry {
            value: 17,
            name: "VCF Freq",
        },
        ValueEntry {
            value: 18,
            name: "VCF Res",
        },
        ValueEntry {
            value: 19,
            name: "VCF Env",
        },
        ValueEntry {
            value: 20,
            name: "VCF LFO",
        },
        ValueEntry {
            value: 21,
            name: "Env Rates",
        },
        ValueEntry {
            value: 22,
            name: "All Attack",
        },
        ValueEntry {
            value: 23,
            name: "All Decay",
        },
        ValueEntry {
            value: 24,
            name: "All Sus",
        },
        ValueEntry {
            value: 25,
            name: "All Rel",
        },
        ValueEntry {
            value: 26,
            name: "Env1 Rates",
        },
        ValueEntry {
            value: 27,
            name: "Env2 Rates",
        },
        ValueEntry {
            value: 28,
            name: "Env3 Rates",
        },
        ValueEntry {
            value: 29,
            name: "Env1CurveS",
        },
        ValueEntry {
            value: 30,
            name: "Env2CurveS",
        },
        ValueEntry {
            value: 31,
            name: "Env3CurveS",
        },
        ValueEntry {
            value: 32,
            name: "Env1 Attack",
        },
        ValueEntry {
            value: 33,
            name: "Env1 Decay",
        },
        ValueEntry {
            value: 34,
            name: "Env1 Sus",
        },
        ValueEntry {
            value: 35,
            name: "Env1 Rel",
        },
        ValueEntry {
            value: 36,
            name: "Env1 AtCur",
        },
        ValueEntry {
            value: 37,
            name: "Env1 DcyCur",
        },
        ValueEntry {
            value: 38,
            name: "Env1 SuSCur",
        },
        ValueEntry {
            value: 39,
            name: "Env1 RelCur",
        },
        ValueEntry {
            value: 40,
            name: "Env2 Attack",
        },
        ValueEntry {
            value: 41,
            name: "Env2 Decay",
        },
        ValueEntry {
            value: 42,
            name: "Env2 Sus",
        },
        ValueEntry {
            value: 43,
            name: "Env2 Rel",
        },
        ValueEntry {
            value: 44,
            name: "Env2 AtCur",
        },
        ValueEntry {
            value: 45,
            name: "Env2 DcyCur",
        },
        ValueEntry {
            value: 46,
            name: "Env2 SuSCur",
        },
        ValueEntry {
            value: 47,
            name: "Env2 RelCur",
        },
        ValueEntry {
            value: 48,
            name: "Env3 Attack",
        },
        ValueEntry {
            value: 49,
            name: "Env3 Decay",
        },
        ValueEntry {
            value: 50,
            name: "Env3 Sus",
        },
        ValueEntry {
            value: 51,
            name: "Env3 Rel",
        },
        ValueEntry {
            value: 52,
            name: "Env3 AtCur",
        },
        ValueEntry {
            value: 53,
            name: "Env3 DcyCur",
        },
        ValueEntry {
            value: 54,
            name: "Env3 SuSCur",
        },
        ValueEntry {
            value: 55,
            name: "Env3 RelCur",
        },
        ValueEntry {
            value: 56,
            name: "VCA All",
        },
        ValueEntry {
            value: 57,
            name: "VCA Active",
        },
        ValueEntry {
            value: 58,
            name: "VCA EnvDep",
        },
        ValueEntry {
            value: 59,
            name: "Pan Spread",
        },
        ValueEntry {
            value: 60,
            name: "VCA Pan",
        },
        ValueEntry {
            value: 61,
            name: "OSC2 Lvl",
        },
        ValueEntry {
            value: 62,
            name: "Noise Lvl",
        },
        ValueEntry {
            value: 63,
            name: "HP Freq",
        },
        ValueEntry {
            value: 64,
            name: "Uni Detune",
        },
        ValueEntry {
            value: 65,
            name: "OSC Drift",
        },
        ValueEntry {
            value: 66,
            name: "Param Drift",
        },
        ValueEntry {
            value: 67,
            name: "Drift Rate",
        },
        ValueEntry {
            value: 68,
            name: "Arp Gate",
        },
        ValueEntry {
            value: 69,
            name: "Seq Slew",
        },
        ValueEntry {
            value: 70,
            name: "Mod 1 Dep",
        },
        ValueEntry {
            value: 71,
            name: "Mod 2 Dep",
        },
        ValueEntry {
            value: 72,
            name: "Mod 3 Dep",
        },
        ValueEntry {
            value: 73,
            name: "Mod 4 Dep",
        },
        ValueEntry {
            value: 74,
            name: "Mod 5 Dep",
        },
        ValueEntry {
            value: 75,
            name: "Mod 6 Dep",
        },
        ValueEntry {
            value: 76,
            name: "Mod 7 Dep",
        },
        ValueEntry {
            value: 77,
            name: "Mod 8 Dep",
        },
        ValueEntry {
            value: 78,
            name: "Fx 1 Param 1",
        },
        ValueEntry {
            value: 79,
            name: "Fx 1 Param 2",
        },
        ValueEntry {
            value: 80,
            name: "Fx 1 Param 3",
        },
        ValueEntry {
            value: 81,
            name: "Fx 1 Param 4",
        },
        ValueEntry {
            value: 82,
            name: "Fx 1 Param 5",
        },
        ValueEntry {
            value: 83,
            name: "Fx 1 Param 6",
        },
        ValueEntry {
            value: 84,
            name: "Fx 1 Param 7",
        },
        ValueEntry {
            value: 85,
            name: "Fx 1 Param 8",
        },
        ValueEntry {
            value: 86,
            name: "Fx 1 Param 9",
        },
        ValueEntry {
            value: 87,
            name: "Fx 1 Param 10",
        },
        ValueEntry {
            value: 88,
            name: "Fx 1 Param 11",
        },
        ValueEntry {
            value: 89,
            name: "Fx 1 Param 12",
        },
        ValueEntry {
            value: 90,
            name: "Fx 2 Param 1",
        },
        ValueEntry {
            value: 91,
            name: "Fx 2 Param 2",
        },
        ValueEntry {
            value: 92,
            name: "Fx 2 Param 3",
        },
        ValueEntry {
            value: 93,
            name: "Fx 2 Param 4",
        },
        ValueEntry {
            value: 94,
            name: "Fx 2 Param 5",
        },
        ValueEntry {
            value: 95,
            name: "Fx 2 Param 6",
        },
        ValueEntry {
            value: 96,
            name: "Fx 2 Param 7",
        },
        ValueEntry {
            value: 97,
            name: "Fx 2 Param 8",
        },
        ValueEntry {
            value: 98,
            name: "Fx 2 Param 9",
        },
        ValueEntry {
            value: 99,
            name: "Fx 2 Param 10",
        },
        ValueEntry {
            value: 100,
            name: "Fx 2 Param 11",
        },
        ValueEntry {
            value: 101,
            name: "Fx 2 Param 12",
        },
        ValueEntry {
            value: 102,
            name: "Fx 3 Param 1",
        },
        ValueEntry {
            value: 103,
            name: "Fx 3 Param 2",
        },
        ValueEntry {
            value: 104,
            name: "Fx 3 Param 3",
        },
        ValueEntry {
            value: 105,
            name: "Fx 3 Param 4",
        },
        ValueEntry {
            value: 106,
            name: "Fx 3 Param 5",
        },
        ValueEntry {
            value: 107,
            name: "Fx 3 Param 6",
        },
        ValueEntry {
            value: 108,
            name: "Fx 3 Param 7",
        },
        ValueEntry {
            value: 109,
            name: "Fx 3 Param 8",
        },
        ValueEntry {
            value: 110,
            name: "Fx 3 Param 9",
        },
        ValueEntry {
            value: 111,
            name: "Fx 3 Param 10",
        },
        ValueEntry {
            value: 112,
            name: "Fx 3 Param 11",
        },
        ValueEntry {
            value: 113,
            name: "Fx 3 Param 12",
        },
        ValueEntry {
            value: 114,
            name: "Fx 4 Param 1",
        },
        ValueEntry {
            value: 115,
            name: "Fx 4 Param 2",
        },
        ValueEntry {
            value: 116,
            name: "Fx 4 Param 3",
        },
        ValueEntry {
            value: 117,
            name: "Fx 4 Param 4",
        },
        ValueEntry {
            value: 118,
            name: "Fx 4 Param 5",
        },
        ValueEntry {
            value: 119,
            name: "Fx 4 Param 6",
        },
        ValueEntry {
            value: 120,
            name: "Fx 4 Param 7",
        },
        ValueEntry {
            value: 121,
            name: "Fx 4 Param 8",
        },
        ValueEntry {
            value: 122,
            name: "Fx 4 Param 9",
        },
        ValueEntry {
            value: 123,
            name: "Fx 4 Param 10",
        },
        ValueEntry {
            value: 124,
            name: "Fx 4 Param 11",
        },
        ValueEntry {
            value: 125,
            name: "Fx 4 Param 12",
        },
        ValueEntry {
            value: 126,
            name: "Fx 1 Level",
        },
        ValueEntry {
            value: 127,
            name: "Fx 2 Level",
        },
        ValueEntry {
            value: 128,
            name: "Fx 3 Level",
        },
        ValueEntry {
            value: 129,
            name: "Fx 4 Level",
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
        },
        ValueEntry {
            value: 1,
            name: "AmbVerb",
        },
        ValueEntry {
            value: 2,
            name: "RoomRev",
        },
        ValueEntry {
            value: 3,
            name: "VintageRev",
        },
        ValueEntry {
            value: 4,
            name: "HallRev",
        },
        ValueEntry {
            value: 5,
            name: "ChamberRev",
        },
        ValueEntry {
            value: 6,
            name: "PlateRev",
        },
        ValueEntry {
            value: 7,
            name: "RichPltRev",
        },
        ValueEntry {
            value: 8,
            name: "GatedRev",
        },
        ValueEntry {
            value: 9,
            name: "Reverse",
        },
        ValueEntry {
            value: 10,
            name: "ChorusVerb",
        },
        ValueEntry {
            value: 11,
            name: "DelayVerb",
        },
        ValueEntry {
            value: 12,
            name: "FlangVerb",
        },
        ValueEntry {
            value: 13,
            name: "MidasEQ",
        },
        ValueEntry {
            value: 14,
            name: "Enhancer",
        },
        ValueEntry {
            value: 15,
            name: "FairComp",
        },
        ValueEntry {
            value: 16,
            name: "MulBndDist",
        },
        ValueEntry {
            value: 17,
            name: "RackAmp",
        },
        ValueEntry {
            value: 18,
            name: "EdisonEX1",
        },
        ValueEntry {
            value: 19,
            name: "Auto Pan",
        },
        ValueEntry {
            value: 20,
            name: "NoiseGate",
        },
        ValueEntry {
            value: 21,
            name: "Delay",
        },
        ValueEntry {
            value: 22,
            name: "3TapDelay",
        },
        ValueEntry {
            value: 23,
            name: "4TapDelay",
        },
        ValueEntry {
            value: 24,
            name: "T-RayDelay",
        },
        ValueEntry {
            value: 25,
            name: "DecimDelay",
        },
        ValueEntry {
            value: 26,
            name: "ModDlyRev",
        },
        ValueEntry {
            value: 27,
            name: "Chorus",
        },
        ValueEntry {
            value: 28,
            name: "Chorus-D",
        },
        ValueEntry {
            value: 29,
            name: "Flanger",
        },
        ValueEntry {
            value: 30,
            name: "Phaser",
        },
        ValueEntry {
            value: 31,
            name: "MoodFilter",
        },
        ValueEntry {
            value: 32,
            name: "DualPitch",
        },
        ValueEntry {
            value: 33,
            name: "RotarySpkr",
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
