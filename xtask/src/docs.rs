//! Generates the reference tables in `docs/midi-spec.md` from `spec/`.
//!
//! Prose in that file is written by hand. Tables live between marker comments
//! and are owned by this module:
//!
//! ```text
//! <!-- generated:parameters -->
//! ...replaced on every run...
//! <!-- /generated:parameters -->
//! ```
//!
//! `cargo xtask docs` rewrites the marked regions; `cargo xtask docs --check`
//! reports staleness without touching the file. The `generated_documentation_is_current`
//! test runs the same comparison, so a stale checkout fails `cargo test`.

use std::fmt::Write as _;
use std::path::Path;

use crate::output::Output;
use crate::spec::Spec;
use crate::{diagrams, fx};

/// Path of the protocol document, relative to the repository root.
pub const DOC_PATH: &str = "docs/midi-spec.md";

/// Path of the effects document, relative to the repository root.
pub const EFFECTS_PATH: &str = "docs/effects.md";

/// Loads the spec, then renders the documentation as [`generate`].
///
/// # Errors
///
/// As [`generate`], or a message when the spec cannot be loaded.
pub fn run(root: &Path, check: bool) -> Result<Vec<String>, String> {
    generate(&Spec::load(root)?, root, check)
}

/// Renders the marked regions and the diagrams, and compares them against the
/// files on disk.
///
/// Writes each file that differs unless `check` is set, and returns the paths,
/// relative to the root, of the files that differed: the two documents, the
/// diagram sources under `docs/diagrams/`, and the effect drawings under
/// `docs/diagrams/fx/`. A diagram or drawing left behind by something renamed
/// is removed, and counts as a stale path too.
///
/// # Errors
///
/// Returns a message when a document is missing, a marker pair is missing or
/// malformed, or a file cannot be written.
pub fn generate(spec: &Spec, root: &Path, check: bool) -> Result<Vec<String>, String> {
    let diagrams = diagrams::all(spec)?;
    let drawings = fx::all(spec)?;

    let mut output = Output::new(root, check);
    let sources: Vec<(String, String)> = diagrams
        .iter()
        .map(|d| (d.file_name(), d.file_contents()))
        .collect();
    output.sync_dir(diagrams::DIR, &sources, &["mmd", "svg"])?;
    let panels: Vec<(String, &str)> = drawings
        .iter()
        .map(|d| (format!("{}.svg", d.id), d.source.as_str()))
        .collect();
    output.sync_dir(fx::DIR, &panels, &["svg"])?;

    for (path, sections) in [
        (
            DOC_PATH,
            vec![
                ("structure", render_structure(spec, &diagrams)?),
                ("messages", render_messages(spec)),
                ("parameters", render_parameters(spec)),
                ("firmware", render_firmware(spec)),
                ("mapping", render_mapping(spec)),
                ("value-tables", render_value_tables(spec)),
                ("globals", render_globals(spec)),
                ("controllers", render_controllers(spec)),
                ("front-panel", render_front(spec)),
                ("routing", render_routing(spec)),
                ("measurements", render_measurements(spec)),
                ("corrections", render_corrections(spec)),
            ],
        ),
        (
            EFFECTS_PATH,
            vec![
                ("grid", render_grid(spec)),
                ("glyphs", render_glyphs(spec)),
                ("characters", render_characters(spec)),
                ("effect-index", render_effect_index(spec)),
                ("effects", render_effects(spec, &drawings)),
                ("effect-corrections", render_effect_corrections(spec)),
            ],
        ),
    ] {
        let file = root.join(path);
        let mut document =
            std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        for (marker, body) in sections {
            document = splice(path, &document, marker, &body)?;
        }
        output.write(path, &document)?;
    }

    Ok(output.stale())
}

/// Replaces the text between `<!-- generated:NAME -->` and `<!-- /generated:NAME -->`.
fn splice(path: &str, document: &str, marker: &str, body: &str) -> Result<String, String> {
    let open = format!("<!-- generated:{marker} -->");
    let close = format!("<!-- /generated:{marker} -->");
    let start = document
        .find(&open)
        .ok_or_else(|| format!("{path}: missing marker {open}"))?
        + open.len();
    let end = document
        .find(&close)
        .ok_or_else(|| format!("{path}: missing marker {close}"))?;
    if end < start {
        return Err(format!("{path}: marker {close} precedes {open}"));
    }
    Ok(format!(
        "{}\n\n{}\n\n{}",
        &document[..start],
        body.trim(),
        &document[end..]
    ))
}

/// Escapes the characters that would otherwise break out of a table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

/// Renders the structure section: each diagram inline, with the prose around it.
///
/// The sources are the same strings written to `docs/diagrams`, so the document
/// and the standalone files always agree.
///
/// # Errors
///
/// Returns a message when a diagram is missing from the generated set, or when
/// a parameter group the prose describes is missing a parameter it names.
fn render_structure(spec: &Spec, all: &[diagrams::Diagram]) -> Result<String, String> {
    let diagram = |id: &str| -> Result<&diagrams::Diagram, String> {
        all.iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("missing diagram {id}"))
    };
    // A Mermaid diagram is inlined; an SVG is referenced where it was written.
    let embed = |id: &str| -> Result<String, String> {
        let d = diagram(id)?;
        Ok(match d.kind {
            diagrams::Kind::Mermaid => format!("```mermaid\n{}\n```\n", d.source),
            diagrams::Kind::Svg => format!(
                "<img src=\"{}/{}\" alt=\"{}\" width=\"640\">\n",
                diagrams::DIR.trim_start_matches("docs/"),
                d.file_name(),
                d.title
            ),
        })
    };

    let mut out = String::new();
    let _ = writeln!(out, "### {}\n", diagram("signal-path")?.title);
    out.push_str(
        "Each block is a parameter group, labelled with its offsets. Solid \
         arrows carry audio, dashed arrows carry modulation.\n\n",
    );
    let _ = writeln!(out, "{}", embed("signal-path")?);
    out.push_str(
        "Offsets 40 and 52, the high pass frequency and the bass boost, appear \
         twice on purpose. The front panel groups them with the VCF and this \
         specification follows the panel, but the block diagram in section 6 of \
         the manual places both after the VCA, where they act on the mixed voices \
         rather than on one. The analog path skips the FX block; the \
         [`FX Mode`](#fx-routing) parameter decides which paths carry signal.\n\n",
    );

    let _ = writeln!(out, "### {}\n", diagram("modulation-matrix")?.title);
    out.push_str("Eight independent busses, each a source, a destination and a signed depth.\n\n");
    let _ = writeln!(out, "{}", embed("modulation-matrix")?);

    // The first bus is the group's first three offsets and the last bus its
    // last three.
    let busses: Vec<u16> = spec
        .parameters
        .iter()
        .filter(|p| p.group == "Mod Matrix")
        .map(|p| p.offset)
        .collect();
    let (Some(first), Some(last)) = (busses.first_chunk::<3>(), busses.last_chunk::<3>()) else {
        return Err("no parameters in group Mod Matrix".to_owned());
    };
    let _ = writeln!(
        out,
        "Each bus occupies three consecutive offsets: source, destination, depth. \
         Bus 1 is {}, {}, {}, and bus {} is {}, {}, {}.\n",
        first[0],
        first[1],
        first[2],
        busses.len() / 3,
        last[0],
        last[1],
        last[2],
    );

    let _ = writeln!(out, "### {}\n", diagram("envelope")?.title);
    out.push_str(
        "Three identical envelopes: VCA, VCF and mod. Each has the four \
         stages plus a curve control per stage, which bends the segment \
         from linear towards exponential in either direction.\n\n",
    );
    let _ = writeln!(out, "{}", embed("envelope")?);
    out.push_str("| Stage | VCA | VCF | Mod |\n|---|---|---|---|\n");
    // Each envelope's group names its parameters `<group> <stage>`, so a stage
    // is found by the end of the name.
    let stage = |group: &str, stage: &str| -> Result<u16, String> {
        spec.parameters
            .iter()
            .filter(|p| p.group == group)
            .find(|p| p.name.to_lowercase().ends_with(&stage.to_lowercase()))
            .map(|p| p.offset)
            .ok_or_else(|| format!("no {stage} parameter in group {group}"))
    };
    for name in [
        "Attack time",
        "Decay time",
        "Sustain level",
        "Release time",
        "Attack curve",
        "Decay curve",
        "Sustain curve",
        "Release curve",
    ] {
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} |",
            stage("VCA Envelope", name)?,
            stage("VCF Envelope", name)?,
            stage("Mod Envelope", name)?,
        );
    }

    Ok(out)
}

fn render_messages(spec: &Spec) -> String {
    let mut out = String::from("| Cmd | Message | Direction | Payload | Raw | Packed |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    for message in &spec.messages {
        let direction = if message.direction == "to_device" {
            "host to synth"
        } else {
            "synth to host"
        };
        let raw = message.raw_len.map_or_else(String::new, |n| n.to_string());
        let packed = message
            .packed_len
            .map_or_else(String::new, |n| n.to_string());
        let _ = writeln!(
            out,
            "| `{:02X}` | {} | {direction} | {} | {raw} | {packed} |",
            message.command,
            cell(&message.name),
            cell(message.payload.as_deref().unwrap_or("none")),
        );
    }

    let notes: Vec<&crate::spec::Message> =
        spec.messages.iter().filter(|m| m.note.is_some()).collect();
    if !notes.is_empty() {
        out.push('\n');
        for message in notes {
            let note = message.note.as_deref().unwrap_or_default().trim();
            let _ = writeln!(out, "**{}.** {note}\n", message.name);
        }
    }
    out
}

fn render_parameters(spec: &Spec) -> String {
    let mut out = String::new();
    let mut group = "";
    for parameter in &spec.parameters {
        if parameter.group != group {
            group = &parameter.group;
            let _ = write!(
                out,
                "\n### {group}\n\n\
                 | Offset | Parameter | Raw | Values | Shows as | Glyph | What it does |\n\
                 |---|---|---|---|---|---|---|\n"
            );
        }
        let values = match (&parameter.kind, &parameter.value_table, &parameter.note) {
            (Some(kind), _, _) if kind == "switch" => "Off (0), On (1)".to_owned(),
            (_, Some(id), _) => {
                let name = spec.table(id).map_or(id.as_str(), |t| t.name.as_str());
                format!("[{name}](#{id})")
            }
            (_, _, Some(note)) => cell(note),
            _ => String::new(),
        };
        let values = if parameter.confirmed {
            values
        } else {
            format!("**Unconfirmed.** {values}")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {}-{} | {values} | {} | {} | {} |",
            parameter.offset,
            cell(&parameter.name),
            parameter.min,
            parameter.max,
            cell(parameter.display.as_deref().unwrap_or("")),
            glyph_link(parameter.glyph.as_deref()),
            cell(parameter.description.as_deref().unwrap_or("")),
        );
    }
    out
}

/// Renders each transport's byte pattern, its fields, and the encodings.
fn render_mapping(spec: &Spec) -> String {
    let mut out = String::new();
    for transport in &spec.transports {
        let _ = write!(
            out,
            "\n### {}\n\n{}\n\n```\n{}\n```\n\n",
            cell(&transport.name),
            transport.summary,
            transport.pattern
        );
        if let Some(note) = &transport.note {
            let _ = write!(out, "{note}\n\n");
        }
        out.push_str("| Field | From | Encoding | Bits | Notes |\n|---|---|---|---|---|\n");
        for field in &transport.fields {
            let note = cell(field.note.as_deref().unwrap_or(""));
            let notes = if field.optional {
                let when = cell(field.optional_when.as_deref().unwrap_or(""));
                format!("Optional. {when} {note}").trim().to_owned()
            } else {
                note
            };
            let _ = writeln!(
                out,
                "| `{}` | `{}` | {} | {} | {notes} |",
                cell(&field.name),
                cell(&field.source),
                field
                    .encoding
                    .as_deref()
                    .map_or_else(|| "-".to_owned(), |id| format!("[{id}](#encodings)")),
                field.bits.map_or_else(|| "-".to_owned(), |b| b.to_string()),
            );
        }
    }

    out.push_str("\n<a id=\"encodings\"></a>\n\n### Encodings\n\n");
    out.push_str("| Encoding | Name | Rule | Notes |\n|---|---|---|---|\n");
    for encoding in &spec.encodings {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} |",
            cell(&encoding.id),
            cell(&encoding.name),
            cell(&encoding.rule),
            cell(encoding.note.as_deref().unwrap_or(""))
        );
    }
    out
}

/// Renders the firmware list and what each version changed.
fn render_firmware(spec: &Spec) -> String {
    let default = spec.default_firmware();
    let mut out = String::from("| Version | Notes |\n|---|---|\n");
    for firmware in &spec.firmwares {
        let marker = if firmware.version == default {
            " (assumed by default)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "| {}{marker} | {} |",
            cell(&firmware.version),
            cell(firmware.note.as_deref().unwrap_or(""))
        );
    }
    out
}

/// Anchor for a value table, kept unique when firmware split it in two.
fn table_anchor(table: &crate::spec::ValueTable, default: &str) -> String {
    match &table.firmware {
        Some(range) if !crate::spec::Firmware::range_covers(Some(range), default) => {
            format!("{}-fw{}", table.id, range.replace(['.', '+'], ""))
        }
        _ => table.id.clone(),
    }
}

fn render_value_tables(spec: &Spec) -> String {
    let default = spec.default_firmware();
    let mut out = String::new();
    for table in &spec.tables {
        let _ = write!(
            out,
            "\n<a id=\"{}\"></a>\n\n#### {}\n\n",
            table_anchor(table, default),
            table.name
        );
        if let Some(range) = &table.firmware {
            let _ = write!(out, "Firmware {range}.\n\n");
        }
        if let Some(note) = &table.note {
            let _ = write!(out, "{note}\n\n");
        }
        if !table.confirmed {
            out.push_str(
                "> Unconfirmed. This mapping is inferred and needs checking against hardware.\n\n",
            );
        }
        // The cells are drawn once, beside the table the newest firmware reads.
        if crate::spec::Firmware::range_covers(table.firmware.as_deref(), default)
            && table
                .entries
                .iter()
                .any(|entry| spec.cell_for(table, entry).is_some())
        {
            let _ = write!(
                out,
                "<img src=\"diagrams/cells.svg\" alt=\"{}\" width=\"{}\">\n\n",
                diagrams::CELLS_ALT,
                diagrams::cells_width(spec)
            );
        }
        // Only the modulation destinations join to the parameter table, and
        // only the sources swing, so each column appears where there is
        // something in it rather than as an empty one on all twenty-odd tables.
        let joins = table
            .entries
            .iter()
            .any(|entry| !entry.parameters.is_empty());
        let swings = table.entries.iter().any(|entry| entry.swing.is_some());
        let mut header = String::from("| Value | Name |");
        if joins {
            header.push_str(" Moves |");
        }
        if swings {
            header.push_str(" Swing |");
        }
        header.push_str(" Notes |");
        let rule = "|---".repeat(header.matches('|').count() - 1) + "|";
        let _ = writeln!(out, "{header}\n{rule}");
        for entry in &table.entries {
            let mut row = format!("| {} | {} |", entry.value, cell(&entry.name));
            if joins {
                let _ = write!(row, " {} |", cell(&entry.parameters.join(", ")));
            }
            if swings {
                let _ = write!(row, " {} |", entry.swing.as_deref().unwrap_or(""));
            }
            let _ = writeln!(
                out,
                "{row} {} |",
                cell(entry.description.as_deref().unwrap_or(""))
            );
        }
    }
    out
}

fn render_globals(spec: &Spec) -> String {
    let mut out = String::from("| Setting | Range | Notes |\n|---|---|---|\n");
    for global in &spec.globals {
        let mut notes = cell(global.note.as_deref().unwrap_or(""));
        if !global.confirmed {
            notes = format!("**Unconfirmed.** {notes}");
        }
        let _ = writeln!(
            out,
            "| {} | {}-{} | {notes} |",
            cell(&global.name),
            global.min,
            global.max
        );
    }
    out
}

fn render_controllers(spec: &Spec) -> String {
    let mut out = String::new();
    for (kind, heading, blurb) in [
        (
            "parameter",
            "Program parameters",
            "Each of these drives one program parameter. The offset column is that \
             parameter's entry in the table above.",
        ),
        (
            "standard",
            "Standard controllers",
            "Ordinary MIDI controllers, answered in the usual way.",
        ),
        (
            "other",
            "Everything else",
            "Controllers that do something but are not a single program parameter.",
        ),
    ] {
        let rows: Vec<&crate::spec::Controller> =
            spec.controllers.iter().filter(|c| c.kind == kind).collect();
        if rows.is_empty() {
            continue;
        }

        let _ = write!(out, "\n#### {heading}\n\n{blurb}\n\n| CC | Controls |");
        if kind == "parameter" {
            out.push_str(" Offset |");
        }
        out.push_str(" Glyph | Notes |\n|---|---|");
        if kind == "parameter" {
            out.push_str("---|");
        }
        out.push_str("---|---|\n");

        for controller in rows {
            let mut notes = cell(controller.note.as_deref().unwrap_or(""));
            if !controller.confirmed {
                notes = format!("**Unconfirmed.** {notes}");
            }
            let _ = write!(out, "| {} | {} |", controller.cc, cell(&controller.name));
            if kind == "parameter" {
                let _ = write!(
                    out,
                    " {} |",
                    controller
                        .parameter
                        .map_or_else(|| "-".to_owned(), |o| o.to_string())
                );
            }
            // A controller that drives a parameter is pictured as the
            // parameter is, which is what the library's table carries too.
            let glyph = controller.glyph.as_deref().or_else(|| {
                let offset = usize::from(controller.parameter?);
                spec.parameters.get(offset)?.glyph.as_deref()
            });
            let _ = writeln!(out, " {} | {notes} |", glyph_link(glyph));
        }
    }
    out
}

/// Renders a glyph's name as a link to its row in the effects document's
/// catalogue, or nothing for a parameter that has none.
fn glyph_link(name: Option<&str>) -> String {
    name.map_or_else(String::new, |name| {
        format!("[{name}](effects.md#glyph-{})", glyph_anchor(name))
    })
}

/// The anchor a glyph's row in the catalogue carries.
fn glyph_anchor(name: &str) -> String {
    name.replace(' ', "-")
}

/// Renders the glyph catalogue: the drawing of every glyph, then a row per
/// glyph saying what it pictures and how many slots and parameters carry it.
fn render_glyphs(spec: &Spec) -> String {
    let mut out = format!(
        "<img src=\"diagrams/glyphs.svg\" alt=\"{}\" width=\"{}\">\n\n\
         | Glyph | Picture of | Effect slots | Parameters | Controllers |\n|---|---|---|---|---|\n",
        diagrams::GLYPHS_ALT,
        diagrams::glyphs_width(spec)
    );
    for glyph in &spec.glyphs {
        let slots = spec
            .panels
            .iter()
            .flat_map(|panel| panel.slots.iter())
            .filter(|slot| slot.glyph == glyph.name)
            .count();
        let parameters = spec
            .parameters
            .iter()
            .filter(|parameter| parameter.glyph.as_deref() == Some(glyph.name.as_str()))
            .count();
        let controllers = spec
            .controllers
            .iter()
            .filter(|controller| controller.glyph.as_deref() == Some(glyph.name.as_str()))
            .count();
        let _ = writeln!(
            out,
            "| <a id=\"glyph-{}\"></a>`{}` | {} | {} | {} | {} |",
            glyph_anchor(&glyph.name),
            glyph.name,
            cell(&glyph.description),
            slots,
            parameters,
            controllers
        );
    }
    out
}

/// Renders the characters: what each means, and which algorithms have it,
/// each with the reason it is listed.
fn render_characters(spec: &Spec) -> String {
    let mut out = String::new();
    for character in &spec.characters {
        let _ = write!(
            out,
            "\n#### {}\n\n{}\n\n| Algorithm | Because |\n|---|---|\n",
            character.name,
            cell(&character.description)
        );
        for member in &character.algorithms {
            let effect = spec.effects.iter().find(|e| e.name == member.name);
            let link = effect.map_or_else(
                || cell(&member.name),
                |effect| {
                    format!(
                        "[{}](#{})",
                        cell(&member.name),
                        fx::stem(effect.r#type, &effect.full_name)
                    )
                },
            );
            let _ = writeln!(out, "| {link} | {} |", cell(&member.because));
        }
    }
    out
}

/// Renders the front panel: which parameters the instrument puts a control
/// under, what is printed over each one, and which row its plate is in.
///
/// One table per row, because the rows are the arrangement: the upper one is
/// what a player reaches for between notes and the lower one is the voice, left
/// to right in the order the signal takes it.
fn render_front(spec: &Spec) -> String {
    let controls: usize = spec.sections.iter().map(|s| s.controls.len()).sum();
    let mut out = format!(
        "{controls} of the {} parameters have a control on the front of the \
         instrument, across {} plates in {} rows.\n",
        spec.parameters.len(),
        spec.sections.len(),
        spec.panel_rows,
    );
    for row in 0..spec.panel_rows {
        let plates: Vec<&crate::spec::Section> = spec
            .sections
            .iter()
            .filter(|section| section.row == row)
            .collect();
        if plates.is_empty() {
            continue;
        }
        let names: Vec<String> = plates
            .iter()
            .map(|section| format!("`{}`", section.name))
            .collect();
        let _ = write!(
            out,
            "\n#### Row {row}\n\n{}, left to right.\n\n\
             | Plate | Printed | Control | Parameter | Offset |\n|---|---|---|---|---|\n",
            names.join(", ")
        );
        for section in plates {
            for control in &section.controls {
                let parameter = spec.parameters.iter().find(|p| p.name == control.parameter);
                let _ = writeln!(
                    out,
                    "| {} | `{}` | {} | {} | {} |",
                    cell(&section.name),
                    cell(&control.legend),
                    control.shape,
                    parameter.map_or_else(
                        || cell(&control.parameter),
                        |parameter| cell(&parameter.name)
                    ),
                    parameter
                        .map_or_else(|| "-".to_owned(), |parameter| parameter.offset.to_string())
                );
            }
        }
        for section in spec.sections.iter().filter(|s| s.row == row) {
            if let Some(note) = &section.note {
                let _ = writeln!(out, "\n- **{}.** {}", cell(&section.name), cell(note));
            }
        }
    }
    out
}

/// Renders the measured grid, so the numbers the drawings are built from are
/// readable next to them./// Renders the measured grid, so the numbers the drawings are built from are
/// readable next to them.
fn render_grid(spec: &Spec) -> String {
    let grid = &spec.grid;
    let rows = [
        ("Columns".to_owned(), grid.columns.to_string()),
        ("Rows".to_owned(), grid.rows.to_string()),
        ("Fill order".to_owned(), cell(&grid.order)),
        (
            "Alignment of a row that is not full".to_owned(),
            cell(&grid.align),
        ),
        (
            "Shape the FX page draws for every slot".to_owned(),
            cell(&grid.shape),
        ),
        (
            "Display measured on".to_owned(),
            format!(
                "{:.0} x {:.0} pixels",
                grid.display_width, grid.display_height
            ),
        ),
        (
            "First control centre".to_owned(),
            format!("{:.1}, {:.1}", grid.first_x, grid.first_y),
        ),
        (
            "Column pitch".to_owned(),
            format!("{:.1}", grid.column_pitch),
        ),
        ("Row pitch".to_owned(), format!("{:.1}", grid.row_pitch)),
        (
            "Control diameter".to_owned(),
            format!("{:.1}", grid.control_diameter),
        ),
    ];

    let mut out = String::from("| Property | Value |\n|---|---|\n");
    for (property, value) in rows {
        let _ = writeln!(out, "| {property} | {value} |");
    }
    let _ = write!(
        out,
        "\nThe drawings below space their rows further apart than {:.1} pixels, because \
         the synthesizer has room for a three-letter label and these have room for the \
         parameter's name. Everything across a row is as measured. The shape at each \
         position is not: the FX page draws every slot as a circle, and the drawings \
         use what the effect's own panel uses instead. Every handle is drawn at the \
         {}.\n",
        grid.row_pitch,
        fx::handle_note(),
    );
    out
}

/// Renders the list of algorithms, grouped as the manual's effects table groups
/// them.
fn render_effect_index(spec: &Spec) -> String {
    let mut out = String::new();
    let mut seen: Vec<&str> = Vec::new();
    for layout in &spec.layouts {
        if !seen.contains(&layout.category.as_str()) {
            seen.push(&layout.category);
        }
    }
    for category in seen {
        let _ = write!(
            out,
            "\n### {category}\n\n| `FX Type` | Effect | Name | Slots | Page | Panel |\n\
             |---|---|---|---|---|---|\n"
        );
        for layout in spec.layouts.iter().filter(|l| l.category == category) {
            let Some(effect) = spec.effects.iter().find(|e| e.r#type == layout.r#type) else {
                continue;
            };
            let rows: Vec<String> = layout
                .rows
                .iter()
                .map(|r| r.slots.len().to_string())
                .collect();
            let _ = writeln!(
                out,
                "| {} | [{}](#{}) | {} | {} | {} | {} |",
                layout.r#type,
                cell(&effect.name),
                fx::stem(effect.r#type, &effect.full_name),
                cell(&effect.full_name),
                effect.parameters.len(),
                match rows.len() {
                    1 => format!("one row of {}", rows[0]),
                    _ => format!("rows of {}", rows.join(" and ")),
                },
                cell(&layout.control),
            );
        }
    }
    out
}

/// Renders one section per algorithm: the drawing of its page, then what each
/// of an engine's twelve raw parameters means when that algorithm is loaded.
fn render_effects(spec: &Spec, drawings: &[fx::Drawing]) -> String {
    let mut out = String::new();
    for effect in &spec.effects {
        let layout = spec.layouts.iter().find(|l| l.r#type == effect.r#type);
        let panel = spec.panels.iter().find(|p| p.r#type == effect.r#type);
        let stem = fx::stem(effect.r#type, &effect.full_name);
        let drawing = drawings.iter().find(|d| d.id == stem);
        let _ = write!(
            out,
            "\n<a id=\"{}\"></a>\n\n### {} ({})\n\n",
            stem,
            cell(&effect.full_name),
            cell(&effect.name),
        );
        if let Some(drawing) = drawing {
            let _ = write!(
                out,
                "<img src=\"diagrams/fx/{}.svg\" alt=\"{} front panel\" width=\"720\">\n\n",
                drawing.id,
                cell(&drawing.title),
            );
        }
        let family = spec
            .families
            .iter()
            .find(|f| f.algorithms.iter().any(|a| a == &effect.name));
        let mark = match (spec.variant_of(&effect.name), family) {
            (Some(variant), Some(family)) => format!(
                ", drawn with the {} mark of the {} family",
                variant.name.to_lowercase(),
                family.name.to_lowercase()
            ),
            (None, Some(family)) => {
                format!(", drawn with the {} mark", family.name.to_lowercase())
            }
            _ => String::new(),
        };
        let characters: Vec<String> = spec
            .characters_of(&effect.name)
            .iter()
            .map(|character| format!("[{}](#{})", character.name, character.name.to_lowercase()))
            .collect();
        let characters = if characters.is_empty() {
            String::new()
        } else {
            format!(" Characters: {}.", characters.join(", "))
        };
        let _ = write!(
            out,
            "`FX Type` {}{}{mark}. {} slots{}.{characters}\n\n\
             | Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | \
             Description |\n|---|---|---|---|---|---|---|---|---|---|\n",
            effect.r#type,
            layout.map_or(String::new(), |l| format!(
                ", {}",
                l.category.to_lowercase()
            )),
            effect.parameters.len(),
            layout.map_or(String::new(), |l| format!(", drawn as {}s", l.control)),
        );
        for parameter in &effect.parameters {
            let slot = panel.and_then(|p| p.slots.iter().find(|s| s.slot == parameter.slot));
            let range = match (&parameter.values, &parameter.min, &parameter.max) {
                (Some(values), _, _) => cell(values),
                (_, Some(min), Some(max)) => {
                    let unit = parameter.unit.as_deref().unwrap_or("");
                    let unit = if unit.is_empty() {
                        String::new()
                    } else {
                        format!(" {unit}")
                    };
                    format!("{min} to {max}{unit}")
                }
                _ => String::new(),
            };
            let range = match &parameter.note {
                Some(note) => format!("{range}. {}", cell(note)),
                None => range,
            };
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} | {} | {} | {range} | {} | {} | {} |",
                parameter.slot,
                cell(&parameter.r#ref),
                cell(&parameter.name),
                slot.map_or(String::new(), |s| cell(&s.title)),
                slot.map_or("", |s| s.kind.as_str()),
                slot.and_then(|s| s.group.as_deref()).unwrap_or(""),
                if parameter.mod_dest { "yes" } else { "" },
                slot.map_or(String::new(), |s| glyph_link(Some(&s.glyph))
                    .replace("effects.md#", "#")),
                cell(parameter.description.as_deref().unwrap_or(""))
            );
        }
    }
    out
}

/// Renders the rows of the manual's effect tables that had to be corrected.
fn render_effect_corrections(spec: &Spec) -> String {
    let mut out = String::from("| Effect | Slot | Parameter | Correction |\n|---|---|---|---|\n");
    let mut any = false;
    for effect in &spec.effects {
        for parameter in &effect.parameters {
            let Some(correction) = parameter.correction.as_deref() else {
                continue;
            };
            any = true;
            let _ = writeln!(
                out,
                "| [{}](#{}) | {} | {} | {} |",
                cell(&effect.name),
                fx::stem(effect.r#type, &effect.full_name),
                parameter.slot,
                cell(&parameter.name),
                cell(correction),
            );
        }
    }
    if any {
        out
    } else {
        "None recorded.".to_owned()
    }
}

/// Renders the ten FX topologies and the three FX modes.
///
/// Each slot's cell says what reaches it, which together with the output column
/// is the edge list of the diagram the manual prints.
fn render_routing(spec: &Spec) -> String {
    let slots = |routing: &crate::spec::Routing, slot: u8| -> String {
        routing
            .slots
            .iter()
            .find(|s| s.slot == slot)
            .map_or_else(String::new, |s| {
                s.from
                    .iter()
                    .map(|&f| {
                        if f == 0 {
                            "input".to_owned()
                        } else {
                            f.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            })
    };

    let mut out = String::from(
        "| Value | | Routing | Slot 1 from | Slot 2 from | Slot 3 from | \
         Slot 4 from | To output |\n|---|---|---|---|---|---|---|---|\n",
    );
    for routing in &spec.routings {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            routing.value,
            cell(&routing.label),
            cell(&routing.name),
            slots(routing, 1),
            slots(routing, 2),
            slots(routing, 3),
            slots(routing, 4),
            routing
                .output
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" + "),
        );
    }

    let feedback: Vec<&crate::spec::Routing> =
        spec.routings.iter().filter(|r| r.feedback).collect();
    if !feedback.is_empty() {
        out.push_str(
            "\nA feedback routing feeds a slot, directly or through others, from \
             a slot downstream of it. The manual notes a 30 Hz high pass filter \
             in the feedback path of these.\n\n",
        );
        for routing in feedback {
            let _ = writeln!(
                out,
                "- **{}**, {}. {}",
                cell(&routing.label),
                cell(&routing.name),
                cell(routing.note.as_deref().unwrap_or("")).trim()
            );
        }
    }

    out.push_str(
        "\nThe `FX Mode` parameter at offset 222 decides which paths the voices \
         take. The analog path runs from the voices to the output stage \
         untouched; the digital path runs them through the FX block. Bypass is a \
         true bypass, with the DSP out of circuit rather than muted.\n\n\
         | Value | Mode | Analog path | Digital path |\n|---|---|---|---|\n",
    );
    let mark = |on: bool| if on { "on" } else { "off" };
    for mode in &spec.fx_modes {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            mode.value,
            cell(&mode.name),
            mark(mode.analog_path),
            mark(mode.digital_path)
        );
    }
    out
}

/// Renders the raw-to-displayed readings, grouped by parameter.
fn render_measurements(spec: &Spec) -> String {
    let mut out = String::from(
        "| Offset | Parameter | Raw | Displayed | Fits | Note |\n|---|---|---|---|---|---|\n",
    );
    for measurement in &spec.measurements {
        let name = spec
            .parameters
            .iter()
            .find(|p| p.offset == measurement.offset)
            .map_or("", |p| p.name.as_str());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            measurement.offset,
            cell(name),
            measurement.raw,
            cell(&measurement.shown),
            cell(&measurement.fit),
            cell(measurement.note.as_deref().unwrap_or("")),
        );
    }

    let count = |fit: &str| spec.measurements.iter().filter(|m| m.fit == fit).count();
    let parameters = {
        let mut offsets: Vec<u16> = spec.measurements.iter().map(|m| m.offset).collect();
        offsets.sort_unstable();
        offsets.dedup();
        offsets.len()
    };
    let _ = write!(
        out,
        "\n{} readings across {parameters} parameters: {} match a linear \
         interpolation between the parameter's stated ends, {} an exponential \
         one, {} sit on the two-segment fader response section 8.3.1 draws, {} \
         are at an end of a range rather than inside it, {} are of a parameter \
         the manual states no range for, and {} match nothing simple.\n",
        spec.measurements.len(),
        count("linear"),
        count("exponential"),
        count("piecewise"),
        count("endpoint"),
        count("untested"),
        count("none"),
    );
    out
}

fn render_corrections(spec: &Spec) -> String {
    // Several corrections apply identically to every slot of a repeated group,
    // such as the eight modulation matrix busses, so rows are grouped by text.
    let mut groups: Vec<(&str, Vec<&crate::spec::Parameter>)> = Vec::new();
    for parameter in &spec.parameters {
        let Some(correction) = parameter.correction.as_deref() else {
            continue;
        };
        match groups.iter_mut().find(|(text, _)| *text == correction) {
            Some((_, members)) => members.push(parameter),
            None => groups.push((correction, vec![parameter])),
        }
    }
    if groups.is_empty() {
        return "None recorded.".to_owned();
    }

    let mut out = String::from("| Offsets | Parameter | Correction |\n|---|---|---|\n");
    for (correction, members) in groups {
        let offsets = if members.len() <= 8 {
            members
                .iter()
                .map(|p| p.offset.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            format!(
                "{} values from {} to {}",
                members.len(),
                members[0].offset,
                members[members.len() - 1].offset
            )
        };
        let name = match members.as_slice() {
            [only] => cell(&only.name),
            _ => format!("{} and {} more", cell(&members[0].name), members.len() - 1),
        };
        let _ = writeln!(out, "| {offsets} | {name} | {} |", cell(correction));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails when a generated Markdown table has a row whose column count differs
    /// from its header.
    ///
    /// A row with one more column than its header renders as a broken table
    /// rather than an error.
    #[test]
    fn generated_tables_have_consistent_columns() {
        let root = crate::root();
        let columns = |line: &str| line.trim().trim_matches('|').split('|').count();
        for path in [DOC_PATH, EFFECTS_PATH] {
            let document = std::fs::read_to_string(root.join(path)).expect("document is readable");
            let mut expected: Option<usize> = None;
            for (number, raw) in document.lines().enumerate() {
                let line = raw.trim();
                // An indented row is a code block to Markdown, not a table. The
                // continuations in a Rust string literal make that easy to do by
                // accident and impossible to see in the source.
                assert!(
                    !line.starts_with('|') || raw.starts_with('|'),
                    "{path} line {}: table row is indented",
                    number + 1
                );
                if !line.starts_with('|') {
                    expected = None;
                    continue;
                }
                // The dashed rule under a header sets the width for the rows that follow.
                if line.chars().all(|c| "|-: ".contains(c)) {
                    expected = Some(columns(line));
                    continue;
                }
                if let Some(want) = expected {
                    assert_eq!(
                        columns(line),
                        want,
                        "{path} line {}: table row has {} columns, header has {want}",
                        number + 1,
                        columns(line)
                    );
                }
            }
        }
    }

    /// Fails when `docs/midi-spec.md` does not match the spec files.
    ///
    /// This is the safety net that keeps generated documentation honest: editing
    /// a spec file without regenerating turns `cargo test` red.
    #[test]
    fn generated_documentation_is_current() {
        match generate(crate::spec::shared(), &crate::root(), true) {
            Ok(stale) if stale.is_empty() => {}
            Ok(stale) => panic!(
                "out of date with spec/: {}. Run `cargo xtask docs` and commit the result.",
                stale.join(", ")
            ),
            Err(message) => panic!("{message}"),
        }
    }
}
