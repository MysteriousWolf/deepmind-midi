//! Programs, unpacked from arbitrary payloads.
//!
//! This is the layer a host reads parameters out of, and the one where a length
//! that does not match its version turns into an index. Every accessor is
//! walked, because an out-of-range offset is the failure mode worth finding.

#![no_main]

use libfuzzer_sys::fuzz_target;

use deepmind_midi::ids::ProtocolVersion;
use deepmind_midi::param::ParamId;
use deepmind_midi::program::Program;
use deepmind_midi::sysex::packed;

fuzz_target!(|data: &[u8]| {
    for version in [ProtocolVersion::V6, ProtocolVersion::V7] {
        let Ok(program) = Program::from_packed(version, data) else {
            continue;
        };

        assert_eq!(
            program.version().program_data_len(),
            program.as_bytes().len()
        );

        // Reading every parameter, which is what a host does with a dump.
        for parameter in ParamId::ALL.iter().copied() {
            let value = program.get(parameter);
            let _ = parameter.label(u16::from(value));
        }
        let _ = program.name();
        let _ = program.transpose();
        let _ = program.validate();
        let _ = program.invalid().count();

        // And back out the way it came in.
        let repacked = packed::pack(program.as_bytes());
        let back = Program::from_packed(version, &repacked)
            .expect("what this library just packed unpacks");
        assert_eq!(back.as_bytes(), program.as_bytes());
    }
});
