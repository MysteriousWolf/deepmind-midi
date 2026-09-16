//! The shapes a set of parameters makes, for a host that draws them.
//!
//! [`param`](crate::param) says what every parameter *is*. This says what a set
//! of them *makes*: the envelope four bytes describe, the wave an LFO shape
//! selects, the response a filter frequency and a pole count give, the gates an
//! arpeggiator opens.
//!
//! Those are facts about the instrument rather than facts about pixels. Where a
//! filter's corner sits for a given byte, what a `Sample & Hold` looks like, how
//! an attack bends at a curve of 200 — a window is the wrong place to decide any
//! of them and the wrong place to keep them right.
//!
//! ```
//! use deepmind_midi::generator::{self, EnvelopeId, Scale};
//! use deepmind_midi::ProtocolVersion;
//! use deepmind_midi::program::Program;
//!
//! let mut program = Program::new(ProtocolVersion::V6);
//! program.set_vca_envelope_attack_time(0);
//! program.set_vca_envelope_decay_time(0);
//! program.set_vca_envelope_sustain_level(255);
//! program.set_vca_envelope_sustain_curve(128);   // a level sustain
//!
//! let envelope = generator::envelope(&program, EnvelopeId::Vca);
//! assert_eq!(envelope.at(0.5), 1.0);             // straight to full, and held
//! assert_eq!(envelope.scale(), Scale::Normalised);
//!
//! // Below the centre the sustain sags instead, which is what the byte is for.
//! program.set_vca_envelope_sustain_curve(0);
//! assert!(generator::envelope(&program, EnvelopeId::Vca).at(1.0) < 0.01);
//! ```
//!
//! # Sampled, not rendered
//!
//! A [`Generator`] is a function of the parameters, and [`Generator::at`] is
//! how a host samples it: `t` walks `0..=1` along the shape and the answer is
//! `0..=1` of its range. Nothing here produces a bitmap, an SVG, a widget or a
//! colour, for the reason the [`effect`](crate::effect) panels give: a desktop
//! window and a plugin window want the same shape at two sizes, and neither can
//! theme a picture it did not draw.
//!
//! Nor does anything here run a clock. A host that wants an LFO to move owns its
//! own frame timing and samples a phase; this library never blocks and never
//! spawns a thread.
//!
//! Nothing here touches audio. A generator a host samples into a picture is a
//! description of the instrument, not an emulation of it.
//!
//! # What the axis is, and what it is not
//!
//! [`Generator::scale`] is the honest half. The manual prints the two ends of a
//! parameter's range and never the curve between them, so a plausible `2.4 s`
//! for a byte is wrong in a way nobody looking at it can see. Where that is the
//! case the answer is [`Scale::Normalised`], which says *this shape is real and
//! its axis is not published* — which is exactly what a host needs in order not
//! to invent one.
//!
//! There is no `Seconds` and no `Hertz`. Not one generator here has a published
//! one: every time and every frequency on this instrument is a byte whose curve
//! has not been measured. See [Raw values stay
//! raw](https://github.com/MysteriousWolf/deepmind-midi/blob/main/docs/architecture.md).
//! The two scales beside it are the ones that *are* published: a turn of an LFO
//! and an octave either side of a filter's corner. [`Scale`] is
//! `non_exhaustive` so that a measured curve can add to them.
//!
//! The arpeggiator's gates carry no [`Scale`] at all, because they are not a
//! [`Generator`]: a gate is two numbers rather than a curve, and [`Gate`] gives
//! both of them in steps of the arpeggiator's clock outright.
//!
//! # Where a shape is this library's reading
//!
//! Two things are drawn from what the manual states in words and pictures rather
//! than in numbers, and both are marked where they are used:
//!
//! - **The envelope curves.** Section 8.8.4 prints the response at five curve
//!   settings and the feature list calls them *"linear, exponential and reverse
//!   exponential"*, so the family is published and the law is not. The law used
//!   here is stated on [`envelope`], and it has the properties the pictures
//!   have: the centre value is a straight line and the two ends bend the two
//!   ways.
//! - **`Sample & Hold` and `Sample & Glide`.** A step per cycle is the
//!   instrument's behaviour; the numbers it lands on are random. [`lfo`] gives a
//!   fixed, documented sequence so that a picture is stable between frames, and
//!   says plainly that it is not the instrument's stream.
//!
//! Everything else follows from a number the manual prints.

use crate::math;
use crate::param::ParamId;
use crate::program::{LfoShape, Program, VcfPoleMode};

/// How far `0..=1` reaches along a [`Generator`]'s horizontal.
///
/// Reached through [`Generator::scale`]. A host that cannot label an axis draws
/// the shape without one, which is the right answer rather than a failure: see
/// the module documentation for why there is no `Seconds`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Scale {
    /// An ordering and nothing more.
    ///
    /// The shape is real; what its horizontal is measured in has not been
    /// published. A host draws it without an axis.
    Normalised,
    /// `0..=1` covers this many turns of a repeating cycle.
    ///
    /// A cycle is a cycle whatever the rate byte does, so this one is exact.
    Turns(f32),
    /// `0..=1` covers this many octaves, centred on the shape's own corner.
    ///
    /// The slope of a filter is published in decibels per octave, so the
    /// response *about* the corner is known even though the frequency the
    /// corner sits at is a byte with no published curve.
    Octaves(f32),
}

/// A shape a set of parameters makes, sampled by the host.
///
/// Built by [`envelope`], [`lfo`] and [`filter_response`], and by
/// [`effect::response`](crate::effect::response) for the effects that have one.
/// Copy it, keep it, sample it as many times as the picture needs: it holds the
/// parameters it was built from and reads nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Generator {
    shape: Shape,
    scale: Scale,
}

/// What a generator is a shape of.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Shape {
    /// An envelope, laid out with each segment's share of the width taken from
    /// its own byte.
    Envelope(Envelope),
    /// An LFO, over as many cycles as its shape needs to show itself.
    Lfo {
        /// Which of the seven.
        shape: LfoShape,
        /// Cycles the horizontal covers.
        turns: f32,
    },
    /// The taps of a delay, as an impulse train.
    Taps {
        /// Where each tap falls along the horizontal, and how tall it is.
        taps: [(f32, f32); MAX_TAPS],
        /// How many of `taps` are used.
        used: usize,
    },
    /// A low-pass response about its own corner.
    Filter {
        /// Decibels per octave the response rolls off at.
        slope: f32,
        /// Resonant peak height, `0..=1` of the resonance byte.
        resonance: f32,
        /// Octaves either side of the corner the horizontal covers.
        span: f32,
    },
}

/// An envelope's four segments, as shares of the width and bends within it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Envelope {
    /// Where the attack ends, the decay ends and the sustain ends, as fractions
    /// of the whole width. The release runs from the last to 1.
    ends: [f32; 3],
    /// Level the decay falls to and the sustain starts from.
    sustain: f32,
    /// Level the sustain has drifted to by the time the release begins.
    sustain_end: f32,
    /// Bend of the attack, decay, sustain and release segments.
    bends: [f32; 4],
}

impl Generator {
    /// Returns the value at `t` along the shape, in `0..=1` of its range.
    ///
    /// `t` is clamped to `0..=1`, so a host sampling a pixel either side of its
    /// box gets the ends rather than nonsense.
    ///
    /// ```
    /// use deepmind_midi::generator::{self, LfoId};
    /// use deepmind_midi::ProtocolVersion;
    /// use deepmind_midi::program::{LfoShape, Program};
    ///
    /// let mut program = Program::new(ProtocolVersion::V6);
    /// program.set_lfo1_shape(LfoShape::Triangle);
    /// let triangle = generator::lfo(&program, LfoId::One);
    ///
    /// // A triangle starts at the bottom, peaks halfway and comes back.
    /// assert!(triangle.at(0.0) < 0.01);
    /// assert!(triangle.at(0.5) > 0.99);
    /// assert!(triangle.at(1.0) < 0.01);
    /// ```
    #[must_use]
    pub fn at(&self, t: f32) -> f32 {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let value = match self.shape {
            Shape::Envelope(envelope) => envelope.at(t),
            Shape::Lfo { shape, turns } => lfo_at(shape, t * turns),
            Shape::Taps { taps, used } => taps_at(&taps, used, t),
            Shape::Filter {
                slope,
                resonance,
                span,
            } => filter_at(slope, resonance, span, t),
        };
        value.clamp(0.0, 1.0)
    }

    /// Returns what this shape's horizontal is measured in.
    #[must_use]
    pub const fn scale(&self) -> Scale {
        self.scale
    }
}

/// Which of the three envelopes to read.
///
/// The instrument's own names: the VCA envelope shapes the level, the VCF
/// envelope the filter, and the third is spare and reaches the modulation
/// matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EnvelopeId {
    /// The amplifier envelope, `VCA Envelope` on the front panel.
    Vca,
    /// The filter envelope, `VCF Envelope`.
    Vcf,
    /// The auxiliary envelope, `MOD ENV`.
    Mod,
}

impl EnvelopeId {
    /// Every envelope, in the order the program stores them.
    pub const ALL: [Self; 3] = [Self::Vca, Self::Vcf, Self::Mod];

    /// Returns the eight parameters this envelope is made of: the four times
    /// and levels, then the four curves, in attack, decay, sustain, release
    /// order.
    const fn parameters(self) -> [ParamId; 8] {
        match self {
            Self::Vca => [
                ParamId::VcaEnvelopeAttackTime,
                ParamId::VcaEnvelopeDecayTime,
                ParamId::VcaEnvelopeSustainLevel,
                ParamId::VcaEnvelopeReleaseTime,
                ParamId::VcaEnvelopeAttackCurve,
                ParamId::VcaEnvelopeDecayCurve,
                ParamId::VcaEnvelopeSustainCurve,
                ParamId::VcaEnvelopeReleaseCurve,
            ],
            Self::Vcf => [
                ParamId::VcfEnvelopeAttackTime,
                ParamId::VcfEnvelopeDecayTime,
                ParamId::VcfEnvelopeSustainLevel,
                ParamId::VcfEnvelopeReleaseTime,
                ParamId::VcfEnvelopeAttackCurve,
                ParamId::VcfEnvelopeDecayCurve,
                ParamId::VcfEnvelopeSustainCurve,
                ParamId::VcfEnvelopeReleaseCurve,
            ],
            Self::Mod => [
                ParamId::ModEnvelopeAttackTime,
                ParamId::ModEnvelopeDecayTime,
                ParamId::ModEnvelopeSustainLevel,
                ParamId::ModEnvelopeReleaseTime,
                ParamId::ModEnvelopeAttackCurve,
                ParamId::ModEnvelopeDecayCurve,
                ParamId::ModEnvelopeSustainCurve,
                ParamId::ModEnvelopeReleaseCurve,
            ],
        }
    }
}

/// Which of the two LFOs to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LfoId {
    /// `LFO 1`.
    One,
    /// `LFO 2`.
    Two,
}

impl LfoId {
    /// Every LFO, in the order the program stores them.
    pub const ALL: [Self; 2] = [Self::One, Self::Two];

    /// Returns the parameter holding this LFO's shape.
    const fn shape(self) -> ParamId {
        match self {
            Self::One => ParamId::Lfo1Shape,
            Self::Two => ParamId::Lfo2Shape,
        }
    }
}

/// How long the sustain is drawn for, against the three timed segments.
///
/// A held note is as long as the player holds it, so there is no byte that says
/// how wide the sustain should be. This is the width it is given when the three
/// times are at their shortest, in the same units they are measured in, which
/// puts a flat stretch in the picture rather than an instant corner. It is a
/// drawing convention and the only one in this module.
const SUSTAIN_WIDTH: f32 = 0.5;

/// Returns the shape one of the three envelopes makes.
///
/// The three times share the width in proportion to their own bytes, with a
/// fixed stretch for the sustain between the decay and the release, so an
/// envelope with a long attack is drawn with a long attack. The horizontal is
/// [`Scale::Normalised`]: the manual gives no mapping from a time byte to
/// seconds, and one invented here would be wrong in a way a picture could not
/// show.
///
/// # The bends
///
/// Each segment is bent by its own curve byte, which the manual describes as
/// moving *"between linear, exponential and reverse exponential"* and prints at
/// five settings in section 8.8.4. The law is this library's, because the manual
/// gives pictures rather than an equation, and it is the rational bend
/// `u / (u + k(1 - u))` with `k` a power of two either side of 1 at the centre
/// byte. That has the properties the printed curves have: 128 is a straight
/// line, below it bends one way, above it the other, and both ends of every
/// segment stay where they were.
///
/// The sustain byte is a slope rather than a bend, which is what the manual
/// calls it: at 128 the sustain is level, below it sags toward silence, and
/// above it climbs, reaching no further than the level it started from could go.
///
/// ```
/// use deepmind_midi::generator::{self, EnvelopeId};
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::Program;
///
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_vcf_envelope_attack_time(255);
/// program.set_vcf_envelope_decay_time(0);
/// program.set_vcf_envelope_sustain_level(128);
/// program.set_vcf_envelope_release_time(0);
///
/// // A long attack against nothing else takes most of the width.
/// let envelope = generator::envelope(&program, EnvelopeId::Vcf);
/// assert!(envelope.at(0.25) < 1.0);
/// assert!(envelope.at(0.95) < 0.6);   // down at the sustain by the end
/// ```
#[must_use]
pub fn envelope(program: &Program, which: EnvelopeId) -> Generator {
    let [
        attack,
        decay,
        sustain,
        release,
        attack_curve,
        decay_curve,
        sustain_curve,
        release_curve,
    ] = which.parameters().map(|parameter| program.get(parameter));

    let attack = byte(attack);
    let decay = byte(decay);
    let release = byte(release);
    let sustain = byte(sustain);

    let total = attack + decay + SUSTAIN_WIDTH + release;
    // `total` is at least SUSTAIN_WIDTH, so this never divides by zero.
    let share = |of: f32| of / total;
    let first = share(attack);
    let second = first + share(decay);
    let third = second + share(SUSTAIN_WIDTH);

    // The sustain slope runs toward silence below the centre and toward full
    // above it, reaching the end of its travel at the ends of the byte.
    let slope = (byte(sustain_curve) - 0.5) * 2.0;
    let sustain_end = if slope < 0.0 {
        sustain * (1.0 + slope)
    } else {
        sustain + (1.0 - sustain) * slope
    };

    Generator {
        shape: Shape::Envelope(Envelope {
            ends: [first, second, third],
            sustain,
            sustain_end,
            bends: [
                bend(attack_curve),
                bend(decay_curve),
                1.0,
                bend(release_curve),
            ],
        }),
        scale: Scale::Normalised,
    }
}

/// Returns one cycle of the wave an LFO's shape selects.
///
/// The horizontal is [`Scale::Turns`] of exactly one: a cycle is a cycle
/// whatever the rate byte is doing, which makes this the one generator here
/// whose axis is fully published. What a host cannot label is how long the cycle
/// takes, and that is the rate parameter rather than this shape.
///
/// A program whose shape byte names no shape is drawn as a sine, which is the
/// byte the instrument ships at.
///
/// # The two random shapes
///
/// `Sample & Hold` takes a new random value each cycle and holds it; `Sample &
/// Glide` slides to it instead. One cycle of either is one value, so one cycle
/// of it is a flat line and shows nothing: these two are the reason
/// [`Scale::Turns`] carries a number rather than being a marker. They are given
/// [`SAMPLED_TURNS`] cycles, and the scale says so, so a host that labels its
/// axis still labels it correctly and one that does not gets a picture with the
/// stepping in it.
///
/// The stepping is the instrument's behaviour and is drawn as such. The values
/// are not: they are random on the instrument, and what this returns is a fixed
/// sequence, the same on every call, so that a picture does not flicker between
/// frames and two hosts drawing the same patch draw the same thing. It is not
/// the instrument's stream and no host should present it as one.
#[must_use]
pub fn lfo(program: &Program, which: LfoId) -> Generator {
    let shape = LfoShape::from_raw(program.get(which.shape())).unwrap_or(LfoShape::Sine);
    let turns = match shape {
        LfoShape::SampleAndHold | LfoShape::SampleAndGlide => SAMPLED_TURNS,
        _ => 1.0,
    };
    Generator {
        shape: Shape::Lfo { shape, turns },
        scale: Scale::Turns(turns),
    }
}

/// Octaves either side of the corner that [`filter_response`] covers.
///
/// Wide enough to show a 24 dB per octave slope reaching the floor and narrow
/// enough that the flat part is not the whole picture.
const FILTER_SPAN: f32 = 3.0;

/// Decibels above unity that the top of [`filter_response`]'s vertical is.
///
/// Headroom for the resonant peak, which rises above the passband: without it a
/// filter with its resonance up would draw as a flat line with a notch in it.
pub const FILTER_CEILING_DB: f32 = 12.0;

/// Decibels below unity that the bottom of [`filter_response`]'s vertical is.
///
/// Far enough down that a 24 dB per octave slope takes two octaves to reach it,
/// which is what puts the knee in the middle of the picture rather than at the
/// edge of it.
pub const FILTER_FLOOR_DB: f32 = -48.0;

/// Where unity gain sits on [`filter_response`]'s vertical, as `0..=1`.
///
/// The vertical is linear in decibels, from [`FILTER_FLOOR_DB`] at the bottom to
/// [`FILTER_CEILING_DB`] at the top, so this follows from the two. Decibels are
/// the unit the instrument's own slope is published in — 24 per octave with four
/// poles and 12 with two — so this is the axis the response is actually a fact
/// about, rather than a linear magnitude that draws every filter as a cliff.
///
/// A host marking the level the filter passes at draws its line here.
pub const FILTER_UNITY: f32 = -FILTER_FLOOR_DB / (FILTER_CEILING_DB - FILTER_FLOOR_DB);

/// Returns the low-pass response of the filter the program describes.
///
/// The horizontal is [`Scale::Octaves`] centred on the filter's own corner, which
/// sits at `t = 0.5`. That is the honest axis: the slope is published — 24 dB per
/// octave with four poles, 12 with two, from section 8.5.3 — so the response
/// *about* the corner is a fact, while the frequency the corner sits at is a
/// byte whose curve the manual does not give. A host that wants to move the
/// picture as `VCF Frequency` moves reads that byte itself; the byte's position
/// in its range is the only honest thing to move it by.
///
/// The vertical is decibels, linear from [`FILTER_FLOOR_DB`] at the bottom to
/// [`FILTER_CEILING_DB`] at the top, with unity gain at [`FILTER_UNITY`]. That
/// is the unit the slope is published in, so a host can label this axis and be
/// right. The resonance byte lifts a peak at the corner into the headroom above
/// unity. The instrument's filter self-oscillates at high resonance, which is a
/// tone rather than a gain, and no height drawn here should be read as how loud
/// that is.
///
/// # The high-pass is not in this
///
/// There is a 6 dB per octave high-pass beside the low-pass, and it is left out
/// rather than drawn wrong: putting the two on one axis needs the spacing
/// between their corners, and that is two bytes whose curves are both
/// unpublished. An octave axis about one corner says nothing about where the
/// other one is.
///
/// ```
/// use deepmind_midi::generator::{self, Scale};
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::{Program, VcfPoleMode};
///
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_vcf2_pole_mode(VcfPoleMode::FourPole);   // 24 dB/octave
/// program.set_vcf_resonance(0);
///
/// let response = generator::filter_response(&program);
/// assert!(matches!(response.scale(), Scale::Octaves(_)));
///
/// // Flat at unity below the corner, and well down above it.
/// assert!((response.at(0.2) - generator::FILTER_UNITY).abs() < 0.01);
/// assert!(response.at(0.5) > response.at(0.8));
/// ```
#[must_use]
pub fn filter_response(program: &Program) -> Generator {
    // Four poles unless the byte says two, which is also what a byte the table
    // does not name falls back to: the instrument's own default.
    let slope = match program.vcf2_pole_mode() {
        Some(VcfPoleMode::TwoPole) => 12.0,
        Some(VcfPoleMode::FourPole) | None => 24.0,
    };
    Generator {
        shape: Shape::Filter {
            slope,
            resonance: byte(program.get(ParamId::VcfResonance)),
            span: FILTER_SPAN,
        },
        scale: Scale::Octaves(FILTER_SPAN * 2.0),
    }
}

/// One note the arpeggiator opens the gate for.
///
/// Reached through [`arpeggiator_gates`]. Both ends are in steps of the
/// arpeggiator's clock, counting from the start of the first step drawn, so a
/// gate from `1.0` to `1.5` is the second step held for half its length.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gate {
    start: f32,
    end: f32,
}

impl Gate {
    /// Returns the step the gate opens at.
    #[must_use]
    pub const fn start(&self) -> f32 {
        self.start
    }

    /// Returns the step the gate closes at.
    #[must_use]
    pub const fn end(&self) -> f32 {
        self.end
    }

    /// Returns how long the gate is open, in steps.
    #[must_use]
    pub fn length(&self) -> f32 {
        self.end - self.start
    }
}

/// How many steps [`arpeggiator_gates`] draws.
///
/// The manual's own illustration of gate time uses two consecutive notes. Four
/// is two of those, which reads as a pattern rather than as a pair.
pub const GATES_DRAWN: usize = 4;

/// Returns the gates the arpeggiator opens, one per step.
///
/// This is the one generator here whose vertical *and* horizontal are both
/// published outright. Section 8.1.6 gives the mapping in words: the gate time
/// byte runs 0 to 255, *"with 0 being no note, and 255 being a full note"*, and
/// 128, the default, *"represents half of a step"*. So the open fraction is the
/// byte over 255 and nothing is being guessed.
///
/// A gate byte of zero yields no gates at all, because that is what no note
/// means.
///
/// # What is not in this
///
/// Which steps have a note on them. That is the arpeggiator pattern, and the
/// 32 presets and 32 user patterns are values `Arp Pattern` selects without the
/// manual printing what is in them. So these are the step slots and their gate
/// lengths, which is what the gate time fader controls; a host drawing a pattern
/// it has read from the instrument masks these with it.
///
/// The per-step gate time in the pattern editor multiplies this, and is part of
/// the same unpublished pattern, so what is returned is the full-length case the
/// manual describes.
///
/// ```
/// use deepmind_midi::generator::{self, GATES_DRAWN};
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::Program;
///
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_arp_gate_time(128);
///
/// let gates: Vec<_> = generator::arpeggiator_gates(&program).collect();
/// assert_eq!(gates.len(), GATES_DRAWN);
///
/// // 128 is half a step, which the manual states outright.
/// assert!((gates[0].length() - 128.0 / 255.0).abs() < 1e-6);
/// assert_eq!(gates[1].start(), 1.0);
/// ```
pub fn arpeggiator_gates(program: &Program) -> impl Iterator<Item = Gate> {
    let open = byte(program.get(ParamId::ArpGateTime));
    (0..GATES_DRAWN).filter_map(move |step| {
        if open <= 0.0 {
            return None;
        }
        let start = step_start(step);
        Some(Gate {
            start,
            end: start + open,
        })
    })
}

/// Returns the step a gate starts on, as a float.
///
/// `GATES_DRAWN` is four, so the cast is exact; it is written this way rather
/// than with an `as` so that a larger count could not silently lose precision.
fn step_start(step: usize) -> f32 {
    f32::from(u8::try_from(step).unwrap_or(u8::MAX))
}

/// Most taps any delay on this instrument enumerates, which is the four-tap
/// delay's.
pub(crate) const MAX_TAPS: usize = 4;

/// Builds the impulse train a delay's taps make.
///
/// Used by [`effect::response`](crate::effect::response), which works the taps
/// out where the algorithm's slots are known. Each pair is where a tap falls
/// along `0..=1` and how tall it is.
pub(crate) fn taps(taps: [(f32, f32); MAX_TAPS], used: usize) -> Generator {
    Generator {
        shape: Shape::Taps {
            taps,
            used: used.min(MAX_TAPS),
        },
        scale: Scale::Normalised,
    }
}

/// Returns the height of a delay's impulse train at `t`.
///
/// Each tap is a spike of its own height. A host sampling at a sensible
/// resolution lands on every one of them, because the spikes are given a width
/// rather than being a single point that a sample could step over.
fn taps_at(taps: &[(f32, f32); MAX_TAPS], used: usize, t: f32) -> f32 {
    /// Half-width of a tap, as a fraction of the whole.
    const WIDTH: f32 = 0.014;

    let mut height = 0.0_f32;
    for &(at, tall) in taps.iter().take(used) {
        let distance = (t - at).abs();
        if distance < WIDTH {
            height = height.max(tall * (1.0 - distance / WIDTH));
        }
    }
    height
}

/// Returns a raw byte as `0..=1`.
fn byte(raw: u8) -> f32 {
    f32::from(raw) / 255.0
}

/// Returns the bend factor a curve byte selects.
///
/// One at the centre byte, which is a straight line, and a power of two either
/// side of it, so the byte reads symmetrically: a curve as far below the centre
/// bends exactly as far the other way as one the same distance above.
fn bend(curve: u8) -> f32 {
    math::exp2((f32::from(curve) - 128.0) / 64.0)
}

/// Applies a bend to a position along a segment.
///
/// `u / (u + k(1 - u))`: the identity at `k = 1`, above the diagonal below it
/// and below above it, and fixed at both ends whatever `k` is.
fn bent(u: f32, k: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    let denominator = u + k * (1.0 - u);
    if denominator <= 0.0 {
        u
    } else {
        u / denominator
    }
}

impl Envelope {
    /// Returns the level at `t` along the whole envelope.
    fn at(&self, t: f32) -> f32 {
        let [first, second, third] = self.ends;
        if t < first {
            return bent(divide(t, 0.0, first), self.bends[0]);
        }
        if t < second {
            let u = bent(divide(t, first, second), self.bends[1]);
            return 1.0 + (self.sustain - 1.0) * u;
        }
        if t < third {
            let u = divide(t, second, third);
            return self.sustain + (self.sustain_end - self.sustain) * u;
        }
        let u = bent(divide(t, third, 1.0), self.bends[3]);
        self.sustain_end * (1.0 - u)
    }
}

/// Returns where `t` falls between `from` and `to`, as `0..=1`.
///
/// Zero for a segment with no width, which is a segment whose time byte is zero
/// and which the caller is about to step straight over.
fn divide(t: f32, from: f32, to: f32) -> f32 {
    let width = to - from;
    if width <= 0.0 {
        0.0
    } else {
        ((t - from) / width).clamp(0.0, 1.0)
    }
}

/// Cycles [`lfo`] draws for the two shapes that hold a value for a whole one.
///
/// Enough to show that the value changes and that it does not repeat, without
/// making the steps too narrow to see.
pub const SAMPLED_TURNS: f32 = 6.0;

/// The values `Sample & Hold` and `Sample & Glide` step through.
///
/// Fixed rather than random, for the reason [`lfo`] gives. Eight of them so that
/// a host drawing several cycles sees a sequence rather than a repeat, and the
/// last is not the first so the loop does not look like a flat join.
const SAMPLED: [f32; 8] = [0.72, 0.18, 0.95, 0.41, 0.08, 0.63, 0.31, 0.86];

/// Returns one cycle of an LFO shape, as `0..=1`.
fn lfo_at(shape: LfoShape, t: f32) -> f32 {
    match shape {
        // Starting at the bottom rather than the middle, so that every shape in
        // the table begins where the others do and a host can draw them side by
        // side without one appearing shifted.
        LfoShape::Sine => 0.5 - 0.5 * math::cos_turns(t),
        LfoShape::Triangle => {
            if t < 0.5 {
                t * 2.0
            } else {
                2.0 - t * 2.0
            }
        }
        LfoShape::Square => {
            if t < 0.5 {
                1.0
            } else {
                0.0
            }
        }
        LfoShape::RampUp => t,
        LfoShape::RampDown => 1.0 - t,
        LfoShape::SampleAndHold => sampled(t),
        LfoShape::SampleAndGlide => {
            // One cycle is one value, so the glide runs from the value the
            // cycle before landed on to the one this cycle lands on.
            let previous = sampled(t - 1.0);
            previous + (sampled(t) - previous) * math::fract(t)
        }
    }
}

/// Returns the held value for the cycle `t` falls in.
fn sampled(t: f32) -> f32 {
    let cycle = math::floor(t);
    // `SAMPLED` has eight entries, so the remainder is always an index into it.
    let index = (cycle - math::floor(cycle / 8.0) * 8.0).clamp(0.0, 7.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to 0..=7 on the line above"
    )]
    let index = index as usize;
    SAMPLED.get(index).copied().unwrap_or(0.5)
}

/// Returns the low-pass response `t` octaves along, as `0..=1`.
fn filter_at(slope: f32, resonance: f32, span: f32, t: f32) -> f32 {
    // `t` walks `span` octaves either side of the corner, which sits at 0.5.
    let octaves = (t - 0.5) * 2.0 * span;

    // The magnitude of an n-pole low pass is 1 / sqrt(1 + (f/fc)^2n), which in
    // decibels is -10 log10 of the squared term. `slope` is decibels per
    // octave, and a pole is 6 of them, so the exponent is the octaves from the
    // corner times the slope over 6.
    let squared = 2.0 * (slope / 6.0) * octaves;
    // Well above the corner the one is lost in the rounding and the logarithm
    // is the exponent itself, which is also where `exp2` would overflow.
    let log = if squared > 30.0 {
        squared
    } else {
        math::log2(1.0 + math::exp2(squared))
    };
    // 10 log10(x) is 10 log2(x) log10(2), and log10(2) is 0.30103.
    let magnitude_db = -3.010_3 * log;

    // The resonant peak sits at the corner and narrows as it grows, which is
    // what a rising Q does. It is added rather than multiplied so that a filter
    // with no resonance is exactly the roll-off above, and it rises into the
    // headroom above unity rather than pushing against the top of the box.
    let distance = octaves * (2.0 + resonance * 6.0);
    let peak_db = resonance * FILTER_CEILING_DB / (1.0 + distance * distance);

    let db = magnitude_db + peak_db;
    ((db - FILTER_FLOOR_DB) / (FILTER_CEILING_DB - FILTER_FLOOR_DB)).clamp(0.0, 1.0)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{
        EnvelopeId, FILTER_CEILING_DB, FILTER_FLOOR_DB, FILTER_SPAN, FILTER_UNITY, GATES_DRAWN,
        Generator, LfoId, SAMPLED_TURNS, Scale, arpeggiator_gates, envelope, filter_response, lfo,
    };
    use crate::ids::ProtocolVersion;
    use crate::program::{LfoShape, Program, VcfPoleMode};

    /// A program with every byte at its minimum, which is what `Program::new`
    /// builds, plus the curve bytes at their centre so that a segment under
    /// test is a straight line unless the test bends it.
    fn program() -> Program {
        let mut program = Program::new(ProtocolVersion::V6);
        for envelope in EnvelopeId::ALL {
            let [_, _, _, _, attack, decay, sustain, release] = envelope.parameters();
            for curve in [attack, decay, sustain, release] {
                program.set_clamped(curve, 128);
            }
        }
        program
    }

    /// Asserts two values are the same to within what a picture could show.
    fn close(left: f32, right: f32, what: &str) {
        assert!((left - right).abs() < 1e-6, "{what}: {left} is not {right}");
    }

    /// The same, for two samples that should be exactly equal because they come
    /// from the same held value.
    fn same(left: f32, right: f32, what: &str) {
        close(left, right, what);
    }

    /// Samples a generator across its whole range, which is what a host does.
    ///
    /// An iterator rather than a collection, because the crate's own tests
    /// build with no features on and there is no allocator there.
    fn walk(generator: &Generator, steps: u8) -> impl Iterator<Item = f32> + '_ {
        (0..=steps).map(move |step| generator.at(f32::from(step) / f32::from(steps)))
    }

    /// Nothing a host samples can leave the box it is drawing into, whatever it
    /// asks for, including the values outside the range and a NaN.
    #[test]
    fn every_generator_stays_within_its_range() {
        let mut program = program();
        program.set_vca_envelope_attack_time(90);
        program.set_vca_envelope_decay_time(40);
        program.set_vca_envelope_sustain_level(180);
        program.set_vca_envelope_release_time(200);
        program.set_vcf_resonance(255);

        let generators = [
            envelope(&program, EnvelopeId::Vca),
            lfo(&program, LfoId::One),
            filter_response(&program),
        ];
        for generator in generators {
            for value in walk(&generator, 200) {
                assert!((0.0..=1.0).contains(&value), "{value} is outside the box");
            }
            for odd in [-1.0, 1.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let value = generator.at(odd);
                assert!((0.0..=1.0).contains(&value), "at({odd}) gave {value}");
            }
        }
    }

    /// The horizontal is proportional to the time bytes, so a long attack
    /// against a short decay takes more of the width than the other way round.
    #[test]
    fn an_envelope_gives_each_segment_its_own_share_of_the_width() {
        let mut program = program();
        program.set_vcf_envelope_attack_time(255);
        program.set_vcf_envelope_decay_time(0);
        program.set_vcf_envelope_sustain_level(255);
        program.set_vcf_envelope_release_time(0);

        // Attack 1.0 against a sustain width of 0.5: two thirds of the picture
        // is the rise, and it is only halfway up a third of the way along.
        let long = envelope(&program, EnvelopeId::Vcf);
        assert!(long.at(0.33) < 0.6, "{}", long.at(0.33));
        assert!(long.at(0.7) > 0.99, "{}", long.at(0.7));

        // The same envelope with no attack is at the top immediately.
        program.set_vcf_envelope_attack_time(0);
        let short = envelope(&program, EnvelopeId::Vcf);
        assert!(short.at(0.01) > 0.99, "{}", short.at(0.01));
    }

    /// The centre curve byte is a straight line, and the two sides of it bend
    /// the two ways the manual prints. That is the whole claim the bend law
    /// makes, so it is the whole of what is checked.
    #[test]
    fn a_curve_byte_bends_a_segment_both_ways_about_the_centre() {
        let mut program = program();
        program.set_mod_envelope_attack_time(255);
        program.set_mod_envelope_decay_time(0);
        program.set_mod_envelope_sustain_level(255);
        program.set_mod_envelope_release_time(0);

        let rise_at = |curve: u8, program: &mut Program| {
            program.set_mod_envelope_attack_curve(curve);
            envelope(program, EnvelopeId::Mod).at(1.0 / 3.0)
        };

        let straight = rise_at(128, &mut program);
        let fast = rise_at(0, &mut program);
        let slow = rise_at(255, &mut program);

        // Halfway up the attack, a straight line is halfway up.
        assert!((straight - 0.5).abs() < 0.02, "{straight}");
        assert!(fast > straight + 0.2, "{fast} against {straight}");
        assert!(slow < straight - 0.2, "{slow} against {straight}");

        // Whichever way it bends, the ends of the segment do not move.
        for curve in [0, 64, 128, 192, 255] {
            program.set_mod_envelope_attack_curve(curve);
            let bent = envelope(&program, EnvelopeId::Mod);
            assert!(
                bent.at(0.0) < 0.01,
                "curve {curve} starts at {}",
                bent.at(0.0)
            );
        }
    }

    /// Every shape in the table is drawn and moves, and the scale says how many
    /// cycles the picture covers, so a host can lay the seven out side by side
    /// and still label each one correctly.
    #[test]
    fn every_lfo_shape_is_drawn_and_says_how_many_cycles_it_covers() {
        let mut program = program();
        for shape in LfoShape::ALL.iter().copied() {
            program.set_lfo1_shape(shape);
            let generator = lfo(&program, LfoId::One);
            let expected = match shape {
                LfoShape::SampleAndHold | LfoShape::SampleAndGlide => SAMPLED_TURNS,
                _ => 1.0,
            };
            assert_eq!(generator.scale(), Scale::Turns(expected), "{shape}");

            let high = walk(&generator, 64).fold(f32::MIN, f32::max);
            let low = walk(&generator, 64).fold(f32::MAX, f32::min);
            // A shape that does not move is a shape that is not drawn.
            assert!(high - low > 0.3, "{shape} spans {low} to {high}");
        }

        // The shapes the manual names by their geometry are that geometry.
        program.set_lfo1_shape(LfoShape::RampUp);
        let up = lfo(&program, LfoId::One);
        assert!(up.at(0.0) < up.at(0.5) && up.at(0.5) < up.at(1.0));

        program.set_lfo1_shape(LfoShape::RampDown);
        let down = lfo(&program, LfoId::One);
        assert!(down.at(0.0) > down.at(0.5) && down.at(0.5) > down.at(1.0));

        program.set_lfo1_shape(LfoShape::Square);
        let square = lfo(&program, LfoId::One);
        close(square.at(0.25), 1.0, "the top of a square");
        close(square.at(0.75), 0.0, "the bottom of a square");
    }

    /// The random shapes step, and they step the same way every time. A picture
    /// that changed between frames would be worse than no picture.
    #[test]
    fn the_sampled_shapes_step_and_do_not_move_between_calls() {
        let mut program = program();
        program.set_lfo1_shape(LfoShape::SampleAndHold);
        let held = lfo(&program, LfoId::One);

        // One cycle is one value, so within a cycle the hold is flat, and the
        // picture covers several so that the stepping is in it at all.
        let cycle = 1.0 / SAMPLED_TURNS;
        same(
            held.at(cycle * 0.1),
            held.at(cycle * 0.9),
            "a cycle of the hold",
        );
        let steps: [f32; 6] = core::array::from_fn(|cycle| {
            let cycle = f32::from(u8::try_from(cycle).unwrap_or(u8::MAX));
            held.at((cycle + 0.5) / SAMPLED_TURNS)
        });
        assert!(
            steps
                .windows(2)
                .any(|pair| matches!(pair, [left, right] if (left - right).abs() > 1e-6)),
            "the held value never changes: {steps:?}"
        );

        // And the same sequence every time, so a picture does not flicker.
        assert!(walk(&held, 64).eq(walk(&lfo(&program, LfoId::One), 64)));

        // The glide lands on the same values, by sliding rather than stepping,
        // so it moves within a cycle where the hold does not and the two meet
        // where each cycle ends.
        program.set_lfo1_shape(LfoShape::SampleAndGlide);
        let glide = lfo(&program, LfoId::One);
        assert!((glide.at(cycle * 0.1) - glide.at(cycle * 0.9)).abs() > 1e-6);
        // By the end of a cycle the glide has arrived at the value the hold
        // sat on for the whole of it, which is what "glide to it" means.
        let late = cycle * 0.99;
        assert!(
            (glide.at(late) - held.at(late)).abs() < 0.01,
            "{} against {}",
            glide.at(late),
            held.at(late)
        );
    }

    /// Four poles roll off twice as fast as two, which is the published 24
    /// against 12 decibels per octave and the only claim the response makes
    /// about the world.
    #[test]
    fn four_poles_roll_off_twice_as_fast_as_two() {
        let mut program = program();
        program.set_vcf_resonance(0);

        program.set_vcf2_pole_mode(VcfPoleMode::FourPole);
        let steep = filter_response(&program);
        program.set_vcf2_pole_mode(VcfPoleMode::TwoPole);
        let shallow = filter_response(&program);

        // At the corner both are the same 3 dB down, which is what a corner is.
        assert!((steep.at(0.5) - shallow.at(0.5)).abs() < 0.01);
        // Past it the steeper one is further down, everywhere.
        for step in 6_u8..=9 {
            let t = f32::from(step) / 10.0;
            assert!(steep.at(t) < shallow.at(t), "at {t}");
        }
        // And below it both are flat, at the unity the module publishes.
        assert!((steep.at(0.1) - FILTER_UNITY).abs() < 0.01);
        assert!((shallow.at(0.1) - FILTER_UNITY).abs() < 0.01);
    }

    /// The vertical is decibels, so the published slope can be read straight
    /// off the picture: an octave above the corner a four-pole filter is 24 dB
    /// down and a two-pole one is 12. That is section 8.5.3 checked against
    /// what a host would draw.
    #[test]
    fn the_drawn_slope_is_the_published_decibels_per_octave() {
        let mut program = program();
        program.set_vcf_resonance(0);
        let window = FILTER_CEILING_DB - FILTER_FLOOR_DB;

        for (poles, per_octave) in [(VcfPoleMode::FourPole, 24.0), (VcfPoleMode::TwoPole, 12.0)] {
            program.set_vcf2_pole_mode(poles);
            let response = filter_response(&program);

            // One and two octaves above the corner, where the roll-off has
            // settled and the 3 dB at the corner itself is behind it.
            let one = response.at(0.5 + 1.0 / (FILTER_SPAN * 2.0));
            let two = response.at(0.5 + 2.0 / (FILTER_SPAN * 2.0));
            let fallen = (one - two) * window;
            assert!(
                (fallen - per_octave).abs() < 0.5,
                "{poles:?} fell {fallen} dB over the octave, not {per_octave}"
            );
        }
    }

    /// Resonance lifts a peak at the corner and nowhere else.
    #[test]
    fn resonance_lifts_the_corner() {
        let mut program = program();
        program.set_vcf2_pole_mode(VcfPoleMode::FourPole);

        program.set_vcf_resonance(0);
        let dry = filter_response(&program);
        program.set_vcf_resonance(255);
        let wet = filter_response(&program);

        // The peak rises above the passband rather than clipping against the
        // top of the box, which is the whole reason unity is not at 1.0. The
        // vertical is linear in decibels, so a full peak is the ceiling's worth
        // of the height and can be checked as such.
        let window = FILTER_CEILING_DB - FILTER_FLOOR_DB;
        let lift = wet.at(0.5) - dry.at(0.5);
        assert!(
            (lift - FILTER_CEILING_DB / window).abs() < 0.02,
            "lifted by {lift} of the height"
        );
        assert!(wet.at(0.5) > FILTER_UNITY, "{}", wet.at(0.5));
        // Far below the corner the filter is passing everything either way.
        assert!((wet.at(0.05) - dry.at(0.05)).abs() < 0.05);
    }

    /// The gate mapping is the one the manual states in words, so this is a
    /// check against the manual rather than against the implementation.
    #[test]
    fn a_gate_is_the_fraction_of_a_step_the_manual_gives() {
        let mut program = program();

        // "0 being no note": no gates at all.
        program.set_arp_gate_time(0);
        assert_eq!(arpeggiator_gates(&program).count(), 0);

        // "255 being a full note": the gate fills its step and the next one
        // starts where it ends.
        program.set_arp_gate_time(255);
        assert_eq!(arpeggiator_gates(&program).count(), GATES_DRAWN);
        {
            let mut full = arpeggiator_gates(&program);
            let first = full.next().expect("four gates");
            let second = full.next().expect("four gates");
            close(first.length(), 1.0, "a full gate");
            close(first.end(), second.start(), "the next step");
        }

        // "128 ... represents half of a step".
        program.set_arp_gate_time(128);
        let first = arpeggiator_gates(&program).next().expect("four gates");
        close(first.length(), 128.0 / 255.0, "half a step");

        // Every gate opens on its own step and closes before the next one.
        for (step, gate) in arpeggiator_gates(&program).enumerate() {
            let start = f32::from(u8::try_from(step).expect("four steps"));
            close(gate.start(), start, "the step a gate opens on");
            assert!(gate.end() <= start + 1.0);
        }
    }

    /// The three envelopes read three different sets of bytes. A copied match
    /// arm would show up here and nowhere else.
    #[test]
    fn the_three_envelopes_read_their_own_parameters() {
        let mut program = program();
        for envelope in EnvelopeId::ALL {
            let [_, _, sustain, ..] = envelope.parameters();
            program.set_clamped(sustain, 255);
        }
        program.set_vca_envelope_attack_time(255);
        program.set_vcf_envelope_attack_time(0);
        program.set_mod_envelope_attack_time(0);

        let vca = envelope(&program, EnvelopeId::Vca);
        let vcf = envelope(&program, EnvelopeId::Vcf);
        let modulation = envelope(&program, EnvelopeId::Mod);

        // Only the VCA envelope is still rising early on.
        assert!(vca.at(0.1) < 0.5);
        assert!(vcf.at(0.1) > 0.9);
        assert!(modulation.at(0.1) > 0.9);

        // And the two LFOs likewise.
        program.set_lfo1_shape(LfoShape::RampUp);
        program.set_lfo2_shape(LfoShape::RampDown);
        assert!(lfo(&program, LfoId::One).at(0.9) > 0.8);
        assert!(lfo(&program, LfoId::Two).at(0.9) < 0.2);
    }
}
