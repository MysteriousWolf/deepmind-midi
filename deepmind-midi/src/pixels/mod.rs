//! A square one-bit grid, for a display too small to stroke anything.
//!
//! Two things in this crate are drawn this way: the mark of an effect family,
//! through [`Mark::pixels`](crate::effect::Mark::pixels), and the cell of a
//! modulation source, through [`ValueTable::cell_of`](crate::param::ValueTable::cell_of).
//! Both are [`SIDE`] dots by [`SIDE`], which is what the instrument's own
//! 128 by 64 display has room for beside a name, and a host that wants one
//! bigger draws each dot as more than one dot rather than resampling anything.
//!
//! They are drawn by hand rather than reduced from geometry. At forty-nine dots
//! which of them are lit is the whole of the design, and a one-pixel stroke put
//! through a rasteriser at this size comes out as a smear with the idea gone.
//! Seven is odd, so every drawing has a true centre dot to hang symmetry on.

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

#[cfg(test)]
mod tests {
    use super::{Pixels, SIDE};

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
