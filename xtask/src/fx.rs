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

/// The colours one drawing uses, resolved from the three the spec measured.
struct Palette {
    body: String,
    knob: String,
    edge: String,
    ink: String,
    faint: String,
    accent: String,
}

impl Palette {
    /// Fills in what a measurement could not supply.
    ///
    /// Some panels are a single colour, so `panel` comes back equal to `body`
    /// and `accent` can land close to it. Where that happens the drawing shades
    /// the measured colour rather than substituting an invented one.
    fn resolve(layout: &Layout) -> Self {
        let body = Rgb::parse(&layout.body);
        let ink = if body.luminance() > 500 { BLACK } else { WHITE };
        let mut knob = Rgb::parse(&layout.panel);
        if knob.distance(body) < 60 {
            knob = body.mix(ink, 220);
        }
        let mut accent = Rgb::parse(&layout.accent);
        if accent.distance(body) < 90 || accent.distance(knob) < 70 {
            let against = if knob.luminance() > 500 { BLACK } else { WHITE };
            accent = knob.mix(against, 620);
        }
        Self {
            body: body.hex(),
            knob: knob.hex(),
            edge: knob.mix(ink, 350).hex(),
            ink: ink.mix(body, 80).hex(),
            faint: ink.mix(body, 550).hex(),
            accent: accent.hex(),
        }
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

/// Draws the arc a continuous control sweeps through.
fn track(cx: f32, cy: f32, radius: f32) -> String {
    let (x1, y1) = point(cx, cy, radius, -SWEEP);
    let (x2, y2) = point(cx, cy, radius, SWEEP);
    format!("M {x1:.2} {y1:.2} A {radius:.2} {radius:.2} 0 1 1 {x2:.2} {y2:.2}")
}

/// Draws the tick marks that say what kind of control this is.
///
/// A switch gets one at each end of the sweep, a selector one per option, and a
/// continuous control none: its arc track is already its whole range. They sit
/// inside the face, which is the only part of a cell nothing else uses.
fn ticks(cx: f32, cy: f32, radius: f32, count: usize, colour: &str) -> String {
    let mut out = String::new();
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
             stroke=\"{colour}\" stroke-width=\"0.7\" stroke-linecap=\"round\"/>"
        );
    }
    out
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
         fill=\"{}\">{}</text>\
         \n  <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.7\" \
         stroke-linecap=\"round\"/>\
         \n  <circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{radius:.2}\" fill=\"{}\" \
         stroke=\"{}\" stroke-width=\"0.6\"/>",
        top + REF,
        colours.ink,
        escape(&parameter.r#ref),
        track(cx, cy, radius + 1.8),
        colours.faint,
        colours.knob,
        colours.edge,
    );

    // Ticks say what kind of control this is. A selector shows one per option,
    // which is also how many positions the control has.
    let kind = presentation.map_or("continuous", |s| s.kind.as_str());
    let options = parameter
        .values
        .as_deref()
        .map_or(0, |values| values.split(',').count().min(9));
    let count = match kind {
        "switch" => 2,
        "selector" => options.max(2),
        _ => 0,
    };
    out.push_str(&ticks(cx, cy, radius, count, &colours.accent));

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

    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.1}\" \
         width=\"{:.0}\" height=\"{:.0}\" role=\"img\" \
         aria-label=\"{} front panel\" font-family=\"ui-sans-serif, system-ui, sans-serif\">\n\
         <title>{} ({})</title>\n\
         <rect width=\"{width:.0}\" height=\"{height:.1}\" fill=\"{}\"/>\n\
         <text x=\"4\" y=\"8.6\" font-size=\"5.6\" font-weight=\"600\" fill=\"{}\">{}</text>\n\
         <text x=\"{:.0}\" y=\"8.6\" font-size=\"4.2\" text-anchor=\"end\" fill=\"{}\">{}</text>\n\
         <rect x=\"4\" y=\"11.2\" width=\"{:.0}\" height=\"0.5\" fill=\"{}\"/>",
        width * 6.0,
        height * 6.0,
        escape(&effect.full_name),
        escape(&effect.full_name),
        escape(&effect.name),
        colours.body,
        colours.ink,
        escape(&effect.name),
        width - 4.0,
        colours.faint,
        escape(&layout.category),
        width - 8.0,
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
