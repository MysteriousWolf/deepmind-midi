//! Programs, unpacked from arbitrary payloads and edited from them.
//!
//! This is the layer a host reads parameters out of, and the one where a length
//! that does not match its version turns into an index. Every accessor is
//! walked, because an out-of-range offset is the failure mode worth finding.
//!
//! Two ways in. The packed form is what arrives on the wire, but it is long and
//! most inputs are refused before any accessor runs. The raw form is what a
//! decoded dump becomes, and it takes a prefix of the input, so short inputs
//! reach every accessor too; what is left over drives the setters.

#![no_main]

use libfuzzer_sys::fuzz_target;

use deepmind_midi::ids::ProtocolVersion;
use deepmind_midi::param::ParamId;
use deepmind_midi::program::Program;
use deepmind_midi::sysex::packed;

/// Reads every parameter, which is what a host does with a dump.
fn walk(program: &Program) {
    assert_eq!(
        program.version().program_data_len(),
        program.as_bytes().len()
    );
    for parameter in ParamId::ALL.iter().copied() {
        let value = program.get(parameter);
        let _ = parameter.label(u16::from(value));
    }
    let _ = program.name();
    let _ = program.transpose();
    let _ = program.validate();
    let _ = program.invalid().count();
}

/// Edits `program` as `script` says: pairs of a parameter index and a value.
///
/// A clamped write has to land on a value the parameter accepts, and a checked
/// write has to agree with the parameter's own idea of what it accepts.
fn edit(program: &mut Program, script: &[u8]) {
    for pair in script.chunks_exact(2) {
        let [index, value] = pair else {
            continue;
        };
        let parameter = ParamId::ALL[usize::from(*index) % ParamId::ALL.len()];

        let clamped = program.set_clamped(parameter, *value);
        assert!(
            parameter.accepts(u16::from(clamped)),
            "clamping left {parameter} holding {clamped}"
        );
        assert_eq!(program.get(parameter), clamped);

        let accepted = program.set(parameter, *value).is_ok();
        assert_eq!(
            accepted,
            parameter.accepts(u16::from(*value)),
            "set and accepts disagree on {parameter} = {value}"
        );
        if accepted {
            assert_eq!(program.get(parameter), *value);
        }
    }
}

fuzz_target!(|data: &[u8]| {
    for version in [ProtocolVersion::V6, ProtocolVersion::V7] {
        // The raw form: exactly the version's length, then the edit script.
        if let Some((raw, script)) = data.split_at_checked(version.program_data_len()) {
            let mut program = Program::from_bytes(version, raw).expect("the documented length");
            assert_eq!(program.as_bytes(), raw);
            walk(&program);
            edit(&mut program, script);
            walk(&program);
        }

        // The packed form, as a port delivers it.
        let Ok(program) = Program::from_packed(version, data) else {
            continue;
        };
        walk(&program);

        // And back out the way it came in.
        let repacked = packed::pack(program.as_bytes());
        let back = Program::from_packed(version, &repacked)
            .expect("what this library just packed unpacks");
        assert_eq!(back.as_bytes(), program.as_bytes());
    }
});
