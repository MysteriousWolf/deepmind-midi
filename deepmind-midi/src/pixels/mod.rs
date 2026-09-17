//! A square one-bit grid, for a display too small to stroke anything.
//!
//! Three things in this crate are drawn this way: the mark of an effect,
//! through [`Mark::pixels`](crate::effect::Mark::pixels); the cell of a
//! modulation source, through
//! [`ValueTable::cell_of`](crate::param::ValueTable::cell_of); and a
//! [`Glyph`], the picture of what a parameter does, through
//! [`FxSlot::glyph`](crate::effect::FxSlot::glyph),
//! [`ParamId::glyph`](crate::param::ParamId::glyph) and
//! [`Controller::glyph`](crate::param::Controller::glyph). All are [`SIDE`]
//! dots by [`SIDE`], which is what the instrument's own 128 by 64 display has
//! room for beside a name, and a host that wants one bigger draws each dot as
//! more than one dot rather than resampling anything.
//!
//! They are drawn by hand rather than reduced from geometry. At forty-nine dots
//! which of them are lit is the whole of the design, and a one-pixel stroke put
//! through a rasteriser at this size comes out as a smear with the idea gone.
//! Seven is odd, so every drawing has a true centre dot to hang symmetry on.
//!
//! # A glyph beside a value
//!
//! A display with a row of characters per parameter has room for a name and a
//! number, and a controller with a small screen beside each knob has room for
//! less. A glyph is what goes in that room: a picture of what turning the
//! control does, the same one wherever the same kind of control is met, so
//! that a decay reads as a tail and a mix as wet against dry before the name
//! is. Which glyph a parameter carries is a reading of what it does, made once
//! in `spec/glyphs.toml`, so that every host draws the same picture on the
//! same control.
//!
//! ```
//! use deepmind_midi::effect::Algorithm;
//! use deepmind_midi::param::ParamId;
//! use deepmind_midi::pixels::Glyph;
//!
//! // A room reverb's decay and the filter envelope's decay are one picture.
//! let room = Algorithm::by_name("RoomRev").expect("a Room Reverb");
//! let decay = room.slot(2).expect("a Decay slot").glyph();
//! assert_eq!(decay, Glyph::Decay);
//! assert_eq!(ParamId::VcfEnvelopeDecayTime.glyph(), Some(Glyph::Decay));
//!
//! // And the picture is a grid to blit, like a mark or a cell.
//! assert!(decay.pixels().is_lit(0, 0));
//! ```

mod generated;

pub use generated::{GLYPH_COUNT, Glyph};

use generated::GLYPHS;

/// Dots across and down a [`Pixels`] grid.
pub const SIDE: usize = 7;

/// A drawing on a [`SIDE`] by [`SIDE`] one-bit grid, with the origin top left.
///
/// ```
/// use deepmind_midi::effect::Algorithm;
/// use deepmind_midi::pixels::SIDE;
///
/// let filter = Algorithm::by_name("MoodFilter").expect("a Mood Filter");
/// let pixels = filter.mark().pixels();
///
/// // Blitting it is a walk over the grid.
/// let mut lit = 0;
/// for y in 0..SIDE as u8 {
///     for x in 0..SIDE as u8 {
///         if pixels.is_lit(x, y) {
///             lit += 1;
///         }
///     }
/// }
/// assert!(lit > 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pixels {
    rows: [u8; SIDE],
}

impl Pixels {
    /// Builds a grid from its rows, a bit a pixel and bit 0 leftmost.
    #[must_use]
    pub const fn new(rows: [u8; SIDE]) -> Self {
        Self { rows }
    }

    /// Returns whether the pixel at `x`, `y` is lit, counting from the top
    /// left.
    ///
    /// `false` outside the grid, so a host walking a larger box than the
    /// drawing gets blank rather than an answer it has to bounds-check itself.
    #[must_use]
    pub fn is_lit(&self, x: u8, y: u8) -> bool {
        if usize::from(x) >= SIDE {
            return false;
        }
        self.row(y) & (1 << x) != 0
    }

    /// Returns one row as its low [`SIDE`] bits, bit 0 leftmost.
    ///
    /// What a host blitting a row at a time wants. Zero past the bottom of the
    /// grid.
    #[must_use]
    pub fn row(&self, y: u8) -> u8 {
        self.rows.get(usize::from(y)).copied().unwrap_or(0)
    }

    /// Returns every row, top to bottom.
    #[must_use]
    pub const fn rows(&self) -> &[u8; SIDE] {
        &self.rows
    }
}

impl Glyph {
    /// Returns the glyph's drawing.
    ///
    /// ```
    /// use deepmind_midi::pixels::{Glyph, SIDE};
    ///
    /// // A level is a wedge, so its bottom row is full and its top is not.
    /// let level = Glyph::Level.pixels();
    /// assert_eq!(level.row(SIDE as u8 - 1), 0b111_1111);
    /// assert!(!level.is_lit(0, 0));
    /// ```
    #[must_use]
    pub fn pixels(self) -> &'static Pixels {
        // Unreachable fallback: `ALL` and the generated table are declared the
        // same length and `index` is the position in `ALL`.
        GLYPHS.get(self.index()).unwrap_or(&GLYPHS[0])
    }
}

impl core::fmt::Display for Glyph {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::{GLYPH_COUNT, Glyph, Pixels, SIDE};

    /// Every glyph is drawn, draws something, and is its own picture.
    #[test]
    fn every_glyph_is_a_drawing_of_its_own() {
        assert_eq!(Glyph::ALL.len(), GLYPH_COUNT);
        for (index, glyph) in Glyph::ALL.iter().enumerate() {
            assert_eq!(glyph.index(), index, "{glyph}");
            let pixels = glyph.pixels();
            assert!(
                pixels.rows().iter().any(|row| *row != 0),
                "{glyph} is blank"
            );
            assert!(
                pixels.rows().iter().any(|row| *row != 0x7f),
                "{glyph} is solid"
            );
            for other in Glyph::ALL.iter().skip(index + 1) {
                assert_ne!(
                    pixels,
                    other.pixels(),
                    "{glyph} and {other} are one drawing"
                );
            }
        }
    }

    /// A bit a pixel, bit 0 leftmost, and nothing outside the grid.
    #[test]
    fn a_pixel_is_a_bit_of_its_row() {
        let mut rows = [0; SIDE];
        rows[0] = 0b000_0001;
        rows[6] = 0b100_0000;
        let pixels = Pixels::new(rows);

        assert!(pixels.is_lit(0, 0));
        assert!(!pixels.is_lit(1, 0));
        assert!(pixels.is_lit(6, 6));
        assert!(!pixels.is_lit(7, 6));
        assert!(!pixels.is_lit(0, 7));
        assert_eq!(pixels.row(6), 0b100_0000);
        assert_eq!(pixels.row(7), 0);
        assert_eq!(pixels.rows(), &rows);
    }
}
