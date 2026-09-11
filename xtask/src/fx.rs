//! Panel drawings for the effects, generated from `spec/`.
//!
//! One SVG per algorithm, laid out on the grid measured off the synthesizer's
//! own FX page and coloured from the panel the manual prints beside it. Both
//! come from `layout.toml`; the labels and control kinds come from
//! `effects.toml` and `panels.toml`. Nothing here is drawn by hand, so a
//! correction to the specification redraws all 35.
//!
//! The drawing is a map of the page, not a snapshot of a patch. Control
//! positions are deliberately absent: the specification has no default value
//! for an effect parameter, and a pointer sitting at some arbitrary angle would
//! claim one.

use std::fmt::Write as _;

use crate::spec::{Layout, LayoutRow, Spec};

/// Directory holding the drawings, relative to the repository root.
pub const DIR: &str = "docs/diagrams/fx";

/// Units of vertical space one label line takes.
const LINE: f32 = 4.2;
/// Point size of a label line.
const LABEL: f32 = 3.0;
/// Point size of a slot's short name.
const REF: f32 = 4.4;
/// Height of the title bar.
const HEADER: f32 = 13.0;
/// Space below the last row.
const FOOTER: f32 = 4.0;
/// Widest label line, in characters, before it wraps.
const WRAP: usize = 10;
/// Space between a slot's short name and its control.
const GAP: f32 = 3.0;
/// Longest sweep an arc track covers, in degrees either side of straight up.
const SWEEP: f32 = 135.0;

/// One effect's drawing.
#[derive(Debug)]
pub struct Drawing {
    /// File stem, used for `docs/diagrams/fx/<id>.svg`.
    pub id: String,
    /// Effect name, used as the figure's caption.
    pub title: String,
    /// The SVG document.
    pub source: String,
}

/// Draws every effect.
///
/// # Errors
///
/// Returns a message when an effect has no layout or no panel, which
/// `Spec::load` already rejects, or when a layout names a slot that the effect
/// does not have.
pub fn all(spec: &Spec) -> Result<Vec<Drawing>, String> {
    spec.effects
        .iter()
        .map(|effect| {
            let layout = spec
                .layouts
                .iter()
                .find(|l| l.r#type == effect.r#type)
                .ok_or_else(|| format!("no layout for {}", effect.name))?;
            Ok(Drawing {
                id: slug(&effect.name),
                title: effect.full_name.clone(),
                source: draw(spec, layout)?,
            })
        })
        .collect()
}

/// Returns the file stem for an effect name.
#[must_use]
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// A colour as three channels, so it can be measured and mixed.
///
/// Held as bytes and mixed in thousandths, which keeps the arithmetic exact and
/// the output byte-for-byte reproducible.
#[derive(Clone, Copy)]
struct Rgb(u8, u8, u8);

impl Rgb {
    /// Parses `#rrggbb`. `Spec::load` has already checked the shape.
    fn parse(hex: &str) -> Self {
        let byte =
            |at: usize| u8::from_str_radix(hex.get(at..at + 2).unwrap_or("00"), 16).unwrap_or(0);
        Self(byte(1), byte(3), byte(5))
    }

    /// Relative luminance, 0 for black and 1000 for white.
    fn luminance(self) -> u32 {
        (2126 * u32::from(self.0) + 7152 * u32::from(self.1) + 722 * u32::from(self.2)) / 2550
    }

    /// Moves this colour towards `other` by `amount` thousandths.
    fn mix(self, other: Self, amount: u32) -> Self {
        let lerp = |a: u8, b: u8| {
            let blended = (u32::from(a) * (1000 - amount) + u32::from(b) * amount) / 1000;
            u8::try_from(blended).unwrap_or(u8::MAX)
        };
        Self(
            lerp(self.0, other.0),
            lerp(self.1, other.1),
            lerp(self.2, other.2),
        )
    }

    /// How far apart two colours are, summed over the three channels.
    fn distance(self, other: Self) -> u32 {
        let gap = |a: u8, b: u8| u32::from(a.abs_diff(b));
        gap(self.0, other.0) + gap(self.1, other.1) + gap(self.2, other.2)
    }

    fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

const WHITE: Rgb = Rgb(255, 255, 255);
const BLACK: Rgb = Rgb(0, 0, 0);

/// The colours one drawing uses, resolved from the four the spec measured.
struct Palette {
    chassis: String,
    face: String,
    cap: String,
    edge: String,
    ink: String,
    faint: String,
    accent: String,
}

impl Palette {
    /// Fills in what a measurement could not supply.
    ///
    /// A panel built from one colour measures as one colour, so `cap` can come
    /// back equal to `face` and `accent` equal to `cap`. Where that happens the
    /// drawing shades the measured colour rather than substituting an invented
    /// one, so the page still reads as the panel it came from.
    fn resolve(layout: &Layout) -> Self {
        let chassis = Rgb::parse(&layout.chassis);
        let face = Rgb::parse(&layout.face);
        let ink = if face.luminance() > 500 { BLACK } else { WHITE };
        let mut cap = Rgb::parse(&layout.cap);
        if cap.distance(face) < 60 {
            cap = face.mix(ink, 220);
        }
        let mut accent = Rgb::parse(&layout.accent);
        if accent.distance(face) < 90 || accent.distance(cap) < 70 {
            let against = if cap.luminance() > 500 { BLACK } else { WHITE };
            accent = cap.mix(against, 620);
        }
        // The case reads as a case only when it is not the surface it holds.
        let chassis = if chassis.distance(face) < 40 {
            face.mix(ink, 120)
        } else {
            chassis
        };
        Self {
            chassis: chassis.hex(),
            face: face.hex(),
            cap: cap.hex(),
            edge: cap.mix(ink, 350).hex(),
            ink: ink.mix(face, 80).hex(),
            faint: ink.mix(face, 550).hex(),
            accent: accent.hex(),
        }
    }

    /// Text that reads on the chassis rather than on the face.
    fn on_chassis(layout: &Layout) -> (String, String) {
        let chassis = Rgb::parse(&layout.chassis);
        let ink = if chassis.luminance() > 500 {
            BLACK
        } else {
            WHITE
        };
        (ink.mix(chassis, 80).hex(), ink.mix(chassis, 500).hex())
    }
}

/// Returns a count as a length. Counts here are slots and label lines, so a
/// byte holds every value one can take.
fn units(count: usize) -> f32 {
    f32::from(u8::try_from(count).unwrap_or(u8::MAX))
}

/// Splits a title into label lines of at most `WRAP` characters.
fn wrap(title: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in title.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= WRAP => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

/// Escapes the characters that cannot appear in SVG text.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Returns a point on a circle, with 0 degrees straight up and positive clockwise.
fn point(cx: f32, cy: f32, radius: f32, degrees: f32) -> (f32, f32) {
    let radians = degrees.to_radians();
    (cx + radius * radians.sin(), cy - radius * radians.cos())
}

/// Draws the arc a knob sweeps through.
fn track(cx: f32, cy: f32, radius: f32) -> String {
    let (x1, y1) = point(cx, cy, radius, -SWEEP);
    let (x2, y2) = point(cx, cy, radius, SWEEP);
    format!("M {x1:.2} {y1:.2} A {radius:.2} {radius:.2} 0 1 1 {x2:.2} {y2:.2}")
}

/// How many marks a control carries, and in which colour.
///
/// A switch gets one mark per state and a selector one per option, so the marks
/// count the positions the control has. A continuous control gets none: its
/// travel is already drawn, and a mark on it would mean a detent that is not
/// there.
fn marks(kind: &str, options: usize) -> usize {
    match kind {
        "switch" => 2,
        "selector" => options.max(2),
        _ => 0,
    }
}

/// Draws a knob: the body, the travel behind it, and any position marks.
///
/// The marks sit inside the face, which is the only part of a cell nothing else
/// uses.
fn knob(cx: f32, cy: f32, radius: f32, count: usize, colours: &Palette) -> String {
    let mut out = format!(
        "\n  <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.7\" \
         stroke-linecap=\"round\"/>\
         \n  <circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{radius:.2}\" fill=\"{}\" \
         stroke=\"{}\" stroke-width=\"0.6\"/>",
        track(cx, cy, radius + 1.8),
        colours.faint,
        colours.cap,
        colours.edge,
    );
    for index in 0..count {
        let fraction = if count == 1 {
            0.5
        } else {
            units(index) / units(count - 1)
        };
        let angle = -SWEEP + fraction * 2.0 * SWEEP;
        let (x1, y1) = point(cx, cy, radius - 2.4, angle);
        let (x2, y2) = point(cx, cy, radius - 0.9, angle);
        let _ = write!(
            out,
            "\n  <line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" \
             stroke=\"{}\" stroke-width=\"0.7\" stroke-linecap=\"round\"/>",
            colours.accent
        );
    }
    out
}

/// Draws a vertical fader: the slot, the ladder beside it, and any marks.
///
/// The panels that use faders draw a ladder of rungs the length of the travel,
/// so the rungs are the travel here too. Marked positions replace them, in the
/// accent colour, which keeps a switch and a selector readable at this size.
fn fader(cx: f32, cy: f32, radius: f32, count: usize, colours: &Palette) -> String {
    let half = radius * 1.15;
    let slot = 0.9;
    let reach = radius * 1.0;
    let mut out = format!(
        "\n  <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{slot:.2}\" height=\"{:.2}\" \
         rx=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>",
        cx - slot / 2.0,
        cy - half,
        half * 2.0,
        slot / 2.0,
        colours.cap,
        colours.edge,
    );
    let rungs = if count == 0 { 7 } else { count };
    let colour = if count == 0 {
        &colours.faint
    } else {
        &colours.accent
    };
    for index in 0..rungs {
        let fraction = if rungs == 1 {
            0.5
        } else {
            units(index) / units(rungs - 1)
        };
        let y = cy + half - fraction * half * 2.0;
        let _ = write!(
            out,
            "\n  <line x1=\"{:.2}\" y1=\"{y:.2}\" x2=\"{:.2}\" y2=\"{y:.2}\" \
             stroke=\"{colour}\" stroke-width=\"0.6\" stroke-linecap=\"round\"/>\
             \n  <line x1=\"{:.2}\" y1=\"{y:.2}\" x2=\"{:.2}\" y2=\"{y:.2}\" \
             stroke=\"{colour}\" stroke-width=\"0.6\" stroke-linecap=\"round\"/>",
            cx - reach,
            cx - slot * 1.8,
            cx + slot * 1.8,
            cx + reach,
        );
    }
    out
}

/// Draws a numeric readout: the bezel and the lit field, with no value in it.
fn display(cx: f32, cy: f32, radius: f32, colours: &Palette) -> String {
    let w = radius * 1.42;
    let h = radius * 0.8;
    format!(
        "\n  <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"1\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"0.6\"/>\
         \n  <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"0.8\" rx=\"0.4\" \
         fill=\"{}\" opacity=\"0.85\"/>",
        cx - w,
        cy - h,
        w * 2.0,
        h * 2.0,
        colours.cap,
        colours.edge,
        cx - w * 0.6,
        cy - 0.4,
        w * 1.2,
        colours.accent,
    )
}

/// Draws one row of controls, honouring what the row says about alignment.
///
/// The measured grid is left aligned, so every row in `layout.toml` simply
/// stops where it runs out of slots. The other alignments are here because a
/// host with a panel of a different width may want one.
fn draw_row(
    out: &mut String,
    spec: &Spec,
    layout: &Layout,
    colours: &Palette,
    row: &LayoutRow,
    top: f32,
) -> Result<(), String> {
    let grid = &spec.grid;
    let free = grid.display_width - 2.0 * grid.first_x - units(row.slots.len()) * grid.column_pitch
        + grid.column_pitch;
    let shift = match row.align.as_str() {
        "right" => free,
        "centre" => free * 0.5,
        _ => 0.0,
    };
    let pitch = if row.align == "spread" && row.slots.len() > 1 {
        (grid.display_width - 2.0 * grid.first_x) / units(row.slots.len() - 1)
    } else {
        grid.column_pitch
    };
    for (column, slot) in row.slots.iter().enumerate() {
        let cx = grid.first_x + shift + units(column) * pitch;
        draw_slot(out, spec, layout, colours, *slot, cx, top)?;
    }
    Ok(())
}

/// Draws one slot: its short name, its control, and its name in full.
fn draw_slot(
    out: &mut String,
    spec: &Spec,
    layout: &Layout,
    colours: &Palette,
    slot: u8,
    cx: f32,
    top: f32,
) -> Result<(), String> {
    let grid = &spec.grid;
    let radius = grid.control_diameter / 2.0;
    let cy = top + REF + GAP + radius;
    let effect = spec
        .effects
        .iter()
        .find(|e| e.r#type == layout.r#type)
        .ok_or_else(|| format!("no effect for layout {}", layout.name))?;
    let parameter = effect
        .parameters
        .iter()
        .find(|p| p.slot == slot)
        .ok_or_else(|| format!("{} has no slot {slot}", layout.name))?;
    let presentation = spec
        .panels
        .iter()
        .find(|p| p.r#type == layout.r#type)
        .and_then(|p| p.slots.iter().find(|s| s.slot == slot));

    let _ = write!(
        out,
        "\n  <text x=\"{cx:.2}\" y=\"{:.2}\" font-size=\"{REF}\" text-anchor=\"middle\" \
         font-family=\"ui-monospace, SFMono-Regular, Menlo, monospace\" \
         fill=\"{}\">{}</text>",
        top + REF,
        colours.ink,
        escape(&parameter.r#ref),
    );

    // The shape is the one the effect's own editor panel uses. Switches and
    // selectors keep it and carry their positions as marks, which is what the
    // panels do too.
    let kind = presentation.map_or("continuous", |s| s.kind.as_str());
    let options = parameter
        .values
        .as_deref()
        .map_or(0, |values| values.split(',').count().min(9));
    let count = marks(kind, options);
    out.push_str(&match layout.control.as_str() {
        "fader" => fader(cx, cy, radius, count, colours),
        "display" if count == 0 => display(cx, cy, radius, colours),
        _ => knob(cx, cy, radius, count, colours),
    });

    if parameter.mod_dest {
        let (mx, my) = point(cx, cy, radius + 3.4, 42.0);
        let _ = write!(
            out,
            "\n  <circle cx=\"{mx:.2}\" cy=\"{my:.2}\" r=\"0.9\" fill=\"{}\"/>",
            colours.accent
        );
    }

    let title = presentation.map_or_else(|| parameter.name.clone(), |s| s.title.clone());
    for (line_index, line) in wrap(&title).iter().enumerate() {
        let squeeze = if line.chars().count() > WRAP {
            format!(
                " textLength=\"{:.1}\" lengthAdjust=\"spacingAndGlyphs\"",
                grid.column_pitch - 3.0
            )
        } else {
            String::new()
        };
        let _ = write!(
            out,
            "\n  <text x=\"{cx:.2}\" y=\"{:.2}\" font-size=\"{LABEL}\" \
             text-anchor=\"middle\" fill=\"{}\"{squeeze}>{}</text>",
            cy + radius + 4.6 + units(line_index) * LINE,
            colours.faint,
            escape(line),
        );
    }
    Ok(())
}

/// Draws one effect's page.
fn draw(spec: &Spec, layout: &Layout) -> Result<String, String> {
    let effect = spec
        .effects
        .iter()
        .find(|e| e.r#type == layout.r#type)
        .ok_or_else(|| format!("no effect for layout {}", layout.name))?;
    let panel = spec.panels.iter().find(|p| p.r#type == layout.r#type);
    let grid = &spec.grid;
    let colours = Palette::resolve(layout);

    // Every row is as tall as the longest title on the page needs, so the
    // circles stay on one grid however long the words are.
    let deepest = effect
        .parameters
        .iter()
        .map(|p| {
            panel
                .and_then(|panel| panel.slots.iter().find(|s| s.slot == p.slot))
                .map_or(1, |slot| wrap(&slot.title).len())
        })
        .max()
        .unwrap_or(1);
    let row_height = REF + GAP + grid.control_diameter + 2.0 + units(deepest) * LINE;
    let width = grid.display_width;
    let height = HEADER + units(layout.rows.len()) * row_height + FOOTER;

    // A rack unit: the case, then the surface the controls sit on inside it.
    let (chassis_ink, chassis_faint) = Palette::on_chassis(layout);
    let inset = 2.5;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.1}\" \
         width=\"{:.0}\" height=\"{:.0}\" role=\"img\" \
         aria-label=\"{} front panel\" font-family=\"ui-sans-serif, system-ui, sans-serif\">\n\
         <title>{} ({})</title>\n\
         <rect width=\"{width:.0}\" height=\"{height:.1}\" fill=\"{}\"/>\n\
         <text x=\"{inset:.1}\" y=\"8.4\" font-size=\"5.6\" font-weight=\"600\" \
         fill=\"{chassis_ink}\">{}</text>\n\
         <text x=\"{:.1}\" y=\"8.4\" font-size=\"4.2\" text-anchor=\"end\" \
         fill=\"{chassis_faint}\">{}</text>\n\
         <rect x=\"{inset:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"1.5\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"0.4\"/>",
        width * 6.0,
        height * 6.0,
        escape(&effect.full_name),
        escape(&effect.full_name),
        escape(&effect.name),
        colours.chassis,
        escape(&effect.name),
        width - inset,
        escape(&layout.category),
        HEADER - 2.0,
        width - inset * 2.0,
        height - HEADER,
        colours.face,
        colours.accent,
    );

    for (index, row) in layout.rows.iter().enumerate() {
        let top = HEADER + units(index) * row_height;
        draw_row(&mut out, spec, layout, &colours, row, top)?;
    }

    out.push_str("\n</svg>\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{slug, wrap};

    #[test]
    fn slugs_are_url_safe() {
        assert_eq!(slug("TC-DeepVRB"), "tc-deepvrb");
        assert_eq!(slug("Vintage Pitch"), "vintage-pitch");
        assert_eq!(slug("3TapDelay"), "3tapdelay");
    }

    #[test]
    fn titles_wrap_on_words() {
        assert_eq!(wrap("Mix"), ["Mix"]);
        assert_eq!(wrap("High Shelf Frequency"), ["High Shelf", "Frequency"]);
        assert_eq!(
            wrap("Early Reflection Level"),
            ["Early", "Reflection", "Level"]
        );
        assert_eq!(wrap("Semitones, channel 2"), ["Semitones,", "channel 2"]);
    }
}
