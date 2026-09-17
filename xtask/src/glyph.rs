//! Generates the library's glyph catalogue from `spec/glyphs.toml`.
//!
//! A glyph is a picture of what a parameter does, on the crate's one pixel
//! grid. The catalogue is an enum with a drawing per variant; which glyph an
//! effect slot, a program parameter or a controller carries is rendered beside
//! those tables by the generators that own them.
//!
//! Only data is generated. `Pixels` and everything that reads a glyph are
//! hand-written in `deepmind-midi/src/pixels/mod.rs`.

use std::fmt::Write as _;

use crate::codegen::{Identifiers, doc};
use crate::spec::Spec;

/// Path of the generated glyph catalogue, relative to the repository root.
pub const CODE_PATH: &str = "deepmind-midi/src/pixels/generated.rs";

const HEADER: &str = "\
//! The glyph catalogue, generated from `spec/glyphs.toml`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. The type that reads
//! these, and everything that names one, is in the parent module.

use super::Pixels;

";

/// Renders the whole file, unformatted.
///
/// # Errors
///
/// Returns a message when a glyph's name makes no Rust identifier, which
/// [`Identifiers::new`] has already checked.
pub fn render(spec: &Spec, idents: &Identifiers) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(HEADER);
    let _ = writeln!(
        out,
        "/// Number of glyphs in the catalogue.\npub const GLYPH_COUNT: usize = {};\n",
        spec.glyphs.len()
    );

    out.push_str(
        "\
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
#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Glyph {
",
    );
    for glyph in &spec.glyphs {
        let ident = ident(idents, &glyph.name)?;
        let _ = writeln!(out, "    /// {}\n    {ident},", doc(&glyph.description));
    }
    out.push_str(
        "\
}

impl Glyph {
    /// Every glyph, in the order the catalogue draws them.
    pub const ALL: [Self; GLYPH_COUNT] = [
",
    );
    for glyph in &spec.glyphs {
        let _ = writeln!(out, "        Self::{},", ident(idents, &glyph.name)?);
    }
    out.push_str(
        "\
    ];

    /// Returns where this glyph sits in [`Glyph::ALL`], which is where its
    /// drawing is.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
",
    );
    for (index, glyph) in spec.glyphs.iter().enumerate() {
        let _ = writeln!(
            out,
            "            Self::{} => {index},",
            ident(idents, &glyph.name)?
        );
    }
    out.push_str(
        "\
        }
    }

    /// Returns the glyph's name, as `spec/glyphs.toml` writes it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
",
    );
    for glyph in &spec.glyphs {
        let _ = writeln!(
            out,
            "            Self::{} => {:?},",
            ident(idents, &glyph.name)?,
            glyph.name
        );
    }
    out.push_str(
        "\
        }
    }
}

/// The drawing of each glyph, in the order `Glyph::ALL` gives them.
pub(super) static GLYPHS: [Pixels; GLYPH_COUNT] = [
",
    );
    for glyph in &spec.glyphs {
        let _ = writeln!(out, "    {},", crate::codegen::pixels(&glyph.pixels));
    }
    out.push_str("];\n");
    Ok(out)
}

/// Returns the variant a glyph's name became.
///
/// # Errors
///
/// Returns a message for a name the identifier table does not hold, which the
/// spec loader's checks make unreachable.
pub fn ident<'a>(idents: &'a Identifiers, name: &str) -> Result<&'a str, String> {
    idents
        .glyphs
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("glyphs.toml: {name:?} makes no identifier"))
}
