//! Typed view of the machine-readable specification in `spec/`.
//!
//! These files are the single source of truth for the protocol. Documentation is
//! generated from them, and the parameter tables the library will use are
//! generated from the same data, so a correction only ever has to be made once.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::codegen::pascal;

/// A program parameter.
///
/// `offset` is both the parameter's 14-bit NRPN number and its byte offset in an
/// unpacked program dump; on this synthesizer those are the same number.
#[derive(Debug, Deserialize)]
pub struct Parameter {
    /// NRPN number, and byte offset into unpacked program data.
    pub offset: u16,
    /// Human-readable parameter name.
    pub name: String,
    /// Section of the synthesizer this parameter belongs to.
    pub group: String,
    /// Lowest accepted value.
    pub min: u16,
    /// Highest accepted value.
    pub max: u16,
    /// `"switch"` for two-state parameters, absent otherwise.
    #[serde(default)]
    pub kind: Option<String>,
    /// `"bipolar"` where the manual states the raw value that reads as zero,
    /// absent where the value counts up from `min` like every other parameter.
    #[serde(default)]
    pub shape: Option<String>,
    /// The raw value that reads as zero, for a bipolar parameter.
    #[serde(default)]
    pub centre: Option<u16>,
    /// A value that means "not set" rather than a position in the range.
    ///
    /// `0` on a sequencer step, where zero is "skip this step" and not the
    /// smallest modulation the step can apply.
    #[serde(default)]
    pub inactive: Option<u16>,
    /// The parameter saying how many of a run are used, by its name here.
    ///
    /// `Sequence Length` on each of the 32 steps, so that a host drawing the
    /// run can dim what is not played rather than implying all of it is.
    #[serde(default)]
    pub bounded_by: Option<String>,
    /// Identifier of the value table in `enums.toml` that decodes this parameter.
    #[serde(default, rename = "enum")]
    pub value_table: Option<String>,
    /// The glyph in `glyphs.toml` that pictures what the parameter does, where
    /// one fits.
    #[serde(default)]
    pub glyph: Option<String>,
    /// Free-form note carried through from the manual.
    #[serde(default)]
    pub note: Option<String>,
    /// One sentence saying what the parameter does, for a host with room to
    /// print it.
    ///
    /// Written here rather than transcribed: see the `descriptions` key in this
    /// file's `[meta]` for where these come from and what they are not. Absent
    /// where nothing can be said without guessing.
    #[serde(default)]
    pub description: Option<String>,
    /// The physical range the synthesizer shows for the same raw value, where the
    /// manual states one. Hz, dB, seconds and so on.
    #[serde(default)]
    pub display: Option<String>,
    /// Why this row departs from what the manual prints.
    #[serde(default)]
    pub correction: Option<String>,
    /// `false` when the value is inferred and needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
}

/// One value of an enumerated parameter.
#[derive(Debug, Deserialize)]
pub struct EnumEntry {
    /// The wire value.
    pub value: u16,
    /// Name as the synthesizer displays it.
    pub name: String,
    /// Longer explanation, where the manual gives one.
    #[serde(default)]
    pub description: Option<String>,
    /// Program parameters this value names, by their `parameters.toml` names.
    ///
    /// The modulation matrix is what needs it: a destination is an
    /// abbreviation the display prints, and this is what it moves. Empty is
    /// both "nothing written yet" and "nothing a program parameter
    /// addresses", and the two are told apart by the entry's description.
    #[serde(default)]
    pub parameters: Vec<String>,
    /// Which way this value, as a modulation source, moves what it reaches:
    /// one of [`SWINGS`]. Absent where the specification does not settle it.
    #[serde(default)]
    pub swing: Option<String>,
}

/// The swings a modulation source may declare.
///
/// `centred` swings either side of where the destination sits; `rising` moves
/// it one way from there and back. The reasons are in the table's own note.
pub const SWINGS: [&str; 2] = ["centred", "rising"];

/// A dot-matrix cell for one entry of one value table.
///
/// The modulation sources are what has them: a picture the size of a character,
/// for a patch bay drawn in pictures rather than in abbreviations.
#[derive(Debug, Deserialize)]
pub struct Cell {
    /// The value table, by its `enums.toml` id.
    pub table: String,
    /// The entry, by the name it carries; see `meta.addressing` in cells.toml.
    pub name: String,
    /// The cell, one string per row, `#` for a lit dot and `.` for an unlit one.
    ///
    /// Drawn here, or copied from the glyph the cell names when the spec loads,
    /// so that everything downstream reads one field.
    #[serde(default)]
    pub pixels: Vec<String>,
    /// The glyph in `glyphs.toml` this cell is drawn with, instead of `pixels`.
    #[serde(default)]
    pub glyph: Option<String>,
}

/// One algorithm's membership of a character, with the reason it is listed.
#[derive(Debug, Deserialize)]
pub struct Member {
    /// The algorithm, by its `effects.toml` name.
    pub name: String,
    /// Why it has the character: the manual's name, its slots, or the unit its
    /// name refers to.
    pub because: String,
}

/// What kind of thing an effect is, beside what it does.
///
/// A family is one per algorithm; characters are any number, and
/// `characters.toml` says how they are decided.
#[derive(Debug, Deserialize)]
pub struct Character {
    /// The character's name, which becomes a variant of `effect::Character`.
    pub name: String,
    /// What having the character means.
    pub description: String,
    /// The algorithms that have it, each with its reason.
    pub algorithms: Vec<Member>,
}

/// A picture of what a parameter does, on the same grid the cells are drawn on.
///
/// Reached from an effect slot, a program parameter, a controller or a cell by
/// name; `glyphs.toml` says how they are chosen.
#[derive(Debug, Deserialize)]
pub struct Glyph {
    /// The glyph's name, which becomes a variant of `pixels::Glyph`.
    pub name: String,
    /// What the glyph is a picture of.
    pub description: String,
    /// The drawing, one string per row, `#` for a lit dot and `.` for an unlit one.
    pub pixels: Vec<String>,
}

/// A firmware version that changes the protocol.
#[derive(Debug, Deserialize)]
pub struct Firmware {
    /// Dotted version as a device inquiry reports it, such as `"1.1"`.
    pub version: String,
    /// `true` for the version assumed when a caller names none.
    #[serde(default)]
    pub default: bool,
    /// What this version changed.
    #[serde(default)]
    pub note: Option<String>,
}

impl Firmware {
    /// Returns whether `range` covers `version`.
    ///
    /// A range is a bare version for exactly that one, a version with a
    /// trailing `+` for that one and later, or absent for every version.
    #[must_use]
    pub fn range_covers(range: Option<&str>, version: &str) -> bool {
        match range {
            None => true,
            Some(range) => match range.strip_suffix('+') {
                Some(from) => version_parts(version) >= version_parts(from),
                None => range == version,
            },
        }
    }
}

/// Splits a version or version range, `"1.1"` or `"1.1+"`, into its two numbers.
#[must_use]
pub fn version_parts(version: &str) -> (u32, u32) {
    let mut parts = version.trim_end_matches('+').split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor)
}

/// A named table of parameter values.
#[derive(Debug, Deserialize)]
pub struct ValueTable {
    /// Identifier referenced by [`Parameter::value_table`].
    pub id: String,
    /// Human-readable table name.
    pub name: String,
    /// Which firmware versions this table describes. See [`Firmware::range_covers`].
    #[serde(default)]
    pub firmware: Option<String>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// `false` when the mapping is inferred and still needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
    /// `true` when the entries list only the start of a documented range, such
    /// as SPREAD-1 standing for SPREAD-1 through SPREAD-254.
    #[serde(default)]
    pub partial: bool,
    /// The values themselves.
    pub entries: Vec<EnumEntry>,
}

const fn yes() -> bool {
    true
}

/// A `SysEx` message.
#[derive(Debug, Deserialize)]
pub struct Message {
    /// Command byte, the one following the device ID.
    pub command: u8,
    /// Human-readable message name.
    pub name: String,
    /// `"to_device"` or `"from_device"`.
    pub direction: String,
    /// Summary of the bytes between the command byte and `F7`.
    #[serde(default)]
    pub payload: Option<String>,
    /// Unpacked payload length in bytes, where the message carries bulk data.
    #[serde(default)]
    pub raw_len: Option<u32>,
    /// Packed payload length in bytes, where the message carries bulk data.
    #[serde(default)]
    pub packed_len: Option<u32>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// A MIDI continuous controller the synthesizer answers.
#[derive(Debug, Deserialize)]
pub struct Controller {
    /// Controller number, 0-127.
    pub cc: u8,
    /// What it controls.
    pub name: String,
    /// `"parameter"`, `"standard"` or `"other"`.
    pub kind: String,
    /// Offset of the program parameter it drives, when it drives one.
    #[serde(default)]
    pub parameter: Option<u16>,
    /// `false` when the assignment is inferred and needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// The glyph in `glyphs.toml` that pictures a standard controller.
    ///
    /// A controller that drives a parameter takes the parameter's glyph, so
    /// this is only set on the ones the MIDI specification defines.
    #[serde(default)]
    pub glyph: Option<String>,
}

/// One parameter of one effect algorithm.
///
/// An engine holds twelve raw bytes whatever it is running; `slot` says which
/// of those twelve this is, and the meaning comes from the algorithm.
#[derive(Debug, Deserialize)]
pub struct EffectParameter {
    /// Position within the engine's twelve parameters, counting from 1.
    pub slot: u8,
    /// Short name as the synthesizer's display shows it.
    pub r#ref: String,
    /// Full parameter name.
    pub name: String,
    /// Unit of the displayed value, where there is one.
    #[serde(default)]
    pub unit: Option<String>,
    /// Lowest displayed value, for a parameter with a numeric range.
    #[serde(default)]
    pub min: Option<String>,
    /// Highest displayed value, for a parameter with a numeric range.
    #[serde(default)]
    pub max: Option<String>,
    /// Options this parameter selects between, for a parameter without a range.
    #[serde(default)]
    pub values: Option<String>,
    /// `true` when the manual marks the parameter as responding to modulation.
    ///
    /// Every slot is addressable from the modulation matrix regardless, as
    /// `Fx <engine> Param <slot>`; this says the engine acts on what arrives.
    #[serde(default)]
    pub mod_dest: bool,
    /// What the parameter does, from the manual. See NOTICE.
    #[serde(default)]
    pub description: Option<String>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// What this row reads in the manual, where the manual is wrong.
    #[serde(default)]
    pub correction: Option<String>,
}

/// An effect algorithm and the twelve parameters it gives its engine.
#[derive(Debug, Deserialize)]
pub struct Effect {
    /// Value of the `FX Type` parameter that selects this algorithm.
    pub r#type: u16,
    /// Short name, matching the `fx_type` value table.
    pub name: String,
    /// Full name as the manual writes it.
    pub full_name: String,
    /// The parameters, ordered by slot.
    pub parameters: Vec<EffectParameter>,
}

/// How one effect slot presents itself to a host.
#[derive(Debug, Deserialize)]
pub struct PanelSlot {
    /// Position within the engine's twelve parameters, counting from 1.
    pub slot: u8,
    /// The parameter written out in full, for a panel with room for it.
    ///
    /// `effects.toml` keeps the manual's abbreviated spelling, which matches the
    /// synthesizer's own display; this is that name expanded.
    pub title: String,
    /// `continuous`, `switch` or `selector`.
    pub kind: String,
    /// What the slot does to a signal, as against what it is called.
    ///
    /// One of `meta.quantities`. A different question from `kind`, which is
    /// what control to draw: a delay's `Factor` is drawn as a selector and is a
    /// time.
    pub quantity: String,
    /// The glyph in `glyphs.toml` that pictures what the slot does.
    pub glyph: String,
    /// Slots sharing a label belong together, such as one side of a dual engine.
    #[serde(default)]
    pub group: Option<String>,
    /// `true` when the engine acts on modulation reaching this slot.
    #[serde(default)]
    pub modulatable: bool,
    /// `true` when this switch takes the whole effect out of circuit.
    ///
    /// Three of the 35 have one, which is the only slot-level off the
    /// instrument has; see the note on the `fx_type` table in `enums.toml`.
    #[serde(default)]
    pub enable: bool,
}

/// The grid the synthesizer draws its FX page on, in display pixels.
///
/// Measured from the 35 screenshots in section 9.3 of the manual, which all
/// agree: one circle per slot, on six columns and two rows, filled in slot
/// order from the top left.
#[derive(Debug, Deserialize)]
pub struct Grid {
    /// Slots per row.
    pub columns: u8,
    /// Rows per page.
    pub rows: u8,
    /// What a row that is not full does with the space left over.
    pub align: String,
    /// The order slots are placed in.
    pub order: String,
    /// The shape the synthesizer draws for every slot on its own FX page.
    pub shape: String,
    /// Width of the display the grid was measured on.
    pub display_width: f32,
    /// Height of the display the grid was measured on.
    pub display_height: f32,
    /// Centre of the first column.
    pub first_x: f32,
    /// Centre of the first row.
    pub first_y: f32,
    /// Distance between column centres.
    pub column_pitch: f32,
    /// Distance between row centres.
    pub row_pitch: f32,
    /// Diameter of the circle drawn for a slot.
    pub control_diameter: f32,
}

/// One row of an effect's FX page.
#[derive(Debug, Deserialize)]
pub struct LayoutRow {
    /// Slots on this row, in the order they are drawn.
    pub slots: Vec<u8>,
    /// What this row does with the space left over when it is not full.
    pub align: String,
}

/// Where one effect's controls sit, and what colour its panel is.
#[derive(Debug, Deserialize)]
pub struct Layout {
    /// Value of the `FX Type` parameter that selects this algorithm.
    pub r#type: u16,
    /// Short name, matching the effect and the `fx_type` value table.
    pub name: String,
    /// The manual's own classification: Reverb, Processing, Delay or Creative.
    pub category: String,
    /// What the effect's own editor panel uses for a sweeping parameter:
    /// `knob`, `fader` or `display`.
    ///
    /// This is not the FX page, which draws every slot as a circle. It is the
    /// panel printed beside it, and the two disagree for five effects.
    pub control: String,
    /// The case around the controls.
    pub chassis: String,
    /// The surface the controls sit on.
    pub face: String,
    /// The part a finger moves: a knob body or a fader cap.
    pub cap: String,
    /// Most saturated colour covering a visible share of the panel.
    pub accent: String,
    /// The rows, in the order they are drawn.
    pub rows: Vec<LayoutRow>,
}

/// One family of effects and the mark it is drawn with.
///
/// `category` in layout.toml is the manual's own four buckets; a family is what
/// an effect does to a signal, at the resolution a symbol needs.
#[derive(Debug, Deserialize)]
pub struct Family {
    /// The family's name, which becomes a variant of `effect::Family`.
    pub name: String,
    /// What the mark is a picture of.
    pub description: String,
    /// The algorithms in this family, by their `effects.toml` name.
    pub algorithms: Vec<String>,
    /// The mark, as strokes in a unit box with the origin top left.
    pub strokes: Vec<Stroke>,
    /// The same mark on a square one-bit grid, one string per row.
    pub pixels: Vec<String>,
}

/// A mark drawn for a kind of effect within a family, where the kind is
/// something a symbol can carry.
///
/// The same shape as a [`Family`], and the same language: every reverb is a
/// source with wavefronts leaving it, and a plate is one whose source is a
/// plate. An algorithm in no variant is drawn with its family's mark.
#[derive(Debug, Deserialize)]
pub struct Variant {
    /// The variant's name, one or two words.
    pub name: String,
    /// What the mark is a picture of, and what it adds to the family's.
    pub description: String,
    /// The algorithms drawn with it, by their `effects.toml` name.
    pub algorithms: Vec<String>,
    /// The mark, as strokes in a unit box with the origin top left.
    pub strokes: Vec<Stroke>,
    /// The same mark on a square one-bit grid, one string per row.
    pub pixels: Vec<String>,
}

impl Arc {
    /// Returns the points the swept part of the arc has to be checked against.
    ///
    /// Its two ends, plus each compass point the sweep actually passes through,
    /// which is where a circle reaches its extremes. Checking the whole circle
    /// instead would forbid a shallow arc drawn from a distant centre, which is
    /// how the reverb's wavefronts are drawn.
    fn extent(&self) -> Vec<(f32, f32)> {
        let at = |turns: f32| {
            let radians = turns * std::f32::consts::TAU;
            (
                self.centre[0] + self.radius * radians.cos(),
                self.centre[1] + self.radius * radians.sin(),
            )
        };
        let (from, to) = if self.sweep >= 0.0 {
            (self.start, self.start + self.sweep)
        } else {
            (self.start + self.sweep, self.start)
        };

        let mut points = vec![at(from), at(to)];
        // Every quarter turn inside the swept range, whichever turn it is in.
        let first = (from * 4.0).ceil();
        let last = (to * 4.0).floor();
        let mut quarter = first;
        while quarter <= last {
            points.push(at(quarter / 4.0));
            quarter += 1.0;
        }
        points
    }
}

impl Wave {
    /// Returns the corners of the box the wave is drawn inside.
    ///
    /// The centre line's own ends, each pushed `amplitude` both ways along the
    /// perpendicular. A sine reaches its amplitude somewhere whenever it runs a
    /// half cycle or more, and every mark's wave does, so this is the box
    /// rather than an over-estimate of it.
    fn extent(&self) -> Vec<(f32, f32)> {
        let (dx, dy) = (self.end[0] - self.start[0], self.end[1] - self.start[1]);
        let length = dx.hypot(dy);
        if length <= 0.0 {
            return vec![(self.start[0], self.start[1])];
        }
        // The start-to-end direction turned a quarter turn anticlockwise, which
        // in a y-down box points up the screen.
        let (nx, ny) = (dy / length, -dx / length);

        let mut points = Vec::new();
        for end in [self.start, self.end] {
            for side in [1.0, -1.0] {
                points.push((
                    end[0] + nx * self.amplitude * side,
                    end[1] + ny * self.amplitude * side,
                ));
            }
        }
        points
    }
}

/// Checks a mark's strokes are drawable and inside the unit box.
///
/// # Errors
///
/// Returns a message when a stroke sets neither `line` nor `arc` or both,
/// when a polyline has fewer than two points, when an arc has no radius or
/// no sweep, or when any of it falls outside `margin..=1.0 - margin`.
fn validate_strokes(name: &str, strokes: &[Stroke], margin: f32) -> Result<(), String> {
    let inside = |what: &str, x: f32, y: f32| -> Result<(), String> {
        if (margin..=1.0 - margin).contains(&x) && (margin..=1.0 - margin).contains(&y) {
            return Ok(());
        }
        Err(format!(
            "marks.toml: {}'s {what} reaches ({x}, {y}), outside {margin} to {}",
            name,
            1.0 - margin
        ))
    };

    for (index, stroke) in strokes.iter().enumerate() {
        let kinds = usize::from(stroke.line.is_some())
            + usize::from(stroke.arc.is_some())
            + usize::from(stroke.dot.is_some())
            + usize::from(stroke.wave.is_some());
        if kinds != 1 {
            return Err(format!(
                "marks.toml: {name} stroke {index} sets {kinds} of line, arc, dot and wave, not 1"
            ));
        }

        if let Some(points) = &stroke.line {
            if points.len() < 2 {
                return Err(format!(
                    "marks.toml: {} stroke {index} is a line of {} point(s)",
                    name,
                    points.len()
                ));
            }
            for &[x, y] in points {
                inside("line", x, y)?;
            }
        }

        if let Some(arc) = &stroke.arc {
            if arc.radius <= 0.0 {
                return Err(format!(
                    "marks.toml: {} stroke {index} is an arc of radius {}",
                    name, arc.radius
                ));
            }
            if arc.sweep == 0.0 {
                return Err(format!(
                    "marks.toml: {name} stroke {index} is an arc that sweeps nothing"
                ));
            }
            // Only the part actually swept, so that an arc may be taken
            // from a circle reaching outside the box. That is what lets a
            // shallow wavefront be drawn from a distant centre.
            for (x, y) in arc.extent() {
                inside("arc", x, y)?;
            }
        }

        if let Some(dot) = &stroke.dot {
            if dot.radius <= 0.0 {
                return Err(format!(
                    "marks.toml: {} stroke {index} is a dot of radius {}",
                    name, dot.radius
                ));
            }
            inside(
                "dot",
                dot.centre[0] - dot.radius,
                dot.centre[1] - dot.radius,
            )?;
            inside(
                "dot",
                dot.centre[0] + dot.radius,
                dot.centre[1] + dot.radius,
            )?;
        }

        if let Some(wave) = &stroke.wave {
            if wave.amplitude <= 0.0 {
                return Err(format!(
                    "marks.toml: {} stroke {index} is a wave of amplitude {}",
                    name, wave.amplitude
                ));
            }
            if wave.cycles <= 0.0 {
                return Err(format!(
                    "marks.toml: {} stroke {index} is a wave of {} cycles",
                    name, wave.cycles
                ));
            }
            for (x, y) in wave.extent() {
                inside("wave", x, y)?;
            }
        }
    }
    Ok(())
}

/// Side of the grid every pixel drawing is on: the marks and the cells both.
///
/// The library's `pixels::SIDE` is the same number, and a grid here that was
/// not this wide would not fit the type there.
pub const PIXEL_SIDE: usize = 7;

/// Checks that a pixel grid is `side` rows of `side` characters, each `#` or
/// `.`, and draws something a reader could tell from anything else.
///
/// # Errors
///
/// Returns a message naming `file` and `what` when the grid is the wrong
/// shape, uses another character, or is empty or completely full, both of
/// which draw nothing.
fn validate_pixels(file: &str, what: &str, rows: &[String], side: usize) -> Result<(), String> {
    if rows.len() != side {
        return Err(format!(
            "{file}: {what} has {} pixel rows, not {side}",
            rows.len()
        ));
    }
    let mut lit = 0;
    for (y, row) in rows.iter().enumerate() {
        if row.chars().count() != side {
            return Err(format!(
                "{file}: {what} pixel row {y} is {} characters, not {side}",
                row.chars().count()
            ));
        }
        for (x, pixel) in row.chars().enumerate() {
            match pixel {
                '#' => lit += 1,
                '.' => {}
                other => {
                    return Err(format!(
                        "{file}: {what} pixel ({x}, {y}) is {other:?}, not '#' or '.'"
                    ));
                }
            }
        }
    }
    if lit == 0 || lit == side * side {
        return Err(format!(
            "{file}: {what} lights {lit} pixels of {}, which draws nothing",
            side * side
        ));
    }
    Ok(())
}

/// One stroke of a mark: a polyline, a circular arc or a filled dot.
///
/// Exactly one of the three is set, which the loader checks; TOML has no tagged
/// union, so this is how a stroke says which it is.
#[derive(Debug, Deserialize)]
pub struct Stroke {
    /// Two or more points, in a unit box.
    #[serde(default)]
    pub line: Option<Vec<[f32; 2]>>,
    /// A circular arc, in the same box.
    #[serde(default)]
    pub arc: Option<Arc>,
    /// A filled disc, in the same box.
    #[serde(default)]
    pub dot: Option<Dot>,
    /// A sine along a line, in the same box.
    #[serde(default)]
    pub wave: Option<Wave>,
}

/// A sine of a mark, along the line from `start` to `end`.
///
/// A function rather than a set of points: how finely to sample it is a
/// question about the size the host is drawing at.
#[derive(Debug, Deserialize)]
pub struct Wave {
    /// Where the centre line begins.
    pub start: [f32; 2],
    /// Where the centre line ends.
    pub end: [f32; 2],
    /// Peak displacement from the centre line, perpendicular to it.
    pub amplitude: f32,
    /// Cycles between `start` and `end`.
    pub cycles: f32,
}

/// A filled dot of a mark, in a unit box with the origin top left.
#[derive(Debug, Deserialize)]
pub struct Dot {
    /// Centre of the disc.
    pub centre: [f32; 2],
    /// Radius of the disc.
    pub radius: f32,
}

/// A circular arc of a mark, in a unit box with the origin top left.
#[derive(Debug, Deserialize)]
pub struct Arc {
    /// Centre of the circle the arc is taken from.
    pub centre: [f32; 2],
    /// Radius of that circle.
    pub radius: f32,
    /// Where the arc starts, in turns clockwise from three o'clock.
    pub start: f32,
    /// How far it goes, in turns, clockwise.
    pub sweep: f32,
}

/// The presentation of one effect algorithm's slots.
#[derive(Debug, Deserialize)]
pub struct Panel {
    /// Value of the `FX Type` parameter that selects this algorithm.
    pub r#type: u16,
    /// Short name, matching the effect and the `fx_type` value table.
    pub name: String,
    /// The slots, ordered.
    pub slots: Vec<PanelSlot>,
}

/// A raw parameter value printed beside the value the synthesizer displays.
///
/// Taken from the PROG screenshots in the manual, where the upper number is the
/// parameter's MIDI value and the line along the bottom of the screen is the
/// same parameter in its own units. These decide which curves are still
/// possible between a parameter's two stated ends; none of them is a conversion.
#[derive(Debug, Deserialize)]
pub struct Measurement {
    /// Offset of the parameter read, joining to `parameters.toml`.
    pub offset: u16,
    /// The value on the wire, 0-255.
    pub raw: u16,
    /// What the synthesizer displayed for it, units and all.
    pub shown: String,
    /// Which interpolation between the parameter's stated ends reproduces this.
    pub fit: String,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// One effect slot's place in a routing topology.
#[derive(Debug, Deserialize)]
pub struct RoutingSlot {
    /// Which of the four engines this is, counting from 1.
    pub slot: u8,
    /// What reaches this slot: 0 is the FX block's input, 1-4 are other slots.
    pub from: Vec<u8>,
}

/// One of the ten ways the four effect engines can be wired together.
#[derive(Debug, Deserialize)]
pub struct Routing {
    /// Value of the `FX Routing` parameter that selects this topology.
    pub value: u16,
    /// The manual's label for it, `M-1` through `M-10`.
    pub label: String,
    /// Name, matching the `fx_routing` value table.
    pub name: String,
    /// `true` when the topology contains a feedback loop.
    #[serde(default)]
    pub feedback: bool,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// The four slots, ordered.
    pub slots: Vec<RoutingSlot>,
    /// Slots whose outputs are summed to leave the FX block.
    pub output: Vec<u8>,
}

/// What one `FX Mode` setting does to the two paths through the instrument.
#[derive(Debug, Deserialize)]
pub struct FxMode {
    /// Value of the `FX Mode` parameter that selects this setting.
    pub value: u16,
    /// Name, matching the `fx_mode` value table.
    pub name: String,
    /// `true` when the analog path from the voices to the output stage is live.
    pub analog_path: bool,
    /// `true` when the voices also run through the FX block.
    pub digital_path: bool,
}

/// One control the front panel puts under a legend.
#[derive(Debug, Deserialize)]
pub struct FrontControl {
    /// The program parameter it moves, by its `parameters.toml` name.
    pub parameter: String,
    /// What the panel prints over it, as the silkscreen has it.
    pub legend: String,
    /// What a hand touches: `fader`, `button` or `lamps`.
    pub shape: String,
}

/// One group of controls as the front panel prints it, such as `VCF`.
#[derive(Debug, Deserialize)]
pub struct Section {
    /// The name across the top of the plate, as the silkscreen has it.
    pub name: String,
    /// Which of the panel's rows it is in, counting from 0.
    pub row: u8,
    /// The group of the parameter table its controls belong to.
    pub group: String,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// The controls, in the order the panel puts them.
    pub controls: Vec<FrontControl>,
}

/// One field of a transport's byte pattern.
#[derive(Debug, Deserialize)]
pub struct MappingField {
    /// Name used by the `<placeholder>` in the pattern.
    pub name: String,
    /// Where the value comes from, such as `parameter.offset`.
    pub source: String,
    /// Identifier of the encoding applied, if any.
    #[serde(default)]
    pub encoding: Option<String>,
    /// Width in bits, where the field is a fixed-width number.
    #[serde(default)]
    pub bits: Option<u8>,
    /// `true` when the field may be left out.
    #[serde(default)]
    pub optional: bool,
    /// When the field may be left out.
    #[serde(default)]
    pub optional_when: Option<String>,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// A way of carrying an address and a value over MIDI.
#[derive(Debug, Deserialize)]
pub struct Transport {
    /// Identifier, such as `nrpn`.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// One-line summary of what it reaches.
    pub summary: String,
    /// Byte pattern, with `<name>` standing for a field.
    pub pattern: String,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// The fields the pattern refers to.
    #[serde(default, rename = "field")]
    pub fields: Vec<MappingField>,
}

/// A named rule turning a value into wire bytes.
#[derive(Debug, Deserialize)]
pub struct Encoding {
    /// Identifier referenced by [`MappingField::encoding`].
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// The rule itself.
    pub rule: String,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
}

/// A device-wide setting.
#[derive(Debug, Deserialize)]
pub struct Global {
    /// Human-readable setting name.
    pub name: String,
    /// Lowest accepted value.
    pub min: u16,
    /// Highest accepted value.
    pub max: u16,
    /// Free-form note.
    #[serde(default)]
    pub note: Option<String>,
    /// `false` when the range or ordering is inferred and needs hardware confirmation.
    #[serde(default = "yes")]
    pub confirmed: bool,
}

#[derive(Debug, Deserialize)]
struct Parameters {
    parameter: Vec<Parameter>,
}

#[derive(Debug, Deserialize)]
struct ValueTables {
    #[serde(rename = "enum")]
    tables: Vec<ValueTable>,
}

#[derive(Debug, Deserialize)]
struct Messages {
    message: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Controllers {
    controller: Vec<Controller>,
}

#[derive(Debug, Deserialize)]
struct Panels {
    meta: PanelMeta,
    panel: Vec<Panel>,
}

#[derive(Debug, Deserialize)]
struct PanelMeta {
    quantities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Marks {
    family: Vec<Family>,
    #[serde(default)]
    variant: Vec<Variant>,
}

#[derive(Debug, Deserialize)]
struct Cells {
    cell: Vec<Cell>,
}

#[derive(Debug, Deserialize)]
struct Glyphs {
    glyph: Vec<Glyph>,
}

#[derive(Debug, Deserialize)]
struct Characters {
    character: Vec<Character>,
}

#[derive(Debug, Deserialize)]
struct Layouts {
    meta: LayoutMeta,
    grid: Grid,
    layout: Vec<Layout>,
}

#[derive(Debug, Deserialize)]
struct LayoutMeta {
    aligns: Vec<String>,
    controls: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Measurements {
    measurement: Vec<Measurement>,
}

#[derive(Debug, Deserialize)]
struct Routings {
    routing: Vec<Routing>,
    mode: Vec<FxMode>,
}

#[derive(Debug, Deserialize)]
struct Mapping {
    transport: Vec<Transport>,
    #[serde(rename = "encoding")]
    encodings: Vec<Encoding>,
}

#[derive(Debug, Deserialize)]
struct Firmwares {
    firmware: Vec<Firmware>,
}

#[derive(Debug, Deserialize)]
struct Effects {
    meta: EffectsMeta,
    effect: Vec<Effect>,
}

/// What every effect engine has in common, whatever it is running.
#[derive(Debug, Deserialize)]
pub struct EffectsMeta {
    /// Program offset of each engine's first parameter slot, in engine order.
    pub engine_offsets: Vec<u16>,
    /// Parameter slots one engine holds.
    pub slots_per_engine: u8,
}

#[derive(Debug, Deserialize)]
struct Globals {
    #[serde(rename = "global")]
    globals: Vec<Global>,
}

#[derive(Debug, Deserialize)]
struct Front {
    meta: FrontMeta,
    section: Vec<Section>,
}

#[derive(Debug, Deserialize)]
struct FrontMeta {
    rows: u8,
    shape_kinds: Vec<String>,
}

/// Everything in `spec/`, loaded and validated.
#[derive(Debug)]
pub struct Spec {
    /// Program parameters, ordered by offset.
    pub parameters: Vec<Parameter>,
    /// Value tables, ordered as written.
    pub tables: Vec<ValueTable>,
    /// `SysEx` messages, ordered by command byte.
    pub messages: Vec<Message>,
    /// Device-wide settings.
    pub globals: Vec<Global>,
    /// Continuous controllers, ordered by number.
    pub controllers: Vec<Controller>,
    /// Effect algorithms, ordered by `FX Type` value.
    pub effects: Vec<Effect>,
    /// What the four effect engines have in common.
    pub engines: EffectsMeta,
    /// Firmware versions that change the protocol, oldest first.
    pub firmwares: Vec<Firmware>,
    /// Ways of carrying an address and a value over MIDI.
    pub transports: Vec<Transport>,
    /// Rules turning a value into wire bytes.
    pub encodings: Vec<Encoding>,
    /// The grid every effect's FX page is drawn on.
    pub grid: Grid,
    /// Where each effect's controls sit and what colour it is, by `FX Type`.
    pub layouts: Vec<Layout>,
    /// The alignments a row may declare.
    pub aligns: Vec<String>,
    /// The control shapes a layout may declare.
    pub controls: Vec<String>,
    /// How each effect presents its slots, ordered by `FX Type` value.
    pub panels: Vec<Panel>,
    /// The quantities a slot may declare.
    pub quantities: Vec<String>,
    /// What each effect does to a signal, with the mark it is drawn with.
    pub families: Vec<Family>,
    /// The marks drawn for kinds of effect within a family, as written.
    pub variants: Vec<Variant>,
    /// The dot-matrix cells drawn for value table entries, as written, each
    /// with its pixels filled in from its glyph where it named one.
    pub cells: Vec<Cell>,
    /// The pictures of what a parameter does, as written.
    pub glyphs: Vec<Glyph>,
    /// What kind of thing each effect is, as written.
    pub characters: Vec<Character>,
    /// How the four engines can be wired, ordered by `FX Routing` value.
    pub routings: Vec<Routing>,
    /// What each `FX Mode` setting does to the analog and digital paths.
    pub fx_modes: Vec<FxMode>,
    /// Raw values read off the manual's screenshots with what they displayed.
    pub measurements: Vec<Measurement>,
    /// The front panel's groups, in the order the instrument prints them.
    pub sections: Vec<Section>,
    /// How many rows the front panel is printed in.
    pub panel_rows: u8,
    /// The control shapes a front panel control may declare.
    pub shape_kinds: Vec<String>,
}

impl Spec {
    /// Loads and validates every file under `spec/`.
    ///
    /// # Errors
    ///
    /// Returns a message naming the file and the problem when a file is missing,
    /// is not valid TOML, or fails one of the consistency checks: offsets must
    /// cover 0..=241 exactly, every referenced value table must exist, and every
    /// enumerated parameter's maximum must match its table.
    pub fn load(root: &Path) -> Result<Self, String> {
        let spec = root.join("spec");
        let parameters: Parameters = read(&spec.join("parameters.toml"))?;
        let tables: ValueTables = read(&spec.join("enums.toml"))?;
        let messages: Messages = read(&spec.join("messages.toml"))?;
        let globals: Globals = read(&spec.join("globals.toml"))?;
        let controllers: Controllers = read(&spec.join("controllers.toml"))?;
        let effects: Effects = read(&spec.join("effects.toml"))?;
        let firmwares: Firmwares = read(&spec.join("firmware.toml"))?;
        let mapping: Mapping = read(&spec.join("mapping.toml"))?;
        let panels: Panels = read(&spec.join("panels.toml"))?;
        let marks: Marks = read(&spec.join("marks.toml"))?;
        let cells: Cells = read(&spec.join("cells.toml"))?;
        let glyphs: Glyphs = read(&spec.join("glyphs.toml"))?;
        let characters: Characters = read(&spec.join("characters.toml"))?;
        let layouts: Layouts = read(&spec.join("layout.toml"))?;
        let routings: Routings = read(&spec.join("routing.toml"))?;
        let measurements: Measurements = read(&spec.join("measurements.toml"))?;
        let front: Front = read(&spec.join("front.toml"))?;

        let mut this = Self {
            parameters: parameters.parameter,
            tables: tables.tables,
            messages: messages.message,
            globals: globals.globals,
            controllers: controllers.controller,
            effects: effects.effect,
            engines: effects.meta,
            firmwares: firmwares.firmware,
            transports: mapping.transport,
            encodings: mapping.encodings,
            grid: layouts.grid,
            layouts: layouts.layout,
            aligns: layouts.meta.aligns,
            controls: layouts.meta.controls,
            panels: panels.panel,
            quantities: panels.meta.quantities,
            families: marks.family,
            variants: marks.variant,
            cells: cells.cell,
            glyphs: glyphs.glyph,
            characters: characters.character,
            routings: routings.routing,
            fx_modes: routings.mode,
            measurements: measurements.measurement,
            sections: front.section,
            panel_rows: front.meta.rows,
            shape_kinds: front.meta.shape_kinds,
        };
        this.validate_glyphs()?;
        this.resolve_cells()?;
        this.validate()?;
        Ok(this)
    }

    /// Checks the glyph catalogue and everything that names a glyph.
    ///
    /// Every glyph is drawn once, on the one grid; every effect slot names one;
    /// and a parameter, a controller or a cell that names one names one that
    /// exists. A reference to a glyph nobody drew is an error here rather than
    /// a table that does not compile.
    fn validate_glyphs(&self) -> Result<(), String> {
        let mut names: Vec<&str> = Vec::new();
        for glyph in &self.glyphs {
            if glyph.name.trim().is_empty() {
                return Err("glyphs.toml: a glyph has no name".to_owned());
            }
            if names.contains(&glyph.name.as_str()) {
                return Err(format!("glyphs.toml: {:?} is drawn twice", glyph.name));
            }
            names.push(&glyph.name);
            if glyph.description.trim().is_empty() {
                return Err(format!("glyphs.toml: {} has no description", glyph.name));
            }
            validate_pixels("glyphs.toml", &glyph.name, &glyph.pixels, PIXEL_SIDE)?;
        }
        let known = |file: &str, what: &str, name: &str| -> Result<(), String> {
            if names.contains(&name) {
                Ok(())
            } else {
                Err(format!(
                    "{file}: {what} names glyph {name:?}, which glyphs.toml does not draw"
                ))
            }
        };
        for panel in &self.panels {
            for slot in &panel.slots {
                known(
                    "panels.toml",
                    &format!("{} slot {}", panel.name, slot.slot),
                    &slot.glyph,
                )?;
            }
        }
        for parameter in &self.parameters {
            if let Some(glyph) = &parameter.glyph {
                known("parameters.toml", &parameter.name, glyph)?;
            }
        }
        for controller in &self.controllers {
            if let Some(glyph) = &controller.glyph {
                known("controllers.toml", &controller.name, glyph)?;
                if controller.parameter.is_some() {
                    return Err(format!(
                        "controllers.toml: {} drives a parameter and names a glyph; the parameter's is the one it gets",
                        controller.name
                    ));
                }
            }
        }
        for cell in &self.cells {
            if let Some(glyph) = &cell.glyph {
                known("cells.toml", &cell.name, glyph)?;
            }
        }
        Ok(())
    }

    /// Copies each glyph-drawn cell's pixels in from its glyph.
    ///
    /// A cell is either drawn or named, never both and never neither, so that
    /// the picture of a source lives in exactly one place.
    fn resolve_cells(&mut self) -> Result<(), String> {
        let glyphs = &self.glyphs;
        for cell in &mut self.cells {
            match (&cell.glyph, cell.pixels.is_empty()) {
                (Some(name), true) => {
                    let glyph = glyphs
                        .iter()
                        .find(|glyph| &glyph.name == name)
                        .ok_or_else(|| format!("cells.toml: {} names no glyph", cell.name))?;
                    cell.pixels.clone_from(&glyph.pixels);
                }
                (None, false) => {}
                (Some(_), false) => {
                    return Err(format!(
                        "cells.toml: {} both draws pixels and names a glyph",
                        cell.name
                    ));
                }
                (None, true) => {
                    return Err(format!(
                        "cells.toml: {} neither draws pixels nor names a glyph",
                        cell.name
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the firmware version used when a caller names none.
    ///
    /// The one marked `default` in `firmware.toml`, or the last listed.
    #[must_use]
    pub fn default_firmware(&self) -> &str {
        self.firmwares
            .iter()
            .find(|f| f.default)
            .or_else(|| self.firmwares.last())
            .map_or("", |f| f.version.as_str())
    }

    /// Returns the value table with this identifier as of `firmware`.
    ///
    /// Several tables may share an identifier when firmware renumbered their
    /// entries; the one whose firmware range covers `firmware` is the right one.
    #[must_use]
    pub fn table_for(&self, id: &str, firmware: &str) -> Option<&ValueTable> {
        self.tables
            .iter()
            .find(|t| t.id == id && Firmware::range_covers(t.firmware.as_deref(), firmware))
    }

    /// Returns the value table with this identifier on the default firmware.
    #[must_use]
    pub fn table(&self, id: &str) -> Option<&ValueTable> {
        self.table_for(id, self.default_firmware())
    }

    /// Returns every table with this identifier, newest firmware first.
    ///
    /// One table for most identifiers; one per firmware version for a table
    /// that firmware renumbered. An unversioned table sorts last.
    #[must_use]
    pub fn versions_of(&self, id: &str) -> Vec<&ValueTable> {
        let mut versions: Vec<&ValueTable> = self.tables.iter().filter(|t| t.id == id).collect();
        versions.sort_by_key(|table| {
            std::cmp::Reverse(version_parts(table.firmware.as_deref().unwrap_or("0.0")))
        });
        versions
    }

    /// Returns the cell drawn for `entry` of `table`, if one has been.
    ///
    /// Joined by the name as an identifier, the way the value types join the
    /// versions of a renumbered table, so one cell serves `Note Off Vel` and
    /// `NoteOff Vel` alike.
    #[must_use]
    pub fn cell_for(&self, table: &ValueTable, entry: &EnumEntry) -> Option<&Cell> {
        let wanted = pascal(&entry.name);
        self.cells
            .iter()
            .find(|cell| cell.table == table.id && pascal(&cell.name) == wanted)
    }

    fn validate(&self) -> Result<(), String> {
        self.validate_firmware()?;
        self.validate_parameters()?;
        self.validate_enum_parameters()?;
        self.validate_swings()?;
        self.validate_twins()?;
        self.validate_cells()?;
        self.validate_controllers()?;
        self.validate_effects()?;
        self.validate_characters()?;
        self.validate_engines()?;
        self.validate_layout()?;
        self.validate_routing()?;
        self.validate_measurements()?;
        self.validate_front()?;
        self.validate_mapping()?;
        self.validate_messages()
    }

    /// Checks that every table identifier resolves for every known firmware.
    ///
    /// A table that covers no version is unreachable; two that cover the same
    /// version make lookup depend on file order, which is how a renumbering
    /// silently goes wrong.
    fn validate_firmware(&self) -> Result<(), String> {
        if self.firmwares.is_empty() {
            return Err("firmware.toml: no versions listed".to_owned());
        }
        if self.firmwares.iter().filter(|f| f.default).count() > 1 {
            return Err("firmware.toml: more than one version marked default".to_owned());
        }

        for table in &self.tables {
            // The library binary-searches these, so a table out of order would
            // be a name the lookup cannot find rather than a slow one.
            let ascending = table
                .entries
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a.value < b.value));
            if !ascending {
                return Err(format!(
                    "enums.toml: table {} is not in ascending value order",
                    table.id
                ));
            }
        }

        let mut ids: Vec<&str> = self.tables.iter().map(|t| t.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        for id in ids {
            for firmware in &self.firmwares {
                let matches = self
                    .tables
                    .iter()
                    .filter(|t| {
                        t.id == id
                            && Firmware::range_covers(t.firmware.as_deref(), &firmware.version)
                    })
                    .count();
                if matches != 1 {
                    return Err(format!(
                        "enums.toml: table {id} has {matches} definitions for firmware {}, want 1",
                        firmware.version
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_parameters(&self) -> Result<(), String> {
        let offsets: Vec<u16> = self.parameters.iter().map(|p| p.offset).collect();
        let expected: Vec<u16> = (0..242).collect();
        if offsets != expected {
            return Err(format!(
                "parameters.toml: offsets must be 0..=241 in order, found {} entries starting {:?}",
                offsets.len(),
                &offsets[..offsets.len().min(5)]
            ));
        }

        for parameter in &self.parameters {
            self.validate_shape(parameter)?;
            let Some(id) = &parameter.value_table else {
                continue;
            };
            // A parameter's stated maximum is the one for the default firmware.
            // Older firmware reaches a lower maximum through its own table.
            let table = self.table(id).ok_or_else(|| {
                format!(
                    "parameter {} references unknown table {id}",
                    parameter.offset
                )
            })?;
            let highest = table.entries.iter().map(|e| e.value).max().unwrap_or(0);
            // Only exact, confirmed tables are checked. A table whose tail is a
            // documented range (SPREAD-n, User-n) lists just its first entry, and an
            // unconfirmed one is where the manual contradicts itself about the range.
            let contiguous = !table.partial
                && table.confirmed
                && table
                    .entries
                    .iter()
                    .enumerate()
                    .all(|(i, e)| u16::try_from(i) == Ok(e.value));
            if contiguous && highest != parameter.max {
                return Err(format!(
                    "parameter {} ({}) has max {} but table {id} tops out at {highest}",
                    parameter.offset, parameter.name, parameter.max
                ));
            }
        }

        Ok(())
    }

    /// Checks what a parameter says about its value beyond the range.
    ///
    /// A centre outside the range, or a bound naming a parameter that does not
    /// exist, would reach a host as a control drawn about the wrong point or a
    /// run dimmed by nothing. Both are transcription slips rather than things
    /// hardware could settle, so they are caught here.
    fn validate_shape(&self, parameter: &Parameter) -> Result<(), String> {
        let named = |field: &str, name: &Option<String>| -> Result<(), String> {
            let Some(name) = name else { return Ok(()) };
            if self.parameters.iter().any(|p| &p.name == name) {
                return Ok(());
            }
            Err(format!(
                "parameters.toml: {} {field} names unknown parameter {name:?}",
                parameter.name
            ))
        };
        named("bounded_by", &parameter.bounded_by)?;

        match (parameter.shape.as_deref(), parameter.centre) {
            (None, None) => {}
            (Some("bipolar"), Some(centre)) => {
                if centre < parameter.min || centre > parameter.max {
                    return Err(format!(
                        "parameters.toml: {} centres on {centre}, outside its range {}-{}",
                        parameter.name, parameter.min, parameter.max
                    ));
                }
                if centre == parameter.min || centre == parameter.max {
                    return Err(format!(
                        "parameters.toml: {} centres on one end of its range, which is not bipolar",
                        parameter.name
                    ));
                }
            }
            (Some("bipolar"), None) => {
                return Err(format!(
                    "parameters.toml: {} is bipolar and says nothing reads as zero",
                    parameter.name
                ));
            }
            (None, Some(_)) => {
                return Err(format!(
                    "parameters.toml: {} has a centre and no shape",
                    parameter.name
                ));
            }
            (Some(other), _) => {
                return Err(format!(
                    "parameters.toml: {} has unknown shape {other:?}",
                    parameter.name
                ));
            }
        }

        if let Some(inactive) = parameter.inactive {
            if inactive < parameter.min || inactive > parameter.max {
                return Err(format!(
                    "parameters.toml: {} is inactive at {inactive}, outside its range",
                    parameter.name
                ));
            }
        }
        Ok(())
    }

    /// Checks the parameters a value table's entries name.
    ///
    /// Two things, both of which are how this table stays maintained rather
    /// than transcribed: every name has to resolve, and an entry must not name
    /// the same parameter twice. That the versions of a renumbered table agree
    /// about what a name moves is [`validate_twins`](Self::validate_twins).
    fn validate_enum_parameters(&self) -> Result<(), String> {
        let named = |name: &str| self.parameters.iter().any(|p| p.name == name);
        for table in &self.tables {
            for entry in &table.entries {
                for (index, name) in entry.parameters.iter().enumerate() {
                    if !named(name) {
                        return Err(format!(
                            "enums.toml: table {} value {} ({}) names unknown parameter {name:?}",
                            table.id, entry.value, entry.name
                        ));
                    }
                    if entry.parameters[..index].contains(name) {
                        return Err(format!(
                            "enums.toml: table {} value {} ({}) names parameter {name:?} twice",
                            table.id, entry.value, entry.name
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks that the versions of a renumbered table agree about an entry:
    /// what it moves and which way it swings.
    ///
    /// Joined by the name as an identifier, the way the value types are, so
    /// `NoteOff Vel` on 1.0 is `Note Off Vel` on 1.1. This matters most for
    /// the destinations: the firmware 1.0 table is derived from the 1.1 one, so
    /// a mapping added to one and forgotten in the other would otherwise go
    /// unnoticed.
    fn validate_twins(&self) -> Result<(), String> {
        for table in &self.tables {
            for other in self.versions_of(&table.id) {
                if core::ptr::eq(other, table) {
                    continue;
                }
                for entry in &table.entries {
                    let wanted = pascal(&entry.name);
                    let Some(twin) = other.entries.iter().find(|e| pascal(&e.name) == wanted)
                    else {
                        continue;
                    };
                    if twin.parameters != entry.parameters {
                        return Err(format!(
                            "enums.toml: {:?} moves different parameters in {} and {}",
                            entry.name, table.name, other.name
                        ));
                    }
                    if twin.swing != entry.swing {
                        return Err(format!(
                            "enums.toml: {:?} swings differently in {} and {}",
                            entry.name, table.name, other.name
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks that a swing is one of the two named.
    fn validate_swings(&self) -> Result<(), String> {
        for table in &self.tables {
            for entry in &table.entries {
                let Some(swing) = &entry.swing else {
                    continue;
                };
                if !SWINGS.contains(&swing.as_str()) {
                    return Err(format!(
                        "enums.toml: table {} value {} ({}) swings {swing:?}, not one of {SWINGS:?}",
                        table.id, entry.value, entry.name
                    ));
                }
            }
        }
        Ok(())
    }

    /// Checks that every cell lands on an entry that exists, that no entry is
    /// drawn twice, and that each grid is a drawing.
    fn validate_cells(&self) -> Result<(), String> {
        let mut drawn: Vec<(&str, String)> = Vec::new();
        for cell in &self.cells {
            let versions = self.versions_of(&cell.table);
            if versions.is_empty() {
                return Err(format!(
                    "cells.toml: {:?} is drawn for table {:?}, which enums.toml does not list",
                    cell.name, cell.table
                ));
            }
            let wanted = pascal(&cell.name);
            if !versions
                .iter()
                .any(|table| table.entries.iter().any(|e| pascal(&e.name) == wanted))
            {
                return Err(format!(
                    "cells.toml: table {} has no entry named {:?}",
                    cell.table, cell.name
                ));
            }
            let key = (cell.table.as_str(), wanted);
            if drawn.contains(&key) {
                return Err(format!(
                    "cells.toml: {:?} in table {} is drawn twice",
                    cell.name, cell.table
                ));
            }
            drawn.push(key);
            validate_pixels("cells.toml", &cell.name, &cell.pixels, PIXEL_SIDE)?;
        }
        Ok(())
    }

    fn validate_controllers(&self) -> Result<(), String> {
        let mut seen_cc: Vec<u8> = Vec::new();
        for controller in &self.controllers {
            if seen_cc.contains(&controller.cc) {
                return Err(format!(
                    "controllers.toml: CC {} appears twice",
                    controller.cc
                ));
            }
            seen_cc.push(controller.cc);
            let Some(offset) = controller.parameter else {
                continue;
            };
            if !self.parameters.iter().any(|p| p.offset == offset) {
                return Err(format!(
                    "controllers.toml: CC {} maps to unknown parameter offset {offset}",
                    controller.cc
                ));
            }
            if self
                .controllers
                .iter()
                .filter(|c| c.parameter == Some(offset))
                .count()
                > 1
            {
                return Err(format!(
                    "controllers.toml: parameter offset {offset} is claimed by more than one CC"
                ));
            }
        }

        Ok(())
    }

    /// Checks the front panel against the parameter table it names.
    ///
    /// The panel is the one part of this specification that cannot be read off
    /// the MIDI appendix, so it is held to the rest of the specification
    /// instead: every parameter it names has to exist, a parameter may carry
    /// one control and not two, and a plate's controls have to belong to the
    /// group its name claims. That last one is what catches a parameter moved
    /// between groups by a later reading of the manual.
    fn validate_front(&self) -> Result<(), String> {
        let groups: Vec<&str> = self.parameters.iter().map(|p| p.group.as_str()).collect();
        let mut claimed: Vec<&str> = Vec::new();
        if self.sections.is_empty() {
            return Err("front.toml: no sections".to_owned());
        }
        for section in &self.sections {
            if section.row >= self.panel_rows {
                return Err(format!(
                    "front.toml: {} is in row {}, the panel has {}",
                    section.name, section.row, self.panel_rows
                ));
            }
            if !groups.contains(&section.group.as_str()) {
                return Err(format!(
                    "front.toml: {} opens {:?}, which is not a group",
                    section.name, section.group
                ));
            }
            if section.controls.is_empty() {
                return Err(format!("front.toml: {} has no controls", section.name));
            }
            for control in &section.controls {
                let parameter = self
                    .parameters
                    .iter()
                    .find(|p| p.name == control.parameter)
                    .ok_or_else(|| {
                        format!(
                            "front.toml: {} names unknown parameter {:?}",
                            section.name, control.parameter
                        )
                    })?;
                if parameter.group != section.group {
                    return Err(format!(
                        "front.toml: {} is on the {} plate and parameters.toml puts it in {}",
                        parameter.name, section.name, parameter.group
                    ));
                }
                if claimed.contains(&parameter.name.as_str()) {
                    return Err(format!(
                        "front.toml: {} has a control on the panel twice",
                        parameter.name
                    ));
                }
                claimed.push(parameter.name.as_str());
                if !self.shape_kinds.contains(&control.shape) {
                    return Err(format!(
                        "front.toml: {} has shape {:?}, which meta.shape_kinds does not list",
                        control.legend, control.shape
                    ));
                }
                // A legend is a silkscreen: caps, and short enough for a lane.
                if control.legend.trim().is_empty()
                    || control.legend.to_uppercase() != control.legend
                {
                    return Err(format!(
                        "front.toml: {} is not printed the way a panel prints one",
                        section.name
                    ));
                }
                if control.legend.split(' ').any(|word| word.len() > 6) {
                    return Err(format!(
                        "front.toml: {:?} is too long a word to print over a fader",
                        control.legend
                    ));
                }
                // A column of lit legends is only worth drawing for a parameter
                // that has names to light.
                if control.shape == "lamps" && parameter.value_table.is_none() {
                    return Err(format!(
                        "front.toml: {} is drawn as lamps and names no value table",
                        parameter.name
                    ));
                }
            }
        }
        // Not a second rack: the panel is the handful a player reaches for, and
        // a table that grew past a fraction of the instrument would have stopped
        // describing one.
        if claimed.len() * 4 >= self.parameters.len() {
            return Err(format!(
                "front.toml: {} of {} parameters are on the panel, which is not a front panel",
                claimed.len(),
                self.parameters.len()
            ));
        }
        Ok(())
    }

    /// Checks that every effect has a layout that places each of its slots once.
    ///
    /// The layout is a measurement of the manual's screenshots, so a slot count
    /// that disagrees with `effects.toml` means one of the two misread the
    /// manual. That is how the missing twelfth slot of `MoodFilter` was found.
    fn validate_layout(&self) -> Result<(), String> {
        if self.layouts.len() != self.effects.len() {
            return Err(format!(
                "layout.toml: {} layouts for {} effects",
                self.layouts.len(),
                self.effects.len()
            ));
        }
        for effect in &self.effects {
            let layout = self
                .layouts
                .iter()
                .find(|l| l.r#type == effect.r#type)
                .ok_or_else(|| format!("layout.toml: no layout for type {}", effect.r#type))?;
            if layout.name != effect.name {
                return Err(format!(
                    "layout.toml: type {} is {:?} but effects.toml calls it {:?}",
                    layout.r#type, layout.name, effect.name
                ));
            }
            if !self.controls.contains(&layout.control) {
                return Err(format!(
                    "layout.toml: {} has control {:?}, which meta.controls does not list",
                    layout.name, layout.control
                ));
            }
            for colour in [&layout.chassis, &layout.face, &layout.cap, &layout.accent] {
                if colour.len() != 7
                    || !colour.starts_with('#')
                    || !colour[1..].chars().all(|c| c.is_ascii_hexdigit())
                {
                    return Err(format!(
                        "layout.toml: {} has {colour:?}, which is not an #rrggbb colour",
                        layout.name
                    ));
                }
            }
            if layout.rows.len() > usize::from(self.grid.rows) {
                return Err(format!(
                    "layout.toml: {} has {} rows, the page holds {}",
                    layout.name,
                    layout.rows.len(),
                    self.grid.rows
                ));
            }
            let mut placed: Vec<u8> = Vec::new();
            for row in &layout.rows {
                if !self.aligns.contains(&row.align) {
                    return Err(format!(
                        "layout.toml: {} has align {:?}, which meta.aligns does not list",
                        layout.name, row.align
                    ));
                }
                if row.slots.len() > usize::from(self.grid.columns) {
                    return Err(format!(
                        "layout.toml: {} has a row of {}, the grid is {} wide",
                        layout.name,
                        row.slots.len(),
                        self.grid.columns
                    ));
                }
                placed.extend(&row.slots);
            }
            let wanted: Vec<u8> =
                (1..=u8::try_from(effect.parameters.len()).unwrap_or(u8::MAX)).collect();
            if placed != wanted {
                return Err(format!(
                    "layout.toml: {} places slots {placed:?}, but effects.toml has {} of them",
                    layout.name,
                    effect.parameters.len()
                ));
            }
        }
        Ok(())
    }

    /// Checks that the engine offsets `effects.toml` declares are where the
    /// parameter table actually puts the slots.
    ///
    /// The offsets are written down twice: as a base per engine here, and as 52
    /// named parameters in `parameters.toml`. A host addressing a slot has to be
    /// able to trust that the two agree. Naming them rather than counting from a
    /// base is what makes a renamed parameter a build error.
    fn validate_engines(&self) -> Result<(), String> {
        let named = |offset: u16, name: &str| -> Result<(), String> {
            let parameter = self
                .parameters
                .iter()
                .find(|p| p.offset == offset)
                .ok_or_else(|| format!("effects.toml: no parameter at offset {offset}"))?;
            if parameter.name == name {
                return Ok(());
            }
            Err(format!(
                "effects.toml: offset {offset} should be {name:?} and parameters.toml calls it {:?}",
                parameter.name
            ))
        };
        let slots = u16::from(self.engines.slots_per_engine);
        if slots == 0 {
            return Err("effects.toml: an engine holds no slots".to_owned());
        }
        for (index, base) in self.engines.engine_offsets.iter().enumerate() {
            let engine = index + 1;
            // The type byte sits immediately before the engine's slots, which
            // is what makes one base enough to describe an engine.
            let before = base
                .checked_sub(1)
                .ok_or_else(|| format!("effects.toml: engine {engine} starts at offset 0"))?;
            named(before, &format!("FX {engine} Type"))?;
            for slot in 1..=slots {
                named(base + slot - 1, &format!("FX {engine} Param {slot}"))?;
            }
            if !self
                .parameters
                .iter()
                .any(|p| p.name == format!("FX {engine} Output Gain"))
            {
                return Err(format!(
                    "effects.toml: parameters.toml has no FX {engine} Output Gain"
                ));
            }
        }
        for effect in &self.effects {
            if effect.parameters.len() > usize::from(self.engines.slots_per_engine) {
                return Err(format!(
                    "effects.toml: {} has {} parameters, an engine holds {}",
                    effect.name,
                    effect.parameters.len(),
                    self.engines.slots_per_engine
                ));
            }
        }
        Ok(())
    }

    fn validate_effects(&self) -> Result<(), String> {
        let fx_types: Vec<u16> = self.effects.iter().map(|e| e.r#type).collect();
        let expected_types: Vec<u16> = (0..35).collect();
        if fx_types != expected_types {
            return Err(format!(
                "effects.toml: type values must be 0..=34 in order, found {} entries",
                fx_types.len()
            ));
        }
        let fx_table = self
            .table("fx_type")
            .ok_or("enums.toml: no fx_type table to check effects.toml against")?;
        for effect in &self.effects {
            let listed = fx_table
                .entries
                .iter()
                .find(|e| e.value == effect.r#type)
                .ok_or_else(|| format!("fx_type has no value {}", effect.r#type))?;
            if listed.name != effect.name {
                return Err(format!(
                    "effects.toml: type {} is {:?} but fx_type calls it {:?}",
                    effect.r#type, effect.name, listed.name
                ));
            }
            if effect.parameters.len() > 12 {
                return Err(format!(
                    "effects.toml: {} has {} parameters, an engine holds 12",
                    effect.name,
                    effect.parameters.len()
                ));
            }
            for (index, parameter) in effect.parameters.iter().enumerate() {
                if usize::from(parameter.slot) != index + 1 {
                    return Err(format!(
                        "effects.toml: {} slot {} is out of order",
                        effect.name, parameter.slot
                    ));
                }
                if parameter.values.is_none() && parameter.min.is_none() {
                    return Err(format!(
                        "effects.toml: {} slot {} has neither a range nor a list of options",
                        effect.name, parameter.slot
                    ));
                }
            }
        }

        Ok(())
    }

    /// Checks that every character names algorithms that exist, once each,
    /// with a reason.
    ///
    /// A membership without a reason is a judgement about how something sounds,
    /// which is the one kind of evidence `characters.toml` refuses.
    fn validate_characters(&self) -> Result<(), String> {
        let mut names: Vec<&str> = Vec::new();
        for character in &self.characters {
            if character.name.trim().is_empty() {
                return Err("characters.toml: a character has no name".to_owned());
            }
            if names.contains(&character.name.as_str()) {
                return Err(format!(
                    "characters.toml: {} is listed twice",
                    character.name
                ));
            }
            names.push(&character.name);
            if character.description.trim().is_empty() {
                return Err(format!(
                    "characters.toml: {} has no description",
                    character.name
                ));
            }
            if character.algorithms.is_empty() {
                return Err(format!(
                    "characters.toml: {} names no algorithm",
                    character.name
                ));
            }
            let mut members: Vec<&str> = Vec::new();
            for member in &character.algorithms {
                if !self.effects.iter().any(|effect| effect.name == member.name) {
                    return Err(format!(
                        "characters.toml: {} claims {:?}, which effects.toml does not list",
                        character.name, member.name
                    ));
                }
                if members.contains(&member.name.as_str()) {
                    return Err(format!(
                        "characters.toml: {} lists {:?} twice",
                        character.name, member.name
                    ));
                }
                members.push(&member.name);
                if member.because.trim().is_empty() {
                    return Err(format!(
                        "characters.toml: {} lists {:?} without a reason",
                        character.name, member.name
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the characters `algorithm` has, in the order the file declares
    /// them.
    #[must_use]
    pub fn characters_of(&self, algorithm: &str) -> Vec<&Character> {
        self.characters
            .iter()
            .filter(|character| character.algorithms.iter().any(|m| m.name == algorithm))
            .collect()
    }

    /// Checks the ten routing topologies against the value tables and themselves.
    ///
    /// These were transcribed by eye from ten small printed diagrams. A slot
    /// that nothing feeds, or that reaches no output, is a misread line rather
    /// than a topology the hardware offers.
    fn validate_routing(&self) -> Result<(), String> {
        let table = self
            .table("fx_routing")
            .ok_or("enums.toml: no fx_routing table to check routing.toml against")?;
        if self.routings.len() != table.entries.len() {
            return Err(format!(
                "routing.toml has {} routings but fx_routing lists {}",
                self.routings.len(),
                table.entries.len()
            ));
        }
        for (index, routing) in self.routings.iter().enumerate() {
            if u16::try_from(index) != Ok(routing.value) {
                return Err(format!(
                    "routing.toml: {} has value {} at position {index}",
                    routing.label, routing.value
                ));
            }
            let listed = table
                .entries
                .iter()
                .find(|e| e.value == routing.value)
                .ok_or_else(|| format!("fx_routing has no value {}", routing.value))?;
            if listed.name != routing.name {
                return Err(format!(
                    "routing.toml: value {} is {:?} but fx_routing calls it {:?}",
                    routing.value, routing.name, listed.name
                ));
            }

            let slots: Vec<u8> = routing.slots.iter().map(|s| s.slot).collect();
            if slots != [1, 2, 3, 4] {
                return Err(format!(
                    "routing.toml: {} lists slots {slots:?}, want 1 to 4 in order",
                    routing.label
                ));
            }
            for slot in &routing.slots {
                if slot.from.is_empty() {
                    return Err(format!(
                        "routing.toml: {} slot {} has nothing feeding it",
                        routing.label, slot.slot
                    ));
                }
                for &from in &slot.from {
                    if from > 4 {
                        return Err(format!(
                            "routing.toml: {} slot {} is fed by {from}, which is not a slot",
                            routing.label, slot.slot
                        ));
                    }
                    if from == slot.slot {
                        return Err(format!(
                            "routing.toml: {} slot {} feeds itself",
                            routing.label, slot.slot
                        ));
                    }
                }
            }
            if routing.output.is_empty() {
                return Err(format!(
                    "routing.toml: {} sends no slot to the output",
                    routing.label
                ));
            }
            for &slot in &routing.output {
                if slot == 0 || slot > 4 {
                    return Err(format!(
                        "routing.toml: {} sends {slot} to the output, which is not a slot",
                        routing.label
                    ));
                }
            }
            Self::validate_routing_graph(routing)?;
        }
        self.validate_fx_modes()
    }

    /// Checks the three FX modes against the `fx_mode` value table.
    fn validate_fx_modes(&self) -> Result<(), String> {
        let table = self
            .table("fx_mode")
            .ok_or("enums.toml: no fx_mode table to check routing.toml against")?;
        if self.fx_modes.len() != table.entries.len() {
            return Err(format!(
                "routing.toml has {} modes but fx_mode lists {}",
                self.fx_modes.len(),
                table.entries.len()
            ));
        }
        for mode in &self.fx_modes {
            let listed = table
                .entries
                .iter()
                .find(|e| e.value == mode.value)
                .ok_or_else(|| format!("fx_mode has no value {}", mode.value))?;
            if listed.name != mode.name {
                return Err(format!(
                    "routing.toml: mode {} is {:?} but fx_mode calls it {:?}",
                    mode.value, mode.name, listed.name
                ));
            }
            if !mode.analog_path && !mode.digital_path {
                return Err(format!(
                    "routing.toml: mode {} leaves no path to the output",
                    mode.name
                ));
            }
        }
        Ok(())
    }

    /// Checks that a topology is connected and that `feedback` matches the graph.
    ///
    /// Every slot must be reachable from the block input and must reach the
    /// block output, and a loop must be declared rather than discovered.
    fn validate_routing_graph(routing: &Routing) -> Result<(), String> {
        let feeds = |slot: u8| -> &[u8] {
            routing
                .slots
                .iter()
                .find(|s| s.slot == slot)
                .map_or(&[][..], |s| s.from.as_slice())
        };

        // Walk forwards from the input, ignoring edges that close a loop.
        let mut reached = [false; 5];
        reached[0] = true;
        for _ in 0..4 {
            for slot in 1..=4u8 {
                if feeds(slot).iter().any(|&f| reached[usize::from(f)]) {
                    reached[usize::from(slot)] = true;
                }
            }
        }
        for slot in 1..=4u8 {
            if !reached[usize::from(slot)] {
                return Err(format!(
                    "routing.toml: {} slot {slot} is not reachable from the input",
                    routing.label
                ));
            }
        }

        // Walk backwards from the output the same way.
        let mut leads_out = [false; 5];
        for &slot in &routing.output {
            leads_out[usize::from(slot)] = true;
        }
        for _ in 0..4 {
            for slot in 1..=4u8 {
                let downstream = (1..=4u8)
                    .any(|other| leads_out[usize::from(other)] && feeds(other).contains(&slot));
                if downstream {
                    leads_out[usize::from(slot)] = true;
                }
            }
        }
        for slot in 1..=4u8 {
            if !leads_out[usize::from(slot)] {
                return Err(format!(
                    "routing.toml: {} slot {slot} reaches no output",
                    routing.label
                ));
            }
        }

        // A loop exists when some slot feeds itself through the others.
        let mut cyclic = false;
        for start in 1..=4u8 {
            let mut seen = [false; 5];
            let mut frontier = vec![start];
            while let Some(slot) = frontier.pop() {
                for &from in feeds(slot) {
                    if from == 0 {
                        continue;
                    }
                    if from == start {
                        cyclic = true;
                    }
                    if !seen[usize::from(from)] {
                        seen[usize::from(from)] = true;
                        frontier.push(from);
                    }
                }
            }
        }
        if cyclic != routing.feedback {
            return Err(format!(
                "routing.toml: {} declares feedback = {} but its graph {} a loop",
                routing.label,
                routing.feedback,
                if cyclic { "has" } else { "has no" }
            ));
        }
        Ok(())
    }

    /// Checks each reading against the parameter it claims to be of.
    ///
    /// A reading outside the parameter's own range is a misread screenshot, and
    /// a reading of an offset that does not exist is a typo. Neither is
    /// recoverable later: these are the only record of what the screen said.
    fn validate_measurements(&self) -> Result<(), String> {
        const FITS: [&str; 6] = [
            "linear",
            "exponential",
            "piecewise",
            "endpoint",
            "untested",
            "none",
        ];
        for measurement in &self.measurements {
            let parameter = self
                .parameters
                .iter()
                .find(|p| p.offset == measurement.offset)
                .ok_or_else(|| {
                    format!(
                        "measurements.toml: offset {} is not a parameter",
                        measurement.offset
                    )
                })?;
            if measurement.raw < parameter.min || measurement.raw > parameter.max {
                return Err(format!(
                    "measurements.toml: {} reads {} but the parameter runs {}-{}",
                    parameter.name, measurement.raw, parameter.min, parameter.max
                ));
            }
            if !FITS.contains(&measurement.fit.as_str()) {
                return Err(format!(
                    "measurements.toml: {} has unknown fit {:?}",
                    parameter.name, measurement.fit
                ));
            }
            if measurement.shown.trim().is_empty() {
                return Err(format!(
                    "measurements.toml: {} records no displayed value",
                    parameter.name
                ));
            }
        }
        Ok(())
    }

    /// Checks that every pattern placeholder has a field and every field an
    /// encoding that exists.
    fn validate_mapping(&self) -> Result<(), String> {
        for transport in &self.transports {
            for field in &transport.fields {
                let placeholder = format!("<{}>", field.name);
                if !transport.pattern.contains(&placeholder) {
                    return Err(format!(
                        "mapping.toml: {} defines field {} which its pattern never uses",
                        transport.id, field.name
                    ));
                }
                // Not a let chain: those need Rust 1.88, and the rust-version
                // in Cargo.toml is older.
                if let Some(id) = &field.encoding {
                    if !self.encodings.iter().any(|e| &e.id == id) {
                        return Err(format!(
                            "mapping.toml: {} field {} uses unknown encoding {id}",
                            transport.id, field.name
                        ));
                    }
                }
            }
            for placeholder in transport.pattern.split('<').skip(1) {
                let Some(name) = placeholder.split('>').next() else {
                    continue;
                };
                if !transport.fields.iter().any(|f| f.name == name) {
                    return Err(format!(
                        "mapping.toml: {} pattern uses <{name}> with no field to fill it",
                        transport.id
                    ));
                }
            }
        }
        self.validate_panels()?;
        self.validate_marks()
    }

    /// Checks that every algorithm has exactly one family and every mark is
    /// geometry a host can draw.
    ///
    /// The first half is what stops an algorithm added to effects.toml from
    /// being silently drawn as whatever the first family happens to be. The
    /// second is that a stroke says which of the two kinds it is, and stays
    /// inside the box the marks are declared to live in.
    fn validate_marks(&self) -> Result<(), String> {
        /// Coordinates are kept this far inside the unit box so that a host's
        /// stroke width has somewhere to go; see `meta.geometry`.
        const MARGIN: f32 = 0.08;

        let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();
        for family in &self.families {
            if family.name.trim().is_empty() {
                return Err("marks.toml: a family has no name".to_owned());
            }
            if family.strokes.is_empty() {
                return Err(format!("marks.toml: {} has no mark", family.name));
            }
            for name in &family.algorithms {
                if !self.effects.iter().any(|effect| &effect.name == name) {
                    return Err(format!(
                        "marks.toml: {} claims {name:?}, which effects.toml does not list",
                        family.name
                    ));
                }
                if let Some(first) = claimed.insert(name, &family.name) {
                    return Err(format!(
                        "marks.toml: {name:?} is in both {first} and {}",
                        family.name
                    ));
                }
            }
            validate_strokes(&family.name, &family.strokes, MARGIN)?;
            validate_pixels("marks.toml", &family.name, &family.pixels, PIXEL_SIDE)?;
        }

        if let Some(effect) = self
            .effects
            .iter()
            .find(|effect| !claimed.contains_key(effect.name.as_str()))
        {
            return Err(format!(
                "marks.toml: {} is in no family, so there is no mark to draw for it",
                effect.name
            ));
        }

        // A variant is drawn for algorithms of one family, at most one variant
        // each; an algorithm in none falls back to the family's mark.
        let mut drawn: BTreeMap<&str, &str> = BTreeMap::new();
        let mut names: Vec<&str> = Vec::new();
        for variant in &self.variants {
            if variant.name.trim().is_empty() {
                return Err("marks.toml: a variant has no name".to_owned());
            }
            if names.contains(&variant.name.as_str()) {
                return Err(format!(
                    "marks.toml: variant {} is drawn twice",
                    variant.name
                ));
            }
            names.push(&variant.name);
            if variant.algorithms.is_empty() {
                return Err(format!(
                    "marks.toml: variant {} is drawn for no algorithm",
                    variant.name
                ));
            }
            let mut family: Option<&str> = None;
            for name in &variant.algorithms {
                let Some(in_family) = claimed.get(name.as_str()) else {
                    return Err(format!(
                        "marks.toml: variant {} claims {name:?}, which effects.toml does not list",
                        variant.name
                    ));
                };
                // Spelled for Rust 1.85, which has no let chains.
                if family.is_some_and(|first| first != *in_family) {
                    return Err(format!(
                        "marks.toml: variant {} spans {} and {in_family}; a variant is of one family",
                        variant.name,
                        family.unwrap_or_default()
                    ));
                }
                family = Some(in_family);
                if let Some(first) = drawn.insert(name, &variant.name) {
                    return Err(format!(
                        "marks.toml: {name:?} is drawn as both {first} and {}",
                        variant.name
                    ));
                }
            }
            validate_strokes(&variant.name, &variant.strokes, MARGIN)?;
            validate_pixels("marks.toml", &variant.name, &variant.pixels, PIXEL_SIDE)?;
        }
        Ok(())
    }

    /// Returns the variant `algorithm` is drawn with, if one is.
    #[must_use]
    pub fn variant_of(&self, algorithm: &str) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|variant| variant.algorithms.iter().any(|a| a == algorithm))
    }

    /// Checks that panels.toml lines up with effects.toml slot for slot.
    ///
    /// The two are generated together, so a mismatch means one was edited by
    /// hand and the other was not.
    fn validate_panels(&self) -> Result<(), String> {
        const KINDS: [&str; 3] = ["continuous", "switch", "selector"];
        if self.panels.len() != self.effects.len() {
            return Err(format!(
                "panels.toml has {} panels but effects.toml has {} effects",
                self.panels.len(),
                self.effects.len()
            ));
        }
        for (panel, effect) in self.panels.iter().zip(&self.effects) {
            if panel.r#type != effect.r#type || panel.name != effect.name {
                return Err(format!(
                    "panels.toml has {} at type {} where effects.toml has {} at type {}",
                    panel.name, panel.r#type, effect.name, effect.r#type
                ));
            }
            if panel.slots.len() != effect.parameters.len() {
                return Err(format!(
                    "panels.toml: {} has {} slots but effects.toml has {} parameters",
                    panel.name,
                    panel.slots.len(),
                    effect.parameters.len()
                ));
            }
            for (slot, parameter) in panel.slots.iter().zip(&effect.parameters) {
                if slot.slot != parameter.slot {
                    return Err(format!(
                        "panels.toml: {} slot {} does not line up with effects.toml",
                        panel.name, slot.slot
                    ));
                }
                if slot.title.trim().is_empty() {
                    return Err(format!(
                        "panels.toml: {} slot {} has no title",
                        panel.name, slot.slot
                    ));
                }
                if !self.quantities.contains(&slot.quantity) {
                    return Err(format!(
                        "panels.toml: {} slot {} has unknown quantity {:?}",
                        panel.name, slot.slot, slot.quantity
                    ));
                }
                if !KINDS.contains(&slot.kind.as_str()) {
                    return Err(format!(
                        "panels.toml: {} slot {} has unknown kind {:?}",
                        panel.name, slot.slot, slot.kind
                    ));
                }
                if slot.modulatable != parameter.mod_dest {
                    return Err(format!(
                        "panels.toml: {} slot {} disagrees with effects.toml about modulation",
                        panel.name, slot.slot
                    ));
                }
                if slot.enable && slot.kind != "switch" {
                    return Err(format!(
                        "panels.toml: {} slot {} is an enable but has kind {:?}, \
                         and an effect is switched out of circuit or not",
                        panel.name, slot.slot, slot.kind
                    ));
                }
                // An enable is the manual's own claim that the slot bypasses the
                // effect, so it is tied to the wording rather than left to a
                // reading of the parameter's name. `bypass` catches the Noise
                // Gate, whose entry says the gate is bypassed rather than off.
                if let Some(description) = &parameter.description {
                    let says_bypass = description.contains("turned On or Off")
                        || description.contains("is bypassed");
                    if slot.enable != says_bypass {
                        return Err(format!(
                            "panels.toml: {} slot {} is marked enable = {} but effects.toml \
                             describes it as {description:?}",
                            panel.name, slot.slot, slot.enable
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_messages(&self) -> Result<(), String> {
        let mut commands: Vec<u8> = self.messages.iter().map(|m| m.command).collect();
        commands.sort_unstable();
        if commands.windows(2).any(|w| w[0] == w[1]) {
            // 0x08 is deliberately shared by two pattern responses.
            let duplicates: Vec<u8> = commands
                .windows(2)
                .filter(|w| w[0] == w[1])
                .map(|w| w[0])
                .filter(|c| *c != 0x08)
                .collect();
            if !duplicates.is_empty() {
                return Err(format!(
                    "messages.toml: duplicate command bytes {duplicates:02X?}"
                ));
            }
        }

        Ok(())
    }
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The checked-in specification, loaded once for every test that reads it.
#[cfg(test)]
pub(crate) fn shared() -> &'static Spec {
    static SPEC: std::sync::LazyLock<Spec> =
        std::sync::LazyLock::new(|| Spec::load(&crate::root()).expect("spec/ loads"));
    &SPEC
}

#[cfg(test)]
mod tests {
    use deepmind_midi::sysex::packed::packed_len;
    use deepmind_midi::sysex::{
        BANK_NAMES_LEN, CHORD_MEMORY_LEN, Command, Direction, GLOBAL_DATA_LEN, PATTERN_DATA_LEN,
        POLY_CHORD_MEMORY_LEN, PROGRAM_NAME_LEN,
    };

    use super::{shared as spec, version_parts};

    #[test]
    fn firmware_ranges_split_into_numbers() {
        assert_eq!(version_parts("1.1+"), (1, 1));
        assert_eq!(version_parts("1.0"), (1, 0));
    }

    /// Every modulation destination either names the parameters it moves or
    /// says why it names none.
    ///
    /// The blank and the not-yet-written look the same in the file, and this is
    /// what keeps them apart: a destination added without a mapping fails here
    /// rather than reaching a host as a silent empty answer.
    #[test]
    fn every_modulation_destination_is_accounted_for() {
        let spec = spec();
        let tables: Vec<_> = spec
            .tables
            .iter()
            .filter(|table| table.id == "mod_destination")
            .collect();
        assert_eq!(tables.len(), 2, "one destination table per firmware");
        for table in tables {
            for entry in &table.entries {
                assert!(
                    !entry.parameters.is_empty() || entry.description.is_some(),
                    "{}: {} ({}) names no parameter and does not say why",
                    table.name,
                    entry.value,
                    entry.name
                );
            }
        }
    }

    /// Fails when the library's command table drifts from `spec/messages.toml`.
    ///
    /// The library cannot read the spec files at runtime, so the table is
    /// written out by hand there. This is what keeps the two copies the same
    /// one.
    #[test]
    fn the_library_command_table_matches_the_spec() {
        let spec = spec();
        assert_eq!(
            spec.messages.len(),
            Command::ALL.len(),
            "messages.toml has {} commands, the library has {}",
            spec.messages.len(),
            Command::ALL.len()
        );
        for (message, command) in spec.messages.iter().zip(Command::ALL) {
            assert_eq!(
                command.to_byte(),
                message.command,
                "{} is command {:#04X} in the library and {:#04X} in the spec",
                message.name,
                command.to_byte(),
                message.command
            );
            assert_eq!(
                command.name(),
                message.name,
                "command {:#04X}",
                message.command
            );
            let direction = match message.direction.as_str() {
                "to_device" => Direction::ToDevice,
                "from_device" => Direction::FromDevice,
                other => panic!("messages.toml: unknown direction {other:?}"),
            };
            assert_eq!(command.direction(), direction, "{}", message.name);
        }
    }

    /// Fails when the drawn catalogues drift from what the library carries.
    ///
    /// The counts are generated, so a mismatch means a stale checkout; the
    /// glyph names are checked one by one because the enum's variants are
    /// what a host writes and a renamed glyph should be loud.
    #[test]
    fn the_library_catalogues_match_the_spec() {
        use deepmind_midi::effect::{Algorithm, CHARACTER_COUNT, VARIANT_COUNT};
        use deepmind_midi::pixels::Glyph;

        let spec = spec();
        assert_eq!(spec.glyphs.len(), Glyph::ALL.len());
        for (glyph, drawn) in Glyph::ALL.iter().zip(&spec.glyphs) {
            assert_eq!(glyph.name(), drawn.name);
        }
        assert_eq!(spec.characters.len(), CHARACTER_COUNT);
        assert_eq!(spec.variants.len(), VARIANT_COUNT);
        for effect in &spec.effects {
            let algorithm = Algorithm::by_name(&effect.name).expect("an algorithm per effect");
            assert_eq!(
                algorithm.own_mark().is_some(),
                spec.variant_of(&effect.name).is_some(),
                "{}",
                effect.name
            );
            assert_eq!(
                algorithm.characters().len(),
                spec.characters_of(&effect.name).len(),
                "{}",
                effect.name
            );
        }
    }

    /// Fails when a payload length the library hard-codes leaves the spec behind.
    #[test]
    fn the_library_payload_lengths_match_the_spec() {
        let spec = spec();
        let lengths = [
            (Command::GlobalParameterDumpResponse, GLOBAL_DATA_LEN),
            (Command::UserPatternDumpResponse, PATTERN_DATA_LEN),
            (Command::BankProgramNamesDumpResponse, BANK_NAMES_LEN),
            (Command::SingleProgramNameDumpResponse, PROGRAM_NAME_LEN),
            (Command::ChordMemoryDumpResponse, CHORD_MEMORY_LEN),
            (Command::PolyChordMemoryDumpResponse, POLY_CHORD_MEMORY_LEN),
        ];
        for (command, expected) in lengths {
            let message = spec
                .messages
                .iter()
                .find(|m| m.command == command.to_byte())
                .unwrap_or_else(|| panic!("{command} is not in messages.toml"));
            assert_eq!(
                message.raw_len.map(|len| len as usize),
                Some(expected),
                "{} carries {expected} unpacked bytes in the library",
                message.name
            );
        }
    }

    /// The manual's own packed lengths, checked against the codec.
    ///
    /// Five of the six agree exactly with padding the last group out to eight
    /// bytes, which is why the library packs that way. The sixth is the program
    /// dump, printed as 278 packed bytes for 242 raw where padding gives 280 and
    /// a short last group gives 277. It matches no rule and is recorded as an
    /// open question rather than reproduced.
    #[test]
    fn the_tabulated_packed_lengths_follow_the_padded_rule_bar_the_program_dump() {
        let mut checked = 0;
        for message in &spec().messages {
            let (Some(raw), Some(packed)) = (message.raw_len, message.packed_len) else {
                continue;
            };
            let (raw, packed) = (raw as usize, packed as usize);
            if raw == 242 {
                assert_eq!(packed, 278, "{}: the printed figure moved", message.name);
                assert_ne!(packed_len(raw), packed);
                continue;
            }
            assert_eq!(packed_len(raw), packed, "{}", message.name);
            checked += 1;
        }
        assert_eq!(checked, 6, "expected six figures to check");
    }
}
