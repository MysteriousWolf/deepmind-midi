//! The shapes a set of parameters makes, for a host that draws them.
//!
//! [`param`](crate::param) says what every parameter *is*. This says what a set
//! of them *makes*: the envelope four bytes describe, the wave an LFO shape
//! selects, the response a filter frequency and a pole count give, the gates an
//! arpeggiator opens, what the two oscillators are putting out.
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
//! # What a shape says about itself
//!
//! Three things beside the curve, so that a host can rule a screen it did not
//! invent:
//!
//! - [`Generator::rest`] is where the shape sits when it is doing nothing: the
//!   floor of an envelope, the middle of an LFO's range, unity on a filter. A
//!   host draws its baseline there.
//! - [`Generator::marks`] is where along the horizontal something happens: the
//!   segment boundaries of an envelope, the corner of a filter, the end of each
//!   cycle, each tap of a delay. They are the positions this module computed to
//!   build the shape, published rather than thrown away.
//! - [`Generator::anchored`] says whether the left edge means anything. An
//!   envelope starts with the note; a free-running LFO is caught wherever it had
//!   got to, and a host should not rule a phase it does not have.
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
//! or an oscillator, and an octave either side of a filter's corner. [`Scale`]
//! is `non_exhaustive` so that a measured curve can add to them.
//!
//! The arpeggiator's gates carry no [`Scale`] at all, because they are not a
//! [`Generator`]: a gate is two numbers rather than a curve, and [`Gate`] gives
//! both of them in steps of the arpeggiator's clock outright.
//!
//! # Where a shape is this library's reading
//!
//! Some things are drawn from what the manual states in words and pictures
//! rather than in numbers, and every one is marked where it is used:
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
//!   says plainly that it is not the instrument's stream. [`noise`] does the
//!   same for the noise generator.
//! - **The slew.** The manual says the slew rate rounds the corners off an LFO
//!   and turns a square into a ramp between its levels, which is what a rate
//!   limit does; how much limit a byte is worth is this library's, and is
//!   stated on [`lfo`].
//! - **The fade.** [`lfo_fade`] rises in a straight line, because the manual
//!   gives a time for the fade and nothing about its shape.
//! - **The oscillator mix.** The manual prints OSC 1's two waves and not how
//!   they sum. [`oscillator`] adds them at equal weight, which is stated there.
//!
//! Everything else follows from a number the manual prints.

use crate::math;
use crate::param::ParamId;
use crate::program::{LfoShape, Program, PwmSource, VcfPoleMode};

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

/// A position along a [`Generator`]'s horizontal that means something.
///
/// Reached through [`Generator::marks`]. A host decides how to draw one: two
/// dots, a dashed rule, a caption, nothing at all on a screen too small. What it
/// should not be deciding is where one goes, because where the decay starts is a
/// fact about the instrument and not about the screen.
///
/// No name and no unit. [`MarkKind`] says what is there; what to write beside
/// it, if anything, depends on the room a host has and the language it is in.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mark {
    at: f32,
    kind: MarkKind,
}

impl Mark {
    /// Returns where the mark falls, in `0..=1` along the shape.
    #[must_use]
    pub const fn at(&self) -> f32 {
        self.at
    }

    /// Returns what is there.
    #[must_use]
    pub const fn kind(&self) -> MarkKind {
        self.kind
    }
}

/// What a [`Mark`] marks.
///
/// `non_exhaustive` for the same reason [`Scale`] is: a shape that turns out to
/// have a mark nobody thought of should be able to say so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum MarkKind {
    /// A segment of an envelope, or of an LFO's fade, begins here.
    Segment(Segment),
    /// The corner of a filter.
    Corner,
    /// One cycle of a repeating shape ends here.
    Cycle,
    /// A tap of a delay.
    Tap,
    /// A pulse falls here: the width its `PWM Depth` byte sets.
    Width,
    /// As far as pulse width modulation swings the falling edge, one way.
    ///
    /// Two of these either side of the [`Width`](Self::Width), on an
    /// oscillator whose pulse width is being moved by an LFO or an envelope.
    Sweep,
}

/// A segment of an envelope, in the order they run.
///
/// Also the two an LFO's fade has: it rises and is then held, which is an
/// attack and a sustain with nothing after them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Segment {
    /// From nothing to full.
    Attack,
    /// From full down to the sustain level.
    Decay,
    /// Held for as long as the note is, and drawn a fixed width.
    Sustain,
    /// From the sustain level back to nothing.
    Release,
}

impl Segment {
    /// Every segment, in the order an envelope runs through them.
    pub const ALL: [Self; 4] = [Self::Attack, Self::Decay, Self::Sustain, Self::Release];
}

/// A shape a set of parameters makes, sampled by the host.
///
/// Built by [`envelope`], [`lfo`], [`lfo_unipolar`], [`lfo_fade`],
/// [`filter_response`], [`high_pass_response`], [`oscillator`] and [`noise`],
/// and by [`effect::response`](crate::effect::response) for the effects that
/// have one. Copy it, keep it, sample it as many times as the picture needs: it
/// holds the parameters it was built from and reads nothing else.
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
    /// An LFO's fade: a rise to full depth, then held there.
    Fade {
        /// Where the rise ends, as a fraction of the whole width.
        end: f32,
    },
    /// An LFO, over as many cycles as its shape needs to show itself.
    Lfo(Lfo),
    /// The taps of a delay, as an impulse train.
    Taps {
        /// Where each tap falls along the horizontal, and how tall it is.
        taps: [(f32, f32); MAX_TAPS],
        /// How many of `taps` are used.
        used: usize,
    },
    /// A filter response about its own corner.
    Filter(Filter),
    /// What one oscillator is putting out.
    Oscillator(Oscillator),
    /// The noise generator, at its level.
    Noise {
        /// How far from the centre it scatters, `0..=1` of the level byte.
        level: f32,
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

/// What shapes one LFO's picture.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Lfo {
    /// Which of the seven.
    shape: LfoShape,
    /// Cycles the horizontal covers.
    turns: f32,
    /// How much the slew rate byte rounds the corners, `0..=1` of the byte.
    slew: f32,
    /// Whether the picture is read riding up from the floor rather than
    /// swinging about the middle.
    unipolar: bool,
    /// Whether the cycle restarts with each note, so that phase 0 is a moment.
    synced: bool,
}

/// A filter response about its own corner.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Filter {
    /// Decibels per octave the response rolls off at.
    slope: f32,
    /// Resonant peak height, `0..=1` of the resonance byte.
    resonance: f32,
    /// Octaves either side of the corner the horizontal covers.
    span: f32,
    /// Which side of the corner is passed.
    pass: Pass,
}

/// Which side of its corner a filter passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pass {
    /// Below the corner: the response falls to the right.
    Low,
    /// Above the corner: the response rises to the right.
    High,
}

/// What one oscillator is putting out, over one cycle.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Oscillator {
    /// Whether the sawtooth is switched on.
    saw: bool,
    /// Whether the pulse is switched on.
    pulse: bool,
    /// Where in the cycle the pulse falls, `0..=1`.
    width: f32,
    /// How far either side of `width` modulation swings the falling edge, or
    /// zero where the width is set by hand.
    sweep: f32,
    /// How tall the wave is drawn, `0..=1`.
    level: f32,
}

/// Most marks any shape here has: one per cycle of the sampled LFO shapes.
const MAX_MARKS: usize = 6;

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
            Shape::Fade { end } => {
                if t < end {
                    t / end
                } else {
                    1.0
                }
            }
            Shape::Lfo(lfo) => lfo.at(t * lfo.turns),
            Shape::Taps { taps, used } => taps_at(&taps, used, t),
            Shape::Filter(filter) => filter.at(t),
            Shape::Oscillator(oscillator) => oscillator.at(t),
            Shape::Noise { level } => 0.5 + (noise_at(t) - 0.5) * level,
        };
        value.clamp(0.0, 1.0)
    }

    /// Returns what this shape's horizontal is measured in.
    #[must_use]
    pub const fn scale(&self) -> Scale {
        self.scale
    }

    /// Returns where the shape sits when it is doing nothing, in `0..=1` of
    /// its range.
    ///
    /// The floor for an envelope, a fade, or a train of taps. The middle for an
    /// LFO, which swings either side of it, and the floor for one read
    /// unipolar, which rides up from it: see [`lfo_unipolar`]. [`FILTER_UNITY`]
    /// for a filter. The middle for an oscillator and for the noise, which are
    /// waves about zero.
    ///
    /// A host draws its baseline here. It is the level a modulation depth of
    /// nothing leaves a destination at, which is what makes it the line to
    /// measure the rest of the picture from.
    ///
    /// ```
    /// use deepmind_midi::generator::{self, LfoId};
    /// use deepmind_midi::ProtocolVersion;
    /// use deepmind_midi::program::Program;
    ///
    /// let program = Program::new(ProtocolVersion::V6);
    /// assert_eq!(generator::lfo(&program, LfoId::One).rest(), 0.5);
    /// assert_eq!(generator::lfo_unipolar(&program, LfoId::One).rest(), 0.0);
    /// assert_eq!(generator::filter_response(&program).rest(), generator::FILTER_UNITY);
    /// ```
    #[must_use]
    pub const fn rest(&self) -> f32 {
        match self.shape {
            Shape::Envelope(_) | Shape::Fade { .. } | Shape::Taps { .. } => 0.0,
            Shape::Lfo(lfo) => {
                if lfo.unipolar {
                    0.0
                } else {
                    0.5
                }
            }
            Shape::Filter(_) => FILTER_UNITY,
            Shape::Oscillator(_) | Shape::Noise { .. } => 0.5,
        }
    }

    /// Returns whether `t = 0` is a moment the instrument defines.
    ///
    /// `true` for an envelope, a fade or a delay's taps, which begin with the
    /// note, and for a filter, whose left edge is a frequency. `true` for an
    /// LFO whose `Key Sync` is on, because its cycle restarts with each note
    /// and phase 0 is where that note finds it. `false` for one that is free
    /// running, which a note catches wherever it had got to, and for an
    /// oscillator or the noise, whose phase nothing resets.
    ///
    /// A host ruling a screen should not rule a phase it does not have: the
    /// left edge of a free-running LFO's picture is a cycle boundary because a
    /// picture has to start somewhere, not because the instrument does.
    #[must_use]
    pub const fn anchored(&self) -> bool {
        match self.shape {
            Shape::Envelope(_) | Shape::Fade { .. } | Shape::Taps { .. } | Shape::Filter(_) => true,
            Shape::Lfo(lfo) => lfo.synced,
            Shape::Oscillator(_) | Shape::Noise { .. } => false,
        }
    }

    /// Returns the positions along this shape worth ruling, in order.
    ///
    /// Where each segment of an envelope begins, where a filter's corner sits,
    /// where each cycle of a repeating shape ends, where each tap of a delay
    /// falls, where a pulse falls and how far modulation swings it. Every one
    /// is a number this module computed to build the shape, so a host that
    /// wants to write *decay* under the decay does not derive the segment
    /// split a second time from the same four bytes.
    ///
    /// Empty for a shape with nothing to rule, which is the right answer the
    /// same way [`Scale::Normalised`] is for an axis with no units. Two marks
    /// can share a position: an attack whose time byte is zero begins and ends
    /// at the left edge, and the decay begins there too.
    ///
    /// ```
    /// use deepmind_midi::generator::{self, EnvelopeId, MarkKind, Segment};
    /// use deepmind_midi::ProtocolVersion;
    /// use deepmind_midi::program::Program;
    ///
    /// let mut program = Program::new(ProtocolVersion::V6);
    /// program.set_vca_envelope_attack_time(255);
    /// program.set_vca_envelope_decay_time(255);
    ///
    /// let envelope = generator::envelope(&program, EnvelopeId::Vca);
    /// let decay = envelope
    ///     .marks()
    ///     .find(|mark| mark.kind() == MarkKind::Segment(Segment::Decay))
    ///     .expect("an envelope has a decay");
    ///
    /// // The decay begins where the attack has reached the top.
    /// assert!(decay.at() > 0.3);
    /// assert!(envelope.at(decay.at()) > 0.99);
    /// ```
    pub fn marks(&self) -> impl Iterator<Item = Mark> {
        let mut marks = [None; MAX_MARKS];
        let mut count = 0;
        let mut push = |at: f32, kind: MarkKind| {
            if let Some(slot) = marks.get_mut(count) {
                *slot = Some(Mark { at, kind });
                count += 1;
            }
        };
        match self.shape {
            Shape::Envelope(envelope) => {
                let [first, second, third] = envelope.ends;
                for (at, segment) in [0.0, first, second, third].into_iter().zip(Segment::ALL) {
                    push(at, MarkKind::Segment(segment));
                }
            }
            Shape::Fade { end } => {
                push(0.0, MarkKind::Segment(Segment::Attack));
                push(end, MarkKind::Segment(Segment::Sustain));
            }
            Shape::Lfo(lfo) => cycles(lfo.turns, &mut push),
            Shape::Taps { taps, used } => {
                for &(at, _) in taps.iter().take(used) {
                    push(at, MarkKind::Tap);
                }
            }
            Shape::Filter(_) => push(0.5, MarkKind::Corner),
            Shape::Oscillator(oscillator) => {
                if oscillator.pulse {
                    push(oscillator.width, MarkKind::Width);
                    if oscillator.sweep > 0.0 {
                        push(oscillator.width - oscillator.sweep, MarkKind::Sweep);
                        push(oscillator.width + oscillator.sweep, MarkKind::Sweep);
                    }
                }
                cycles(1.0, &mut push);
            }
            Shape::Noise { .. } => {}
        }
        marks.into_iter().flatten()
    }
}

/// Pushes a [`MarkKind::Cycle`] at the end of each of `turns` cycles.
///
/// `turns` is one or [`SAMPLED_TURNS`], so the count is small and exact; the
/// loop is written over a byte so that a larger count could not be cast wrong.
fn cycles(turns: f32, push: &mut impl FnMut(f32, MarkKind)) {
    for cycle in 1..=u8::MAX {
        let cycle = f32::from(cycle);
        if cycle > turns {
            break;
        }
        push(cycle / turns, MarkKind::Cycle);
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

    /// Returns the four parameters that shape this LFO's picture: its shape,
    /// its slew rate, its key sync and its delay.
    ///
    /// Not its rate, which is how long a cycle takes and not what one looks
    /// like, and not its arp sync or mono mode, which decide where the rate
    /// comes from and how many copies of the LFO there are.
    const fn parameters(self) -> [ParamId; 4] {
        match self {
            Self::One => [
                ParamId::Lfo1Shape,
                ParamId::Lfo1SlewRate,
                ParamId::Lfo1KeySync,
                ParamId::Lfo1DelayFade,
            ],
            Self::Two => [
                ParamId::Lfo2Shape,
                ParamId::Lfo2SlewRate,
                ParamId::Lfo2KeySync,
                ParamId::Lfo2DelayFade,
            ],
        }
    }
}

/// Which of the two oscillators to read.
///
/// They are not the same instrument: OSC 1 has a sawtooth and a pulse that can
/// both be switched on, and OSC 2 is one wave at a level. See [`oscillator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OscillatorId {
    /// `OSC 1`.
    One,
    /// `OSC 2`.
    Two,
}

impl OscillatorId {
    /// Every oscillator, in the order the program stores them.
    pub const ALL: [Self; 2] = [Self::One, Self::Two];
}

/// How long a held level is drawn for, against the timed segments beside it.
///
/// A held note is as long as the player holds it, so there is no byte that says
/// how wide an envelope's sustain, or the full depth after an LFO's fade, should
/// be. This is the width it is given when the timed segments are at their
/// shortest, in the same units they are measured in, which puts a flat stretch
/// in the picture rather than an instant corner. It is a drawing convention and
/// the only one in this module.
const HOLD_WIDTH: f32 = 0.5;

/// Returns the shape one of the three envelopes makes.
///
/// The three times share the width in proportion to their own bytes, with a
/// fixed stretch for the sustain between the decay and the release, so an
/// envelope with a long attack is drawn with a long attack. The horizontal is
/// [`Scale::Normalised`]: the manual gives no mapping from a time byte to
/// seconds, and one invented here would be wrong in a way a picture could not
/// show. Where each segment begins is in [`Generator::marks`].
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

    let total = attack + decay + HOLD_WIDTH + release;
    // `total` is at least HOLD_WIDTH, so this never divides by zero.
    let share = |of: f32| of / total;
    let first = share(attack);
    let second = first + share(decay);
    let third = second + share(HOLD_WIDTH);

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

/// Returns the wave an LFO makes, swinging about the middle of its range.
///
/// The horizontal is [`Scale::Turns`] of exactly one: a cycle is a cycle
/// whatever the rate byte is doing, which makes this the one generator here
/// whose axis is fully published. What a host cannot label is how long the cycle
/// takes, and that is the rate parameter rather than this shape.
///
/// A program whose shape byte names no shape is drawn as a sine, which is the
/// byte the instrument ships at.
///
/// # What is read
///
/// The shape byte, and three of the four beside it:
///
/// - **Slew rate** rounds the corners off. The manual says it turns a square
///   into something that ramps between its two levels rather than jumping,
///   which is what a limit on how fast the level may change does, so that is
///   what is drawn: a rate limit, with none at all at the bottom of the byte and
///   at the top one that takes half a cycle to cross the range, which is the
///   limit that turns a square into a triangle. The family is the manual's; the
///   number is this library's reading, the same way the envelope bend is.
/// - **Key sync** decides whether phase 0 is a moment. It changes nothing in
///   the picture and everything about whether the left edge of it means
///   anything, which is what [`Generator::anchored`] reports.
/// - **Delay / fade** is not in this picture, because it is a shape of its own
///   with an axis of its own: how many cycles a fade lasts is two bytes whose
///   curves are both unpublished. [`lfo_fade`] draws it beside this.
///
/// Whether the LFO swings about the middle or rides up from the floor is not a
/// parameter of the LFO. It is decided by what takes it: `LFO 1` and `LFO 1
/// Unipolar` are two entries in the tables that choose a modulation source, and
/// [`lfo_unipolar`] is this same wave read the second way.
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
    lfo_read(program, which, false)
}

/// Returns the wave an LFO makes, read riding up from the floor.
///
/// The same wave as [`lfo`] with its [`rest`](Generator::rest) at the bottom
/// rather than the middle, which is how `LFO 1 Unipolar` in the oscillators'
/// pitch modulation tables and `LFO1 (Uni)` in the modulation matrix take it.
/// The instrument has no unipolar switch on the LFO itself; the reading belongs
/// to whatever the LFO is routed to, so it is asked for here rather than read
/// off the program.
///
/// ```
/// use deepmind_midi::generator::{self, LfoId};
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::Program;
///
/// let program = Program::new(ProtocolVersion::V6);
/// let bipolar = generator::lfo(&program, LfoId::One);
/// let unipolar = generator::lfo_unipolar(&program, LfoId::One);
///
/// // One picture, two baselines.
/// assert_eq!(bipolar.at(0.3), unipolar.at(0.3));
/// assert_eq!(bipolar.rest(), 0.5);
/// assert_eq!(unipolar.rest(), 0.0);
/// ```
#[must_use]
pub fn lfo_unipolar(program: &Program, which: LfoId) -> Generator {
    lfo_read(program, which, true)
}

/// Builds an LFO's generator from the program, read one way or the other.
fn lfo_read(program: &Program, which: LfoId, unipolar: bool) -> Generator {
    let [shape, slew, key_sync, _] = which.parameters().map(|parameter| program.get(parameter));
    let shape = LfoShape::from_raw(shape).unwrap_or(LfoShape::Sine);
    let turns = match shape {
        LfoShape::SampleAndHold | LfoShape::SampleAndGlide => SAMPLED_TURNS,
        _ => 1.0,
    };
    Generator {
        shape: Shape::Lfo(Lfo {
            shape,
            turns,
            slew: byte(slew),
            unipolar,
            synced: key_sync != 0,
        }),
        scale: Scale::Turns(turns),
    }
}

/// Returns the fade an LFO's depth arrives over.
///
/// `LFO n Delay / Fade` is how long the LFO takes to reach full depth after a
/// note starts, so that vibrato can arrive rather than being there from the
/// attack. That makes it an envelope of its own — the instrument lists `LFO1
/// (Fade)` as a modulation source beside `LFO1` — and this is its shape: a rise
/// from nothing to full, then held. The rise takes its share of the width from
/// its byte against the same fixed stretch an envelope's sustain is given, so a
/// fade of zero is a flat line at full depth and a long one is mostly rise.
///
/// The horizontal is [`Scale::Normalised`], for the reason [`envelope`] gives.
/// The rise is a straight line, because the manual gives the fade a time and
/// says nothing about its shape; that is this library's reading and the only
/// one in this function. Where the rise ends is a
/// [`Segment::Sustain`](MarkKind::Segment) mark.
///
/// ```
/// use deepmind_midi::generator::{self, LfoId};
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::Program;
///
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_lfo2_delay_fade(255);
///
/// let fade = generator::lfo_fade(&program, LfoId::Two);
/// assert!(fade.at(0.0) < 0.01);       // nothing yet
/// assert!(fade.at(1.0) > 0.99);       // and at full depth by the end
/// ```
#[must_use]
pub fn lfo_fade(program: &Program, which: LfoId) -> Generator {
    let [_, _, _, fade] = which.parameters();
    let fade = byte(program.get(fade));
    Generator {
        shape: Shape::Fade {
            end: fade / (fade + HOLD_WIDTH),
        },
        scale: Scale::Normalised,
    }
}

/// Octaves either side of the corner that a filter response covers.
///
/// Wide enough to show a 24 dB per octave slope reaching the floor and narrow
/// enough that the flat part is not the whole picture.
const FILTER_SPAN: f32 = 3.0;

/// Decibels above unity that the top of a filter response's vertical is.
///
/// Headroom for the resonant peak, which rises above the passband: without it a
/// filter with its resonance up would draw as a flat line with a notch in it.
pub const FILTER_CEILING_DB: f32 = 12.0;

/// Decibels below unity that the bottom of a filter response's vertical is.
///
/// Far enough down that a 24 dB per octave slope takes two octaves to reach it,
/// which is what puts the knee in the middle of the picture rather than at the
/// edge of it.
pub const FILTER_FLOOR_DB: f32 = -48.0;

/// Where unity gain sits on a filter response's vertical, as `0..=1`.
///
/// The vertical is linear in decibels, from [`FILTER_FLOOR_DB`] at the bottom to
/// [`FILTER_CEILING_DB`] at the top, so this follows from the two. Decibels are
/// the unit the instrument's own slopes are published in — 24 per octave with
/// four poles and 12 with two on the low-pass, 6 on the high-pass — so this is
/// the axis the response is actually a fact about, rather than a linear
/// magnitude that draws every filter as a cliff.
///
/// A host marking the level the filter passes at draws its line here, which is
/// also what [`Generator::rest`] answers for either filter.
pub const FILTER_UNITY: f32 = -FILTER_FLOOR_DB / (FILTER_CEILING_DB - FILTER_FLOOR_DB);

/// Returns the low-pass response of the filter the program describes.
///
/// The horizontal is [`Scale::Octaves`] centred on the filter's own corner, which
/// sits at `t = 0.5` and is a [`MarkKind::Corner`] mark. That is the honest
/// axis: the slope is published — 24 dB per octave with four poles, 12 with
/// two, from section 8.5.3 — so the response *about* the corner is a fact,
/// while the frequency the corner sits at is a byte whose curve the manual does
/// not give. A host that wants to move the picture as `VCF Frequency` moves
/// reads that byte itself; the byte's position in its range is the only honest
/// thing to move it by.
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
/// There is a 6 dB per octave high-pass beside the low-pass, and it is not put
/// on this axis: doing that needs the spacing between the two corners, and that
/// is two bytes whose curves are both unpublished. An octave axis about one
/// corner says nothing about where the other one is. [`high_pass_response`]
/// draws it on an axis of its own.
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
    filter(Filter {
        slope,
        resonance: byte(program.get(ParamId::VcfResonance)),
        span: FILTER_SPAN,
        pass: Pass::Low,
    })
}

/// Decibels per octave the high-pass filter rolls off at.
///
/// One pole, where the low-pass has two or four.
const HIGH_PASS_SLOPE: f32 = 6.0;

/// Returns the response of the high-pass filter the program describes.
///
/// The mirror of [`filter_response`], on the same axes so that the two plates
/// of one section read against each other: [`Scale::Octaves`] about its own
/// corner at `t = 0.5`, the same [`FILTER_FLOOR_DB`] to [`FILTER_CEILING_DB`]
/// vertical, unity at [`FILTER_UNITY`]. The slope is the high-pass's own 6 dB
/// per octave, one pole where the low-pass has two or four, and it falls to
/// the left, which is what a high-pass does. Where the corner sits is `VCF HighPass Frequency`,
/// which a host reads itself for the reason [`filter_response`] gives.
///
/// This is not the two filters on one axis. The refusal there stands: their
/// spacing is two unpublished curves. What this is is the one thing that can be
/// said about the high-pass on its own, which is the same one thing said about
/// the low-pass.
///
/// # The boost is not in this
///
/// `VCF Bass Boost` is a switch on the same plate of the front panel, and what
/// it lifts the low end by is not published. A shelf drawn under the corner at
/// a height this library chose would look exactly as confident as the corner
/// beside it, so the curve is the filter alone and the switch is the host's to
/// print.
///
/// ```
/// use deepmind_midi::generator;
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::Program;
///
/// let program = Program::new(ProtocolVersion::V6);
/// let response = generator::high_pass_response(&program);
///
/// // Flat at unity above the corner, and down below it.
/// assert!((response.at(0.8) - generator::FILTER_UNITY).abs() < 0.01);
/// assert!(response.at(0.2) < response.at(0.5));
/// ```
#[must_use]
pub fn high_pass_response(program: &Program) -> Generator {
    // Nothing in the program moves this curve: the corner's position along
    // the frequency axis is the host's to place, for the reason above, and
    // the slope is fixed. The program is taken so that every generator here
    // is called the same way.
    let _ = program;
    filter(Filter {
        slope: HIGH_PASS_SLOPE,
        resonance: 0.0,
        span: FILTER_SPAN,
        pass: Pass::High,
    })
}

/// Wraps a filter in a generator over its own octave span.
const fn filter(filter: Filter) -> Generator {
    Generator {
        shape: Shape::Filter(filter),
        scale: Scale::Octaves(filter.span * 2.0),
    }
}

/// How far pulse width modulation can move the falling edge of OSC 1's pulse,
/// as a fraction of the cycle.
///
/// The manual prints `OSC 1 PWM Depth` two ways: a width of 50.0% to 99.0% while
/// the source is `Manual`, and 0 to plus or minus 49% of modulation once it is
/// not. Both are the same 49% of the cycle beyond the square, so this is the
/// one number for both readings.
const PULSE_REACH: f32 = 0.49;

/// Returns what one oscillator is putting out, over one cycle.
///
/// The horizontal is [`Scale::Turns`] of one, the same honest axis [`lfo`] has:
/// a cycle is a cycle whatever the pitch is. The two oscillators are not
/// symmetrical, and the asymmetry is the instrument's rather than something a
/// host should be encoding:
///
/// - **OSC 1** has a sawtooth and a pulse, either, both or neither of which can
///   be on. The saw rises through the cycle and drops; the pulse is high until
///   it falls at the width `OSC 1 PWM Depth` sets — 50% to 99% of the cycle,
///   the ends the manual prints, with the byte placed between them — and that
///   fall is a [`MarkKind::Width`] mark. Once the PWM source is an LFO or an
///   envelope the width is being swept rather than set, so the pulse is drawn
///   square and the same byte says how far the edge swings either way, as two
///   [`MarkKind::Sweep`] marks. With both waves on they are added at equal
///   weight, which is this library's reading: the manual prints the two waves
///   and not the law they sum by. With neither on the oscillator is making
///   nothing and the picture is a flat line at [`rest`](Generator::rest).
/// - **OSC 2** is a square wave at a level. `OSC 2 Level` is the one loudness on
///   either oscillator that is a parameter, so the wave is drawn as tall as the
///   byte's position in its range. That is the fader's travel and not a gain:
///   the byte reads `Off` and then -48 dB to 0 dB, and a wave drawn 48 dB down
///   would be a wave drawn invisible. What `OSC 2 Tone Mod` does to the square
///   the manual does not print beyond its name, so it is not drawn.
///
/// The noise generator is mixed in beside the two, and [`noise`] is its picture.
///
/// ```
/// use deepmind_midi::generator::{self, MarkKind, OscillatorId};
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::program::{Program, PwmSource};
///
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_osc1_pulse_enable(true);
/// program.set_osc1_saw_enable(false);
/// program.set_osc1_pwm_source(PwmSource::Manual);
/// program.set_osc1_pwm_depth(0);       // 50%: a square
///
/// let square = generator::oscillator(&program, OscillatorId::One);
/// assert_eq!(square.at(0.25), 1.0);
/// assert_eq!(square.at(0.75), 0.0);
/// let width = square.marks().find(|mark| mark.kind() == MarkKind::Width);
/// assert_eq!(width.map(|mark| mark.at()), Some(0.5));
/// ```
#[must_use]
pub fn oscillator(program: &Program, which: OscillatorId) -> Generator {
    let oscillator = match which {
        OscillatorId::One => {
            let depth = byte(program.get(ParamId::Osc1PwmDepth)) * PULSE_REACH;
            // A byte the table does not name reads as the width being set by
            // hand, which is the byte the instrument ships at.
            let manual = matches!(program.osc1_pwm_source(), Some(PwmSource::Manual) | None);
            Oscillator {
                saw: program.osc1_saw_enable(),
                pulse: program.osc1_pulse_enable(),
                width: if manual { 0.5 + depth } else { 0.5 },
                sweep: if manual { 0.0 } else { depth },
                level: 1.0,
            }
        }
        OscillatorId::Two => Oscillator {
            saw: false,
            pulse: true,
            width: 0.5,
            sweep: 0.0,
            level: byte(program.get(ParamId::Osc2Level)),
        },
    };
    Generator {
        shape: Shape::Oscillator(oscillator),
        scale: Scale::Turns(1.0),
    }
}

/// Returns the noise generator, at its level.
///
/// A scatter about the middle of the range, as far from it as `Noise Level` is
/// through its range, so a level of zero is a flat line. The horizontal is
/// [`Scale::Normalised`], because noise has no cycle to count.
///
/// The values are this library's and not the instrument's, for the reason
/// [`lfo`] gives for `Sample & Hold`: a fixed function of `t`, the same on every
/// call, so that a picture does not crawl between frames and two hosts draw the
/// same patch the same way. Nothing about the instrument's noise is published
/// beyond that it is there and how loud, and nothing more is claimed.
///
/// The height is the byte's position in its range and not a gain, for the
/// reason [`oscillator`] gives for `OSC 2 Level`.
#[must_use]
pub fn noise(program: &Program) -> Generator {
    Generator {
        shape: Shape::Noise {
            level: byte(program.get(ParamId::NoiseLevel)),
        },
        scale: Scale::Normalised,
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

/// Steps per cycle the slew limit is walked at.
///
/// Fine enough that a limited edge is a line rather than a staircase at any
/// size a host draws, and coarse enough that sampling a whole picture is a few
/// thousand cheap steps.
const SLEW_STEPS: u8 = 96;

impl Lfo {
    /// Returns the level at `phase` cycles along, as `0..=1`.
    fn at(&self, phase: f32) -> f32 {
        if self.slew <= 0.0 {
            return shape_at(self.shape, phase);
        }

        // The most the level may move in one cycle: a full swing in half a
        // cycle at the top of the byte, and more the further down it is.
        let limit = 2.0 / self.slew;

        // Walked from a cycle before the picture starts, so that the settling
        // from wherever the raw shape begins is behind the left edge and what
        // is drawn is the steady state. Each step is placed from its count
        // rather than by adding to the last, so nothing accumulates, and the
        // last is cut short at `phase` with its limit cut to match.
        let start = -1.0;
        let step = 1.0 / f32::from(SLEW_STEPS);
        let mut level = shape_at(self.shape, start);
        let mut previous = start;
        for count in 1..=u16::MAX {
            let at = (start + f32::from(count) * step).min(phase);
            let most = limit * (at - previous);
            let target = shape_at(self.shape, at);
            level += (target - level).clamp(-most, most);
            if at >= phase {
                break;
            }
            previous = at;
        }
        level
    }
}

/// Returns one cycle of an LFO shape, as `0..=1`.
fn shape_at(shape: LfoShape, t: f32) -> f32 {
    match shape {
        // Starting at the bottom rather than the middle, so that every shape in
        // the table begins where the others do and a host can draw them side by
        // side without one appearing shifted.
        LfoShape::Sine => 0.5 - 0.5 * math::cos_turns(t),
        LfoShape::Triangle => {
            let t = wrap(t);
            if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 }
        }
        LfoShape::Square => {
            if wrap(t) < 0.5 {
                1.0
            } else {
                0.0
            }
        }
        LfoShape::RampUp => wrap(t),
        LfoShape::RampDown => 1.0 - wrap(t),
        LfoShape::SampleAndHold => sampled(t),
        LfoShape::SampleAndGlide => {
            // One cycle is one value, so the glide runs from the value the
            // cycle before landed on to the one this cycle lands on.
            let previous = sampled(t - 1.0);
            previous + (sampled(t) - previous) * math::fract(t)
        }
    }
}

/// Returns where `t` falls within its cycle, in `0..=1`.
///
/// The fraction of `t`, except that the end of a cycle is the end of one and
/// not the start of the next: a ramp sampled at exactly one turn is at its
/// top, so that a host drawing one cycle draws the whole of it.
fn wrap(t: f32) -> f32 {
    let within = math::fract(t);
    if t > 0.0 && within <= 0.0 {
        1.0
    } else {
        within
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

/// Returns a scatter value for `t`, in `0..=1`.
///
/// A fixed function of its argument and nothing else, so that it is the same
/// on every call. It is a sine driven far past one turn and its fraction taken
/// at a large multiple, which is the usual way of getting a number that looks
/// unchosen out of one that is not.
fn noise_at(t: f32) -> f32 {
    math::fract((math::sin_turns(t * 89.0) + 1.5) * 1_013.0)
}

impl Filter {
    /// Returns the response `t` of the way across, as `0..=1`.
    fn at(&self, t: f32) -> f32 {
        // `t` walks `span` octaves either side of the corner, which sits at
        // 0.5. A high-pass is the same curve with the octaves counted the
        // other way, so that the response falls to the left.
        let octaves = (t - 0.5) * 2.0 * self.span;
        let octaves = match self.pass {
            Pass::Low => octaves,
            Pass::High => -octaves,
        };

        // The magnitude of an n-pole filter is 1 / sqrt(1 + (f/fc)^2n), which
        // in decibels is -10 log10 of the squared term. `slope` is decibels
        // per octave, and a pole is 6 of them, so the exponent is the octaves
        // from the corner times the slope over 6.
        let squared = 2.0 * (self.slope / 6.0) * octaves;
        // Well past the corner the one is lost in the rounding and the
        // logarithm is the exponent itself, which is also where `exp2` would
        // overflow.
        let log = if squared > 30.0 {
            squared
        } else {
            math::log2(1.0 + math::exp2(squared))
        };
        // 10 log10(x) is 10 log2(x) log10(2), and log10(2) is 0.30103.
        let magnitude_db = -3.010_3 * log;

        // The resonant peak sits at the corner and narrows as it grows, which
        // is what a rising Q does. It is added rather than multiplied so that
        // a filter with no resonance is exactly the roll-off above, and it
        // rises into the headroom above unity rather than pushing against the
        // top of the box.
        let distance = octaves * (2.0 + self.resonance * 6.0);
        let peak_db = self.resonance * FILTER_CEILING_DB / (1.0 + distance * distance);

        let db = magnitude_db + peak_db;
        ((db - FILTER_FLOOR_DB) / (FILTER_CEILING_DB - FILTER_FLOOR_DB)).clamp(0.0, 1.0)
    }
}

impl Oscillator {
    /// Returns the level `t` of the way through the cycle, as `0..=1`.
    fn at(&self, t: f32) -> f32 {
        // Each wave in -1..=1 about zero, which is what a wave is; the sum is
        // scaled by how many are on, so two waves fill the same range as one.
        let mut sum = 0.0;
        let mut waves = 0_u8;
        if self.saw {
            sum += t * 2.0 - 1.0;
            waves += 1;
        }
        if self.pulse {
            sum += if t < self.width { 1.0 } else { -1.0 };
            waves += 1;
        }
        if waves == 0 {
            return 0.5;
        }
        0.5 + 0.5 * self.level * sum / f32::from(waves)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{
        EnvelopeId, FILTER_CEILING_DB, FILTER_FLOOR_DB, FILTER_SPAN, FILTER_UNITY, GATES_DRAWN,
        Generator, HIGH_PASS_SLOPE, LfoId, MAX_MARKS, MarkKind, OscillatorId, SAMPLED_TURNS, Scale,
        Segment, arpeggiator_gates, envelope, filter_response, high_pass_response, lfo, lfo_fade,
        lfo_unipolar, noise, oscillator,
    };
    use crate::ids::ProtocolVersion;
    use crate::program::{LfoShape, Program, PwmSource, VcfPoleMode};

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

    /// Every generator this module builds from one program, so that a test
    /// about all of them cannot leave one out.
    fn every(program: &Program) -> [Generator; 10] {
        [
            envelope(program, EnvelopeId::Vca),
            lfo(program, LfoId::One),
            lfo_unipolar(program, LfoId::Two),
            lfo_fade(program, LfoId::One),
            filter_response(program),
            high_pass_response(program),
            oscillator(program, OscillatorId::One),
            oscillator(program, OscillatorId::Two),
            noise(program),
            crate::generator::taps([(0.5, 1.0), (1.0, 0.5), (0.0, 0.0), (0.0, 0.0)], 2),
        ]
    }

    /// Nothing a host samples can leave the box it is drawing into, whatever it
    /// asks for, including the values outside the range and a NaN. Nor can a
    /// mark fall outside the horizontal.
    #[test]
    fn every_generator_stays_within_its_range() {
        let mut program = program();
        program.set_vca_envelope_attack_time(90);
        program.set_vca_envelope_decay_time(40);
        program.set_vca_envelope_sustain_level(180);
        program.set_vca_envelope_release_time(200);
        program.set_vcf_resonance(255);
        program.set_lfo1_slew_rate(200);
        program.set_lfo2_shape(LfoShape::SampleAndGlide);
        program.set_lfo2_slew_rate(255);
        program.set_osc1_saw_enable(true);
        program.set_osc1_pulse_enable(true);
        program.set_osc1_pwm_source(PwmSource::Lfo1);
        program.set_osc1_pwm_depth(255);
        program.set_osc2_level(255);
        program.set_noise_level(255);

        for generator in every(&program) {
            for value in walk(&generator, 200) {
                assert!((0.0..=1.0).contains(&value), "{value} is outside the box");
            }
            for odd in [-1.0, 1.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let value = generator.at(odd);
                assert!((0.0..=1.0).contains(&value), "at({odd}) gave {value}");
            }
            assert!((0.0..=1.0).contains(&generator.rest()));
            let mut marks = 0;
            for mark in generator.marks() {
                assert!((0.0..=1.0).contains(&mark.at()), "{mark:?} is off the axis");
                marks += 1;
            }
            assert!(marks <= MAX_MARKS);
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

    /// The marks are the segment split the shape was built from: one per
    /// segment, in order, and the shape is at the level a boundary implies.
    #[test]
    fn an_envelope_marks_where_each_segment_begins() {
        let mut program = program();
        program.set_mod_envelope_attack_time(255);
        program.set_mod_envelope_decay_time(255);
        program.set_mod_envelope_sustain_level(128);
        program.set_mod_envelope_release_time(255);
        let shape = envelope(&program, EnvelopeId::Mod);

        let mut marks = shape.marks();
        let mut previous = 0.0;
        for segment in Segment::ALL {
            let mark = marks.next().expect("four segments");
            assert_eq!(mark.kind(), MarkKind::Segment(segment));
            assert!(mark.at() >= previous, "{segment:?} is out of order");
            previous = mark.at();
            match segment {
                Segment::Attack => close(mark.at(), 0.0, "the attack begins at the start"),
                Segment::Decay => close(shape.at(mark.at()), 1.0, "the decay begins at the top"),
                Segment::Sustain => close(
                    shape.at(mark.at()),
                    128.0 / 255.0,
                    "the sustain begins at its level",
                ),
                Segment::Release => assert!(mark.at() < 1.0),
            }
        }
        assert!(marks.next().is_none(), "a fifth mark");
        close(shape.rest(), 0.0, "an envelope rests on the floor");
        assert!(shape.anchored());
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
    /// and still label each one correctly. The cycle marks agree with it.
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

            // One mark per cycle, each at the end of its cycle, so the last
            // is at the right edge.
            let mut cycles = 0.0;
            for mark in generator.marks() {
                assert_eq!(mark.kind(), MarkKind::Cycle);
                cycles += 1.0;
                close(mark.at(), cycles / expected, "a cycle boundary");
            }
            close(cycles, expected, "one mark per cycle");
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

    /// The slew is a rate limit: nothing at the bottom of the byte, and at the
    /// top a square's edge takes half the cycle to cross the range, which is
    /// what turns it into a triangle. Between the two the edge is steeper than
    /// at the top and shallower than with no limit.
    #[test]
    fn the_slew_rate_rounds_a_square_into_a_ramp() {
        let mut program = program();
        program.set_lfo1_shape(LfoShape::Square);

        program.set_lfo1_slew_rate(0);
        let sharp = lfo(&program, LfoId::One);
        close(sharp.at(0.51), 0.0, "an unslewed edge has fallen at once");

        program.set_lfo1_slew_rate(255);
        let full = lfo(&program, LfoId::One);
        // The fall starts at the half cycle and takes the other half, so
        // halfway through the fall the level is halfway down and the bottom
        // is reached at the end of the cycle.
        assert!((full.at(0.75) - 0.5).abs() < 0.03, "{}", full.at(0.75));
        assert!(full.at(1.0) < 0.03, "{}", full.at(1.0));
        // And the picture is the steady state: the rise at the start of the
        // cycle is the mirror of the fall, not a settling from nowhere.
        assert!((full.at(0.25) - 0.5).abs() < 0.03, "{}", full.at(0.25));
        assert!(full.at(0.5) > 0.97, "{}", full.at(0.5));

        // Halfway up the byte the edge takes a quarter of the cycle, so at
        // any point in the fall it is between the two.
        program.set_lfo1_slew_rate(128);
        let half = lfo(&program, LfoId::One);
        for t in [0.55, 0.625, 0.7] {
            assert!(half.at(t) > sharp.at(t), "at {t}");
            assert!(half.at(t) < full.at(t), "at {t}");
        }

        // The slew is a rate limit and not a change of shape: a sine, which
        // never moves faster than the limit at this byte, is left alone.
        program.set_lfo1_shape(LfoShape::Sine);
        program.set_lfo1_slew_rate(64);
        let slewed = lfo(&program, LfoId::One);
        program.set_lfo1_slew_rate(0);
        let plain = lfo(&program, LfoId::One);
        for (left, right) in walk(&slewed, 32).zip(walk(&plain, 32)) {
            assert!((left - right).abs() < 0.03, "{left} against {right}");
        }
    }

    /// Key sync is what makes phase 0 a moment; nothing about the picture
    /// changes with it.
    #[test]
    fn key_sync_anchors_the_picture_and_changes_nothing_in_it() {
        let mut program = program();
        program.set_lfo1_key_sync(false);
        let free = lfo(&program, LfoId::One);
        program.set_lfo1_key_sync(true);
        let synced = lfo(&program, LfoId::One);

        assert!(!free.anchored());
        assert!(synced.anchored());
        assert!(walk(&free, 32).eq(walk(&synced, 32)));

        // The other LFO's switch is its own.
        assert!(!lfo(&program, LfoId::Two).anchored());
    }

    /// The fade is a rise then a hold, sharing the width with the hold the way
    /// an envelope's segments do, and a fade of nothing is at full depth
    /// throughout.
    #[test]
    fn the_fade_rises_to_full_depth_and_holds() {
        let mut program = program();

        program.set_lfo1_delay_fade(0);
        let none = lfo_fade(&program, LfoId::One);
        for value in walk(&none, 16) {
            close(value, 1.0, "no fade is full depth");
        }

        program.set_lfo1_delay_fade(255);
        let long = lfo_fade(&program, LfoId::One);
        assert_eq!(long.scale(), Scale::Normalised);
        close(long.rest(), 0.0, "a fade rests on the floor");
        assert!(long.anchored());
        // A fade of 1.0 against a hold of 0.5: two thirds of the width is
        // the rise, and it is a straight line.
        let end = 1.0 / 1.5;
        close(long.at(end / 2.0), 0.5, "halfway up the rise");
        close(long.at(end), 1.0, "the top of the rise");
        close(long.at(1.0), 1.0, "held");

        let mut marks = long.marks();
        let attack = marks.next().expect("the rise");
        let hold = marks.next().expect("the hold");
        assert_eq!(attack.kind(), MarkKind::Segment(Segment::Attack));
        assert_eq!(hold.kind(), MarkKind::Segment(Segment::Sustain));
        close(hold.at(), end, "where the hold begins");
        assert!(marks.next().is_none());

        // The other LFO's fade is its own byte.
        program.set_lfo2_delay_fade(0);
        close(lfo_fade(&program, LfoId::Two).at(0.0), 1.0, "LFO 2 at full");
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
        close(steep.rest(), FILTER_UNITY, "a filter rests at unity");

        // The corner is the one mark on the picture.
        let mut marks = steep.marks();
        let corner = marks.next().expect("a corner");
        assert_eq!(corner.kind(), MarkKind::Corner);
        close(corner.at(), 0.5, "the corner");
        assert!(marks.next().is_none());
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

    /// The high-pass is the low-pass turned round: the same axes, a corner in
    /// the middle, unity on the other side and its own slope on the way down.
    /// Being one pole, it takes twice as many octaves as a two-pole filter to
    /// fall the same distance.
    #[test]
    fn the_high_pass_falls_to_the_left_at_six_decibels_an_octave() {
        let mut program = program();
        let high = high_pass_response(&program);
        program.set_vcf2_pole_mode(VcfPoleMode::TwoPole);
        program.set_vcf_resonance(0);
        let low = filter_response(&program);

        assert_eq!(high.scale(), low.scale());
        close(high.rest(), FILTER_UNITY, "a filter rests at unity");
        assert!(high.anchored());

        // Flat at unity above the corner, the corner 3 dB down, and falling
        // below it.
        assert!((high.at(0.9) - FILTER_UNITY).abs() < 0.01);
        close(high.at(0.5), low.at(0.5), "both corners are 3 dB down");
        assert!(high.at(0.3) < high.at(0.4) && high.at(0.4) < high.at(0.5));

        // Two and three octaves below the corner rather than one and two: a
        // single pole is still bending an octave from its corner, and only
        // settles to its slope further out.
        let window = FILTER_CEILING_DB - FILTER_FLOOR_DB;
        let one = high.at(0.5 - 2.0 / (FILTER_SPAN * 2.0));
        let two = high.at(0.5 - 3.0 / (FILTER_SPAN * 2.0));
        let fallen = (one - two) * window;
        assert!(
            (fallen - HIGH_PASS_SLOPE).abs() < 0.5,
            "fell {fallen} dB over the octave, not {HIGH_PASS_SLOPE}"
        );

        // The corner is marked, and the boost switch changes nothing drawn.
        let corner = high.marks().next().expect("a corner");
        assert_eq!(corner.kind(), MarkKind::Corner);
        program.set_vcf_bass_boost(true);
        assert!(walk(&high, 16).eq(walk(&high_pass_response(&program), 16)));
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

    /// OSC 1 is what is switched on: a saw, a pulse at the width its byte sets,
    /// the two added, or a flat line at rest.
    #[test]
    fn the_first_oscillator_is_what_is_switched_on() {
        let mut program = program();
        program.set_osc1_pwm_source(PwmSource::Manual);
        program.set_osc1_pwm_depth(0);

        program.set_osc1_saw_enable(false);
        program.set_osc1_pulse_enable(false);
        let silent = oscillator(&program, OscillatorId::One);
        assert_eq!(silent.scale(), Scale::Turns(1.0));
        close(silent.rest(), 0.5, "an oscillator rests in the middle");
        assert!(!silent.anchored());
        for value in walk(&silent, 16) {
            close(value, 0.5, "an oscillator making nothing");
        }
        // A cycle mark and nothing else: no pulse, so no width.
        assert!(silent.marks().all(|mark| mark.kind() == MarkKind::Cycle));

        program.set_osc1_saw_enable(true);
        let saw = oscillator(&program, OscillatorId::One);
        close(saw.at(0.0), 0.0, "a saw starts at the bottom");
        close(saw.at(0.5), 0.5, "and rises through the middle");
        close(saw.at(1.0), 1.0, "to the top");

        program.set_osc1_saw_enable(false);
        program.set_osc1_pulse_enable(true);
        program.set_osc1_pwm_depth(255);
        let wide = oscillator(&program, OscillatorId::One);
        // 99% of the cycle high, which is the top of the printed range.
        close(wide.at(0.98), 1.0, "still high near the end");
        close(wide.at(0.995), 0.0, "and down at the very end");
        let width = wide
            .marks()
            .find(|mark| mark.kind() == MarkKind::Width)
            .expect("a pulse has a width");
        close(width.at(), 0.99, "the falling edge");
        assert!(!wide.marks().any(|mark| mark.kind() == MarkKind::Sweep));

        program.set_osc1_saw_enable(true);
        program.set_osc1_pwm_depth(0);
        let both = oscillator(&program, OscillatorId::One);
        // The sum of a saw at its middle and a pulse at its top is three
        // quarters of the way up, and at its bottom a quarter.
        assert!((both.at(0.499) - 0.75).abs() < 0.01, "{}", both.at(0.499));
        close(both.at(0.5), 0.25, "saw and pulse, pulse low");
    }

    /// Once the width is being swept the pulse is drawn square and the depth
    /// byte becomes how far the edge reaches either way.
    #[test]
    fn a_modulated_pulse_is_square_with_its_reach_marked() {
        let mut program = program();
        program.set_osc1_saw_enable(false);
        program.set_osc1_pulse_enable(true);
        program.set_osc1_pwm_source(PwmSource::Lfo1);
        program.set_osc1_pwm_depth(255);

        let swept = oscillator(&program, OscillatorId::One);
        close(swept.at(0.25), 1.0, "square");
        close(swept.at(0.75), 0.0, "square");

        let mut sweeps = swept.marks().filter(|mark| mark.kind() == MarkKind::Sweep);
        close(
            sweeps.next().expect("one side").at(),
            0.01,
            "as far left as it goes",
        );
        close(
            sweeps.next().expect("the other").at(),
            0.99,
            "as far right as it goes",
        );
        assert!(sweeps.next().is_none());

        // No depth, no reach.
        program.set_osc1_pwm_depth(0);
        assert!(
            !oscillator(&program, OscillatorId::One)
                .marks()
                .any(|mark| mark.kind() == MarkKind::Sweep)
        );
    }

    /// OSC 2 is a square as tall as its level, and off is a flat line.
    #[test]
    fn the_second_oscillator_is_a_square_at_its_level() {
        let mut program = program();

        program.set_osc2_level(0);
        let off = oscillator(&program, OscillatorId::Two);
        for value in walk(&off, 16) {
            close(value, 0.5, "off");
        }

        program.set_osc2_level(255);
        let full = oscillator(&program, OscillatorId::Two);
        close(full.at(0.25), 1.0, "the top of a full square");
        close(full.at(0.75), 0.0, "the bottom");

        program.set_osc2_level(128);
        let half = oscillator(&program, OscillatorId::Two);
        let tall = half.at(0.25) - 0.5;
        assert!(tall > 0.2 && tall < 0.3, "{tall}");

        // The width is the square's, whatever OSC 1's pulse is doing.
        program.set_osc1_pwm_depth(255);
        let width = oscillator(&program, OscillatorId::Two)
            .marks()
            .find(|mark| mark.kind() == MarkKind::Width)
            .expect("a square has a width");
        close(width.at(), 0.5, "a square");
    }

    /// The noise scatters as far as its level, is the same every time it is
    /// asked, and is a flat line when there is none.
    #[test]
    fn the_noise_scatters_to_its_level_and_holds_still() {
        let mut program = program();

        program.set_noise_level(0);
        for value in walk(&noise(&program), 16) {
            close(value, 0.5, "no noise");
        }

        program.set_noise_level(255);
        let loud = noise(&program);
        assert_eq!(loud.scale(), Scale::Normalised);
        close(loud.rest(), 0.5, "the noise rests in the middle");
        assert!(!loud.anchored());
        assert_eq!(loud.marks().count(), 0);
        let high = walk(&loud, 64).fold(f32::MIN, f32::max);
        let low = walk(&loud, 64).fold(f32::MAX, f32::min);
        assert!(high > 0.8 && low < 0.2, "{low} to {high}");
        assert!(walk(&loud, 64).eq(walk(&noise(&program), 64)));

        program.set_noise_level(128);
        let quiet = noise(&program);
        let high = walk(&quiet, 64).fold(f32::MIN, f32::max);
        assert!(high < 0.8, "{high}");
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

    /// The three envelopes read three different sets of bytes, and so do the
    /// two LFOs. A copied match arm would show up here and nowhere else.
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
