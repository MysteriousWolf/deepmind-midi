//! The `serde` feature: what the public data types look like serialized, and
//! that what comes back is what went in.
//!
//! JSON is the format used here because it is the one a host is likeliest to
//! write a program to, and because a text form makes the shape visible.

#![expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]

use deepmind_midi::device::Known;
use deepmind_midi::effect::{Algorithm, Colour, Engine, Routing, Source};
use deepmind_midi::front::{self, PanelShape};
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::{DEFAULT_FIRMWARE, Kind, ParamId, Shape};
use deepmind_midi::program::{LfoShape, Program, ProgramName};
use deepmind_midi::sysex::Interface;

#[test]
fn a_program_name_is_a_string() {
    let name = ProgramName::new("Bass Sweep").expect("a name the display can show");
    let json = serde_json::to_string(&name).expect("serializes");
    assert_eq!(json, "\"Bass Sweep\"");
    let back: ProgramName = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, name);
}

#[test]
fn a_program_name_keeps_its_rules_on_the_way_in() {
    assert!(serde_json::from_str::<ProgramName>("\"Seventeen Charact\"").is_err());
    assert!(serde_json::from_str::<ProgramName>("\"Caff\\u00e8\"").is_err());
    assert!(serde_json::from_str::<ProgramName>("[66, 97, 115, 115]").is_err());
}

#[test]
fn the_addressing_types_round_trip() {
    let slot = Slot::new(Bank::H, ProgramNumber::LAST);
    let json = serde_json::to_string(&slot).expect("serializes");
    assert_eq!(
        serde_json::from_str::<Slot>(&json).expect("deserializes"),
        slot
    );

    for id in [DeviceId::Unit(3), DeviceId::Broadcast] {
        let json = serde_json::to_string(&id).expect("serializes");
        assert_eq!(
            serde_json::from_str::<DeviceId>(&json).expect("deserializes"),
            id
        );
    }

    let json = serde_json::to_string(&ProtocolVersion::V7).expect("serializes");
    assert_eq!(
        serde_json::from_str::<ProtocolVersion>(&json).expect("deserializes"),
        ProtocolVersion::V7
    );

    let json = serde_json::to_string(&Interface::Usb).expect("serializes");
    assert_eq!(
        serde_json::from_str::<Interface>(&json).expect("deserializes"),
        Interface::Usb
    );
}

#[test]
fn a_tracked_program_carries_its_claim() {
    let mut program = Program::new(ProtocolVersion::V6);
    program.set_lfo1_shape(LfoShape::Triangle);
    let known = Known::Confirmed {
        value: program.clone(),
        at: 42,
    };
    let json = serde_json::to_string(&known).expect("serializes");
    let back: Known<Program> = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, known);
    assert_eq!(
        back.value().and_then(Program::lfo1_shape),
        Some(LfoShape::Triangle)
    );
}

#[test]
fn a_parameter_and_its_kind_serialize_by_name() {
    let json = serde_json::to_string(&ParamId::Lfo1Rate).expect("serializes");
    assert_eq!(json, "\"Lfo1Rate\"");
    assert_eq!(
        serde_json::from_str::<ParamId>(&json).expect("deserializes"),
        ParamId::Lfo1Rate
    );
    let json = serde_json::to_string(&Kind::Switch).expect("serializes");
    assert_eq!(json, "\"Switch\"");
}

/// The two new readings of a raw value: the point a bipolar parameter is read
/// about, and how the panel's own control shapes name themselves.
#[test]
fn what_a_control_is_serializes_by_name() {
    let json = serde_json::to_string(&Shape::Unipolar).expect("serializes");
    assert_eq!(json, "\"Unipolar\"");
    assert_eq!(
        serde_json::from_str::<Shape>(&json).expect("deserializes"),
        Shape::Unipolar
    );

    let bipolar = ParamId::Mod1Depth.shape();
    let json = serde_json::to_string(&bipolar).expect("serializes");
    assert_eq!(json, "{\"Bipolar\":{\"centre\":128}}");
    assert_eq!(
        serde_json::from_str::<Shape>(&json).expect("deserializes"),
        bipolar
    );

    let json = serde_json::to_string(&PanelShape::Lamps).expect("serializes");
    assert_eq!(json, "\"Lamps\"");
    assert_eq!(
        serde_json::from_str::<PanelShape>(&json).expect("deserializes"),
        PanelShape::Lamps
    );
}

/// A panel serializes as what it is made of, not as the index it is reached by:
/// the algorithm's own place in the table is the library's business.
#[test]
fn an_effect_panel_serializes_as_a_measurement() {
    let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
    let json = serde_json::to_string(room.panel()).expect("serializes");
    assert!(json.contains("\"control\":\"Fader\""), "{json}");
    assert!(json.contains("\"cap\":{\"red\":152,"), "{json}");

    let algorithm = serde_json::to_string(room).expect("serializes");
    assert!(
        algorithm.starts_with("{\"name\":\"RoomRev\""),
        "{algorithm}"
    );
    assert!(!algorithm.contains("index"), "{algorithm}");

    let colour = serde_json::to_string(&room.panel().cap()).expect("serializes");
    assert_eq!(
        serde_json::from_str::<Colour>(&colour).expect("deserializes"),
        room.panel().cap()
    );
}

/// What reaches an engine names the block's input rather than numbering it,
/// which is the whole reason it is an enum.
#[test]
fn a_routing_serializes_as_a_graph() {
    let serial = Routing::for_value(0).expect("ten topologies");
    let json = serde_json::to_string(serial).expect("serializes");
    assert!(json.contains("\"label\":\"M-1\""), "{json}");
    assert!(json.contains("[\"Input\"]"), "{json}");
    assert!(json.contains("{\"Engine\":\"One\"}"), "{json}");

    let source = serde_json::to_string(&Source::Engine(Engine::Four)).expect("serializes");
    assert_eq!(
        serde_json::from_str::<Source>(&source).expect("deserializes"),
        Source::Engine(Engine::Four)
    );
}

/// A plate carries its legends and the parameters they move, which is what a
/// host sending its panel somewhere needs.
#[test]
fn the_front_panel_serializes_with_its_legends() {
    let vcf = front::sections()
        .iter()
        .find(|section| section.name() == "VCF")
        .expect("a filter");
    let json = serde_json::to_string(vcf).expect("serializes");
    assert!(json.contains("\"name\":\"VCF\""), "{json}");
    assert!(json.contains("\"group\":\"Vcf\""), "{json}");
    assert!(json.contains("\"legend\":\"KYBD\""), "{json}");
    assert!(
        json.contains("\"parameter\":\"VcfKeyboardTracking\""),
        "{json}"
    );
}
