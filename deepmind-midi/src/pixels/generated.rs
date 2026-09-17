//! The glyph catalogue, generated from `spec/glyphs.toml`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The type that reads
//! these, and everything that names one, is in the parent module.

use super::Pixels;

/// Number of glyphs in the catalogue.
pub const GLYPH_COUNT: usize = 54;

/// A picture of what a parameter does, the size of a character.
///
/// One per kind of thing a control does to a signal rather than one per
/// control: every `Low Cut` is [`Glyph::LowCut`], whether it is on a reverb or
/// a delay, and a host that has learned one has learned them all. Reached from
/// [`FxSlot::glyph`](crate::effect::FxSlot::glyph),
/// [`ParamId::glyph`](crate::param::ParamId::glyph) and
/// [`Controller::glyph`](crate::param::Controller::glyph); the drawing itself
/// is [`Glyph::pixels`].
///
/// Which glyph a parameter carries is this crate's reading of what the
/// parameter does, the same way [`Quantity`](crate::effect::Quantity) is, and
/// `spec/glyphs.toml` says how it was decided. The drawings are this crate's
/// own: no font, no raster and no licensed symbol set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Glyph {
    /// How loud: a wedge rising to the right, the way a fader's travel is printed.
    Level,
    /// Wet against dry: a disc half filled.
    Mix,
    /// When: a clock face with one hand.
    Time,
    /// A gap before the effect starts: the dry hit, empty space, then the tail arriving.
    PreDelay,
    /// A tail falling away: an exponential fall from full.
    Decay,
    /// A rise to full and held there: the onset of an envelope.
    Attack,
    /// Held, then let go: a level falling to the floor at the end.
    Release,
    /// A level held: a line kept up between the moment it starts and the moment it ends.
    Hold,
    /// How big: the corners of a box, with what is inside it marked.
    Size,
    /// The highs taken off: level, then falling to the right. A low-pass corner.
    HighCut,
    /// The lows taken off: rising from the left, then level. A high-pass corner.
    LowCut,
    /// The lows lifted: a shelf above the level at the left.
    LowShelf,
    /// The highs lifted: a shelf above the level at the right.
    HighShelf,
    /// A band lifted about a centre: a bump on a level.
    Bell,
    /// A peak: the response standing up at one frequency.
    Resonance,
    /// Where in the spectrum: a ruler whose ticks close up towards the right, the way octaves do.
    Frequency,
    /// Sent round again: a loop with an arrowhead on it.
    Feedback,
    /// How fast: a wave, a cycle and a half of it.
    Rate,
    /// The wave itself: one cycle of a sine.
    Wave,
    /// How far the wave swings: a cycle between the two rails it reaches.
    Depth,
    /// How wide: a span with a mark at each end.
    Width,
    /// Pushed apart: arrows pointing both ways from the middle.
    Spread,
    /// Where between left and right: a dot between the two ways it can go.
    Pan,
    /// A level things happen above: a line across, with one peak over it.
    Threshold,
    /// How hard a level is held down past the threshold: a slope with a knee in it.
    Ratio,
    /// Pushed hard: a bolt.
    Drive,
    /// Tilted: more of one end of the spectrum and less of the other, about a pivot.
    Tone,
    /// A pitch: a note.
    Pitch,
    /// Two pitches almost the same: two waves almost in step.
    Detune,
    /// Where in the cycle: a wave with its axis through it.
    Phase,
    /// One of a list: the list.
    Selection,
    /// In or out: the power ring.
    Switch,
    /// Scattered: dots with no line through them.
    Diffusion,
    /// Random: dots everywhere.
    Noise,
    /// In steps: a staircase.
    Steps,
    /// Repeats: the same mark three times, shorter each time.
    Tap,
    /// A pulse: a wave with two levels and nothing between them.
    Square,
    /// A saw: a ramp up and a drop.
    Saw,
    /// How long a note is held open: one pulse and its width.
    Gate,
    /// Long then short: two pulses that do not share a width.
    Swing,
    /// Bent away from straight: a line that bows.
    Curve,
    /// Slid from one pitch to the next rather than stepped.
    Glide,
    /// Which way up: plus over minus.
    Polarity,
    /// The keyboard: keys, with the black ones between them.
    Keys,
    /// An envelope: up fast, down to a level, held, and let go.
    Envelope,
    /// How hard the key was hit: an arrow with motion beside it.
    Velocity,
    /// Pressed after the key is down: an arrow pressing on a key.
    Pressure,
    /// The pitch bender: a wheel seen from the side, resting in the middle.
    Bend,
    /// The modulation wheel: a wheel seen from the side, resting at the bottom.
    ModWheel,
    /// A pedal under a foot: a treadle over its base, tipped by the toe.
    Pedal,
    /// An expression pedal: the same treadle, carrying a level.
    Expression,
    /// A footswitch: a box on the floor with a button on top.
    Footswitch,
    /// Breath down a tube: a flow between two walls.
    Breath,
    /// A speaker cabinet: a box with a driver in it.
    Cabinet,
}

impl Glyph {
    /// Every glyph, in the order the catalogue draws them.
    pub const ALL: [Self; GLYPH_COUNT] = [
        Self::Level,
        Self::Mix,
        Self::Time,
        Self::PreDelay,
        Self::Decay,
        Self::Attack,
        Self::Release,
        Self::Hold,
        Self::Size,
        Self::HighCut,
        Self::LowCut,
        Self::LowShelf,
        Self::HighShelf,
        Self::Bell,
        Self::Resonance,
        Self::Frequency,
        Self::Feedback,
        Self::Rate,
        Self::Wave,
        Self::Depth,
        Self::Width,
        Self::Spread,
        Self::Pan,
        Self::Threshold,
        Self::Ratio,
        Self::Drive,
        Self::Tone,
        Self::Pitch,
        Self::Detune,
        Self::Phase,
        Self::Selection,
        Self::Switch,
        Self::Diffusion,
        Self::Noise,
        Self::Steps,
        Self::Tap,
        Self::Square,
        Self::Saw,
        Self::Gate,
        Self::Swing,
        Self::Curve,
        Self::Glide,
        Self::Polarity,
        Self::Keys,
        Self::Envelope,
        Self::Velocity,
        Self::Pressure,
        Self::Bend,
        Self::ModWheel,
        Self::Pedal,
        Self::Expression,
        Self::Footswitch,
        Self::Breath,
        Self::Cabinet,
    ];

    /// Returns where this glyph sits in [`Glyph::ALL`], which is where its
    /// drawing is.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Level => 0,
            Self::Mix => 1,
            Self::Time => 2,
            Self::PreDelay => 3,
            Self::Decay => 4,
            Self::Attack => 5,
            Self::Release => 6,
            Self::Hold => 7,
            Self::Size => 8,
            Self::HighCut => 9,
            Self::LowCut => 10,
            Self::LowShelf => 11,
            Self::HighShelf => 12,
            Self::Bell => 13,
            Self::Resonance => 14,
            Self::Frequency => 15,
            Self::Feedback => 16,
            Self::Rate => 17,
            Self::Wave => 18,
            Self::Depth => 19,
            Self::Width => 20,
            Self::Spread => 21,
            Self::Pan => 22,
            Self::Threshold => 23,
            Self::Ratio => 24,
            Self::Drive => 25,
            Self::Tone => 26,
            Self::Pitch => 27,
            Self::Detune => 28,
            Self::Phase => 29,
            Self::Selection => 30,
            Self::Switch => 31,
            Self::Diffusion => 32,
            Self::Noise => 33,
            Self::Steps => 34,
            Self::Tap => 35,
            Self::Square => 36,
            Self::Saw => 37,
            Self::Gate => 38,
            Self::Swing => 39,
            Self::Curve => 40,
            Self::Glide => 41,
            Self::Polarity => 42,
            Self::Keys => 43,
            Self::Envelope => 44,
            Self::Velocity => 45,
            Self::Pressure => 46,
            Self::Bend => 47,
            Self::ModWheel => 48,
            Self::Pedal => 49,
            Self::Expression => 50,
            Self::Footswitch => 51,
            Self::Breath => 52,
            Self::Cabinet => 53,
        }
    }

    /// Returns the glyph's name, as `spec/glyphs.toml` writes it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Level => "level",
            Self::Mix => "mix",
            Self::Time => "time",
            Self::PreDelay => "pre delay",
            Self::Decay => "decay",
            Self::Attack => "attack",
            Self::Release => "release",
            Self::Hold => "hold",
            Self::Size => "size",
            Self::HighCut => "high cut",
            Self::LowCut => "low cut",
            Self::LowShelf => "low shelf",
            Self::HighShelf => "high shelf",
            Self::Bell => "bell",
            Self::Resonance => "resonance",
            Self::Frequency => "frequency",
            Self::Feedback => "feedback",
            Self::Rate => "rate",
            Self::Wave => "wave",
            Self::Depth => "depth",
            Self::Width => "width",
            Self::Spread => "spread",
            Self::Pan => "pan",
            Self::Threshold => "threshold",
            Self::Ratio => "ratio",
            Self::Drive => "drive",
            Self::Tone => "tone",
            Self::Pitch => "pitch",
            Self::Detune => "detune",
            Self::Phase => "phase",
            Self::Selection => "selection",
            Self::Switch => "switch",
            Self::Diffusion => "diffusion",
            Self::Noise => "noise",
            Self::Steps => "steps",
            Self::Tap => "tap",
            Self::Square => "square",
            Self::Saw => "saw",
            Self::Gate => "gate",
            Self::Swing => "swing",
            Self::Curve => "curve",
            Self::Glide => "glide",
            Self::Polarity => "polarity",
            Self::Keys => "keys",
            Self::Envelope => "envelope",
            Self::Velocity => "velocity",
            Self::Pressure => "pressure",
            Self::Bend => "bend",
            Self::ModWheel => "mod wheel",
            Self::Pedal => "pedal",
            Self::Expression => "expression",
            Self::Footswitch => "footswitch",
            Self::Breath => "breath",
            Self::Cabinet => "cabinet",
        }
    }
}

/// The drawing of each glyph, in the order `Glyph::ALL` gives them.
pub(super) static GLYPHS: [Pixels; GLYPH_COUNT] = [
    Pixels::new([
        0b000_0000, 0b100_0000, 0b110_0000, 0b111_0000, 0b111_1000, 0b111_1100, 0b111_1111,
    ]),
    Pixels::new([
        0b001_1100, 0b011_0010, 0b111_0001, 0b111_0001, 0b111_0001, 0b011_0010, 0b001_1100,
    ]),
    Pixels::new([
        0b001_1100, 0b010_0010, 0b100_1001, 0b101_1001, 0b100_0001, 0b010_0010, 0b001_1100,
    ]),
    Pixels::new([
        0b000_0001, 0b000_0001, 0b011_0001, 0b111_0001, 0b111_1001, 0b111_1001, 0b111_1111,
    ]),
    Pixels::new([
        0b000_0001, 0b000_0001, 0b000_0010, 0b000_0010, 0b000_0100, 0b001_1000, 0b110_0000,
    ]),
    Pixels::new([
        0b111_1000, 0b000_1000, 0b000_0100, 0b000_0100, 0b000_0010, 0b000_0010, 0b000_0001,
    ]),
    Pixels::new([
        0b000_1111, 0b001_0000, 0b001_0000, 0b010_0000, 0b010_0000, 0b100_0000, 0b100_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b111_1111, 0b000_0000, 0b000_0000, 0b000_0000, 0b100_0001, 0b100_0001,
    ]),
    Pixels::new([
        0b110_0011, 0b100_0001, 0b000_0000, 0b000_1000, 0b000_0000, 0b100_0001, 0b110_0011,
    ]),
    Pixels::new([
        0b000_0000, 0b000_1111, 0b001_0000, 0b010_0000, 0b100_0000, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b111_1000, 0b000_0100, 0b000_0010, 0b000_0001, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0111, 0b000_1000, 0b111_0000, 0b000_0000, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b111_0000, 0b000_1000, 0b000_0111, 0b000_0000, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b000_1000, 0b001_0100, 0b110_0011, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_1000, 0b000_1000, 0b001_0100, 0b001_0100, 0b010_0010, 0b110_0011,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b101_0001, 0b101_0001, 0b111_1111, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_1100, 0b001_1010, 0b100_1001, 0b100_0001, 0b100_0001, 0b010_0010, 0b001_1100,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b010_0010, 0b101_0101, 0b000_1000, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0110, 0b000_1001, 0b100_1001, 0b100_1000, 0b011_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b111_1111, 0b000_0000, 0b000_0110, 0b100_1001, 0b011_0000, 0b000_0000, 0b111_1111,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b100_0001, 0b111_1111, 0b100_0001, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b010_0010, 0b111_1111, 0b010_0010, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b010_0010, 0b110_1011, 0b010_0010, 0b000_0000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_1000, 0b111_1111, 0b010_1010, 0b010_1010, 0b010_1010, 0b010_1010,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0000, 0b111_0000, 0b000_1000, 0b000_0100, 0b000_0010, 0b000_0001,
    ]),
    Pixels::new([
        0b001_0000, 0b001_1000, 0b000_1100, 0b001_1110, 0b000_1000, 0b000_0100, 0b000_0010,
    ]),
    Pixels::new([
        0b000_0000, 0b110_0000, 0b001_1000, 0b000_0110, 0b000_1001, 0b001_1100, 0b000_0000,
    ]),
    Pixels::new([
        0b001_0000, 0b011_0000, 0b101_0000, 0b001_0000, 0b001_0000, 0b001_1110, 0b001_1110,
    ]),
    Pixels::new([
        0b000_0000, 0b010_0110, 0b001_1001, 0b000_0000, 0b010_0110, 0b001_1001, 0b000_0000,
    ]),
    Pixels::new([
        0b000_1000, 0b001_1100, 0b010_1010, 0b010_1010, 0b010_1010, 0b001_1100, 0b000_1000,
    ]),
    Pixels::new([
        0b000_0000, 0b111_1101, 0b000_0000, 0b111_1101, 0b000_0000, 0b111_1101, 0b000_0000,
    ]),
    Pixels::new([
        0b000_1000, 0b010_1010, 0b100_1001, 0b100_1001, 0b100_0001, 0b010_0010, 0b001_1100,
    ]),
    Pixels::new([
        0b000_0000, 0b001_0010, 0b100_1000, 0b010_0010, 0b001_0100, 0b001_0001, 0b000_0000,
    ]),
    Pixels::new([
        0b101_0010, 0b000_1001, 0b010_0100, 0b001_0001, 0b100_1010, 0b010_0100, 0b010_0001,
    ]),
    Pixels::new([
        0b000_0000, 0b111_0000, 0b001_0000, 0b001_1100, 0b000_0100, 0b000_0111, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b000_0001, 0b000_0001, 0b000_1001, 0b000_1001, 0b100_1001, 0b100_1001,
    ]),
    Pixels::new([
        0b000_0000, 0b000_1110, 0b000_1010, 0b000_1010, 0b000_1010, 0b111_1011, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b010_0000, 0b011_0000, 0b010_1000, 0b010_0100, 0b010_0010, 0b010_0001,
    ]),
    Pixels::new([
        0b000_0000, 0b001_1100, 0b001_0100, 0b001_0100, 0b001_0100, 0b111_0111, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b101_1110, 0b101_0010, 0b101_0010, 0b101_0010, 0b111_0011, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b100_0000, 0b010_0000, 0b001_0000, 0b000_1100, 0b000_0011, 0b000_0000,
    ]),
    Pixels::new([
        0b000_0000, 0b111_0000, 0b000_1000, 0b000_1000, 0b000_0100, 0b000_0111, 0b000_0000,
    ]),
    Pixels::new([
        0b000_1000, 0b000_1000, 0b011_1110, 0b000_1000, 0b000_1000, 0b000_0000, 0b011_1110,
    ]),
    Pixels::new([
        0b111_1111, 0b101_1011, 0b101_1011, 0b100_0001, 0b101_0101, 0b101_0101, 0b111_1111,
    ]),
    Pixels::new([
        0b000_0100, 0b000_0100, 0b000_1010, 0b011_1010, 0b010_0010, 0b100_0001, 0b100_0001,
    ]),
    Pixels::new([
        0b000_1000, 0b010_1010, 0b010_1010, 0b011_1110, 0b001_1100, 0b000_1000, 0b000_0000,
    ]),
    Pixels::new([
        0b000_1000, 0b000_1000, 0b011_1110, 0b001_1100, 0b000_1000, 0b000_0000, 0b111_1111,
    ]),
    Pixels::new([
        0b001_1100, 0b010_0010, 0b010_0010, 0b011_1110, 0b010_0010, 0b010_0010, 0b001_1100,
    ]),
    Pixels::new([
        0b001_1100, 0b010_0010, 0b010_0010, 0b010_0010, 0b011_1110, 0b011_1110, 0b001_1100,
    ]),
    Pixels::new([
        0b000_0000, 0b010_0000, 0b001_1000, 0b000_0110, 0b000_0001, 0b000_0000, 0b111_1111,
    ]),
    Pixels::new([
        0b000_0000, 0b010_0000, 0b011_1000, 0b011_1110, 0b111_1111, 0b000_0000, 0b111_1111,
    ]),
    Pixels::new([
        0b001_1100, 0b001_0100, 0b111_1111, 0b100_0001, 0b100_0001, 0b100_0001, 0b111_1111,
    ]),
    Pixels::new([
        0b000_0000, 0b111_1111, 0b001_0000, 0b011_1110, 0b001_0000, 0b111_1111, 0b000_0000,
    ]),
    Pixels::new([
        0b111_1111, 0b101_1101, 0b110_0011, 0b110_1011, 0b110_0011, 0b101_1101, 0b111_1111,
    ]),
];
