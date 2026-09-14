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
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::{Kind, ParamId};
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
